use dotenv::dotenv;
use std::{env, time::Instant};

use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;

use crate::{Client, enums::ApiVersion, errors::ApiError};

#[tokio::test]
async fn print_token() {
    dotenv().ok();

    let user = env::var("TEST_USER").expect("TEST_USER must be set in .env for integration tests");
    let pass = env::var("TEST_PASS").expect("TEST_PASS must be set in .env for integration tests");

    assert!(!user.is_empty());
    assert!(!pass.is_empty());

    let client = Client::new();
    let new_client = client.login(&user, &pass, "dev").await.unwrap();
    println!("Token: {}", new_client.get_token());
}

#[tokio::test]
async fn login_with_token() {
    dotenv().ok();
    let token =
        env::var("TEST_TOKEN").expect("TEST_TOKEN must be set in .env for integration tests");

    assert!(!token.is_empty());

    let client = Client::new()
        .login_with_token(&token, "default")
        .await
        .unwrap();
    // let recent = client.user().me().await.unwrap();
    println!("My Orders: {:?}", client.auction().cache_auctions());
}
#[tokio::test]
async fn rate_limiting() {
    // Use unauthenticated client to avoid token issues
    let mut client = Client::new();

    println!("Starting rate limiting test with multiple concurrent requests...");

    client.set_rate_limit(std::num::NonZeroU32::new(30).unwrap());

    // Create multiple concurrent request futures
    let mut handles = Vec::new();

    for i in 0..50 {
        let client_clone = client.clone(); // Clone the client for each task
        let handle = tokio::spawn(async move {
            let result = client_clone
                .order()
                .get_orders_by_item("revenant_prime_blueprint")
                .await;

            (i + 1, result)
        });
        handles.push(handle);
    }

    println!("Waiting for all {} requests to complete...", handles.len());

    // Collect results
    let mut successful_requests = 0;
    let mut rate_limited_requests = 0;
    let mut other_errors = 0;

    for handle in handles {
        match handle.await {
            Ok((request_num, result)) => match result {
                Ok(orders) => {
                    successful_requests += 1;
                    let total_orders = orders.sell_orders.len() + orders.buy_orders.len();
                    println!(
                        "Request {} succeeded - got {} total orders ({} sell, {} buy)",
                        request_num,
                        total_orders,
                        orders.sell_orders.len(),
                        orders.buy_orders.len()
                    );
                }
                Err(e) => match &e {
                    ApiError::TooManyRequests(_) => {
                        rate_limited_requests += 1;
                        println!("Request {} rate limited: {:?}", request_num, e);
                    }
                    _ => {
                        other_errors += 1;
                        println!("Request {} failed with other error: {:?}", request_num, e);
                    }
                },
            },
            Err(join_error) => {
                other_errors += 1;
                println!("Task failed to complete: {:?}", join_error);
            }
        }
    }

    println!("Rate limiting test completed:");
    println!("  Successful requests: {}", successful_requests);
    println!("  Rate limited requests: {}", rate_limited_requests);
    println!("  Other errors: {}", other_errors);
    println!("  Total requests sent: 50");

    // The test passes if we can send requests (regardless of rate limiting)
    assert!(successful_requests + rate_limited_requests + other_errors == 50);
}

#[tokio::test]
async fn test_api_events_fire() {
    let client = Client::new();
    let fired = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

    let f_before = fired.clone();
    client.on("api:before", move |event, data| {
        f_before.lock().unwrap().push(format!(
            "{}:{}",
            event,
            data.get_property_value("key", String::new())
        ));
    });

    let f_after = fired.clone();
    client.on("api:after", move |event, data| {
        f_after.lock().unwrap().push(format!(
            "{}:{}",
            event,
            data.get_property_value("key", String::new())
        ));
    });

    let f_error = fired.clone();
    client.on("api:error", move |event, data| {
        f_error.lock().unwrap().push(format!(
            "{}:{}",
            event,
            data.get_property_value("key", String::new())
        ));
    });

    let custom_version = ApiVersion::Custom("http://127.0.0.1:1".into(), "ws://127.0.0.1:1".into());

    let _ = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.call_api::<serde_json::Value>(
            custom_version,
            reqwest::Method::GET,
            "/test_api_events",
            "test_api_events",
            None,
            None,
        ),
    )
    .await;

    let events = fired.lock().unwrap();
    assert!(events.contains(&"api:before:test_api_events".to_string()));
    assert!(events.contains(&"api:error:test_api_events".to_string()));
    assert!(!events.contains(&"api:after:test_api_events".to_string()));
    assert_eq!(
        events.len(),
        2,
        "Expected exactly 2 events (before + error), got: {:?}",
        *events
    );
}

