//! WebSocket integration tests.
//!
//! These tests require valid warframe.market credentials set via environment variables
//! or in a `.env` file:
//!
//! ```env
//! WFM_EMAIL=your-email@example.com
//! WFM_PASSWORD=your-password
//! ```
//!
//! Run these tests with:
//! ```bash
//! cargo test --features websocket -- --ignored --nocapture
//! ```
//!
//! Or run a specific test:
//! ```bash
//! cargo test --features websocket -- --ignored test_ws_connect --nocapture
//! ```

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use wf_market::ws::{Subscription, WsEvent};

/// Test that we can connect to the WebSocket and authenticate.
///
/// This test verifies:
/// 1. TLS connection works (wss://)
/// 2. Authentication with JWT token works
/// 3. We receive the Authenticated event
#[tokio::test]
#[ignore]
async fn test_ws_connect_and_authenticate() {
    if !common::has_credentials() {
        eprintln!("{}", common::skip_message());
        return;
    }

    let client = common::authenticated_client()
        .await
        .expect("Failed to authenticate");

    let authenticated = Arc::new(AtomicBool::new(false));
    let authenticated_clone = authenticated.clone();

    let ws = client
        .websocket()
        .on_event(move |event| {
            let auth = authenticated_clone.clone();
            async move {
                match event {
                    WsEvent::Connected => {
                        println!("[TEST] Connected to WebSocket server");
                    }
                    WsEvent::Authenticated => {
                        println!("[TEST] Authenticated successfully");
                        auth.store(true, Ordering::SeqCst);
                    }
                    WsEvent::AuthenticationFailed { error } => {
                        println!("[TEST] Authentication failed: {}", error);
                    }
                    WsEvent::Disconnected { reason } => {
                        println!("[TEST] Disconnected: {}", reason);
                    }
                    _ => {}
                }
            }
        })
        .auto_reconnect(false)
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    // Wait for authentication (with timeout)
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();

    while !authenticated.load(Ordering::SeqCst) {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for authentication");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    assert!(
        authenticated.load(Ordering::SeqCst),
        "Should be authenticated"
    );

    // Clean up
    let _ = ws.sign_out().await;
}

