//! Authentication handling for the client.

use serde::{Deserialize, Serialize};

use crate::error::{ApiErrorResponse, Error, Result};
use crate::internal::{V1_API_URL, build_authenticated_client, build_rate_limiter};
use crate::models::{Credentials, FullUser};

use super::{Authenticated, Client, Unauthenticated};

/// Login request body.
#[derive(Serialize)]
struct LoginRequest<'a> {
    auth_type: &'static str,
    email: &'a str,
    password: &'a str,
    device_id: &'a str,
}

/// Login response from the V1 API.
#[derive(Deserialize)]
#[allow(dead_code)]
struct LoginResponse {
    payload: LoginPayload,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LoginPayload {
    user: FullUser,
}

impl Client<Unauthenticated> {
    /// Login with credentials to get an authenticated client.
    ///
    /// This method consumes the unauthenticated client and returns an
    /// authenticated one. The credentials will be updated with the
    /// authentication token for future session restoration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::{Client, Credentials};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build().await?;
    ///     
    ///     let creds = Credentials::new(
    ///         "user@example.com",
    ///         "password",
    ///         Credentials::generate_device_id(),
    ///     );
    ///     
    ///     let client = client.login(creds).await?;
    ///     
    ///     // Now we can access authenticated endpoints
    ///     let orders = client.my_orders().await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Token-based Login
    ///
    /// If the credentials already contain a token (from a previous session),
    /// the token will be validated and used directly without re-authenticating.
    ///
    /// ```no_run
    /// use wf_market::{Client, Credentials};
    ///
    /// async fn restore_session() -> wf_market::Result<()> {
    ///     let creds = Credentials::from_token(
    ///         "user@example.com",
    ///         "device-id",
    ///         "saved-jwt-token",
    ///     );
    ///     
    ///     let client = Client::builder().build().await?.login(creds).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn login(self, mut credentials: Credentials) -> Result<Client<Authenticated>> {
        let token = if let Some(token) = credentials.token() {
            // Token-based: use existing token
            token.to_string()
        } else if let Some(password) = credentials.password() {
            // Password-based: perform login
            let token = self
                .perform_login(&credentials.email, password, &credentials.device_id)
                .await?;
            credentials.set_token(token.clone());
            token
        } else {
            return Err(Error::auth(
                "Credentials must have either password or token",
            ));
        };

        // Build authenticated HTTP client
        let http = build_authenticated_client(
            self.config.platform,
            self.config.language,
            self.config.crossplay,
            &token,
        )
        .map_err(Error::Network)?;

        // Reuse or rebuild rate limiter
        let limiter = if self.config.rate_limit == 3 {
            self.limiter
        } else {
            build_rate_limiter(self.config.rate_limit)
        };

        Ok(Client::new_authenticated(
            http,
            self.config,
            limiter,
            credentials,
            self.items,
        ))
    }

    /// Perform the actual login request.
    async fn perform_login(&self, email: &str, password: &str, device_id: &str) -> Result<String> {
        let request = LoginRequest {
            auth_type: "header",
            email,
            password,
            device_id,
        };

        let response = self
            .http
            .post(format!("{}/auth/signin", V1_API_URL))
            .header("Authorization", "JWT")
            .json(&request)
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();
        let headers = response.headers().clone();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            // Try to parse as API error
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::auth_with_details(
                    format!("Login failed: {}", status),
                    error_response,
                ));
            }

            return Err(Error::auth(format!(
                "Login failed with status {}: {}",
                status, body
            )));
        }

        // Extract token from Authorization header
        let auth_header = headers
            .get("Authorization")
            .ok_or_else(|| Error::auth("No Authorization header in response"))?
            .to_str()
            .map_err(|_| Error::auth("Invalid Authorization header encoding"))?;

        // Token is prefixed with "JWT "
        let token = auth_header
            .strip_prefix("JWT ")
            .ok_or_else(|| Error::auth("Invalid Authorization header format"))?
            .to_string();

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_request_serialization() {
        let request = LoginRequest {
            auth_type: "header",
            email: "test@example.com",
            password: "password123",
            device_id: "device-123",
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"auth_type\":\"header\""));
        assert!(json.contains("\"email\":\"test@example.com\""));
    }
}
