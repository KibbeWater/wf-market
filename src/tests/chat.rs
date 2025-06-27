use crate::{Authenticated, Client, errors::AuthError};
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
#[tokio::test]
async fn get_chats() {
    let client = setup_client().await.unwrap();
    let items = client.chat().get_chats().await.unwrap();
    println!("Chats: {:?}", items.len());
}

#[tokio::test]
async fn get_chat_messages() {
    let chat_id = "65be253bb0360b0139b65222"; // Replace with a valid chat ID
    let client = setup_client().await.unwrap();
    let items = client.chat().get_chat_messages(chat_id).await.unwrap();
    println!("Chat Messages: {:?}", items.first().unwrap().message);
}
