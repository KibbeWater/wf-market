use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;

use crate::{client::Client, enums::*, errors::*, types::*};

pub struct ManifestRoute<State> {
    versions_cache: Mutex<Option<VersionsResponse>>,
    locations_cache: Mutex<Option<Vec<Location>>>,
    npcs_cache: Mutex<Option<Vec<Npc>>>,
    missions_cache: Mutex<Option<Vec<Mission>>>,
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
            locations_cache: Mutex::new(None),
            npcs_cache: Mutex::new(None),
            missions_cache: Mutex::new(None),
            client: Arc::downgrade(&client),
        })
    }

    /**
     * Fetches the versions of the api.
     * # Returns
     * - `Ok(VersionsResponse)` The versions response
     * - `Err(ApiError)` if there was an error fetching the versions
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
                "GET:versions",
                None,
                None,
            )
            .await
        {
            Ok((user, _, _)) => {
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
     * Fetches the locations available in the game.
     * # Returns
     * - `Ok(Vec<Location>)` The locations response
     * - `Err(ApiError)` if there was an error fetching the locations
     */
    pub async fn locations(&self) -> Result<Vec<Location>, ApiError> {
        // Check if the locations are already cached
        if let Some(locations) = self.locations_cache.lock().unwrap().as_ref() {
            return Ok(locations.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<Location>>>(
                ApiVersion::V2,
                Method::GET,
                "/locations",
                "GET:locations",
                None,
                None,
            )
            .await
        {
            Ok((user, _, _)) => {
                // Cache the versions response
                let mut cache = self.locations_cache.lock().unwrap();
                *cache = Some(user.data.clone());
                Ok(user.data)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Fetches the NPCs available in the game.
     * # Returns
     * - `Ok(Vec<Npc>)` The NPCs response
     * - `Err(ApiError)` if there was an error fetching the NPCs
     */
    pub async fn npcs(&self) -> Result<Vec<Npc>, ApiError> {
        // Check if the NPCs are already cached
        if let Some(npcs) = self.npcs_cache.lock().unwrap().as_ref() {
            return Ok(npcs.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<Npc>>>(
                ApiVersion::V2,
                Method::GET,
                "/npcs",
                "GET:npcs",
                None,
                None,
            )
            .await
        {
            Ok((user, _, _)) => {
                // Cache the versions response
                let mut cache = self.npcs_cache.lock().unwrap();
                *cache = Some(user.data.clone());
                Ok(user.data)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Fetches the missions available in the game.
     * # Returns
     * - `Ok(Vec<Mission>)` The missions response
     * - `Err(ApiError)` if there was an error fetching the missions
     */
    pub async fn missions(&self) -> Result<Vec<Mission>, ApiError> {
        // Check if the missions are already cached
        if let Some(missions) = self.missions_cache.lock().unwrap().as_ref() {
            return Ok(missions.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<Mission>>>(
                ApiVersion::V2,
                Method::GET,
                "/missions",
                "GET:missions",
                None,
                None,
            )
            .await
        {
            Ok((user, _, _)) => {
                // Cache the versions response
                let mut cache = self.missions_cache.lock().unwrap();
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
            locations_cache: Mutex::new(old.locations_cache.lock().unwrap().clone()),
            npcs_cache: Mutex::new(old.npcs_cache.lock().unwrap().clone()),
            missions_cache: Mutex::new(old.missions_cache.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}
