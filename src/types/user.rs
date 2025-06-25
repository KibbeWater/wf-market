use serde::Deserialize;

use crate::types::*;

#[derive(Deserialize, Clone, Debug)]
pub struct User {
    /// Unique identifier of the user.
    pub id: String,
    /// User's in-game name.
    #[serde(rename = "ingameName")]
    pub ingame_name: String,
    /// Optional link to the user's avatar image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    /// Optional link to the user's profile background image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// Optional HTML-formatted text about the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /// User's reputation score.
    pub reputation: i16,
    /// Optional in-game mastery level.
    #[serde(rename = "masteryLevel", skip_serializing_if = "Option::is_none")]
    pub mastery_level: Option<i8>,

    /// Platform the user plays on.
    pub platform: String,
    /// Indicates if the user is open to cross-platform trading.
    pub crossplay: bool,
    /// User's locale or preferred language.
    pub locale: String,

    /// List of achievements the user chose to showcase.
    #[serde(
        rename = "achievementShowcase",
        skip_serializing_if = "Option::is_none"
    )]
    pub achievement_showcase: Option<Vec<Achievement>>,

    /// Current status of the user.
    pub status: String,
    /// Current activity the user is engaged in.
    pub activity: Activity,
    /// Timestamp of the user's last online presence.
    #[serde(rename = "lastSeen")]
    pub last_seen: String,

    /// Indicates whether the user is currently banned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banned: Option<bool>,
    /// End date of the current ban, if applicable.
    #[serde(rename = "banUntil", skip_serializing_if = "Option::is_none")]
    pub ban_until: Option<String>,

    // Fields below are accessible only to moderators and admins.
    /// Indicates whether the user has been warned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warned: Option<bool>,
    /// Warning message, if any.
    #[serde(rename = "warnMessage", skip_serializing_if = "Option::is_none")]
    pub warn_message: Option<String>,
    /// Ban message or reason for the ban, if any.
    #[serde(rename = "banMessage", skip_serializing_if = "Option::is_none")]
    pub ban_message: Option<String>,
}
