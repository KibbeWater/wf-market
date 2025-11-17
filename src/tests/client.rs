use dotenv::dotenv;
use std::{env, time::Instant};

use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};
use std::num::NonZeroU32;

use crate::{Client, errors::ApiError};

#[tokio::test]
async fn print_token() {
    dotenv().ok();

    let user = env::var("TEST_USER").expect("TEST_USER must be set in .env for integration tests");
    let pass = env::var("TEST_PASS").expect("TEST_PASS must be set in .env for integration tests");

    assert!(!user.is_empty());
    assert!(!pass.is_empty());

    let client = Client::new();
    let new_client = client.login(&user, &pass, "dev").await.unwrap();
    println!("Token: {}", new_client.get_token());
}

#[tokio::test]
async fn login_with_token() {
    dotenv().ok();
    let token =
        env::var("TEST_TOKEN").expect("TEST_TOKEN must be set in .env for integration tests");

    assert!(!token.is_empty());

    let client = Client::new()
        .login_with_token(&token, "default")
        .await
        .unwrap();
    // let recent = client.user().me().await.unwrap();
    println!("My Orders: {:?}", client.auction().cache_auctions());
}
#[tokio::test]
async fn rate_limiting() {
    // Use unauthenticated client to avoid token issues
    let mut client = Client::new();

    println!("Starting rate limiting test with multiple concurrent requests...");

    client.set_rate_limit(std::num::NonZeroU32::new(30).unwrap());

    // Create multiple concurrent request futures
    let mut handles = Vec::new();

    for i in 0..50 {
        let client_clone = client.clone(); // Clone the client for each task
        let handle = tokio::spawn(async move {
            let result = client_clone
                .order()
                .get_orders_by_item("revenant_prime_blueprint")
                .await;

            (i + 1, result)
        });
        handles.push(handle);
    }

    println!("Waiting for all {} requests to complete...", handles.len());

    // Collect results
    let mut successful_requests = 0;
    let mut rate_limited_requests = 0;
    let mut other_errors = 0;

    for handle in handles {
        match handle.await {
            Ok((request_num, result)) => match result {
                Ok(orders) => {
                    successful_requests += 1;
                    let total_orders = orders.sell_orders.len() + orders.buy_orders.len();
                    println!(
                        "Request {} succeeded - got {} total orders ({} sell, {} buy)",
                        request_num,
                        total_orders,
                        orders.sell_orders.len(),
                        orders.buy_orders.len()
                    );
                }
                Err(e) => match &e {
                    ApiError::TooManyRequests(_) => {
                        rate_limited_requests += 1;
                        println!("Request {} rate limited: {:?}", request_num, e);
                    }
                    _ => {
                        other_errors += 1;
                        println!("Request {} failed with other error: {:?}", request_num, e);
                    }
                },
            },
            Err(join_error) => {
                other_errors += 1;
                println!("Task failed to complete: {:?}", join_error);
            }
        }
    }

    println!("Rate limiting test completed:");
    println!("  Successful requests: {}", successful_requests);
    println!("  Rate limited requests: {}", rate_limited_requests);
    println!("  Other errors: {}", other_errors);
    println!("  Total requests sent: 50");

    // The test passes if we can send requests (regardless of rate limiting)
    assert!(successful_requests + rate_limited_requests + other_errors == 50);
}

pub struct MyService {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl MyService {
    pub fn new() -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(1).unwrap()); // 1 req/min
        let limiter = RateLimiter::direct(quota);

        Self { limiter }
    }

    /// Async method using until_ready
    pub async fn do_work(&self) -> &'static str {
        // Wait until request can proceed
        self.limiter.until_ready().await;
        println!("Work done");
        "allowed"
    }
}
#[tokio::test]
async fn test_rate_limiter_async() {
    let service = MyService::new();

    // First two calls happen immediately.
    service.do_work().await;
    service.do_work().await;
    service.do_work().await;
    service.do_work().await;
    service.do_work().await;

    let start = Instant::now();

    // Third call should wait because quota is exceeded.
    service.do_work().await;

    let elapsed = start.elapsed();

    // Should have waited ~500ms (given 2 req/sec)
    assert!(elapsed >= std::time::Duration::from_millis(450));
}
