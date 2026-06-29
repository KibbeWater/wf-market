use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApiVersion {
    V1,
    V2,
    // Custom API version with specified URLs for API and WebSocket.
    Custom(String, String), // (api_url, websocket_url)
}

impl ApiVersion {
    pub fn api_url(&self) -> &str {
        match self {
            ApiVersion::V1 => "https://api.warframe.market/v1",
            ApiVersion::V2 => "https://api.warframe.market/v2",
            ApiVersion::Custom(api_url, _) => api_url,
        }
    }
    pub fn websocket_url(&self) -> &str {
        match self {
            ApiVersion::V1 => "wss://warframe.market/socket?platform=pc",
            ApiVersion::V2 => "wss://ws.warframe.market/socket",
            ApiVersion::Custom(_, websocket_url) => websocket_url,
        }
    }
}
impl Default for ApiVersion {
    fn default() -> Self {
        ApiVersion::V2
    }
}
