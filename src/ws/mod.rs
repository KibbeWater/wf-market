//! WebSocket client for real-time updates.
//!
//! This module provides a typed WebSocket client for receiving real-time
//! updates from warframe.market.
//!
//! # Feature Flag
//!
//! This module is only available with the `websocket` feature enabled:
//!
//! ```toml
//! [dependencies]
//! wf-market = { version = "0.2", features = ["websocket"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use wf_market::{Client, Credentials};
//! use wf_market::ws::{WsEvent, Subscription};
//!
//! async fn example() -> wf_market::Result<()> {
//!     let client = Client::from_credentials(/* ... */).await?;
//!
//!     let ws = client.websocket()
//!         .on_event(|event| async move {
//!             match event {
//!                 WsEvent::Connected => println!("Connected!"),
//!                 WsEvent::OnlineCount { authorized, .. } => {
//!                     println!("Users online: {}", authorized);
//!                 }
//!                 WsEvent::OrderCreated { order } => {
//!                     println!("New order: {}p", order.platinum);
//!                 }
//!                 _ => {}
//!             }
//!         })
//!         .subscribe(Subscription::all_new_orders())
//!         .connect()
//!         .await?;
//!
//!     Ok(())
//! }
//! # fn main() {}
//! ```

mod builder;
mod client;
mod events;
mod message;
mod subscription;

pub use builder::WebSocketBuilder;
pub use client::WebSocket;
pub use events::{Activity, ActivityType, UserStatus as WsUserStatus, WsEvent};
pub use subscription::Subscription;

/// WebSocket server URL.
pub(crate) const WS_URL: &str = "wss://ws.warframe.market/socket";

/// WebSocket sub-protocol.
pub(crate) const WS_PROTOCOL: &str = "wfm";
