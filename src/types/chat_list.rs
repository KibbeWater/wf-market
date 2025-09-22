use std::{fmt::Display, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::{
    enums::*,
    types::{websocket::WsMessage, *},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ChatList {
    #[serde(rename = "chats")]
    pub chats: Vec<Chat>,
}

impl ChatList {
    pub fn new(chats: Vec<Chat>) -> Self {
        ChatList { chats }
    }
    pub fn len(&self) -> usize {
        self.chats.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chats.is_empty()
    }

    pub fn get_by_id(&mut self, id: &str, set_unread: bool) -> Option<Chat> {
        if let Some(chat) = self.chats.iter_mut().find(|chat| chat.id == id) {
            if set_unread {
                chat.unread_count = 0;
            }
            Some(chat.clone())
        } else {
            None
        }
    }

    pub fn delete_by_id(&mut self, id: &str) -> Option<Chat> {
        if let Some(pos) = self.chats.iter().position(|chat| chat.id == id) {
            Some(self.chats.remove(pos))
        } else {
            None
        }
    }
    pub fn handle_chat_message(
        &mut self,
        message: &ChatMessage,
        active_chat_id: impl Into<String>,
    ) -> Option<Chat> {
        if let Some(chat) = self
            .chats
            .iter_mut()
            .find(|chat| chat.id == message.chat_id)
        {
            chat.messages[0] = message.clone();

            chat.last_update = message.send_date.clone();

            if active_chat_id.into() != message.chat_id {
                chat.unread_count += 1;
            }
            return Some(chat.clone());
        }
        None
    }
    pub fn total_unread_count(&self) -> u32 {
        self.chats.iter().map(|chat| chat.unread_count).sum()
    }
}
