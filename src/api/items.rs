//! Items API endpoints.

use std::sync::Arc;
use std::time::Duration;

use crate::cache::ApiCache;
use crate::client::{AuthState, Client};
use crate::error::{ApiErrorResponse, Error, Result};
use crate::internal::BASE_URL;
use crate::models::{Item, ItemIndex, ItemSet};

use super::ApiResponse;

impl<S: AuthState> Client<S> {
    /// Fetch all tradable items directly from the API.
    ///
    /// This always makes a network request. Consider using [`get_items`](Self::get_items)
    /// with a cache for better performance.
    ///
    /// # Caching Recommendation
    ///
    /// This endpoint returns ~4000 items and the list rarely changes
    /// (only when new items are added to the game). Consider caching
    /// the result for 12-24 hours.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build().await?;
    ///     // Items are already loaded, but you can fetch fresh ones:
    ///     let items = client.fetch_items().await?;
    ///     println!("Found {} items", items.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn fetch_items(&self) -> Result<Vec<Item>> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/items", BASE_URL))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    "Failed to fetch items",
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch items: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Vec<Item>> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.data)
    }

    /// Get all tradable items, using cache if provided.
    ///
    /// If `cache` is `Some`, uses cached data if available, otherwise
    /// fetches from the API and populates the cache.
    ///
    /// If `cache` is `None`, fetches directly from the API (equivalent
    /// to [`fetch_items`](Self::fetch_items)).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, ApiCache};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build().await?;
    ///     let mut cache = ApiCache::new();
    ///
    ///     // First call fetches from API
    ///     let items = client.get_items(Some(&mut cache)).await?;
    ///
    ///     // Second call uses cache
    ///     let items = client.get_items(Some(&mut cache)).await?;
    ///
    ///     // Without cache
    ///     let items = client.get_items(None).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_items(&self, cache: Option<&mut ApiCache>) -> Result<Vec<Item>> {
        match cache {
            Some(c) => {
                if let Some(items) = c.get_items() {
                    return Ok(items.to_vec());
                }

                let items = self.fetch_items().await?;
                c.set_items(items.clone());
                Ok(items)
            }
            None => self.fetch_items().await,
        }
    }

    /// Get items with a maximum cache age (TTL).
    ///
    /// If the cache is older than `max_age`, it will be invalidated
    /// and fresh data will be fetched.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, ApiCache};
    /// use std::time::Duration;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build().await?;
    ///     let mut cache = ApiCache::new();
    ///
    ///     // Refresh if cache is older than 24 hours
    ///     let items = client.get_items_with_ttl(
    ///         Some(&mut cache),
    ///         Duration::from_secs(24 * 60 * 60),
    ///     ).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_items_with_ttl(
        &self,
        cache: Option<&mut ApiCache>,
        max_age: Duration,
    ) -> Result<Vec<Item>> {
        if let Some(c) = cache {
            c.invalidate_items_if_older_than(max_age);
            self.get_items(Some(c)).await
        } else {
            self.fetch_items().await
        }
    }

    /// Fetch a single item by its slug.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build().await?;
    ///     let item = client.get_item("nikana_prime_set").await?;
    ///     println!("Item: {}", item.name());
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_item(&self, slug: &str) -> Result<Item> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/item/{}", BASE_URL, slug))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("Item not found: {}", slug)));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    format!("Failed to fetch item: {}", slug),
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch item {}: {}", slug, body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Item> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.data)
    }

    /// Get all items in a set.
    ///
    /// Returns the complete set containing all parts for items that belong to a set.
    /// If the item is not part of a set, returns an array containing only that item.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build().await?;
    ///     let set = client.get_item_set("nikana_prime_set").await?;
    ///
    ///     println!("Set contains {} items:", set.len());
    ///     for item in &set.items {
    ///         println!("  - {} ({})", item.name(), item.slug);
    ///     }
    ///
    ///     // Get just the parts (excluding the set itself)
    ///     for part in set.parts() {
    ///         println!("Part: {}", part.name());
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_item_set(&self, slug: &str) -> Result<ItemSet> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/item/{}/set", BASE_URL, slug))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("Item not found: {}", slug)));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    format!("Failed to fetch item set: {}", slug),
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch item set {}: {}", slug, body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<ItemSet> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.data)
    }

    /// Refresh the client's item index with fresh data from the API.
    ///
    /// This fetches the latest item list and rebuilds the item index.
    /// Use this if you suspect the item list has changed (e.g., after
    /// a game update) or if item lookups are returning `None` unexpectedly.
    ///
    /// Note: Items are automatically loaded when the client is created,
    /// and the list rarely changes. You typically don't need to call this
    /// unless your application runs for extended periods.
    ///
    /// # Stale References
    ///
    /// **Important**: Orders fetched before calling this method will
    /// continue to reference the old item index due to `Arc` sharing.
    /// To ensure all orders use the latest index, re-fetch orders after
    /// calling this method.
    ///
    /// ```ignore
    /// let orders = client.get_orders("serration").await?;  // Uses index v1
    /// client.revalidate_items().await?;                     // Creates index v2
    /// // `orders` still references index v1 internally
    /// let fresh_orders = client.get_orders("serration").await?; // Uses index v2
    /// ```
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let mut client = Client::builder().build().await?;
    ///
    ///     // ... application runs for a long time ...
    ///
    ///     // Refresh items if needed
    ///     client.revalidate_items().await?;
    ///
    ///     println!("Refreshed {} items", client.items().len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn revalidate_items(&mut self) -> Result<()> {
        let items = self.fetch_items().await?;
        self.items = Arc::new(ItemIndex::new(items));
        Ok(())
    }
}
