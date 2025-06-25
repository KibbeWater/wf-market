use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "moderator")]
    Moderator,
    #[serde(rename = "admin")]
    Admin,
}