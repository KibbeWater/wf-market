use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Riven {
    pub id: String,
    pub slug: String,
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(rename = "rivenType", skip_serializing_if = "Option::is_none")]
    pub riven_type: Option<String>,
    pub disposition: f64,
    #[serde(rename = "reqMasteryRank")]
    pub req_mastery_rank: i8,
    #[serde(default = "HashMap::new")]
    pub i18n: HashMap<String, RivenTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RivenTranslation {
    #[serde(rename = "itemName", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "wikiLink", skip_serializing_if = "Option::is_none")]
    pub wiki_link: Option<String>,
    pub icon: String,
    pub thumb: String,
}
