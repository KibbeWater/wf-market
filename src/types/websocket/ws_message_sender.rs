use tokio::sync::mpsc;

use crate::{enums::*, errors::WsError, types::websocket::WsMessage};

// Message sender handle that can be cloned and passed to callbacks
#[derive(Clone)]
pub struct MessageSender {
    pub version: ApiVersion,
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
        let message = WsMessage::new(route, Some(payload), self.version.clone())
            .with_id(&uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, ref_id.as_bytes()).to_string())
            .with_ref_id(ref_id);
        self.send_message(message)
    }

    pub fn send_request(&self, route: &str, payload: serde_json::Value) -> Result<String, WsError> {
        let id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, route.as_bytes()).to_string();
        let message = WsMessage::new(route, Some(payload), self.version.clone()).with_id(&id);
        self.send_message(message)?;
        Ok(id)
    }
}
