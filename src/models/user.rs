//! User models for warframe.market.
//!
//! This module provides types for representing users, including
//! minimal user info (attached to orders) and full user profiles.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use serde::Serialize;

use super::common::{Activity, Language, Platform, SubscriptionTier, Theme, UserRole, UserStatus};

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

/// Private user profile (returned from /me endpoint).
///
/// Contains all public profile data plus private settings and account info.
#[derive(Debug, Clone, Deserialize)]
pub struct UserPrivate {
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

    /// Featured achievements on profile
    #[serde(rename = "achievementShowcase", default)]
    pub achievement_showcase: Vec<Achievement>,

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

    /// Email address
    #[serde(default)]
    pub email: Option<String>,

    /// User role on the site
    #[serde(default)]
    pub role: UserRole,

    /// Subscription tier
    #[serde(default)]
    pub tier: Option<SubscriptionTier>,

    /// UI theme preference
    #[serde(default)]
    pub theme: Option<Theme>,

    /// Whether to sync locale across devices
    #[serde(rename = "syncLocale", default)]
    pub sync_locale: Option<bool>,

    /// Whether to sync theme across devices
    #[serde(rename = "syncTheme", default)]
    pub sync_theme: Option<bool>,

    /// Whether account is verified
    #[serde(default)]
    pub verified: Option<bool>,
}

impl UserPrivate {
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

    /// Check if the user is verified.
    pub fn is_verified(&self) -> bool {
        self.verified.unwrap_or(false)
    }

    /// Check if the user is a moderator or admin.
    pub fn is_staff(&self) -> bool {
        matches!(self.role, UserRole::Moderator | UserRole::Admin)
    }

    /// Convert to public user profile.
    pub fn to_profile(&self) -> UserProfile {
        UserProfile {
            id: self.id.clone(),
            ingame_name: self.ingame_name.clone(),
            slug: self.slug.clone(),
            avatar: self.avatar.clone(),
            background: self.background.clone(),
            about: self.about.clone(),
            reputation: self.reputation,
            mastery_level: self.mastery_level,
            platform: self.platform,
            crossplay: self.crossplay,
            locale: self.locale.clone(),
            status: self.status,
            activity: self.activity.clone(),
            last_seen: self.last_seen,
            achievement_showcase: self.achievement_showcase.clone(),
            banned: self.banned,
            ban_until: self.ban_until,
            ban_message: self.ban_message.clone(),
            warned: self.warned,
            warn_message: self.warn_message.clone(),
        }
    }
}

/// Request to update the current user's profile.
///
/// Only include the fields you want to change.
///
/// # Example
///
/// ```
/// use wf_market::{UpdateProfile, Platform, Theme};
///
/// // Update platform and enable crossplay
/// let update = UpdateProfile::new()
///     .platform(Platform::Pc)
///     .crossplay(true);
///
/// // Update theme preference
/// let update = UpdateProfile::new().theme(Theme::Dark);
///
/// // Update profile description
/// let update = UpdateProfile::new().about("Trading Prime parts!");
/// ```
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfile {
    /// New profile description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,

    /// New platform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<Platform>,

    /// Enable/disable crossplay
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossplay: Option<bool>,

    /// New locale
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<Language>,

    /// New theme preference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,

    /// Sync locale across devices
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_locale: Option<bool>,

    /// Sync theme across devices
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_theme: Option<bool>,
}

impl UpdateProfile {
    /// Create a new empty profile update request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the profile description.
    pub fn about(mut self, about: impl Into<String>) -> Self {
        self.about = Some(about.into());
        self
    }

    /// Set the platform.
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = Some(platform);
        self
    }

    /// Set crossplay enabled/disabled.
    pub fn crossplay(mut self, crossplay: bool) -> Self {
        self.crossplay = Some(crossplay);
        self
    }

    /// Set the locale.
    pub fn locale(mut self, locale: Language) -> Self {
        self.locale = Some(locale);
        self
    }

    /// Set the theme preference.
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Set whether to sync locale across devices.
    pub fn sync_locale(mut self, sync: bool) -> Self {
        self.sync_locale = Some(sync);
        self
    }

    /// Set whether to sync theme across devices.
    pub fn sync_theme(mut self, sync: bool) -> Self {
        self.sync_theme = Some(sync);
        self
    }

    /// Check if any fields are set.
    pub fn is_empty(&self) -> bool {
        self.about.is_none()
            && self.platform.is_none()
            && self.crossplay.is_none()
            && self.locale.is_none()
            && self.theme.is_none()
            && self.sync_locale.is_none()
            && self.sync_theme.is_none()
    }
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
