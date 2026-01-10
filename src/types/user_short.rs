use serde::{Deserialize, Serialize};

use crate::enums::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UserShort {
    pub id: String,
    #[serde(rename = "ingame_name", alias = "ingameName")]
    pub name: String,
    /// Optional link to the user's avatar image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    pub reputation: f64,
    #[serde(rename = "status", default = "StatusType::default")]
    pub status: StatusType,
}
