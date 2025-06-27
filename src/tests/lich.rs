use crate::Client;

#[tokio::test]
async fn all_weapons() {
    let client = Client::new();
    let items = client.lich().get_all_weapons().await.unwrap();
    println!("Total Lich Weapons: {:?}", items.len());
}

#[tokio::test]
async fn get_weapon_by_slug() {
    let client = Client::new();
    let items = client
        .lich()
        .get_weapon_by_slug("kuva_drakgoon")
        .await
        .unwrap();
    println!("Lich Weapon: {:?}", items);
}

#[tokio::test]
async fn get_all_ephemeras() {
    let client = Client::new();
    let items = client.lich().get_all_ephemeras().await.unwrap();
    println!("Total Lich Ephemeras: {:?}", items.len());
}

#[tokio::test]
async fn get_all_quirks() {
    let client = Client::new();
    let items = client.lich().get_all_quirks().await.unwrap();
    println!("Total Lich Quirks: {:?}", items.len());
}
