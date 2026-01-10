//! WebSocket message types for internal use.

use serde::{Deserialize, Serialize};

/// WebSocket message structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WsMessage {
    /// Message route identifier
    pub route: String,

    /// Message payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,

    /// Unique message identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Reference to another message ID (for responses)
    #[serde(rename = "refId", skip_serializing_if = "Option::is_none")]
    pub ref_id: Option<String>,
}

impl WsMessage {
    /// Create a new message with a route and payload.
    pub fn new(route: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            route: route.into(),
            payload: Some(payload),
            id: Some(uuid::Uuid::new_v4().to_string()),
            ref_id: None,
        }
    }

    /// Create a message without payload.
    pub fn route_only(route: impl Into<String>) -> Self {
        Self {
            route: route.into(),
            payload: None,
            id: Some(uuid::Uuid::new_v4().to_string()),
            ref_id: None,
        }
    }

    /// Create an authentication message.
    pub fn sign_in(token: &str) -> Self {
        Self::new(
            "@user/signIn",
            serde_json::json!({
                "token": token,
            }),
        )
    }

    /// Create a sign out message.
    pub fn sign_out() -> Self {
        Self::route_only("@user/signOut")
    }

    /// Create a set status message.
    pub fn set_status(
        status: &str,
        duration: Option<u64>,
        activity: Option<serde_json::Value>,
    ) -> Self {
        let mut payload = serde_json::json!({
            "status": status,
        });

        if let Some(d) = duration {
            payload["duration"] = serde_json::json!(d);
        }

        if let Some(a) = activity {
            payload["activity"] = a;
        }

        Self::new("@user/setStatus", payload)
    }
}

/// Parsed route information.
#[derive(Debug, Clone)]
pub(crate) struct ParsedRoute {
    /// Protocol/module (e.g., "@user", "@wfm", "@reports")
    pub module: String,
    /// Action path (e.g., "signIn", "statusUpdate")
    pub action: String,
    /// Optional parameter (e.g., "ok", "error")
    pub parameter: Option<String>,
}

impl ParsedRoute {
    /// Parse a route string.
    pub fn parse(route: &str) -> Option<Self> {
        // Routes are in format: @module/action or @module/action:parameter
        if !route.starts_with('@') {
            return None;
        }

        let route = &route[1..]; // Remove @

        // Split by /
        let parts: Vec<&str> = route.splitn(2, '/').collect();
        if parts.len() < 2 {
            return None;
        }

        let module = format!("@{}", parts[0]);
        let action_part = parts[1];

        // Check for parameter
        if let Some(colon_pos) = action_part.find(':') {
            let action = action_part[..colon_pos].to_string();
            let parameter = Some(action_part[colon_pos + 1..].to_string());
            Some(Self {
                module,
                action,
                parameter,
            })
        } else {
            Some(Self {
                module,
                action: action_part.to_string(),
                parameter: None,
            })
        }
    }

    /// Get the full action path (without parameter).
    pub fn full_action(&self) -> String {
        format!("{}/{}", self.module, self.action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_route_simple() {
        let route = ParsedRoute::parse("@user/signIn").unwrap();
        assert_eq!(route.module, "@user");
        assert_eq!(route.action, "signIn");
        assert!(route.parameter.is_none());
    }

    #[test]
    fn test_parse_route_with_parameter() {
        let route = ParsedRoute::parse("@user/signIn:ok").unwrap();
        assert_eq!(route.module, "@user");
        assert_eq!(route.action, "signIn");
        assert_eq!(route.parameter, Some("ok".to_string()));
    }

    #[test]
    fn test_parse_reports_route() {
        let route = ParsedRoute::parse("@reports/onlineCount").unwrap();
        assert_eq!(route.module, "@reports");
        assert_eq!(route.action, "onlineCount");
    }

    #[test]
    fn test_message_creation() {
        let msg = WsMessage::sign_in("test-token");
        assert_eq!(msg.route, "@user/signIn");
        assert!(msg.payload.is_some());
        assert!(msg.id.is_some());
    }
}
