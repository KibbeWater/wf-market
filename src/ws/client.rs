//! WebSocket client implementation.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

use crate::error::{Error, Result, WsError};
use crate::models::OrderListing;

use super::builder::WebSocketBuilder;
use super::events::{Activity, UserStatus, WsEvent};
use super::message::{ParsedRoute, WsMessage};
use super::subscription::Subscription;
use super::{WS_PROTOCOL, WS_URL};

/// Active WebSocket connection.
///
/// Provides methods for subscribing to events and sending commands.
pub struct WebSocket {
    sender: mpsc::UnboundedSender<WsMessage>,
    subscriptions: Arc<RwLock<HashSet<Subscription>>>,
}

impl WebSocket {
    /// Connect to the WebSocket server.
    pub(crate) async fn connect(builder: WebSocketBuilder<'_>) -> Result<Self> {
        let token = builder.client.token().to_string();
        let event_handler = builder.event_handler;
        let initial_subscriptions = builder.subscriptions;
        let auto_reconnect = builder.auto_reconnect;
        let reconnect_delay = builder.reconnect_delay;
        let user_agent = builder.user_agent;

        let (msg_tx, msg_rx) = mpsc::unbounded_channel::<WsMessage>();
        let subscriptions = Arc::new(RwLock::new(HashSet::new()));
        let subscriptions_clone = subscriptions.clone();

        // Spawn the connection task
        tokio::spawn(async move {
            // Wrap in Option to allow taking/replacing across reconnections
            let mut msg_rx = Some(msg_rx);

            loop {
                // Build request with protocol
                let mut request = WS_URL.into_client_request().expect("Invalid WebSocket URL");
                let headers = request.headers_mut();
                headers.insert("Sec-WebSocket-Protocol", WS_PROTOCOL.parse().unwrap());
                headers.insert("User-Agent", user_agent.parse().unwrap());

                match connect_async(request).await {
                    Ok((ws_stream, _)) => {
                        let (mut write, mut read) = ws_stream.split();

                        // Emit connected event
                        if let Some(ref handler) = event_handler {
                            handler(WsEvent::Connected).await;
                        }

                        // Send authentication
                        let auth_msg = WsMessage::sign_in(&token);
                        let auth_json = serde_json::to_string(&auth_msg).unwrap();
                        if write.send(Message::Text(auth_json.into())).await.is_err() {
                            if let Some(ref handler) = event_handler {
                                handler(WsEvent::Disconnected {
                                    reason: "Failed to send auth".to_string(),
                                })
                                .await;
                            }
                            if !auto_reconnect {
                                break;
                            }
                            tokio::time::sleep(reconnect_delay).await;
                            continue;
                        }

                        // Create a channel for sending messages from this task
                        let (internal_tx, mut internal_rx) = mpsc::unbounded_channel::<WsMessage>();

                        // Take the receiver for the write task
                        let msg_rx_for_write = match msg_rx.take() {
                            Some(rx) => rx,
                            None => {
                                // This should never happen, but handle gracefully
                                if let Some(ref handler) = event_handler {
                                    handler(WsEvent::Disconnected {
                                        reason: "Internal error: message receiver lost".to_string(),
                                    })
                                    .await;
                                }
                                break;
                            }
                        };

                        // Write task - wraps receiver in Option for cancellation recovery
                        let write_task = tokio::spawn(async move {
                            let mut rx = msg_rx_for_write;
                            loop {
                                tokio::select! {
                                    Some(msg) = internal_rx.recv() => {
                                        let json = serde_json::to_string(&msg).unwrap();
                                        if write.send(Message::Text(json.into())).await.is_err() {
                                            break;
                                        }
                                    }
                                    Some(msg) = rx.recv() => {
                                        let json = serde_json::to_string(&msg).unwrap();
                                        if write.send(Message::Text(json.into())).await.is_err() {
                                            break;
                                        }
                                    }
                                    else => break,
                                }
                            }
                            rx
                        });

                        // Read task
                        let event_handler_clone = event_handler.clone();
                        let subscriptions_for_read = subscriptions_clone.clone();
                        let initial_subs = initial_subscriptions.clone();
                        let internal_tx_clone = internal_tx.clone();

                        let read_task = tokio::spawn(async move {
                            let mut authenticated = false;

                            while let Some(msg_result) = read.next().await {
                                match msg_result {
                                    Ok(Message::Text(text)) => {
                                        if let Ok(msg) = serde_json::from_str::<WsMessage>(&text) {
                                            let event = Self::parse_event(&msg);

                                            // Handle authentication success
                                            if matches!(event, WsEvent::Authenticated)
                                                && !authenticated
                                            {
                                                authenticated = true;

                                                // Register initial subscriptions
                                                for sub in &initial_subs {
                                                    let sub_msg = WsMessage::new(
                                                        sub.subscribe_route(),
                                                        sub.to_payload(),
                                                    );
                                                    let _ = internal_tx_clone.send(sub_msg);
                                                    subscriptions_for_read
                                                        .write()
                                                        .await
                                                        .insert(sub.clone());
                                                }
                                            }

                                            // Emit event to handler
                                            if let Some(ref handler) = event_handler_clone {
                                                handler(event).await;
                                            }
                                        }
                                    }
                                    Ok(Message::Close(_)) => {
                                        if let Some(ref handler) = event_handler_clone {
                                            handler(WsEvent::Disconnected {
                                                reason: "Connection closed by server".to_string(),
                                            })
                                            .await;
                                        }
                                        break;
                                    }
                                    Err(e) => {
                                        if let Some(ref handler) = event_handler_clone {
                                            handler(WsEvent::Disconnected {
                                                reason: format!("Read error: {}", e),
                                            })
                                            .await;
                                        }
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        });

                        // Wait for either task to complete
                        let recovered_rx = tokio::select! {
                            result = write_task => {
                                // Write task finished - get receiver back
                                result.ok()
                            }
                            _ = read_task => {
                                // Read task finished first - receiver is consumed by write task
                                // which is now orphaned. We cannot recover it, so return None.
                                None
                            }
                        };

                        msg_rx = recovered_rx;

                        // If msg_rx is None, the receiver is lost. The connection is broken
                        // and the sender will error on next send attempt.
                        if msg_rx.is_none() {
                            break;
                        }

                        if let Some(ref handler) = event_handler {
                            handler(WsEvent::Disconnected {
                                reason: "Connection lost".to_string(),
                            })
                            .await;
                        }
                    }
                    Err(e) => {
                        if let Some(ref handler) = event_handler {
                            handler(WsEvent::Disconnected {
                                reason: format!("Connection failed: {}", e),
                            })
                            .await;
                        }
                    }
                }

                if !auto_reconnect {
                    break;
                }

                tokio::time::sleep(reconnect_delay).await;
            }
        });

        // Wait a bit for initial connection
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(Self {
            sender: msg_tx,
            subscriptions,
        })
    }

    /// Parse a WebSocket message into an event.
    ///
    /// Routes follow the format: `@wfm|type/path` or `@wfm|type/path:parameter`
    /// Examples from the API:
    /// - `@wfm|cmd/auth/signIn:ok` -> Authenticated
    /// - `@wfm|event/reports/online` -> OnlineCount
    /// - `@wfm|event/status/set` -> StatusUpdate
    /// - `@wfm|event/subscriptions/newOrder` -> OrderCreated
    fn parse_event(msg: &WsMessage) -> WsEvent {
        let route = match ParsedRoute::parse(&msg.route) {
            Some(r) => r,
            None => {
                return WsEvent::Unknown {
                    route: msg.route.clone(),
                    payload: msg.payload.clone().unwrap_or_default(),
                };
            }
        };

        // Match on (msg_type, path) from the parsed route
        match (route.msg_type.as_str(), route.path.as_str()) {
            // === Command responses ===

            // Authentication response: @wfm|cmd/auth/signIn:ok or @wfm|cmd/auth/signIn:error
            ("cmd", "auth/signIn") => match route.parameter.as_deref() {
                Some("ok") => WsEvent::Authenticated,
                Some("error") => WsEvent::AuthenticationFailed {
                    error: msg
                        .payload
                        .as_ref()
                        .and_then(|p| p["error"].as_str())
                        .unwrap_or("Unknown error")
                        .to_string(),
                },
                _ => WsEvent::Unknown {
                    route: msg.route.clone(),
                    payload: msg.payload.clone().unwrap_or_default(),
                },
            },

            // === Events ===

            // Online count: @wfm|event/reports/online
            // Payload: {"connections": 79086, "authorizedUsers": 46183}
            ("event", "reports/online") => {
                if let Some(ref payload) = msg.payload {
                    let connections = payload["connections"].as_u64().unwrap_or(0);
                    // Note: API uses "authorizedUsers" not "authorized"
                    let authorized = payload["authorizedUsers"].as_u64().unwrap_or(0);
                    WsEvent::OnlineCount {
                        connections,
                        authorized,
                    }
                } else {
                    WsEvent::OnlineCount {
                        connections: 0,
                        authorized: 0,
                    }
                }
            }

            // Status update: @wfm|event/status/set
            // Payload: {"status": "invisible", "statusSetAt": "2026-01-10T20:16:19Z"}
            ("event", "status/set") => {
                if let Some(ref payload) = msg.payload {
                    let status: UserStatus =
                        serde_json::from_value(payload["status"].clone()).unwrap_or_default();

                    // API sends "statusSetAt" - timestamp when status was set
                    // Note: Field is named status_until for backwards compatibility,
                    // but semantically represents when the status was set, not when it expires
                    let status_until = payload["statusSetAt"]
                        .as_str()
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc));

                    let activity: Option<Activity> =
                        serde_json::from_value(payload["activity"].clone()).ok();

                    WsEvent::StatusUpdate {
                        status,
                        status_until,
                        activity,
                    }
                } else {
                    WsEvent::StatusUpdate {
                        status: UserStatus::default(),
                        status_until: None,
                        activity: None,
                    }
                }
            }

            // New order from subscription: @wfm|event/subscriptions/newOrder
            ("event", "subscriptions/newOrder") => {
                if let Some(ref payload) = msg.payload {
                    if let Ok(order) = serde_json::from_value::<OrderListing>(payload.clone()) {
                        return WsEvent::OrderCreated { order };
                    }
                }
                WsEvent::Unknown {
                    route: msg.route.clone(),
                    payload: msg.payload.clone().unwrap_or_default(),
                }
            }

            // Order updated: @wfm|event/subscriptions/orderUpdated
            ("event", "subscriptions/orderUpdated") => {
                if let Some(ref payload) = msg.payload {
                    if let Ok(order) = serde_json::from_value::<OrderListing>(payload.clone()) {
                        return WsEvent::OrderUpdated { order };
                    }
                }
                WsEvent::Unknown {
                    route: msg.route.clone(),
                    payload: msg.payload.clone().unwrap_or_default(),
                }
            }

            // Order removed: @wfm|event/subscriptions/orderRemoved
            ("event", "subscriptions/orderRemoved") => {
                if let Some(ref payload) = msg.payload {
                    let order_id = payload["id"]
                        .as_str()
                        .or_else(|| payload["orderId"].as_str())
                        .unwrap_or("")
                        .to_string();
                    WsEvent::OrderRemoved { order_id }
                } else {
                    WsEvent::OrderRemoved {
                        order_id: String::new(),
                    }
                }
            }

            // Session revoked: @wfm|event/auth/revoke
            ("event", "auth/revoke") => WsEvent::SessionRevoked,

            // Verification update: @wfm|event/user/verificationUpdate
            ("event", "user/verificationUpdate") => {
                if let Some(ref payload) = msg.payload {
                    WsEvent::VerificationUpdate {
                        ingame_name: payload["ingameName"].as_str().unwrap_or("").to_string(),
                        verified: payload["verified"].as_bool().unwrap_or(false),
                    }
                } else {
                    WsEvent::VerificationUpdate {
                        ingame_name: String::new(),
                        verified: false,
                    }
                }
            }

            // Ban event: @wfm|event/user/banned
            ("event", "user/banned") => {
                if let Some(ref payload) = msg.payload {
                    let until = payload["banUntil"]
                        .as_str()
                        .or_else(|| payload["until"].as_str())
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(chrono::Utc::now);

                    WsEvent::Banned {
                        until,
                        message: payload["banMessage"]
                            .as_str()
                            .or_else(|| payload["message"].as_str())
                            .unwrap_or("")
                            .to_string(),
                    }
                } else {
                    WsEvent::Banned {
                        until: chrono::Utc::now(),
                        message: String::new(),
                    }
                }
            }

            // Warning event: @wfm|event/user/warned
            ("event", "user/warned") => WsEvent::Warned {
                message: msg
                    .payload
                    .as_ref()
                    .and_then(|p| p["warnMessage"].as_str().or_else(|| p["message"].as_str()))
                    .unwrap_or("")
                    .to_string(),
            },

            // Ban lifted: @wfm|event/user/banLifted
            ("event", "user/banLifted") => WsEvent::BanLifted,

            // Warning lifted: @wfm|event/user/warningLifted
            ("event", "user/warningLifted") => WsEvent::WarningLifted,

            // Fallback for unknown events
            _ => WsEvent::Unknown {
                route: msg.route.clone(),
                payload: msg.payload.clone().unwrap_or_default(),
            },
        }
    }

    /// Subscribe to real-time updates.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::ws::Subscription;
    /// # use wf_market::ws::WebSocket;
    /// # async fn example(ws: &WebSocket) -> wf_market::Result<()> {
    ///
    /// ws.subscribe(Subscription::item("mesa_prime_set")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(&self, subscription: Subscription) -> Result<()> {
        let msg = WsMessage::new(subscription.subscribe_route(), subscription.to_payload());

        self.sender
            .send(msg)
            .map_err(|e| Error::WebSocket(WsError::SendFailed(e.to_string())))?;

        self.subscriptions.write().await.insert(subscription);
        Ok(())
    }

