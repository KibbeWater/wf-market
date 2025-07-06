/*!
Provides `Client` struct for interacting with the Warframe Market API.

# Examples

Running unauthenticated:
```rust
use wf_market::client::Client;

let client = Client::new();
```

Running authenticated:
```rust
use wf_market::{
    client::Client,
    utils::generate_device_id,
};

async fn main() {
    let client = {
        // device_id should be stored and reused
        Client::new()
            .login("username", "password", generate_device_id().as_str()).await.unwrap()
    };

    let user = client.user.unwrap();
    println!("Logged in as: {}", user.name);
}
```
*/

// Submodules
mod auth;
mod constants;
mod item;
mod order;
mod riven;
mod utils;
pub mod ws;
pub mod oauth;

use crate::error::{ApiError, ApiErrorBody, ErrorResponse};
use crate::types::filter::OrdersTopFilters;
use crate::types::http::ApiResult;
use crate::types::item::{Item as ItemObject, Order as OrderItem, OrderWithUser, OrdersTopResult};
use crate::types::riven::Riven as RivenObject;
use crate::types::user::{FullUser, StatusType};
use governor::RateLimiter;
use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use reqwest::Method as HttpMethod;
use serde::Serialize;
use std::marker::PhantomData;
use std::sync::Arc;

pub use auth::*;
use constants::*;
pub use item::*;
pub use order::*;
pub use riven::*;
use utils::*;

pub struct Unauthenticated;
pub struct Authenticated;

pub struct Client<State = Unauthenticated> {
    pub(crate) http: reqwest::Client,
    pub user: Option<FullUser>,
    pub orders: Vec<Order<Owned>>,
    pub status: StatusType,
    items_cache: Vec<Item>,
    rivens_cache: Vec<Riven>,
    token: Option<String>,
    device_id: Option<String>,
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    _state: PhantomData<State>,
}

#[derive(Serialize)]
struct NoBody;

pub enum Method {
    Get,
    Post,
    Patch,
    Put,
    Delete,
}

