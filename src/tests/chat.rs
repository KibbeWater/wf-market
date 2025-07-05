use crate::{Authenticated, Client, errors::ApiError};
use dotenv::dotenv;
use std::env;
async fn setup_client() -> Result<Client<Authenticated>, ApiError> {
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
    let chat_id = "65be253abb0360b0139b65222"; // Replace with a valid chat ID
    let client = setup_client().await.unwrap();
    let items = client.chat().get_chat_messages(chat_id).await.unwrap();
    println!("Chat Messages: {:?}", items.first().unwrap().message);
}

#[tokio::test]
async fn leave_chat() {
    let chat_id = "65be253bb0360b0139b65222"; // Replace with a valid chat ID
    let client = setup_client().await.unwrap();
    let response = client.chat().leave_chat(chat_id).await.unwrap();
    println!("Leave Chat Response: {:?}", response);
}

#[tokio::test]
async fn ignore_users() {
    let client = setup_client().await.unwrap();
    let response = client.chat().ignore_users().await.unwrap();
    println!("Ignore Users: {:?}", response.len());
}

#[tokio::test]
async fn ignore_user_add() {
    let chat_id = "65be253bb0360b0139b65222"; // Replace with a valid chat ID
    let user_id = "5e3df7a17b027500c676864f"; // Replace with a valid user ID
    let client = setup_client().await.unwrap();
    let response = client.chat().ignore_user(chat_id, user_id).await.unwrap();
    println!("Ignore User Add Response: {:?}", response);
}

#[tokio::test]
async fn ignore_user_remove() {
    let user_id = "5e3df7a17b027500c676864f"; // Replace with a valid user ID
    let client = setup_client().await.unwrap();
    let response = client.chat().ignore_user_remove(user_id).await.unwrap();
    println!("Ignore User Remove Response: {:?}", response);
}
