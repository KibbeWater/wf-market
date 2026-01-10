# wf-market

A Rust client library for the [warframe.market](https://warframe.market) API.

[![Crates.io](https://img.shields.io/crates/v/wf-market.svg)](https://crates.io/crates/wf-market)
[![Documentation](https://docs.rs/wf-market/badge.svg)](https://docs.rs/wf-market)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://opensource.org/licenses/GPL-3.0)

## Features

- **Type-safe API** - Compile-time guarantees prevent common mistakes like updating orders you don't own
- **Async/await** - Built on Tokio for efficient async operations
- **Session persistence** - Save and restore login sessions with serde-compatible credentials
- **Rate limiting** - Built-in rate limiter to prevent API throttling
- **Caching** - Optional caching for slowly-changing data (items, rivens)
- **WebSocket support** - Real-time order updates (optional feature)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wf-market = "0.2"

# With WebSocket support
wf-market = { version = "0.2", features = ["websocket"] }
```

## Quick Start

```rust
use wf_market::{Client, Credentials, CreateOrder};

#[tokio::main]
async fn main() -> wf_market::Result<()> {
    // Create an unauthenticated client for public data
    let client = Client::builder().build()?;
    
    // Fetch items
    let items = client.fetch_items().await?;
    println!("Found {} items", items.len());
    
    // Get orders for an item
    let orders = client.get_orders("nikana_prime_set").await?;
    for order in orders.iter().take(5) {
        println!("{}: {}p", order.user.ingame_name, order.order.platinum);
    }
    
    // Login for authenticated operations
    let creds = Credentials::new(
        "your@email.com",
        "password",
        Credentials::generate_device_id(),
    );
    let client = client.login(creds).await?;
    
    // Create an order
    let order = client.create_order(
        CreateOrder::sell("nikana_prime_set", 100, 1)
    ).await?;
    println!("Created order: {}", order.id());
    
    Ok(())
}
```

## Authentication

### Login with Credentials

```rust
use wf_market::{Client, Credentials};

let creds = Credentials::new("email@example.com", "password", Credentials::generate_device_id());
let client = Client::from_credentials(creds).await?;
```

### Session Persistence

Save and restore sessions to avoid re-authenticating:

```rust
use wf_market::{Client, Credentials};

// After login, export the session
let session = client.export_session();
let json = serde_json::to_string(&session)?;
std::fs::write("session.json", &json)?;

// Later: restore session
let saved: Credentials = serde_json::from_str(&std::fs::read_to_string("session.json")?)?;

// Validate before using (recommended)
if Client::validate_credentials(&saved).await? {
    let client = Client::from_credentials(saved).await?;
}
```

## Working with Orders

### Fetching Orders

```rust
// Get orders with user info
let orders = client.get_orders("nikana_prime_set").await?;

// Get just order data (lighter response)
let listings = client.get_listings("nikana_prime_set").await?;

// Get top buy/sell prices
let top = client.get_top_orders("nikana_prime_set").await?;
println!("Best sell: {:?}p", top.best_sell_price());
println!("Best buy: {:?}p", top.best_buy_price());
```

### Managing Your Orders

```rust
use wf_market::{CreateOrder, UpdateOrder};

// Get your orders
let my_orders = client.my_orders().await?;

// Create a sell order
let order = client.create_order(
    CreateOrder::sell("nikana_prime_set", 100, 1).visible(true)
).await?;

// Update order price
client.update_order(
    order.id(),
    UpdateOrder::new().platinum(95)
).await?;

// Delete order
client.delete_order(order.id()).await?;

// Close order (record a sale)
let transaction = client.close_order(order.id(), 1).await?;
```

### Type-Safe Order IDs

The `OwnedOrderId` type ensures you can only update/delete orders you own:

```rust
// This compiles - my_orders() returns OwnedOrder with OwnedOrderId
let orders = client.my_orders().await?;
client.delete_order(orders[0].id()).await?;

// This won't compile - get_orders() returns OrderListing with String id
let orders = client.get_orders("item").await?;
// client.delete_order(&orders[0].order.id); // Error!
```

## Caching

Use `ApiCache` for endpoints that rarely change:

```rust
use wf_market::ApiCache;
use std::time::Duration;

let mut cache = ApiCache::new();

// First call fetches from API
let items = client.get_items(Some(&mut cache)).await?;

// Subsequent calls use cache (instant)
let items = client.get_items(Some(&mut cache)).await?;

// With TTL - refresh if older than 24 hours
let items = client.get_items_with_ttl(
    Some(&mut cache),
    Duration::from_secs(24 * 60 * 60),
).await?;

// Cache is serializable for persistence
let serializable = cache.to_serializable();
let json = serde_json::to_string(&serializable)?;
```

## WebSocket (Real-time Updates)

Enable the `websocket` feature for real-time order updates:

```toml
[dependencies]
wf-market = { version = "0.2", features = ["websocket"] }
```

```rust
use wf_market::ws::{WsEvent, Subscription, WsUserStatus};

let ws = client.websocket()
    .on_event(|event| async move {
        match event {
            WsEvent::Connected => println!("Connected!"),
            WsEvent::OnlineCount { authorized, .. } => {
                println!("Users online: {}", authorized);
            }
            WsEvent::OrderCreated { order } => {
                println!("New order: {}p by {}", 
                    order.order.platinum, 
                    order.user.ingame_name
                );
            }
            _ => {}
        }
    })
    .subscribe(Subscription::all_new_orders())
    .auto_reconnect(true)
    .connect()
    .await?;

// Subscribe to specific items
ws.subscribe(Subscription::item("nikana_prime_set")).await?;

// Set your status
ws.set_status(WsUserStatus::Online, Some(3600), None).await?;
```

## Item Types

### Mods

```rust
for item in &items {
    if let Some(mod_view) = item.as_mod() {
        println!("{}: max rank {}", item.name(), mod_view.max_rank());
    }
}

// Create mod order with rank
let order = CreateOrder::sell("serration", 50, 1)
    .with_mod_rank(10);
```

### Ayatan Sculptures

```rust
for item in &items {
    if let Some(sculpture) = item.as_sculpture() {
        let full_endo = sculpture.calculate_endo(None, None);
        println!("{}: {} endo (full)", item.name(), full_endo);
    }
}
```

## Configuration

```rust
use wf_market::{Client, ClientConfig, Platform, Language};

let config = ClientConfig {
    platform: Platform::Pc,
    language: Language::English,
    crossplay: true,
    rate_limit: 3, // requests per second
};

let client = Client::builder()
    .config(config)
    .build()?;
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `rustls-tls` | Yes | Use rustls for TLS |
| `native-tls` | No | Use native TLS instead of rustls |
| `websocket` | No | Enable WebSocket support for real-time updates |

## Examples

Run the examples with:

```bash
# Basic usage (no auth required)
cargo run --example basic_usage

# Authenticated examples (set WFM_EMAIL and WFM_PASSWORD)
export WFM_EMAIL="your@email.com"
export WFM_PASSWORD="your_password"

cargo run --example create_orders
cargo run --example session_persistence
cargo run --example websocket --features websocket

# Generate JWT token for testing (saves to .env)
cargo run --example update_token
```

## Development

### Running Tests

Unit tests run without credentials:

```bash
cargo test --all-features
```

### Integration Tests

Integration tests require warframe.market credentials:

```bash
# 1. Set up credentials
cp .env.example .env
# Edit .env with your email and password

# 2. Generate a JWT token (avoids rate limiting)
cargo run --example update_token

# 3. Run integration tests (serially - server allows one WS connection)
cargo test --all-features -- --ignored --nocapture --test-threads=1
```

**Troubleshooting:**

- **"Rate limited"**: Wait 10-15 minutes, then run `cargo run --example update_token`
- **"JWT token expired"**: Run `cargo run --example update_token` to refresh
- **"Unknown error" on WebSocket**: Use `--test-threads=1` to run tests serially

## Minimum Supported Rust Version

This crate requires Rust 1.85 or later (2024 edition).

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
