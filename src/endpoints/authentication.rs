use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use reqwest::Method;
use serde_json::Value;

use crate::{
    client::{Client, IsUnauthenticated},
    enums::ApiVersion,
    errors::AuthError,
    types::*,
};

#[derive(Debug)]
pub struct AuthenticationRoute<State> {
    client: Weak<Client<State>>,
}

impl<State> AuthenticationRoute<State> {
    /**
     * Creates a new `AuthenticationRoute` with an empty Authentication list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            client: Arc::downgrade(&client),
        })
    }

    /**
     * Creates a new `AuthenticationRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(
        _old: &AuthenticationRoute<T>,
        client: Arc<Client<State>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            client: Arc::downgrade(&client),
        })
    }
}

impl<State> AuthenticationRoute<State>
where
    State: IsUnauthenticated + Clone + 'static,
{
    pub async fn signin(
        &self,
        username: &str,
        password: &str,
        device_id: &str,
    ) -> Result<(SigninResponse, String), AuthError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        let mut map = HashMap::new();
        map.insert("auth_type", "header");
        map.insert("email", username);
        map.insert("password", password);
        map.insert("device_id", device_id);

        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(
                ApiVersion::V1,
                Method::POST,
                "/auth/signin",
                Some(serde_json::to_value(map).unwrap()),
                Some(HashMap::from([(
                    "Authorization".to_string(),
                    "JWT".to_string(),
                )])),
            )
            .await
        {
            Ok((login_response, headers)) => {
                // Get Payload from the response
                let user_value = login_response.payload.get("user").ok_or_else(|| {
                    AuthError::ParsingError("Missing 'user' field in response".to_string())
                })?;
                let user =
                    serde_json::from_value::<SigninResponse>(user_value.clone()).map_err(|e| {
                        AuthError::ParsingError(format!("Failed to parse user data: {}", e))
                    })?;

                let token = match headers.get("Authorization") {
                    Some(auth) => {
                        let t: String = auth
                            .to_str()
                            .map_err(|_| {
                                AuthError::ParsingError("Invalid token format".to_string())
                            })?
                            .to_string();
                        let jwt = &t[4..]; // Remove the "JWT " from the token.
                        jwt.to_string()
                    }
                    None => return Err(AuthError::NoUser),
                };

                Ok((user, token))
            }
            Err(e) => {
                // Handle error in login response
                eprintln!("Login failed: {:?}", e);
                return Err(AuthError::Unknown(format!("Login failed: {:?}", e)));
            }
        }
    }
}
