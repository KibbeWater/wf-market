//! Authentication credentials for warframe.market.
//!
//! This module provides the [`Credentials`] type which supports both
//! fresh login (email/password) and session restoration (token-based).
//!
//! # Example
//!
//! ```ignore
//! use wf_market::Credentials;
//!
//! // Fresh login
//! let creds = Credentials::new(
//!     "user@example.com",
//!     "password",
//!     Credentials::generate_device_id(),
//! );
//!
//! // Token-based (restored session)
//! let creds = Credentials::from_token(
//!     "user@example.com",
//!     "device-id-here",
//!     "jwt-token-here",
//! );
//! ```

use serde::{Deserialize, Serialize};

/// Authentication credentials for the warframe.market API.
///
/// Supports both fresh login (email/password) and session restoration
/// (token-based). This type is serde-compatible for easy persistence.
///
/// # Session Persistence
///
/// After a successful login, you can save the credentials (which now
/// contain the JWT token) and restore the session later without
/// re-entering the password:
///
/// ```ignore
/// use wf_market::{Client, Credentials};
///
/// async fn example() -> wf_market::Result<()> {
///     // Initial login
///     let creds = Credentials::new("user@example.com", "password", Credentials::generate_device_id());
///     let client = Client::from_credentials(creds).await?;
///
///     // Save session for later
///     let session = client.credentials().clone();
///     let json = serde_json::to_string(&session)?;
///     std::fs::write("session.json", &json)?;
///
///     // Later: restore session
///     let saved: Credentials = serde_json::from_str(&std::fs::read_to_string("session.json")?)?;
///     let client = Client::from_credentials(saved).await?;
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// User's email address or username
    pub email: String,

    /// Unique device identifier (should be consistent across sessions)
    pub device_id: String,

    /// User's password (only used for fresh login, cleared after successful auth)
    #[serde(skip_serializing_if = "Option::is_none")]
    password: Option<String>,

    /// JWT authentication token (set after successful login)
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
}

impl Credentials {
    /// Create credentials for a fresh login.
    ///
    /// # Arguments
    ///
    /// * `email` - User's email address or username
    /// * `password` - User's password
    /// * `device_id` - Unique device identifier (use [`generate_device_id`](Self::generate_device_id) for a new one)
    ///
    /// # Example
    ///
    /// ```
    /// use wf_market::Credentials;
    ///
    /// let creds = Credentials::new(
    ///     "user@example.com",
    ///     "password123",
    ///     Credentials::generate_device_id(),
    /// );
    /// ```
    pub fn new(
        email: impl Into<String>,
        password: impl Into<String>,
        device_id: impl Into<String>,
    ) -> Self {
        Self {
            email: email.into(),
            device_id: device_id.into(),
            password: Some(password.into()),
            token: None,
        }
    }

    /// Create credentials from a saved session token.
    ///
    /// Use this when restoring a previously authenticated session.
    /// The token will be validated when used with the client.
    ///
    /// # Arguments
    ///
    /// * `email` - User's email address (for reference)
    /// * `device_id` - The same device ID used during original login
    /// * `token` - JWT token from a previous session
    ///
    /// # Example
    ///
    /// ```
    /// use wf_market::Credentials;
    ///
    /// let creds = Credentials::from_token(
    ///     "user@example.com",
    ///     "550e8400-e29b-41d4-a716-446655440000",
    ///     "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    /// );
    /// ```
    pub fn from_token(
        email: impl Into<String>,
        device_id: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            email: email.into(),
            device_id: device_id.into(),
            password: None,
            token: Some(token.into()),
        }
    }

    /// Generate a new random device ID (UUID v4).
    ///
    /// Device IDs should be generated once and reused for all sessions
    /// on the same device. Store this value alongside your credentials.
    ///
    /// # Example
    ///
    /// ```
    /// use wf_market::Credentials;
    ///
    /// let device_id = Credentials::generate_device_id();
    /// println!("Generated device ID: {}", device_id);
    /// ```
    pub fn generate_device_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Get the authentication token if present.
    ///
    /// Returns `Some` if the credentials contain a token (either from
    /// [`from_token`](Self::from_token) or set after successful login).
    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    /// Check if these credentials use token-based authentication.
    ///
    /// Returns `true` if the credentials have a token and no password,
    /// indicating they're for session restoration rather than fresh login.
    pub fn is_token_based(&self) -> bool {
        self.token.is_some() && self.password.is_none()
    }

    /// Check if these credentials have a password for fresh login.
    pub fn has_password(&self) -> bool {
        self.password.is_some()
    }

    /// Get the password if present (for internal use during login).
    pub(crate) fn password(&self) -> Option<&str> {
        self.password.as_deref()
    }

    /// Set the token and clear the password (called after successful login).
    pub(crate) fn set_token(&mut self, token: String) {
        self.token = Some(token);
        self.password = None; // Clear password after successful auth
    }

    /// Clear the token (e.g., on logout or token expiration).
    #[allow(dead_code)]
    pub(crate) fn clear_token(&mut self) {
        self.token = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_credentials() {
        let creds = Credentials::new("test@example.com", "password123", "device-123");

        assert_eq!(creds.email, "test@example.com");
        assert_eq!(creds.device_id, "device-123");
        assert!(creds.has_password());
        assert!(!creds.is_token_based());
        assert!(creds.token().is_none());
    }

    #[test]
    fn test_token_credentials() {
        let creds = Credentials::from_token("test@example.com", "device-123", "jwt-token");

        assert_eq!(creds.email, "test@example.com");
        assert_eq!(creds.device_id, "device-123");
        assert!(!creds.has_password());
        assert!(creds.is_token_based());
        assert_eq!(creds.token(), Some("jwt-token"));
    }

    #[test]
    fn test_set_token_clears_password() {
        let mut creds = Credentials::new("test@example.com", "password123", "device-123");
        assert!(creds.has_password());

        creds.set_token("new-token".to_string());

        assert!(!creds.has_password());
        assert_eq!(creds.token(), Some("new-token"));
        assert!(creds.is_token_based());
    }

    #[test]
    fn test_generate_device_id_is_unique() {
        let id1 = Credentials::generate_device_id();
        let id2 = Credentials::generate_device_id();

        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID format
    }

    #[test]
    fn test_serialization_without_password() {
        let mut creds = Credentials::new("test@example.com", "password123", "device-123");
        creds.set_token("jwt-token".to_string());

        let json = serde_json::to_string(&creds).unwrap();

        // Password should not be in serialized output
        assert!(!json.contains("password"));
        assert!(json.contains("jwt-token"));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "email": "test@example.com",
            "device_id": "device-123",
            "token": "jwt-token"
        }"#;

        let creds: Credentials = serde_json::from_str(json).unwrap();

        assert_eq!(creds.email, "test@example.com");
        assert_eq!(creds.device_id, "device-123");
        assert_eq!(creds.token(), Some("jwt-token"));
        assert!(!creds.has_password());
    }
}
