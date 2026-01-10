//! WebSocket subscription types.

use crate::models::Platform;

/// Subscriptions for real-time updates.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Subscription {
    /// Subscribe to new orders feed.
    ///
    /// Receives live feed of newly posted orders from online users.
    /// Can filter by platform and crossplay settings.
    NewOrders {
        platform: Option<Platform>,
        crossplay: bool,
    },

    /// Subscribe to order updates for a specific item.
    ///
    /// Requires authentication with `realTime` scope.
    Item { item_id: String },

    /// Subscribe to order updates for a specific user profile.
    ///
    /// Requires authentication with `realTime` scope.
    Profile { user_id: String },
}

impl Subscription {
    /// Subscribe to all new orders (all platforms, crossplay enabled).
    pub fn all_new_orders() -> Self {
        Self::NewOrders {
            platform: None,
            crossplay: true,
        }
    }

    /// Subscribe to new orders for a specific platform.
    pub fn new_orders_for(platform: Platform) -> Self {
        Self::NewOrders {
            platform: Some(platform),
            crossplay: true,
        }
    }

    /// Subscribe to new orders with specific settings.
    pub fn new_orders(platform: Option<Platform>, crossplay: bool) -> Self {
        Self::NewOrders {
            platform,
            crossplay,
        }
    }

    /// Subscribe to item order updates.
    pub fn item(item_id: impl Into<String>) -> Self {
        Self::Item {
            item_id: item_id.into(),
        }
    }

    /// Subscribe to profile order updates.
    pub fn profile(user_id: impl Into<String>) -> Self {
        Self::Profile {
            user_id: user_id.into(),
        }
    }

    /// Get the route for subscribing.
    pub(crate) fn subscribe_route(&self) -> &'static str {
        match self {
            Self::NewOrders { .. } => "@wfm|cmd/subscribe/newOrders",
            Self::Item { .. } => "@wfm|cmd/subscribe/item",
            Self::Profile { .. } => "@wfm|cmd/subscribe/profile",
        }
    }

    /// Get the route for unsubscribing.
    pub(crate) fn unsubscribe_route(&self) -> &'static str {
        match self {
            Self::NewOrders { .. } => "@wfm|cmd/unsubscribe/newOrders",
            Self::Item { .. } => "@wfm|cmd/unsubscribe/item",
            Self::Profile { .. } => "@wfm|cmd/unsubscribe/profile",
        }
    }

    /// Build the payload for this subscription.
    pub(crate) fn to_payload(&self) -> serde_json::Value {
        match self {
            Self::NewOrders {
                platform,
                crossplay,
            } => {
                let mut payload = serde_json::json!({
                    "crossplay": crossplay,
                });

                if let Some(p) = platform {
                    payload["platform"] = serde_json::json!(p);
                }

                payload
            }
            Self::Item { item_id } => {
                serde_json::json!({
                    "itemId": item_id,
                })
            }
            Self::Profile { user_id } => {
                serde_json::json!({
                    "userId": user_id,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_new_orders() {
        let sub = Subscription::all_new_orders();
        assert!(matches!(
            sub,
            Subscription::NewOrders {
                platform: None,
                crossplay: true
            }
        ));
    }

    #[test]
    fn test_item_subscription() {
        let sub = Subscription::item("nikana_prime_set");
        assert!(matches!(sub, Subscription::Item { item_id } if item_id == "nikana_prime_set"));
    }

    #[test]
    fn test_payload_generation() {
        let sub = Subscription::item("test-item");
        let payload = sub.to_payload();
        assert_eq!(payload["itemId"], "test-item");
    }
}
