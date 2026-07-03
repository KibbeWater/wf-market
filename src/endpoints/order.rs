use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;
use serde_json::json;

use crate::{
    client::{Client, IsAuthenticated},
    enums::*,
    errors::*,
    types::*,
};

pub struct OrderRoute<State> {
    orders: Mutex<OrderList<Order>>,
    limit: Mutex<usize>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> OrderRoute<State> {
    /**
     * Creates a new `OrderRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            orders: Mutex::new(OrderList::new(vec![])),
            limit: Mutex::new(100), // Default limit for recent orders
            client: Arc::downgrade(&client),
        })
    }
    /**
     * Fetches an order by its ID.
     * # Arguments
     * - `order_id`: The ID of the order to fetch
     * TODO: Test if it only works for owners orders.
     * # Returns
     * - `Ok(OrderWithUser)` if the order was found
     * - `Err(ApiError)` if there was an error fetching the order
     */
    pub async fn get_by_id(&self, order_id: &str) -> Result<OrderWithUser, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<OrderWithUser>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/orders/{}", order_id),
                "GET:orders:order_id",
                None,
                None,
            )
            .await
        {
            Ok((orders, _, _)) => Ok(orders.data),
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Get the most recent orders
     * 500 max, for the last 4 hours, sorted by created_at descending.
     * Cached, with 1min refresh interval.
     * # Returns
     * - `Ok(OrderList<OrderWithUser>)` if the orders were fetched successfully
     * - `Err(ApiError)` if there was an error fetching the order
     */
    pub async fn recent(&self) -> Result<OrderList<OrderWithUser>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<OrderWithUser>>>(
                ApiVersion::V2,
                Method::GET,
                "/orders/recent",
                "GET:orders:recent",
                None,
                None,
            )
            .await
        {
            Ok((orders, _, _)) => Ok(OrderList::new(orders.data)),
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Get a list of all orders for an item from users who was online within the last 7 days.
     * # Returns
     * - `Ok(OrderList<OrderWithUser>)` if the orders were fetched successfully
     * - `Err(ApiError)` if there was an error fetching the order
     */
    pub async fn get_orders_by_item(
        &self,
        slug: &str,
    ) -> Result<OrderList<OrderWithUser>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<OrderWithUser>>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/orders/item/{}", slug),
                "GET:orders:item:slug",
                None,
                None,
            )
            .await
        {
            Ok((orders, _, _)) => {
                let list = OrderList::new(orders.data);
                Ok(list)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * This endpoint is designed to fetch the top 5 buy and top 5 sell orders for a specific item, exclusively from online users
     * # Returns
     * - `Ok(Vec<OrderWithUser>)` if the orders were fetched successfully
     * - `Err(ApiError)` if there was an error fetching the order
     */
    pub async fn get_top_orders_by_item(
        &self,
        slug: &str,
        filters: Option<TopOrdersFilters>,
    ) -> Result<OrdersTop, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        let query: String = if let Some(filters) = filters.clone() {
            let params = serde_urlencoded::to_string(filters)
                .map_err(|_| ApiError::Unknown("Failed to serialize filters".to_string()))?;
            format!("?{}", params)
        } else {
            String::new()
        };

        match client
            .as_ref()
            .call_api::<ApiResultV2<OrdersTop>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/orders/item/{}/top{}", slug, query),
                "GET:orders:item:slug:top",
                None,
                None,
            )
            .await
        {
            Ok((orders, _, _)) => Ok(orders.data),
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Creates a new `OrderRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &OrderRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            orders: Mutex::new(old.orders.lock().unwrap().clone()),
            limit: Mutex::new(old.limit.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}

impl<State> OrderRoute<State>
where
    State: IsAuthenticated + Clone + 'static,
{
    /**
    Get the current orders from the cache.
    # Returns
    - `Vec<Order>`: A vector of orders currently stored in the cache.
    */
    pub fn cache_orders(&self) -> OrderList<Order> {
        let ca_orders = self.orders.lock().unwrap();
        ca_orders.clone()
    }

    /**
    Get a mutable reference to the current orders from the cache.
    # Returns
    - `std::sync::MutexGuard<OrderList<Order>>`: A mutable reference to the cached orders.
    */
    pub fn cache_orders_mut(&'_ self) -> std::sync::MutexGuard<'_, OrderList<Order>> {
        self.orders.lock().unwrap()
    }
    /**
     * Set orders in the cache
     */
    pub fn set_orders(&self, orders: OrderList<Order>) {
        let mut ca_orders = self.orders.lock().unwrap();
        *ca_orders = orders;
    }

    /**
    Get the authenticated users orders
    * # Returns
    * - `Ok(OrderList<Order>)` if the orders were fetched successfully
    * - `Err(ApiError)` if there was an error fetching the orders
    */
    pub async fn my_orders(&self) -> Result<OrderList<Order>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<Order>>>(
                ApiVersion::V2,
                Method::GET,
                "/orders/my",
                "GET:orders:my",
                None,
                None,
            )
            .await
        {
            Ok((orders, _, _)) => {
                let orders = OrderList::new(orders.data);
                let mut ca_orders = self.orders.lock().unwrap();
                *ca_orders = orders.clone();
                Ok(orders)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
    Update order information

    # Arguments
    - `order_id`: The ID of the order to update
    - `args`: The [`UpdateOrderParams`][crate::types::request::UpdateOrderParams] to update the order with
    # Returns
    The updated order
    */
    pub async fn update(&self, order_id: &str, args: UpdateOrderParams) -> Result<Order, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");
        match client
            .as_ref()
            .call_api::<ApiResultV2<Order>>(
                ApiVersion::V2,
                Method::PATCH,
                format!("/order/{}", order_id).as_str(),
                "PATCH:order:order_id",
                Some(json!(args)),
                None,
            )
            .await
        {
            Ok((existing_order, _, _)) => {
                let mut order = existing_order.data;
                order.properties.set_properties(args.properties.clone());
                let mut ca_orders = self.orders.lock().unwrap();
                ca_orders.update(order_id, args);
                return Ok(order);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Create a new order
     * # Arguments
     * - `args`: The [`OrderCreationRequest`][crate::types::request::OrderCreationRequest] to create the order with
     * # Returns
     * The created order
     */
    pub async fn create(&self, args: CreateOrderParams) -> Result<Order, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");
        if !self.can_create_order() {
            return Err(ApiError::OrderLimitExceeded(RequestError::new(
                ApiVersion::V2,
                "POST".to_string(),
                "/order".to_string(),
                Some(json!(args)),
            )));
        }
        match client
            .as_ref()
            .call_api::<ApiResultV2<Order>>(
                ApiVersion::V2,
                Method::POST,
                "/order",
                "POST:order",
                Some(json!(args)),
                None,
            )
            .await
        {
            Ok((new_order, _, _)) => {
                let mut order = new_order.data;
                let mut ca_orders = self.orders.lock().unwrap();
                order.properties.set_properties(args.properties.clone());
                ca_orders.add(order.clone());
                return Ok(order);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
    Close a portion or all of an existing order.
    Allows you to close part of an open order by specifying a quantity to reduce.
    For example, if your order was initially created with a quantity of 20, and you send a request to close 8 units, the remaining quantity will be 12.
    If you close the entire remaining quantity, the order will be considered fully closed and removed.
    # Arguments
    - `order_id`: The ID of the order to delete
    - `quantity`: The quantity of the order to delete
    # Returns
    - `Ok(Transaction)` if the order was successfully deleted
    - `Err(ApiError)` if there was an error deleting the order
    */

    pub async fn close(&self, order_id: &str, quantity: u32) -> Result<Transaction, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Transaction>>(
                ApiVersion::V2,
                Method::POST,
                format!("/order/{}/close", order_id).as_str(),
                "POST:order:order_id:close",
                Some(json!({ "quantity": quantity })),
                None,
            )
            .await
        {
            Ok((transaction, _, _)) => {
                let transaction = transaction.data;
                let mut ca_orders = self.orders.lock().unwrap();
                ca_orders.close_order(order_id, quantity);
                Ok(transaction)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Delete an order
     * # Arguments
     * - `order_id`: The ID of the order to delete
     * # Returns
     * - `Ok(Order)` if the order was successfully deleted
     * - `Err(ApiError)` if there was an error deleting the order
     */
    pub async fn delete(&self, order_id: impl Into<String>) -> Result<Order, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");
        let order_id = order_id.into();

        match client
            .as_ref()
            .call_api::<ApiResultV2<Order>>(
                ApiVersion::V2,
                Method::DELETE,
                format!("/order/{}", order_id).as_str(),
                "DELETE:order:order_id",
                None,
                None,
            )
            .await
        {
            Ok((order, _, _)) => {
                let mut ca_orders = self.orders.lock().unwrap();
                ca_orders.remove_by_id(&order.data.id);
                return Ok(order.data);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Set the limit for active orders
     * # Arguments
     * - `limit`: The new limit for active orders
     * # Returns
     * - `Ok(())` if the limit was successfully set
     * - `Err(ApiError)` if there was an error setting the limit
     */
    pub fn set_order_limit(&self, limit: usize) {
        let mut ca_orders = self.limit.lock().unwrap();
        *ca_orders = limit;
    }

    /**
     * Get the current limit for active orders
     * # Returns
     * - `Ok(usize)` if the limit was successfully retrieved
     * - `Err(ApiError)` if there was an error retrieving the limit
     */
    pub fn get_order_limit(&self) -> usize {
        let ca_orders = self.limit.lock().unwrap();
        *ca_orders
    }
    /**
     * Check if a new order can be created
     * # Returns
     * - `true` if a new order can be created
     * - `false` if the order limit has been reached
     */
    pub fn can_create_order(&self) -> bool {
        let ca_orders = self.orders.lock().unwrap();
        let order_limit = self.get_order_limit();
        ca_orders.total_orders() < order_limit
    }
}
