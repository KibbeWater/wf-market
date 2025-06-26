use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;

use crate::{client::Client, enums::*, errors::*, types::*};

#[derive(Debug)]
pub struct RivenRoute<State> {
    rivens_cache: Mutex<Vec<Riven>>,
    attributes_cache: Mutex<Vec<RivenAttribute>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> RivenRoute<State> {
    /**
     * Creates a new `RivenRoute` with an empty order list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            rivens_cache: Mutex::new(Vec::new()),
            attributes_cache: Mutex::new(Vec::new()),
            client: Arc::downgrade(&client),            
        })
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
