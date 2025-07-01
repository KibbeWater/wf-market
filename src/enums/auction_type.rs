use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Copy, Deserialize, Debug, Eq, PartialEq)]
pub enum AuctionType {
    #[serde(rename = "riven")]
    Riven,
    #[serde(rename = "lich")]
    Lich,
    #[serde(rename = "sister")]
    Sister,
}

impl AuctionType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "riven" => Some(AuctionType::Riven),
            "lich" => Some(AuctionType::Lich),
            "sister" => Some(AuctionType::Sister),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            AuctionType::Riven => "riven",
            AuctionType::Lich => "lich",
            AuctionType::Sister => "sister",
        }
    }
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl Default for AuctionType {
    fn default() -> Self {
        AuctionType::Riven
    }
}
