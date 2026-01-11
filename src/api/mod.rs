//! API endpoint implementations.
//!
//! This module contains the actual API calls organized by resource type.

mod items;
mod orders;
mod rivens;
mod users;

// V1 API endpoints (deprecated, feature-gated)
#[cfg(feature = "v1-api")]
mod v1;

// Response wrapper types used by multiple endpoints
use serde::Deserialize;

/// Standard API response wrapper.
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    #[serde(rename = "apiVersion")]
    #[allow(dead_code)]
    pub api_version: Option<String>,
    pub data: T,
}
