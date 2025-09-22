use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub id: String,
    pub chat_id: String,
    pub message: String,
    pub raw_message: String,
    pub send_date: String,
    pub message_from: String,
}
