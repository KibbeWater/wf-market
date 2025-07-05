use serde_json::Error;
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
    Unknown(String),
    InvalidType { expected: String, found: String },
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
        }
    }
}
