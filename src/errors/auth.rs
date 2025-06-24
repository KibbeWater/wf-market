#[derive(Debug, Eq, PartialEq)]
pub enum AuthError {
    NoUser,
    ParsingError(String),
    Unknown(String),
}
