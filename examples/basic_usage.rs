//! Basic usage example for wf-market.
//!
//! This example shows how to use the unauthenticated client for public operations.
//!
//! Run with: `cargo run --example basic_usage`

use wf_market::{Client, OrderType};

#[tokio::main]
async fn main() -> wf_market::Result<()> {
    // Create an unauthenticated client (fetches items automatically)
    let client = Client::builder().build().await?;

    println!("=== Loaded {} items ===\n", client.items().len());

    // Find a specific item by slug (O(1) lookup)
    let nikana = client.get_item_by_slug("nikana_prime_set");

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
            // Item info is automatically available via get_item()
            let item_type = order
                .get_item()
                .map(|i| {
                    if i.is_mod() {
                        "mod"
                    } else if i.is_sculpture() {
                        "sculpture"
                    } else {
                        "item"
                    }
                })
                .unwrap_or("unknown");

            println!(
                "  {} - {}p x{} ({}) [{}]",
                order.user.ingame_name,
                order.order.platinum,
                order.order.quantity,
                order.user.status,
                item_type
            );
        }

        // Get statistics
        println!("\n=== Top Orders (Statistics) ===");
        let top = client.get_top_orders(&item.slug, None).await?;
        println!("Lowest sell: {:?}p", top.sell.first().map(|o| o.platinum));
        println!("Highest buy: {:?}p", top.buy.first().map(|o| o.platinum));
    }

    // Demonstrate caching (items are already cached in client)
    println!("\n=== Using Cache ===");
    println!("Items are automatically cached in the client.");
    println!("Use build_with_cache() to persist across restarts:");
    println!("  let mut cache = ApiCache::new();");
    println!("  let client = Client::builder().build_with_cache(&mut cache).await?;");
    println!("  // Cache is used if items are < 1 day old");

    println!("\nDone!");
    Ok(())
}
