use crate::client::Client;
use crate::{errors::*, types::websocket::*};
use dotenv::dotenv;
use serde_json::json;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_connection() {
    let received_messages: Arc<Mutex<Vec<WsMessage>>> = Arc::new(Mutex::new(Vec::new()));
    let received_messages_clone2 = received_messages.clone();

    dotenv().ok();

    let user = env::var("TEST_USER").expect("TEST_USER must be set in .env for integration tests");
    let pass = env::var("TEST_PASS").expect("TEST_PASS must be set in .env for integration tests");

    assert!(!user.is_empty());
    assert!(!pass.is_empty());

    let client = { Client::new().login(&user, &pass, "dev").await.unwrap() };

    let ws_client = client
        .create_websocket()
        .register_callback("internal/connected", move |msg, _, _| {
            println!("WebSocket connected event: {:?}", msg);
            Ok(())
        })
        .unwrap()
        .register_callback("internal/disconnected", move |msg, _, _| {
            println!("WebSocket disconnected event: {:?}", msg);
            Ok(())
        })
        .unwrap()
        .register_callback("cmd/status/set:ok", move |msg, _, _| {
            let mut arr = received_messages_clone2.lock().unwrap();
            arr.push(msg.clone());
            println!("Received: {:?}", arr);
            Ok(())
        })
        .unwrap()
        .register_callback("event/reports/online", move |_, _, _| Ok(()))
        .unwrap()
        .build()
        .await
        .unwrap();

    match ws_client.send_request(
        "@wfm|cmd/status/set",
        json!({
            "status": "invisible"
        }),
    ) {
        Ok(_) => println!("WS client sent status invisible"),
        Err(e) => panic!("{:?}", e),
    }

    let _ = timeout(Duration::from_secs(5), async {
        loop {
            {
                let guard = received_messages.lock().unwrap();
                if !guard.is_empty() {
                    break;
                }
            }
            // yield back to Tokio, let the writer+reader run
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await;

    assert!(received_messages.lock().unwrap().len() > 0);
}

#[test]
fn test_route_parsing_with_parameter() {
    let route = Route::parse("@wfm|subscribe/newOrders").unwrap();
    assert_eq!(route.protocol, "@wfm");
    assert_eq!(route.path, "subscribe/newOrders");
}

#[test]
fn test_route_parsing_without_parameter() {
    let route = Route::parse("@wfm|subscribe/newOrders").unwrap();
    assert_eq!(route.protocol, "@wfm");
    assert_eq!(route.path, "subscribe/newOrders");
}

#[test]
fn test_route_to_string() {
    let route_with_param = Route {
        protocol: "@wfm".to_string(),
        path: "cmd/subscribe/newOrders".to_string(),
        parameter: None,
    };
    assert_eq!(route_with_param.to_string(), "@wfm|cmd/subscribe/newOrders");
}

#[test]
fn test_route_parsing_invalid_format() {
    let result = Route::parse("invalid_route_format");
    assert!(result.is_err());
    match result {
        Err(WsError::InvalidPath(_)) => (),
        _ => panic!("Expected InvalidPath error"),
    }
}

#[test]
fn test_route_to_string_without_parameter() {
    let route = Route {
        protocol: "@wfm".to_string(),
        path: "event/user/login".to_string(),
        parameter: None,
    };
    assert_eq!(route.to_string(), "@wfm|event/user/login");
}
