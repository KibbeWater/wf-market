use serde::Serialize;

use crate::{enums::*, types::*};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderParams {
    pub item_id: String,
    #[serde(rename = "type", default)]
    pub order_type: OrderType,
    pub platinum: u32,
    pub quantity: u32,
    pub visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "perTrade")]
    pub per_trade: Option<u32>, // Minimum number of items per transaction

    #[serde(flatten)]
    pub subtype: Option<SubType>, // Subtype for mods, ayatan sculptures, etc.

    #[serde(skip_serializing)]
    pub properties: Option<serde_json::Value>, // Additional properties for the order
}

impl CreateOrderParams {
    pub fn new(
        item_id: &str,
        order_type: OrderType,
        platinum: u32,
        quantity: u32,
        visible: bool,
        per_trade: Option<u32>,
    ) -> Self {
        CreateOrderParams {
            item_id: item_id.to_string(),
            per_trade,
            order_type,
            platinum,
            quantity,
            visible,
            subtype: None,
            properties: None,
        }
    }
    pub fn new_with_subtype(
        item_id: &str,
        order_type: OrderType,
        platinum: u32,
        quantity: u32,
        visible: bool,
        per_trade: Option<u32>,
        subtype: SubType,
    ) -> Self {
        let mut order =
            CreateOrderParams::new(item_id, order_type, platinum, quantity, visible, per_trade);
        order.subtype = Some(subtype);
        order
    }
    pub fn with_subtype(mut self, subtype: SubType) -> Self {
        self.subtype = Some(subtype);
        self
    }
    pub fn with_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = Some(properties);
        self
    }
}
