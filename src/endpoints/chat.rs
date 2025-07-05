use std::sync::{Arc, Mutex, Weak};

use reqwest::Method;
use serde::de::Error;
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
            Ok((data, _, err)) => {
                let user_value = data.payload.get("chats").ok_or_else(|| {
                    ApiError::ParsingError(err.clone(), serde_json::Error::missing_field("chats"))
                })?;
                let chats = serde_json::from_value::<Vec<Chat>>(user_value.clone())
                    .map_err(|e| ApiError::ParsingError(err.clone(), e))?;
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
            Ok((data, _, err)) => {
                let user_value = data.payload.get("messages").ok_or_else(|| {
                    ApiError::ParsingError(
                        err.clone(),
                        serde_json::Error::missing_field("messages"),
                    )
                })?;
                let messages = serde_json::from_value::<Vec<ChatMessage>>(user_value.clone())
                    .map_err(|e| ApiError::ParsingError(err.clone(), e))?;
                Ok(messages)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Leaves a chat with the given chat ID.
     * # Arguments
     * - `chat_id`: A string slice that holds the ID of the chat to leave.
     * # Returns
     * - `Ok(String)` containing the chat ID if successful.
     * - `Err(ApiError)` if the API call fails or if the response cannot be parsed.
     */
    pub async fn leave_chat(&self, chat_id: &str) -> Result<String, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(
                ApiVersion::V1,
                Method::DELETE,
                &format!("/im/chats/{}", chat_id),
                None,
                None,
            )
            .await
        {
            Ok((data, _, err)) => {
                let id = data.payload.get("chat_id").ok_or_else(|| {
                    ApiError::ParsingError(err.clone(), serde_json::Error::missing_field("chat_id"))
                })?;
                let id_str = id.as_str().ok_or_else(|| {
                    ApiError::ParsingError(
                        err.clone(),
                        serde_json::Error::custom("Chat ID is not a string"),
                    )
                })?;
                Ok(id_str.to_string())
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Fetches a list of users that are ignored in the chat.
     * # Returns
     * - `Ok(Vec<UserShort>)` containing the list of ignored users if successful.
     * - `Err(ApiError)` if the API call fails or if the response cannot be parsed.
     */
    pub async fn ignore_users(&self) -> Result<Vec<UserShort>, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV1<Vec<UserShort>>>(
                ApiVersion::V1,
                Method::GET,
                "/im/ignore",
                None,
                None,
            )
            .await
        {
            Ok((data, _, _)) => Ok(data.payload),
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Adds a user to the ignore list for a specific chat.
     * # Arguments
     * - `chat_id`: A string slice that holds the ID of the chat.
     * - `user_id`: A string slice that holds the ID of the user to ignore.
     * # Returns
     * - `Ok(UserShort)` containing the ignored user data if successful.
     * - `Err(ApiError)` if the API call fails or if the response cannot be parsed.
     */
    pub async fn ignore_user(&self, chat_id: &str, user_id: &str) -> Result<UserShort, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(
                ApiVersion::V1,
                Method::POST,
                "/im/ignore",
                Some(serde_json::json!({
                    "chat_id": chat_id,
                    "user_id": user_id,
                })),
                None,
            )
            .await
        {
            Ok((data, _, err)) => {
                let user_value = data.payload.get("user").ok_or_else(|| {
                    ApiError::ParsingError(err.clone(), serde_json::Error::missing_field("user"))
                })?;
                let user = serde_json::from_value::<UserShort>(user_value.clone())
                    .map_err(|e| ApiError::ParsingError(err.clone(), e))?;
                Ok(user)
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    /**
     * Removes a user from the ignore list.
     * # Arguments
     * - `user_id`: A string slice that holds the ID of the user to remove from the ignore list.
     * # Returns
     * - `Ok(String)` containing the user ID if successful.
     * - `Err(ApiError)` if the API call fails or if the response cannot be parsed.
     */
    pub async fn ignore_user_remove(&self, user_id: &str) -> Result<String, ApiError> {
        let client = self.client.upgrade().expect("Client should not be dropped");

        match client
            .as_ref()
            .call_api::<ApiResultV1<Value>>(
                ApiVersion::V1,
                Method::DELETE,
                &format!("/im/ignore/{}", user_id),
                None,
                None,
            )
            .await
        {
            Ok((data, _, err)) => {
                let id = data.payload.get("user_id").ok_or_else(|| {
                    ApiError::ParsingError(err.clone(), serde_json::Error::missing_field("user_id"))
                })?;
                let id_str = id.as_str().ok_or_else(|| {
                    ApiError::ParsingError(
                        err.clone(),
                        serde_json::Error::custom("User ID is not a string"),
                    )
                })?;
                Ok(id_str.to_string())
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