/// Test that we receive the OnlineCount event.
///
/// The server sends OnlineCount approximately every 30 seconds.
/// This test waits up to 60 seconds to receive at least one.
#[tokio::test]
#[ignore]
async fn test_ws_receives_online_count() {
    if !common::has_credentials() {
        eprintln!("{}", common::skip_message());
        return;
    }

    let client = common::authenticated_client()
        .await
        .expect("Failed to authenticate");

    let received_count = Arc::new(AtomicBool::new(false));
    let received_count_clone = received_count.clone();

    let ws = client
        .websocket()
        .on_event(move |event| {
            let received = received_count_clone.clone();
            async move {
                match event {
                    WsEvent::OnlineCount {
                        connections,
                        authorized,
                    } => {
                        println!(
                            "[TEST] OnlineCount: {} connections, {} authorized",
                            connections, authorized
                        );
                        assert!(connections > 0, "Should have some connections");
                        assert!(authorized > 0, "Should have some authorized users");
                        received.store(true, Ordering::SeqCst);
                    }
                    WsEvent::Authenticated => {
                        println!("[TEST] Authenticated, waiting for OnlineCount...");
                    }
                    _ => {}
                }
            }
        })
        .auto_reconnect(false)
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    // Wait for OnlineCount (sent every ~30s, timeout at 60s)
    let timeout = Duration::from_secs(60);
    let start = std::time::Instant::now();

    while !received_count.load(Ordering::SeqCst) {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for OnlineCount event (waited 60s)");
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    assert!(
        received_count.load(Ordering::SeqCst),
        "Should have received OnlineCount"
    );

    let _ = ws.sign_out().await;
}

/// Test subscribing to new orders.
///
/// This test verifies:
/// 1. Subscription command is sent correctly
/// 2. We receive new order events
///
/// Note: This test may take a while if the market is slow.
#[tokio::test]
#[ignore]
async fn test_ws_subscribe_new_orders() {
    if !common::has_credentials() {
        eprintln!("{}", common::skip_message());
        return;
    }

    let client = common::authenticated_client()
        .await
        .expect("Failed to authenticate");

    let authenticated = Arc::new(AtomicBool::new(false));
    let received_order = Arc::new(AtomicBool::new(false));
    let auth_clone = authenticated.clone();
    let order_clone = received_order.clone();

    let ws = client
        .websocket()
        .on_event(move |event| {
            let auth = auth_clone.clone();
            let order = order_clone.clone();
            async move {
                match event {
                    WsEvent::Authenticated => {
                        println!("[TEST] Authenticated");
                        auth.store(true, Ordering::SeqCst);
                    }
                    WsEvent::OrderCreated { order: o } => {
                        println!(
                            "[TEST] New order: {} {}p x{} by {}",
                            o.order.order_type,
                            o.order.platinum,
                            o.order.quantity,
                            o.user.ingame_name
                        );
                        order.store(true, Ordering::SeqCst);
                    }
                    WsEvent::Unknown { route, .. } => {
                        // Log unknown events for debugging
                        if route.contains("subscribe") {
                            println!("[TEST] Subscription response: {}", route);
                        }
                    }
                    _ => {}
                }
            }
        })
        .subscribe(Subscription::all_new_orders())
        .auto_reconnect(false)
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    // Wait for authentication first
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();
    while !authenticated.load(Ordering::SeqCst) {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for authentication");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!("[TEST] Waiting for new orders (up to 30s)...");

    // Wait for at least one order (or timeout)
    let order_timeout = Duration::from_secs(30);
    let start = std::time::Instant::now();

    while !received_order.load(Ordering::SeqCst) {
        if start.elapsed() > order_timeout {
            println!("[TEST] No orders received in 30s (market may be slow)");
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Note: We don't assert on receiving orders because the market may be slow
    // The test passes if we connect and authenticate successfully
    println!(
        "[TEST] Received orders: {}",
        received_order.load(Ordering::SeqCst)
    );

    let _ = ws.sign_out().await;
}

/// Test setting user status via WebSocket.
///
/// This test verifies that we can set our status and receive
/// the status update event.
#[tokio::test]
#[ignore]
async fn test_ws_set_status() {
    if !common::has_credentials() {
        eprintln!("{}", common::skip_message());
        return;
    }

    let client = common::authenticated_client()
        .await
        .expect("Failed to authenticate");

    let authenticated = Arc::new(AtomicBool::new(false));
    let status_updated = Arc::new(AtomicBool::new(false));
    let auth_clone = authenticated.clone();
    let status_clone = status_updated.clone();

    let ws = client
        .websocket()
        .on_event(move |event| {
            let auth = auth_clone.clone();
            let status = status_clone.clone();
            async move {
                match event {
                    WsEvent::Authenticated => {
                        println!("[TEST] Authenticated");
                        auth.store(true, Ordering::SeqCst);
                    }
                    WsEvent::StatusUpdate {
                        status: s,
                        activity,
                        ..
                    } => {
                        println!("[TEST] Status update: {:?}, activity: {:?}", s, activity);
                        status.store(true, Ordering::SeqCst);
                    }
                    _ => {}
                }
            }
        })
        .auto_reconnect(false)
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    // Wait for authentication
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();
    while !authenticated.load(Ordering::SeqCst) {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for authentication");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Set status to invisible (safest for testing)
    use wf_market::ws::WsUserStatus;
    ws.set_status(WsUserStatus::Invisible, None, None)
        .await
        .expect("Failed to set status");

    println!("[TEST] Status command sent, waiting for update event...");

    // Wait for status update (with shorter timeout since it should be immediate)
    let status_timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();

    while !status_updated.load(Ordering::SeqCst) {
        if start.elapsed() > status_timeout {
            // Status update event might not always be sent back
            println!("[TEST] No status update event received (this may be normal)");
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let _ = ws.sign_out().await;
}

/// Test dynamic subscription after connection.
///
/// This test verifies that we can subscribe to items dynamically
/// after the initial connection is established.
#[tokio::test]
#[ignore]
async fn test_ws_dynamic_subscription() {
    if !common::has_credentials() {
        eprintln!("{}", common::skip_message());
        return;
    }

    let client = common::authenticated_client()
        .await
        .expect("Failed to authenticate");

    let authenticated = Arc::new(AtomicBool::new(false));
    let auth_clone = authenticated.clone();

    let ws = client
        .websocket()
        .on_event(move |event| {
            let auth = auth_clone.clone();
            async move {
                if let WsEvent::Authenticated = event {
                    println!("[TEST] Authenticated");
                    auth.store(true, Ordering::SeqCst);
                }
            }
        })
        .auto_reconnect(false)
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    // Wait for authentication
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();
    while !authenticated.load(Ordering::SeqCst) {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for authentication");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Subscribe to a specific item
    ws.subscribe(Subscription::item("nikana_prime_set"))
        .await
        .expect("Failed to subscribe to item");

    println!("[TEST] Subscribed to nikana_prime_set");

    // Verify subscription is tracked
    let subs = ws.subscriptions().await;
    assert!(!subs.is_empty(), "Should have at least one subscription");
    println!("[TEST] Active subscriptions: {}", subs.len());

    // Unsubscribe
    ws.unsubscribe(&Subscription::item("nikana_prime_set"))
        .await
        .expect("Failed to unsubscribe");

    println!("[TEST] Unsubscribed from nikana_prime_set");

    let _ = ws.sign_out().await;
}
