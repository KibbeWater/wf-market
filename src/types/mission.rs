use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Mission {
    pub id: String,
    pub slug: String,
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    #[serde(default)]
    pub i18n: HashMap<String, MissionTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MissionTranslation {
    pub name: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub thumb: String,
}
