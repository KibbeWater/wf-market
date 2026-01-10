//! Order models for warframe.market.
//!
//! This module provides types for representing trading orders, including:
//!
//! - [`Order`] - A trading order (buy or sell)
//! - [`OrderListing`] - An order with associated user information
//! - [`OwnedOrder`] - An order belonging to the authenticated user
//! - [`OwnedOrderId`] - A type-safe ID for owned orders
//!
//! # Type Safety
//!
//! The [`OwnedOrderId`] type provides compile-time guarantees that order
//! mutation operations (update, delete, close) target orders owned by
//! the authenticated user:
//!
//! ```ignore
//! use wf_market::{Client, Credentials, OwnedOrderId};
//!
//! async fn example() -> wf_market::Result<()> {
//!     let client = Client::from_credentials(/* ... */).await?;
//!
//!     // Get user's orders - returns OwnedOrder with OwnedOrderId
//!     let orders = client.my_orders().await?;
//!
//!     for order in &orders {
//!         // Type-safe: can only update orders you own
//!         client.delete_order(order.id()).await?;
//!     }
//!
//!     // Can also restore from saved ID
//!     let saved_id = OwnedOrderId::from_raw("saved-order-id");
//!     client.delete_order(&saved_id).await?;
//!
//!     Ok(())
//! }
//! # fn main() {}
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::OrderType;
use super::user::User;

/// A trading order on warframe.market.
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    /// Unique order identifier
    pub id: String,

    /// Order type (buy or sell)
    #[serde(rename = "type")]
    pub order_type: OrderType,

    /// Price in platinum
    pub platinum: u32,

    /// Available quantity
    pub quantity: u32,

    /// ID of the item being traded
    #[serde(rename = "itemId")]
    pub item_id: String,

    /// Whether the order is visible to other users
    pub visible: bool,

    /// When the order was created
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    /// When the order was last updated
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,

    // === Optional fields based on item type ===
    /// Minimum quantity per trade
    #[serde(rename = "perTrade", default)]
    pub per_trade: Option<u32>,

    /// Item subtype (e.g., blueprint, crafted)
    #[serde(default)]
    pub subtype: Option<String>,

    /// Mod rank (for rankable mods)
    #[serde(default)]
    pub rank: Option<u8>,

    /// Remaining charges (for consumable mods)
    #[serde(default)]
    pub charges: Option<u8>,

    /// Installed amber stars (for Ayatan sculptures)
    #[serde(rename = "amberStars", default)]
    pub amber_stars: Option<u8>,

    /// Installed cyan stars (for Ayatan sculptures)
    #[serde(rename = "cyanStars", default)]
    pub cyan_stars: Option<u8>,

    /// Order group (default: 'all')
    #[serde(default)]
    pub group: Option<String>,
}

impl Order {
    /// Check if this is a buy order.
    pub fn is_buy(&self) -> bool {
        matches!(self.order_type, OrderType::Buy)
    }

    /// Check if this is a sell order.
    pub fn is_sell(&self) -> bool {
        matches!(self.order_type, OrderType::Sell)
    }

    /// Get the total platinum value (price * quantity).
    pub fn total_value(&self) -> u64 {
        self.platinum as u64 * self.quantity as u64
    }

    /// Check if this order is for a mod (has rank).
    pub fn is_mod_order(&self) -> bool {
        self.rank.is_some()
    }

    /// Check if this order is for a sculpture (has stars).
    pub fn is_sculpture_order(&self) -> bool {
        self.amber_stars.is_some() || self.cyan_stars.is_some()
    }
}

/// An order listing with associated user information.
///
/// This is what you get from `get_orders()` - orders from other users
/// with their profile information attached.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderListing {
    /// The order details
    #[serde(flatten)]
    pub order: Order,

    /// The user who posted this order
    pub user: User,
}

impl OrderListing {
    /// Check if the seller/buyer is available for trading.
    pub fn is_user_available(&self) -> bool {
        self.user.is_available()
    }

    /// Get just the order without user info.
    pub fn into_order(self) -> Order {
        self.order
    }
}

impl std::ops::Deref for OrderListing {
    type Target = Order;

    fn deref(&self) -> &Self::Target {
        &self.order
    }
}

/// An order ID that belongs to the authenticated user.
///
/// This type provides compile-time guarantees that order mutation
/// operations target orders owned by the current user. It can only
/// be obtained from:
///
/// - `Client::my_orders()` - returns orders with `OwnedOrderId`
/// - `Client::create_order()` - returns the created order
/// - [`OwnedOrderId::from_raw()`] - for restoring saved IDs
///
/// # Example
///
/// ```
/// use wf_market::OwnedOrderId;
///
/// // Restore from saved ID
/// let id = OwnedOrderId::from_raw("saved-order-id");
///
/// // Can be serialized for storage
/// let json = serde_json::to_string(&id).unwrap();
/// let restored: OwnedOrderId = serde_json::from_str(&json).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OwnedOrderId(String);

