use crate::{
    Authenticated, Client,
    errors::ApiError,
    types::{
        AuctionFilter, CreateAuctionItem, CreateAuctionParams, ItemAttribute, UpdateAuctionParams,
    },
};
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
async fn get_auctions() {
    let client = Client::new();
    let items = client.auction().get_recent_auctions().await.unwrap();
    // Loop through the items and print them
    println!("Total Auctions: {}", items.len());
    // Print the first item for brevity
    if items.is_empty() {
        println!("No auctions found.");
        return;
    }
    for item in &items {
        println!("Auction: {:?}", item.auction.note_raw);
    }
}

#[tokio::test]
async fn search_auctions() {
    let client = Client::new();
    let items = client
        .auction()
        .search_auctions(
            AuctionFilter::new(crate::enums::AuctionType::Riven, "cortege")
                .with_polarity(crate::enums::Polarity::Madurai),
        )
        .await
        .unwrap();
    // Loop through the items and print them
    println!("Total Auctions: {}", items.len());
    // Print the first item for brevity
    if items.is_empty() {
        println!("No auctions found.");
        return;
    }
    for item in &items {
        println!("Auction: {:?}", item.auction.note_raw);
    }
}
// Can Only Run on Authenticated Client

#[tokio::test]
async fn my_auctions() {
    let client = setup_client().await.unwrap();

    let auctions = client.auction().my_auctions().await.unwrap();
    println!("My Auctions: {:?}", auctions.len());
    for item in &auctions {
        println!("Auction: {:?}", item.note_raw);
    }
}

#[tokio::test]
async fn create_riven() {
    let client = setup_client().await.unwrap();

    let riven = CreateAuctionParams::new(
        10,
        None, // Buyout price
        0,
        true,
        "asd",
        CreateAuctionItem::new_riven(
            "cortege",
            "Ampido",
            vec![
                ItemAttribute::new("ammo_maximum", true, 5000.0),
                ItemAttribute::new("cold_damage", true, 5.0),
                ItemAttribute::new("critical_chance", false, -55.0),
            ],
            5,
            8,
            5,
            crate::enums::Polarity::Vazarin,
        ),
    );
    match client.auction().create(riven).await {
        Ok(auction) => println!("New Auction Created: {:?}", auction),
        Err(e) => {
            eprintln!("Error creating auction: {}", e);
            return;
        }
    }
}

#[tokio::test]
async fn create_lich() {
    let client = setup_client().await.unwrap();

    let lich = CreateAuctionParams::new(
        10,
        None, // Buyout price
        0,
        true,
        "asd",
        CreateAuctionItem::new_lich(
            "kuva_hek",
            "pyromaniac",
            "magnetic",
            true, // having_ephemera
            33,   // damage
        ),
    );

    let auction = client.auction().create(lich).await.unwrap();
    println!("New Auction Created: {:?}", auction);
}

#[tokio::test]
async fn create_sister() {
    let client = setup_client().await.unwrap();

    let sister = CreateAuctionParams::new(
        10,
        None, // Buyout price
        0,
        true,
        "asd",
        CreateAuctionItem::new_sister(
            "tenet_arca_plasmor",
            "bloodhound",
            "impact",
            true,  // having_ephemera
            11111, // damage
        ),
    );

    let auction = client.auction().create(sister).await.unwrap();
    println!("New Auction Created: {:?}", auction);
}

#[tokio::test]
async fn close_auction() {
    let id = "685b24e313559c82fc63b7d9"; // Auction ID to close
    let client = setup_client().await.unwrap();
    let data = client.auction().delete(id).await.unwrap();
    println!("Close order response: {:?}", data);
}

#[tokio::test]
async fn update_auction() {
    let id = "686587c999a6b600s43a39046"; // Auction ID to update
    let client = setup_client().await.unwrap();

    let order = client
        .auction()
        .update(
            id,
            UpdateAuctionParams::new()
                .with_buyout_price(Some(100))
                .with_starting_price(100),
        )
        .await
        .unwrap();
    println!("Auction Updated: {:?}", order);
}
