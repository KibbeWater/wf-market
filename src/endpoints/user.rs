use std::sync::{Arc, Weak};

use crate::client::{Client, IsUnauthenticated};

#[derive(Debug)]
pub struct UserRoute<State> {
    client: Weak<Client<State>>,
}

impl<State> UserRoute<State> {
    /**
     * Creates a new `UserRoute` with an empty User list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            client: Arc::downgrade(&client),
        })
    }

    /**
     * Creates a new `UserRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(_old: &UserRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            client: Arc::downgrade(&client),
        })
    }
}

impl<State> UserRoute<State> where State: IsUnauthenticated + Clone + 'static {}