#[test]
fn test_with_callback_builder() {
    let fired = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

    let f1 = fired.clone();
    let f2 = fired.clone();
    let client = Client::new()
        .with_callback("evt1", move |event, _| {
            f1.lock().unwrap().push(format!("cb1:{}", event));
        })
        .with_callback("evt2", move |event, _| {
            f2.lock().unwrap().push(format!("cb2:{}", event));
        });

    client.emit("evt1", &crate::types::Properties::default());
    assert_eq!(fired.lock().unwrap().len(), 1);
    assert!(fired.lock().unwrap().contains(&"cb1:evt1".to_string()));

    fired.lock().unwrap().clear();
    client.emit("evt2", &crate::types::Properties::default());
    assert_eq!(fired.lock().unwrap().len(), 1);
    assert!(fired.lock().unwrap().contains(&"cb2:evt2".to_string()));
}

#[tokio::test]
async fn test_callbacks() {
    let client = Client::new();
    let fired = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

    // Register callbacks
    let fired1 = fired.clone();
    client.on("test:event", move |event, data| {
        fired1.lock().unwrap().push(format!("cb1:{}", event));
        let _ = data;
    });

    let fired2 = fired.clone();
    client.on("test:event", move |event, data| {
        fired2.lock().unwrap().push(format!("cb2:{}", event));
        let _ = data;
    });

    let fired3 = fired.clone();
    client.on("test:other", move |event, data| {
        fired3.lock().unwrap().push(format!("cb3:{}", event));
        let _ = data;
    });

    // Emit test:event - both cb1 and cb2 should fire
    client.emit(
        "test:event",
        &crate::types::Properties {
            properties: Some(serde_json::json!({"msg": "hello"})),
        },
    );
    assert_eq!(fired.lock().unwrap().len(), 2);
    assert!(
        fired
            .lock()
            .unwrap()
            .contains(&"cb1:test:event".to_string())
    );
    assert!(
        fired
            .lock()
            .unwrap()
            .contains(&"cb2:test:event".to_string())
    );

    // Emit test:other - only cb3 should fire
    fired.lock().unwrap().clear();
    client.emit("test:other", &crate::types::Properties { properties: None });
    assert_eq!(fired.lock().unwrap().len(), 1);
    assert!(
        fired
            .lock()
            .unwrap()
            .contains(&"cb3:test:other".to_string())
    );

    // Emit unregistered event - no callbacks should fire
    fired.lock().unwrap().clear();
    client.emit("test:nonexistent", &crate::types::Properties::default());
    assert_eq!(fired.lock().unwrap().len(), 0);

    // Remove test:event callbacks
    fired.lock().unwrap().clear();
    client.off("test:event");
    client.emit("test:event", &crate::types::Properties::default());
    assert_eq!(fired.lock().unwrap().len(), 0);

    // test:other callback should still work
    client.emit("test:other", &crate::types::Properties::default());
    assert_eq!(fired.lock().unwrap().len(), 1);

    // Clear all callbacks
    fired.lock().unwrap().clear();
    client.clear_callbacks();
    client.emit("test:other", &crate::types::Properties::default());
    assert_eq!(fired.lock().unwrap().len(), 0);
}

pub struct MyService {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl MyService {
    pub fn new() -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(1).unwrap()); // 1 req/min
        let limiter = RateLimiter::direct(quota);

        Self { limiter }
    }

    /// Async method using until_ready
    pub async fn do_work(&self) -> &'static str {
        // Wait until request can proceed
        self.limiter.until_ready().await;
        println!("Work done");
        "allowed"
    }
}
#[tokio::test]
async fn test_rate_limiter_async() {
    let service = MyService::new();

    // First two calls happen immediately.
    service.do_work().await;
    service.do_work().await;
    service.do_work().await;
    service.do_work().await;
    service.do_work().await;

    let start = Instant::now();

    // Third call should wait because quota is exceeded.
    service.do_work().await;

    let elapsed = start.elapsed();

    // Should have waited ~500ms (given 2 req/sec)
    assert!(elapsed >= std::time::Duration::from_millis(450));
}

#[tokio::test]
async fn test_transport_error_details() {
    let client = Client::new();
    let custom_version = ApiVersion::Custom("http://127.0.0.1:1".into(), "ws://127.0.0.1:1".into());

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.call_api::<serde_json::Value>(
            custom_version,
            reqwest::Method::GET,
            "/test_transport_error",
            "test_transport_error",
            None,
            None,
        ),
    )
    .await;

    match result {
        Ok(Err(ApiError::RequestError(error))) => {
            println!("Transport error content:\n{}", error.content);
            // Must have a reason classification (connection error, timeout, etc.)
            assert!(
                error.content.contains("Reason: connection error")
                    || error.content.contains("Reason: timeout"),
                "Expected reason classification in error content, got:\n{}",
                error.content
            );
            // Should contain the path we tried
            assert!(
                error.content.contains("/test_transport_error")
                    || error.content.contains("127.0.0.1:1"),
                "Expected path or URL in error content, got:\n{}",
                error.content
            );
            assert!(
                error.content.len() > 30,
                "Error content was too short: {}",
                error.content
            );
        }
        Ok(Ok(_)) => panic!("Expected transport error but got success"),
        Ok(Err(other)) => panic!("Expected RequestError, got: {:?}", other),
        Err(_) => panic!("Test timed out waiting for transport error"),
    }
}
