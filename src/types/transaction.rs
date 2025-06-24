use serde::{Deserialize, Serialize};

use crate::types::UserShort;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub id: String,
    #[serde(rename = "type")]
    pub order_type: String,
    #[serde(rename = "originId")]
    pub origin_id: String,
    pub platinum: u32,
    pub quantity: u32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub item: TransactionItem,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TransactionItem {
    pub id: String,
    pub rank: Option<i32>,
    pub charges: Option<i32>,
    pub subtype: Option<String>,
    #[serde(rename = "amberStars")]
    pub amber_stars: Option<i32>,
    #[serde(rename = "cyanStars")]
    pub cyan_stars: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct TransactionWithUser {
    #[serde(flatten)]
    pub transaction: Transaction,
    pub user: UserShort,
}

impl TransactionWithUser {
    pub fn downgrade(&self) -> Transaction {
        let t = self.transaction.clone();
        Transaction {
            id: t.id,
            order_type: t.order_type,
            origin_id: t.origin_id,
            platinum: t.platinum,
            quantity: t.quantity,
            created_at: t.created_at,
            updated_at: t.updated_at,
            item: t.item,
        }
    }
}
