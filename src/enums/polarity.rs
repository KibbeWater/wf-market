use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Copy, Deserialize, Debug, Eq, PartialEq)]
pub enum Polarity {
    #[serde(rename = "madurai")]
    Madurai,
    #[serde(rename = "vazarin")]
    Vazarin,
    #[serde(rename = "naramon")]
    Naramon,
    #[serde(rename = "zenurik")]
    Zenurik,
}

impl Polarity {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "madurai" => Some(Polarity::Madurai),
            "vazarin" => Some(Polarity::Vazarin),
            "naramon" => Some(Polarity::Naramon),
            "zenurik" => Some(Polarity::Zenurik),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Polarity::Madurai => "madurai".to_string(),
            Polarity::Vazarin => "vazarin".to_string(),
            Polarity::Naramon => "naramon".to_string(),
            Polarity::Zenurik => "zenurik".to_string(),
        }
    }
}

impl Default for Polarity {
    fn default() -> Self {
        Polarity::Madurai
    }
}
