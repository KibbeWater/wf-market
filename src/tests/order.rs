use crate::{
    client::{Authenticated, Client},
    enums::*,
    errors::ApiError,
    types::{CreateOrderParams, SubType, TopOrdersFilters, UpdateOrderParams},
};
use dotenv::dotenv;
use serde_json::json;
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

// Can Run on any Client
#[tokio::test]
async fn recent() {
    let client = Client::new();

    match client.order().recent().await {
        Ok(recent) => {
            // println!(
            //     "✅ Successfully fetched recent orders: {} total",
            //     recent.len()
            // );
            // if !recent.is_empty() {
            //     println!(
            //         "   First order: {} - {} platinum",
            //         recent[0].order.id, recent[0].order.platinum
            //     );
            // }
        }
        Err(e) => {
            eprintln!("💥 Failed to fetch recent orders: {:?}", e);
            assert!(false, "Failed to fetch recent orders: {:?}", e);
        }
    }
}

#[tokio::test]
async fn get_orders_by_item() {
    let client = Client::new();
    let slug = "primed_target_cracker"; // Item slug to fetch orders for

    match client.order().get_orders_by_item(slug).await {
        Ok(mut orders) => {
            orders.filter_by_sub_type(SubType::mods(10), false);
            orders.filter_user_status(StatusType::InGame, false);
            println!(
                "✅ Lowest sell order price for '{}': {} platinum",
                slug,
                orders.lowest_price(OrderType::Sell)
            );
            println!(
                "✅ Highest buy order price for '{}': {} platinum",
                slug,
                orders.highest_price(OrderType::Buy)
            );
            match crate::utils::write_json_file("orders_by_item.json", &json!(orders)) {
                Ok(_) => println!("✅ Orders by item saved to 'orders_by_item.json'"),
                Err(e) => eprintln!("💥 Failed to save orders to file: {:?}", e),
            }
        }
        Err(e) => {
            eprintln!("💥 Failed to fetch orders for '{}': {:?}", slug, e);
            assert!(false, "Failed to fetch orders for '{}': {:?}", slug, e);
        }
    }
}

#[tokio::test]
async fn get_top_orders_by_item() {
    let client = Client::new();
    let slug = "primed_target_cracker"; // Item slug to fetch orders for

    match client
        .order()
        .get_top_orders_by_item(slug, Some(TopOrdersFilters::new()))
        .await
    {
        Ok(orders) => {
            println!(
                "✅ Successfully fetched top orders for '{}': Buy: {}, Sell: {}",
                slug,
                orders.buy.len(),
                orders.sell.len()
            );
            if !orders.buy.is_empty() {
                println!(
                    "   Lowest buy price: {} platinum",
                    orders.buy[0].order.platinum
                );
            }
            if !orders.sell.is_empty() {
                println!(
                    "   Highest sell price: {} platinum",
                    orders.sell[0].order.platinum
                );
            }
        }
        Err(e) => {
            eprintln!("💥 Failed to fetch top orders for '{}': {:?}", slug, e);
            assert!(false, "Failed to fetch top orders for '{}': {:?}", slug, e);
        }
    }
}

#[tokio::test]
async fn get_by_id() {
    let id = "6859657e57605a002b649eee"; // Order ID to fetch
    let client = Client::new();

    match client.order().get_by_id(id).await {
        Ok(order) => {
            println!("✅ Successfully fetched order: {}", order.order.id);
            println!("   Type: {:?}", order.order.order_type);
            println!("   Price: {} platinum", order.order.platinum);
            println!("   Quantity: {}", order.order.quantity);
            println!("   User: {}", order.user.name);
            println!("   User Status: {:?}", order.user.status);
            println!("   Visible: {}", order.order.visible);
        }
        Err(e) => match e {
            ApiError::NotFound(_) => {
                println!("❌ Order with ID '{}' not found", id);
                // Don't panic in tests, just assert
                assert!(false, "Order not found");
            }
            ApiError::ParsingError(req_err, parse_err) => {
                eprintln!("📝 Parsing error: {:?} - {}", req_err, parse_err);
                assert!(false, "Parsing error: {}", parse_err);
            }
            _ => {
                eprintln!("💥 Unexpected error: {:?}", e);
                assert!(false, "Unexpected error: {:?}", e);
            }
        },
    }
}

