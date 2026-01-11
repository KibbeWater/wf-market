//! # wf-market
//!
//! A Rust client library for the [warframe.market](https://warframe.market) API.
//!
//! This library provides a type-safe, async client for interacting with the
//! warframe.market trading platform, including both HTTP REST API and WebSocket
//! real-time updates.
//!
//! ## Features
//!
//! - **Type-safe API**: Compile-time guarantees prevent common mistakes
//! - **Async/await**: Built on Tokio for efficient async operations
//! - **Session persistence**: Save and restore login sessions
//! - **Rate limiting**: Built-in rate limiter to prevent API throttling
//! - **Caching**: Optional caching for slowly-changing data
//! - **WebSocket support**: Real-time updates (optional feature)
//!
//! ## Quick Start
//!
//! ```ignore
//! use wf_market::{Client, Credentials, CreateOrder};
//!
//! #[tokio::main]
//! async fn main() -> wf_market::Result<()> {
//!     // Create a client (fetches items automatically)
//!     let client = Client::builder().build().await?;
//!     
//!     // Items are pre-loaded and accessible via client.items()
//!     println!("Loaded {} items", client.items().len());
//!     
//!     // Get orders for an item
//!     let orders = client.get_orders("nikana_prime_set").await?;
//!     for order in orders.iter().take(5) {
//!         // Item info is automatically available on orders
//!         if let Some(item) = order.get_item() {
//!             println!("{}: {} @ {}p", order.user.ingame_name, item.name(), order.platinum);
//!         }
//!     }
//!     
//!     // Login for authenticated operations
//!     let creds = Credentials::new(
//!         "your@email.com",
//!         "password",
//!         Credentials::generate_device_id(),
//!     );
//!     let client = client.login(creds).await?;
//!     
//!     // Create an order
//!     let order = client.create_order(
//!         CreateOrder::sell("nikana_prime_set", 100, 1)
//!     ).await?;
//!     println!("Created order: {}", order.id());
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Session Persistence
//!
//! Save and restore login sessions to avoid re-authenticating:
//!
//! ```ignore
//! use wf_market::{Client, Credentials};
//!
//! async fn example() -> wf_market::Result<()> {
//!     // Initial login
//!     let creds = Credentials::new("email", "password", Credentials::generate_device_id());
//!     let client = Client::from_credentials(creds).await?;
//!     
//!     // Save session
//!     let session = client.export_session();
//!     let json = serde_json::to_string(&session)?;
//!     std::fs::write("session.json", &json)?;
//!     
//!     // Later: restore session
//!     let saved: Credentials = serde_json::from_str(&std::fs::read_to_string("session.json")?)?;
//!     
//!     // Validate before using (recommended)
//!     if Client::validate_credentials(&saved).await? {
//!         let client = Client::from_credentials(saved).await?;
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Caching
//!
//! Use the [`ApiCache`] to persist items across application restarts:
//!
//! ```ignore
//! use wf_market::{Client, ApiCache, SerializableCache};
//!
//! async fn example() -> wf_market::Result<()> {
//!     // Load cache from disk (or create new)
//!     let mut cache = match std::fs::read_to_string("cache.json") {
//!         Ok(json) => serde_json::from_str::<SerializableCache>(&json)?
//!             .into_api_cache(),
//!         Err(_) => ApiCache::new(),
//!     };
//!     
//!     // Build client using cache (uses cached items if < 1 day old)
//!     let client = Client::builder()
//!         .build_with_cache(&mut cache)
//!         .await?;
//!     
//!     // Items are loaded from cache or API
//!     println!("Loaded {} items", client.items().len());
//!     
//!     // Save cache for next time
//!     let serializable = SerializableCache::from(&cache);
//!     std::fs::write("cache.json", serde_json::to_string(&serializable)?)?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## WebSocket (Real-time Updates)
//!
//! Enable the `websocket` feature for real-time order updates:
//!
//! ```toml
//! [dependencies]
//! wf-market = { version = "0.2", features = ["websocket"] }
//! ```
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
//! ```
//!
//! ## Feature Flags
//!
//! - `default` = `["rustls-tls"]`
//! - `rustls-tls`: Use rustls for TLS (default)
//! - `native-tls`: Use native TLS instead of rustls
//! - `websocket`: Enable WebSocket support for real-time updates

// Modules
mod api;
pub mod cache;
pub mod client;
pub mod error;
mod internal;
pub mod models;

#[cfg(feature = "websocket")]
pub mod ws;

// Re-exports for convenience
pub use cache::{ApiCache, CachedItems, CachedRivens, SerializableCache};
pub use client::{AuthState, Authenticated, Client, ClientBuilder, ClientConfig, Unauthenticated};
pub use error::{ApiErrorDetails, ApiErrorResponse, Error, Result};

#[cfg(feature = "websocket")]
pub use error::WsError;

// Model re-exports
pub use models::{
    // Users
    Achievement,
    AchievementType,
    // Common types
    Activity,
    ActivityType,
    // Orders
    CreateOrder,
    // Auth
    Credentials,
    FullUser,
    // Items
    Item,
    ItemIndex,
    ItemSet,
    ItemTranslation,
    Language,
    ModView,
    Order,
    OrderListing,
    OrderType,
    OwnedOrder,
    OwnedOrderId,
    Platform,
    Rarity,
    // Rivens
    Riven,
    RivenAttribute,
    RivenAttributeTranslation,
    RivenTranslation,
    RivenType,
    SculptureView,
    SubscriptionTier,
    Theme,
    TopOrderFilters,
    TopOrders,
    // Transactions
    Transaction,
    UpdateOrder,
    UpdateProfile,
    User,
    UserPrivate,
    UserProfile,
    UserRole,
    UserStatus,
    WithPassword,
};
