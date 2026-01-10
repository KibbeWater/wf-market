# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2025-01-10

### Added

- **Type-state authentication**: `Client<Unauthenticated>` and `Client<Authenticated>` provide compile-time guarantees about which endpoints are accessible
- **`OwnedOrderId`**: Type-safe order ID that ensures you can only update/delete orders you own
- **`OwnedOrder`**: Wrapper around orders returned from `my_orders()` and `create_order()` with type-safe ID access
- **`Credentials`**: Serde-compatible struct for session persistence with `export_session()` and `from_credentials()`
- **`ApiCache`**: User-controlled caching for slowly-changing data (items, rivens) with optional TTL
- **`SerializableCache`**: Serializable version of `ApiCache` for persistence
- **`ModView` and `SculptureView`**: Zero-cost view types for accessing mod-specific and sculpture-specific item properties
- **`CreateOrder` and `UpdateOrder`**: Builder pattern for order requests
- **`TopOrders`**: Response type with `best_sell_price()`, `best_buy_price()`, and `spread()` helpers
- **WebSocket client** (feature-gated): Real-time order updates with typed `WsEvent` enum
  - `WebSocketBuilder` for fluent configuration
  - `Subscription` types for items, profiles, and new orders feed
  - Auto-reconnect support
  - Async event handlers
- **Rate limiting**: Built-in rate limiter using `governor` crate
- **Examples**: `basic_usage`, `create_orders`, `session_persistence`, `websocket`

#### New Endpoints

- **Orders**:
  - `get_recent_orders()` - Get recent orders from the last 4 hours (max 500)
  - `get_user_orders(slug)` - Get all public orders for a specific user
  - `get_top_orders(slug, filters)` - Now accepts optional `TopOrderFilters` for filtering by rank, charges, stars, and subtype
- **Items**:
  - `get_item_set(slug)` - Get all items in a set (returns `ItemSet`)
- **Users**:
  - `get_user(slug)` - Get public user profile
  - `me()` - Get current user's private profile (authenticated)
  - `update_me(update)` - Update current user's profile settings (authenticated)
- **Rivens**:
  - `get_riven(slug)` - Get single riven weapon details
  - `get_riven_attributes()` - Get all riven attribute definitions

#### New Types

- `TopOrderFilters` - Filter options for `get_top_orders()` with support for:
  - `rank`, `rank_lt` - Mod rank filters
  - `charges`, `charges_lt` - Consumable mod charges filters
  - `amber_stars`, `amber_stars_lt`, `cyan_stars`, `cyan_stars_lt` - Ayatan sculpture filters
  - `subtype` - Item subtype filter (e.g., "blueprint", "crafted")
- `ItemSet` - Response for item set endpoint with `root()` and `parts()` helpers
- `UserPrivate` - Private user profile with settings (from `/me` endpoint)
- `UpdateProfile` - Builder for updating user profile settings
- `RivenAttribute` and `RivenAttributeTranslation` - Riven attribute definitions
- `Theme`, `UserRole`, `SubscriptionTier` - New enums for user profile data

#### Enhanced Types

- `UpdateOrder` - Added missing fields: `charges`, `amber_stars`, `cyan_stars`, `subtype`

### Changed

- **Complete API redesign**: Simplified and more ergonomic API surface
- **Item model**: Replaced `Item<Regular/Mod/Sculpture>` generic states with simple `Item` struct plus `as_mod()` and `as_sculpture()` view methods
- **Order model**: Split into `Order` (data), `OrderListing` (with user), and `OwnedOrder` (yours)
- **Client construction**: Now uses builder pattern via `Client::builder()`
- **Minimum Rust version**: Updated to 1.85 (2024 edition)
- **Error handling**: Unified `Error` enum with `thiserror` derive
- **Dependencies**: Updated to latest versions of all dependencies

### Breaking Changes

- **`get_top_orders`**: Now takes an optional `TopOrderFilters` parameter
  ```rust
  // Old
  client.get_top_orders("item_slug").await?;
  
  // New (without filters)
  client.get_top_orders("item_slug", None).await?;
  
  // New (with filters)
  let filters = TopOrderFilters::new().rank(10);
  client.get_top_orders("serration", Some(&filters)).await?;
  ```
- **`OrderFilters` renamed to `TopOrderFilters`**: The type was renamed and expanded with additional filter options

### Removed

- `Item<S>` generic type parameter system (replaced with view pattern)
- Old OAuth flow (replaced with credentials-based auth)
- Legacy test infrastructure

### Migration Guide

#### Client Creation

```rust
// Old (v0.1.x)
let client = WfmClient::new();

// New (v0.2.0)
let client = Client::builder().build()?;
```

#### Authentication

```rust
// Old (v0.1.x)
client.login("email", "password").await?;

// New (v0.2.0)
let creds = Credentials::new("email", "password", Credentials::generate_device_id());
let client = Client::from_credentials(creds).await?;
// Or from unauthenticated:
let client = client.login(creds).await?;
```

#### Getting Orders

```rust
// Old (v0.1.x)
let orders = client.get_orders("nikana_prime_set").await?;

// New (v0.2.0) - Same API, but returns OrderListing with nested order/user
let orders = client.get_orders("nikana_prime_set").await?;
for order in &orders {
    println!("{}: {}p", order.user.ingame_name, order.order.platinum);
}
```

#### Managing Your Orders

```rust
// Old (v0.1.x)
client.create_order("nikana_prime_set", 100, 1, OrderType::Sell).await?;

// New (v0.2.0)
let order = client.create_order(
    CreateOrder::sell("nikana_prime_set", 100, 1)
).await?;
// order.id() returns OwnedOrderId for type-safe updates
client.update_order(order.id(), UpdateOrder::new().platinum(95)).await?;
```

#### Item Properties

```rust
// Old (v0.1.x)
let item: Item<Mod> = ...;
let rank = item.max_rank;

// New (v0.2.0)
let item: Item = ...;
if let Some(mod_view) = item.as_mod() {
    let rank = mod_view.max_rank();
}
```

## [0.1.x] - Previous Releases

See git history for changes prior to v0.2.0.
