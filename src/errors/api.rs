use crate::errors::ResponseError;

#[derive(Debug, Eq, PartialEq)]
pub enum ApiError {
    ParsingError(String),
    RequestError,
    Unauthorized,
    NotFound(String),
    Forbidden,
    WFMError(ResponseError),
    Unknown(String),
    InvalidType { expected: String, found: String },
}
