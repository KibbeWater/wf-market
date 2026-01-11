//! V1 API endpoints (deprecated).
//!
//! This module contains endpoints from the legacy V1 API that have not
//! yet been migrated to V2. These endpoints will be removed when the
//! V1 API is fully deprecated by warframe.market.
//!
//! # Feature Flag
//!
//! This module requires the `v1-api` feature flag:
//!
//! ```toml
//! [dependencies]
//! wf-market = { version = "0.3", features = ["v1-api"] }
//! ```
//!
//! # Available Endpoints
//!
//! - [`get_item_statistics`](crate::Client::get_item_statistics) - Get trading statistics for an item

mod statistics;

use serde::Deserialize;

/// V1 API response wrapper.
///
/// V1 endpoints wrap response data in a `payload` field instead of
/// V2's `data` field.
#[derive(Debug, Deserialize)]
pub(crate) struct V1ApiResponse<T> {
    pub payload: T,
}
