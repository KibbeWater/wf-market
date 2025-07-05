#[derive(Debug, Eq, PartialEq)]
pub enum AuthError {
    NoUser,
    ParsingError(String),
    InvalidCredentials(String),
    Unknown(String),
}
