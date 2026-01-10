//! User models for warframe.market.
//!
//! This module provides types for representing users, including
//! minimal user info (attached to orders) and full user profiles.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::common::{Activity, Platform, UserStatus};

/// Minimal user information (attached to orders and listings).
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    /// Unique user identifier
    pub id: String,

    /// In-game name (IGN)
    #[serde(rename = "ingameName")]
    pub ingame_name: String,

    /// Path to user avatar (relative to static assets)
    #[serde(default)]
    pub avatar: Option<String>,

    /// User reputation score
    #[serde(default)]
    pub reputation: i32,

    /// Preferred locale
    #[serde(default)]
    pub locale: Option<String>,

    /// User's gaming platform
    pub platform: Platform,

    /// Whether cross-play is enabled
    #[serde(default)]
    pub crossplay: bool,

    /// Current online status
    #[serde(rename = "status", default)]
    pub status: UserStatus,

    /// Current in-game activity
    #[serde(default)]
    pub activity: Option<Activity>,

    /// Last seen timestamp
    #[serde(rename = "lastSeen", default)]
    pub last_seen: Option<DateTime<Utc>>,
}

impl User {
    /// Check if the user is currently available for trading.
    pub fn is_available(&self) -> bool {
        self.status.is_available()
    }

    /// Get the avatar URL (relative to static assets base).
    pub fn avatar_url(&self) -> Option<&str> {
        self.avatar.as_deref()
    }
}

/// Full user information (returned from /me endpoint).
#[derive(Debug, Clone, Deserialize)]
pub struct FullUser {
    /// Unique user identifier
    pub id: String,

    /// In-game name (IGN)
    #[serde(rename = "ingameName")]
    pub ingame_name: String,

    /// URL-friendly identifier
    #[serde(default)]
    pub slug: Option<String>,

    /// Path to user avatar
    #[serde(default)]
    pub avatar: Option<String>,

    /// Path to profile background
    #[serde(default)]
    pub background: Option<String>,

    /// Profile description (may contain HTML)
    #[serde(default)]
    pub about: Option<String>,

    /// User reputation score
    #[serde(default)]
    pub reputation: i32,

    /// In-game mastery rank
    #[serde(rename = "masteryLevel", default)]
    pub mastery_level: Option<u32>,

    /// User's gaming platform
    pub platform: Platform,

    /// Whether cross-play is enabled
    #[serde(default)]
    pub crossplay: bool,

    /// Preferred locale
    #[serde(default)]
    pub locale: Option<String>,

    /// Current online status
    #[serde(rename = "status", default)]
    pub status: UserStatus,

    /// Current in-game activity
    #[serde(default)]
    pub activity: Option<Activity>,

    /// Last seen timestamp
    #[serde(rename = "lastSeen", default)]
    pub last_seen: Option<DateTime<Utc>>,

    /// Whether the user is banned
    #[serde(default)]
    pub banned: Option<bool>,

    /// Ban expiration time
    #[serde(rename = "banUntil", default)]
    pub ban_until: Option<DateTime<Utc>>,

    /// Ban reason message
    #[serde(rename = "banMessage", default)]
    pub ban_message: Option<String>,

    /// Whether the user has an active warning
    #[serde(default)]
    pub warned: Option<bool>,

    /// Warning message
    #[serde(rename = "warnMessage", default)]
    pub warn_message: Option<String>,

    /// Email address (only visible to self)
    #[serde(default)]
    pub email: Option<String>,

    /// Whether email is verified
    #[serde(rename = "emailVerified", default)]
    pub email_verified: Option<bool>,

    /// Whether IGN is verified
    #[serde(rename = "ingameVerified", default)]
    pub ingame_verified: Option<bool>,
}

impl FullUser {
    /// Check if the user is currently available for trading.
    pub fn is_available(&self) -> bool {
        self.status.is_available()
    }

    /// Check if the user is banned.
    pub fn is_banned(&self) -> bool {
        self.banned.unwrap_or(false)
    }

    /// Check if the user has an active warning.
    pub fn has_warning(&self) -> bool {
        self.warned.unwrap_or(false)
    }

