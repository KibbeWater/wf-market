//! HTTP client utilities.

use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};

use crate::models::{Language, Platform};

/// Base URL for the V2 API.
pub const BASE_URL: &str = "https://api.warframe.market/v2";

/// Base URL for the V1 API (authentication).
pub const V1_API_URL: &str = "https://api.warframe.market/v1";

/// Default user agent for API requests.
pub const DEFAULT_USER_AGENT: &str = "wf-market-rs/0.2.0";

/// Build default headers for API requests.
pub fn build_default_headers(platform: Platform, language: Language, crossplay: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));

    // Platform header
    headers.insert(
        "Platform",
        HeaderValue::from_str(platform.as_header_value()).unwrap(),
    );

    // Language header
    headers.insert(
        "Language",
        HeaderValue::from_str(language.as_code()).unwrap(),
    );

    // Crossplay header
    headers.insert(
        "Crossplay",
        HeaderValue::from_str(if crossplay { "true" } else { "false" }).unwrap(),
    );

    headers
}

/// Build an HTTP client without authentication.
pub fn build_http_client(
    platform: Platform,
    language: Language,
    crossplay: bool,
) -> reqwest::Result<reqwest::Client> {
    let headers = build_default_headers(platform, language, crossplay);

    reqwest::Client::builder().default_headers(headers).build()
}

/// Build an HTTP client with bearer token authentication.
pub fn build_authenticated_client(
    platform: Platform,
    language: Language,
    crossplay: bool,
    token: &str,
) -> reqwest::Result<reqwest::Client> {
    let mut headers = build_default_headers(platform, language, crossplay);

    // Add authorization header
    let auth_value = format!("Bearer {}", token);
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_value).expect("Invalid token format"),
    );

    reqwest::Client::builder().default_headers(headers).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_headers() {
        let headers = build_default_headers(Platform::Pc, Language::English, true);

        assert!(headers.contains_key(CONTENT_TYPE));
        assert!(headers.contains_key("Platform"));
        assert!(headers.contains_key("Language"));
        assert!(headers.contains_key("Crossplay"));

        assert_eq!(headers.get("Platform").unwrap(), "pc");
        assert_eq!(headers.get("Language").unwrap(), "en");
        assert_eq!(headers.get("Crossplay").unwrap(), "true");
    }

    #[test]
    fn test_build_client() {
        let client = build_http_client(Platform::Pc, Language::English, true);
        assert!(client.is_ok());
    }

    #[test]
    fn test_build_authenticated_client() {
        let client =
            build_authenticated_client(Platform::Pc, Language::English, true, "test-token");
        assert!(client.is_ok());
    }
}
