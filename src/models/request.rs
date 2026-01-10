//! Request models for creating and updating orders.
//!
//! This module provides builder-pattern types for constructing
//! API requests with compile-time safety.

use serde::Serialize;

use super::common::OrderType;

/// Request to create a new order.
///
/// Use the builder methods to construct orders with proper validation.
///
/// # Example
///
/// ```
/// use wf_market::{CreateOrder, OrderType};
///
/// // Simple sell order
/// let order = CreateOrder::sell("nikana_prime_set", 100, 1);
///
/// // Mod order with rank
/// let mod_order = CreateOrder::sell("serration", 50, 1)
///     .with_mod_rank(10);
///
/// // Hidden order
/// let hidden = CreateOrder::buy("mesa_prime_set", 200, 1)
///     .hidden();
///
/// // Sculpture with stars
/// let sculpture = CreateOrder::sell("ayatan_anasa_sculpture", 25, 1)
///     .with_sculpture_stars(2, 4);
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrder {
    /// ID of the item to trade
    pub item_id: String,

    /// Order type (buy or sell)
    #[serde(rename = "type")]
    pub order_type: OrderType,

    /// Price in platinum
    pub platinum: u32,

    /// Quantity to trade
    pub quantity: u32,

    /// Whether the order is visible
    pub visible: bool,

    /// Minimum items per trade
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_trade: Option<u32>,

    /// Mod rank (for rankable mods)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>,

    /// Charges remaining (for consumable mods)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charges: Option<u8>,

    /// Item subtype (e.g., blueprint, crafted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,

    /// Amber stars installed (for sculptures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amber_stars: Option<u8>,

    /// Cyan stars installed (for sculptures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cyan_stars: Option<u8>,
}

impl CreateOrder {
    /// Create a new sell order.
    ///
    /// # Arguments
    ///
    /// * `item_id` - The item's ID or slug
    /// * `platinum` - Price in platinum
    /// * `quantity` - Number of items to sell
    pub fn sell(item_id: impl Into<String>, platinum: u32, quantity: u32) -> Self {
        Self {
            item_id: item_id.into(),
            order_type: OrderType::Sell,
            platinum,
            quantity,
            visible: true,
            per_trade: None,
            rank: None,
            charges: None,
            subtype: None,
            amber_stars: None,
            cyan_stars: None,
        }
    }

    /// Create a new buy order.
    ///
    /// # Arguments
    ///
    /// * `item_id` - The item's ID or slug
    /// * `platinum` - Price in platinum
    /// * `quantity` - Number of items to buy
    pub fn buy(item_id: impl Into<String>, platinum: u32, quantity: u32) -> Self {
        Self {
            item_id: item_id.into(),
            order_type: OrderType::Buy,
            platinum,
            quantity,
            visible: true,
            per_trade: None,
            rank: None,
            charges: None,
            subtype: None,
            amber_stars: None,
            cyan_stars: None,
        }
    }

    /// Set the mod rank (for rankable mods).
    pub fn with_mod_rank(mut self, rank: u8) -> Self {
        self.rank = Some(rank);
        self
    }

    /// Set the charges (for consumable mods like Requiem).
    pub fn with_charges(mut self, charges: u8) -> Self {
        self.charges = Some(charges);
        self
    }

    /// Set the item subtype (e.g., "blueprint", "crafted").
    pub fn with_subtype(mut self, subtype: impl Into<String>) -> Self {
        self.subtype = Some(subtype.into());
        self
    }

    /// Set the installed stars (for Ayatan sculptures).
    pub fn with_sculpture_stars(mut self, amber: u8, cyan: u8) -> Self {
        self.amber_stars = Some(amber);
        self.cyan_stars = Some(cyan);
        self
    }

    /// Set the minimum items per trade.
    pub fn with_per_trade(mut self, per_trade: u32) -> Self {
        self.per_trade = Some(per_trade);
        self
    }

