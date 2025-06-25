use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Mission {
    pub id: String,
    pub slug: String,
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub i18n: Option<HashMap<String, MissionTranslation>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MissionTranslation {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<String>,
}