    /// Unsubscribe from updates.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::ws::Subscription;
    /// # use wf_market::ws::WebSocket;
    /// # async fn example(ws: &WebSocket) -> wf_market::Result<()> {
    ///
    /// let sub = Subscription::item("mesa_prime_set");
    /// ws.unsubscribe(&sub).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unsubscribe(&self, subscription: &Subscription) -> Result<()> {
        let msg = WsMessage::new(subscription.unsubscribe_route(), subscription.to_payload());

        self.sender
            .send(msg)
            .map_err(|e| Error::WebSocket(WsError::SendFailed(e.to_string())))?;

        self.subscriptions.write().await.remove(subscription);
        Ok(())
    }

    /// Set user status.
    ///
    /// # Arguments
    ///
    /// * `status` - New status (online, ingame, invisible)
    /// * `duration` - Optional duration in seconds (60-21600)
    /// * `activity` - Optional in-game activity
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::ws::{UserStatus, Activity, ActivityType};
    /// # use wf_market::ws::WebSocket;
    /// # async fn example(ws: &WebSocket) -> wf_market::Result<()> {
    ///
    /// ws.set_status(
    ///     UserStatus::Ingame,
    ///     Some(3600), // 1 hour
    ///     Some(Activity::new(ActivityType::OnMission).with_details("Index")),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_status(
        &self,
        status: UserStatus,
        duration: Option<u64>,
        activity: Option<Activity>,
    ) -> Result<()> {
        let status_str = match status {
            UserStatus::Online => "online",
            UserStatus::Ingame => "ingame",
            UserStatus::Invisible => "invisible",
            UserStatus::Offline => "offline",
        };

        let activity_json = activity.map(|a| serde_json::to_value(a).unwrap());
        let msg = WsMessage::set_status(status_str, duration, activity_json);

        self.sender
            .send(msg)
            .map_err(|e| Error::WebSocket(WsError::SendFailed(e.to_string())))?;

        Ok(())
    }

    /// Sign out from the WebSocket (but keep connection open).
    pub async fn sign_out(&self) -> Result<()> {
        let msg = WsMessage::sign_out();

        self.sender
            .send(msg)
            .map_err(|e| Error::WebSocket(WsError::SendFailed(e.to_string())))?;

        Ok(())
    }

    /// Get currently active subscriptions.
    pub async fn subscriptions(&self) -> Vec<Subscription> {
        self.subscriptions.read().await.iter().cloned().collect()
    }

    /// Send a raw message (for advanced/undocumented endpoints).
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use wf_market::ws::WebSocket;
    /// # async fn example(ws: &WebSocket) -> wf_market::Result<()> {
    /// ws.send_raw("@custom/route", serde_json::json!({ "key": "value" }))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_raw(&self, route: &str, payload: serde_json::Value) -> Result<()> {
        let msg = WsMessage::new(route, payload);

        self.sender
            .send(msg)
            .map_err(|e| Error::WebSocket(WsError::SendFailed(e.to_string())))?;

        Ok(())
    }
}
