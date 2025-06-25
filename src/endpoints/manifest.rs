use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;

use crate::{client::Client, enums::*, errors::*, types::*};

#[derive(Debug)]
pub struct ManifestRoute<State> {
    versions_cache: Mutex<Option<VersionsResponse>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> ManifestRoute<State> {
    /**
     * Creates a new `ManifestRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            versions_cache: Mutex::new(None),
            client: Arc::downgrade(&client),
        })
    }
    /**
     * Fetches the versions of a user by their slug.
     * # Returns
     * - `Ok(VersionsResponse)` if the user was found
     * - `Err(ApiError)` if there was an error fetching the user
     */
    pub async fn versions(&self) -> Result<VersionsResponse, ApiError> {
        // Check if the versions are already cached
        if let Some(versions) = self.versions_cache.lock().unwrap().as_ref() {
            return Ok(versions.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<VersionsResponse>>(
                ApiVersion::V2,
                Method::GET,
                "/versions",
                None,
                None,
            )
            .await
        {
            Ok((user, _headers)) => {
                // Cache the versions response
                let mut cache = self.versions_cache.lock().unwrap();
                *cache = Some(user.data.clone());
                Ok(user.data)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Creates a new `ManifestRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &ManifestRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            versions_cache: Mutex::new(old.versions_cache.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}
