use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;

use crate::{client::Client, enums::*, errors::*, types::*};

#[derive(Debug)]
pub struct ItemRoute<State> {
    items_cache: Mutex<Option<Vec<Item>>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> ItemRoute<State> {
    /**
     * Creates a new `ItemRoute` with an empty User list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            items_cache: Mutex::new(None),
            client: Arc::downgrade(&client),
        })
    }

    /**
     * Fetches all items from the API.
     * If the items are already cached, it returns them directly.
     * Otherwise, it makes an API call to fetch the items and caches the response.
     * # Errors
     * Returns an `ApiError` if the API call fails.
     * # Returns
     * Returns a `Result<Vec<Item>, ApiError>` containing the list of items or an
     */
    pub async fn get_all(&self) -> Result<Vec<Item>, ApiError> {
        // Check if the items are already cached
        // let items = self.items_cache.lock().unwrap();
        // Check if the versions are already cached
        if let Some(items) = self.items_cache.lock().unwrap().as_ref() {
            return Ok(items.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<Item>>>(ApiVersion::V2, Method::GET, "/items", None, None)
            .await
        {
            Ok((items, _, _)) => {
                // Cache the items response
                let mut cache = self.items_cache.lock().unwrap();
                *cache = Some(items.data.clone());
                Ok(items.data.clone())
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Fetches the a item by its slug.
     * # Returns
     * - `Ok(Item)` if the item was found
     * - `Err(ApiError)` if there was an error fetching the item
     */
    pub async fn get_by_slug(&self, slug: &str) -> Result<Item, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Item>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/item/{}", slug),
                None,
                None,
            )
            .await
        {
            Ok((user, _, _)) => Ok(user.data),
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Creates a new `ItemRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &ItemRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            items_cache: Mutex::new(old.items_cache.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}
