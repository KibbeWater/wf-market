use serde::{Deserialize, Serialize};

use crate::{enums::*, types::*};

pub struct Owned;
#[derive(Clone)]
pub struct Unowned;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Order {
    pub id: String,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub platinum: u32, // AKA the price
    pub quantity: u32,

    #[serde(rename = "perTrade", skip_serializing_if = "Option::is_none")]
    pub per_trade: Option<u8>, // Amount of items per trade

    #[serde(flatten)]
    pub subtype: SubType, // Subtype for mods, ayatan sculptures, etc.

    pub visible: bool, // Whether the order is visible to other players

    #[serde(rename = "itemId")]
    pub item_id: String, // ID of the item

    #[serde(rename = "createdAt")]
    pub created_at: String, // Timestamp of when the order was created
    #[serde(rename = "updatedAt")]
    pub updated_at: String, // Timestamp of when the order was last updated

    pub properties: Option<serde_json::Value>, // Additional properties for the order
}

impl Default for Order {
    fn default() -> Self {
        Self {
            id: String::from("N/A"),
            order_type: OrderType::Buy,
            platinum: 0,
            quantity: 0,
            per_trade: None,
            subtype: SubType::default(),
            visible: true,
            item_id: String::new(),
            created_at: String::new(),
            updated_at: String::new(),
            properties: None,
        }
    }
}

impl Order {
    pub fn set_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = Some(properties);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderWithUser {
    #[serde(flatten)]
    pub order: Order,

    pub user: UserShort,
}

impl OrderWithUser {
    pub fn downgrade(&self) -> Order {
        let o = self.order.clone();
        Order {
            id: o.id,
            per_trade: o.per_trade,
            order_type: o.order_type,
            platinum: o.platinum,
            quantity: o.quantity,
            subtype: o.subtype,
            visible: o.visible,
            created_at: o.created_at,
            updated_at: o.updated_at,
            item_id: o.item_id,
            properties: o.properties,
        }
    }
}
