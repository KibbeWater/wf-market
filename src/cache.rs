//! Caching utilities for API responses.
//!
//! This module provides an explicit caching mechanism for API data that
//! rarely changes, such as the items list and riven weapons.
//!
//! # Why Cache?
//!
//! Some warframe.market endpoints return large datasets that rarely change:
//!
//! - **Items** (~4000 items): Only changes when new items are added to the game
//! - **Rivens** (~300 weapons): Only changes when new weapons are added
//!
//! Using a cache significantly reduces API calls and improves performance.
//!
//! # Usage
//!
//! The cache is **opt-in** and **user-controlled**. You decide when to use
//! caching and how long to keep cached data.
//!
//! ```ignore
//! use wf_market::{Client, ApiCache};
//! use std::time::Duration;
//!
//! async fn example() -> wf_market::Result<()> {
//!     let client = Client::builder().build()?;
//!     let mut cache = ApiCache::new();
//!
//!     // First call fetches from API and caches
//!     let items = client.get_items(Some(&mut cache)).await?;
//!
//!     // Subsequent calls use cached data
//!     let items = client.get_items(Some(&mut cache)).await?;
//!
//!     // With TTL - refreshes if cache is older than specified duration
//!     let items = client.get_items_with_ttl(
//!         Some(&mut cache),
//!         Duration::from_secs(24 * 60 * 60) // 24 hours
//!     ).await?;
//!
//!     // Opt-out: fetch directly without caching
//!     let items = client.get_items(None).await?;
//!     // Or explicitly:
//!     let items = client.fetch_items().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Cache Persistence
//!
//! The cache can be serialized for persistence across application restarts:
//!
//! ```ignore
//! use wf_market::ApiCache;
//!
//! // Save cache to disk
//! fn save_cache(cache: &ApiCache) -> std::io::Result<()> {
//!     let json = serde_json::to_string(cache)?;
//!     std::fs::write("cache.json", json)?;
//!     Ok(())
//! }
//!
//! // Load cache from disk
//! fn load_cache() -> std::io::Result<ApiCache> {
//!     let json = std::fs::read_to_string("cache.json")?;
//!     Ok(serde_json::from_str(&json)?)
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::models::{Item, Riven};

/// Cache for slowly-changing API data.
///
/// Use this to cache items and rivens data between requests.
/// The cache is completely user-controlled - you decide when to
/// use it and when to invalidate it.
#[derive(Debug, Default)]
pub struct ApiCache {
    items: Option<CacheEntry<Vec<Item>>>,
    rivens: Option<CacheEntry<Vec<Riven>>>,
}

/// A cached data entry with metadata.
#[derive(Debug)]
struct CacheEntry<T> {
    data: T,
    fetched_at: Instant,
}

impl ApiCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if items are cached.
    pub fn has_items(&self) -> bool {
        self.items.is_some()
    }

    /// Check if rivens are cached.
    pub fn has_rivens(&self) -> bool {
        self.rivens.is_some()
    }

    /// Get the age of the items cache.
    ///
    /// Returns `None` if items are not cached.
    pub fn items_age(&self) -> Option<Duration> {
        self.items.as_ref().map(|e| e.fetched_at.elapsed())
    }

    /// Get the age of the rivens cache.
    ///
    /// Returns `None` if rivens are not cached.
    pub fn rivens_age(&self) -> Option<Duration> {
        self.rivens.as_ref().map(|e| e.fetched_at.elapsed())
    }

    /// Clear all cached data.
    pub fn clear(&mut self) {
        self.items = None;
        self.rivens = None;
    }

    /// Clear the items cache.
    pub fn invalidate_items(&mut self) {
        self.items = None;
    }

    /// Clear the rivens cache.
    pub fn invalidate_rivens(&mut self) {
        self.rivens = None;
    }

    /// Invalidate items cache if older than the given duration.
    ///
    /// Returns `true` if the cache was invalidated.
    pub fn invalidate_items_if_older_than(&mut self, max_age: Duration) -> bool {
        if self.items_age().is_some_and(|age| age > max_age) {
            self.invalidate_items();
            true
        } else {
            false
        }
    }

    /// Invalidate rivens cache if older than the given duration.
    ///
    /// Returns `true` if the cache was invalidated.
    pub fn invalidate_rivens_if_older_than(&mut self, max_age: Duration) -> bool {
        if self.rivens_age().is_some_and(|age| age > max_age) {
            self.invalidate_rivens();
            true
        } else {
            false
        }
    }

    // Internal methods for Client use

    pub(crate) fn get_items(&self) -> Option<&[Item]> {
        self.items.as_ref().map(|e| e.data.as_slice())
    }

    pub(crate) fn set_items(&mut self, items: Vec<Item>) {
        self.items = Some(CacheEntry {
            data: items,
            fetched_at: Instant::now(),
        });
    }

    pub(crate) fn get_rivens(&self) -> Option<&[Riven]> {
        self.rivens.as_ref().map(|e| e.data.as_slice())
    }

    pub(crate) fn set_rivens(&mut self, rivens: Vec<Riven>) {
        self.rivens = Some(CacheEntry {
            data: rivens,
            fetched_at: Instant::now(),
        });
    }
}

