//! Orders API endpoints.

use serde_json::json;

use crate::client::{AuthState, Authenticated, Client};
use crate::error::{ApiErrorResponse, Error, Result};
use crate::internal::BASE_URL;
use crate::models::{
    CreateOrder, Order, OrderListing, OwnedOrder, OwnedOrderId, TopOrderFilters, TopOrders,
    Transaction, UpdateOrder,
};

use super::ApiResponse;

// === Public endpoints (both authenticated and unauthenticated) ===

impl<S: AuthState> Client<S> {
    /// Get orders for an item, including seller/buyer information.
    ///
    /// Returns orders from users who were online in the last 7 days.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let orders = client.get_orders("nikana_prime_set").await?;
    ///
    ///     for order in &orders {
    ///         println!(
    ///             "{} wants to {} for {}p ({})",
    ///             order.user.ingame_name,
    ///             order.order_type,
    ///             order.platinum,
    ///             order.user.status
    ///         );
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_orders(&self, slug: &str) -> Result<Vec<OrderListing>> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/orders/item/{}", BASE_URL, slug))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("Item not found: {}", slug)));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    format!("Failed to fetch orders for: {}", slug),
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch orders: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Vec<OrderListing>> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        // Inject item index into orders
        let mut orders = api_response.data;
        let item_index = self.items_arc();
        for order in &mut orders {
            order.set_item_index(item_index.clone());
        }

        Ok(orders)
    }

    /// Get orders for an item without user information.
    ///
    /// This is a lighter response when you only need order data
    /// (price, quantity, etc.) without seller/buyer details.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let orders = client.get_listings("nikana_prime_set").await?;
    ///
    ///     let avg_price: f64 = orders.iter()
    ///         .map(|o| o.platinum as f64)
    ///         .sum::<f64>() / orders.len() as f64;
    ///
    ///     println!("Average price: {:.0}p", avg_price);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_listings(&self, slug: &str) -> Result<Vec<Order>> {
        let listings = self.get_orders(slug).await?;
        // Item index is already injected via get_orders
        Ok(listings.into_iter().map(|l| l.order).collect())
    }

    /// Get top orders for an item (best buy and sell prices).
    ///
    /// Returns the top 5 buy orders (highest prices) and top 5 sell
    /// orders (lowest prices). Only includes orders from online users.
    ///
    /// # Arguments
    ///
    /// * `slug` - The item's URL-friendly identifier
    /// * `filters` - Optional filters for mod rank, charges, stars, or subtype
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, TopOrderFilters};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///
    ///     // Get top orders without filters
    ///     let top = client.get_top_orders("nikana_prime_set", None).await?;
    ///
    ///     // Get top orders for max rank mods
    ///     let filters = TopOrderFilters::new().rank(10);
    ///     let top = client.get_top_orders("serration", Some(&filters)).await?;
    ///
    ///     // Get top orders for sculptures with specific stars
    ///     let filters = TopOrderFilters::new().amber_stars(2).cyan_stars(4);
    ///     let top = client.get_top_orders("ayatan_anasa_sculpture", Some(&filters)).await?;
    ///
    ///     if let (Some(sell), Some(buy)) = (top.best_sell_price(), top.best_buy_price()) {
    ///         println!("Best sell: {}p, Best buy: {}p", sell, buy);
    ///         if let Some(spread) = top.spread() {
    ///             println!("Spread: {}p", spread);
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_top_orders(
        &self,
        slug: &str,
        filters: Option<&TopOrderFilters>,
    ) -> Result<TopOrders> {
        self.wait_for_rate_limit().await;

        let query = filters.map_or(String::new(), |f| f.to_query_string());
        let url = format!("{}/orders/item/{}/top{}", BASE_URL, slug, query);

        let response = self.http.get(&url).send().await.map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("Item not found: {}", slug)));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(
                status,
                format!("Failed to fetch top orders: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<TopOrders> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        // Inject item index into orders
        let mut top_orders = api_response.data;
        top_orders.set_item_index(self.items_arc());

        Ok(top_orders)
    }

    /// Get recent orders from the last 4 hours.
    ///
    /// Returns up to 500 of the most recent orders, sorted by creation time.
    /// Results are cached server-side with a 1-minute refresh interval.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let recent = client.get_recent_orders().await?;
    ///
    ///     for order in &recent {
    ///         println!(
    ///             "{} wants to {} {} for {}p",
    ///             order.user.ingame_name,
    ///             order.order_type,
    ///             order.order.item_id,
    ///             order.order.platinum
    ///         );
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_recent_orders(&self) -> Result<Vec<OrderListing>> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/orders/recent", BASE_URL))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    "Failed to fetch recent orders",
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch recent orders: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Vec<OrderListing>> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        // Inject item index into orders
        let mut orders = api_response.data;
        let item_index = self.items_arc();
        for order in &mut orders {
            order.set_item_index(item_index.clone());
        }

        Ok(orders)
    }

    /// Get all public orders for a specific user.
    ///
    /// Returns orders without user information (since the user is known).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let orders = client.get_user_orders("some_user").await?;
    ///
    ///     for order in &orders {
    ///         println!(
    ///             "{}: {} @ {}p",
    ///             order.order_type,
    ///             order.item_id,
    ///             order.platinum
    ///         );
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_user_orders(&self, user_slug: &str) -> Result<Vec<Order>> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/orders/user/{}", BASE_URL, user_slug))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("User not found: {}", user_slug)));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    format!("Failed to fetch orders for user: {}", user_slug),
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch user orders: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Vec<Order>> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        // Inject item index into orders
        let mut orders = api_response.data;
        let item_index = self.items_arc();
        for order in &mut orders {
            order.set_item_index(item_index.clone());
        }

        Ok(orders)
    }

    /// Get a single order by ID.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let order = client.get_order("order-id-here").await?;
    ///     println!("Order: {} @ {}p", order.item_id, order.platinum);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_order(&self, id: &str) -> Result<Order> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/order/{}", BASE_URL, id))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("Order not found: {}", id)));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    format!("Failed to fetch order: {}", id),
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch order: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Order> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        // Inject item index into order
        let mut order = api_response.data;
        order.set_item_index(self.items_arc());

        Ok(order)
    }
}

