use serde_json::{Error, json};
use std::fmt::{Display, Formatter};

use crate::errors::*;

#[derive(Debug)]
pub enum ApiError {
    TooManyRequests(RequestError),
    RequestError(RequestError),
    Unauthorized(RequestError),
    ParsingError(RequestError, Error),
    NotFound(RequestError),
    BadRequest(RequestError),
    InvalidCredentials(RequestError),
    Forbidden(RequestError),
    EndOfFile(RequestError),
    InternalServerError(RequestError),
    OrderLimitExceeded(RequestError),
    OrderLimitExceededSamePrice(RequestError),
    AuctionLimitExceeded(RequestError),
    InvalidType { expected: String, found: String },
    Unknown(String),
}
impl ApiError {
    pub fn error_type(&self) -> &'static str {
        match self {
            Self::TooManyRequests(_) => "TooManyRequests",
            Self::RequestError(_) => "RequestError",
            Self::Unauthorized(_) => "Unauthorized",
            Self::ParsingError(_, _) => "ParsingError",
            Self::NotFound(_) => "NotFound",
            Self::BadRequest(_) => "BadRequest",
            Self::InvalidCredentials(_) => "InvalidCredentials",
            Self::Forbidden(_) => "Forbidden",
            Self::EndOfFile(_) => "EndOfFile",
            Self::InternalServerError(_) => "InternalServerError",
            Self::OrderLimitExceeded(_) => "OrderLimitExceeded",
            Self::OrderLimitExceededSamePrice(_) => "OrderLimitExceededSamePrice",
            Self::AuctionLimitExceeded(_) => "AuctionLimitExceeded",
            Self::InvalidType { .. } => "InvalidType",
            Self::Unknown(_) => "Unknown",
        }
    }
    pub fn mask_sensitive_data(&mut self, properties: &[&str]) {
        match self {
            ApiError::TooManyRequests(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::RequestError(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::Unauthorized(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::ParsingError(req_err, _) => req_err.mask_sensitive_data(properties),
            ApiError::NotFound(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::InternalServerError(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::BadRequest(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::InvalidCredentials(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::Forbidden(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::OrderLimitExceeded(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::OrderLimitExceededSamePrice(req_err) => {
                req_err.mask_sensitive_data(properties)
            }
            ApiError::AuctionLimitExceeded(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::EndOfFile(req_err) => req_err.mask_sensitive_data(properties),
            ApiError::InvalidType { .. } | ApiError::Unknown(_) => {}
        }
    }
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            ApiError::TooManyRequests(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::RequestError(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::Unauthorized(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::ParsingError(req_err, parse_err) => json!({
                "type": self.error_type(),
                "error": req_err,
                "parse_error": parse_err.to_string(),
            }),
            ApiError::NotFound(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::BadRequest(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::InvalidCredentials(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::Forbidden(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::InternalServerError(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::Unknown(_) => json!({
                "type": self.error_type(),
                "message": "An unknown error occurred."
            }),
            ApiError::InvalidType { expected, found } => json!({
                "type": self.error_type(),
                "expected": expected,
                "found": found,
            }),
            ApiError::OrderLimitExceeded(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::OrderLimitExceededSamePrice(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::AuctionLimitExceeded(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
            ApiError::EndOfFile(req_err) => json!({
                "type": self.error_type(),
                "error": req_err,
            }),
        }
    }
}
impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::TooManyRequests(req_err) => {
                write!(f, "Too many requests: {}", req_err.error_sentence())
            }
            ApiError::RequestError(req_err) => {
                write!(f, "Request error: {}", req_err.error_sentence())
            }
            ApiError::Unauthorized(req_err) => {
                write!(f, "Unauthorized: {}", req_err.error_sentence())
            }
            ApiError::ParsingError(req_err, parse_err) => {
                write!(
                    f,
                    "Parsing error: {} - {}",
                    req_err.error_sentence(),
                    parse_err
                )
            }
            ApiError::NotFound(req_err) => {
                write!(f, "Not found: {}", req_err.error_sentence())
            }
            ApiError::BadRequest(req_err) => {
                write!(f, "Bad request: {}", req_err.error_sentence())
            }
            ApiError::InvalidCredentials(req_err) => {
                write!(f, "Invalid credentials: {}", req_err.error_sentence())
            }
            ApiError::Forbidden(req_err) => {
                write!(f, "Forbidden: {}", req_err.error_sentence())
            }
            ApiError::Unknown(msg) => {
                write!(f, "Unknown error: {}", msg)
            }
            ApiError::InvalidType { expected, found } => {
                write!(
                    f,
                    "Invalid type: expected '{}', found '{}'",
                    expected, found
                )
            }
            ApiError::InternalServerError(req_err) => {
                write!(f, "Internal server error: {}", req_err.error_sentence())
            }
            ApiError::OrderLimitExceeded(req_err) => {
                write!(f, "Order limit exceeded: {}", req_err.error_sentence())
            }
            ApiError::OrderLimitExceededSamePrice(req_err) => {
                write!(
                    f,
                    "Order limit exceeded (same price): {}",
                    req_err.error_sentence()
                )
            }
            ApiError::AuctionLimitExceeded(req_err) => {
                write!(f, "Auction limit exceeded: {}", req_err.error_sentence())
            }
            ApiError::EndOfFile(req_err) => {
                write!(f, "End of file: {}", req_err.error_sentence())
            }
        }
    }
}
