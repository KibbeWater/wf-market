//! Shared test utilities for integration tests.

use std::env;

use wf_market::{Authenticated, Client, Credentials};

/// Load environment variables from .env file.
///
/// This silently ignores missing .env files, allowing tests to run
/// with environment variables set directly.
pub fn load_env() {
    let _ = dotenv::dotenv();
}

/// Get test credentials from environment variables.
///
/// Returns `None` if WFM_EMAIL or WFM_PASSWORD are not set.
pub fn get_credentials() -> Option<Credentials> {
    load_env();

    let email = env::var("WFM_EMAIL").ok()?;
    let password = env::var("WFM_PASSWORD").ok()?;

    // Use a stable device ID for testing (or generate one)
    let device_id = env::var("WFM_DEVICE_ID").unwrap_or_else(|_| Credentials::generate_device_id());

    Some(Credentials::new(&email, &password, device_id))
}

/// Create an authenticated client using credentials from environment.
///
/// Returns `None` if credentials are not available or authentication fails.
pub async fn authenticated_client() -> Option<Client<Authenticated>> {
    let creds = get_credentials()?;
    Client::from_credentials(creds).await.ok()
}

/// Check if integration test credentials are available.
pub fn has_credentials() -> bool {
    load_env();
    env::var("WFM_EMAIL").is_ok() && env::var("WFM_PASSWORD").is_ok()
}

/// Print a message when skipping a test due to missing credentials.
pub fn skip_message() -> &'static str {
    "Skipping: WFM_EMAIL and WFM_PASSWORD not set. \
     Copy .env.example to .env and fill in credentials to run this test."
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
}
