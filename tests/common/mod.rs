//! Shared test utilities for integration tests.

use std::env;
use std::fmt;

use wf_market::{Authenticated, Client, Credentials, Error as WfmError};

/// Error type for test authentication failures.
#[derive(Debug)]
pub enum AuthError {
    /// No credentials configured (missing env vars)
    NoCredentials,
    /// JWT token is expired or invalid
    TokenExpired { message: String },
    /// Rate limited by the API
    RateLimited { message: String },
    /// Other authentication error
    AuthFailed { message: String },
    /// Network or connection error
    NetworkError { message: String },
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::NoCredentials => write!(
                f,
                "No credentials configured. Set WFM_JWT_TOKEN or WFM_EMAIL+WFM_PASSWORD in .env"
            ),
            AuthError::TokenExpired { message } => write!(
                f,
                "JWT token expired or invalid. Run `cargo run --example update_token` to refresh.\nDetails: {}",
                message
            ),
            AuthError::RateLimited { message } => write!(
                f,
                "Rate limited by API. Wait 10-15 minutes before retrying.\nDetails: {}",
                message
            ),
            AuthError::AuthFailed { message } => write!(
                f,
                "Authentication failed. Check your credentials.\nDetails: {}",
                message
            ),
            AuthError::NetworkError { message } => {
                write!(
                    f,
                    "Network error during authentication.\nDetails: {}",
                    message
                )
            }
        }
    }
}

impl std::error::Error for AuthError {}

impl From<WfmError> for AuthError {
    fn from(err: WfmError) -> Self {
        let msg = err.to_string();

        // Check for rate limiting (HTTP 429 or Cloudflare 1015)
        if msg.contains("429") || msg.contains("1015") || msg.to_lowercase().contains("rate") {
            return AuthError::RateLimited { message: msg };
        }

        // Check for auth errors (401, 403, invalid token)
        if msg.contains("401")
            || msg.contains("403")
            || msg.to_lowercase().contains("unauthorized")
            || msg.to_lowercase().contains("invalid")
            || msg.to_lowercase().contains("expired")
        {
            return AuthError::TokenExpired { message: msg };
        }

        // Check for network errors
        if msg.to_lowercase().contains("connection")
            || msg.to_lowercase().contains("network")
            || msg.to_lowercase().contains("dns")
            || msg.to_lowercase().contains("timeout")
        {
            return AuthError::NetworkError { message: msg };
        }

        // Generic auth failure
        AuthError::AuthFailed { message: msg }
    }
}

/// Load environment variables from .env file.
///
/// This silently ignores missing .env files, allowing tests to run
/// with environment variables set directly.
pub fn load_env() {
    let _ = dotenv::dotenv();
}

/// Describes how credentials were obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialSource {
    /// Using a JWT token from WFM_JWT_TOKEN
    JwtToken,
    /// Using email/password from WFM_EMAIL + WFM_PASSWORD
    EmailPassword,
}

impl fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialSource::JwtToken => write!(f, "JWT token (WFM_JWT_TOKEN)"),
            CredentialSource::EmailPassword => write!(f, "email/password (WFM_EMAIL+WFM_PASSWORD)"),
        }
    }
}

/// Get test credentials from environment variables.
///
/// Tries WFM_JWT_TOKEN first (to avoid rate limiting), then falls back
/// to WFM_EMAIL + WFM_PASSWORD.
///
/// Returns the credentials and how they were obtained.
#[allow(dead_code)]
pub fn get_credentials() -> Result<(Credentials, CredentialSource), AuthError> {
    load_env();

    let device_id = env::var("WFM_DEVICE_ID").unwrap_or_else(|_| Credentials::generate_device_id());

    // Prefer JWT token to avoid rate limiting on login
    if let Ok(token) = env::var("WFM_JWT_TOKEN") {
        if !token.is_empty() {
            // Email is not used for token auth, but required by struct
            let email = env::var("WFM_EMAIL").unwrap_or_else(|_| "token-auth@local".to_string());
            let creds = Credentials::from_token(email, device_id, token);
            return Ok((creds, CredentialSource::JwtToken));
        }
    }

    // Fall back to email/password
    let email = env::var("WFM_EMAIL").map_err(|_| AuthError::NoCredentials)?;
    let password = env::var("WFM_PASSWORD").map_err(|_| AuthError::NoCredentials)?;

    let creds = Credentials::new(&email, &password, device_id);
    Ok((creds, CredentialSource::EmailPassword))
}

