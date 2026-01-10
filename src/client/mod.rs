//! HTTP client for the warframe.market API.
//!
//! This module provides the [`Client`] type which is the main entry point
//! for interacting with the warframe.market API.
//!
//! # Type States
//!
//! The client uses a type-state pattern to provide compile-time safety:
//!
//! - [`Client<Unauthenticated>`]: Can only access public endpoints
//! - [`Client<Authenticated>`]: Can access all endpoints including user-specific ones
//!
//! # Example
//!
//! ```no_run
//! use wf_market::{Client, Credentials};
//!
//! async fn example() -> wf_market::Result<()> {
//!     // Create an unauthenticated client
//!     let client = Client::builder().build()?;
//!
//!     // Fetch public data
//!     let items = client.fetch_items().await?;
//!
//!     // Login to access authenticated endpoints
//!     let creds = Credentials::new("email", "password", Credentials::generate_device_id());
//!     let client = client.login(creds).await?;
//!
//!     // Now we can access user-specific endpoints
//!     let my_orders = client.my_orders().await?;
//!
//!     Ok(())
//! }
//! ```

mod auth;
mod builder;

pub use builder::*;

use std::marker::PhantomData;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::internal::ApiRateLimiter;
use crate::models::{Credentials, Language, Platform};

// Sealed trait pattern for auth states
mod private {
    pub trait Sealed {}
    impl Sealed for super::Unauthenticated {}
    impl Sealed for super::Authenticated {}
}

/// Marker trait for authentication states.
pub trait AuthState: private::Sealed {}

/// Unauthenticated state - can only access public endpoints.
#[derive(Debug, Clone, Copy)]
pub struct Unauthenticated;
impl AuthState for Unauthenticated {}

/// Authenticated state - can access all endpoints.
#[derive(Debug, Clone, Copy)]
pub struct Authenticated;
impl AuthState for Authenticated {}

/// Client configuration.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Gaming platform (default: PC)
    pub platform: Platform,
    /// Language for responses (default: English)
    pub language: Language,
    /// Enable cross-play orders (default: true)
    pub crossplay: bool,
    /// Rate limit (requests per second, default: 3)
    pub rate_limit: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            platform: Platform::Pc,
            language: Language::English,
            crossplay: true,
            rate_limit: 3,
        }
    }
}

/// HTTP client for the warframe.market API.
///
/// The client is parameterized by an authentication state:
///
/// - `Client<Unauthenticated>`: Can only access public endpoints
/// - `Client<Authenticated>`: Can access all endpoints
///
/// Use [`Client::builder()`] to create a new client.
pub struct Client<S: AuthState = Unauthenticated> {
    pub(crate) http: reqwest::Client,
    pub(crate) config: ClientConfig,
    pub(crate) limiter: Arc<ApiRateLimiter>,
    pub(crate) credentials: Option<Credentials>,
    pub(crate) _state: PhantomData<S>,
}

