use futures_util::stream::{AbortHandle, Abortable};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};

use crate::enums::ApiVersion;
use crate::{
    errors::WsError,
    types::websocket::{MessageSender, Route, Router, WsClient, WsMessage},
};

// WebSocket client builder
pub struct WsClientBuilder {
    version: ApiVersion,
    router: Router,
    token: String,
    device_id: String,
}

impl WsClientBuilder {
    pub(crate) fn new(version: ApiVersion, token: String, device_id: String) -> Self {
        Self {
            version,
            router: Router::new(false),
            token,
            device_id,
        }
    }

    /// Register a callback for a specific path with optional parameter
    ///
    /// Examples:
    /// - `register_callback("cmd/subscribe/newOrders", callback)` - matches any parameter
    /// - `register_callback("cmd/subscribe/newOrders:ok", callback)` - matches only :ok parameter
    /// Note:
    /// - You can register multiple paths separated by commas, e.g., "path1,path2"
    pub fn register_callback<F>(mut self, path: &str, callback: F) -> Result<Self, WsError>
    where
        F: Fn(&WsMessage, &Route, &MessageSender) -> Result<(), WsError> + Send + Sync + 'static,
    {
        if path.contains(',') {
            let value = Arc::new(callback);
            for p in path.split(',') {
                self.router.register(p, value.clone())?;
            }
        } else {
            self.router.register(path, Arc::new(callback))?;
        }
        Ok(self)
    }

