use crate::{
    endpoints::*,
    enums::*,
    errors::*,
    types::{UserPrivate, websocket::WsClientBuilder},
    utils::*,
};
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use reqwest::{
    Method,
    header::{HeaderMap, HeaderName, HeaderValue, RETRY_AFTER},
};
use serde_json::Value;
use std::{
    collections::HashMap,
    marker::PhantomData,
    num::{NonZero, NonZeroU32},
    sync::{Arc, Mutex, OnceLock},
};

const REQUESTS_PER_SECOND: NonZeroU32 = NonZero::new(3).unwrap();

#[derive(Debug, Clone)]
pub enum RateLimitTier {
    PerSecond(u32),
    PerMinute(u32),
    PerHour(u32),
}

impl RateLimitTier {
    pub fn downgrade(&self) -> RateLimitTier {
        match *self {
            RateLimitTier::PerSecond(_) => RateLimitTier::PerMinute(50),
            RateLimitTier::PerMinute(_) => RateLimitTier::PerHour(2),
            RateLimitTier::PerHour(_) => RateLimitTier::PerHour(1),
        }
    }

    pub fn current_limit(&self) -> u32 {
        match *self {
            RateLimitTier::PerSecond(x)
            | RateLimitTier::PerMinute(x)
            | RateLimitTier::PerHour(x) => x,
        }
    }

