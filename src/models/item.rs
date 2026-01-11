//! Item model and specialized views.
//!
//! This module provides the [`Item`] type representing tradable items
//! on warframe.market, along with specialized views for item subtypes
//! like mods ([`ModView`]) and Ayatan sculptures ([`SculptureView`]).
//!
//! # Example
//!
//! ```no_run
//! use wf_market::Client;
//!
//! async fn example() -> wf_market::Result<()> {
//!     let client = Client::builder().build()?;
//!     let items = client.fetch_items().await?;
//!
//!     for item in &items {
//!         // Check if it's a mod
//!         if let Some(mod_view) = item.as_mod() {
//!             println!("{}: max rank {}", item.name(), mod_view.max_rank());
//!         }
//!
//!         // Check if it's a sculpture
//!         if let Some(sculpture) = item.as_sculpture() {
//!             println!("{}: {} endo (full)", item.name(), sculpture.calculate_endo(None, None));
//!         }
//!     }
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::common::Rarity;

/// A tradable item on warframe.market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// Unique item identifier
    pub id: String,

    /// URL-friendly identifier (used in API endpoints)
    pub slug: String,

    /// Internal game reference path
    #[serde(rename = "gameRef", default)]
    pub game_ref: Option<String>,

    /// Whether the item can be traded
    #[serde(default)]
    pub tradable: Option<bool>,

    /// Item categorization tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Localized content (keyed by language code)
    #[serde(default)]
    pub i18n: HashMap<String, ItemTranslation>,

    /// Item rarity
    #[serde(default)]
    pub rarity: Option<Rarity>,

    /// Whether the item is vaulted (Prime items)
    #[serde(default)]
    pub vaulted: Option<bool>,

    /// Ducat value when sold to Baro Ki'Teer
    #[serde(default)]
    pub ducats: Option<u32>,

    /// Trading tax in credits
    #[serde(rename = "tradingTax", default)]
    pub trading_tax: Option<u32>,

    /// Required mastery rank to trade
    #[serde(rename = "reqMasteryRank", default)]
    pub mastery_rank: Option<u32>,

    // === Mod-specific fields ===
    /// Maximum mod rank (for rankable mods)
    #[serde(rename = "maxRank", default)]
    pub max_rank: Option<u32>,

    /// Maximum charges (for consumable mods like Requiem)
    #[serde(rename = "maxCharges", default)]
    pub max_charges: Option<u32>,

    // === Ayatan Sculpture fields ===
    /// Maximum amber stars (for Ayatan sculptures)
    #[serde(rename = "maxAmberStars", default)]
    pub max_amber_stars: Option<u32>,

    /// Maximum cyan stars (for Ayatan sculptures)
    #[serde(rename = "maxCyanStars", default)]
    pub max_cyan_stars: Option<u32>,

    /// Base endo value (for Ayatan sculptures)
    #[serde(rename = "baseEndo", default)]
    pub base_endo: Option<u32>,

    /// Endo multiplier per filled socket (for Ayatan sculptures)
    #[serde(rename = "endoMultiplier", default)]
    pub endo_multiplier: Option<f32>,

    // === Set-related fields ===
    /// Whether this item is the root of a set
    #[serde(rename = "setRoot", default)]
    pub set_root: Option<bool>,

    /// Item IDs that are part of this set (if set_root is true)
    #[serde(rename = "setParts", default)]
    pub set_parts: Option<Vec<String>>,

    /// Quantity of this item in its parent set
    #[serde(rename = "quantityInSet", default)]
    pub quantity_in_set: Option<u32>,

    /// Whether multiple items can be traded at once
    #[serde(rename = "bulkTradable", default)]
    pub bulk_tradable: Option<bool>,

    /// Available subtypes (e.g., blueprint, crafted)
    #[serde(default)]
    pub subtypes: Option<Vec<String>>,
}

/// Localized item content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTranslation {
    /// Localized item name
    pub name: String,

    /// Path to item icon (relative to static assets)
    pub icon: String,

    /// Path to item thumbnail
    #[serde(default)]
    pub thumb: Option<String>,

    /// Path to sub-icon (e.g., blueprint indicator)
    #[serde(rename = "subIcon", default)]
    pub sub_icon: Option<String>,

    /// Localized item description
    #[serde(default)]
    pub description: Option<String>,

    /// Link to the Warframe wiki page
    #[serde(rename = "wikiLink", default)]
    pub wiki_link: Option<String>,
}

