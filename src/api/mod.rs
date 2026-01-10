//! API endpoint implementations.
//!
//! This module contains the actual API calls organized by resource type.

mod items;
mod orders;
mod rivens;

// Response wrapper types used by multiple endpoints
use serde::Deserialize;

/// Standard API response wrapper.
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    #[serde(rename = "apiVersion")]
    pub api_version: Option<String>,
    pub data: T,
}
