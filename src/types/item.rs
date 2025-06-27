use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Item {
    pub id: String,
    #[serde(default = "Vec::new")]
    pub tags: Vec<String>,
    pub slug: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tradable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vaulted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ducats: Option<u32>,

    // MODS
    #[serde(rename = "maxRank", skip_serializing_if = "Option::is_none")]
    pub max_rank: Option<u32>,
    #[serde(rename = "maxCharges", skip_serializing_if = "Option::is_none")]
    pub max_charges: Option<u32>,

    // AYATAN SCULPTURES
    #[serde(rename = "maxAmberStars", skip_serializing_if = "Option::is_none")]
    pub max_amber_stars: Option<u32>,
    #[serde(rename = "maxCyanStars", skip_serializing_if = "Option::is_none")]
    pub max_cyan_stars: Option<u32>,
    #[serde(rename = "baseEndo", skip_serializing_if = "Option::is_none")]
    pub base_endo: Option<u32>,
    #[serde(rename = "endoMultiplier", skip_serializing_if = "Option::is_none")]
    pub endo_multiplier: Option<f32>,

    #[serde(rename = "reqMasteryRank", skip_serializing_if = "Option::is_none")]
    pub mastery_rank: Option<u32>,
    #[serde(default = "HashMap::new")]
    pub i18n: HashMap<String, ItemTranslation>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemTranslation {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "wikiLink", skip_serializing_if = "Option::is_none")]
    pub wiki_link: Option<String>,
    pub icon: String,
}