    pub fn sub_tract(&self, value: u32) -> RateLimitTier {
        match *self {
            RateLimitTier::PerSecond(x) => {
                if x > value {
                    RateLimitTier::PerSecond(x - value)
                } else {
                    RateLimitTier::PerSecond(1)
                }
            }
            RateLimitTier::PerMinute(x) => {
                if x > value {
                    RateLimitTier::PerMinute(x - value)
                } else {
                    RateLimitTier::PerMinute(1)
                }
            }
            RateLimitTier::PerHour(x) => {
                if x > value {
                    RateLimitTier::PerHour(x - value)
                } else {
                    RateLimitTier::PerHour(1)
                }
            }
        }
    }
    pub fn quota_type(&self) -> &'static str {
        match *self {
            RateLimitTier::PerSecond(_) => "per_second",
            RateLimitTier::PerMinute(_) => "per_minute",
            RateLimitTier::PerHour(_) => "per_hour",
        }
    }
    pub fn build_limiter(&self) -> RateLimiter<NotKeyed, InMemoryState, DefaultClock> {
        match *self {
            RateLimitTier::PerSecond(x) => {
                RateLimiter::direct(Quota::per_second(NonZeroU32::new(x).unwrap()))
            }
            RateLimitTier::PerMinute(x) => {
                RateLimiter::direct(Quota::per_minute(NonZeroU32::new(x).unwrap()))
            }
            RateLimitTier::PerHour(x) => {
                RateLimiter::direct(Quota::per_hour(NonZeroU32::new(x).unwrap()))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RouteLimiter {
    pub quota_type: RateLimitTier,
    pub limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    pub wait_time_sec: f64,
}
impl RouteLimiter {
    pub fn new(quota_type: RateLimitTier, wait_time_sec: f64) -> Self {
        Self {
            limiter: quota_type.build_limiter().into(),
            quota_type,
            wait_time_sec,
        }
    }
}

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
    per_route_limiter: Arc<Mutex<HashMap<String, RouteLimiter>>>,
    tracking: Arc<Mutex<HashMap<String, usize>>>,
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
                    per_route_limiter: self.per_route_limiter.clone(),
                    tracking: self.tracking.clone(),
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
        key: impl Into<String>,
        body: Option<Value>,
        headers: Option<HashMap<String, String>>,
    ) -> Result<(T, HeaderMap, RequestError), ApiError> {
        let url = version.api_url().to_owned() + path;
        let start = chrono::Utc::now();
        let mut default_headers = reqwest::header::HeaderMap::new();
        let key = key.into();

        self.tracking
            .lock()
            .unwrap()
            .entry(key.clone())
            .and_modify(|e| *e += 1)
            .or_insert(1);

        // Get Limiter for the route
        let (mut quota_type, limiter, wait_time_sec) = if let Some(route_limiter) =
            self.per_route_limiter.lock().unwrap().get(&key).cloned()
        {
            (
                route_limiter.quota_type.clone(),
                route_limiter.limiter.clone(),
                route_limiter.wait_time_sec,
            )
        } else {
            (
                RateLimitTier::PerSecond(REQUESTS_PER_SECOND.get()),
                self.limiter.clone(),
                0.0,
            )
        };

        // Create the error object for logging
        let mut error = RequestError::new(
            version.clone(),
            method.to_string(),
            url.clone(),
            body.clone(),
        );

        let prefix = match version {
            ApiVersion::V1 => "JWT",
            ApiVersion::V2 | ApiVersion::Custom(_, _) => "Bearer",
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

        // Add Headers for the error object
        error.set_headers(
            default_headers
                .iter()
                .map(|(k, v)| {
                    (
                        k.to_string(),
                        v.to_str().unwrap_or("Invalid Header").to_string(),
                    )
                })
                .collect(),
        );

        // Create the HTTP client with the headers
        let http_client = reqwest::Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        let mut builder = http_client.request(method, &url);
        // If the client needs a body, serialize it
        if let Some(b) = body {
            builder = builder.json(&b);
        }
        if wait_time_sec > 0.0 {
            tokio::time::sleep(std::time::Duration::from_secs_f64(wait_time_sec)).await;
            self.clear_wait_time(key.clone());
        }
        limiter.until_ready().await;

        match builder.send().await {
            Ok(resp) => {
                let end = chrono::Utc::now();
                error.duration_ms = Some((end - start).num_milliseconds() as u128);
                println!(
                    "API call to {} {} took ({})ms",
                    url,
                    error.method,
                    error.duration_ms.unwrap_or(0)
                );
                let headers = resp.headers().clone();
                let status = resp.status();
                error.set_status_code(status.as_u16());

                let body = resp.text().await.map_err(|e| {
                    error.set_content(format!("Failed to read response body: {}", e));
                    ApiError::RequestError(error.clone())
                })?;
                // Log the error with the response body
                error.set_content(body.clone());

                // Check if the status code indicates an error
                match status {
                    reqwest::StatusCode::OK | reqwest::StatusCode::CREATED => {}
                    reqwest::StatusCode::UNAUTHORIZED => {
                        return Err(ApiError::Unauthorized(error));
                    }
                    reqwest::StatusCode::TOO_MANY_REQUESTS => {
                        let retry_after = headers
                            .get(RETRY_AFTER)
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(1);
                        error.set_retry_after(Some(retry_after));
                        if quota_type.current_limit() > 1 {
                            quota_type = quota_type.sub_tract(1);
                        } else {
                            quota_type = quota_type.downgrade();
                        };
                        self.set_rate_limit_for_route(key, quota_type, retry_after as f64);
                        return Err(ApiError::TooManyRequests(error));
                    }
                    reqwest::StatusCode::INTERNAL_SERVER_ERROR
                    | reqwest::StatusCode::NOT_IMPLEMENTED
                    | reqwest::StatusCode::BAD_GATEWAY
                    | reqwest::StatusCode::SERVICE_UNAVAILABLE
                    | reqwest::StatusCode::GATEWAY_TIMEOUT
                    | reqwest::StatusCode::HTTP_VERSION_NOT_SUPPORTED
                    | reqwest::StatusCode::VARIANT_ALSO_NEGOTIATES
                    | reqwest::StatusCode::INSUFFICIENT_STORAGE
                    | reqwest::StatusCode::LOOP_DETECTED
                    | reqwest::StatusCode::NOT_EXTENDED
                    | reqwest::StatusCode::NETWORK_AUTHENTICATION_REQUIRED => {
                        return Err(ApiError::InternalServerError(error));
                    }
                    reqwest::StatusCode::BAD_REQUEST
                    | reqwest::StatusCode::FORBIDDEN
                    | reqwest::StatusCode::NOT_FOUND => {
                        // Attempt to parse the body as a ResponseError
                        let wfm_err = match version {
                            ApiVersion::V1 => {
                                let error_body = match serde_json::from_str::<Value>(&body) {
                                    Ok(v) => v,
                                    Err(e) => return Err(ApiError::ParsingError(error, e)),
                                };
                                ResponseError::from_v1(error_body)
                            }
                            ApiVersion::V2 => match serde_json::from_str::<ResponseError>(&body) {
                                Ok(api_result) => api_result,
                                Err(e) => return Err(ApiError::ParsingError(error, e)),
                            },
                            _ => match serde_json::from_str::<ResponseError>(&body) {
                                Ok(api_result) => api_result,
                                Err(e) => return Err(ApiError::ParsingError(error, e)),
                            },
                        };
                        // Set the error in the RequestError
                        error.set_error(Some(wfm_err.clone()));
                        if wfm_err.contains_error("app.order.error.exceededOrderLimitSamePrice") {
                            return Err(ApiError::OrderLimitExceededSamePrice(error));
                        } else if wfm_err.contains_error("app.order.error.exceededOrderLimit") {
                            return Err(ApiError::OrderLimitExceeded(error));
                        } else if wfm_err.contains_error("exceededAuctionLimit") {
                            return Err(ApiError::AuctionLimitExceeded(error));
                        } else if status == reqwest::StatusCode::FORBIDDEN {
                            return Err(ApiError::Forbidden(error));
                        } else if status == reqwest::StatusCode::NOT_FOUND {
                            return Err(ApiError::NotFound(error));
                        } else {
                            return Err(ApiError::BadRequest(error));
                        }
                    }
                    _ => match status.as_u16() {
                        500..=599 => {
                            return Err(ApiError::InternalServerError(error));
                        }
                        _ => {
                            return Err(ApiError::Unknown(format!(
                                "Unexpected status code: {} with body: {}",
                                status, body
                            )));
                        }
                    },
                }

                let data = serde_json::from_str::<T>(&body);

                match data {
                    Ok(data) => Ok((data, headers, error)),
                    Err(e) => Err(ApiError::ParsingError(error, e)),
                }
            }
            Err(e) => {
                let end = chrono::Utc::now();
                let mut details = format!("Request failed: {}", e);
                details.push_str(&format!("\n  URL: {}", url));
                if e.is_timeout() {
                    details.push_str("\n  Reason: timeout");
                } else if e.is_connect() {
                    details.push_str("\n  Reason: connection error");
                }
                if let Some(status) = e.status() {
                    details.push_str(&format!("\n  HTTP status: {}", status));
                }
                error.set_content(details);
                error.duration_ms = Some((end - start).num_milliseconds() as u128);
                Err(ApiError::RequestError(error))
            }
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

    /**
     * Sets the rate limit for the client
     * # Arguments
     * - `requests_per_second`: The maximum number of requests per second
     */
    pub fn set_rate_limit(&mut self, requests_per_second: NonZeroU32) {
        self.limiter = build_limiter(requests_per_second).into();
    }

    /**
     * Gets the tracking data for the client
     * # Returns
     * An `Arc<Mutex<HashMap<String, usize>>>` containing the tracking data
     */
    pub fn get_tracking(&self) -> Arc<Mutex<HashMap<String, usize>>> {
        self.tracking.clone()
    }

    /**
     * Gets the per-route limiter for the client
     * # Returns
     * An `Arc<Mutex<HashMap<String, RouteLimiter>>>` containing the per-route limiters
     */
    pub fn get_per_route_limiter(&self) -> Arc<Mutex<HashMap<String, RouteLimiter>>> {
        self.per_route_limiter.clone()
    }

    /**
     * Sets the rate limit for a specific route
     * # Arguments
     * - `route`: The route to set the rate limit for
     * - `requests_per_second`: The maximum number of requests per second for the route
     */
    fn set_rate_limit_for_route(
        &self,
        route: impl Into<String>,
        quota_type: RateLimitTier,
        wait_time_sec: f64,
    ) {
        let mut per_route_limiter = self.per_route_limiter.lock().unwrap();
        let route_str = route.into();
        let quota_type = quota_type.into();
        if let Some(limiter) = per_route_limiter.get_mut(&route_str) {
            *limiter = RouteLimiter::new(quota_type, wait_time_sec);
        } else {
            per_route_limiter.insert(route_str, RouteLimiter::new(quota_type, wait_time_sec));
        }
    }

    pub fn clear_wait_time(&self, route: impl Into<String>) {
        let mut per_route_limiter = self.per_route_limiter.lock().unwrap();
        let route_str = route.into();
        if let Some(limiter) = per_route_limiter.get_mut(&route_str) {
            limiter.wait_time_sec = 0.0;
        }
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
            auction_route: OnceLock::new(),
            authentication_route: OnceLock::new(),
            chat_route: OnceLock::new(),
            limiter: build_limiter(REQUESTS_PER_SECOND).into(),
            per_route_limiter: Arc::new(Mutex::new(HashMap::new())),
            tracking: Arc::new(Mutex::new(HashMap::new())),
            _state: PhantomData,
        }
    }

    async fn create_authenticated_client(
        &self,
        token: String,
        device_id: String,
        refresh: bool,
    ) -> Result<Client<Authenticated>, ApiError> {
        let client = Client::<Authenticated> {
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
            per_route_limiter: self.per_route_limiter.clone(),
            tracking: self.tracking.clone(),
            _state: PhantomData,
        };
        let arc = Arc::new(client);

        // Set the self_arc inside the client to point to this Arc
        arc.self_arc.set(arc.clone()).unwrap();

        // Refresh the internal data
        if refresh {
            match arc.refresh().await {
                Ok(_) => {}
                Err(e) => return Err(e),
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
    ) -> Result<Client<Authenticated>, ApiError> {
        let (_, token) = match self
            .authentication()
            .signin(username, password, device_id)
            .await
        {
            Ok((user, token)) => (user, token),
            Err(e) => return Err(e),
        };

        let new_client = self
            .create_authenticated_client(token, device_id.to_string(), true)
            .await?;
        Ok(new_client)
    }

    /**
     * Creates a new `Client` with the specified token and device ID.
     * # Arguments
     * - `token`: The JWT token to use for authentication
     * - `device_id`: The device ID to use for authentication
     * # Returns
     * A `Result` containing the authenticated `Client` or an `ApiError` if the token is invalid.
     */
    pub async fn login_with_token(
        self,
        token: &str,
        device_id: &str,
    ) -> Result<Client<Authenticated>, ApiError> {
        // Validate the token
        if token.is_empty() || device_id.is_empty() {
            return Err(ApiError::Unknown(
                "Token or device ID cannot be empty".to_string(),
            ));
        }

        let new_client = self
            .create_authenticated_client(token.to_string(), device_id.to_string(), true)
            .await?;
        Ok(new_client)
    }
}

impl Client<Authenticated> {
    /**
     * Creates a new `Client` with the specified parameters.
     * # Arguments
     * - `token`: The JWT token to use for authentication
     * - `device_id`: The device ID to use for authentication
     * # Returns
     * A `Client` with the specified token and device ID.
     */
    pub async fn new_default(
        token: &str,
        device_id: &str,
    ) -> Result<Client<Authenticated>, ApiError> {
        let ua_client = Client::<Unauthenticated>::new();
        match ua_client
            .create_authenticated_client(token.to_string(), device_id.to_string(), false)
            .await
        {
            Ok(client) => Ok(client),
            Err(e) => panic!("Failed to create authenticated client: {}", e),
        }
    }

    /**
     * Returns the current user data
     * # Returns
     * If the user data is successfully fetched.
     */
    pub fn get_user(&self) -> Result<UserPrivate, ApiError> {
        match self.user().get_user() {
            Ok(user) => Ok(user),
            Err(e) => match e {
                ApiError::Unauthorized(e) => return Err(ApiError::InvalidCredentials(e)),
                _ => Err(e),
            },
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
    Set the authentication token
    # Arguments
    - `token`: The token to set for the client
    # Returns
    The client with the token set
    */
    pub fn set_token(&mut self, token: String) {
        self.token = token;
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
    Set the device ID for the client
    # Arguments
    - `device_id`: The device ID to set for the client
    # Returns
    The client with the device ID set
    */
    pub fn set_device_id(&mut self, device_id: &str) {
        self.device_id = device_id.to_string();
    }
    /**
    Returns the current internal data
    # Returns
    If the data is successfully refreshed.
    */
    pub async fn refresh(&self) -> Result<String, ApiError> {
        let user = match self.user().me().await {
            Ok(user) => user,
            Err(e) => return Err(e),
        };
        match self.order().my_orders().await {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        match self.auction().my_auctions().await {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        match user.tier {
            Some(tier) => {
                if tier.is_premium() {
                    self.order().set_order_limit(9999);
                    self.auction().set_auction_limit(9999);
                }
            }
            None => return Err(ApiError::Unknown("User tier not found".to_string())),
        }
        match self.chat().get_chats().await {
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        Ok("Successfully refreshed user data".to_string())
    }
}
