use std::num::NonZeroU32;

use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};

/**
INTERNAL: Build the HTTP client with default settings

# Arguments
- `auth`: Authentication token used when communicating with authenticated endpoints

# Returns
- A `reqwest::Client` with assigned default headers
*/
pub(super) fn build_http(auth: Option<String>) -> reqwest::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    if let Some(auth) = auth {
        headers.insert(reqwest::header::AUTHORIZATION, auth.parse().unwrap());
    }
    headers.insert("language", "en".parse().unwrap());
    headers.insert("platform", "pc".parse().unwrap());

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
}

/**
INTERNAL: Build the rate limiter for throttling outgoing requests to max allowed speeds
*/
pub(super) fn build_limiter(rps: NonZeroU32) -> RateLimiter<NotKeyed, InMemoryState, DefaultClock> {
    RateLimiter::direct(Quota::per_second(rps))
}
