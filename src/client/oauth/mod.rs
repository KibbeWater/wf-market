/*!
Provides the [`OAuthClient`][crate::client::oauth::OAuthClient] struct to obtain OAuth access tokens

# Examples

Using the local server helper:
```rust
use tokio::task;

// NOTE: Requires the "server" feature to use `server::start_listener_server`
use wf_market::client::oauth::{
    server::start_listener_server,
    ChallengeMethod, 
    OAuth2Client
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oauth = OAuth2Client::new(
        "client_id",
        "redirect_uri",
        ChallengeMethod::S256,
        vec![""]
    );
    
    let listener = task::spawn(async move {
        start_listener_server(4321)
    });
    
    println!("Authorize: {}",
             oauth.create_auth_url());

    let code = listener.await.unwrap()
        .map_err(|e| e as Box<dyn std::error::Error>)?;
    
    match oauth.exchange_code(code).await { 
        Ok(access_token) => println!("Your access token: {}", access_token),
        Err(err) => println!("Error: {}", err),
    }
    
    Ok(())
}
```
*/

use std::collections::HashMap;
use rand::Rng;
use sha2::{Digest, Sha256};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use url::{Url};

pub mod credentials;
#[cfg(feature = "server")]
pub mod server;

const AUTH_URL: &str = "https://discord.com/oauth2/authorize";
const TOKEN_URL: &str = "https://discord.com/api/oauth2/token";

#[derive(Copy, Clone)]
pub enum ChallengeMethod {
    S256,
    Plain
}

impl std::fmt::Display for ChallengeMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChallengeMethod::S256 => write!(f, "S256"),
            ChallengeMethod::Plain => write!(f, "plain"),
        }
    }
}

/*
    TODO: Untested on actual API, a couple of assumptions are made and features missing due to those assumptions
    1. Only return an access_token on code exchanges, as it's the only one required by the OAuth spec
    2. There are no deviations from the OAuth spec, and therefore errors are not implemented nor tested
 */
/**
OAuth Client used to exchange access tokens with the server using OAuth PKCE
*/
pub struct OAuth2Client {
    // Configurable stuffz
    client_id: String,
    redirect_uri: String,
    method: ChallengeMethod,
    scopes: Vec<String>,
    
    // Priv stuffz
    code_verifier: String,
}

impl OAuth2Client {
    pub fn new(client_id: &str, redirect_uri: &str, challenge_method: ChallengeMethod, scopes: Vec<&str>) -> Self {
        OAuth2Client {
            client_id: client_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            method: challenge_method,
            code_verifier: OAuth2Client::generate_code_verifier(128),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
        }
    }
    
    pub fn create_auth_url(&self) -> String {
        let mut url = Url::parse(AUTH_URL).unwrap();
        
        {
            let mut query_params = url.query_pairs_mut();
            
            query_params.append_pair("response_type", "code");
            query_params.append_pair("client_id", &self.client_id);
            query_params.append_pair("redirect_uri", &self.redirect_uri);
            query_params.append_pair("code_challenge", &self.generate_code_challenge());
            query_params.append_pair("code_challenge_method", &self.method.to_string());
            query_params.append_pair("scope", &self.scopes.join(" "));
        }
        
        url.to_string()
    }
    
    pub async fn exchange_code(&self, code: String) -> Result<String, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        // Your form data as key-value pairs
        let mut params = HashMap::new();
        params.insert("grant_type", "authorization_code");
        params.insert("code", code.as_str());
        params.insert("redirect_uri", &self.redirect_uri);
        params.insert("client_id", &self.client_id);
        params.insert("code_verifier", &self.code_verifier);

        let res = client
            .post(TOKEN_URL) // replace with TOKEN_URL
            .form(&params) // This sets the content type to application/x-www-form-urlencoded
            .send()
            .await?;

        let body = res.text().await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        
        let json: serde_json::Value = serde_json::from_str(&body)?;
        
        Ok(json["access_token"].as_str().unwrap_or(body.as_str()).to_string())
    }
    
    fn generate_code_challenge(&self) -> String {
        match self.method {
            ChallengeMethod::S256 => {
                let digest = Sha256::digest(self.code_verifier.as_bytes());
                URL_SAFE_NO_PAD.encode(digest)
            }
            ChallengeMethod::Plain => {
                self.code_verifier.to_string()
            }
        }
    }

    fn generate_code_verifier(len: usize) -> String {
        assert!(
            (43..=128).contains(&len),
            "code_verifier length must be between 43 and 128 characters"
        );

        let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                    abcdefghijklmnopqrstuvwxyz\
                    0123456789-._~";

        let mut rng = rand::rng();

        (0..len)
            .map(|_| {
                let idx = rng.random_range(0..charset.len());
                charset[idx] as char
            })
            .collect()
    }
}
