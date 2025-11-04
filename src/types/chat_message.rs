use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub id: String,
    #[serde(default)]
    pub chat_id: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub raw_message: String,
    #[serde(default)]
    pub send_date: String,
    #[serde(default)]
    pub message_from: String,
}
