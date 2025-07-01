use serde::{Deserialize, Serialize};

use crate::{
    enums::{AuctionType, Polarity},
    errors::ApiError,
    types::UserShort,
};

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Auction {
    pub id: String,
    pub minimal_reputation: i32,
    #[serde(rename = "winner")]
    pub winner_id: Option<String>,
    pub private: bool,
    pub visible: bool,
    pub note_raw: String,
    pub note: String,
    pub starting_price: i32,
    pub buyout_price: Option<i32>,
    pub is_direct_sell: bool,
    pub top_bid: Option<i32>,
    pub created: String,
    pub updated: String,
    pub platform: String,
    pub closed: bool,
    pub is_marked_for: Option<String>,
    pub marked_operation_at: Option<String>,
    pub item: AuctionItem,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AuctionWithOwner {
    #[serde(flatten)]
    pub auction: Auction,

    pub owner: UserShort,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct AuctionItem {
    #[serde(rename = "type")]
    pub item_type: AuctionType,

    pub weapon_url_name: String,

    // RIVEN
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub mod_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<ItemAttribute>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub re_rolls: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mastery_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mod_rank: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polarity: Option<String>,

    // SISTER / LICH
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quirk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub having_ephemera: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemAttribute {
    pub url_name: String,
    pub positive: bool,
    pub value: f64,
}

impl ItemAttribute {
    pub fn new(url_name: &str, positive: bool, value: f64) -> Self {
        Self {
            url_name: url_name.to_string(),
            positive,
            value,
        }
    }
}
