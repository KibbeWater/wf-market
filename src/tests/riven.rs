use crate::Client;

#[tokio::test]
async fn all_rivens() {
    let client = Client::new();
    let items = client.riven().get_all_rivens().await.unwrap();
    println!("Total Riven: {:?}", items.len());
}

#[tokio::test]
async fn get_riven_by_slug() {
    let client = Client::new();
    let items = client
        .riven()
        .get_riven_by_slug("kulstar")
        .await
        .unwrap();
    println!("Riven: {:?}", items);
}

#[tokio::test]
async fn get_all_attributes() {
    let client = Client::new();
    let items = client.riven().get_all_attributes().await.unwrap();
    println!("Total Riven Attributes: {:?}", items.len());
}
