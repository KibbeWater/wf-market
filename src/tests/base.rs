use crate::{
    client::{Authenticated, Client},
    enums::*,
    errors::AuthError,
    types::{CreateOrderParams, TopOrdersFilters, UpdateOrderParams},
};
use dotenv::dotenv;
use std::env;

// Can Run on any Client
#[tokio::test]
async fn recent() {
    let client = Client::new();
    let versions = client.manifest().versions().await.unwrap();
    println!("Manifests Versions: {:?}", versions);
}