impl Client<Unauthenticated> {
    /// Create a new client builder.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::{Client, Platform, Language};
    ///
    /// let client = Client::builder()
    ///     .platform(Platform::Pc)
    ///     .language(Language::English)
    ///     .build()?;
    /// # Ok::<(), wf_market::Error>(())
    /// ```
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Create a client and login in one step.
    ///
    /// This is a convenience method equivalent to:
    /// ```no_run
    /// # use wf_market::{Client, Credentials};
    /// # async fn example() -> wf_market::Result<()> {
    /// # let credentials = Credentials::new("", "", "");
    /// let client = Client::builder().build()?.login(credentials).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::{Client, Credentials};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let creds = Credentials::new("email", "password", Credentials::generate_device_id());
    ///     let client = Client::from_credentials(creds).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn from_credentials(credentials: Credentials) -> Result<Client<Authenticated>> {
        Self::builder().build()?.login(credentials).await
    }

    /// Create a client with custom config and login in one step.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::{Client, ClientConfig, Credentials, Platform};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let config = ClientConfig {
    ///         platform: Platform::Ps4,
    ///         ..Default::default()
    ///     };
    ///     let creds = Credentials::new("email", "password", Credentials::generate_device_id());
    ///     let client = Client::from_credentials_with_config(creds, config).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn from_credentials_with_config(
        credentials: Credentials,
        config: ClientConfig,
    ) -> Result<Client<Authenticated>> {
        Self::builder()
            .config(config)
            .build()?
            .login(credentials)
            .await
    }

    /// Validate credentials without fully logging in.
    ///
    /// This is useful for checking if a saved token is still valid
    /// before creating a full authenticated client.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::{Client, Credentials};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let saved = Credentials::from_token("email", "device-id", "saved-token");
    ///
    ///     if Client::validate_credentials(&saved).await? {
    ///         let client = Client::from_credentials(saved).await?;
    ///         println!("Session restored!");
    ///     } else {
    ///         println!("Token expired, please login again");
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn validate_credentials(credentials: &Credentials) -> Result<bool> {
        use crate::internal::{BASE_URL, build_authenticated_client};

        if let Some(token) = credentials.token() {
            let http = build_authenticated_client(Platform::Pc, Language::English, true, token)
                .map_err(Error::Network)?;

            match http.get(format!("{}/me", BASE_URL)).send().await {
                Ok(resp) if resp.status().is_success() => Ok(true),
                Ok(resp) if resp.status() == reqwest::StatusCode::UNAUTHORIZED => Ok(false),
                Ok(resp) => Err(Error::api(
                    resp.status(),
                    format!("Unexpected response: {}", resp.status()),
                )),
                Err(e) => Err(Error::Network(e)),
            }
        } else {
            // Password-based credentials can't be validated without logging in
            // Return true since we'll find out during login if they're invalid
            Ok(true)
        }
    }

    /// Create an unauthenticated client (internal use).
    pub(crate) fn new_unauthenticated(
        http: reqwest::Client,
        config: ClientConfig,
        limiter: Arc<ApiRateLimiter>,
    ) -> Self {
        Self {
            http,
            config,
            limiter,
            credentials: None,
            _state: PhantomData,
        }
    }
}

impl<S: AuthState> Client<S> {
    /// Get the client configuration.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Get the platform this client is configured for.
    pub fn platform(&self) -> Platform {
        self.config.platform
    }

    /// Get the language this client is configured for.
    pub fn language(&self) -> Language {
        self.config.language
    }

    /// Wait for rate limiter before making a request.
    pub(crate) async fn wait_for_rate_limit(&self) {
        self.limiter.until_ready().await;
    }
}

impl Client<Authenticated> {
    /// Get the current credentials (with token) for persistence.
    ///
    /// The returned credentials can be serialized and stored, then
    /// used with [`Credentials::from_token()`] to restore the session.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::{Client, Credentials};
    ///
    /// async fn save_session(client: &Client<wf_market::client::Authenticated>) -> std::io::Result<()> {
    ///     let creds = client.credentials();
    ///     let json = serde_json::to_string(creds)?;
    ///     std::fs::write("session.json", json)?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Panics
    ///
    /// This method uses `expect()` internally but should never panic because
    /// `Client<Authenticated>` can only be constructed with valid credentials.
    /// If this panics, it indicates an internal invariant violation.
    pub fn credentials(&self) -> &Credentials {
        self.credentials
            .as_ref()
            .expect("Authenticated client must have credentials")
    }

    /// Export credentials for saving (convenience method).
    ///
    /// This clones the credentials so they can be serialized and stored.
    pub fn export_session(&self) -> Credentials {
        self.credentials().clone()
    }

    /// Get the authentication token.
    ///
    /// # Panics
    ///
    /// This method uses `expect()` internally but should never panic because
    /// `Client<Authenticated>` can only be constructed with valid credentials
    /// that include a token. If this panics, it indicates an internal invariant violation.
    pub fn token(&self) -> &str {
        self.credentials()
            .token()
            .expect("Authenticated client must have token")
    }

    /// Get the device ID.
    pub fn device_id(&self) -> &str {
        &self.credentials().device_id
    }

    /// Create an authenticated client (internal use).
    pub(crate) fn new_authenticated(
        http: reqwest::Client,
        config: ClientConfig,
        limiter: Arc<ApiRateLimiter>,
        credentials: Credentials,
    ) -> Self {
        Self {
            http,
            config,
            limiter,
            credentials: Some(credentials),
            _state: PhantomData,
        }
    }
}

// Clone implementations
impl Clone for Client<Unauthenticated> {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            config: self.config.clone(),
            limiter: self.limiter.clone(),
            credentials: None,
            _state: PhantomData,
        }
    }
}

impl Clone for Client<Authenticated> {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            config: self.config.clone(),
            limiter: self.limiter.clone(),
            credentials: self.credentials.clone(),
            _state: PhantomData,
        }
    }
}
