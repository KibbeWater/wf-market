use std::collections::HashMap;

use serde::Deserialize;
#[derive(Debug, Eq, PartialEq)]
pub enum AuthError {
    NoUser,
    ParsingError,
    Unknown(String),
}

#[derive(Debug, Eq, PartialEq)]
pub enum ApiError {
    ParsingError(String),
    RequestError,
    Unauthorized,
    NotFound(String),
    Forbidden,
    WFMError(ErrorResponse),
    Unknown(String),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct ErrorResponse {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub data: Option<serde_json::Value>,
    pub error: ApiErrorBody,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct ApiErrorBody {
    pub request: Option<Vec<String>>,
    pub inputs: Option<HashMap<String, String>>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum WsError {
    ReservedPath(String),
    InvalidPath(String),
    AlreadyRegistered(String),
    InvalidMessageReceived(String),
    ConnectionError,
    InvalidMessage,
    SendError(String),
    NotConnected,
}