    /// Convert to minimal user info.
    pub fn to_user(&self) -> User {
        User {
            id: self.id.clone(),
            ingame_name: self.ingame_name.clone(),
            avatar: self.avatar.clone(),
            reputation: self.reputation,
            locale: self.locale.clone(),
            platform: self.platform,
            crossplay: self.crossplay,
            status: self.status,
            activity: self.activity.clone(),
            last_seen: self.last_seen,
        }
    }
}

/// Public user profile (from /user/{slug} endpoint).
#[derive(Debug, Clone, Deserialize)]
pub struct UserProfile {
    /// Unique user identifier
    pub id: String,

    /// In-game name (IGN)
    #[serde(rename = "ingameName")]
    pub ingame_name: String,

    /// URL-friendly identifier
    pub slug: String,

    /// Path to user avatar
    #[serde(default)]
    pub avatar: Option<String>,

    /// Path to profile background
    #[serde(default)]
    pub background: Option<String>,

    /// Profile description (may contain HTML)
    #[serde(default)]
    pub about: Option<String>,

    /// User reputation score
    #[serde(default)]
    pub reputation: i32,

    /// In-game mastery rank
    #[serde(rename = "masteryLevel", default)]
    pub mastery_level: Option<u32>,

    /// User's gaming platform
    pub platform: Platform,

    /// Whether cross-play is enabled
    #[serde(default)]
    pub crossplay: bool,

    /// Preferred locale
    #[serde(default)]
    pub locale: Option<String>,

    /// Current online status
    #[serde(rename = "status", default)]
    pub status: UserStatus,

    /// Current in-game activity
    #[serde(default)]
    pub activity: Option<Activity>,

    /// Last seen timestamp
    #[serde(rename = "lastSeen", default)]
    pub last_seen: Option<DateTime<Utc>>,

    /// Featured achievements on profile
    #[serde(rename = "achievementShowcase", default)]
    pub achievement_showcase: Vec<Achievement>,

    /// Whether the user is banned
    #[serde(default)]
    pub banned: Option<bool>,

    /// Ban expiration time
    #[serde(rename = "banUntil", default)]
    pub ban_until: Option<DateTime<Utc>>,

    /// Ban reason message
    #[serde(rename = "banMessage", default)]
    pub ban_message: Option<String>,

    /// Whether the user has an active warning
    #[serde(default)]
    pub warned: Option<bool>,

    /// Warning message
    #[serde(rename = "warnMessage", default)]
    pub warn_message: Option<String>,
}

impl UserProfile {
    /// Check if the user is currently available for trading.
    pub fn is_available(&self) -> bool {
        self.status.is_available()
    }

    /// Check if the user is banned.
    pub fn is_banned(&self) -> bool {
        self.banned.unwrap_or(false)
    }
}

/// User achievement display.
#[derive(Debug, Clone, Deserialize)]
pub struct Achievement {
    /// Achievement ID
    pub id: String,

    /// Path to achievement icon
    pub icon: String,

    /// Path to achievement thumbnail
    #[serde(default)]
    pub thumb: Option<String>,

    /// Achievement type
    #[serde(rename = "type")]
    pub achievement_type: AchievementType,

    /// Localized achievement content
    #[serde(default)]
    pub i18n: Option<AchievementTranslation>,
}

/// Achievement type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AchievementType {
    Patreon,
    Github,
    Custom,
}

/// Localized achievement content.
#[derive(Debug, Clone, Deserialize)]
pub struct AchievementTranslation {
    /// Achievement name
    pub name: String,

    /// Achievement description
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_is_available() {
        let user = User {
            id: "test".to_string(),
            ingame_name: "TestUser".to_string(),
            avatar: None,
            reputation: 0,
            locale: None,
            platform: Platform::Pc,
            crossplay: true,
            status: UserStatus::Online,
            activity: None,
            last_seen: None,
        };

        assert!(user.is_available());
    }

    #[test]
    fn test_user_offline_not_available() {
        let user = User {
            id: "test".to_string(),
            ingame_name: "TestUser".to_string(),
            avatar: None,
            reputation: 0,
            locale: None,
            platform: Platform::Pc,
            crossplay: true,
            status: UserStatus::Offline,
            activity: None,
            last_seen: None,
        };

        assert!(!user.is_available());
    }
}
