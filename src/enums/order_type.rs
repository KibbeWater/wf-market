use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Copy, Deserialize, Debug, Eq, PartialEq)]
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
