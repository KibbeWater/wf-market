use crate::{
    client::{Authenticated, Client},
    enums::*,
    errors::AuthError,
    types::{CreateOrderParams, TopOrdersFilters, UpdateOrderParams},
};
use dotenv::dotenv;
use std::env;

async fn setup_client() -> Result<Client<Authenticated>, AuthError> {
    dotenv().ok();

    let user = env::var("TEST_USER").expect("TEST_USER must be set in .env for integration tests");
    let pass = env::var("TEST_PASS").expect("TEST_PASS must be set in .env for integration tests");

    assert!(!user.is_empty());
    assert!(!pass.is_empty());

    let _client = Client::new();
    _client.login(&user, &pass, "dev").await
}

// Can Run on any Client
#[tokio::test]
async fn get_by_slug() {
    let slug = ""; // User slug to fetch
    let client = Client::new();
    let user = client.user().get_by_slug(slug).await.unwrap();
    println!("User by slug: {:?}", user);
}
#[tokio::test]
async fn get_by_id() {
    let id = ""; // User ID to fetch
    let client = Client::new();
    let user = client.user().get_by_slug(id).await.unwrap();
    println!("User by id: {:?}", user);
}

// Can Run on Authenticated Client