    /// Get list of paths reserved by the client for internal usage
    pub fn get_reserved_paths() -> Vec<&'static str> {
        Router::get_reserved_paths()
    }

    /*
       Set whether to log unhandled routes
    */
    pub fn set_log_unhandled(mut self, log: bool) -> Self {
        self.router.log_unhandled = log;
        self
    }
    /// Build and start the WebSocket client
    pub async fn build(self) -> Result<WsClient, WsError> {
        let router = Arc::new(self.router);
        let sender_holder = Arc::new(Mutex::new(None));
        let abort_handle_holder = Arc::new(Mutex::new(None));
        let should_stop = Arc::new(AtomicBool::new(false));

        tokio::spawn({
            let retry_interval = Duration::from_secs(5);
            let should_stop_spawn = Arc::clone(&should_stop);
            let sender_holder = Arc::clone(&sender_holder);
            let router = Arc::clone(&router);
            let abort_handle_holder = Arc::clone(&abort_handle_holder);

            async move {
                let version = &self.version;
                let ws_url = version.websocket_url();
                loop {
                    if should_stop_spawn.load(Ordering::Relaxed) {
                        break;
                    }
                    let mut request = ws_url.into_client_request().unwrap();
                    let headers = request.headers_mut();
                    if version == &ApiVersion::V2 {
                        headers.append("Sec-WebSocket-Protocol", "wfm".parse().unwrap());
                    } else if version == &ApiVersion::V1 {
                        headers.append("cookie", format!("JWT={}", self.token).parse().unwrap());
                    }
                    headers.append("User-Agent", "wf-market-rs".parse().unwrap());

                    match connect_async(request).await {
                        Ok((ws_stream, _)) => {
                            let ws_error = Arc::new(Mutex::new(None));
                            let ws_error_write = Arc::clone(&ws_error);
                            let ws_error_read = Arc::clone(&ws_error);
                            let (mut write, read) = ws_stream.split();
                            let (tx, mut rx) = mpsc::unbounded_channel::<WsMessage>();
                            let sender = MessageSender {
                                version: version.clone(),
                                tx: tx.clone(),
                            };

                            // Send connection message to the router
                            WsClient::send_ws_message(
                                &router,
                                &WsMessage::connect(version.clone()),
                                &sender,
                            )
                            .unwrap();

                            // Send authentication
                            if version == &ApiVersion::V2 {
                                let auth_payload = json!({
                                    "token": self.token,
                                    "deviceId": self.device_id,
                                });
                                match sender.send_request("@wfm|cmd/auth/signIn", auth_payload) {
                                    Ok(_) => {
                                        println!("Authentication request sent successfully.");
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to send authentication request: {:?}", e);
                                        continue; // Retry connection
                                    }
                                }
                            }

                            *sender_holder.lock().unwrap() = Some(sender.clone());

                            // Create and store abort handle
                            let (abort_handle, abort_registration) = AbortHandle::new_pair();
                            *abort_handle_holder.lock().unwrap() = Some(abort_handle.clone());

                            // Write task (wrapped in Abortable) Is responsible for sending messages
                            // It will be aborted if the read task fails or ends
                            let write_task = tokio::spawn(Abortable::new(
                                {
                                    async move {
                                        let ws_error_write = Arc::clone(&ws_error_write);
                                        while let Some(msg) = rx.recv().await {
                                            if let Ok(json) = serde_json::to_string(&msg) {
                                                if let Err(e) = write
                                                    .send(Message::Text(Utf8Bytes::from(json)))
                                                    .await
                                                {
                                                    eprintln!("Write failed: {}", e);
                                                    *ws_error_write.lock().unwrap() = Some(e);
                                                    break;
                                                }
                                            }
                                        }
                                        println!("Write task ended.");
                                    }
                                },
                                abort_registration,
                            ));

                            // Read task (will trigger abort on write if it fails or ends)
                            let read_task = tokio::spawn({
                                let sender = sender.clone();
                                let version = version.clone();
                                let router = Arc::clone(&router);
                                let abort_handle = abort_handle.clone(); // Move handle in
                                let should_stop_read = Arc::clone(&should_stop_spawn);
                                let mut read = read;

                                async move {
                                    let ws_error_read = Arc::clone(&ws_error_read);
                                    loop {
                                        // Check stop signal before trying to read
                                        if should_stop_read.load(Ordering::Relaxed) {
                                            break;
                                        }

                                        // Use timeout to avoid blocking indefinitely on read
                                        match tokio::time::timeout(
                                            Duration::from_millis(100),
                                            read.next(),
                                        )
                                        .await
                                        {
                                            Ok(Some(msg)) => match msg {
                                                Ok(Message::Text(text)) => {
                                                    if let Err(e) = WsClient::handle_text_message(
                                                        &router,
                                                        &text,
                                                        &sender,
                                                        version.clone(),
                                                    ) {
                                                        eprintln!("Handle error: {:?}", e);
                                                    }
                                                }
                                                Ok(Message::Close(_)) => {
                                                    println!("Connection closed by server.");
                                                    break;
                                                }
                                                Ok(_) => (),
                                                Err(e) => {
                                                    eprintln!("Read error: {}", e);
                                                    *ws_error_read.lock().unwrap() = Some(e);
                                                    break;
                                                }
                                            },
                                            Ok(None) => {
                                                break;
                                            }
                                            Err(_) => {
                                                // Timeout occurred, continue loop to check stop signal
                                                continue;
                                            }
                                        }
                                    }
                                    // If we exit the read loop, abort the write task
                                    abort_handle.abort();
                                }
                            });

                            // Wait for both tasks
                            let _ = tokio::join!(read_task, write_task);
                            // Send a message to the sender to indicate disconnection
                            let reason = if should_stop_spawn.load(Ordering::Relaxed) {
                                "Manual disconnect"
                            } else {
                                &format!(
                                    "Connection lost: {:?} will retry in {} seconds",
                                    ws_error.lock().unwrap(),
                                    retry_interval.as_secs()
                                )
                            };
                            WsClient::send_ws_message(
                                &router,
                                &WsMessage::disconnect(
                                    reason,
                                    retry_interval.as_secs(),
                                    self.version.clone(),
                                ),
                                &sender,
                            )
                            .unwrap();
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }

                        Err(err) => {
                            eprintln!("WebSocket connection failed: {}", err);
                            // Send connection failed message to the router
                            let failed_message = WsMessage::reconnect(
                                retry_interval.as_secs(),
                                err,
                                self.version.clone(),
                            )
                            .with_id("INTERNAL");

                            // Create a temporary sender for the error message
                            let (temp_tx, _temp_rx) = mpsc::unbounded_channel::<WsMessage>();
                            let temp_sender = MessageSender {
                                version: self.version.clone(),
                                tx: temp_tx,
                            };

                            if let Err(e) = router.route_message(&failed_message, &temp_sender) {
                                eprintln!("Failed to route connection failed message: {:?}", e);
                            }

                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        });

        tokio::time::sleep(Duration::from_secs(1)).await;

        Ok(WsClient {
            sender: Arc::clone(&sender_holder),
            abort_handle: Arc::clone(&abort_handle_holder),
            should_stop: Arc::clone(&should_stop),
        })
    }
}
