use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, Copy, Deserialize, Debug, Eq, PartialEq)]
pub enum OrderType {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}
