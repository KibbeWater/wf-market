//! Update JWT token in .env file using login credentials.
//!
//! Usage:
//!   cargo run --example update_token
//!   cargo run --example update_token -- --email user@example.com --password secret
//!
//! The script will:
//! 1. Read WFM_EMAIL and WFM_PASSWORD from .env (or use CLI args)
//! 2. Authenticate with warframe.market
//! 3. Update WFM_JWT_TOKEN in .env

use std::env;
use std::fs;
use std::path::Path;

use wf_market::{Client, Credentials};

fn load_env() {
    let _ = dotenv::dotenv();
}

fn parse_args() -> (Option<String>, Option<String>) {
    let args: Vec<String> = env::args().collect();
    let mut email = None;
    let mut password = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--email" | "-e" => {
                if i + 1 < args.len() {
                    email = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--password" | "-p" => {
                if i + 1 < args.len() {
                    password = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    (email, password)
}

fn print_help() {
    println!(
        r#"Update JWT token in .env file

Usage:
  cargo run --example update_token [OPTIONS]

Options:
  -e, --email <EMAIL>       Email address (overrides WFM_EMAIL from .env)
  -p, --password <PASSWORD> Password (overrides WFM_PASSWORD from .env)
  -h, --help                Show this help message

Environment variables (from .env):
  WFM_EMAIL                 Email address for login
  WFM_PASSWORD              Password for login
  WFM_DEVICE_ID             Device ID (optional, will generate if missing)

The script will update WFM_JWT_TOKEN in your .env file."#
    );
}

fn update_env_var(lines: &mut Vec<String>, key: &str, value: &str) {
    let prefix = format!("{}=", key);
    let mut found = false;

    for line in lines.iter_mut() {
        if line.starts_with(&prefix) {
            *line = format!("{}={}", key, value);
            found = true;
            break;
        }
    }

    if !found {
        lines.push(format!("{}={}", key, value));
    }
}

fn update_env_file(token: &str, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env_path = Path::new(".env");

    let content = if env_path.exists() {
        fs::read_to_string(env_path)?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    update_env_var(&mut lines, "WFM_JWT_TOKEN", token);
    update_env_var(&mut lines, "WFM_DEVICE_ID", device_id);

    // Ensure file ends with newline
    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }

    fs::write(env_path, output)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    load_env();

    let (arg_email, arg_password) = parse_args();

    // Get credentials from args or environment
    let email = arg_email
        .or_else(|| env::var("WFM_EMAIL").ok())
        .ok_or("Missing email. Set WFM_EMAIL in .env or use --email")?;

    let password = arg_password
        .or_else(|| env::var("WFM_PASSWORD").ok())
        .ok_or("Missing password. Set WFM_PASSWORD in .env or use --password")?;

    let device_id = env::var("WFM_DEVICE_ID").unwrap_or_else(|_| Credentials::generate_device_id());

    println!("Authenticating as {}...", email);

    let credentials = Credentials::new(&email, &password, &device_id);
    let client = Client::from_credentials(credentials).await?;

    let token = client
        .credentials()
        .token()
        .ok_or("No token received from authentication")?;

    println!("Authentication successful!");
    println!("Token: {}...{}", &token[..20], &token[token.len() - 10..]);

    update_env_file(token, &device_id)?;
    println!("Updated WFM_JWT_TOKEN and WFM_DEVICE_ID in .env");

    Ok(())
}
