use serde_json::Error;

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
