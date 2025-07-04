use crate::{
    endpoints::*,
    enums::*,
    errors::*,
    types::{UserPrivate, websocket::WsClientBuilder},
    utils::*,
};
use governor::{
    RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use reqwest::{
    Method,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use serde_json::Value;
use std::{
    collections::HashMap,
    marker::PhantomData,
    num::{NonZero, NonZeroU32},
    sync::{Arc, OnceLock},
};

const REQUESTS_PER_SECOND: NonZeroU32 = NonZero::new(3).unwrap();

#[derive(Clone, Debug)]
pub struct Unauthenticated;
#[derive(Clone, Debug)]
pub struct Authenticated;

pub trait IsAuthenticated {}
pub trait IsUnauthenticated {}

impl IsAuthenticated for Authenticated {}
impl IsUnauthenticated for Unauthenticated {}

#[derive(Debug, Clone)]
pub struct Client<State = Unauthenticated> {
    self_arc: OnceLock<Arc<Client<State>>>,
    token: String,
    device_id: String,
    language: Language,
    platform: Platform,
    crossplay: bool,
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    // Routes
    manifest_route: OnceLock<Arc<ManifestRoute<State>>>,
    item_route: OnceLock<Arc<ItemRoute<State>>>,
    riven_route: OnceLock<Arc<RivenRoute<State>>>,
    lich_route: OnceLock<Arc<LichRoute<State>>>,
    sister_route: OnceLock<Arc<SisterRoute<State>>>,
    order_route: OnceLock<Arc<OrderRoute<State>>>,
    user_route: OnceLock<Arc<UserRoute<State>>>,
    achievement_route: OnceLock<Arc<AchievementRoute<State>>>,
    authentication_route: OnceLock<Arc<AuthenticationRoute<State>>>,
    // V1 Routes
    chat_route: OnceLock<Arc<ChatRoute<State>>>,
    auction_route: OnceLock<Arc<AuctionRoute<State>>>,
    _state: PhantomData<State>,
}
impl<State: Clone + 'static> Client<State> {
    fn arc(&self) -> Arc<Self> {
        self.self_arc
            .get_or_init(|| {
                Arc::new(Self {
                    self_arc: OnceLock::new(),
                    token: self.token.clone(),
                    device_id: self.device_id.clone(),
                    platform: self.platform,
                    language: self.language,
                    crossplay: self.crossplay,
                    manifest_route: self.manifest_route.clone(),
                    item_route: self.item_route.clone(),
                    riven_route: self.riven_route.clone(),
                    lich_route: self.lich_route.clone(),
                    sister_route: self.sister_route.clone(),
                    order_route: self.order_route.clone(),
                    user_route: self.user_route.clone(),
                    achievement_route: self.achievement_route.clone(),
                    authentication_route: self.authentication_route.clone(),
                    chat_route: self.chat_route.clone(),
                    auction_route: self.auction_route.clone(),
                    limiter: self.limiter.clone(),
                    _state: PhantomData,
                })
            })
            .clone()
    }

    pub async fn call_api<T: serde::de::DeserializeOwned>(
        &self,
        version: ApiVersion,
        method: Method,
        path: &str,
        body: Option<Value>,
        headers: Option<HashMap<String, String>>,
    ) -> Result<(T, HeaderMap), ApiError> {
        let url = version.api_url().to_owned() + path;
        let mut default_headers = reqwest::header::HeaderMap::new();

        let prefix = match version {
            ApiVersion::V1 => "JWT",
            ApiVersion::V2 => "Bearer",
        };

        // Add the required headers
        default_headers.insert("language", self.language.as_str().parse().unwrap());
        default_headers.insert("platform", self.platform.as_str().parse().unwrap());
        default_headers.insert("crossplay", self.crossplay.to_string().parse().unwrap());
        default_headers.insert(
            "User-Agent",
            format!(
                "wf-market-rs/{} ({}; {})",
                env!("CARGO_PKG_VERSION"),
                self.platform.as_str(),
                self.language.as_str()
            )
            .parse()
            .unwrap(),
        );

        // If the client is authenticated, add the token to the headers
        if self.token != "" {
            default_headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("{} {}", prefix, self.token).parse().unwrap(),
            );
        }

        // Add any additional headers provided
        if let Some(ref items) = headers {
            for (key, value) in items.iter() {
                default_headers.insert(
                    HeaderName::from_bytes(key.as_bytes()).unwrap(),
                    HeaderValue::from_str(value).unwrap(),
                );
            }
        }

        // Create the HTTP client with the headers
        let http_client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        let mut builder = http_client.request(method, &url);
        // If the client needs a body, serialize it
        if let Some(b) = body {
            println!("Payload: {:?}", b);
            builder = builder.json(&b);
        }
        self.limiter.until_ready().await;

        match builder.send().await {
            Ok(resp) => {
                let headers = resp.headers().clone();
                let status = resp.status();

                let body = resp
                    .text()
                    .await
                    .map_err(|_| ApiError::Unknown("Error".to_string()))?;

                // Check if the status code indicates an error
                match status {
                    reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => {}
                    reqwest::StatusCode::UNAUTHORIZED => {
                        return Err(ApiError::Unauthorized);
                    }
                    reqwest::StatusCode::NOT_FOUND => {
                        return Err(ApiError::NotFound(format!(
                            "Resource not found: {}, Message: {}",
                            url, body
                        )));
                    }
                    reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::FORBIDDEN => {
                        match serde_json::from_str::<ResponseError>(&body) {
                            Ok(api_result) => {
                                return Err(ApiError::WFMError(api_result));
                            }
                            Err(e) => {
                                return Err(ApiError::ParsingError(
                                    format!(
                                        "Error Parsing Bad Request Error: {:?}, Body: {:?}",
                                        e,
                                        body.to_string()
                                    )
                                    .to_string(),
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(ApiError::Unknown(format!(
                            "Unexpected status code: {}",
                            status
                        )));
                    }
                }

                let data = serde_json::from_str::<T>(&body);

                match data {
                    Ok(data) => Ok((data, headers)),
                    Err(err) => Err(ApiError::ParsingError(format!(
                        "Error Parsing: {:?}, Body: {}",
                        err, body
                    ))),
                }
            }
            Err(_) => Err(ApiError::RequestError),
        }
    }

    /**
    Set the language for the client
    # Arguments
    - `language`: The language to set for the client, defaults to English if not set
    # Returns
    The client with the language set
    */
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /**
    Set the platform for the client
    # Arguments
    - `platform`: The platform to set for the client, defaults to PC if not set
    # Returns
    The client with the platform set
    */
    pub fn with_platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    /**
    Set the crossplay setting for the client
    # Arguments
    - `crossplay`: Whether to enable crossplay or not, defaults to true if not
    # Returns
    The client with the crossplay setting set
    */
    pub fn with_crossplay(mut self, crossplay: bool) -> Self {
        self.crossplay = crossplay;
        self
    }
    // Endpoint methods to access routes
    pub fn manifest(&self) -> Arc<ManifestRoute<State>> {
        self.manifest_route
            .get_or_init(|| ManifestRoute::new(self.arc()))
            .clone()
    }
    pub fn order(&self) -> Arc<OrderRoute<State>> {
        self.order_route
            .get_or_init(|| OrderRoute::new(self.arc()))
            .clone()
    }
    pub fn user(&self) -> Arc<UserRoute<State>> {
        self.user_route
            .get_or_init(|| UserRoute::new(self.arc()))
            .clone()
    }
    pub fn authentication(&self) -> Arc<AuthenticationRoute<State>> {
        self.authentication_route
            .get_or_init(|| AuthenticationRoute::new(self.arc()))
            .clone()
    }
    pub fn lich(&self) -> Arc<LichRoute<State>> {
        self.lich_route
            .get_or_init(|| LichRoute::new(self.arc()))
            .clone()
    }
    pub fn sister(&self) -> Arc<SisterRoute<State>> {
        self.sister_route
            .get_or_init(|| SisterRoute::new(self.arc()))
            .clone()
    }
    pub fn item(&self) -> Arc<ItemRoute<State>> {
        self.item_route
            .get_or_init(|| ItemRoute::new(self.arc()))
            .clone()
    }
    pub fn riven(&self) -> Arc<RivenRoute<State>> {
        self.riven_route
            .get_or_init(|| RivenRoute::new(self.arc()))
            .clone()
    }
    pub fn chat(&self) -> Arc<ChatRoute<State>> {
        self.chat_route
            .get_or_init(|| ChatRoute::new(self.arc()))
            .clone()
    }
    pub fn auction(&self) -> Arc<AuctionRoute<State>> {
        self.auction_route
            .get_or_init(|| AuctionRoute::new(self.arc()))
            .clone()
    }
    pub fn achievement(&self) -> Arc<AchievementRoute<State>> {
        self.achievement_route
            .get_or_init(|| AchievementRoute::new(self.arc()))
            .clone()
    }
}

impl Client<Unauthenticated> {
    pub fn new() -> Self {
        Self {
            self_arc: OnceLock::new(),
            token: String::new(),
            device_id: String::new(),
            language: Language::default(),
            platform: Platform::default(),
            crossplay: true,
            manifest_route: OnceLock::new(),
            item_route: OnceLock::new(),
            riven_route: OnceLock::new(),
            lich_route: OnceLock::new(),
            sister_route: OnceLock::new(),
            order_route: OnceLock::new(),
            user_route: OnceLock::new(),
            achievement_route: OnceLock::new(),
            authentication_route: OnceLock::new(),
            chat_route: OnceLock::new(),
            auction_route: OnceLock::new(),
            limiter: build_limiter(REQUESTS_PER_SECOND).into(),
            _state: PhantomData,
        }
    }
    /**
     * Creates a new `Client` with the specified parameters.
     * # Arguments
     * - `username`: The username to use for authentication
     * - `password`: The password to use for authentication
     * - `device_id`: The device ID to use for authentication
     * # Returns
     * A `Result` containing the authenticated `Client` or an `AuthError` if authentication fails.
     */
    pub async fn login(
        self,
        username: &str,
        password: &str,
        device_id: &str,
    ) -> Result<Client<Authenticated>, AuthError> {
        let (_, token) = match self
            .authentication()
            .signin(username, password, device_id)
            .await
        {
            Ok((user, token)) => (user, token),
            Err(e) => return Err(e),
        };

        let new_client = Client::<Authenticated> {
            self_arc: OnceLock::new(),
            token,
            device_id: device_id.to_string(),
            platform: self.platform,
            language: self.language,
            crossplay: self.crossplay,
            manifest_route: OnceLock::new(),
            item_route: OnceLock::new(),
            riven_route: OnceLock::new(),
            lich_route: OnceLock::new(),
            sister_route: OnceLock::new(),
            order_route: OnceLock::new(),
            user_route: OnceLock::new(),
            achievement_route: OnceLock::new(),
            authentication_route: OnceLock::new(),
            chat_route: OnceLock::new(),
            auction_route: OnceLock::new(),
            limiter: self.limiter.clone(),
            _state: PhantomData,
        };

        let arc = Arc::new(new_client);

        // Set the self_arc inside the client to point to this Arc
        arc.self_arc.set(arc.clone()).unwrap();

        // Refresh the internal data
        match arc.refresh().await {
            Ok(_) => {}
            Err(e) => {
                return Err(AuthError::Unknown(format!(
                    "Failed to refresh user data: {:?}",
                    e
                )));
            }
        }
        // Copy routes if they were initialized
        if let Some(manifest) = self.manifest_route.get() {
            arc.manifest_route
                .set(ManifestRoute::from_existing(manifest, arc.clone()))
                .ok();
        }
        if let Some(order) = self.order_route.get() {
            arc.order_route
                .set(OrderRoute::from_existing(order, arc.clone()))
                .ok();
        }
        if let Some(user) = self.user_route.get() {
            arc.user_route
                .set(UserRoute::from_existing(user, arc.clone()))
                .ok();
        }
        if let Some(auth) = self.authentication_route.get() {
            arc.authentication_route
                .set(AuthenticationRoute::from_existing(auth, arc.clone()))
                .ok();
        }
        if let Some(item) = self.item_route.get() {
            arc.item_route
                .set(ItemRoute::from_existing(item, arc.clone()))
                .ok();
        }
        if let Some(riven) = self.riven_route.get() {
            arc.riven_route
                .set(RivenRoute::from_existing(riven, arc.clone()))
                .ok();
        }
        if let Some(lich) = self.lich_route.get() {
            arc.lich_route
                .set(LichRoute::from_existing(lich, arc.clone()))
                .ok();
        }
        if let Some(sister) = self.sister_route.get() {
            arc.sister_route
                .set(SisterRoute::from_existing(sister, arc.clone()))
                .ok();
        }
        if let Some(chat) = self.chat_route.get() {
            arc.chat_route
                .set(ChatRoute::from_existing(chat, arc.clone()))
                .ok();
        }
        if let Some(auction) = self.auction_route.get() {
            arc.auction_route
                .set(AuctionRoute::from_existing(auction, arc.clone()))
                .ok();
        }
        if let Some(achievement) = self.achievement_route.get() {
            arc.achievement_route
                .set(AchievementRoute::from_existing(achievement, arc.clone()))
                .ok();
        }
        // Return the new authenticated client

        Ok(Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone()))
    }
}

impl Client<Authenticated> {
    pub fn new_authenticated(token: &str, device_id: &str) -> Self {
        // This constructor is only for internal use, it assumes the client is authenticated
        // If this panics, we got hit by a cosmic particle
        assert!(!token.is_empty(), "Token cannot be empty");
        assert!(!device_id.is_empty(), "Device ID cannot be empty");
        let client = Client::<Authenticated> {
            self_arc: OnceLock::new(),
            token: token.to_string(),
            device_id: device_id.to_string(),
            platform: Platform::default(),
            language: Language::default(),
            crossplay: true,
            manifest_route: OnceLock::new(),
            item_route: OnceLock::new(),
            riven_route: OnceLock::new(),
            lich_route: OnceLock::new(),
            sister_route: OnceLock::new(),
            order_route: OnceLock::new(),
            user_route: OnceLock::new(),
            achievement_route: OnceLock::new(),
            authentication_route: OnceLock::new(),
            chat_route: OnceLock::new(),
            auction_route: OnceLock::new(),
            limiter: build_limiter(REQUESTS_PER_SECOND).into(),
            _state: PhantomData,
        };
        client
    }
    /**
     * Returns the current user data
     * # Returns
     * If the user data is successfully fetched.
     */
    pub fn get_user(&self) -> Result<UserPrivate, ApiError> {
        match self.user().get_user() {
            Ok(user) => Ok(user),
            Err(e) => Err(ApiError::Unknown(format!(
                "Failed to get user data: {:?}",
                e
            ))),
        }
    }
    /**
    Return the authentication token

    # Returns
    The users JWT token
    */
    pub fn get_token(&self) -> String {
        // Only accessible on authed clients, if this panics we got hit by a cosmic particle
        self.token.clone()
    }
    /**
    Create a WebSocket builder

    # Returns
    A WsClient Builder
    */
    pub fn create_websocket(&self, version: ApiVersion) -> WsClientBuilder {
        WsClientBuilder::new(version, self.get_token(), self.get_device_id())
    }
    /**
    Returns the clients device id

    # Returns
    The Device ID used when authenticating
    */
    pub fn get_device_id(&self) -> String {
        // Again, panics, cosmic particle, you get the gist of it now
        self.device_id.clone()
    }

    /**
    Returns the current internal data
    # Returns
    If the data is successfully refreshed.
    */
    pub async fn refresh(&self) -> Result<String, ApiError> {
        match self.user().me().await {
            Ok(_) => {}
            Err(e) => {
                return Err(ApiError::Unknown(format!(
                    "Failed to refresh user data: {:?}",
                    e
                )));
            }
        }
        Ok("Successfully refreshed user data".to_string())
    }
}
