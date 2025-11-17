use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;

use crate::{client::Client, enums::ApiVersion, errors::ApiError, types::*};

#[derive(Debug)]
pub struct AchievementRoute<State> {
    achievement_cache: Mutex<Option<Vec<Achievement>>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> AchievementRoute<State> {
    /**
     * Creates a new `AchievementRoute` with an empty Authentication list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            achievement_cache: Mutex::new(None),
            client: Arc::downgrade(&client),
        })
    }
    /**
     * Fetches the achievements from the API.
     *
     * # Returns
     * - `Ok(Vec<Achievement>)` if the achievements were fetched successfully
     * - `Err(ApiError)` if there was an error fetching the achievements
     */
    pub async fn get_achievements(&self) -> Result<Vec<Achievement>, ApiError> {
        // Check if the achievements are already cached
        if let Some(achievements) = self.achievement_cache.lock().unwrap().as_ref() {
            return Ok(achievements.clone());
        }
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<Achievement>>>(
                ApiVersion::V2,
                Method::GET,
                "/achievements",
                "GET:achievements",
                None,
                None,
            )
            .await
        {
            Ok((user, _, _)) => {
                // Cache the versions response
                let mut cache = self.achievement_cache.lock().unwrap();
                *cache = Some(user.data.clone());
                Ok(user.data)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    /**
     * Fetches the achievements for a user by their slug.
     *
     * # Arguments
     * * `slug` - The slug of the user whose achievements are to be fetched.
     *
     * # Returns
     * A `Result` containing a vector of `Achievement` objects or an `ApiError`.
     */
    pub async fn get_achievements_for_user_by_slug(
        &self,
        slug: &str,
    ) -> Result<Vec<Achievement>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<Achievement>>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/achievements/user/{}", slug),
                "GET:achievements:user",
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
     * Fetches the achievements for a user by their user ID.
     *
     * # Arguments
     * * `user_id` - The user ID of the user whose achievements are to be fetched.
     *
     * # Returns
     * A `Result` containing a vector of `Achievement` objects or an `ApiError`.
     */
    pub async fn get_achievements_for_user_by_id(
        &self,
        user_id: &str,
    ) -> Result<Vec<Achievement>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV2<Vec<Achievement>>>(
                ApiVersion::V2,
                Method::GET,
                &format!("/achievements/userId/{}", user_id),
                "GET:achievements:userId",
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
     * Creates a new `AchievementRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &AchievementRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            achievement_cache: Mutex::new(old.achievement_cache.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}
