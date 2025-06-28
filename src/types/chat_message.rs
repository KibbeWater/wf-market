use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub message: String,
    pub raw_message: String,
    pub id: String,
    pub send_date: String,
    pub chat_id: String,
    pub message_from: String,
}
