use serde::Deserialize;

use crate::enums::*;

#[derive(Deserialize, Clone, Debug)]
pub struct UserShort {
    pub id: String,
    #[serde(rename = "ingame_name", alias = "ingameName")]
    pub name: String,
    pub reputation: i32,
    #[serde(rename = "status", default = "StatusType::default")]
    pub status_type: StatusType,
}
