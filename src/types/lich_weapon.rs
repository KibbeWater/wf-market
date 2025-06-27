use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct LichWeapon {
    pub id: String,
    pub slug: String,
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    #[serde(rename = "reqMasteryRank")]
    pub req_mastery_rank: i8,
    #[serde(default = "HashMap::new")]
    pub i18n: HashMap<String, LichWeaponTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LichWeaponTranslation {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "wikiLink", skip_serializing_if = "Option::is_none")]
    pub wiki_link: Option<String>,
    pub icon: String,
    pub thumb: String,
}