// === Authenticated endpoints ===

impl Client<Authenticated> {
    /// Get the authenticated user's orders.
    ///
    /// Returns orders as [`OwnedOrder`] which provides type-safe
    /// access to mutation operations.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, Credentials};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::from_credentials(/* ... */).await?;
    ///
    ///     let orders = client.my_orders().await?;
    ///     for order in &orders {
    ///         println!(
    ///             "{}: {} @ {}p (qty: {})",
    ///             order.id(),
    ///             order.item_id(),
    ///             order.platinum(),
    ///             order.quantity()
    ///         );
    ///     }
    ///     Ok(())
    /// }
    /// # fn main() {}
    /// ```
    pub async fn my_orders(&self) -> Result<Vec<OwnedOrder>> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/orders/my", BASE_URL))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::auth("Session expired or invalid"));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(
                status,
                format!("Failed to fetch orders: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Vec<Order>> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        // Inject item index into orders
        let item_index = self.items_arc();
        let orders: Vec<OwnedOrder> = api_response
            .data
            .into_iter()
            .map(|mut order| {
                order.set_item_index(item_index.clone());
                OwnedOrder::new(order)
            })
            .collect();

        Ok(orders)
    }

    /// Create a new order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, Credentials, CreateOrder};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::from_credentials(/* ... */).await?;
    ///
    ///     // Simple sell order
    ///     let order = client.create_order(
    ///         CreateOrder::sell("nikana_prime_set", 100, 1)
    ///     ).await?;
    ///
    ///     // Mod order with rank
    ///     let order = client.create_order(
    ///         CreateOrder::sell("serration", 50, 1).with_mod_rank(10)
    ///     ).await?;
    ///
    ///     println!("Created order: {}", order.id());
    ///     Ok(())
    /// }
    /// # fn main() {}
    /// ```
    pub async fn create_order(&self, request: CreateOrder) -> Result<OwnedOrder> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .post(format!("{}/order", BASE_URL))
            .json(&request)
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::auth("Session expired or invalid"));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    "Failed to create order",
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to create order: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Order> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        // Inject item index into order
        let mut order = api_response.data;
        order.set_item_index(self.items_arc());

        Ok(OwnedOrder::new(order))
    }

    /// Update an existing order.
    ///
    /// Only include the fields you want to change in the [`UpdateOrder`].
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, Credentials, UpdateOrder};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::from_credentials(/* ... */).await?;
    ///
    ///     let orders = client.my_orders().await?;
    ///     if let Some(order) = orders.first() {
    ///         // Update price
    ///         let updated = client.update_order(
    ///             order.id(),
    ///             UpdateOrder::new().platinum(90)
    ///         ).await?;
    ///
    ///         println!("Updated to {}p", updated.platinum());
    ///     }
    ///     Ok(())
    /// }
    /// # fn main() {}
    /// ```
    pub async fn update_order(&self, id: &OwnedOrderId, update: UpdateOrder) -> Result<OwnedOrder> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .patch(format!("{}/order/{}", BASE_URL, id.as_str()))
            .json(&update)
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::auth("Session expired or invalid"));
        }

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("Order not found: {}", id)));
        }

        if status == reqwest::StatusCode::FORBIDDEN {
            return Err(Error::auth("You don't own this order"));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    "Failed to update order",
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to update order: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Order> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        // Inject item index into order
        let mut order = api_response.data;
        order.set_item_index(self.items_arc());

        Ok(OwnedOrder::new(order))
    }

    /// Delete an order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, Credentials};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::from_credentials(/* ... */).await?;
    ///
    ///     let orders = client.my_orders().await?;
    ///     if let Some(order) = orders.first() {
    ///         client.delete_order(order.id()).await?;
    ///         println!("Deleted order");
    ///     }
    ///     Ok(())
    /// }
    /// # fn main() {}
    /// ```
    pub async fn delete_order(&self, id: &OwnedOrderId) -> Result<()> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .delete(format!("{}/order/{}", BASE_URL, id.as_str()))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::auth("Session expired or invalid"));
        }

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("Order not found: {}", id)));
        }

        if status == reqwest::StatusCode::FORBIDDEN {
            return Err(Error::auth("You don't own this order"));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(
                status,
                format!("Failed to delete order: {}", body),
            ));
        }

        Ok(())
    }

    /// Close (partially or fully) an order.
    ///
    /// This records a transaction for the specified quantity. If you
    /// close the entire remaining quantity, the order will be removed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, Credentials};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::from_credentials(/* ... */).await?;
    ///
    ///     let orders = client.my_orders().await?;
    ///     if let Some(order) = orders.first() {
    ///         // Close 5 units of the order
    ///         let transaction = client.close_order(order.id(), 5).await?;
    ///         println!(
    ///             "Closed {} units for {} total platinum",
    ///             transaction.quantity,
    ///             transaction.total_value()
    ///         );
    ///     }
    ///     Ok(())
    /// }
    /// # fn main() {}
    /// ```
    pub async fn close_order(&self, id: &OwnedOrderId, quantity: u32) -> Result<Transaction> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .post(format!("{}/order/{}/close", BASE_URL, id.as_str()))
            .json(&json!({ "quantity": quantity }))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::auth("Session expired or invalid"));
        }

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("Order not found: {}", id)));
        }

        if status == reqwest::StatusCode::FORBIDDEN {
            return Err(Error::auth("You don't own this order"));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    "Failed to close order",
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to close order: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Transaction> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.data)
    }
}
