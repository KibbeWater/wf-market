//! Transaction models for warframe.market.
//!
//! Transactions represent completed trades when orders are closed.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::common::OrderType;

/// A completed transaction from closing an order.
#[derive(Debug, Clone, Deserialize)]
pub struct Transaction {
    /// Unique transaction identifier
    pub id: String,

    /// Order type (buy or sell)
    #[serde(rename = "type")]
    pub order_type: OrderType,

    /// Price per unit in platinum
    pub platinum: u32,

    /// Quantity traded
    pub quantity: u32,

    /// ID of the item traded
    #[serde(rename = "itemId")]
    pub item_id: String,

    /// When the transaction occurred
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    /// Get the total platinum value of this transaction.
    pub fn total_value(&self) -> u64 {
        self.platinum as u64 * self.quantity as u64
    }

    /// Check if this was a sale (you sold items).
    pub fn is_sale(&self) -> bool {
        matches!(self.order_type, OrderType::Sell)
    }

    /// Check if this was a purchase (you bought items).
    pub fn is_purchase(&self) -> bool {
        matches!(self.order_type, OrderType::Buy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_total_value() {
        let tx = Transaction {
            id: "tx-1".to_string(),
            order_type: OrderType::Sell,
            platinum: 50,
            quantity: 3,
            item_id: "item-1".to_string(),
            created_at: Utc::now(),
        };

        assert_eq!(tx.total_value(), 150);
        assert!(tx.is_sale());
        assert!(!tx.is_purchase());
    }
}
