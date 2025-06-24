use serde::Deserialize;

use crate::{enums::*, types::*};

pub struct Owned;
#[derive(Clone)]
pub struct Unowned;

#[derive(Clone, Deserialize, Debug)]
pub struct Order {
    pub id: String,
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub platinum: u32, // AKA the price
    pub quantity: u32,

    #[serde(rename = "perTrade", skip_serializing_if = "Option::is_none")]
    pub per_trade: Option<u8>, // Amount of items per trade
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>, // Subtype of the item, if applicable

    // MODS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>, // Rank of the mod, if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charges: Option<u8>, // Charges remaining (Requiem mods)

    // AYATAN SCULPTURES
    #[serde(rename = "amberStars", skip_serializing_if = "Option::is_none")]
    pub amber_stars: Option<u8>, // Number of Amber Stars, if applicable
    #[serde(rename = "cyanStars", skip_serializing_if = "Option::is_none")]
    pub cyan_stars: Option<u8>, // Number of Cyan Stars, if applicable

    pub visible: bool, // Whether the order is visible to other players

    #[serde(rename = "itemId")]
    pub item_id: String, // ID of the item

    #[serde(rename = "createdAt")]
    pub created_at: String, // Timestamp of when the order was created
    #[serde(rename = "updatedAt")]
    pub updated_at: String, // Timestamp of when the order was last updated
}

#[derive(Debug, Clone, Deserialize)]
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
            order_type: o.order_type,
            platinum: o.platinum,
            quantity: o.quantity,
            per_trade: o.per_trade,
            subtype: o.subtype,
            rank: o.rank,
            charges: o.charges,
            amber_stars: o.amber_stars,
            cyan_stars: o.cyan_stars,
            visible: o.visible,
            created_at: o.created_at,
            updated_at: o.updated_at,
            item_id: o.item_id,
        }
    }
}
