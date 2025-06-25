use tokio::sync::mpsc;

use crate::{errors::WsError, types::websocket::WsMessage};

// Message sender handle that can be cloned and passed to callbacks
#[derive(Clone)]
pub struct MessageSender {
    pub tx: mpsc::UnboundedSender<WsMessage>,
}

impl MessageSender {
    pub fn send_message(&self, message: WsMessage) -> Result<(), WsError> {
        self.tx
            .send(message)
            .map_err(|e| WsError::SendError(e.to_string()))?;
        Ok(())
    }

    pub fn send_response(
        &self,
        route: &str,
        payload: serde_json::Value,
        ref_id: &str,
    ) -> Result<(), WsError> {
        let message = WsMessage {
            route: route.to_string(),
            payload: Some(payload),
            id: Some(uuid::Uuid::new_v4().to_string()),
            ref_id: Some(ref_id.to_string()),
        };
        self.send_message(message)
    }

    pub fn send_request(&self, route: &str, payload: serde_json::Value) -> Result<String, WsError> {
        let id = uuid::Uuid::new_v4().to_string();
        let message = WsMessage {
            route: route.to_string(),
            payload: Some(payload),
            id: Some(id.clone()),
            ref_id: None,
        };
        self.send_message(message)?;
        Ok(id)
    }
}
