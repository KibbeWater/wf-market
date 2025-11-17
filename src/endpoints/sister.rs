use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;

use crate::{client::Client, enums::*, errors::*, types::*};

#[derive(Debug)]
pub struct SisterRoute<State> {
    ephemeras_cache: Mutex<Option<Vec<SisterEphemera>>>,
    quirks_cache: Mutex<Option<Vec<SisterQuirk>>>,
    weapons_cache: Mutex<Option<Vec<SisterWeapon>>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> SisterRoute<State> {
    /**
     * Creates a new `SisterRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            ephemeras_cache: Mutex::new(None),
            quirks_cache: Mutex::new(None),
            weapons_cache: Mutex::new(None),
            client: Arc::downgrade(&client),
        })
    }
    /**
     * Fetches all weapons from the API.
     * If the weapons are already cached, it returns them directly.
     * Otherwise, it makes an API call to fetch the weapons and caches the response.
     * # Errors
     * Returns an `ApiError` if the API call fails.
     * # Returns
     * Returns a `Result<Vec<SisterWeapon>, ApiError>` containing the list of weapons or an error.
     */
    pub async fn get_all_weapons(&self) -> Result<Vec<SisterWeapon>, ApiError> {
        if let Some(items) = self.weapons_cache.lock().unwrap().as_ref() {
            return Ok(items.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<SisterWeapon>>>(
                ApiVersion::V2,
                Method::GET,
                "/sister/weapons",
                "GET:sister:weapons",
                None,
                None,
            )
            .await
        {
            Ok((items, _, _)) => {
                // Cache the items response
                let mut cache = self.weapons_cache.lock().unwrap();
                *cache = Some(items.data.clone());
                Ok(items.data.clone())
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Fetches the a weapon by its slug.
     * # Returns
     * - `Ok(SisterWeapon)` if the weapon was found
     * - `Err(ApiError)` if there was an error fetching the weapon
     */
    pub async fn get_weapon_by_slug(&self, slug: &str) -> Result<SisterWeapon, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<SisterWeapon>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/sister/weapon/{}", slug),
                "GET:sister:weapon:slug",
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
     * Fetches all ephemeras from the API.
     * If the ephemeras are already cached, it returns them directly.
     * Otherwise, it makes an API call to fetch the ephemeras and caches the response.
     * # Errors
     * Returns an `ApiError` if the API call fails.
     * # Returns
     * Returns a `Result<Vec<SisterEphemera>, ApiError>` containing the list of ephemeras or an error.
     */
    pub async fn get_all_ephemeras(&self) -> Result<Vec<SisterEphemera>, ApiError> {
        if let Some(items) = self.ephemeras_cache.lock().unwrap().as_ref() {
            return Ok(items.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<SisterEphemera>>>(
                ApiVersion::V2,
                Method::GET,
                "/sister/ephemeras",
                "GET:sister:ephemeras",
                None,
                None,
            )
            .await
        {
            Ok((items, _, _)) => {
                // Cache the items response
                let mut cache = self.ephemeras_cache.lock().unwrap();
                *cache = Some(items.data.clone());
                Ok(items.data.clone())
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Fetches all quirks from the API.
     * If the quirks are already cached, it returns them directly.
     * Otherwise, it makes an API call to fetch the quirks and caches the response.
     * # Errors
     * Returns an `ApiError` if the API call fails.
     * # Returns
     * Returns a `Result<Vec<SisterQuirk>, ApiError>` containing the list of quirks or an error.
     */
    pub async fn get_all_quirks(&self) -> Result<Vec<SisterQuirk>, ApiError> {
        if let Some(items) = self.quirks_cache.lock().unwrap().as_ref() {
            return Ok(items.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<SisterQuirk>>>(
                ApiVersion::V2,
                Method::GET,
                "/sister/quirks",
                "GET:sister:quirks",
                None,
                None,
            )
            .await
        {
            Ok((items, _, _)) => {
                // Cache the items response
                let mut cache = self.quirks_cache.lock().unwrap();
                *cache = Some(items.data.clone());
                Ok(items.data.clone())
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Creates a new `SisterRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &SisterRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            ephemeras_cache: Mutex::new(old.ephemeras_cache.lock().unwrap().clone()),
            quirks_cache: Mutex::new(old.quirks_cache.lock().unwrap().clone()),
            weapons_cache: Mutex::new(old.weapons_cache.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}
