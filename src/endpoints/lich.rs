use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;

use crate::{client::Client, enums::*, errors::*, types::*};

#[derive(Debug)]
pub struct LichRoute<State> {
    ephemeras_cache: Mutex<Vec<LichEphemera>>,
    quirs_cache: Mutex<Vec<LichQuirk>>,
    weapons_cache: Mutex<Vec<LichWeapon>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> LichRoute<State> {
    /**
     * Creates a new `LichRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            ephemeras_cache: Mutex::new(Vec::new()),
            quirs_cache: Mutex::new(Vec::new()),
            weapons_cache: Mutex::new(Vec::new()),
            client: Arc::downgrade(&client),
        })
    }

    /**
     * Creates a new `LichRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &LichRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            ephemeras_cache: Mutex::new(old.ephemeras_cache.lock().unwrap().clone()),
            quirs_cache: Mutex::new(old.quirs_cache.lock().unwrap().clone()),
            weapons_cache: Mutex::new(old.weapons_cache.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}
