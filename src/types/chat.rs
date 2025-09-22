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

impl Chat {
    pub fn find_user(&self, user_id: impl Into<String>) -> Option<UserShort> {
        let user_id = user_id.into();
        self.chat_with
            .iter()
            .find(|user| user.id == user_id)
            .cloned()
    }
}
