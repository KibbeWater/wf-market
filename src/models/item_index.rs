//! Item index for efficient lookups.
//!
//! This module provides [`ItemIndex`], a data structure for O(1) item lookups
//! by ID or slug. The index is built once from a list of items and provides
//! fast access for order-to-item resolution.

use std::collections::HashMap;

use crate::error::Result;
use crate::internal::http::{build_http_client, fetch_items_internal};
use crate::models::{Language, Platform};

use super::Item;

/// An indexed collection of items for efficient lookups.
///
/// `ItemIndex` stores items and maintains hash maps for O(1) lookups
/// by item ID or slug. This is used internally by the client to enable
/// `order.get_item()` without requiring additional parameters.
///
/// # Fetching Without a Client
///
/// You can fetch items and build an index without creating a client first:
///
/// ```ignore
/// use wf_market::{Client, ItemIndex};
///
/// // Fetch items independently (no client needed)
/// let index = ItemIndex::fetch().await?;
/// println!("Fetched {} items", index.len());
///
/// // Build client synchronously with pre-fetched items
/// let client = Client::builder().build_with_items(index);
/// ```
///
/// # Building from Existing Items
///
/// ```ignore
/// use wf_market::ItemIndex;
///
/// let items = client.fetch_items().await?;
/// let index = ItemIndex::new(items);
///
/// // O(1) lookup by ID
/// if let Some(item) = index.get_by_id("some-item-id") {
///     println!("Found: {}", item.name());
/// }
///
/// // O(1) lookup by slug
/// if let Some(item) = index.get_by_slug("nikana_prime_set") {
///     println!("Found: {}", item.name());
/// }
/// ```
#[derive(Debug)]
pub struct ItemIndex {
    items: Vec<Item>,
    by_id: HashMap<String, usize>,
    by_slug: HashMap<String, usize>,
}

impl ItemIndex {
    /// Fetch items from the API and build an index.
    ///
    /// This is a standalone method that doesn't require a [`Client`](crate::Client).
    /// Useful for pre-fetching items before building a client with
    /// [`build_with_items`](crate::ClientBuilder::build_with_items).
    ///
    /// Uses default settings (PC platform, English language, crossplay enabled).
    /// For custom settings, use [`fetch_with_config`](Self::fetch_with_config).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, ItemIndex};
    ///
    /// // Fetch items independently
    /// let index = ItemIndex::fetch().await?;
    /// println!("Fetched {} items", index.len());
    ///
    /// // Build client synchronously with pre-fetched items
    /// let client = Client::builder().build_with_items(index);
    /// ```
    pub async fn fetch() -> Result<Self> {
        Self::fetch_with_config(Platform::Pc, Language::English, true).await
    }

    /// Fetch items from the API with custom configuration.
    ///
    /// This is a standalone method that doesn't require a [`Client`](crate::Client).
    ///
    /// # Arguments
    ///
    /// * `platform` - The gaming platform (PC, PlayStation, Xbox, Switch)
    /// * `language` - The language for item names and descriptions
    /// * `crossplay` - Whether to include crossplay data
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, ItemIndex, Platform, Language};
    ///
    /// // Fetch items with custom settings
    /// let index = ItemIndex::fetch_with_config(
    ///     Platform::Playstation,
    ///     Language::German,
    ///     false,
    /// ).await?;
    ///
    /// // Build client synchronously
    /// let client = Client::builder().build_with_items(index);
    /// ```
    pub async fn fetch_with_config(
        platform: Platform,
        language: Language,
        crossplay: bool,
    ) -> Result<Self> {
        let http = build_http_client(platform, language, crossplay)
            .map_err(crate::error::Error::Network)?;
        let items = fetch_items_internal(&http).await?;
        Ok(Self::new(items))
    }

    /// Create a new item index from a list of items.
    ///
    /// This builds the internal hash maps for O(1) lookups.
    /// Construction is O(n) where n is the number of items.
    pub fn new(items: Vec<Item>) -> Self {
        let mut by_id = HashMap::with_capacity(items.len());
        let mut by_slug = HashMap::with_capacity(items.len());

        for (idx, item) in items.iter().enumerate() {
            by_id.insert(item.id.clone(), idx);
            by_slug.insert(item.slug.clone(), idx);
        }

        Self {
            items,
            by_id,
            by_slug,
        }
    }

    /// Get an item by its unique ID.
    ///
    /// Returns `None` if no item with the given ID exists.
    pub fn get_by_id(&self, id: &str) -> Option<&Item> {
        self.by_id.get(id).and_then(|&idx| self.items.get(idx))
    }

    /// Get an item by its URL-friendly slug.
    ///
    /// Returns `None` if no item with the given slug exists.
    pub fn get_by_slug(&self, slug: &str) -> Option<&Item> {
        self.by_slug.get(slug).and_then(|&idx| self.items.get(idx))
    }

    /// Iterate over all items in the index.
    pub fn iter(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }

    /// Get the number of items in the index.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get all items as a slice.
    pub fn as_slice(&self) -> &[Item] {
        &self.items
    }

    /// Consume the index and return the underlying items.
    pub fn into_items(self) -> Vec<Item> {
        self.items
    }
}

