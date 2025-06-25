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
async fn recent() {
    let client = Client::new();
    let recent = client.order().recent().await.unwrap();
    println!("Recent Orders: {:?}", recent.len());
}

#[tokio::test]
async fn get_orders_by_item() {
    let client = Client::new();
    let slug = "primed_target_cracker"; // Item slug to fetch orders for
    let orders = client.order().get_orders_by_item(slug).await.unwrap();
    println!("Orders for {}: {:?}", slug, orders.len());
}

#[tokio::test]
async fn get_top_orders_by_item() {
    let client = Client::new();
    let slug = "primed_target_cracker"; // Item slug to fetch orders for
    let orders = client
        .order()
        .get_top_orders_by_item(slug, Some(TopOrdersFilters::new()))
        .await
        .unwrap();
    println!(
        "Top Orders for {}: Buy: {:?}, Sell: {:?}",
        slug,
        orders.buy.len(),
        orders.sell.len()
    );
}

#[tokio::test]
async fn get_by_id() {
    let id = "6859657e57605a002b649eee"; // Order ID to fetch
    let client = Client::new();

    let order = client.order().get_by_id(id).await.unwrap();
    println!("Order: {:?}", order);
}

// Can Only Run on Authenticated Client
#[tokio::test]
async fn my_orders() {
    let client = setup_client().await.unwrap();

    let orders = client.order().my_orders().await.unwrap();
    println!("My Orders: {:?}", orders);
}

#[tokio::test]
async fn update_order() {
    let id = "685b118413559c82fc63b7d8"; // Order ID to update
    let client = setup_client().await.unwrap();

    let order = client
        .order()
        .update(id, UpdateOrderParams::new().with_platinum(999))
        .await
        .unwrap();
    println!("Order Updated: {:?}", order);
    println!("Orders: {:?}", client.order().orders());
}

#[tokio::test]
async fn create_regular_order() {
    let id = "54aae292e7798909064f1575"; // Secura Dual Cestra Item ID
    let client = setup_client().await.unwrap();

    let new_order = client
        .order()
        .create(CreateOrderParams::new(id, OrderType::Buy, 10, 1, true))
        .await
        .unwrap();
    println!("New Order Created: {:?}", new_order);
}

#[tokio::test]
async fn close_order() {
    let id = "685b24e313559c82fc63b7d9"; // Order ID to close
    let client = setup_client().await.unwrap();
    client.order().my_orders().await.unwrap();
    let rep = client.order().close(id, 2).await.unwrap();
    println!("Close order response: {:?}", rep);
    println!("Orders: {:?}", client.order().orders());
}

#[tokio::test]
async fn delete_order() {
    let id = "685b24fb914e45bf7792c7b9"; // Order ID to delete
    let client = setup_client().await.unwrap();
    let rep = client.order().delete(id).await.unwrap();
    println!("Delete order response: {:?}", rep);
}
