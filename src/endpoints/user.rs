use std::sync::{Arc, Weak};

use crate::{
    client::{Client, IsAuthenticated},
    enums::*,
    errors::*,
    types::*,
};

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
                None,
                None,
            )
            .await
        {
            Ok((user, _headers)) => Ok(user.data),
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
                None,
                None,
            )
            .await
        {
            Ok((user, _headers)) => Ok(user.data),
            Err(e) => {
                return Err(e);
            }
        }
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

impl<State> UserRoute<State> where State: IsUnauthenticated + Clone + 'static {
    /**
     * Fetches the authenticated user's private profile.
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
                None,
                None,
            )
            .await
        {
            Ok((user, _headers)) => Ok(user.data),
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
    pub async fn update_profile(&self, args: UpdateUserPrivateParams) -> Result<UserPrivate, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<UserPrivate>>(
                ApiVersion::V2,
                Method::PATCH,
                "/me",
                Some(json!(args)),
                None,
            )
            .await
        {
            Ok((user, _headers)) => Ok(user.data),
            Err(e) => {
                return Err(e);
            }
        }
    }
}
