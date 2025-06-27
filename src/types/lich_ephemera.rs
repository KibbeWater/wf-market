use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct LichEphemera {
    pub id: String,
    pub slug: String,
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    pub animation: String,
    pub element: String,
    #[serde(default = "HashMap::new")]
    pub i18n: HashMap<String, LichEphemeraTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LichEphemeraTranslation {
    #[serde(rename = "name")]
    pub name: String,
    pub icon: String,
    pub thumb: String,
}
