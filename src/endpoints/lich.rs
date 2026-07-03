use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;

use crate::{client::Client, enums::*, errors::*, types::*};

pub struct LichRoute<State> {
    ephemeras_cache: Mutex<Option<Vec<LichEphemera>>>,
    quirks_cache: Mutex<Option<Vec<LichQuirk>>>,
    weapons_cache: Mutex<Option<Vec<LichWeapon>>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> LichRoute<State> {
    /**
     * Creates a new `LichRoute` with an empty order list.
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
     * Returns a `Result<Vec<LichWeapon>, ApiError>` containing the list of weapons or an error.
     */
    pub async fn get_all_weapons(&self) -> Result<Vec<LichWeapon>, ApiError> {
        if let Some(items) = self.weapons_cache.lock().unwrap().as_ref() {
            return Ok(items.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<LichWeapon>>>(
                ApiVersion::V2,
                Method::GET,
                "/lich/weapons",
                "GET:lich:weapons",
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
     * - `Ok(LichWeapon)` if the weapon was found
     * - `Err(ApiError)` if there was an error fetching the weapon
     */
    pub async fn get_weapon_by_slug(&self, slug: &str) -> Result<LichWeapon, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<LichWeapon>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/lich/weapon/{}", slug),
                "GET:lich:weapon:slug",
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
     * Returns a `Result<Vec<LichEphemera>, ApiError>` containing the list of ephemeras or an error.
     */
    pub async fn get_all_ephemeras(&self) -> Result<Vec<LichEphemera>, ApiError> {
        if let Some(items) = self.ephemeras_cache.lock().unwrap().as_ref() {
            return Ok(items.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<LichEphemera>>>(
                ApiVersion::V2,
                Method::GET,
                "/lich/ephemeras",
                "GET:lich:ephemeras",
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
     * Returns a `Result<Vec<LichQuirk>, ApiError>` containing the list of quirks or an error.
     */
    pub async fn get_all_quirks(&self) -> Result<Vec<LichQuirk>, ApiError> {
        if let Some(items) = self.quirks_cache.lock().unwrap().as_ref() {
            return Ok(items.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<LichQuirk>>>(
                ApiVersion::V2,
                Method::GET,
                "/lich/quirks",
                "GET:lich:quirks",
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
     * Creates a new `LichRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &LichRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            ephemeras_cache: Mutex::new(old.ephemeras_cache.lock().unwrap().clone()),
            quirks_cache: Mutex::new(old.quirks_cache.lock().unwrap().clone()),
            weapons_cache: Mutex::new(old.weapons_cache.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}
