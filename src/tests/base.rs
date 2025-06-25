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
async fn versions() {
    let client = Client::new();
    let versions = client.manifest().versions().await.unwrap();
    println!("Manifests Versions: {:?}", versions);
}

#[tokio::test]
async fn locations() {
    let client = Client::new();
    let locations = client.manifest().locations().await.unwrap();
    println!("Locations: {:?}", locations.len());
}
#[tokio::test]
async fn npcs() {
    let client = Client::new();
    let npcs = client.manifest().npcs().await.unwrap();
    println!("NPCs: {:?}", npcs.len());
}
#[tokio::test]
async fn missions() {
    let client = Client::new();
    let missions = client.manifest().missions().await.unwrap();
    println!("Missions: {:?}", missions.len());
}
