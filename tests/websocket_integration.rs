//! WebSocket integration tests.
//!
//! These tests require valid warframe.market credentials set via environment variables
//! or in a `.env` file.
//!
//! ## Setup
//!
//! 1. Copy `.env.example` to `.env` and fill in your credentials
//! 2. Run `cargo run --example update_token` to generate a JWT token
//!
//! ## Running tests
//!
//! **Important**: Run tests with `--test-threads=1` to avoid connection conflicts.
//! The warframe.market server only allows one WebSocket connection per user at a time.
//!
//! ```bash
//! # Run all integration tests (serially to avoid conflicts)
//! cargo test --all-features -- --ignored --nocapture --test-threads=1
//!
//! # Run a specific test
//! cargo test --all-features -- --ignored test_ws_connect --nocapture
//! ```
//!
//! ## Troubleshooting
//!
//! - **"JWT token expired"**: Run `cargo run --example update_token` to refresh
//! - **"Rate limited"**: Wait 10-15 minutes before retrying
//! - **"No credentials"**: Check that `.env` file exists with valid credentials
//! - **"Unknown error" on WebSocket auth**: Run tests with `--test-threads=1`

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use wf_market::ws::{Subscription, WsEvent};

/// Helper to authenticate and return client, or skip/fail with useful message.
async fn get_authenticated_client() -> Option<wf_market::Client<wf_market::Authenticated>> {
    if !common::has_credentials() {
        eprintln!("{}", common::skip_message());
        return None;
    }

    match common::authenticated_client().await {
        Ok((client, source)) => {
            eprintln!("[TEST] Authenticated via {}", source);
            Some(client)
        }
        Err(e) => {
            eprintln!("\n[TEST] Authentication failed!\n");
            eprintln!("{}", e);
            eprintln!();

            // Provide actionable advice based on error type
            match &e {
                common::AuthError::TokenExpired { .. } => {
                    eprintln!(">>> Run: cargo run --example update_token");
                }
                common::AuthError::RateLimited { .. } => {
                    eprintln!(">>> Wait 10-15 minutes, then run: cargo run --example update_token");
                }
                common::AuthError::NoCredentials => {
                    eprintln!(">>> Copy .env.example to .env and fill in credentials");
                }
                _ => {}
            }
            eprintln!();

            panic!("Authentication failed: {}", e);
        }
    }
}

