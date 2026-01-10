//! Common types shared across the API.
//!
//! This module contains fundamental types like Platform, Language,
//! and other shared enumerations used throughout the library.

use serde::{Deserialize, Serialize};

/// Gaming platform for warframe.market.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// PC platform (default)
    #[default]
    Pc,
    /// PlayStation 4
    Ps4,
    /// Xbox
    Xbox,
    /// Nintendo Switch
    Switch,
    /// Mobile
    Mobile,
}

impl Platform {
    /// Get the API header value for this platform.
    pub fn as_header_value(&self) -> &'static str {
        match self {
            Platform::Pc => "pc",
            Platform::Ps4 => "ps4",
            Platform::Xbox => "xbox",
            Platform::Switch => "switch",
            Platform::Mobile => "mobile",
        }
    }

    /// Get all available platforms.
    pub fn all() -> &'static [Platform] {
        &[
            Platform::Pc,
            Platform::Ps4,
            Platform::Xbox,
            Platform::Switch,
            Platform::Mobile,
        ]
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Pc => write!(f, "PC"),
            Platform::Ps4 => write!(f, "PlayStation 4"),
            Platform::Xbox => write!(f, "Xbox"),
            Platform::Switch => write!(f, "Nintendo Switch"),
            Platform::Mobile => write!(f, "Mobile"),
        }
    }
}

/// Language for API responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// English (default)
    #[default]
    #[serde(rename = "en")]
    English,
    /// Korean
    #[serde(rename = "ko")]
    Korean,
    /// Russian
    #[serde(rename = "ru")]
    Russian,
    /// German
    #[serde(rename = "de")]
    German,
    /// French
    #[serde(rename = "fr")]
    French,
    /// Portuguese
    #[serde(rename = "pt")]
    Portuguese,
    /// Simplified Chinese
    #[serde(rename = "zh-hans")]
    ChineseSimplified,
    /// Traditional Chinese
    #[serde(rename = "zh-hant")]
    ChineseTraditional,
    /// Spanish
    #[serde(rename = "es")]
    Spanish,
    /// Italian
    #[serde(rename = "it")]
    Italian,
    /// Polish
    #[serde(rename = "pl")]
    Polish,
    /// Ukrainian
    #[serde(rename = "uk")]
    Ukrainian,
}

impl Language {
    /// Get the API header/parameter value for this language.
    pub fn as_code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Korean => "ko",
            Language::Russian => "ru",
            Language::German => "de",
            Language::French => "fr",
            Language::Portuguese => "pt",
            Language::ChineseSimplified => "zh-hans",
            Language::ChineseTraditional => "zh-hant",
            Language::Spanish => "es",
            Language::Italian => "it",
            Language::Polish => "pl",
            Language::Ukrainian => "uk",
        }
    }

    /// Get all available languages.
    pub fn all() -> &'static [Language] {
        &[
            Language::English,
            Language::Korean,
            Language::Russian,
            Language::German,
            Language::French,
            Language::Portuguese,
            Language::ChineseSimplified,
            Language::ChineseTraditional,
            Language::Spanish,
            Language::Italian,
            Language::Polish,
            Language::Ukrainian,
        ]
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::English => write!(f, "English"),
            Language::Korean => write!(f, "Korean"),
            Language::Russian => write!(f, "Russian"),
            Language::German => write!(f, "German"),
            Language::French => write!(f, "French"),
            Language::Portuguese => write!(f, "Portuguese"),
            Language::ChineseSimplified => write!(f, "Chinese (Simplified)"),
            Language::ChineseTraditional => write!(f, "Chinese (Traditional)"),
            Language::Spanish => write!(f, "Spanish"),
            Language::Italian => write!(f, "Italian"),
            Language::Polish => write!(f, "Polish"),
            Language::Ukrainian => write!(f, "Ukrainian"),
        }
    }
}

/// User online status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    /// User is offline
    #[default]
    Offline,
    /// User is online
    Online,
    /// User is in-game
    Ingame,
    /// User is invisible (appears offline to others)
    Invisible,
}

impl UserStatus {
    /// Check if the user is available for trading.
    pub fn is_available(&self) -> bool {
        matches!(self, UserStatus::Online | UserStatus::Ingame)
    }
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Offline => write!(f, "Offline"),
            UserStatus::Online => write!(f, "Online"),
            UserStatus::Ingame => write!(f, "In Game"),
            UserStatus::Invisible => write!(f, "Invisible"),
        }
    }
}

