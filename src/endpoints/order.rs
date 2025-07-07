use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;
use serde_json::json;

use crate::{
    client::{Client, IsAuthenticated},
    enums::*,
    errors::*,
    types::*,
};

#[derive(Debug)]
pub struct OrderRoute<State> {
    orders: Mutex<Vec<Order>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> OrderRoute<State> {
    /**
     * Creates a new `OrderRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            orders: Mutex::new(Vec::new()),
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
     * - `Ok(Vec<OrderWithUser>)` if the orders were fetched successfully
     * - `Err(ApiError)` if there was an error fetching the order
     */
    pub async fn recent(&self) -> Result<Vec<OrderWithUser>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<OrderWithUser>>>(
                ApiVersion::V2,
                Method::GET,
                "/orders/recent",
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
     * Get a list of all orders for an item from users who was online within the last 7 days.
     * # Returns
     * - `Ok(Vec<OrderWithUser>)` if the orders were fetched successfully
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
            client: Arc::downgrade(&client),
        })
    }
}

impl<State> OrderRoute<State>
where
    State: IsAuthenticated + Clone + 'static,
{
    pub fn orders(&self) -> Vec<Order> {
        let ca_orders = self.orders.lock().unwrap();
        ca_orders.clone()
    }
    /**
    Get the authenticated users orders

    # Returns
    List of all users orders
    */
    pub async fn my_orders(&self) -> Result<OrderList<Order>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<Order>>>(
                ApiVersion::V2,
                Method::GET,
                "/orders/my",
                None,
                None,
            )
            .await
        {
            Ok((orders, _, _)) => {
                let mut ca_orders = self.orders.lock().unwrap();
                ca_orders.clear();
                ca_orders.extend(orders.data.clone());
                Ok(OrderList::new(orders.data))
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
                Some(json!(args)),
                None,
            )
            .await
        {
            Ok((existing_order, _, _)) => {
                let mut ca_orders = self.orders.lock().unwrap();
                if let Some(index) = ca_orders
                    .iter()
                    .position(|o| o.id == existing_order.data.id)
                {
                    ca_orders[index] = existing_order.data.clone();
                } else {
                    ca_orders.push(existing_order.data.clone());
                }
                return Ok(existing_order.data);
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
        match client
            .as_ref()
            .call_api::<ApiResultV2<Order>>(
                ApiVersion::V2,
                Method::POST,
                "/order",
                Some(json!(args)),
                None,
            )
            .await
        {
            Ok((new_order, _, _)) => {
                let mut ca_orders = self.orders.lock().unwrap();
                ca_orders.push(new_order.data.clone());
                return Ok(new_order.data);
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
                Some(json!({ "quantity": quantity })),
                None,
            )
            .await
        {
            Ok((transaction, _, _)) => {
                let mut ca_orders = self.orders.lock().unwrap();
                if let Some(index) = ca_orders.iter().position(|o| o.id == order_id) {
                    let current_quantity = ca_orders[index].quantity;
                    if quantity > current_quantity {
                        ca_orders.remove(index);
                    } else {
                        ca_orders[index].quantity -= quantity;
                    }
                }
                Ok(transaction.data)
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
    pub async fn delete(&self, order_id: &str) -> Result<Order, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Order>>(
                ApiVersion::V2,
                Method::DELETE,
                format!("/order/{}", order_id).as_str(),
                None,
                None,
            )
            .await
        {
            Ok((order, _, _)) => {
                let mut ca_orders = self.orders.lock().unwrap();
                ca_orders.retain(|o| o.id != order.data.id);
                return Ok(order.data);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
