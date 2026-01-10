//! Error types for the wf-market library.
//!
//! This module provides a unified error type that covers all possible
//! failure modes when interacting with the warframe.market API.

use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

/// The main error type for all wf-market operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Authentication failed (invalid credentials, expired token, etc.)
    #[error("Authentication failed: {message}")]
    Auth {
        message: String,
        /// Detailed error response from the API, if available
        details: Option<Box<ApiErrorResponse>>,
    },

    /// API returned an error response
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code
        status: StatusCode,
        /// Human-readable error message
        message: String,
        /// Full error response from WFM API for debugging
        response: Option<ApiErrorResponse>,
    },

    /// Requested resource was not found
    #[error("Resource not found: {resource}")]
    NotFound {
        /// Description of the resource that wasn't found
        resource: String,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimited {
        /// Duration to wait before retrying, if provided by server
        retry_after: Option<std::time::Duration>,
    },

    /// Network/connection error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// Failed to parse API response
    #[error("Failed to parse response: {message}")]
    Parse {
        /// Description of what failed to parse
        message: String,
        /// Raw response body for debugging
        body: Option<String>,
    },

    /// WebSocket error (only available with `websocket` feature)
    #[cfg(feature = "websocket")]
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] WsError),

    /// Invalid configuration or usage
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// WebSocket-specific errors.
#[cfg(feature = "websocket")]
#[derive(Debug, Error)]
pub enum WsError {
    /// Failed to connect to WebSocket server
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Connection was closed unexpectedly
    #[error("Connection closed: {reason}")]
    Disconnected { reason: String },

    /// Failed to send message
    #[error("Failed to send message: {0}")]
    SendFailed(String),

    /// Received invalid message from server
    #[error("Invalid message received: {0}")]
    InvalidMessage(String),

    /// Attempted to use reserved route
    #[error("Route is reserved for internal use: {0}")]
    ReservedRoute(String),

    /// Not connected to server
    #[error("Not connected to WebSocket server")]
    NotConnected,

    /// Authentication failed on WebSocket
    #[error("WebSocket authentication failed: {0}")]
    AuthFailed(String),
}

/// Structured error response from the warframe.market API.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    /// API version that generated the error
    #[serde(rename = "apiVersion")]
    pub api_version: String,

    /// Error details
    pub error: ApiErrorDetails,
}

/// Detailed error information from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorDetails {
    /// General request-level errors
    #[serde(default)]
    pub request: Option<Vec<String>>,

    /// Field-specific validation errors (field name -> error messages)
    #[serde(default)]
    pub inputs: Option<HashMap<String, Vec<String>>>,
}

impl ApiErrorResponse {
    /// Get all request-level error messages.
    pub fn request_errors(&self) -> Vec<&str> {
        self.error
            .request
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get error messages for a specific input field.
    pub fn field_errors(&self, field: &str) -> Vec<&str> {
        self.error
            .inputs
            .as_ref()
            .and_then(|m| m.get(field))
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Check if there are any validation errors.
    pub fn has_validation_errors(&self) -> bool {
        self.error.inputs.as_ref().is_some_and(|m| !m.is_empty())
    }
}

impl Error {
    /// Get detailed API error information if available.
    pub fn api_details(&self) -> Option<&ApiErrorResponse> {
        match self {
            Error::Auth { details, .. } => details.as_ref().map(|b| b.as_ref()),
            Error::Api { response, .. } => response.as_ref(),
            _ => None,
        }
    }

    /// Check if this error is potentially retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Error::RateLimited { .. } | Error::Network(_))
    }

    /// Check if this is an authentication error.
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Error::Auth { .. })
            || matches!(self, Error::Api { status, .. } if *status == StatusCode::UNAUTHORIZED)
    }

    /// Create an authentication error.
    pub(crate) fn auth(message: impl Into<String>) -> Self {
        Error::Auth {
            message: message.into(),
            details: None,
        }
    }

    /// Create an authentication error with details.
    pub(crate) fn auth_with_details(message: impl Into<String>, details: ApiErrorResponse) -> Self {
        Error::Auth {
            message: message.into(),
            details: Some(Box::new(details)),
        }
    }

    /// Create an API error.
    pub(crate) fn api(status: StatusCode, message: impl Into<String>) -> Self {
        Error::Api {
            status,
            message: message.into(),
            response: None,
        }
    }

    /// Create an API error with response details.
    pub(crate) fn api_with_response(
        status: StatusCode,
        message: impl Into<String>,
        response: ApiErrorResponse,
    ) -> Self {
        Error::Api {
            status,
            message: message.into(),
            response: Some(response),
        }
    }

    /// Create a not found error.
    pub(crate) fn not_found(resource: impl Into<String>) -> Self {
        Error::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a parse error.
    #[allow(dead_code)]
    pub(crate) fn parse(message: impl Into<String>) -> Self {
        Error::Parse {
            message: message.into(),
            body: None,
        }
    }

    /// Create a parse error with the raw body.
    pub(crate) fn parse_with_body(message: impl Into<String>, body: String) -> Self {
        Error::Parse {
            message: message.into(),
            body: Some(body),
        }
    }
}

/// A specialized Result type for wf-market operations.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_is_retryable() {
        assert!(Error::RateLimited { retry_after: None }.is_retryable());
        assert!(!Error::auth("test").is_retryable());
        assert!(!Error::not_found("item").is_retryable());
    }

    #[test]
    fn test_error_is_auth_error() {
        assert!(Error::auth("test").is_auth_error());
        assert!(
            Error::Api {
                status: StatusCode::UNAUTHORIZED,
                message: "test".into(),
                response: None
            }
            .is_auth_error()
        );
        assert!(!Error::not_found("item").is_auth_error());
    }

    #[test]
    fn test_api_error_response_helpers() {
        let response = ApiErrorResponse {
            api_version: "2.0".into(),
            error: ApiErrorDetails {
                request: Some(vec!["General error".into()]),
                inputs: Some(HashMap::from([(
                    "email".into(),
                    vec!["Invalid email".into()],
                )])),
            },
        };

        assert_eq!(response.request_errors(), vec!["General error"]);
        assert_eq!(response.field_errors("email"), vec!["Invalid email"]);
        assert!(response.field_errors("password").is_empty());
        assert!(response.has_validation_errors());
    }
}