impl Item {
    /// Get the English name of the item.
    ///
    /// Returns an empty string if no English translation is available.
    pub fn name(&self) -> &str {
        self.i18n.get("en").map(|t| t.name.as_str()).unwrap_or("")
    }

    /// Get the localized name for a specific language.
    ///
    /// Falls back to English if the requested language is not available.
    pub fn name_localized(&self, lang: &str) -> &str {
        self.i18n
            .get(lang)
            .or_else(|| self.i18n.get("en"))
            .map(|t| t.name.as_str())
            .unwrap_or("")
    }

    /// Get the English translation data.
    pub fn translation(&self) -> Option<&ItemTranslation> {
        self.i18n.get("en")
    }

    /// Get the icon URL (relative to static assets base).
    pub fn icon(&self) -> Option<&str> {
        self.i18n.get("en").map(|t| t.icon.as_str())
    }

    /// Get the wiki link if available.
    pub fn wiki_link(&self) -> Option<&str> {
        self.i18n.get("en").and_then(|t| t.wiki_link.as_deref())
    }

    /// Check if this item is a mod (has max_rank).
    pub fn is_mod(&self) -> bool {
        self.max_rank.is_some()
    }

    /// Check if this item is an Ayatan sculpture.
    pub fn is_sculpture(&self) -> bool {
        self.base_endo.is_some() && self.endo_multiplier.is_some()
    }

    /// Check if this item is a set (contains multiple parts).
    pub fn is_set(&self) -> bool {
        self.set_root.unwrap_or(false)
    }

    /// Check if this item is vaulted.
    pub fn is_vaulted(&self) -> bool {
        self.vaulted.unwrap_or(false)
    }

    /// Check if this item has charges (like Requiem mods).
    pub fn has_charges(&self) -> bool {
        self.max_charges.is_some()
    }

    /// Check if this item is a regular item (not a mod or sculpture).
    ///
    /// Regular items include prime parts, blueprints, relics, and other
    /// tradable items that don't have mod ranks or sculpture properties.
    pub fn is_regular(&self) -> bool {
        !self.is_mod() && !self.is_sculpture()
    }

    /// Get a mod-specific view of this item.
    ///
    /// Returns `Some(ModView)` if this item is a mod, `None` otherwise.
    /// The view provides type-safe access to mod-specific data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wf_market::models::Item;
    /// # fn example(item: &Item) {
    /// if let Some(mod_view) = item.as_mod() {
    ///     println!("Max rank: {}", mod_view.max_rank());
    ///     if let Some(charges) = mod_view.max_charges() {
    ///         println!("Max charges: {}", charges);
    ///     }
    /// }
    /// # }
    /// ```
    pub fn as_mod(&self) -> Option<ModView<'_>> {
        if self.max_rank.is_some() {
            Some(ModView(self))
        } else {
            None
        }
    }

    /// Get a sculpture-specific view of this item.
    ///
    /// Returns `Some(SculptureView)` if this item is an Ayatan sculpture,
    /// `None` otherwise. The view provides type-safe access to sculpture
    /// data and endo calculation.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wf_market::models::Item;
    /// # fn example(item: &Item) {
    /// if let Some(sculpture) = item.as_sculpture() {
    ///     let endo = sculpture.calculate_endo(None, None);
    ///     println!("Endo value (full): {}", endo);
    /// }
    /// # }
    /// ```
    pub fn as_sculpture(&self) -> Option<SculptureView<'_>> {
        if self.base_endo.is_some() && self.endo_multiplier.is_some() {
            Some(SculptureView(self))
        } else {
            None
        }
    }
}

/// A zero-cost view into mod-specific item data.
///
/// This type can only be constructed from an [`Item`] that is actually
/// a mod, providing compile-time guarantees that mod-specific methods
/// are only called on appropriate items.
#[derive(Debug, Clone, Copy)]
pub struct ModView<'a>(&'a Item);

