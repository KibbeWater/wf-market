#[derive(Debug)]
pub enum AuthError {
    NoUser,
    ParsingError,
    Unknown(String),
}

#[derive(Debug)]
pub enum ApiError {
    ParsingError(String),
    RequestError,
    Unauthorized,
    Unknown(String),
}

#[derive(Debug)]
pub enum WsError {
    ReservedPath(String),
    InvalidPath(String),
    AlreadyRegistered(String),
    InvalidMessageReceived(String),
    ConnectionError,
    InvalidMessage,
    SendError,
    NotConnected,
}