/// Order type (buy or sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    /// Buying order (WTB - Want To Buy)
    Buy,
    /// Selling order (WTS - Want To Sell)
    Sell,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Buy => write!(f, "Buy"),
            OrderType::Sell => write!(f, "Sell"),
        }
    }
}

/// Item rarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rarity::Common => write!(f, "Common"),
            Rarity::Uncommon => write!(f, "Uncommon"),
            Rarity::Rare => write!(f, "Rare"),
            Rarity::Legendary => write!(f, "Legendary"),
        }
    }
}

/// Riven weapon type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RivenType {
    Kitgun,
    Melee,
    Pistol,
    Rifle,
    Shotgun,
    Zaw,
}

impl std::fmt::Display for RivenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RivenType::Kitgun => write!(f, "Kitgun"),
            RivenType::Melee => write!(f, "Melee"),
            RivenType::Pistol => write!(f, "Pistol"),
            RivenType::Rifle => write!(f, "Rifle"),
            RivenType::Shotgun => write!(f, "Shotgun"),
            RivenType::Zaw => write!(f, "Zaw"),
        }
    }
}

/// User in-game activity type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityType {
    #[default]
    Unknown,
    Idle,
    OnMission,
    InDojo,
    InOrbiter,
    InRelay,
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityType::Unknown => write!(f, "Unknown"),
            ActivityType::Idle => write!(f, "Idle"),
            ActivityType::OnMission => write!(f, "On Mission"),
            ActivityType::InDojo => write!(f, "In Dojo"),
            ActivityType::InOrbiter => write!(f, "In Orbiter"),
            ActivityType::InRelay => write!(f, "In Relay"),
        }
    }
}

/// User in-game activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    /// Type of activity
    #[serde(rename = "type")]
    pub activity_type: ActivityType,

    /// Activity details (e.g., mission name, max 64 chars)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    /// When the activity started
    #[serde(rename = "startedAt", skip_serializing_if = "Option::is_none")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Activity {
    /// Create a new activity.
    pub fn new(activity_type: ActivityType) -> Self {
        Self {
            activity_type,
            details: None,
            started_at: None,
        }
    }

    /// Set activity details.
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Set activity start time.
    pub fn with_started_at(mut self, started_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.started_at = Some(started_at);
        self
    }
}

/// UI theme preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// Follow system preference
    #[default]
    System,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Light => write!(f, "Light"),
            Theme::Dark => write!(f, "Dark"),
            Theme::System => write!(f, "System"),
        }
    }
}

/// User role on the site.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Regular user
    #[default]
    User,
    /// Moderator
    Moderator,
    /// Administrator
    Admin,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::User => write!(f, "User"),
            UserRole::Moderator => write!(f, "Moderator"),
            UserRole::Admin => write!(f, "Admin"),
        }
    }
}

/// Subscription tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionTier {
    /// No subscription
    #[default]
    None,
    /// Bronze tier
    Bronze,
    /// Silver tier
    Silver,
    /// Gold tier
    Gold,
    /// Diamond tier
    Diamond,
}

impl std::fmt::Display for SubscriptionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubscriptionTier::None => write!(f, "None"),
            SubscriptionTier::Bronze => write!(f, "Bronze"),
            SubscriptionTier::Silver => write!(f, "Silver"),
            SubscriptionTier::Gold => write!(f, "Gold"),
            SubscriptionTier::Diamond => write!(f, "Diamond"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_serialization() {
        assert_eq!(serde_json::to_string(&Platform::Pc).unwrap(), "\"pc\"");
        assert_eq!(serde_json::to_string(&Platform::Ps4).unwrap(), "\"ps4\"");
    }

    #[test]
    fn test_language_codes() {
        assert_eq!(Language::English.as_code(), "en");
        assert_eq!(Language::ChineseSimplified.as_code(), "zh-hans");
    }

    #[test]
    fn test_user_status_availability() {
        assert!(UserStatus::Online.is_available());
        assert!(UserStatus::Ingame.is_available());
        assert!(!UserStatus::Offline.is_available());
        assert!(!UserStatus::Invisible.is_available());
    }
}
