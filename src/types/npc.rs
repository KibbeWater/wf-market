use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Npc {
    pub id: String,
    pub slug: String,
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    #[serde(default)]
    pub i18n: HashMap<String, NpcTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpcTranslation {
    pub name: String,
    pub icon: String,
    pub thumb: String,
}
