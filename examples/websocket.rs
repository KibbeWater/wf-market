//! WebSocket example for real-time order updates.
//!
//! This example demonstrates the WebSocket client for receiving
//! real-time market updates.
//!
//! Requires the `websocket` feature:
//! ```toml
//! wf-market = { version = "0.2", features = ["websocket"] }
//! ```
//!
//! Set credentials via environment variables or .env file:
//! - WFM_EMAIL: Your warframe.market email
//! - WFM_PASSWORD: Your warframe.market password
//!
//! Run with: `cargo run --example websocket --features websocket`

use std::env;
use std::time::Duration;

use wf_market::ws::{Subscription, WsEvent, WsUserStatus};
use wf_market::{Client, Credentials};

#[tokio::main]
async fn main() -> wf_market::Result<()> {
    // Load .env file if present
    let _ = dotenv::dotenv();

    // Load credentials
    let email = env::var("WFM_EMAIL").expect("WFM_EMAIL not set (use .env file or environment)");
    let password =
        env::var("WFM_PASSWORD").expect("WFM_PASSWORD not set (use .env file or environment)");

    println!("=== Logging in ===");
    let creds = Credentials::new(&email, &password, Credentials::generate_device_id());
    let client = Client::from_credentials(creds).await?;
    println!("Logged in!");

    println!("\n=== Connecting to WebSocket ===");

    // Create WebSocket connection with event handler
    let ws = client
        .websocket()
        .on_event(|event| async move {
            match event {
                WsEvent::Connected => {
                    println!("[WS] Connected to server");
                }
                WsEvent::Authenticated => {
                    println!("[WS] Authenticated successfully");
                }
                WsEvent::AuthenticationFailed { error } => {
                    println!("[WS] Authentication failed: {}", error);
                }
                WsEvent::OnlineCount {
                    connections,
                    authorized,
                } => {
                    println!(
                        "[WS] Online: {} connections, {} authorized users",
                        connections, authorized
                    );
                }
                WsEvent::StatusUpdate {
                    status, activity, ..
                } => {
                    println!("[WS] Status update: {:?}, Activity: {:?}", status, activity);
                }
                WsEvent::OrderCreated { order } => {
                    println!(
                        "[WS] New order: {} - {}p x{} by {}",
                        order.order.item_id,
                        order.order.platinum,
                        order.order.quantity,
                        order.user.ingame_name
                    );
                }
                WsEvent::Disconnected { reason } => {
                    println!("[WS] Disconnected: {}", reason);
                }
                WsEvent::SessionRevoked => {
                    println!("[WS] Session was revoked!");
                }
                WsEvent::Banned { until, message } => {
                    println!("[WS] Banned until {}: {}", until, message);
                }
                WsEvent::Warned { message } => {
                    println!("[WS] Warning: {}", message);
                }
                WsEvent::Unknown { route, .. } => {
                    println!("[WS] Unknown event: {}", route);
                }
                _ => {}
            }
        })
        // Subscribe to all new orders on connect
        .subscribe(Subscription::all_new_orders())
        // Auto-reconnect if disconnected
        .auto_reconnect(true)
        .reconnect_delay(Duration::from_secs(5))
        .connect()
        .await?;

    println!("WebSocket connected!");

    // You can also subscribe dynamically after connection
    println!("\n=== Dynamic Subscriptions ===");
    ws.subscribe(Subscription::item("nikana_prime_set")).await?;
    println!("Subscribed to nikana_prime_set updates");

    // Set user status
    println!("\n=== Setting Status ===");
    ws.set_status(WsUserStatus::Online, Some(3600), None)
        .await?;
    println!("Status set to online for 1 hour");

    // Keep the connection alive
    println!("\n=== Listening for events (Ctrl+C to stop) ===");
    println!("Watching for new orders...\n");

    // In a real application, you might run this forever or until some condition
    tokio::time::sleep(Duration::from_secs(60)).await;

    // Unsubscribe and cleanup
    ws.unsubscribe(&Subscription::item("nikana_prime_set"))
        .await?;
    ws.sign_out().await?;

    println!("\nDone!");
    Ok(())
}