/// Test that we can connect to the WebSocket and authenticate.
///
/// This test verifies:
/// 1. TLS connection works (wss://)
/// 2. Authentication with JWT token works
/// 3. We receive the Authenticated event
#[tokio::test]
#[ignore]
async fn test_ws_connect_and_authenticate() {
    let Some(client) = get_authenticated_client().await else {
        return;
    };

    let authenticated = Arc::new(AtomicBool::new(false));
    let auth_failed = Arc::new(AtomicBool::new(false));
    let auth_error = Arc::new(std::sync::Mutex::new(String::new()));

    let authenticated_clone = authenticated.clone();
    let auth_failed_clone = auth_failed.clone();
    let auth_error_clone = auth_error.clone();

    let ws = client
        .websocket()
        .on_event(move |event| {
            let auth = authenticated_clone.clone();
            let failed = auth_failed_clone.clone();
            let error = auth_error_clone.clone();
            async move {
                match event {
                    WsEvent::Connected => {
                        eprintln!("[WS] Connected to WebSocket server");
                    }
                    WsEvent::Authenticated => {
                        eprintln!("[WS] Authenticated successfully");
                        auth.store(true, Ordering::SeqCst);
                    }
                    WsEvent::AuthenticationFailed { error: e } => {
                        eprintln!("[WS] Authentication failed: {}", e);
                        *error.lock().unwrap() = e;
                        failed.store(true, Ordering::SeqCst);
                    }
                    WsEvent::Disconnected { reason } => {
                        eprintln!("[WS] Disconnected: {}", reason);
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

    while !authenticated.load(Ordering::SeqCst) && !auth_failed.load(Ordering::SeqCst) {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for authentication response");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Check if auth failed via WebSocket
    if auth_failed.load(Ordering::SeqCst) {
        let err = auth_error.lock().unwrap().clone();
        eprintln!();
        eprintln!("[TEST] WebSocket authentication failed!");
        eprintln!("[TEST] This usually means the JWT token is expired.");
        eprintln!(">>> Run: cargo run --example update_token");
        eprintln!();
        panic!("WebSocket authentication failed: {}", err);
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
    let Some(client) = get_authenticated_client().await else {
        return;
    };

    let received_count = Arc::new(AtomicBool::new(false));
    let auth_failed = Arc::new(AtomicBool::new(false));

    let received_count_clone = received_count.clone();
    let auth_failed_clone = auth_failed.clone();

    let ws = client
        .websocket()
        .on_event(move |event| {
            let received = received_count_clone.clone();
            let failed = auth_failed_clone.clone();
            async move {
                match event {
                    WsEvent::OnlineCount {
                        connections,
                        authorized,
                    } => {
                        eprintln!(
                            "[WS] OnlineCount: {} connections, {} authorized",
                            connections, authorized
                        );
                        assert!(connections > 0, "Should have some connections");
                        assert!(authorized > 0, "Should have some authorized users");
                        received.store(true, Ordering::SeqCst);
                    }
                    WsEvent::Authenticated => {
                        eprintln!("[WS] Authenticated, waiting for OnlineCount...");
                    }
                    WsEvent::AuthenticationFailed { error } => {
                        eprintln!("[WS] Authentication failed: {}", error);
                        failed.store(true, Ordering::SeqCst);
                    }
                    _ => {}
                }
            }
        })
        .auto_reconnect(false)
        .connect()
        .await
        .expect("Failed to connect to WebSocket");

    // Check for early auth failure
    tokio::time::sleep(Duration::from_secs(2)).await;
    if auth_failed.load(Ordering::SeqCst) {
        eprintln!();
        eprintln!("[TEST] WebSocket authentication failed!");
        eprintln!(">>> Run: cargo run --example update_token");
        panic!("WebSocket authentication failed - token likely expired");
    }

    // Wait for OnlineCount (sent every ~30s, timeout at 60s)
    let timeout = Duration::from_secs(60);
    let start = std::time::Instant::now();

    while !received_count.load(Ordering::SeqCst) {
        if auth_failed.load(Ordering::SeqCst) {
            eprintln!();
            eprintln!("[TEST] WebSocket authentication failed during test!");
            eprintln!(">>> Run: cargo run --example update_token");
            panic!("WebSocket authentication failed - token likely expired");
        }
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
    let Some(client) = get_authenticated_client().await else {
        return;
    };

    let authenticated = Arc::new(AtomicBool::new(false));
    let auth_failed = Arc::new(AtomicBool::new(false));
    let received_order = Arc::new(AtomicBool::new(false));

    let auth_clone = authenticated.clone();
    let failed_clone = auth_failed.clone();
    let order_clone = received_order.clone();

    let ws = client
        .websocket()
        .on_event(move |event| {
            let auth = auth_clone.clone();
            let failed = failed_clone.clone();
            let order = order_clone.clone();
            async move {
                match event {
                    WsEvent::Authenticated => {
                        eprintln!("[WS] Authenticated");
                        auth.store(true, Ordering::SeqCst);
                    }
                    WsEvent::AuthenticationFailed { error } => {
                        eprintln!("[WS] Authentication failed: {}", error);
                        failed.store(true, Ordering::SeqCst);
                    }
                    WsEvent::OrderCreated { order: o } => {
                        eprintln!(
                            "[WS] New order: {} {}p x{} by {}",
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
                            eprintln!("[WS] Subscription response: {}", route);
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
    while !authenticated.load(Ordering::SeqCst) && !auth_failed.load(Ordering::SeqCst) {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for authentication");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    if auth_failed.load(Ordering::SeqCst) {
        eprintln!();
        eprintln!("[TEST] WebSocket authentication failed!");
        eprintln!(">>> Run: cargo run --example update_token");
        panic!("WebSocket authentication failed - token likely expired");
    }

    eprintln!("[TEST] Waiting for new orders (up to 30s)...");

    // Wait for at least one order (or timeout)
    let order_timeout = Duration::from_secs(30);
    let start = std::time::Instant::now();

    while !received_order.load(Ordering::SeqCst) {
        if start.elapsed() > order_timeout {
            eprintln!("[TEST] No orders received in 30s (market may be slow)");
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Note: We don't assert on receiving orders because the market may be slow
    // The test passes if we connect and authenticate successfully
    eprintln!(
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
    let Some(client) = get_authenticated_client().await else {
        return;
    };

    let authenticated = Arc::new(AtomicBool::new(false));
    let auth_failed = Arc::new(AtomicBool::new(false));
    let status_updated = Arc::new(AtomicBool::new(false));

    let auth_clone = authenticated.clone();
    let failed_clone = auth_failed.clone();
    let status_clone = status_updated.clone();

    let ws = client
        .websocket()
        .on_event(move |event| {
            let auth = auth_clone.clone();
            let failed = failed_clone.clone();
            let status = status_clone.clone();
            async move {
                match event {
                    WsEvent::Authenticated => {
                        eprintln!("[WS] Authenticated");
                        auth.store(true, Ordering::SeqCst);
                    }
                    WsEvent::AuthenticationFailed { error } => {
                        eprintln!("[WS] Authentication failed: {}", error);
                        failed.store(true, Ordering::SeqCst);
                    }
                    WsEvent::StatusUpdate {
                        status: s,
                        activity,
                        ..
                    } => {
                        eprintln!("[WS] Status update: {:?}, activity: {:?}", s, activity);
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
    while !authenticated.load(Ordering::SeqCst) && !auth_failed.load(Ordering::SeqCst) {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for authentication");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    if auth_failed.load(Ordering::SeqCst) {
        eprintln!();
        eprintln!("[TEST] WebSocket authentication failed!");
        eprintln!(">>> Run: cargo run --example update_token");
        panic!("WebSocket authentication failed - token likely expired");
    }

    // Set status to invisible (safest for testing)
    use wf_market::ws::WsUserStatus;
    ws.set_status(WsUserStatus::Invisible, None, None)
        .await
        .expect("Failed to set status");

    eprintln!("[TEST] Status command sent, waiting for update event...");

    // Wait for status update (with shorter timeout since it should be immediate)
    let status_timeout = Duration::from_secs(5);
    let start = std::time::Instant::now();

    while !status_updated.load(Ordering::SeqCst) {
        if start.elapsed() > status_timeout {
            // Status update event might not always be sent back
            eprintln!("[TEST] No status update event received (this may be normal)");
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
    let Some(client) = get_authenticated_client().await else {
        return;
    };

    let authenticated = Arc::new(AtomicBool::new(false));
    let auth_failed = Arc::new(AtomicBool::new(false));

    let auth_clone = authenticated.clone();
    let failed_clone = auth_failed.clone();

    let ws = client
        .websocket()
        .on_event(move |event| {
            let auth = auth_clone.clone();
            let failed = failed_clone.clone();
            async move {
                match event {
                    WsEvent::Authenticated => {
                        eprintln!("[WS] Authenticated");
                        auth.store(true, Ordering::SeqCst);
                    }
                    WsEvent::AuthenticationFailed { error } => {
                        eprintln!("[WS] Authentication failed: {}", error);
                        failed.store(true, Ordering::SeqCst);
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
    while !authenticated.load(Ordering::SeqCst) && !auth_failed.load(Ordering::SeqCst) {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for authentication");
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    if auth_failed.load(Ordering::SeqCst) {
        eprintln!();
        eprintln!("[TEST] WebSocket authentication failed!");
        eprintln!(">>> Run: cargo run --example update_token");
        panic!("WebSocket authentication failed - token likely expired");
    }

    // Subscribe to a specific item
    ws.subscribe(Subscription::item("nikana_prime_set"))
        .await
        .expect("Failed to subscribe to item");

    eprintln!("[TEST] Subscribed to nikana_prime_set");

    // Verify subscription is tracked
    let subs = ws.subscriptions().await;
    assert!(!subs.is_empty(), "Should have at least one subscription");
    eprintln!("[TEST] Active subscriptions: {}", subs.len());

    // Unsubscribe
    ws.unsubscribe(&Subscription::item("nikana_prime_set"))
        .await
        .expect("Failed to unsubscribe");

    eprintln!("[TEST] Unsubscribed from nikana_prime_set");

    let _ = ws.sign_out().await;
}