/// Create an authenticated client using credentials from environment.
///
/// Tries JWT token first (instant, no network request), then falls back
/// to email/password login.
///
/// Returns the client and credential source, or an error with details.
pub async fn authenticated_client() -> Result<(Client<Authenticated>, CredentialSource), AuthError>
{
    load_env();

    let device_id = env::var("WFM_DEVICE_ID").unwrap_or_else(|_| Credentials::generate_device_id());

    // Try JWT token first - this is instant and doesn't hit rate limits
    if let Ok(token) = env::var("WFM_JWT_TOKEN") {
        if !token.is_empty() {
            eprintln!("[AUTH] Trying JWT token authentication...");
            let email = env::var("WFM_EMAIL").unwrap_or_else(|_| "token-auth@local".to_string());
            let creds = Credentials::from_token(email, &device_id, token);

            match Client::from_credentials(creds).await {
                Ok(client) => {
                    eprintln!("[AUTH] JWT token authentication successful");
                    return Ok((client, CredentialSource::JwtToken));
                }
                Err(e) => {
                    let auth_err = AuthError::from(e);
                    eprintln!("[AUTH] JWT token failed: {}", auth_err);

                    // If token expired, try email/password
                    if matches!(auth_err, AuthError::TokenExpired { .. }) {
                        eprintln!("[AUTH] Token expired, falling back to email/password...");
                    } else {
                        // For rate limiting or network errors, don't try email/password
                        return Err(auth_err);
                    }
                }
            }
        }
    }

    // Fall back to email/password
    let email = env::var("WFM_EMAIL").map_err(|_| AuthError::NoCredentials)?;
    let password = env::var("WFM_PASSWORD").map_err(|_| AuthError::NoCredentials)?;

    eprintln!("[AUTH] Trying email/password authentication...");
    let creds = Credentials::new(&email, &password, device_id);

    match Client::from_credentials(creds).await {
        Ok(client) => {
            eprintln!("[AUTH] Email/password authentication successful");
            Ok((client, CredentialSource::EmailPassword))
        }
        Err(e) => Err(AuthError::from(e)),
    }
}

/// Check if integration test credentials are available.
pub fn has_credentials() -> bool {
    load_env();

    // JWT token is sufficient
    if let Ok(token) = env::var("WFM_JWT_TOKEN") {
        if !token.is_empty() {
            return true;
        }
    }

    // Or email + password
    env::var("WFM_EMAIL").is_ok() && env::var("WFM_PASSWORD").is_ok()
}

/// Print a message when skipping a test due to missing credentials.
pub fn skip_message() -> &'static str {
    "Skipping: No credentials available. Set WFM_JWT_TOKEN or WFM_EMAIL+WFM_PASSWORD in .env. \
     Run `cargo run --example update_token` to generate a token."
}

/// Helper macro to handle authentication in tests with proper error messages.
#[macro_export]
macro_rules! require_auth {
    () => {{
        if !common::has_credentials() {
            eprintln!("{}", common::skip_message());
            return;
        }

        match common::authenticated_client().await {
            Ok((client, source)) => {
                eprintln!("[TEST] Authenticated via {}", source);
                client
            }
            Err(e) => {
                panic!("Authentication failed:\n{}", e);
            }
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_env_does_not_panic() {
        // Should not panic even if .env doesn't exist
        load_env();
    }

    #[test]
    fn test_has_credentials_returns_bool() {
        // Should return true or false, not panic
        let _ = has_credentials();
    }

    #[test]
    fn test_auth_error_display() {
        let err = AuthError::TokenExpired {
            message: "401 Unauthorized".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("expired"));
        assert!(msg.contains("update_token"));

        let err = AuthError::RateLimited {
            message: "429 Too Many Requests".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Rate limited"));
        assert!(msg.contains("Wait"));
    }
}
