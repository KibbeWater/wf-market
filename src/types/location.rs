use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    pub id: String,
    pub slug: String,
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faction: Option<String>,
    #[serde(rename = "minLevel", skip_serializing_if = "Option::is_none")]
    pub min_level: Option<i32>,
    #[serde(rename = "maxLevel", skip_serializing_if = "Option::is_none")]
    pub max_level: Option<i32>,
    #[serde(default)]
    pub i18n: HashMap<String, LocationTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocationTranslation {
    #[serde(rename = "nodeName")]
    pub node_name: String,
    #[serde(rename = "systemName", default)]
    pub system_name: String,
    pub icon: String,
    pub thumb: String,
}
