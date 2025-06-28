use dotenv::dotenv;
use std::env;

use crate::{Authenticated, Client};

#[tokio::test]
async fn recent() {
    dotenv().ok();
    let token =
        env::var("TEST_TOKEN").expect("TEST_TOKEN must be set in .env for integration tests");

    assert!(!token.is_empty());
    let client = Client::<Authenticated>::new_authenticated(token.as_str(), "dev");
    let recent = client.user().me().await.unwrap();
    println!("My Orders: {:?}", recent);
}
