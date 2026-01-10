//! Rivens API endpoints.

use std::time::Duration;

use crate::cache::ApiCache;
use crate::client::{AuthState, Client};
use crate::error::{ApiErrorResponse, Error, Result};
use crate::internal::BASE_URL;
use crate::models::{Riven, RivenAttribute};

use super::ApiResponse;

impl<S: AuthState> Client<S> {
    /// Fetch all riven-compatible weapons directly from the API.
    ///
    /// This always makes a network request. Consider using [`get_rivens`](Self::get_rivens)
    /// with a cache for better performance.
    ///
    /// # Caching Recommendation
    ///
    /// This endpoint returns ~300 weapons and the list rarely changes
    /// (only when new weapons are added to the game). Consider caching
    /// the result for 12-24 hours.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let rivens = client.fetch_rivens().await?;
    ///     println!("Found {} riven weapons", rivens.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn fetch_rivens(&self) -> Result<Vec<Riven>> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/riven/weapons", BASE_URL))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    "Failed to fetch rivens",
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch rivens: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Vec<Riven>> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.data)
    }

    /// Get all riven-compatible weapons, using cache if provided.
    ///
    /// If `cache` is `Some`, uses cached data if available, otherwise
    /// fetches from the API and populates the cache.
    ///
    /// If `cache` is `None`, fetches directly from the API (equivalent
    /// to [`fetch_rivens`](Self::fetch_rivens)).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::{Client, ApiCache};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let mut cache = ApiCache::new();
    ///
    ///     // First call fetches from API
    ///     let rivens = client.get_rivens(Some(&mut cache)).await?;
    ///
    ///     // Second call uses cache
    ///     let rivens = client.get_rivens(Some(&mut cache)).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_rivens(&self, cache: Option<&mut ApiCache>) -> Result<Vec<Riven>> {
        match cache {
            Some(c) => {
                if let Some(rivens) = c.get_rivens() {
                    return Ok(rivens.to_vec());
                }

                let rivens = self.fetch_rivens().await?;
                c.set_rivens(rivens.clone());
                Ok(rivens)
            }
            None => self.fetch_rivens().await,
        }
    }

    /// Get rivens with a maximum cache age (TTL).
    ///
    /// If the cache is older than `max_age`, it will be invalidated
    /// and fresh data will be fetched.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::{Client, ApiCache};
    /// use std::time::Duration;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let mut cache = ApiCache::new();
    ///
    ///     // Refresh if cache is older than 24 hours
    ///     let rivens = client.get_rivens_with_ttl(
    ///         Some(&mut cache),
    ///         Duration::from_secs(24 * 60 * 60),
    ///     ).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_rivens_with_ttl(
        &self,
        cache: Option<&mut ApiCache>,
        max_age: Duration,
    ) -> Result<Vec<Riven>> {
        if let Some(c) = cache {
            c.invalidate_rivens_if_older_than(max_age);
            self.get_rivens(Some(c)).await
        } else {
            self.fetch_rivens().await
        }
    }

    /// Get a single riven weapon by slug.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let riven = client.get_riven("braton").await?;
    ///
    ///     println!("{}: disposition {} (tier {})",
    ///         riven.name(),
    ///         riven.disposition,
    ///         riven.disposition_tier()
    ///     );
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_riven(&self, slug: &str) -> Result<Riven> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/riven/weapon/{}", BASE_URL, slug))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!(
                "Riven weapon not found: {}",
                slug
            )));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    format!("Failed to fetch riven weapon: {}", slug),
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch riven weapon {}: {}", slug, body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Riven> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.data)
    }

    /// Get all riven attributes/stats.
    ///
    /// Returns all possible attributes that can appear on riven mods.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build()?;
    ///     let attributes = client.get_riven_attributes().await?;
    ///
    ///     for attr in &attributes {
    ///         println!("{}: {} / -{}", attr.name(), attr.prefix, attr.suffix);
    ///         if attr.is_inverted() {
    ///             println!("  (inverted - positive is bad)");
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_riven_attributes(&self) -> Result<Vec<RivenAttribute>> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/riven/attributes", BASE_URL))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    "Failed to fetch riven attributes",
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch riven attributes: {}", body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: ApiResponse<Vec<RivenAttribute>> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.data)
    }
}
