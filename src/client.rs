use crate::{endpoints::*, enums::*, errors::*, utils::*};
use governor::{
    RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use reqwest::{Method, header::HeaderMap};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    marker::PhantomData,
    num::{NonZero, NonZeroU32},
    sync::{Arc, Mutex, OnceLock, Weak},
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

impl<State: Clone + 'static> Client<State> {
    fn arc(&self) -> Arc<Self> {
        self.self_arc
            .get_or_init(|| {
                Arc::new(Self {
                    http: self.http.clone(),
                    self_arc: OnceLock::new(),
                    token: self.token.clone(),
                    device_id: self.device_id.clone(),
                    platform: self.platform,
                    language: self.language,
                    crossplay: self.crossplay,
                    order_route: self.order_route.clone(),
                    user_route: self.user_route.clone(),
                    authentication_route: self.authentication_route.clone(),
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
        let url = version.as_str().to_owned() + path;
        let mut headers = reqwest::header::HeaderMap::new();

        // Add the required headers
        headers.insert("language", self.language.as_str().parse().unwrap());
        headers.insert("platform", self.platform.as_str().parse().unwrap());
        headers.insert("crossplay", self.crossplay.to_string().parse().unwrap());
        
        // If the client is authenticated, add the token to the headers
        if self.token != "" {
            headers.insert(reqwest::header::AUTHORIZATION, format!("Bearer {}", self.token));
        }

        // Add any additional headers provided
       if let Some(hards) = headers {
            for (key, value) in hards.iter() {
                headers.insert(key.parse().unwrap(), value.parse().unwrap());
            }
        }

        // Create the HTTP client with the headers
        let http_client =reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

        let mut builder = http_client.request(method, &url);
        // If the client needs a body, serialize it
        if let Some(b) = body {
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
                                        "Error Parsing Bad Request Error: {:?}, Message: {}",
                                        e, body
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
                    Err(err) => Err(ApiError::ParsingError(
                        format!("Error Parsing: {:?}, Body: {}", err, body).to_string(),
                    )),
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
}

#[derive(Debug, Clone)]
pub struct Client<State = Unauthenticated> {
    http: reqwest::Client,
    self_arc: OnceLock<Arc<Client<State>>>,
    token: String,
    device_id: String,
    language: Language,
    platform: Platform,
    crossplay: bool,
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    order_route: OnceLock<Arc<OrderRoute<State>>>,
    user_route: OnceLock<Arc<UserRoute<State>>>,
    authentication_route: OnceLock<Arc<AuthenticationRoute<State>>>,
    _state: PhantomData<State>,
}

impl Client<Unauthenticated> {
    pub fn new() -> Self {
        Self {
            http: build_http(None),
            self_arc: OnceLock::new(),
            token: String::new(),
            device_id: String::new(),
            language: Language::default(),
            platform: Platform::default(),
            crossplay: true,
            order_route: OnceLock::new(),
            user_route: OnceLock::new(),
            authentication_route: OnceLock::new(),
            limiter: build_limiter(REQUESTS_PER_SECOND).into(),
            _state: PhantomData,
        }
    }

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
            http: build_http(Some(format!("Bearer {}", token))),
            self_arc: OnceLock::new(),
            token,
            device_id: device_id.to_string(),
            platform: self.platform,
            language: self.language,
            crossplay: self.crossplay,
            order_route: OnceLock::new(),
            user_route: OnceLock::new(),
            authentication_route: OnceLock::new(),
            limiter: self.limiter.clone(),
            _state: PhantomData,
        };

        let arc = Arc::new(new_client);

        // Set the self_arc inside the client to point to this Arc
        arc.self_arc.set(arc.clone()).unwrap();

        // Copy routes if they were initialized
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

        Ok(Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone()))
    }
}

impl Client<Authenticated> {
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
    Returns the clients device id

    # Returns
    The Device ID used when authenticating
    */
    pub fn get_device_id(&self) -> String {
        // Again, panics, cosmic particle, you get the gist of it now
        self.device_id.clone()
    }
}
