use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct SisterEphemera {
    pub id: String,
    pub slug: String,
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    pub animation: String,
    pub element: String,
    #[serde(default = "HashMap::new")]
    pub i18n: HashMap<String, SisterEphemeraTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SisterEphemeraTranslation {
    pub name: String,
    pub icon: String,
    pub thumb: String,
}
