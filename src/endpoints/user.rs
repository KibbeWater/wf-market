use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;
use serde_json::json;

use crate::{
    client::{Client, IsAuthenticated},
    enums::*,
    errors::*,
    types::*,
};

#[derive(Debug)]
pub struct UserRoute<State> {
    user: Mutex<Option<UserPrivate>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> UserRoute<State> {
    /**
     * Creates a new `UserRoute` with an empty User list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            user: Mutex::new(None), // Initialize with None
            client: Arc::downgrade(&client),
        })
    }
    /**
     * Fetches the a user by their slug.
     * # Returns
     * - `Ok(User)` if the user was found
     * - `Err(ApiError)` if there was an error fetching the user
     */
    pub async fn get_by_slug(&self, slug: &str) -> Result<User, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<User>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/user/{}", slug),
                "GET:user:slug",
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
     * Fetches the a user by their user ID.
     * # Returns
     * - `Ok(User)` if the user was found
     * - `Err(ApiError)` if there was an error fetching the user
     */
    pub async fn get_by_id(&self, user_id: &str) -> Result<User, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<User>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/userId/{}", user_id),
                "GET:user:user_id",
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
     * Creates a new `UserRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &UserRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            user: Mutex::new(old.user.lock().unwrap().clone()), // Clone the user state
            client: Arc::downgrade(&client),
        })
    }
}

impl<State> UserRoute<State>
where
    State: IsAuthenticated + Clone + 'static,
{
    /**
     * Returns the current user's private profile.
     * This is a convenience method that calls `me()` and returns the user data.
     * # Returns
     * - `Ok(UserPrivate)` if the user was found
     * - `Err(ApiError)` if there was an error fetching the user
     */
    pub fn get_user(&self) -> Result<UserPrivate, ApiError> {
        let ca_orders = self.user.lock().unwrap();
        match &*ca_orders {
            Some(user) => Ok(user.clone()),
            None => Err(ApiError::Unknown(
                "User not found. Please call `me()` to fetch the user data.".to_string(),
            )),
        }
    }
    /**
     * Fetches the authenticated user's private profile.
     * Note: This method updates the internal user state with the fetched user data.
     * # Returns
     * - `Ok(UserPrivate)` if the user was found
     * - `Err(ApiError)` if there was an error fetching the user
     */
    pub async fn me(&self) -> Result<UserPrivate, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<UserPrivate>>(
                ApiVersion::V2,
                Method::GET,
                "/me",
                "GET:user:me",
                None,
                None,
            )
            .await
        {
            Ok((user, _, _)) => {
                // Update the user in the route
                let mut user_lock = self.user.lock().unwrap();
                *user_lock = Some(user.data.clone());
                Ok(user.data)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Updates the authenticated user's private profile.
     * # Arguments
     * - `args`: The parameters to update the user's private profile.
     * # Returns
     * - `Ok(UserPrivate)` if the update was successful
     * - `Err(ApiError)` if there was an error updating the user
     */
    pub async fn update_profile(
        &self,
        args: UpdateUserPrivateParams,
    ) -> Result<UserPrivate, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<UserPrivate>>(
                ApiVersion::V2,
                Method::PATCH,
                "/me",
                "PATCH:user:me",
                Some(json!(args)),
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
}
