//! Example of creating and managing orders.
//!
//! This example requires authentication. Set the following environment variables:
//! - WFM_EMAIL: Your warframe.market email
//! - WFM_PASSWORD: Your warframe.market password
//!
//! Run with: `cargo run --example create_orders`

use std::env;

use wf_market::{Client, Credentials};

#[tokio::main]
async fn main() -> wf_market::Result<()> {
    // Load credentials from environment
    let email = env::var("WFM_EMAIL").expect("WFM_EMAIL not set");
    let password = env::var("WFM_PASSWORD").expect("WFM_PASSWORD not set");

    println!("=== Logging in ===");
    let creds = Credentials::new(&email, &password, Credentials::generate_device_id());
    let client = Client::from_credentials(creds).await?;
    println!("Logged in!");

    // Get current orders
    println!("\n=== Current Orders ===");
    let my_orders = client.my_orders().await?;

    println!("Total orders: {}", my_orders.len());

    for order in my_orders.iter().take(5) {
        println!(
            "  [{}] {} - {}p x{} (visible: {})",
            order.order_type(),
            order.item_id(),
            order.platinum(),
            order.quantity(),
            order.is_visible()
        );
    }

    // Example: Create a new sell order (commented out to prevent accidental order creation)
    println!("\n=== Create Order Example ===");
    println!("To create a sell order, uncomment the code below:\n");

    /*
    use wf_market::{CreateOrder, UpdateOrder};

    // Create a sell order for Nikana Prime Set at 100p
    let new_order = CreateOrder::sell("nikana_prime_set", 100, 1)
        .visible(true);

    let created = client.create_order(new_order).await?;
    println!("Created order: {} for {}p", created.id(), created.platinum());

    // The order ID can be used to update or delete the order later
    let order_id = created.id();

    // Update the order price
    let update = UpdateOrder::new()
        .platinum(95);

    client.update_order(order_id, update).await?;
    println!("Updated order price to 95p");

    // Hide the order (set visible to false)
    let update = UpdateOrder::new()
        .visible(false);

    client.update_order(order_id, update).await?;
    println!("Order hidden");

    // Delete the order
    client.delete_order(order_id).await?;
    println!("Order deleted");
    */

    println!(
        r#"
// Create a sell order
use wf_market::{{CreateOrder, UpdateOrder}};

let order = CreateOrder::sell("nikana_prime_set", 100, 1)
    .visible(true);
let created = client.create_order(order).await?;

// Update the order
let update = UpdateOrder::new()
    .platinum(95)
    .quantity(2);
client.update_order(created.id(), update).await?;

// Delete the order
client.delete_order(created.id()).await?;
"#
    );

    // Show owned order type safety
    println!("\n=== Type Safety with OwnedOrderId ===");
    println!("OwnedOrderId ensures you can only update/delete your own orders.");
    println!("This is enforced at compile-time!\n");

    println!(
        r#"
// This compiles - we own this order
let order = client.create_order(...).await?;
client.update_order(order.id(), UpdateOrder::new().platinum(50)).await?;

// This won't compile - OrderListing doesn't have id() that returns OwnedOrderId
// let orders = client.get_orders("item").await?;
// client.delete_order(&orders[0].id); // Error: String is not OwnedOrderId
"#
    );

    println!("Done!");
    Ok(())
}
