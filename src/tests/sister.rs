use crate::Client;

#[tokio::test]
async fn all_weapons() {
    let client = Client::new();
    let items = client.sister().get_all_weapons().await.unwrap();
    println!("Total Sister Weapons: {:?}", items.len());
}

#[tokio::test]
async fn get_weapon_by_slug() {
    let client = Client::new();
    let items = client
        .sister()
        .get_weapon_by_slug("tenet_tetra")
        .await
        .unwrap();
    println!("Sister Weapon: {:?}", items);
}

#[tokio::test]
async fn get_all_ephemeras() {
    let client = Client::new();
    let items = client.sister().get_all_ephemeras().await.unwrap();
    println!("Total Sister Ephemeras: {:?}", items.len());
}

#[tokio::test]
async fn get_all_quirks() {
    let client = Client::new();
    let items = client.sister().get_all_quirks().await.unwrap();
    println!("Total Sister Quirks: {:?}", items.len());
}
