use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;

use crate::{client::Client, enums::*, errors::*, types::*};

#[derive(Debug)]
pub struct RivenRoute<State> {
    rivens_cache: Mutex<Option<Vec<Riven>>>,
    attributes_cache: Mutex<Option<Vec<RivenAttribute>>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> RivenRoute<State> {
    /**
     * Creates a new `RivenRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            rivens_cache: Mutex::new(None),
            attributes_cache: Mutex::new(None),
            client: Arc::downgrade(&client),
        })
    }
    /**
     * Fetches all rivens from the API.
     * If the rivens are already cached, it returns them directly.
     * Otherwise, it makes an API call to fetch the rivens and caches the response.
     * # Errors
     * Returns an `ApiError` if the API call fails.
     * # Returns
     * Returns a `Result<Vec<Riven>, ApiError>` containing the list of rivens or an error.
     */
    pub async fn get_all_rivens(&self) -> Result<Vec<Riven>, ApiError> {
        if let Some(items) = self.rivens_cache.lock().unwrap().as_ref() {
            return Ok(items.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<Riven>>>(
                ApiVersion::V2,
                Method::GET,
                "/riven/weapons",
                None,
                None,
            )
            .await
        {
            Ok((items, _, _)) => {
                // Cache the items response
                let mut cache = self.rivens_cache.lock().unwrap();
                *cache = Some(items.data.clone());
                Ok(items.data.clone())
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Fetches the a riven by its slug.
     * # Returns
     * - `Ok(Riven)` if the riven was found
     * - `Err(ApiError)` if there was an error fetching the riven
     */
    pub async fn get_riven_by_slug(&self, slug: &str) -> Result<Riven, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Riven>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/riven/weapon/{}", slug),
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
     * Fetches all attributes from the API.
     * If the attributes are already cached, it returns them directly.
     * Otherwise, it makes an API call to fetch the attributes and caches the response.
     * # Errors
     * Returns an `ApiError` if the API call fails.
     * # Returns
     * Returns a `Result<Vec<RivenAttribute>, ApiError>` containing the list of attributes or an error.
     */
    pub async fn get_all_attributes(&self) -> Result<Vec<RivenAttribute>, ApiError> {
        if let Some(items) = self.attributes_cache.lock().unwrap().as_ref() {
            return Ok(items.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<RivenAttribute>>>(
                ApiVersion::V2,
                Method::GET,
                "/riven/attributes",
                None,
                None,
            )
            .await
        {
            Ok((items, _, _)) => {
                // Cache the items response
                let mut cache = self.attributes_cache.lock().unwrap();
                *cache = Some(items.data.clone());
                Ok(items.data.clone())
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Creates a new `RivenRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &RivenRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            rivens_cache: Mutex::new(old.rivens_cache.lock().unwrap().clone()),
            attributes_cache: Mutex::new(old.attributes_cache.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}