/// Serializable cache for persistence.
///
/// This is a separate type that can be serialized/deserialized,
/// unlike `ApiCache` which uses `Instant` (not serializable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCache {
    /// Cached items (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<CachedItems>,

    /// Cached rivens (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rivens: Option<CachedRivens>,
}

/// Serializable cached items data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedItems {
    /// The cached items
    pub data: Vec<Item>,

    /// Unix timestamp when the data was fetched
    pub fetched_at_unix: u64,
}

/// Serializable cached rivens data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedRivens {
    /// The cached rivens
    pub data: Vec<Riven>,

    /// Unix timestamp when the data was fetched
    pub fetched_at_unix: u64,
}

impl SerializableCache {
    /// Create a new empty serializable cache.
    pub fn new() -> Self {
        Self {
            items: None,
            rivens: None,
        }
    }

    /// Convert to an `ApiCache`.
    ///
    /// Note: The `fetched_at` times will be approximated based on the
    /// stored Unix timestamps.
    pub fn into_api_cache(self) -> ApiCache {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut cache = ApiCache::new();

        if let Some(items) = self.items {
            // Calculate how old the cache is
            let age_secs = now.saturating_sub(items.fetched_at_unix);
            cache.items = Some(CacheEntry {
                data: items.data,
                // Approximate the Instant by subtracting the age from now
                fetched_at: Instant::now() - Duration::from_secs(age_secs),
            });
        }

        if let Some(rivens) = self.rivens {
            let age_secs = now.saturating_sub(rivens.fetched_at_unix);
            cache.rivens = Some(CacheEntry {
                data: rivens.data,
                fetched_at: Instant::now() - Duration::from_secs(age_secs),
            });
        }

        cache
    }
}

impl Default for SerializableCache {
    fn default() -> Self {
        Self::new()
    }
}

impl From<&ApiCache> for SerializableCache {
    fn from(cache: &ApiCache) -> Self {
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            items: cache.items.as_ref().map(|e| {
                let age = e.fetched_at.elapsed().as_secs();
                CachedItems {
                    data: e.data.clone(),
                    fetched_at_unix: now_unix.saturating_sub(age),
                }
            }),
            rivens: cache.rivens.as_ref().map(|e| {
                let age = e.fetched_at.elapsed().as_secs();
                CachedRivens {
                    data: e.data.clone(),
                    fetched_at_unix: now_unix.saturating_sub(age),
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new_is_empty() {
        let cache = ApiCache::new();
        assert!(!cache.has_items());
        assert!(!cache.has_rivens());
    }

    #[test]
    fn test_cache_set_and_get_items() {
        let mut cache = ApiCache::new();
        cache.set_items(vec![]);

        assert!(cache.has_items());
        assert!(cache.get_items().is_some());
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = ApiCache::new();
        cache.set_items(vec![]);
        cache.set_rivens(vec![]);

        cache.invalidate_items();
        assert!(!cache.has_items());
        assert!(cache.has_rivens());

        cache.clear();
        assert!(!cache.has_rivens());
    }

    #[test]
    fn test_cache_age() {
        let mut cache = ApiCache::new();
        assert!(cache.items_age().is_none());

        cache.set_items(vec![]);

        let age = cache.items_age().unwrap();
        assert!(age < Duration::from_secs(1));
    }

    #[test]
    fn test_serializable_cache_roundtrip() {
        // Create an API cache and convert to serializable
        let mut cache = ApiCache::new();
        cache.set_items(vec![]);

        let serializable = SerializableCache::from(&cache);
        assert!(serializable.items.is_some());

        // Serialize and deserialize
        let json = serde_json::to_string(&serializable).unwrap();
        let restored: SerializableCache = serde_json::from_str(&json).unwrap();

        assert!(restored.items.is_some());
    }
}
