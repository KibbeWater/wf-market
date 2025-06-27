use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct RivenAttribute {
    pub id: String,
    pub slug: String,
    #[serde(rename = "gameRef")]
    pub game_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    pub prefix: String,
    pub suffix: String,
    #[serde(rename = "exclusiveTo", skip_serializing_if = "Option::is_none")]
    pub exclusive_to: Option<Vec<String>>,
    #[serde(rename = "positiveIsNegative", skip_serializing_if = "Option::is_none")]
    pub positive_is_negative: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(rename = "positiveOnly", skip_serializing_if = "Option::is_none")]
    pub positive_only: Option<bool>,
    #[serde(rename = "negativeOnly", skip_serializing_if = "Option::is_none")]
    pub negative_only: Option<bool>,
    #[serde(default = "HashMap::new")]
    pub i18n: HashMap<String, RivenAttributeTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RivenAttributeTranslation {
    #[serde(rename = "name")]
    pub name: String,
    pub icon: String,
    pub thumb: String,
}
