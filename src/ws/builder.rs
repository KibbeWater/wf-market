//! WebSocket client builder.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use crate::client::{Authenticated, Client};
use crate::error::Result;

use super::client::WebSocket;
use super::events::WsEvent;
use super::subscription::Subscription;

/// Type alias for boxed async event handler.
pub type BoxedEventHandler =
    Arc<dyn Fn(WsEvent) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Builder for creating a WebSocket connection.
///
/// # Example
///
/// ```ignore
/// use wf_market::{Client, Credentials};
/// use wf_market::ws::{WsEvent, Subscription};
///
/// async fn example() -> wf_market::Result<()> {
///     let client = Client::from_credentials(/* ... */).await?;
///
///     let ws = client.websocket()
///         .on_event(|event| async move {
///             match event {
///                 WsEvent::Connected => println!("Connected!"),
///                 WsEvent::OnlineCount { authorized, .. } => {
///                     println!("Users online: {}", authorized);
///                 }
///                 _ => {}
///             }
///         })
///         .subscribe(Subscription::all_new_orders())
///         .auto_reconnect(true)
///         .reconnect_delay(Duration::from_secs(5))
///         .connect()
///         .await?;
///
///     Ok(())
/// }
/// # fn main() {}
/// ```
pub struct WebSocketBuilder<'a> {
    pub(crate) client: &'a Client<Authenticated>,
    pub(crate) event_handler: Option<BoxedEventHandler>,
    pub(crate) subscriptions: Vec<Subscription>,
    pub(crate) auto_reconnect: bool,
    pub(crate) reconnect_delay: Duration,
}

impl<'a> WebSocketBuilder<'a> {
    /// Create a new WebSocket builder.
    pub(crate) fn new(client: &'a Client<Authenticated>) -> Self {
        Self {
            client,
            event_handler: None,
            subscriptions: Vec::new(),
            auto_reconnect: true,
            reconnect_delay: Duration::from_secs(5),
        }
    }

    /// Set the async event handler.
    ///
    /// All WebSocket events are delivered to this handler.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::ws::WsEvent;
    /// # use wf_market::{Client, Credentials};
    /// # async fn example() -> wf_market::Result<()> {
    /// # let client = Client::from_credentials(Credentials::new("", "", "")).await?;
    ///
    /// let ws = client.websocket()
    ///     .on_event(|event| async move {
    ///         match event {
    ///             WsEvent::OnlineCount { authorized, .. } => {
    ///                 println!("Online: {}", authorized);
    ///             }
    ///             _ => {}
    ///         }
    ///     })
    ///     .connect()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn on_event<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(WsEvent) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.event_handler = Some(Arc::new(move |event| {
            Box::pin(handler(event)) as Pin<Box<dyn Future<Output = ()> + Send>>
        }));
        self
    }

    /// Add a subscription to register on connect.
    ///
    /// Subscriptions will be automatically registered after
    /// successful authentication.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::ws::Subscription;
    /// # use wf_market::{Client, Credentials};
    /// # async fn example() -> wf_market::Result<()> {
    /// # let client = Client::from_credentials(Credentials::new("", "", "")).await?;
    ///
    /// let ws = client.websocket()
    ///     .subscribe(Subscription::all_new_orders())
    ///     .subscribe(Subscription::item("nikana_prime_set"))
    ///     .connect()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe(mut self, subscription: Subscription) -> Self {
        self.subscriptions.push(subscription);
        self
    }

    /// Enable or disable automatic reconnection.
    ///
    /// Default: `true`
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }

    /// Set the delay between reconnection attempts.
    ///
    /// Default: 5 seconds
    pub fn reconnect_delay(mut self, delay: Duration) -> Self {
        self.reconnect_delay = delay;
        self
    }

    /// Connect to the WebSocket server.
    ///
    /// This will:
    /// 1. Establish a WebSocket connection
    /// 2. Authenticate using the client's token
    /// 3. Register any initial subscriptions
    /// 4. Start the event loop
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be established.
    pub async fn connect(self) -> Result<WebSocket> {
        WebSocket::connect(self).await
    }
}

// Implement for Client
impl Client<Authenticated> {
    /// Create a WebSocket connection builder.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::{Client, Credentials};
    /// use wf_market::ws::{WsEvent, Subscription};
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::from_credentials(/* ... */).await?;
    ///
    ///     let ws = client.websocket()
    ///         .on_event(|event| async move {
    ///             println!("{:?}", event);
    ///         })
    ///         .subscribe(Subscription::all_new_orders())
    ///         .connect()
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// # fn main() {}
    /// ```
    #[cfg(feature = "websocket")]
    pub fn websocket(&self) -> WebSocketBuilder<'_> {
        WebSocketBuilder::new(self)
    }
}
