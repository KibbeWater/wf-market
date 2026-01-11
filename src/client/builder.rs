//! Client builder for configuring the API client.

use std::sync::Arc;
use std::time::Duration;

use crate::cache::ApiCache;
use crate::error::{Error, Result};
use crate::internal::{
    build_default_rate_limiter, build_http_client, build_rate_limiter, fetch_items_internal,
};
use crate::models::{Item, ItemIndex, Language, Platform};

use super::{Client, ClientConfig, Unauthenticated};

/// Default maximum age for cached items (1 day).
const DEFAULT_CACHE_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// Builder for creating a [`Client`].
///
/// # Example
///
/// ```ignore
/// use wf_market::{Client, Platform, Language};
///
/// // Simple async construction (fetches items from API)
/// let client = Client::builder()
///     .platform(Platform::Ps4)
///     .language(Language::German)
///     .build()
///     .await?;
///
/// // With cache (uses cached items if fresh, otherwise fetches)
/// let mut cache = load_cache_from_disk()?;
/// let client = Client::builder()
///     .build_with_cache(&mut cache)
///     .await?;
///
/// // With pre-loaded items (sync, no API call)
/// let items = vec![/* pre-loaded items */];
/// let client = Client::builder()
///     .build_with_items(items)?;
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

    /// Build the client, fetching items from the API.
    ///
    /// This is the simplest way to create a client. It fetches the item
    /// list from the API on every call. For better performance with
    /// persistent caching, use [`build_with_cache`](Self::build_with_cache).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// let client = Client::builder().build().await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP client cannot be created
    /// - The items cannot be fetched from the API
    pub async fn build(self) -> Result<Client<Unauthenticated>> {
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

        // Fetch items from API
        let items = fetch_items_internal(&http).await?;
        let item_index = Arc::new(ItemIndex::new(items));

        Ok(Client::new_unauthenticated(
            http,
            self.config,
            limiter,
            item_index,
        ))
    }

    /// Build the client using cached items if available and fresh.
    ///
    /// If the cache contains items less than 1 day old, they will be used.
    /// Otherwise, items are fetched from the API and the cache is updated.
    ///
    /// This is the recommended way to create a client when you have
    /// persistent cache storage.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, ApiCache, SerializableCache};
    ///
    /// // Load cache from disk (or create new)
    /// let mut cache = match std::fs::read_to_string("cache.json") {
    ///     Ok(json) => serde_json::from_str::<SerializableCache>(&json)?
    ///         .into_api_cache(),
    ///     Err(_) => ApiCache::new(),
    /// };
    ///
    /// // Build client (uses cache if fresh, otherwise fetches)
    /// let client = Client::builder()
    ///     .build_with_cache(&mut cache)
    ///     .await?;
    ///
    /// // Save cache for next time
    /// let serializable = SerializableCache::from(&cache);
    /// std::fs::write("cache.json", serde_json::to_string(&serializable)?)?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP client cannot be created
    /// - Items need to be fetched and the API call fails
    pub async fn build_with_cache(self, cache: &mut ApiCache) -> Result<Client<Unauthenticated>> {
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

        // Check if cache has fresh items
        let items = if cache.has_fresh_items(DEFAULT_CACHE_MAX_AGE) {
            // Use cached items
            cache.get_items().unwrap().to_vec()
        } else {
            // Fetch from API and update cache
            let items = fetch_items_internal(&http).await?;
            cache.set_items(items.clone());
            items
        };

        let item_index = Arc::new(ItemIndex::new(items));

        Ok(Client::new_unauthenticated(
            http,
            self.config,
            limiter,
            item_index,
        ))
    }

    /// Build the client with pre-loaded items (synchronous).
    ///
    /// This method does not make any API calls. Use it when you already
    /// have item data from another source (e.g., a file, database, or
    /// previous API call).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// // Load items from your own storage
    /// let items: Vec<Item> = load_items_from_file()?;
    ///
    /// // Build client synchronously
    /// let client = Client::builder()
    ///     .build_with_items(items)?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn build_with_items(self, items: Vec<Item>) -> Result<Client<Unauthenticated>> {
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

        let item_index = Arc::new(ItemIndex::new(items));

        Ok(Client::new_unauthenticated(
            http,
            self.config,
            limiter,
            item_index,
        ))
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
    fn test_builder_build_with_items() {
        let client = ClientBuilder::new().build_with_items(vec![]);
        assert!(client.is_ok());
        assert_eq!(client.unwrap().items().len(), 0);
    }
}
