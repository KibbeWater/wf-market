use serde::{Deserialize, Serialize};

use crate::enums::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserShort {
    pub id: String,
    #[serde(rename = "ingame_name", alias = "ingameName")]
    pub name: String,
    pub reputation: i32,
    #[serde(rename = "status", default = "StatusType::default")]
    pub status: StatusType,
}
