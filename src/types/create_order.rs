use serde::Serialize;

use crate::enums::OrderType;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderParams {
    pub item_id: String,
    #[serde(rename = "type", default)]
    pub order_type: OrderType,
    pub platinum: i32,
    pub quantity: i32,
    pub visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "perTrade")]
    pub per_trade: Option<i32>, // Minimum number of items per transaction

    // MODS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charges: Option<u8>,

    // VARIANTS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,

    // AYATAN SCULPTURES
    #[serde(rename = "amberStars", skip_serializing_if = "Option::is_none")]
    pub amber_stars: Option<u32>,
    #[serde(rename = "cyanStars", skip_serializing_if = "Option::is_none")]
    pub cyan_stars: Option<u32>,
}

impl CreateOrderParams {
    pub fn new(
        item_id: &str,
        order_type: OrderType,
        platinum: i32,
        quantity: i32,
        visible: bool,
    ) -> Self {
        CreateOrderParams {
            item_id: item_id.to_string(),
            order_type,
            platinum,
            quantity,
            visible,
            per_trade: None,
            rank: None,
            charges: None,
            subtype: None,
            amber_stars: None,
            cyan_stars: None,
        }
    }

    pub fn with_rank(mut self, rank: u8) -> Self {
        self.rank = Some(rank);
        self
    }
    pub fn with_subtype(mut self, subtype: String) -> Self {
        self.subtype = Some(subtype);
        self
    }
    pub fn with_ayatans(mut self, amber_stars: u32, cyan_stars: u32) -> Self {
        self.amber_stars = Some(amber_stars);
        self.cyan_stars = Some(cyan_stars);
        self
    }
    pub fn with_charges(mut self, charges: u8) -> Self {
        self.charges = Some(charges);
        self
    }
    pub fn with_per_trade(mut self, per_trade: i32) -> Self {
        self.per_trade = Some(per_trade);
        self
    }
}
