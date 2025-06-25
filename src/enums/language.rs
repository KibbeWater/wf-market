use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Copy, Deserialize, Debug, Eq, PartialEq)]
pub enum Language {
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "pt")]
    Portuguese,
    #[serde(rename = "zh-hans")]
    ChineseSimplified,
    #[serde(rename = "zh-hant")]
    ChineseTraditional,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "pl")]
    Polish,
    #[serde(rename = "uk")]
    Ukrainian,
    #[serde(rename = "en")]
    English,
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl Language {
    pub fn as_str(&self) -> &str {
        match self {
            Language::Korean => "ko",
            Language::Russian => "ru",
            Language::German => "de",
            Language::French => "fr",
            Language::Portuguese => "pt",
            Language::ChineseSimplified => "zh-hans",
            Language::ChineseTraditional => "zh-hant",
            Language::Spanish => "es",
            Language::Italian => "it",
            Language::Polish => "pl",
            Language::Ukrainian => "uk",
            Language::English => "en",
        }
    }
}