impl OwnedOrderId {
    /// Create from a raw ID string.
    ///
    /// Use this when restoring order IDs from storage. The ID will be
    /// validated by the API when used - invalid or unauthorized IDs
    /// will return an error.
    ///
    /// # Example
    ///
    /// ```
    /// use wf_market::OwnedOrderId;
    ///
    /// let id = OwnedOrderId::from_raw("550e8400-e29b-41d4-a716-446655440000");
    /// ```
    pub fn from_raw(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the inner String.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for OwnedOrderId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OwnedOrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for OwnedOrderId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for OwnedOrderId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// An order owned by the authenticated user.
///
/// This type wraps an [`Order`] with an [`OwnedOrderId`], providing
/// type-safe access to order mutation operations.
#[derive(Debug, Clone)]
pub struct OwnedOrder {
    id: OwnedOrderId,
    /// The order details
    pub order: Order,
}

impl OwnedOrder {
    /// Create a new owned order (internal use).
    pub(crate) fn new(order: Order) -> Self {
        Self {
            id: OwnedOrderId(order.id.clone()),
            order,
        }
    }

    /// Get the owned order ID.
    pub fn id(&self) -> &OwnedOrderId {
        &self.id
    }

    /// Get the order type.
    pub fn order_type(&self) -> OrderType {
        self.order.order_type
    }

    /// Get the platinum price.
    pub fn platinum(&self) -> u32 {
        self.order.platinum
    }

    /// Get the quantity.
    pub fn quantity(&self) -> u32 {
        self.order.quantity
    }

    /// Get the item ID.
    pub fn item_id(&self) -> &str {
        &self.order.item_id
    }

    /// Check if the order is visible.
    pub fn is_visible(&self) -> bool {
        self.order.visible
    }

    /// Get the creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.order.created_at
    }

    /// Get the last update timestamp.
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.order.updated_at
    }

    /// Consume and return the inner order.
    pub fn into_order(self) -> Order {
        self.order
    }
}

impl std::ops::Deref for OwnedOrder {
    type Target = Order;

    fn deref(&self) -> &Self::Target {
        &self.order
    }
}

/// Top orders result (buy and sell separated).
#[derive(Debug, Clone, Deserialize)]
pub struct TopOrders {
    /// Top buy orders (highest prices first)
    pub buy: Vec<OrderListing>,

    /// Top sell orders (lowest prices first)
    pub sell: Vec<OrderListing>,
}

impl TopOrders {
    /// Get all orders combined.
    pub fn all(&self) -> impl Iterator<Item = &OrderListing> {
        self.buy.iter().chain(self.sell.iter())
    }

    /// Get the best buy price (highest).
    pub fn best_buy_price(&self) -> Option<u32> {
        self.buy.first().map(|o| o.order.platinum)
    }

    /// Get the best sell price (lowest).
    pub fn best_sell_price(&self) -> Option<u32> {
        self.sell.first().map(|o| o.order.platinum)
    }

    /// Get the spread (difference between best sell and buy).
    pub fn spread(&self) -> Option<i32> {
        match (self.best_sell_price(), self.best_buy_price()) {
            (Some(sell), Some(buy)) => Some(sell as i32 - buy as i32),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_order() -> Order {
        Order {
            id: "test-id".to_string(),
            order_type: OrderType::Sell,
            platinum: 100,
            quantity: 5,
            item_id: "item-123".to_string(),
            visible: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            per_trade: None,
            subtype: None,
            rank: None,
            charges: None,
            amber_stars: None,
            cyan_stars: None,
            group: None,
        }
    }

    #[test]
    fn test_order_is_sell() {
        let order = make_order();
        assert!(order.is_sell());
        assert!(!order.is_buy());
    }

    #[test]
    fn test_order_total_value() {
        let order = make_order();
        assert_eq!(order.total_value(), 500);
    }

    #[test]
    fn test_owned_order_id() {
        let id = OwnedOrderId::from_raw("test-id");
        assert_eq!(id.as_str(), "test-id");
        assert_eq!(format!("{}", id), "test-id");
    }

    #[test]
    fn test_owned_order_id_serialization() {
        let id = OwnedOrderId::from_raw("test-id");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"test-id\"");

        let restored: OwnedOrderId = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, id);
    }

    #[test]
    fn test_owned_order() {
        let order = make_order();
        let owned = OwnedOrder::new(order);

        assert_eq!(owned.id().as_str(), "test-id");
        assert_eq!(owned.platinum(), 100);
        assert_eq!(owned.quantity(), 5);
    }

    #[test]
    fn test_owned_order_deref() {
        let order = make_order();
        let owned = OwnedOrder::new(order);

        // Can access Order fields through Deref
        assert!(owned.is_sell());
        assert_eq!(owned.total_value(), 500);
    }
}
