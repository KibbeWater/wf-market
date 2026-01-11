//! Integration tests for the V1 statistics endpoint.
//!
//! These tests require network access and are ignored by default.
//! Run with: `cargo test --features v1-api -- --ignored`

#![cfg(feature = "v1-api")]

use wf_market::Client;

/// Test fetching statistics for a regular item.
#[tokio::test]
#[ignore]
async fn test_get_statistics_regular_item() {
    let client = Client::builder().build().await.unwrap();
    let stats = client
        .get_item_statistics("nikana_prime_set")
        .await
        .unwrap();

    // Should have both closed and live statistics
    assert!(
        !stats.statistics_closed.days_90.is_empty(),
        "Should have 90-day closed statistics"
    );

    // Check closed trade data has OHLC fields
    if let Some(entry) = stats.statistics_closed.days_90.first() {
        assert!(
            entry.is_closed_trade(),
            "Closed stats should have OHLC data"
        );
        assert!(entry.open_price.is_some());
        assert!(entry.closed_price.is_some());
        assert!(entry.donch_top.is_some());
        assert!(entry.donch_bot.is_some());
        assert!(
            entry.mod_rank.is_none(),
            "Regular items should not have mod_rank"
        );
    }

    // Check live data structure - live orders have order_type but may or may not have moving_avg
    if let Some(entry) = stats.statistics_live.hours_48.first() {
        assert!(entry.is_live_order(), "Live stats should have order_type");
        assert!(
            entry.order_type.is_some(),
            "Live stats should have order_type"
        );
        // Live orders don't have open/close prices (those are for actual trades)
        assert!(entry.open_price.is_none());
        assert!(entry.closed_price.is_none());
    }
}

/// Test fetching statistics for a mod item (has mod_rank field).
#[tokio::test]
#[ignore]
async fn test_get_statistics_mod_item() {
    let client = Client::builder().build().await.unwrap();
    let stats = client.get_item_statistics("serration").await.unwrap();

    // Mods should have mod_rank in their statistics
    let has_mod_rank = stats
        .statistics_closed
        .days_90
        .iter()
        .any(|e| e.mod_rank.is_some());

    assert!(has_mod_rank, "Mod statistics should include mod_rank");
}

/// Test fetching statistics for an archon mod (reported issue).
#[tokio::test]
#[ignore]
async fn test_get_statistics_archon_mod() {
    let client = Client::builder().build().await.unwrap();
    let stats = client.get_item_statistics("archon_flow").await.unwrap();

    // Should successfully deserialize without errors
    assert!(
        !stats.statistics_closed.days_90.is_empty() || !stats.statistics_closed.hours_48.is_empty(),
        "Should have some closed statistics"
    );

    // Verify mod_rank is present
    let has_mod_rank = stats
        .statistics_closed
        .days_90
        .iter()
        .chain(stats.statistics_closed.hours_48.iter())
        .any(|e| e.mod_rank.is_some());

    assert!(
        has_mod_rank,
        "Archon mod statistics should include mod_rank"
    );

    // Live statistics should work too
    if let Some(entry) = stats.statistics_live.hours_48.first() {
        assert!(
            entry.order_type.is_some(),
            "Live stats should have order_type"
        );
    }
}

/// Test helper methods on statistics.
#[tokio::test]
#[ignore]
async fn test_statistics_helper_methods() {
    let client = Client::builder().build().await.unwrap();
    let stats = client
        .get_item_statistics("nikana_prime_set")
        .await
        .unwrap();

    // Test recent price methods
    if stats.has_sufficient_data() {
        let avg = stats.recent_avg_price();
        let median = stats.recent_median_price();

        assert!(avg.is_some(), "Should have recent average price");
        assert!(median.is_some(), "Should have recent median price");

        // Prices should be reasonable (1-10000 platinum)
        if let Some(price) = avg {
            assert!(price > 0.0 && price < 10000.0, "Price should be reasonable");
        }
    }

    // Test volume methods
    let volume_48h = stats.statistics_closed.total_volume_48h();
    let volume_90d = stats.statistics_closed.total_volume_90d();

    assert!(
        volume_90d >= volume_48h,
        "90-day volume should be >= 48-hour volume"
    );
}

