use crate::{
    client::{Authenticated, Client},
    enums::*,
    errors::AuthError,
    types::{CreateOrderParams, UpdateOrderParams},
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

#[tokio::test]
async fn test_my_orders() {
    let client = setup_client().await.unwrap();

    client.order().my_orders().await.unwrap();
}

#[tokio::test]
async fn create_regular_order() {
    let id = "54aae292e7798909064f1575"; // Secura Dual Cestra Item ID
    let client = setup_client().await.unwrap();

    client
        .order()
        .create(CreateOrderParams::new(id, OrderType::Buy, 10, 1, true))
        .await
        .unwrap();
    println!("Orders: {:?}", client.order().orders());
}

#[tokio::test]
async fn create_mod_order() {
    let id = "5bc1ab93b919f200c18c10ef"; // Adaptation Max Rank 10
    let client = setup_client().await.unwrap();

    client
        .order()
        .create(CreateOrderParams::new(id, OrderType::Sell, 98, 5, true).with_rank(10))
        .await
        .unwrap();
    println!("Orders: {:?}", client.order().orders());
}

#[tokio::test]
async fn update_order() {
    let id = "685b118413559c82fc63b7d8"; // Order ID to update
    let client = setup_client().await.unwrap();

    client
        .order()
        .update(id, UpdateOrderParams::new().with_platinum(999))
        .await
        .unwrap();
    println!("Orders: {:?}", client.order().orders());
}

#[tokio::test]
async fn close_order() {
    let id = "685b147e914e45bf7792c7b7"; // Order ID to close
    let client = setup_client().await.unwrap();
    client.order().my_orders().await.unwrap();
    let rep = client.order().close(id, 2).await.unwrap();
    println!("Close order response: {:?}", rep);
    println!("Orders: {:?}", client.order().orders());
}

// #[tokio::test]
// async fn delete_order() {
//     let id = "685676bfdc98ddb0b2d2519d"; // Order ID to delete
//     let client = setup_client().await.unwrap();
//     let rep = client.delete_order(id).await.unwrap();
//     println!("Delete order response: {:?}", rep.object);
// }

mod order;