    /// Make the order hidden (not visible to others).
    pub fn hidden(mut self) -> Self {
        self.visible = false;
        self
    }

    /// Make the order visible (default).
    pub fn visible(mut self) -> Self {
        self.visible = true;
        self
    }
}

/// Request to update an existing order.
///
/// Only include the fields you want to change.
///
/// # Example
///
/// ```
/// use wf_market::UpdateOrder;
///
/// // Update just the price
/// let update = UpdateOrder::new().platinum(90);
///
/// // Update price and quantity
/// let update = UpdateOrder::new()
///     .platinum(85)
///     .quantity(10);
///
/// // Hide the order
/// let update = UpdateOrder::new().visible(false);
/// ```
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrder {
    /// New price in platinum
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platinum: Option<u32>,

    /// New quantity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,

    /// New visibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,

    /// New minimum items per trade
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_trade: Option<u32>,

    /// New mod rank
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>,
}

impl UpdateOrder {
    /// Create a new empty update request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the new price.
    pub fn platinum(mut self, platinum: u32) -> Self {
        self.platinum = Some(platinum);
        self
    }

    /// Set the new quantity.
    pub fn quantity(mut self, quantity: u32) -> Self {
        self.quantity = Some(quantity);
        self
    }

    /// Set the new visibility.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    /// Set the new minimum items per trade.
    pub fn per_trade(mut self, per_trade: u32) -> Self {
        self.per_trade = Some(per_trade);
        self
    }

    /// Set the new mod rank.
    pub fn rank(mut self, rank: u8) -> Self {
        self.rank = Some(rank);
        self
    }

    /// Check if any fields are set.
    pub fn is_empty(&self) -> bool {
        self.platinum.is_none()
            && self.quantity.is_none()
            && self.visible.is_none()
            && self.per_trade.is_none()
            && self.rank.is_none()
    }
}

/// Filter options for fetching top orders.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFilters {
    /// Filter by mod rank
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<u8>,

    /// Filter by item subtype
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,

    /// Filter by user status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_status: Option<String>,
}

impl OrderFilters {
    /// Create new empty filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by mod rank.
    pub fn rank(mut self, rank: u8) -> Self {
        self.rank = Some(rank);
        self
    }

    /// Filter by item subtype.
    pub fn subtype(mut self, subtype: impl Into<String>) -> Self {
        self.subtype = Some(subtype.into());
        self
    }

    /// Filter by user status (online, ingame).
    pub fn user_status(mut self, status: impl Into<String>) -> Self {
        self.user_status = Some(status.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sell_order() {
        let order = CreateOrder::sell("test-item", 100, 5);

        assert_eq!(order.item_id, "test-item");
        assert!(matches!(order.order_type, OrderType::Sell));
        assert_eq!(order.platinum, 100);
        assert_eq!(order.quantity, 5);
        assert!(order.visible);
    }

    #[test]
    fn test_create_buy_order() {
        let order = CreateOrder::buy("test-item", 50, 10);

        assert!(matches!(order.order_type, OrderType::Buy));
        assert_eq!(order.platinum, 50);
    }

    #[test]
    fn test_order_builder_chain() {
        let order = CreateOrder::sell("mod-item", 100, 1)
            .with_mod_rank(10)
            .hidden();

        assert_eq!(order.rank, Some(10));
        assert!(!order.visible);
    }

    #[test]
    fn test_update_order() {
        let update = UpdateOrder::new().platinum(90).quantity(5);

        assert_eq!(update.platinum, Some(90));
        assert_eq!(update.quantity, Some(5));
        assert!(!update.is_empty());
    }

    #[test]
    fn test_update_order_empty() {
        let update = UpdateOrder::new();
        assert!(update.is_empty());
    }

    #[test]
    fn test_serialization() {
        let order = CreateOrder::sell("item", 100, 1).with_mod_rank(5);

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("\"rank\":5"));
        assert!(!json.contains("charges")); // None fields should be skipped
    }
}