/// Test that a non-existent item returns NotFound error.
#[tokio::test]
#[ignore]
async fn test_get_statistics_not_found() {
    let client = Client::builder().build().await.unwrap();
    let result = client
        .get_item_statistics("this_item_does_not_exist_12345")
        .await;

    assert!(result.is_err(), "Should return error for non-existent item");

    if let Err(e) = result {
        let error_str = format!("{:?}", e);
        assert!(
            error_str.contains("NotFound") || error_str.contains("not found"),
            "Should be a NotFound error: {}",
            error_str
        );
    }
}

// ==================== Mod Rank Filtering Tests ====================

/// Test mod rank filtering on archon_flow.
#[tokio::test]
#[ignore]
async fn test_mod_rank_filtering() {
    let client = Client::builder().build().await.unwrap();
    let stats = client.get_item_statistics("archon_flow").await.unwrap();

    // Should be detected as a mod item
    assert!(
        stats.is_mod_item(),
        "archon_flow should be detected as a mod item"
    );

    // Should have available ranks (typically 0 and 10 for archon mods)
    let ranks = stats.available_mod_ranks();
    assert!(!ranks.is_empty(), "Should have available mod ranks");
    assert!(ranks.contains(&0), "Should have rank 0 (unranked)");

    println!("Available ranks for archon_flow: {:?}", ranks);

    // Get max rank stats
    let max_rank = stats.max_rank_stats().unwrap();
    let max_rank_price = max_rank.recent_avg_price();
    println!("Max rank avg price: {:?}", max_rank_price);

    // Get unranked stats
    let unranked = stats.unranked_stats().unwrap();
    let unranked_price = unranked.recent_avg_price();
    println!("Unranked avg price: {:?}", unranked_price);

    // Max rank should generally be more expensive than unranked
    if let (Some(max_price), Some(min_price)) = (max_rank_price, unranked_price) {
        assert!(
            max_price > min_price,
            "Max rank ({}) should be more expensive than unranked ({})",
            max_price,
            min_price
        );
    }
}

/// Test that max_rank_stats returns correct filtered data.
#[tokio::test]
#[ignore]
async fn test_max_rank_stats_filtering() {
    let client = Client::builder().build().await.unwrap();
    let stats = client.get_item_statistics("serration").await.unwrap();

    // Serration is a common mod, should have rank data
    assert!(stats.is_mod_item());

    let max_rank = stats.max_rank_stats().unwrap();

    // All entries in the filtered view should have the same mod_rank
    let max_rank_value = stats.available_mod_ranks().into_iter().max().unwrap();

    for entry in &max_rank.statistics_closed.hours_48 {
        assert_eq!(
            entry.mod_rank,
            Some(max_rank_value),
            "All filtered entries should have max rank"
        );
    }

    for entry in &max_rank.statistics_closed.days_90 {
        assert_eq!(
            entry.mod_rank,
            Some(max_rank_value),
            "All filtered entries should have max rank"
        );
    }
}

/// Test stats_for_rank with invalid rank returns error.
#[tokio::test]
#[ignore]
async fn test_stats_for_rank_invalid() {
    let client = Client::builder().build().await.unwrap();
    let stats = client.get_item_statistics("archon_flow").await.unwrap();

    // Try to get a rank that doesn't exist (e.g., rank 5 for a 0/10 mod)
    let result = stats.stats_for_rank(5);

    assert!(result.is_err(), "Should return error for invalid mod rank");
}

/// Test that regular items correctly fail mod rank methods.
#[tokio::test]
#[ignore]
async fn test_regular_item_mod_rank_methods() {
    let client = Client::builder().build().await.unwrap();
    let stats = client
        .get_item_statistics("nikana_prime_set")
        .await
        .unwrap();

    // Regular items should not be detected as mods
    assert!(
        !stats.is_mod_item(),
        "nikana_prime_set should not be a mod item"
    );

    // Should have empty available ranks
    assert!(
        stats.available_mod_ranks().is_empty(),
        "Regular items should have no mod ranks"
    );

    // max_rank_stats should fail
    assert!(
        stats.max_rank_stats().is_err(),
        "max_rank_stats should fail for regular items"
    );

    // unranked_stats should fail
    assert!(
        stats.unranked_stats().is_err(),
        "unranked_stats should fail for regular items"
    );
}