impl<'a> ModView<'a> {
    /// Get the underlying item.
    pub fn item(&self) -> &Item {
        self.0
    }

    /// Get the maximum rank of this mod.
    ///
    /// This is guaranteed to return a value since `ModView` can only
    /// be constructed from items that have `max_rank`.
    pub fn max_rank(&self) -> u32 {
        self.0.max_rank.unwrap() // Safe: guaranteed by construction
    }

    /// Get the maximum charges (for consumable mods like Requiem).
    pub fn max_charges(&self) -> Option<u32> {
        self.0.max_charges
    }

    /// Check if this mod has charges.
    pub fn has_charges(&self) -> bool {
        self.0.max_charges.is_some()
    }
}

/// A zero-cost view into Ayatan sculpture-specific item data.
///
/// This type can only be constructed from an [`Item`] that is actually
/// an Ayatan sculpture, providing compile-time guarantees that sculpture
/// methods are only called on appropriate items.
#[derive(Debug, Clone, Copy)]
pub struct SculptureView<'a>(&'a Item);

impl<'a> SculptureView<'a> {
    /// Get the underlying item.
    pub fn item(&self) -> &Item {
        self.0
    }

    /// Get the base endo value before star multipliers.
    pub fn base_endo(&self) -> u32 {
        self.0.base_endo.unwrap() // Safe: guaranteed by construction
    }

    /// Get the endo multiplier per filled socket.
    pub fn endo_multiplier(&self) -> f32 {
        self.0.endo_multiplier.unwrap() // Safe: guaranteed by construction
    }

    /// Get the maximum amber star slots.
    pub fn max_amber_stars(&self) -> u32 {
        self.0.max_amber_stars.unwrap_or(0)
    }

    /// Get the maximum cyan star slots.
    pub fn max_cyan_stars(&self) -> u32 {
        self.0.max_cyan_stars.unwrap_or(0)
    }

    /// Get the total number of star slots.
    pub fn total_slots(&self) -> u32 {
        self.max_amber_stars() + self.max_cyan_stars()
    }

