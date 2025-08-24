use crate::Authenticated;
use crate::client::Client;
use crate::enums::ApiVersion;
use crate::{errors::*, types::websocket::*};
use dotenv::dotenv;
use serde_json::json;
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;

async fn setup_client() -> Result<Client<Authenticated>, ApiError> {
    dotenv().ok();

    let user = env::var("TEST_USER").expect("TEST_USER must be set in .env for integration tests");
    let pass = env::var("TEST_PASS").expect("TEST_PASS must be set in .env for integration tests");

    assert!(!user.is_empty());
    assert!(!pass.is_empty());

    let _client = Client::new();
    _client.login(&user, &pass, "dev").await
}

#[tokio::test]
async fn websocket_v1() {
    use tokio::sync::Notify;

    let received_messages: Arc<Mutex<Vec<WsMessage>>> = Arc::new(Mutex::new(Vec::new()));
    let notify = Arc::new(Notify::new());

    let received_messages_clone = Arc::clone(&received_messages);
    let notify_clone = Arc::clone(&notify);

    let client = setup_client().await.unwrap();

    let ws_client = client
        .create_websocket(ApiVersion::V1)
        .set_log_unhandled(true)
        .register_callback("USER/SET_STATUS", move |msg, _, _| {
            let mut vec = received_messages_clone.lock().unwrap();
            vec.push(msg.clone());
            notify_clone.notify_one(); // signal arrival
            Ok(())
        })
        .unwrap()
        .build()
        .await
        .unwrap();

    match ws_client.send_request("@WS/USER/SET_STATUS", json!("invisible")) {
        Ok(_) => println!("WS client sent status invisible"),
        Err(e) => panic!("{:?}", e),
    }

    // Wait for a message or timeout
    let result = tokio::time::timeout(Duration::from_secs(5), notify.notified()).await;

    assert!(
        result.is_ok() && !received_messages.lock().unwrap().is_empty(),
        "Expected at least one message but got none"
    );
}

#[tokio::test]
async fn websocket_v2() {
    use tokio::sync::Notify;

    let received_messages: Arc<Mutex<Vec<WsMessage>>> = Arc::new(Mutex::new(Vec::new()));
    let notify = Arc::new(Notify::new());

    let received_messages_clone = Arc::clone(&received_messages);
    let notify_clone = Arc::clone(&notify);

    let client = setup_client().await.unwrap();

    let ws_client = client
        .create_websocket(ApiVersion::V2)
        .register_callback("cmd/status/set:ok", move |msg, _, _| {
            let mut vec = received_messages_clone.lock().unwrap();
            vec.push(msg.clone());
            notify_clone.notify_one(); // signal arrival
            Ok(())
        })
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

    // Wait for a message or timeout
    let result = tokio::time::timeout(Duration::from_secs(5), notify.notified()).await;

    assert!(
        result.is_ok() && !received_messages.lock().unwrap().is_empty(),
        "Expected at least one message but got none"
    );
}

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
        .create_websocket(ApiVersion::V1)
        .register_callback("internal/connected", move |msg, _, _| {
            println!("WebSocket connected event: {:?}", msg);
            Ok(())
        })
        .unwrap()
        .register_callback("MESSAGE/ONLINE_COUNT", move |msg, _, _| {
            println!("Received online count message: {:?}", msg);
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

#[tokio::test]
async fn websocket_disconnect() {
    use tokio::sync::Notify;

    let received_messages: Arc<Mutex<Vec<WsMessage>>> = Arc::new(Mutex::new(Vec::new()));
    let notify = Arc::new(Notify::new());

    let received_messages_clone = Arc::clone(&received_messages);
    let notify_clone = Arc::clone(&notify);

    let client = setup_client().await.unwrap();

    let ws_client = client
        .create_websocket(ApiVersion::V1)
        .register_callback("internal/disconnected", move |msg, _, _| {
            println!("WebSocket disconnected event: {:?}", msg);
            let mut vec = received_messages_clone.lock().unwrap();
            vec.push(msg.clone());
            notify_clone.notify_one(); // signal arrival
            Ok(())
        })
        .unwrap()
        .build()
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(50)).await;
    match ws_client.disconnect() {
        Ok(_) => println!("WS client disconnected"),
        Err(e) => panic!("{:?}", e),
    }

    tokio::time::sleep(Duration::from_secs(10)).await;

    // Wait for a message or timeout
    let result = tokio::time::timeout(Duration::from_secs(5), notify.notified()).await;

    println!("Disconnect result: {:?}", result);
    // Wait for 15 seconds to ensure the disconnect message is processed
    tokio::time::sleep(Duration::from_secs(10)).await;

    assert!(
        result.is_ok() && !received_messages.lock().unwrap().is_empty(),
        "Expected at least one message but got none"
    );
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
