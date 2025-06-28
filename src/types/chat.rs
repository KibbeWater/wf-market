use serde::{Deserialize, Serialize};

use crate::types::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Chat {
    pub chat_with: Vec<UserShort>,
    pub unread_count: u32,
    pub last_update: String,
    pub messages: Vec<ChatMessage>,
    pub id: String,
    pub chat_name: String,
}
