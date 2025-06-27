use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct LichQuirk {
    pub id: String,
    pub slug: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default = "HashMap::new")]
    pub i18n: HashMap<String, LichQuirkTranslation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LichQuirkTranslation {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb: Option<String>,
}