// Can Only Run on Authenticated Client
#[tokio::test]
async fn my_orders() {
    match setup_client().await {
        Ok(client) => {
            println!(
                "✅ Successfully authenticated as: {:?}",
                client.get_user().unwrap().ingame_name
            );

            match client.order().my_orders().await {
                Ok(orders) => {
                    let lowest_buy = orders.lowest_order(OrderType::Buy);
                    if let Some(order) = lowest_buy {
                        println!(
                            "   Lowest buy order: {} - {} platinum",
                            order.item_id,
                            order.per_trade.unwrap_or(0),
                        );
                    }
                    println!(
                        "✅ Successfully fetched my orders: {} total",
                        orders.total_orders()
                    );
                    for (i, order) in orders.to_vec().iter().take(3).enumerate() {
                        println!(
                            "   Order {}: {} - {} platinum ({:?})",
                            i + 1,
                            order.id,
                            order.platinum,
                            order.order_type
                        );
                    }
                }
                Err(e) => {
                    eprintln!("💥 Failed to fetch my orders: {:?}", e);
                    assert!(false, "Failed to fetch my orders: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("🔐 Authentication failed: {:?}", e);
            assert!(false, "Authentication failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn update_order() {
    let id = "68992b99c7c642505205d926"; // Order ID to update

    match setup_client().await {
        Ok(client) => {
            match client
                .order()
                .update(id, UpdateOrderParams::new().with_platinum(12))
                .await
            {
                Ok(order) => {
                    println!("✅ Successfully updated order: {}", order.id);
                    println!("   New price: {} platinum", order.platinum);
                    println!(
                        "   Cached orders: {}",
                        client.order().cache_orders().total_orders()
                    );
                    crate::utils::write_json_file(
                        "updated_order.json",
                        &json!(client.order().cache_orders()),
                    )
                    .expect("Failed to write updated order to file");
                }
                Err(e) => {
                    eprintln!("💥 Failed to update order '{}': {:?}", id, e);
                    assert!(false, "Failed to update order: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("🔐 Authentication failed: {:?}", e);
            assert!(false, "Authentication failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn create_regular_order() {
    let id = "5c1bda1314a8e4006b1dad81"; // Secura Dual Cestra Item ID

    match setup_client().await {
        Ok(client) => {
            match client
                .order()
                .create(CreateOrderParams::new_with_subtype(
                    id,
                    OrderType::Buy,
                    10,
                    11,
                    true,
                    Some(6),
                    SubType::mods(5),
                ))
                .await
            {
                Ok(new_order) => {
                    println!("✅ Successfully created new order: {}", new_order.id);
                    println!("   Item ID: {}", id);
                    println!("   Type: {:?}", new_order.order_type);
                    println!("   Price: {} platinum", new_order.platinum);
                    println!("   Quantity: {}", new_order.quantity);
                }
                Err(e) => {
                    eprintln!("💥 Failed to create order: {:?}", e);
                    assert!(false, "Failed to create order: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("🔐 Authentication failed: {:?}", e);
            assert!(false, "Authentication failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn close_order() {
    let id = "68993213a57e82e4a14a7e2e"; // Order ID to close

    match setup_client().await {
        Ok(client) => {
            // First fetch orders to ensure we have them cached
            match client.order().my_orders().await {
                Ok(_) => match client.order().close(id, 2).await {
                    Ok(response) => {
                        println!("✅ Successfully closed order: {}", id);
                        println!("   Response: {:?}", response);
                        println!(
                            "   Remaining cached orders: {}",
                            client.order().cache_orders().total_orders()
                        );
                        crate::utils::write_json_file(
                            "closed_order.json",
                            &json!(client.order().cache_orders()),
                        )
                        .expect("Failed to write closed order to file");
                    }
                    Err(e) => {
                        eprintln!("💥 Failed to close order '{}': {:?}", id, e);
                        assert!(false, "Failed to close order: {:?}", e);
                    }
                },
                Err(e) => {
                    eprintln!("💥 Failed to fetch orders before closing: {:?}", e);
                    assert!(false, "Failed to fetch orders: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("🔐 Authentication failed: {:?}", e);
            assert!(false, "Authentication failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn delete_order() {
    let id = "685b24fb914e45bf7792c7b9"; // Order ID to delete

    match setup_client().await {
        Ok(client) => match client.order().delete(id).await {
            Ok(response) => {
                println!("✅ Successfully deleted order: {}", id);
                println!("   Response: {:?}", response);
            }
            Err(e) => {
                eprintln!("💥 Failed to delete order '{}': {:?}", id, e);
                assert!(false, "Failed to delete order: {:?}", e);
            }
        },
        Err(e) => {
            eprintln!("🔐 Authentication failed: {:?}", e);
            assert!(false, "Authentication failed: {:?}", e);
        }
    }
}
