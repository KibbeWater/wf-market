use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;
use serde_json::Value;

use crate::{IsAuthenticated, client::Client, enums::*, errors::*, types::*};

#[derive(Debug)]
pub struct ChatRoute<State> {
    chats_cache: Mutex<Option<Vec<Chat>>>,
    client: Weak<Client<State>>,
}

impl<State: Clone + 'static> ChatRoute<State> {
    /**
     * Creates a new `ChatRoute` with an empty User list.
     * The `client` parameter is an `Arc<Client<State>>` that allows the route
     */
    pub fn new(client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            chats_cache: Mutex::new(None),
            client: Arc::downgrade(&client),
        })
    }

    /**
     * Creates a new `ChatRoute` from an existing one, sharing the client.
     * This is useful for cloning routes when the client state changes.
     */
    pub fn from_existing<T>(old: &ChatRoute<T>, client: Arc<Client<State>>) -> Arc<Self> {
        Arc::new(Self {
            chats_cache: Mutex::new(old.chats_cache.lock().unwrap().clone()),
            client: Arc::downgrade(&client),
        })
    }
}

impl<State> ChatRoute<State>
where
    State: IsAuthenticated + Clone + 'static,
{
    /**
     * Fetches the a list of chats from the API.
     * # Returns
     * - `Ok(Vec<Chat>)` containing the list of chats if successful.
     * - `Err(ApiError)` if the API call fails or if the response cannot be parsed.
     */
    pub async fn get_chats(&self) -> Result<Vec<Chat>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(ApiVersion::V1, Method::GET, "/im/chats", None, None)
            .await
        {
            Ok((data, _headers)) => {
                let user_value = data.payload.get("chats").ok_or_else(|| {
                    ApiError::ParsingError("Missing 'chats' field in response".to_string())
                })?;
                let chats =
                    serde_json::from_value::<Vec<Chat>>(user_value.clone()).map_err(|e| {
                        ApiError::ParsingError(format!("Failed to parse chats data: {}", e))
                    })?;
                let mut ca_chats = self.chats_cache.lock().unwrap();
                *ca_chats = Some(chats.clone());
                Ok(chats)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Fetches the a list of chat messages for a specific chat.
     * * # Arguments
     * - `chat_id`: A string slice that holds the ID of the chat for which
     * # Returns
     * - `Ok(Vec<ChatMessage>)` containing the list of chats if successful.
     * - `Err(ApiError)` if the cache is empty.
     */
    pub async fn get_chat_messages(&self, chat_id: &str) -> Result<Vec<ChatMessage>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(
                ApiVersion::V1,
                Method::GET,
                &format!("/im/chats/{}", chat_id),
                None,
                None,
            )
            .await
        {
            Ok((data, _headers)) => {
                let user_value = data.payload.get("messages").ok_or_else(|| {
                    ApiError::ParsingError("Missing 'messages' field in response".to_string())
                })?;
                let messages = serde_json::from_value::<Vec<ChatMessage>>(user_value.clone())
                    .map_err(|e| {
                        ApiError::ParsingError(format!(
                            "Failed to parse chats messages data: {}",
                            e
                        ))
                    })?;
                Ok(messages)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
