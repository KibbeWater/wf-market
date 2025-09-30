use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OrderType {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

impl OrderType {
    pub fn to_string(&self) -> String {
        match self {
            OrderType::Buy => "buy".into(),
            OrderType::Sell => "sell".into(),
        }
    }
}