impl<State> Client<State> {
    /**
    INTERNAL: Makes a request to the API, returning the response as a deserialized type.


    # Arguments
    - `method`: The HTTP method to use (GET, POST, PUT, DELETE).
    - `path`: The path to the API endpoint. (e.g., "/me").
    - `body`: An optional body to send with the request, serializable as JSON.

    # Returns
    - A `Result` containing the deserialized response or an `ApiError` on failure.
    */
    pub(crate) async fn call_api<T: serde::de::DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<T, ApiError> {
        let url = BASE_URL.to_owned() + path;
        let builder = self.http.request(transform_method(method), &url);

        let builder = if let Some(body) = body {
            builder.json(body)
        } else {
            builder
        };

        self.limiter.until_ready().await;

        match builder.send().await {
            Ok(resp) => {
                let status = resp.status();

                let body = resp
                    .text()
                    .await
                    .map_err(|_| ApiError::Unknown("Error".to_string()))?;

                // Check if the status code indicates an error
                match status {
                    reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => {}
                    reqwest::StatusCode::UNAUTHORIZED => {
                        return Err(ApiError::Unauthorized);
                    }
                    reqwest::StatusCode::NOT_FOUND => {
                        return Err(ApiError::NotFound(format!(
                            "Resource not found: {}, Message: {}",
                            url, body
                        )));
                    }
                    reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::FORBIDDEN => {
                        match serde_json::from_str::<ErrorResponse>(&body) {
                            Ok(api_result) => {
                                return Err(ApiError::WFMError(api_result));
                            }
                            Err(e) => {
                                return Err(ApiError::ParsingError(
                                    format!(
                                        "Error Parsing Bad Request Error: {:?}, Message: {}",
                                        e, body
                                    )
                                    .to_string(),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(ApiError::Unknown(format!(
                            "Unexpected status code: {}",
                            status
                        )));
                    }
                }

                let data = serde_json::from_str::<T>(&body);

                match data {
                    Ok(data) => Ok(data),
                    Err(err) => Err(ApiError::ParsingError(
                        format!("Error Parsing: {:?}, Body: {}", err, body).to_string(),
                    )),
                }
            }
            Err(_) => Err(ApiError::RequestError),
        }
    }

    /**
    Fetch all listed items from the WFM API

    # Returns
    List of all listed items
    */
    pub async fn get_items(&self) -> Result<Vec<Item<Regular>>, ApiError> {
        if !self.items_cache.is_empty() {
            let mut new_items = Vec::new();
            new_items.clone_from(&self.items_cache);
            return Ok(new_items);
        }

        let items: Result<ApiResult<Vec<ItemObject>>, ApiError> =
            self.call_api(Method::Get, "/items", None::<&NoBody>).await;

        Ok(items?.data.iter().map(|item| Item::new(item)).collect())
    }

    /**
    Fetch an item by an identifiable slug

    # Returns
    Full item object (currently the same from `get_items()`)
    */
    pub async fn get_item(&self, slug: &str) -> Result<Item<Regular>, ApiError> {
        let items: Result<ApiResult<ItemObject>, ApiError> = self
            .call_api(
                Method::Get,
                format!("/item/{}", slug).as_str(),
                None::<&NoBody>,
            )
            .await;

        Ok(Item::new(&items?.data))
    }

    /**
    Fetch all orders from users online within the last 7 days

    # Arguments
    - `slug`: The item whose orders you want to fetch

    # Returns
    A list of orders
    */
    pub async fn get_orders(&self, slug: &str) -> Result<Vec<Order<Unowned>>, ApiError> {
        let items: Result<ApiResult<Vec<OrderWithUser>>, ApiError> = self
            .call_api(
                Method::Get,
                format!("/orders/item/{}", slug).as_str(),
                None::<&NoBody>,
            )
            .await;

        Ok(items?
            .data
            .iter()
            .map(|order| Order::new(&order.downgrade()))
            .collect())
    }

    /**
    Fetch the top 5 orders for the specified slug

    # Arguments
    - `slug`: The item whose orders you want to fetch

    # Returns
    Total of 10 orders, top 5 buy/sell orders
    */
    pub async fn get_orders_top(
        &self,
        slug: &str,
        filters: Option<OrdersTopFilters>,
    ) -> Result<Vec<Order<Unowned>>, ApiError> {
        let query: String = if let Some(filters) = filters.clone() {
            let params = serde_urlencoded::to_string(filters)
                .map_err(|_| ApiError::ParsingError("Unable to serialize filters".to_string()))?;
            format!("?{}", params)
        } else {
            String::new()
        };

        let items: Result<ApiResult<OrdersTopResult>, ApiError> = self
            .call_api(
                Method::Get,
                format!("/orders/item/{}/top{}", slug, query).as_str(),
                None::<&NoBody>,
            )
            .await;

        let data = items?.data;

        let is_filtering_status = if let Some(filters) = filters.clone() {
            filters.user_activity.is_some()
        } else {
            false
        };

        let buy: Vec<Order<Unowned>> = data
            .buy
            .iter()
            .filter(|o| {
                is_filtering_status
                    && o.user.status_type == filters.clone().unwrap().user_activity.unwrap()
            })
            .map(|order| Order::new(&order.downgrade()))
            .collect();
        let sell: Vec<Order<Unowned>> = data
            .sell
            .iter()
            .filter(|o| {
                is_filtering_status
                    && o.user.status_type == filters.clone().unwrap().user_activity.unwrap()
            })
            .map(|order| Order::new(&order.downgrade()))
            .collect();

        let total: Vec<Order<Unowned>> = [buy, sell].concat();

        Ok(total)
    }

    /**
    Get the Item Type of an Order, fetches from updated list of items

    # Arguments
    - `order`: The order to get the type of

    # Returns
    A managed [`Item`][crate::client::item::Item] object
    */
    pub async fn get_order_item(&self, order: &Order) -> Result<Item<Regular>, ApiError> {
        if let Some(item) = self
            .get_items()
            .await?
            .iter()
            .find(|i| i.get_type().id == order.object.item_id)
        {
            return Ok(item.clone());
        }

        Err(ApiError::Unknown("Item not found".to_string()))
    }

    /**
    Get the order from an id

    # Arguments
    - `id`: An order ID

    # Returns
    A managed [`Order`][crate::client::order::Order] object
    */
    pub async fn get_order(&self, id: &str) -> Result<Order<Unowned>, ApiError> {
        let order: Result<ApiResult<OrderItem>, ApiError> = self
            .call_api(
                Method::Get,
                format!("/order/{}", id).as_str(),
                None::<&NoBody>,
            )
            .await;

        Ok(Order::new(&order?.data))
    }
    /**
    Fetch all listed rivens from the WFM API

    # Returns
    List of all listed rivens
    */
    pub async fn get_rivens(&self) -> Result<Vec<Riven>, ApiError> {
        if !self.rivens_cache.is_empty() {
            let mut new_items = Vec::new();
            new_items.clone_from(&self.rivens_cache);
            return Ok(new_items);
        }

        let rivens: Result<ApiResult<Vec<RivenObject>>, ApiError> = self
            .call_api(Method::Get, "/riven/weapons", None::<&NoBody>)
            .await;

        Ok(rivens?.data.iter().map(|riven| Riven::new(riven)).collect())
    }
}

// Honestly idk why i didn't just use reqwest::Method directly, but here we are
fn transform_method(method: Method) -> HttpMethod {
    match method {
        Method::Get => HttpMethod::GET,
        Method::Post => HttpMethod::POST,
        Method::Patch => HttpMethod::PATCH,
        Method::Put => HttpMethod::PUT,
        Method::Delete => HttpMethod::DELETE,
    }
}
