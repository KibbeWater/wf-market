//! Basic usage example for wf-market.
//!
//! This example shows how to use the unauthenticated client for public operations.
//!
//! Run with: `cargo run --example basic_usage`

use wf_market::{Client, OrderType};

#[tokio::main]
async fn main() -> wf_market::Result<()> {
    // Create an unauthenticated client
    let client = Client::builder().build()?;

    println!("=== Fetching all items ===");
    let items = client.fetch_items().await?;
    println!("Found {} tradeable items\n", items.len());

    // Find a specific item by slug
    let nikana = items.iter().find(|i| i.slug == "nikana_prime_set");

    if let Some(item) = nikana {
        println!("=== Item: {} ===", item.slug);
        if let Some(en) = item.i18n.get("en") {
            println!("English name: {}", en.name);
        }
        println!();

        // Get orders for this item
        println!("=== Orders for {} ===", item.slug);
        let orders = client.get_orders(&item.slug).await?;
        println!("Total orders: {}", orders.len());

        // Show top 5 sell orders
        let sell_orders: Vec<_> = orders
            .iter()
            .filter(|o| o.order.order_type == OrderType::Sell)
            .take(5)
            .collect();

        println!("\nTop 5 sell orders:");
        for order in sell_orders {
            println!(
                "  {} - {}p x{} ({})",
                order.user.ingame_name,
                order.order.platinum,
                order.order.quantity,
                order.user.status
            );
        }

        // Get statistics
        println!("\n=== Top Orders (Statistics) ===");
        let top = client.get_top_orders(&item.slug, None).await?;
        println!("Lowest sell: {:?}p", top.sell.first().map(|o| o.platinum));
        println!("Highest buy: {:?}p", top.buy.first().map(|o| o.platinum));
    }

    // Demonstrate caching
    println!("\n=== Using Cache ===");
    let mut cache = wf_market::ApiCache::new();

    // First call fetches from API
    let start = std::time::Instant::now();
    let _items = client.get_items(Some(&mut cache)).await?;
    println!("First fetch: {:?}", start.elapsed());

    // Second call uses cache (instant)
    let start = std::time::Instant::now();
    let _items = client.get_items(Some(&mut cache)).await?;
    println!("Cached fetch: {:?}", start.elapsed());

    println!("\nDone!");
    Ok(())
}
