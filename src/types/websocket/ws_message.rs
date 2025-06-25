use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WsMessage {
    pub route: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "refId", skip_serializing_if = "Option::is_none")]
    pub ref_id: Option<String>,
}
impl WsMessage {
    pub fn new(route: &str, payload: Option<serde_json::Value>) -> Self {
        WsMessage {
            route: route.to_string(),
            payload,
            id: Some(uuid::Uuid::new_v4().to_string()),
            ref_id: None,
        }
    }
    pub fn connect() -> Self {
        WsMessage {
            route: "@internal|internal/connected".to_string(),
            payload: Some(json!({"status": "connected"})),
            id: Some("INTERNAL".to_string()),
            ref_id: None,
        }
    }
    pub fn disconnect(error: String) -> Self {
        WsMessage {
            route: "@internal|internal/disconnected".to_string(),
            payload: Some(json!({"reason": error})),
            id: Some("INTERNAL".to_string()),
            ref_id: None,
        }
    }
}
