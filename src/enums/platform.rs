use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Copy, Deserialize, Debug, Eq, PartialEq)]
pub enum Platform {
    #[serde(rename = "pc")]
    Pc,
    #[serde(rename = "ps4")]
    Ps4,
    #[serde(rename = "xbox")]
    Xbox,
    #[serde(rename = "switch")]
    Switch,
    #[serde(rename = "mobile")]
    Mobile,
}

impl Default for Platform {
    fn default() -> Self {
        Platform::Pc
    }
}

impl Platform {
    pub fn as_str(&self) -> &str {
        match self {
            Platform::Pc => "pc",
            Platform::Ps4 => "ps4",
            Platform::Xbox => "xbox",
            Platform::Switch => "switch",
            Platform::Mobile => "mobile",
        }
    }
}