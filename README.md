# wf-market

A small Rust library to interact with the [warframe.market](https://warframe.market) API.

[![Crates.io](https://img.shields.io/crates/v/wf-market.svg)](https://crates.io/crates/wf-market)
[![Documentation](https://docs.rs/wf-market/badge.svg)](https://docs.rs/wf-market)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](https://opensource.org/licenses/GPL-3.0)

## Features

- **Type-safe API** - The `Client` is generic over its authentication state, so authenticated-only
  operations (orders, auctions, chats) are only available once you log in
- **Async/await** - Built on Tokio and Reqwest for efficient async operations
- **Route-based design** - Each domain (`item`, `order`, `user`, `auction`, `chat`, ...) is exposed
  through its own route on the client
- **Built-in caching** - Items, rivens, lich/sister data, manifest data and orders are cached in-memory
- **Rate limiting** - Built-in adaptive rate limiter that downgrades per-route quotas on `429`s
- **WebSocket support** - Real-time data with automatic reconnection (V1 and V2 protocols)
- **Event callbacks** - Subscribe to `api:before`, `api:after`, `api:error`, `rate_limit:applied`, etc.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
wf-market = "0.1"
```

## Quick Start

```rust
use wf_market::Client;

#[tokio::main]
async fn main() -> Result<(), wf_market::errors::ApiError> {
    // Create an unauthenticated client for public data
    let client = Client::new();

    // Fetch all items
    let items = client.item().get_all().await?;
    println!("Found {} items", items.len());

    // Fetch an item by its slug
    let item = client.item().get_by_slug("nikana_prime_set").await?;
    println!("Item: {}", item.i18n.get("en").map(|t| t.name.as_str()).unwrap_or(&item.slug));

    // Get orders for an item (only sellers, lowest first)
    let orders = client.order().get_orders_by_item("nikana_prime_set").await?;
    println!(
        "Best sell: {}p, best buy: {}p",
        orders.lowest_price(wf_market::enums::OrderType::Sell),
        orders.highest_price(wf_market::enums::OrderType::Buy)
    );

    Ok(())
}
```

## Authentication

`Client` starts in the `Unauthenticated` state. Logging in returns a new `Client<Authenticated>`
which unlocks authenticated-only routes.

```rust
use wf_market::Client;

let client = Client::new();
let client = client
    .login("your@email.com", "your_password", "my_device_id")
    .await?;
```

> **Note**: Reusing the same `device_id` keeps requests associated with the same device. Generating a
> new one every time will create multiple devices on your account.

You can also log in with an existing JWT token (e.g. one you saved earlier):

```rust
let client = Client::new()
    .login_with_token("your-jwt-token", "my_device_id")
    .await?;
```

Once authenticated you can read your profile, orders, auctions and chats:

```rust
let user = client.get_user()?;
println!("Logged in as: {}", user.ingame_name);
println!("Token: {}", client.get_token());
println!("Device ID: {}", client.get_device_id());

// Refresh all internal data (user, orders, auctions, chats)
client.refresh().await?;
```

### Client Configuration

Set the language, platform and crossplay before making requests:

```rust
use wf_market::{Client, enums::{Language, Platform}};

let client = Client::new()
    .with_language(Language::English)
    .with_platform(Platform::Pc)
    .with_crossplay(true);
```

### Rate Limiting

The client applies a global limit (default 3 requests/second) and automatically downgrades the
per-route quota when the API returns a `429`. You can adjust the global limit:

```rust
use std::num::NonZeroU32;

let mut client = Client::new();
client.set_rate_limit(NonZeroU32::new(10).unwrap());
```

## Working with Orders

### Fetching Orders

```rust
// Recent orders (max 500, last 4 hours, cached with 1min refresh)
let recent = client.order().recent().await?;

// All orders for an item, filtered for online users
let mut orders = client.order().get_orders_by_item("nikana_prime_set").await?;
orders.filter_user_status(wf_market::enums::StatusType::InGame, false);

// Top buy/sell orders for an item
let top = client.order()
    .get_top_orders_by_item("nikana_prime_set", Some(wf_market::types::TopOrdersFilters::new()))
    .await?;
println!("Best sell: {:?}p", top.sell.first().map(|o| o.order.platinum));
println!("Best buy: {:?}p", top.buy.first().map(|o| o.order.platinum));

// Fetch a single order by ID
let order = client.order().get_by_id("6859657e57605a002b649eee").await?;
```

### Managing Your Orders (Authenticated)

```rust
use wf_market::{
    enums::OrderType,
    types::{CreateOrderParams, SubType, UpdateOrderParams},
};

// Get your orders (also populates the internal cache)
let my_orders = client.order().my_orders().await?;
println!("You have {} active orders", my_orders.total_orders());

// Create a sell order for a mod with rank 10
let order = client.order()
    .create(CreateOrderParams::new_with_subtype(
        "5c1bda1314a8e4006b1dad81", // item_id
        OrderType::Sell,
        100,
        1,
        true,
        Some(1), // per trade
        SubType::mods(10),
    ))
    .await?;
println!("Created order: {}", order.id);

// Update order price
client.order()
    .update(&order.id, UpdateOrderParams::new().with_platinum(95))
    .await?;

// Close part of an order (records a sale)
let transaction = client.order().close(&order.id, 1).await?;

// Delete an order
client.order().delete(&order.id).await?;
```

`OrderList` provides many helpers: `lowest_price`, `highest_price`, `price_range`, `find_order`,
`filter_by_sub_type`, `filter_username`, `take_top`, `order_ids`, and more.

## Caching

Slowly-changing data (items, rivens, lich/sister data, manifest data, achievements, orders) is cached
in-memory automatically. Repeated calls hit the cache instead of the API:

```rust
let items = client.item().get_all().await?; // fetches from API, caches
let items = client.item().get_all().await?; // served from cache (instant)
```

## WebSocket (Real-time Updates)

The WebSocket client is available on an authenticated client via `create_websocket`, and supports both
the V1 and V2 protocols (with automatic reconnection built in).

```rust
use wf_market::{Client, enums::ApiVersion};

let client = Client::new()
    .login("your@email.com", "your_password", "my_device_id")
    .await?;

let ws = client
    .create_websocket(ApiVersion::V2)
    .set_log_unhandled(true)
    .register_callback("cmd/status/set:ok", |msg, _, _| {
        println!("Status set: {:?}", msg);
        Ok(())
    })?
    .register_callback("event/user/login", |msg, _, _| {
        println!("User logged in: {:?}", msg);
        Ok(())
    })?
    .build()
    .await?;

// Send a request and wait for the :ok callback
ws.send_request("@wfm|cmd/status/set", serde_json::json!({ "status": "invisible" }))?;
```

Internal lifecycle events are available under `internal/connected`, `internal/disconnected` and
`internal/reconnecting`.

## Liches and Sisters

```rust
let weapons = client.lich().get_all_weapons().await?;
let ephemeras = client.lich().get_all_ephemeras().await?;
let quirks = client.lich().get_all_quirks().await?;

let weapons = client.sister().get_all_weapons().await?;
let ephemeras = client.sister().get_all_ephemeras().await?;
let quirks = client.sister().get_all_quirks().await?;
```

## Auctions (Authenticated)

```rust
use wf_market::types::{AuctionFilter, CreateAuctionParams, CreateAuctionItem};

let auctions = client.auction().my_auctions().await?;
let recent = client.auction().get_recent_auctions().await?;

let results = client.auction().search_auctions(AuctionFilter::default()).await?;

// Create an auction for a riven mod
let auction = client.auction().create(CreateAuctionParams::new(
    100,
    Some(300),
    0,
    true,
    "Selling my riven",
    CreateAuctionItem::new_riven(
        "some_weapon",
        "riven_mod_name",
        vec![],
        5,
        18,
        8,
        wf_market::enums::Polarity::Madurai,
    ),
)).await?;
client.auction().delete(&auction.id).await?;
```

## Chats (Authenticated)

```rust
let chats = client.chat().get_chats().await?;
let messages = client.chat().get_chat_messages("some_chat_id").await?;
client.chat().leave_chat("some_chat_id").await?;
let ignored = client.chat().ignore_users().await?;
```

## Events

The client lets you observe every API request, response and error:

```rust
use wf_market::Client;

let client = Client::new();
client.on("api:before", |event, data| {
    println!("{} -> {}", event, data.get_property_value("url", String::new()));
});
client.on("api:after", |event, data| {
    println!("{} -> status {}", event, data.get_property_value("status", 0u16));
});
client.on("api:error", |_, data| {
    println!("request failed: {}", data.get_property_value("key", String::new()));
});
```

You can also register callbacks when building the client with `with_callback`, and remove them with
`off` / `clear_callbacks`.

## Item Types

### Mods

```rust
for item in &items {
    if let Some(max_rank) = item.max_rank {
        println!("{} is a mod with max rank {}", item.slug, max_rank);
    }
}

// Create a mod order with a rank using the SubType
let order = CreateOrderParams::new_with_subtype(
    item_id,
    OrderType::Sell,
    50,
    1,
    true,
    Some(1),
    SubType::mods(10),
);
```

### Ayatan Sculptures

```rust
use wf_market::types::SubType;

for item in &items {
    if let Some(base_endo) = item.base_endo {
        println!("{} base endo: {}", item.slug, base_endo);
    }
}

// An ayatan sculpture order with stars
let order = CreateOrderParams::new_with_subtype(
    item_id,
    OrderType::Sell,
    30,
    1,
    true,
    Some(1),
    SubType::ayatan_sculpture(2, 3),
);
```

## Feature Flags

The crate currently has no optional feature flags - all functionality (including WebSocket support) is
enabled by default.

## Minimum Supported Rust Version

This crate requires Rust 1.85 or later (2024 edition).

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
