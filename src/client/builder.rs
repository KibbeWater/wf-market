//! Client builder for configuring the API client.

use crate::error::{Error, Result};
use crate::internal::{build_default_rate_limiter, build_http_client, build_rate_limiter};
use crate::models::{Language, Platform};

use super::{Client, ClientConfig, Unauthenticated};

/// Builder for creating a [`Client`].
///
/// # Example
///
/// ```no_run
/// use wf_market::{Client, Platform, Language};
///
/// let client = Client::builder()
///     .platform(Platform::Ps4)
///     .language(Language::German)
///     .crossplay(false)
///     .rate_limit(5)
///     .build()?;
/// # Ok::<(), wf_market::Error>(())
/// ```
#[derive(Debug, Clone, Default)]
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    /// Create a new client builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the gaming platform.
    ///
    /// Default: `Platform::Pc`
    pub fn platform(mut self, platform: Platform) -> Self {
        self.config.platform = platform;
        self
    }

    /// Set the response language.
    ///
    /// Default: `Language::English`
    pub fn language(mut self, language: Language) -> Self {
        self.config.language = language;
        self
    }

    /// Enable or disable cross-play orders.
    ///
    /// Default: `true`
    pub fn crossplay(mut self, enabled: bool) -> Self {
        self.config.crossplay = enabled;
        self
    }

    /// Set the rate limit (requests per second).
    ///
    /// Default: `3` (as per WFM API documentation)
    ///
    /// **Warning**: Setting this higher than 3 may result in rate limit errors.
    pub fn rate_limit(mut self, requests_per_second: u32) -> Self {
        self.config.rate_limit = requests_per_second;
        self
    }

    /// Set the entire client configuration.
    pub fn config(mut self, config: ClientConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn build(self) -> Result<Client<Unauthenticated>> {
        let http = build_http_client(
            self.config.platform,
            self.config.language,
            self.config.crossplay,
        )
        .map_err(Error::Network)?;

        let limiter = if self.config.rate_limit == 3 {
            build_default_rate_limiter()
        } else {
            build_rate_limiter(self.config.rate_limit)
        };

        Ok(Client::new_unauthenticated(http, self.config, limiter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = ClientBuilder::new();
        assert_eq!(builder.config.platform, Platform::Pc);
        assert_eq!(builder.config.language, Language::English);
        assert!(builder.config.crossplay);
        assert_eq!(builder.config.rate_limit, 3);
    }

    #[test]
    fn test_builder_chain() {
        let builder = ClientBuilder::new()
            .platform(Platform::Ps4)
            .language(Language::German)
            .crossplay(false)
            .rate_limit(5);

        assert_eq!(builder.config.platform, Platform::Ps4);
        assert_eq!(builder.config.language, Language::German);
        assert!(!builder.config.crossplay);
        assert_eq!(builder.config.rate_limit, 5);
    }

    #[test]
    fn test_builder_build() {
        let client = ClientBuilder::new().build();
        assert!(client.is_ok());
    }
}
