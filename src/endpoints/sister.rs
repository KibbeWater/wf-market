use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;

use crate::{client::Client, enums::*, errors::*, types::*};

#[derive(Debug)]
pub struct SisterRoute<State> {
    ephemeras_cache: Mutex<Vec<SisterEphemera>>,
    quirks_cache: Mutex<Vec<SisterQuirk>>,
    weapons_cache: Mutex<Vec<SisterWeapon>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> SisterRoute<State> {
    /**
     * Creates a new `SisterRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            ephemeras_cache: Mutex::new(Vec::new()),
            quirks_cache: Mutex::new(Vec::new()),
            weapons_cache: Mutex::new(Vec::new()),
            client: Arc::downgrade(&client),
        })
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
