//! Riven weapon models for warframe.market.
//!
//! This module provides types for representing riven-compatible weapons.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::common::RivenType;

/// A weapon that can have riven mods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Riven {
    /// Unique identifier
    pub id: String,

    /// URL-friendly identifier
    pub slug: String,

    /// Internal game reference path
    #[serde(rename = "gameRef")]
    pub game_ref: String,

    /// Weapon group for categorization
    pub group: String,

    /// Riven weapon type
    #[serde(rename = "rivenType")]
    pub riven_type: RivenType,

    /// Riven disposition (affects stat ranges, 0.5-1.55)
    pub disposition: f32,

    /// Required mastery rank to trade
    #[serde(rename = "reqMasteryRank", default)]
    pub mastery_rank: Option<u32>,

    /// Localized content
    #[serde(default)]
    pub i18n: HashMap<String, RivenTranslation>,
}

/// Localized riven weapon content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RivenTranslation {
    /// Localized weapon name
    pub name: String,

    /// Path to weapon icon
    pub icon: String,

    /// Path to weapon thumbnail
    #[serde(default)]
    pub thumb: Option<String>,

    /// Link to the Warframe wiki page
    #[serde(rename = "wikiLink", default)]
    pub wiki_link: Option<String>,
}

impl Riven {
    /// Get the English name of the weapon.
    pub fn name(&self) -> &str {
        self.i18n.get("en").map(|t| t.name.as_str()).unwrap_or("")
    }

    /// Get the localized name for a specific language.
    pub fn name_localized(&self, lang: &str) -> &str {
        self.i18n
            .get(lang)
            .or_else(|| self.i18n.get("en"))
            .map(|t| t.name.as_str())
            .unwrap_or("")
    }

    /// Get the icon URL (relative to static assets base).
    pub fn icon(&self) -> Option<&str> {
        self.i18n.get("en").map(|t| t.icon.as_str())
    }

    /// Get the wiki link if available.
    pub fn wiki_link(&self) -> Option<&str> {
        self.i18n.get("en").and_then(|t| t.wiki_link.as_deref())
    }

    /// Get the disposition as a descriptive tier.
    ///
    /// Returns a value from 1 (weak) to 5 (strong) based on disposition.
    pub fn disposition_tier(&self) -> u8 {
        match self.disposition {
            d if d < 0.7 => 1,
            d if d < 0.9 => 2,
            d if d < 1.1 => 3,
            d if d < 1.3 => 4,
            _ => 5,
        }
    }

    /// Check if this is a melee weapon.
    pub fn is_melee(&self) -> bool {
        matches!(self.riven_type, RivenType::Melee | RivenType::Zaw)
    }

    /// Check if this is a ranged weapon.
    pub fn is_ranged(&self) -> bool {
        matches!(
            self.riven_type,
            RivenType::Rifle | RivenType::Pistol | RivenType::Shotgun | RivenType::Kitgun
        )
    }
}

/// A riven attribute/stat that can appear on riven mods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RivenAttribute {
    /// Unique identifier
    pub id: String,

    /// URL-friendly identifier
    pub slug: String,

    /// Internal game reference
    #[serde(rename = "gameRef")]
    pub game_ref: String,

    /// Attribute group for categorization
    pub group: String,

    /// Prefix used in riven names when positive
    pub prefix: String,

    /// Suffix used in riven names when negative
    pub suffix: String,

    /// Weapon types this attribute can appear on (if restricted)
    #[serde(rename = "exclusiveTo", default)]
    pub exclusive_to: Option<Vec<RivenType>>,

    /// Whether a positive value is actually negative (e.g., recoil)
    #[serde(rename = "positiveIsNegative", default)]
    pub positive_is_negative: Option<bool>,

    /// Whether this attribute can only be positive
    #[serde(rename = "positiveOnly", default)]
    pub positive_only: Option<bool>,

    /// Whether this attribute can only be negative
    #[serde(rename = "negativeOnly", default)]
    pub negative_only: Option<bool>,

    /// Unit of measurement (e.g., percent, flat)
    #[serde(default)]
    pub unit: Option<String>,

    /// Localized content
    #[serde(default)]
    pub i18n: HashMap<String, RivenAttributeTranslation>,
}

/// Localized riven attribute content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RivenAttributeTranslation {
    /// Localized attribute name
    pub name: String,

    /// Path to attribute icon
    pub icon: String,

    /// Path to attribute thumbnail
    #[serde(default)]
    pub thumb: Option<String>,
}

impl RivenAttribute {
    /// Get the English name of the attribute.
    pub fn name(&self) -> &str {
        self.i18n.get("en").map(|t| t.name.as_str()).unwrap_or("")
    }

    /// Get the localized name for a specific language.
    pub fn name_localized(&self, lang: &str) -> &str {
        self.i18n
            .get(lang)
            .or_else(|| self.i18n.get("en"))
            .map(|t| t.name.as_str())
            .unwrap_or("")
    }

    /// Check if this attribute is a "bad" positive (like increased recoil).
    pub fn is_inverted(&self) -> bool {
        self.positive_is_negative.unwrap_or(false)
    }

    /// Check if this attribute is restricted to specific weapon types.
    pub fn is_restricted(&self) -> bool {
        self.exclusive_to.as_ref().is_some_and(|v| !v.is_empty())
    }

    /// Check if this attribute can appear on a specific weapon type.
    pub fn applies_to(&self, riven_type: RivenType) -> bool {
        match &self.exclusive_to {
            None => true,
            Some(types) => types.is_empty() || types.contains(&riven_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_riven() -> Riven {
        Riven {
            id: "test-id".to_string(),
            slug: "braton".to_string(),
            game_ref: "/Lotus/Weapons/Tenno/Rifle/Braton".to_string(),
            group: "rifle".to_string(),
            riven_type: RivenType::Rifle,
            disposition: 1.45,
            mastery_rank: Some(0),
            i18n: HashMap::from([(
                "en".to_string(),
                RivenTranslation {
                    name: "Braton".to_string(),
                    icon: "/icons/braton.png".to_string(),
                    thumb: None,
                    wiki_link: Some("https://warframe.fandom.com/wiki/Braton".to_string()),
                },
            )]),
        }
    }

    #[test]
    fn test_riven_name() {
        let riven = make_riven();
        assert_eq!(riven.name(), "Braton");
    }

    #[test]
    fn test_disposition_tier() {
        let mut riven = make_riven();

        riven.disposition = 0.5;
        assert_eq!(riven.disposition_tier(), 1);

        riven.disposition = 0.8;
        assert_eq!(riven.disposition_tier(), 2);

        riven.disposition = 1.0;
        assert_eq!(riven.disposition_tier(), 3);

        riven.disposition = 1.2;
        assert_eq!(riven.disposition_tier(), 4);

        riven.disposition = 1.5;
        assert_eq!(riven.disposition_tier(), 5);
    }

    #[test]
    fn test_weapon_type() {
        let riven = make_riven();
        assert!(riven.is_ranged());
        assert!(!riven.is_melee());
    }
}
