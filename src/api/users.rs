//! User API endpoints.

use crate::client::{AuthState, Authenticated, Client};
use crate::error::{ApiErrorResponse, Error, Result};
use crate::internal::BASE_URL;
use crate::models::{UpdateProfile, UserPrivate, UserProfile};

use super::ApiResponse;

// === Public endpoints (both authenticated and unauthenticated) ===

impl<S: AuthState> Client<S> {
    /// Get a public user profile by slug.
    ///
    /// Returns the public profile information for any user.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let user = client.get_user("some_user").await?;
    ///
    ///     println!("User: {} ({})", user.ingame_name, user.platform);
    ///     println!("Reputation: {}", user.reputation);
    ///     if user.is_available() {
    ///         println!("Available for trading!");
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_user(&self, slug: &str) -> Result<UserProfile> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/user/{}", BASE_URL, slug))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("User not found: {}", slug)));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    format!("Failed to fetch user: {}", slug),
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch user {}: {}", slug, body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<UserProfile> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.data)
    }
}

// === Authenticated endpoints ===

impl Client<Authenticated> {
    /// Get the current user's private profile.
    ///
    /// Returns the full private profile including settings and account info.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, Credentials};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::from_credentials(/* ... */).await?;
    ///
    ///     let me = client.me().await?;
    ///     println!("Logged in as: {} ({})", me.ingame_name, me.email.as_deref().unwrap_or("no email"));
    ///     println!("Platform: {}, Crossplay: {}", me.platform, me.crossplay);
    ///     println!("Role: {:?}, Tier: {:?}", me.role, me.tier);
    ///     Ok(())
    /// }
    /// # fn main() {}
    /// ```
    pub async fn me(&self) -> Result<UserPrivate> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/me", BASE_URL))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::auth("Session expired or invalid"));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    "Failed to fetch current user profile",
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch current user: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<UserPrivate> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.data)
    }

    /// Update the current user's profile settings.
    ///
    /// Only include the fields you want to change in the [`UpdateProfile`].
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, Credentials, UpdateProfile, Platform, Theme};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::from_credentials(/* ... */).await?;
    ///
    ///     // Update platform and enable crossplay
    ///     let updated = client.update_me(
    ///         UpdateProfile::new()
    ///             .platform(Platform::Pc)
    ///             .crossplay(true)
    ///     ).await?;
    ///
    ///     println!("Updated! Now on {} with crossplay: {}", updated.platform, updated.crossplay);
    ///
    ///     // Update theme
    ///     let updated = client.update_me(
    ///         UpdateProfile::new().theme(Theme::Dark)
    ///     ).await?;
    ///
    ///     Ok(())
    /// }
    /// # fn main() {}
    /// ```
    pub async fn update_me(&self, update: UpdateProfile) -> Result<UserPrivate> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .patch(format!("{}/me", BASE_URL))
            .json(&update)
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::auth("Session expired or invalid"));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    "Failed to update profile",
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to update profile: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<UserPrivate> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.data)
    }
}
