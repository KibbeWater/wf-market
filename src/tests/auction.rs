use crate::{
    Authenticated, Client,
    errors::AuthError,
    types::{
        AuctionFilter, AuctionItem, CreateAuctionItem, CreateAuctionParams, ItemAttribute, Riven,
    },
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
                ItemAttribute::new("ammo_maximum", true, 5.0),
                ItemAttribute::new("cold_damage", true, 5.0),
                ItemAttribute::new("critical_chance", false, -55.0),
            ],
            5,
            8,
            5,
            crate::enums::Polarity::Vazarin,
        ),
    );

    let auction = client.auction().create(riven).await.unwrap();
    println!("New Auction Created: {:?}", auction);
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
            true, // having_ephemera
            33,   // damage
        ),
    );

    let auction = client.auction().create(sister).await.unwrap();
    println!("New Auction Created: {:?}", auction);
}
// Create a new auction Riven
// {"starting_price":4,"buyout_price":4,"minimal_reputation":0,"visible":true,"note":"TEST","item":{"weapon_url_name":"cortege","name":"Ampido","type":"riven","attributes":[{"url_name":"ammo_maximum","positive":true,"value":5},{"url_name":"cold_damage","positive":true,"value":5},{"url_name":"critical_chance","positive":false,"value":-55}],"mastery_level":8,"mod_rank":5,"re_rolls":5,"polarity":"vazarin"}}

// Create a new auction Lich
// {"starting_price":12,"buyout_price":null,"minimal_reputation":0,"visible":true,"note":"TEST","item":{"weapon_url_name":"kuva_hek","type":"lich","quirk":"pyromaniac","damage":33,"having_ephemera":true,"element":"magnetic"}}

// Create a new auction Sister
// {"starting_price":1,"buyout_price":1,"minimal_reputation":0,"visible":true,"note":"TEST","item":{"weapon_url_name":"tenet_arca_plasmor","type":"sister","quirk":"bloodhound","damage":59,"having_ephemera":true,"element":"impact"}}