    /// Calculate the endo value of this sculpture with given stars.
    ///
    /// # Arguments
    ///
    /// * `cyan_stars` - Number of cyan stars installed. If `None`, uses max.
    /// * `amber_stars` - Number of amber stars installed. If `None`, uses max.
    ///
    /// # Formula
    ///
    /// The endo value is calculated as:
    /// ```text
    /// (base + 50*cyan + 100*amber) * (1 + multiplier * filled_slots / total_slots)
    /// ```
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use wf_market::models::Item;
    /// # fn example(item: &Item) {
    /// if let Some(sculpture) = item.as_sculpture() {
    ///     // Full value (all slots filled)
    ///     let full = sculpture.calculate_endo(None, None);
    ///
    ///     // Empty value (no stars)
    ///     let empty = sculpture.calculate_endo(Some(0), Some(0));
    ///
    ///     // Partial (2 cyan, 1 amber)
    ///     let partial = sculpture.calculate_endo(Some(2), Some(1));
    /// }
    /// # }
    /// ```
    pub fn calculate_endo(&self, cyan_stars: Option<u32>, amber_stars: Option<u32>) -> u32 {
        let base = self.base_endo() as f32;
        let multiplier = self.endo_multiplier();
        let total_slots = self.total_slots();

        if total_slots == 0 {
            return base as u32;
        }

        let cyan = cyan_stars.unwrap_or_else(|| self.max_cyan_stars());
        let amber = amber_stars.unwrap_or_else(|| self.max_amber_stars());

        let filled_slots = (cyan + amber) as f32;
        let base_value = base + 50.0 * (cyan as f32) + 100.0 * (amber as f32);
        let socket_factor = 1.0 + multiplier * filled_slots / (total_slots as f32);

        (base_value * socket_factor) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_item() -> Item {
        Item {
            id: "test-id".to_string(),
            slug: "test-slug".to_string(),
            game_ref: None,
            tradable: Some(true),
            tags: vec!["mod".to_string()],
            i18n: HashMap::from([(
                "en".to_string(),
                ItemTranslation {
                    name: "Test Item".to_string(),
                    icon: "/icons/test.png".to_string(),
                    thumb: None,
                    sub_icon: None,
                    description: None,
                    wiki_link: None,
                },
            )]),
            rarity: Some(Rarity::Rare),
            vaulted: None,
            ducats: None,
            trading_tax: None,
            mastery_rank: None,
            max_rank: Some(10),
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

    fn make_sculpture() -> Item {
        Item {
            id: "sculpture-id".to_string(),
            slug: "ayatan-anasa".to_string(),
            game_ref: None,
            tradable: Some(true),
            tags: vec!["ayatan".to_string()],
            i18n: HashMap::from([(
                "en".to_string(),
                ItemTranslation {
                    name: "Ayatan Anasa Sculpture".to_string(),
                    icon: "/icons/anasa.png".to_string(),
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
            max_amber_stars: Some(2),
            max_cyan_stars: Some(4),
            base_endo: Some(450),
            endo_multiplier: Some(0.9),
            set_root: None,
            set_parts: None,
            quantity_in_set: None,
            bulk_tradable: None,
            subtypes: None,
        }
    }

    #[test]
    fn test_item_name() {
        let item = make_test_item();
        assert_eq!(item.name(), "Test Item");
    }

    #[test]
    fn test_item_is_mod() {
        let item = make_test_item();
        assert!(item.is_mod());
        assert!(!item.is_sculpture());
    }

    #[test]
    fn test_mod_view() {
        let item = make_test_item();
        let mod_view = item.as_mod().expect("Should be a mod");
        assert_eq!(mod_view.max_rank(), 10);
    }

    #[test]
    fn test_sculpture_view() {
        let item = make_sculpture();
        assert!(item.is_sculpture());
        assert!(!item.is_mod());

        let sculpture = item.as_sculpture().expect("Should be a sculpture");
        assert_eq!(sculpture.base_endo(), 450);
        assert_eq!(sculpture.max_amber_stars(), 2);
        assert_eq!(sculpture.max_cyan_stars(), 4);
        assert_eq!(sculpture.total_slots(), 6);
    }

    #[test]
    fn test_sculpture_endo_calculation() {
        let item = make_sculpture();
        let sculpture = item.as_sculpture().unwrap();

        // Full value
        let full = sculpture.calculate_endo(None, None);
        assert!(full > sculpture.base_endo());

        // Empty value
        let empty = sculpture.calculate_endo(Some(0), Some(0));
        assert_eq!(empty, sculpture.base_endo());

        // Partial should be between empty and full
        let partial = sculpture.calculate_endo(Some(2), Some(1));
        assert!(partial > empty);
        assert!(partial < full);
    }

    #[test]
    fn test_non_mod_returns_none() {
        let item = make_sculpture();
        assert!(item.as_mod().is_none());
    }

    #[test]
    fn test_non_sculpture_returns_none() {
        let item = make_test_item();
        assert!(item.as_sculpture().is_none());
    }

    // === Realistic item helper functions ===

    fn make_archon_mod() -> Item {
        Item {
            id: "archon-flow-id".to_string(),
            slug: "archon_flow".to_string(),
            game_ref: Some("/Lotus/Upgrades/Mods/Warframe/AvatarArchonFlowMod".to_string()),
            tradable: Some(true),
            tags: vec!["mod".to_string(), "warframe".to_string()],
            i18n: HashMap::from([(
                "en".to_string(),
                ItemTranslation {
                    name: "Archon Flow".to_string(),
                    icon: "/icons/archon_flow.png".to_string(),
                    thumb: None,
                    sub_icon: None,
                    description: Some("Archon mod that increases energy capacity".to_string()),
                    wiki_link: Some("https://warframe.fandom.com/wiki/Archon_Flow".to_string()),
                },
            )]),
            rarity: Some(Rarity::Legendary),
            vaulted: None,
            ducats: None,
            trading_tax: Some(1_000_000),
            mastery_rank: None,
            max_rank: Some(5),
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

    fn make_requiem_mod() -> Item {
        Item {
            id: "requiem-lohk-id".to_string(),
            slug: "requiem_lohk".to_string(),
            game_ref: Some("/Lotus/Upgrades/Mods/Melee/Requiem/RequiemLohkMod".to_string()),
            tradable: Some(true),
            tags: vec!["mod".to_string(), "requiem".to_string()],
            i18n: HashMap::from([(
                "en".to_string(),
                ItemTranslation {
                    name: "Requiem Lohk".to_string(),
                    icon: "/icons/requiem_lohk.png".to_string(),
                    thumb: None,
                    sub_icon: None,
                    description: Some("A Requiem mod used to defeat Kuva Liches".to_string()),
                    wiki_link: Some("https://warframe.fandom.com/wiki/Requiem_Lohk".to_string()),
                },
            )]),
            rarity: Some(Rarity::Rare),
            vaulted: None,
            ducats: None,
            trading_tax: Some(8000),
            mastery_rank: None,
            max_rank: Some(0),
            max_charges: Some(3),
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

    fn make_standard_mod() -> Item {
        Item {
            id: "serration-id".to_string(),
            slug: "serration".to_string(),
            game_ref: Some("/Lotus/Upgrades/Mods/Rifle/DamageRifleMod".to_string()),
            tradable: Some(true),
            tags: vec!["mod".to_string(), "rifle".to_string()],
            i18n: HashMap::from([(
                "en".to_string(),
                ItemTranslation {
                    name: "Serration".to_string(),
                    icon: "/icons/serration.png".to_string(),
                    thumb: None,
                    sub_icon: None,
                    description: Some("Increases rifle damage".to_string()),
                    wiki_link: Some("https://warframe.fandom.com/wiki/Serration".to_string()),
                },
            )]),
            rarity: Some(Rarity::Uncommon),
            vaulted: None,
            ducats: None,
            trading_tax: Some(4000),
            mastery_rank: None,
            max_rank: Some(10),
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

    fn make_sah_sculpture() -> Item {
        Item {
            id: "ayatan-sah-id".to_string(),
            slug: "ayatan_sah_sculpture".to_string(),
            game_ref: Some("/Lotus/Types/Items/MiscItems/AyatanSahSculpture".to_string()),
            tradable: Some(true),
            tags: vec!["ayatan".to_string()],
            i18n: HashMap::from([(
                "en".to_string(),
                ItemTranslation {
                    name: "Ayatan Sah Sculpture".to_string(),
                    icon: "/icons/ayatan_sah.png".to_string(),
                    thumb: None,
                    sub_icon: None,
                    description: Some(
                        "An Ayatan sculpture that can be traded for endo".to_string(),
                    ),
                    wiki_link: Some(
                        "https://warframe.fandom.com/wiki/Ayatan_Sah_Sculpture".to_string(),
                    ),
                },
            )]),
            rarity: None,
            vaulted: None,
            ducats: None,
            trading_tax: Some(4000),
            mastery_rank: None,
            max_rank: None,
            max_charges: None,
            max_amber_stars: Some(1),
            max_cyan_stars: Some(2),
            base_endo: Some(600),
            endo_multiplier: Some(0.9),
            set_root: None,
            set_parts: None,
            quantity_in_set: None,
            bulk_tradable: None,
            subtypes: None,
        }
    }

    fn make_prime_part() -> Item {
        Item {
            id: "nikana-prime-blade-id".to_string(),
            slug: "nikana_prime_blade".to_string(),
            game_ref: Some("/Lotus/Types/Recipes/Weapons/NikanaPrimeBlade".to_string()),
            tradable: Some(true),
            tags: vec!["prime".to_string(), "melee".to_string()],
            i18n: HashMap::from([(
                "en".to_string(),
                ItemTranslation {
                    name: "Nikana Prime Blade".to_string(),
                    icon: "/icons/nikana_prime_blade.png".to_string(),
                    thumb: None,
                    sub_icon: None,
                    description: None,
                    wiki_link: Some("https://warframe.fandom.com/wiki/Nikana_Prime".to_string()),
                },
            )]),
            rarity: Some(Rarity::Uncommon),
            vaulted: Some(false),
            ducats: Some(45),
            trading_tax: Some(4000),
            mastery_rank: None,
            max_rank: None,
            max_charges: None,
            max_amber_stars: None,
            max_cyan_stars: None,
            base_endo: None,
            endo_multiplier: None,
            set_root: None,
            set_parts: None,
            quantity_in_set: Some(1),
            bulk_tradable: None,
            subtypes: None,
        }
    }

    fn make_blueprint() -> Item {
        Item {
            id: "ash-prime-blueprint-id".to_string(),
            slug: "ash_prime_blueprint".to_string(),
            game_ref: Some("/Lotus/Types/Recipes/Warframes/AshPrimeBlueprint".to_string()),
            tradable: Some(true),
            tags: vec!["prime".to_string(), "warframe".to_string()],
            i18n: HashMap::from([(
                "en".to_string(),
                ItemTranslation {
                    name: "Ash Prime Blueprint".to_string(),
                    icon: "/icons/ash_prime_blueprint.png".to_string(),
                    thumb: None,
                    sub_icon: Some("/icons/blueprint_overlay.png".to_string()),
                    description: None,
                    wiki_link: Some("https://warframe.fandom.com/wiki/Ash_Prime".to_string()),
                },
            )]),
            rarity: Some(Rarity::Rare),
            vaulted: Some(true),
            ducats: Some(100),
            trading_tax: Some(8000),
            mastery_rank: None,
            max_rank: None,
            max_charges: None,
            max_amber_stars: None,
            max_cyan_stars: None,
            base_endo: None,
            endo_multiplier: None,
            set_root: None,
            set_parts: None,
            quantity_in_set: Some(1),
            bulk_tradable: None,
            subtypes: None,
        }
    }

    // === Mod detection tests ===

    #[test]
    fn test_archon_mod_detection() {
        let item = make_archon_mod();
        assert!(item.is_mod(), "Archon Flow should be detected as a mod");
        assert!(
            !item.is_sculpture(),
            "Archon Flow should not be a sculpture"
        );
        assert!(
            !item.is_regular(),
            "Archon Flow should not be a regular item"
        );
        assert!(!item.has_charges(), "Archon Flow should not have charges");

        let mod_view = item
            .as_mod()
            .expect("Should return ModView for Archon Flow");
        assert_eq!(mod_view.max_rank(), 5);
    }

    #[test]
    fn test_requiem_mod_with_charges() {
        let item = make_requiem_mod();
        assert!(item.is_mod(), "Requiem Lohk should be detected as a mod");
        assert!(item.has_charges(), "Requiem Lohk should have charges");
        assert!(
            !item.is_sculpture(),
            "Requiem Lohk should not be a sculpture"
        );
        assert!(
            !item.is_regular(),
            "Requiem Lohk should not be a regular item"
        );

        let mod_view = item
            .as_mod()
            .expect("Should return ModView for Requiem Lohk");
        assert_eq!(mod_view.max_rank(), 0);
        assert_eq!(mod_view.max_charges(), Some(3));
        assert!(mod_view.has_charges());
    }

    #[test]
    fn test_standard_mod_detection() {
        let item = make_standard_mod();
        assert!(item.is_mod(), "Serration should be detected as a mod");
        assert!(!item.is_sculpture(), "Serration should not be a sculpture");
        assert!(!item.is_regular(), "Serration should not be a regular item");

        let mod_view = item.as_mod().expect("Should return ModView for Serration");
        assert_eq!(mod_view.max_rank(), 10);
    }

    // === Sculpture detection tests ===

    #[test]
    fn test_ayatan_sah_sculpture_detection() {
        let item = make_sah_sculpture();
        assert!(
            item.is_sculpture(),
            "Ayatan Sah should be detected as a sculpture"
        );
        assert!(!item.is_mod(), "Ayatan Sah should not be a mod");
        assert!(
            !item.is_regular(),
            "Ayatan Sah should not be a regular item"
        );

        let sculpture = item
            .as_sculpture()
            .expect("Should return SculptureView for Ayatan Sah");
        assert_eq!(sculpture.base_endo(), 600);
        assert_eq!(sculpture.max_amber_stars(), 1);
        assert_eq!(sculpture.max_cyan_stars(), 2);
        assert_eq!(sculpture.total_slots(), 3);
    }

    #[test]
    fn test_ayatan_anasa_sculpture_detection() {
        let item = make_sculpture(); // Uses existing helper (Ayatan Anasa)
        assert!(
            item.is_sculpture(),
            "Ayatan Anasa should be detected as a sculpture"
        );
        assert!(!item.is_mod(), "Ayatan Anasa should not be a mod");
        assert!(
            !item.is_regular(),
            "Ayatan Anasa should not be a regular item"
        );

        let sculpture = item
            .as_sculpture()
            .expect("Should return SculptureView for Ayatan Anasa");
        assert_eq!(sculpture.base_endo(), 450);
        assert_eq!(sculpture.max_amber_stars(), 2);
        assert_eq!(sculpture.max_cyan_stars(), 4);
    }

    // === Regular item detection tests ===

    #[test]
    fn test_prime_part_is_regular() {
        let item = make_prime_part();
        assert!(
            item.is_regular(),
            "Nikana Prime Blade should be a regular item"
        );
        assert!(!item.is_mod(), "Nikana Prime Blade should not be a mod");
        assert!(
            !item.is_sculpture(),
            "Nikana Prime Blade should not be a sculpture"
        );
        assert!(item.as_mod().is_none());
        assert!(item.as_sculpture().is_none());
    }

    #[test]
    fn test_blueprint_is_regular() {
        let item = make_blueprint();
        assert!(
            item.is_regular(),
            "Ash Prime Blueprint should be a regular item"
        );
        assert!(!item.is_mod(), "Ash Prime Blueprint should not be a mod");
        assert!(
            !item.is_sculpture(),
            "Ash Prime Blueprint should not be a sculpture"
        );
        assert!(item.as_mod().is_none());
        assert!(item.as_sculpture().is_none());
    }

    // === Mutual exclusivity test ===

    #[test]
    fn test_item_type_mutual_exclusivity() {
        // Collect all test items
        let items: Vec<(&str, Item)> = vec![
            ("Archon Flow (mod)", make_archon_mod()),
            ("Requiem Lohk (mod)", make_requiem_mod()),
            ("Serration (mod)", make_standard_mod()),
            ("Ayatan Sah (sculpture)", make_sah_sculpture()),
            ("Ayatan Anasa (sculpture)", make_sculpture()),
            ("Nikana Prime Blade (regular)", make_prime_part()),
            ("Ash Prime Blueprint (regular)", make_blueprint()),
        ];

        for (name, item) in items {
            let type_flags = [item.is_mod(), item.is_sculpture(), item.is_regular()];
            let true_count = type_flags.iter().filter(|&&x| x).count();

            assert_eq!(
                true_count,
                1,
                "{} should be exactly one type, but got: is_mod={}, is_sculpture={}, is_regular={}",
                name,
                item.is_mod(),
                item.is_sculpture(),
                item.is_regular()
            );
        }
    }
}

/// Response for item set endpoint.
///
/// Contains the requested item's ID and all items in the set.
/// If the item is not part of a set, the items array contains only that item.
///
/// # Example
///
/// ```ignore
/// use wf_market::Client;
///
/// async fn example() -> wf_market::Result<()> {
///     let client = Client::builder().build()?;
///     let set = client.get_item_set("nikana_prime_set").await?;
///
///     println!("Set {} contains {} items:", set.id, set.items.len());
///     for item in &set.items {
///         println!("  - {}", item.name());
///     }
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ItemSet {
    /// ID of the requested item
    pub id: String,

    /// All items in the set (or just the single item if not part of a set)
    pub items: Vec<Item>,
}

impl ItemSet {
    /// Check if this is actually a set (more than one item).
    pub fn is_set(&self) -> bool {
        self.items.len() > 1
    }

    /// Get the number of items in the set.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the set is empty (should never happen with valid API data).
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the set root item (the main set item like "Nikana Prime Set").
    pub fn root(&self) -> Option<&Item> {
        self.items.iter().find(|item| item.is_set())
    }

    /// Get all parts excluding the set root.
    pub fn parts(&self) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(|item| !item.is_set())
    }
}