impl Default for ItemIndex {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::item::ItemTranslation;
    use std::collections::HashMap as StdHashMap;

    fn make_test_item(id: &str, slug: &str, name: &str) -> Item {
        Item {
            id: id.to_string(),
            slug: slug.to_string(),
            game_ref: None,
            tradable: Some(true),
            tags: vec![],
            i18n: StdHashMap::from([(
                "en".to_string(),
                ItemTranslation {
                    name: name.to_string(),
                    icon: "/icons/test.png".to_string(),
                    thumb: None,
                    sub_icon: None,
                    description: None,
                    wiki_link: None,
                },
            )]),
            rarity: None,
            vaulted: None,
            ducats: None,
            trading_tax: None,
            mastery_rank: None,
            max_rank: None,
            max_charges: None,
            max_amber_stars: None,
            max_cyan_stars: None,
            base_endo: None,
            endo_multiplier: None,
            set_root: None,
            set_parts: None,
            quantity_in_set: None,
            bulk_tradable: None,
            subtypes: None,
        }
    }

    #[test]
    fn test_item_index_new() {
        let items = vec![
            make_test_item("id-1", "slug-1", "Item One"),
            make_test_item("id-2", "slug-2", "Item Two"),
        ];
        let index = ItemIndex::new(items);

        assert_eq!(index.len(), 2);
        assert!(!index.is_empty());
    }

    #[test]
    fn test_item_index_get_by_id() {
        let items = vec![
            make_test_item("id-1", "slug-1", "Item One"),
            make_test_item("id-2", "slug-2", "Item Two"),
        ];
        let index = ItemIndex::new(items);

        let item = index.get_by_id("id-1").expect("Should find item");
        assert_eq!(item.name(), "Item One");

        let item = index.get_by_id("id-2").expect("Should find item");
        assert_eq!(item.name(), "Item Two");

        assert!(index.get_by_id("nonexistent").is_none());
    }

    #[test]
    fn test_item_index_get_by_slug() {
        let items = vec![
            make_test_item("id-1", "slug-1", "Item One"),
            make_test_item("id-2", "slug-2", "Item Two"),
        ];
        let index = ItemIndex::new(items);

        let item = index.get_by_slug("slug-1").expect("Should find item");
        assert_eq!(item.name(), "Item One");

        let item = index.get_by_slug("slug-2").expect("Should find item");
        assert_eq!(item.name(), "Item Two");

        assert!(index.get_by_slug("nonexistent").is_none());
    }

    #[test]
    fn test_item_index_iter() {
        let items = vec![
            make_test_item("id-1", "slug-1", "Item One"),
            make_test_item("id-2", "slug-2", "Item Two"),
        ];
        let index = ItemIndex::new(items);

        let names: Vec<&str> = index.iter().map(|i| i.name()).collect();
        assert_eq!(names, vec!["Item One", "Item Two"]);
    }

    #[test]
    fn test_item_index_empty() {
        let index = ItemIndex::default();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        assert!(index.get_by_id("any").is_none());
        assert!(index.get_by_slug("any").is_none());
    }
}
