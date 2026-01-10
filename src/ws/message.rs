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
            id: Some(generate_short_id()),
            ref_id: None,
        }
    }

    /// Create a message without payload.
    pub fn route_only(route: impl Into<String>) -> Self {
        Self {
            route: route.into(),
            payload: None,
            id: Some(generate_short_id()),
            ref_id: None,
        }
    }

    /// Create an authentication message.
    pub fn sign_in(token: &str) -> Self {
        Self::new(
            "@wfm|cmd/auth/signIn",
            serde_json::json!({
                "token": token,
            }),
        )
    }

    /// Create a sign out message.
    pub fn sign_out() -> Self {
        Self::route_only("@wfm|cmd/auth/signOut")
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

        Self::new("@wfm|cmd/status/set", payload)
    }
}

/// Generate a short alphanumeric ID similar to what the API uses.
fn generate_short_id() -> String {
    use uuid::Uuid;
    // Take first 11 chars of base64-encoded UUID (similar to API format)
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    // Simple base64-like encoding using alphanumeric chars
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    bytes
        .iter()
        .take(11)
        .map(|b| chars[(*b as usize) % chars.len()])
        .collect()
}

/// Parsed route information.
///
/// Routes follow the format: `@wfm|type/path` or `@wfm|type/path:parameter`
/// Examples:
/// - `@wfm|cmd/auth/signIn` -> module="@wfm", msg_type="cmd", path="auth/signIn"
/// - `@wfm|cmd/auth/signIn:ok` -> same as above with parameter="ok"
/// - `@wfm|event/reports/online` -> msg_type="event", path="reports/online"
/// - `@wfm|event/subscriptions/newOrder` -> msg_type="event", path="subscriptions/newOrder"
#[derive(Debug, Clone)]
pub(crate) struct ParsedRoute {
    /// Protocol/module (e.g., "@wfm")
    pub module: String,
    /// Message type ("cmd" or "event")
    pub msg_type: String,
    /// Full path after type (e.g., "auth/signIn", "subscriptions/newOrder")
    pub path: String,
    /// Optional parameter (e.g., "ok", "error")
    pub parameter: Option<String>,
}

impl ParsedRoute {
    /// Parse a route string.
    ///
    /// Handles format: `@wfm|type/path` or `@wfm|type/path:parameter`
    pub fn parse(route: &str) -> Option<Self> {
        // Routes are in format: @wfm|type/path or @wfm|type/path:parameter
        if !route.starts_with('@') {
            return None;
        }

        // Split by | to get module and rest
        let (module, rest) = route.split_once('|')?;

        // Split rest by first / to get type and path
        let (msg_type, path_with_param) = rest.split_once('/')?;

        // Check for parameter (after :)
        let (path, parameter) = if let Some(colon_pos) = path_with_param.rfind(':') {
            // Only treat as parameter if it's at a reasonable position
            // (not part of a nested path like "cmd/subscribe/newOrders:ok")
            let potential_param = &path_with_param[colon_pos + 1..];
            // Parameters are typically short like "ok", "error"
            if potential_param.len() <= 10 && !potential_param.contains('/') {
                (
                    path_with_param[..colon_pos].to_string(),
                    Some(potential_param.to_string()),
                )
            } else {
                (path_with_param.to_string(), None)
            }
        } else {
            (path_with_param.to_string(), None)
        };

        Some(Self {
            module: module.to_string(),
            msg_type: msg_type.to_string(),
            path,
            parameter,
        })
    }

    /// Check if this is a command response (has :ok or :error parameter).
    #[allow(dead_code)]
    pub fn is_response(&self) -> bool {
        self.parameter.is_some()
    }

    /// Check if this is a successful response.
    #[allow(dead_code)]
    pub fn is_ok(&self) -> bool {
        self.parameter.as_deref() == Some("ok")
    }

    /// Check if this is an error response.
    #[allow(dead_code)]
    pub fn is_error(&self) -> bool {
        self.parameter.as_deref() == Some("error")
    }

    /// Get the full route without parameter.
    #[allow(dead_code)]
    pub fn full_route(&self) -> String {
        format!("{}|{}/{}", self.module, self.msg_type, self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_route_cmd() {
        let route = ParsedRoute::parse("@wfm|cmd/auth/signIn").unwrap();
        assert_eq!(route.module, "@wfm");
        assert_eq!(route.msg_type, "cmd");
        assert_eq!(route.path, "auth/signIn");
        assert!(route.parameter.is_none());
    }

    #[test]
    fn test_parse_route_cmd_with_ok() {
        let route = ParsedRoute::parse("@wfm|cmd/auth/signIn:ok").unwrap();
        assert_eq!(route.module, "@wfm");
        assert_eq!(route.msg_type, "cmd");
        assert_eq!(route.path, "auth/signIn");
        assert_eq!(route.parameter, Some("ok".to_string()));
        assert!(route.is_ok());
    }

    #[test]
    fn test_parse_route_event() {
        let route = ParsedRoute::parse("@wfm|event/reports/online").unwrap();
        assert_eq!(route.module, "@wfm");
        assert_eq!(route.msg_type, "event");
        assert_eq!(route.path, "reports/online");
        assert!(route.parameter.is_none());
    }

    #[test]
    fn test_parse_route_subscription_event() {
        let route = ParsedRoute::parse("@wfm|event/subscriptions/newOrder").unwrap();
        assert_eq!(route.module, "@wfm");
        assert_eq!(route.msg_type, "event");
        assert_eq!(route.path, "subscriptions/newOrder");
    }

    #[test]
    fn test_parse_route_subscribe_ok() {
        let route = ParsedRoute::parse("@wfm|cmd/subscribe/newOrders:ok").unwrap();
        assert_eq!(route.module, "@wfm");
        assert_eq!(route.msg_type, "cmd");
        assert_eq!(route.path, "subscribe/newOrders");
        assert_eq!(route.parameter, Some("ok".to_string()));
    }

    #[test]
    fn test_parse_route_status_event() {
        let route = ParsedRoute::parse("@wfm|event/status/set").unwrap();
        assert_eq!(route.module, "@wfm");
        assert_eq!(route.msg_type, "event");
        assert_eq!(route.path, "status/set");
    }

    #[test]
    fn test_message_sign_in() {
        let msg = WsMessage::sign_in("test-token");
        assert_eq!(msg.route, "@wfm|cmd/auth/signIn");
        assert!(msg.payload.is_some());
        assert!(msg.id.is_some());
        let payload = msg.payload.unwrap();
        assert_eq!(payload["token"], "test-token");
    }

    #[test]
    fn test_message_sign_out() {
        let msg = WsMessage::sign_out();
        assert_eq!(msg.route, "@wfm|cmd/auth/signOut");
    }

    #[test]
    fn test_message_set_status() {
        let msg = WsMessage::set_status("online", Some(3600), None);
        assert_eq!(msg.route, "@wfm|cmd/status/set");
        let payload = msg.payload.unwrap();
        assert_eq!(payload["status"], "online");
        assert_eq!(payload["duration"], 3600);
    }

    #[test]
    fn test_short_id_generation() {
        let id1 = generate_short_id();
        let id2 = generate_short_id();
        assert_eq!(id1.len(), 11);
        assert_eq!(id2.len(), 11);
        assert_ne!(id1, id2); // Should be unique
    }
}
