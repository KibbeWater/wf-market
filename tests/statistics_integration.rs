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
