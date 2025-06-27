use crate::client::Client;

#[tokio::test]
async fn all_items() {
    let client = Client::new();
    let items = client.items().get_all().await.unwrap();
    println!("Items: {:?}", items.get(0).unwrap().slug);
}

#[tokio::test]
async fn get_by_slug() {
    let client = Client::new();
    let items = client
        .items()
        .get_by_slug("secura_dual_cestra")
        .await
        .unwrap();
    println!("Items: {:?}", items);
}
