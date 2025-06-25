use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Npc {
    pub id: String,
    pub slug: String,
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n: Option<HashMap<String, NpcTranslation>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpcTranslation {
    pub name: String,
    pub icon: String,
    pub thumb: String,
}
