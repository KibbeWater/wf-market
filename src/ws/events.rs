//! WebSocket event types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::OrderListing;

/// Events received from the WebSocket connection.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WsEvent {
    // === Connection Lifecycle ===
    /// Successfully connected to WebSocket server
    Connected,

    /// Disconnected from server
    Disconnected { reason: String },

    /// Successfully authenticated on WebSocket
    Authenticated,

    /// Authentication failed
    AuthenticationFailed { error: String },

    // === Reports ===
    /// Online user count (sent every ~30 seconds)
    OnlineCount {
        /// Total WebSocket connections
        connections: u64,
        /// Authenticated users
        authorized: u64,
    },

    // === User Events ===
    /// User status changed (from any connected client)
    StatusUpdate {
        status: UserStatus,
        status_until: Option<DateTime<Utc>>,
        activity: Option<Activity>,
    },

    /// Session was revoked (logout, password change, token expired)
    SessionRevoked,

    /// Account verification status changed
    VerificationUpdate { ingame_name: String, verified: bool },

    /// User was banned
    Banned {
        until: DateTime<Utc>,
        message: String,
    },

    /// User received a warning
    Warned { message: String },

    /// Ban was lifted
    BanLifted,

    /// Warning was lifted
    WarningLifted,

    // === Order Events ===
    /// New order posted (when subscribed)
    OrderCreated { order: OrderListing },

    /// Order was updated
    OrderUpdated { order: OrderListing },

    /// Order was removed
    OrderRemoved { order_id: String },

    // === Fallback ===
    /// Unknown/unhandled event (for forward compatibility)
    Unknown {
        route: String,
        payload: serde_json::Value,
    },
}

/// User online status (for WebSocket events).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    #[default]
    Offline,
    Online,
    Ingame,
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
    pub started_at: Option<DateTime<Utc>>,
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
    pub fn with_started_at(mut self, started_at: DateTime<Utc>) -> Self {
        self.started_at = Some(started_at);
        self
    }
}
