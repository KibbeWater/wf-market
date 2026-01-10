//! Session persistence example.
//!
//! This example demonstrates how to save and restore login sessions
//! to avoid re-authenticating on every run.
//!
//! Set the following environment variables:
//! - WFM_EMAIL: Your warframe.market email
//! - WFM_PASSWORD: Your warframe.market password
//!
//! Run with: `cargo run --example session_persistence`

use std::env;
use std::path::Path;

use wf_market::{Client, Credentials, Error};

const SESSION_FILE: &str = "wfm_session.json";

#[tokio::main]
async fn main() -> wf_market::Result<()> {
    // Try to load existing session
    let client = if Path::new(SESSION_FILE).exists() {
        println!("=== Found existing session ===");
        match load_session().await {
            Ok(client) => {
                println!("Session restored successfully!");
                client
            }
            Err(e) => {
                println!("Session invalid or expired: {}", e);
                println!("Logging in with credentials...\n");
                login_fresh().await?
            }
        }
    } else {
        println!("=== No existing session ===");
        login_fresh().await?
    };

    // Use the client
    println!("\n=== Session Info ===");
    let creds = client.credentials();
    println!("Email: {}", creds.email);
    println!("Device ID: {}", creds.device_id);

    // Save the session for next time
    save_session(&client)?;
    println!("\nSession saved to {}", SESSION_FILE);

    // Demonstrate session export format
    println!("\n=== Session Data Structure ===");
    let session = client.export_session();
    println!("Email: {}", session.email);
    println!("Has token: {}", session.token().is_some());
    println!("Device ID: {}", session.device_id);

    // The session can be serialized to any serde-compatible format
    println!("\n=== Serialization Options ===");
    println!("JSON: serde_json::to_string(&session)?");
    println!("TOML: toml::to_string(&session)?");
    println!("Binary: bincode::serialize(&session)?");

    println!("\nDone!");
    Ok(())
}

async fn load_session() -> wf_market::Result<Client<wf_market::Authenticated>> {
    let json = std::fs::read_to_string(SESSION_FILE)
        .map_err(|e| Error::InvalidConfig(format!("Failed to read session file: {}", e)))?;

    let creds: Credentials = serde_json::from_str(&json)
        .map_err(|e| Error::InvalidConfig(format!("Failed to parse session: {}", e)))?;

    // Validate the session is still valid
    println!("Validating session...");
    if !Client::validate_credentials(&creds).await? {
        return Err(Error::InvalidConfig(
            "Session expired or invalid".to_string(),
        ));
    }

    // Session is valid, create client
    Client::from_credentials(creds).await
}

async fn login_fresh() -> wf_market::Result<Client<wf_market::Authenticated>> {
    let email = env::var("WFM_EMAIL").expect("WFM_EMAIL not set");
    let password = env::var("WFM_PASSWORD").expect("WFM_PASSWORD not set");

    println!("Logging in as {}...", email);

    let creds = Credentials::new(&email, &password, Credentials::generate_device_id());
    Client::from_credentials(creds).await
}

fn save_session(client: &Client<wf_market::Authenticated>) -> wf_market::Result<()> {
    let session = client.export_session();
    let json = serde_json::to_string_pretty(&session)
        .map_err(|e| Error::InvalidConfig(format!("Failed to serialize session: {}", e)))?;

    std::fs::write(SESSION_FILE, json)
        .map_err(|e| Error::InvalidConfig(format!("Failed to write session file: {}", e)))?;

    Ok(())
}
