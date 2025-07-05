use dotenv::dotenv;
use std::env;

use crate::Client;

#[tokio::test]
async fn print_token() {
    dotenv().ok();

    let user = env::var("TEST_USER").expect("TEST_USER must be set in .env for integration tests");
    let pass = env::var("TEST_PASS").expect("TEST_PASS must be set in .env for integration tests");

    assert!(!user.is_empty());
    assert!(!pass.is_empty());

    let client = Client::new();
    let new_client = client.login(&user, &pass, "dev").await.unwrap();
    println!("Token: {}", new_client.get_token());
}

#[tokio::test]
async fn login_with_token() {
    dotenv().ok();
    let token =
        env::var("TEST_TOKEN").expect("TEST_TOKEN must be set in .env for integration tests");

    assert!(!token.is_empty());

    let client = Client::new()
        .login_with_token(&token, "default")
        .await
        .unwrap();
    let recent = client.user().me().await.unwrap();
    println!("My Orders: {:?}", recent);
}
