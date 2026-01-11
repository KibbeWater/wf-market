# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2] - 2026-01-11

### Fixed

- **Statistics Deserialization**: Handle API responses where numeric values are returned as floats (e.g., `100.0`) instead of integers
- **Statistics Data Structure**: Properly handle differences between `statistics_closed` and `statistics_live`:
  - OHLC fields (`open_price`, `closed_price`, `donch_top`, `donch_bot`) are now optional (only present in closed stats)
  - Added `order_type` field for live order statistics
  - Added `mod_rank` field for mod item statistics

### Added

- Helper methods `is_closed_trade()` and `is_live_order()` on `StatisticEntry`
- Integration tests for statistics endpoint covering regular items, mods, and archon mods

## [0.3.1] - 2026-01-11

### Added

- **V1 API Statistics Endpoint**: `get_item_statistics()` for historical price/volume data
  - Requires `v1-api` feature flag: `wf-market = { features = ["v1-api"] }`
  - Returns 48-hour (hourly) and 90-day (daily) statistics
  - Includes closed trade data and live order data
  - Helper methods: `recent_avg_price()`, `has_sufficient_data()`, etc.
- **New types**: `ItemStatistics`, `TimeframedStatistics`, `StatisticEntry`
- **New feature flag**: `v1-api` for deprecated V1 API endpoints

## [0.3.0] - 2026-01-11

### Added

- **Client-level Item Index**: Items are now automatically fetched and indexed when building a client
  - `Client::items()` - access the loaded items
  - `Client::get_item_by_id()` / `get_item_by_slug()` - O(1) item lookups
  - `order.get_item()` - direct item access from any order type
- **Standalone Item Fetching**: `ItemIndex::fetch()` and `ItemIndex::fetch_with_config()` to fetch items without a client
- **Sync Client Construction**: `ClientBuilder::build_with_items()` for synchronous client creation with pre-loaded items
- **Cached Client Construction**: `ClientBuilder::build_with_cache()` uses cached items if less than 1 day old
- **Item Revalidation**: `Client::revalidate_items()` to refresh items for long-running applications
- **Item Type Detection**: `Item::is_regular()` returns true for items that are neither mods nor sculptures

### Changed

- **BREAKING**: `ClientBuilder::build()` is now async and automatically fetches items
  - Before: `Client::builder().build()?`
  - After: `Client::builder().build().await?`

## [0.2.2] - 2026-01-10

### Fixed

- **WebSocket TLS**: Added TLS support for `tokio-tungstenite` when using `rustls-tls` feature. WebSocket connections to `wss://` now work correctly.
- **WebSocket Protocol**: Fixed message route format to match actual API (`@wfm|cmd/...` and `@wfm|event/...` instead of `@user/...`)
- **WebSocket Events**: Fixed event parsing to match actual API payloads:
  - `OnlineCount` now correctly reads `authorizedUsers` field
  - `StatusUpdate` now correctly reads `statusSetAt` field
  - Added `OrderUpdated` and `OrderRemoved` event handling
- **Subscription Payloads**: Subscriptions now always include `platform` field (defaults to "pc")

### Added

- **Configurable User-Agent**: `WebSocketBuilder::user_agent()` method to set custom User-Agent header
- **`DEFAULT_USER_AGENT`**: Exported constant for reference when building custom User-Agent strings
- **JWT Token Management**: `update_token` example to fetch and cache JWT tokens in `.env` file
- **Integration Tests**: WebSocket integration tests with `.env` file support
  - `test_ws_connect_and_authenticate` - Verify TLS and authentication
  - `test_ws_receives_online_count` - Verify event reception
  - `test_ws_subscribe_new_orders` - Verify subscription flow
  - `test_ws_set_status` - Verify status commands
  - `test_ws_dynamic_subscription` - Verify dynamic subscription/unsubscription
- **Test Utilities**: `tests/common/mod.rs` with credential loading and error handling
- **`AuthError`**: Detailed error types for authentication failures (token expired, rate limited, etc.)
- **Example Config**: `.env.example` file for test credentials

### Changed

- WebSocket User-Agent now uses `CARGO_PKG_VERSION` to stay in sync with crate version
- WebSocket example now supports loading credentials from `.env` file
- Integration tests prefer JWT token over email/password to avoid rate limiting

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
