//! Item statistics models (V1 API).
//!
//! This module contains data structures for item trading statistics
//! from the V1 API endpoint `/v1/items/{slug}/statistics`.
//!
//! # Note
//!
//! These types are only available with the `v1-api` feature flag.
//! The V1 API is deprecated and will be removed when V2 equivalents
//! become available.

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// A single statistics data point for item trading activity.
///
/// Contains OHLC-style price data and technical indicators for either
/// an hourly (48-hour data) or daily (90-day data) period.
///
/// # Price Fields
///
/// - `open_price` / `closed_price`: First and last trade prices in the period (closed trades only)
/// - `min_price` / `max_price`: Price range for the period
/// - `avg_price`: Simple average of all trades
/// - `wa_price`: Volume-weighted average price
/// - `median`: Median price
/// - `moving_avg`: Moving average (smoothed trend, closed trades only)
///
/// # Technical Indicators
///
/// - `donch_top` / `donch_bot`: Donchian channel bounds (closed trades only)
///
/// # Note
///
/// Some fields are only present in `statistics_closed` data:
/// - `open_price`, `closed_price`, `moving_avg`, `donch_top`, `donch_bot`
///
/// The `statistics_live` data has `order_type` instead of OHLC prices.
/// Helper module for deserializing numbers that may be int or float.
mod number_utils {
    use serde::{Deserialize, Deserializer};

    /// Deserialize a number as i32, accepting both integer and float JSON values.
    pub fn deserialize_as_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f64::deserialize(deserializer)?;
        Ok(value as i32)
    }

    /// Deserialize an optional number as i32, accepting both integer and float JSON values.
    pub fn deserialize_optional_as_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: Option<f64> = Option::deserialize(deserializer)?;
        Ok(value.map(|v| v as i32))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatisticEntry {
    /// Unique identifier for this data point.
    pub id: String,

    /// Timestamp for this data point.
    ///
    /// For 48-hour data: hourly intervals.
    /// For 90-day data: daily intervals.
    pub datetime: DateTime<Utc>,

    /// Number of trades/orders in this period.
    #[serde(deserialize_with = "number_utils::deserialize_as_i32")]
    pub volume: i32,

    /// Lowest price in this period.
    #[serde(deserialize_with = "number_utils::deserialize_as_i32")]
    pub min_price: i32,

    /// Highest price in this period.
    #[serde(deserialize_with = "number_utils::deserialize_as_i32")]
    pub max_price: i32,

    /// First trade price in this period (open).
    ///
    /// Only present in `statistics_closed` data.
    #[serde(
        default,
        deserialize_with = "number_utils::deserialize_optional_as_i32"
    )]
    pub open_price: Option<i32>,

    /// Last trade price in this period (close).
    ///
    /// Only present in `statistics_closed` data.
    #[serde(
        default,
        deserialize_with = "number_utils::deserialize_optional_as_i32"
    )]
    pub closed_price: Option<i32>,

    /// Simple average price.
    pub avg_price: f64,

    /// Volume-weighted average price.
    pub wa_price: f64,

    /// Median price.
    pub median: f64,

    /// Moving average price.
    ///
    /// Present in most statistics, may be absent in some live data.
    pub moving_avg: Option<f64>,

    /// Donchian channel top (highest high over lookback period).
    ///
    /// Only present in `statistics_closed` data.
    #[serde(
        default,
        deserialize_with = "number_utils::deserialize_optional_as_i32"
    )]
    pub donch_top: Option<i32>,

    /// Donchian channel bottom (lowest low over lookback period).
    ///
    /// Only present in `statistics_closed` data.
    #[serde(
        default,
        deserialize_with = "number_utils::deserialize_optional_as_i32"
    )]
    pub donch_bot: Option<i32>,

    /// Order type for live statistics.
    ///
    /// Only present in `statistics_live` data. Values: "sell" or "buy".
    pub order_type: Option<String>,

    /// Mod rank for mod items.
    ///
    /// Only present for tradeable mods. Indicates the rank of the mod
    /// that these statistics apply to.
    #[serde(
        default,
        deserialize_with = "number_utils::deserialize_optional_as_i32"
    )]
    pub mod_rank: Option<i32>,
}

impl StatisticEntry {
    /// Returns `true` if no trades occurred in this period.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.volume == 0
    }

    /// Returns the price range (spread) for this period.
    #[must_use]
    pub fn price_range(&self) -> i32 {
        self.max_price - self.min_price
    }

    /// Returns the Donchian channel width.
    ///
    /// Returns `None` if Donchian channel data is not available
    /// (e.g., for live statistics).
    #[must_use]
    pub fn donchian_width(&self) -> Option<i32> {
        match (self.donch_top, self.donch_bot) {
            (Some(top), Some(bot)) => Some(top - bot),
            _ => None,
        }
    }

    /// Returns `true` if this entry is from closed/completed trades.
    ///
    /// Closed trade entries have OHLC data (open_price, closed_price).
    #[must_use]
    pub fn is_closed_trade(&self) -> bool {
        self.open_price.is_some()
    }

    /// Returns `true` if this entry is from live orders.
    ///
    /// Live order entries have order_type instead of OHLC data.
    #[must_use]
    pub fn is_live_order(&self) -> bool {
        self.order_type.is_some()
    }
}

/// Statistics grouped by timeframe.
///
/// Contains both short-term (48 hours, hourly) and long-term (90 days, daily)
/// statistics for an item.
#[derive(Debug, Clone, Deserialize)]
pub struct TimeframedStatistics {
    /// Hourly data points for the last 48 hours.
    ///
    /// Each entry represents one hour of trading activity.
    #[serde(rename = "48hours")]
    pub hours_48: Vec<StatisticEntry>,

    /// Daily data points for the last 90 days.
    ///
    /// Each entry represents one day of trading activity.
    #[serde(rename = "90days")]
    pub days_90: Vec<StatisticEntry>,
}

impl TimeframedStatistics {
    /// Get the most recent hourly data point.
    #[must_use]
    pub fn latest_hourly(&self) -> Option<&StatisticEntry> {
        self.hours_48.last()
    }

    /// Get the most recent daily data point.
    #[must_use]
    pub fn latest_daily(&self) -> Option<&StatisticEntry> {
        self.days_90.last()
    }

    /// Calculate the average price over the last 48 hours.
    ///
    /// Returns `None` if there are no data points with volume.
    #[must_use]
    pub fn avg_price_48h(&self) -> Option<f64> {
        let entries: Vec<_> = self.hours_48.iter().filter(|e| e.volume > 0).collect();
        if entries.is_empty() {
            return None;
        }
        let sum: f64 = entries.iter().map(|e| e.avg_price).sum();
        Some(sum / entries.len() as f64)
    }

    /// Calculate the average price over the last 90 days.
    ///
    /// Returns `None` if there are no data points with volume.
    #[must_use]
    pub fn avg_price_90d(&self) -> Option<f64> {
        let entries: Vec<_> = self.days_90.iter().filter(|e| e.volume > 0).collect();
        if entries.is_empty() {
            return None;
        }
        let sum: f64 = entries.iter().map(|e| e.avg_price).sum();
        Some(sum / entries.len() as f64)
    }

    /// Calculate total volume over the last 48 hours.
    #[must_use]
    pub fn total_volume_48h(&self) -> i32 {
        self.hours_48.iter().map(|e| e.volume).sum()
    }

    /// Calculate total volume over the last 90 days.
    #[must_use]
    pub fn total_volume_90d(&self) -> i32 {
        self.days_90.iter().map(|e| e.volume).sum()
    }

    /// Get all available mod ranks in this timeframe.
    ///
    /// Returns a sorted vector of unique mod ranks found in both 48-hour and 90-day data.
    /// Returns an empty vector for non-mod items.
    #[must_use]
    pub fn available_mod_ranks(&self) -> Vec<i32> {
        use std::collections::BTreeSet;

        self.hours_48
            .iter()
            .chain(self.days_90.iter())
            .filter_map(|e| e.mod_rank)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Filter entries to a specific mod rank, returning a view.
    ///
    /// This is an internal helper used by `ItemStatistics::stats_for_rank`.
    fn filter_by_rank(&self, rank: i32) -> TimeframedStatisticsView<'_> {
        TimeframedStatisticsView {
            hours_48: self
                .hours_48
                .iter()
                .filter(|e| e.mod_rank == Some(rank))
                .collect(),
            days_90: self
                .days_90
                .iter()
                .filter(|e| e.mod_rank == Some(rank))
                .collect(),
        }
    }
}

/// Complete item trading statistics.
///
/// Contains statistics from both completed trades (`statistics_closed`)
/// and live/pending orders (`statistics_live`).
///
/// # Example
///
/// ```ignore
/// let stats = client.get_item_statistics("nikana_prime_set").await?;
///
/// // Get recent closed trade data
/// if let Some(latest) = stats.statistics_closed.latest_daily() {
///     println!("Yesterday: {}p avg, {} trades", latest.avg_price, latest.volume);
/// }
///
/// // Compare with live order data
/// if let Some(live) = stats.statistics_live.latest_hourly() {
///     println!("Current listings: {}p avg", live.avg_price);
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct ItemStatistics {
    /// Statistics from completed/closed trades.
    ///
    /// This reflects actual transaction prices.
    pub statistics_closed: TimeframedStatistics,

    /// Statistics from live/pending orders.
    ///
    /// This reflects current asking/bidding prices.
    pub statistics_live: TimeframedStatistics,
}

impl ItemStatistics {
    /// Get the most recent average price from closed trades (daily).
    ///
    /// This is often the most useful single price indicator.
    #[must_use]
    pub fn recent_avg_price(&self) -> Option<f64> {
        self.statistics_closed
            .latest_daily()
            .filter(|e| e.volume > 0)
            .map(|e| e.avg_price)
    }

    /// Get the most recent median price from closed trades (daily).
    #[must_use]
    pub fn recent_median_price(&self) -> Option<f64> {
        self.statistics_closed
            .latest_daily()
            .filter(|e| e.volume > 0)
            .map(|e| e.median)
    }

    /// Check if there's enough trading activity for reliable statistics.
    ///
    /// Returns `true` if there were trades in at least 7 of the last 90 days.
    #[must_use]
    pub fn has_sufficient_data(&self) -> bool {
        let days_with_trades = self
            .statistics_closed
            .days_90
            .iter()
            .filter(|e| e.volume > 0)
            .count();
        days_with_trades >= 7
    }

    /// Check if this item has mod rank data.
    ///
    /// Returns `true` if any statistic entry has a `mod_rank` field,
    /// indicating this is a mod item with rank-specific statistics.
    #[must_use]
    pub fn is_mod_item(&self) -> bool {
        self.statistics_closed
            .hours_48
            .iter()
            .chain(self.statistics_closed.days_90.iter())
            .chain(self.statistics_live.hours_48.iter())
            .chain(self.statistics_live.days_90.iter())
            .any(|e| e.mod_rank.is_some())
    }

    /// Get all available mod ranks for this item.
    ///
    /// Returns a sorted vector of unique mod ranks found across all statistics.
    /// Returns an empty vector for non-mod items.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let stats = client.get_item_statistics("archon_flow").await?;
    /// let ranks = stats.available_mod_ranks(); // e.g., [0, 10]
    /// ```
    #[must_use]
    pub fn available_mod_ranks(&self) -> Vec<i32> {
        use std::collections::BTreeSet;

        self.statistics_closed
            .hours_48
            .iter()
            .chain(self.statistics_closed.days_90.iter())
            .chain(self.statistics_live.hours_48.iter())
            .chain(self.statistics_live.days_90.iter())
            .filter_map(|e| e.mod_rank)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Get statistics filtered to a specific mod rank.
    ///
    /// Returns a zero-copy view containing only entries matching the specified rank.
    ///
    /// # Errors
    ///
    /// Returns `Error::NotFound` if the specified rank is not available for this item.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let stats = client.get_item_statistics("archon_flow").await?;
    /// let rank10 = stats.stats_for_rank(10)?;
    /// println!("Max rank avg price: {:?}", rank10.recent_avg_price());
    /// ```
    pub fn stats_for_rank(&self, rank: i32) -> crate::error::Result<ItemStatisticsView<'_>> {
        let available = self.available_mod_ranks();
        if !available.contains(&rank) {
            return Err(crate::error::Error::not_found(format!(
                "mod rank {} (available ranks: {:?})",
                rank, available
            )));
        }

        Ok(ItemStatisticsView {
            statistics_closed: self.statistics_closed.filter_by_rank(rank),
            statistics_live: self.statistics_live.filter_by_rank(rank),
        })
    }

    /// Get statistics for the maximum (highest) mod rank.
    ///
    /// This is a convenience method for getting max-rank mod statistics,
    /// which is typically what traders want for pricing.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidConfig` if this is not a mod item.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let stats = client.get_item_statistics("archon_flow").await?;
    /// let max_rank = stats.max_rank_stats()?;
    /// println!("Max rank avg: {:?}", max_rank.recent_avg_price());
    /// ```
    pub fn max_rank_stats(&self) -> crate::error::Result<ItemStatisticsView<'_>> {
        let ranks = self.available_mod_ranks();
        let max_rank = ranks.into_iter().max().ok_or_else(|| {
            crate::error::Error::InvalidConfig(
                "max_rank_stats() requires a mod item with mod_rank data".into(),
            )
        })?;
        self.stats_for_rank(max_rank)
    }

    /// Get statistics for unranked (rank 0) mods.
    ///
    /// This is a convenience method for getting unranked mod statistics.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidConfig` if this is not a mod item.
    /// Returns `Error::NotFound` if rank 0 is not available (rare edge case).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let stats = client.get_item_statistics("archon_flow").await?;
    /// let unranked = stats.unranked_stats()?;
    /// println!("Unranked avg: {:?}", unranked.recent_avg_price());
    /// ```
    pub fn unranked_stats(&self) -> crate::error::Result<ItemStatisticsView<'_>> {
        if !self.is_mod_item() {
            return Err(crate::error::Error::InvalidConfig(
                "unranked_stats() requires a mod item with mod_rank data".into(),
            ));
        }
        self.stats_for_rank(0)
    }
}

/// A zero-copy view into timeframed statistics filtered by mod rank.
///
/// This type provides the same interface as [`TimeframedStatistics`] but
/// holds references to filtered entries rather than owning them.
#[derive(Debug, Clone)]
pub struct TimeframedStatisticsView<'a> {
    /// Hourly data points for the last 48 hours (filtered).
    pub hours_48: Vec<&'a StatisticEntry>,

    /// Daily data points for the last 90 days (filtered).
    pub days_90: Vec<&'a StatisticEntry>,
}

impl<'a> TimeframedStatisticsView<'a> {
    /// Get the most recent hourly data point.
    #[must_use]
    pub fn latest_hourly(&self) -> Option<&StatisticEntry> {
        self.hours_48.last().copied()
    }

    /// Get the most recent daily data point.
    #[must_use]
    pub fn latest_daily(&self) -> Option<&StatisticEntry> {
        self.days_90.last().copied()
    }

    /// Calculate the average price over the last 48 hours.
    ///
    /// Returns `None` if there are no data points with volume.
    #[must_use]
    pub fn avg_price_48h(&self) -> Option<f64> {
        let entries: Vec<_> = self.hours_48.iter().filter(|e| e.volume > 0).collect();
        if entries.is_empty() {
            return None;
        }
        let sum: f64 = entries.iter().map(|e| e.avg_price).sum();
        Some(sum / entries.len() as f64)
    }

    /// Calculate the average price over the last 90 days.
    ///
    /// Returns `None` if there are no data points with volume.
    #[must_use]
    pub fn avg_price_90d(&self) -> Option<f64> {
        let entries: Vec<_> = self.days_90.iter().filter(|e| e.volume > 0).collect();
        if entries.is_empty() {
            return None;
        }
        let sum: f64 = entries.iter().map(|e| e.avg_price).sum();
        Some(sum / entries.len() as f64)
    }

    /// Calculate total volume over the last 48 hours.
    #[must_use]
    pub fn total_volume_48h(&self) -> i32 {
        self.hours_48.iter().map(|e| e.volume).sum()
    }

    /// Calculate total volume over the last 90 days.
    #[must_use]
    pub fn total_volume_90d(&self) -> i32 {
        self.days_90.iter().map(|e| e.volume).sum()
    }
}

/// A zero-copy view into item statistics filtered by mod rank.
///
/// This type provides the same interface as [`ItemStatistics`] but
/// holds references to filtered entries rather than owning them.
///
/// # Example
///
/// ```ignore
/// let stats = client.get_item_statistics("archon_flow").await?;
///
/// // Get max rank statistics
/// let max_rank = stats.max_rank_stats()?;
/// println!("Max rank avg: {:?}", max_rank.recent_avg_price());
///
/// // Compare with unranked
/// let unranked = stats.unranked_stats()?;
/// println!("Unranked avg: {:?}", unranked.recent_avg_price());
/// ```
#[derive(Debug, Clone)]
pub struct ItemStatisticsView<'a> {
    /// Statistics from completed/closed trades (filtered).
    pub statistics_closed: TimeframedStatisticsView<'a>,

    /// Statistics from live/pending orders (filtered).
    pub statistics_live: TimeframedStatisticsView<'a>,
}

impl<'a> ItemStatisticsView<'a> {
    /// Get the most recent average price from closed trades (daily).
    ///
    /// This is often the most useful single price indicator.
    #[must_use]
    pub fn recent_avg_price(&self) -> Option<f64> {
        self.statistics_closed
            .latest_daily()
            .filter(|e| e.volume > 0)
            .map(|e| e.avg_price)
    }

    /// Get the most recent median price from closed trades (daily).
    #[must_use]
    pub fn recent_median_price(&self) -> Option<f64> {
        self.statistics_closed
            .latest_daily()
            .filter(|e| e.volume > 0)
            .map(|e| e.median)
    }

    /// Check if there's enough trading activity for reliable statistics.
    ///
    /// Returns `true` if there were trades in at least 7 of the last 90 days.
    #[must_use]
    pub fn has_sufficient_data(&self) -> bool {
        let days_with_trades = self
            .statistics_closed
            .days_90
            .iter()
            .filter(|e| e.volume > 0)
            .count();
        days_with_trades >= 7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a closed trade entry (has OHLC data).
    fn make_closed_entry(volume: i32, avg_price: f64) -> StatisticEntry {
        StatisticEntry {
            id: "test".to_string(),
            datetime: Utc::now(),
            volume,
            min_price: 10,
            max_price: 20,
            open_price: Some(12),
            closed_price: Some(18),
            avg_price,
            wa_price: avg_price,
            median: avg_price,
            moving_avg: Some(avg_price),
            donch_top: Some(25),
            donch_bot: Some(5),
            order_type: None,
            mod_rank: None,
        }
    }

    /// Create a live order entry (has order_type, no OHLC).
    fn make_live_entry(volume: i32, avg_price: f64, order_type: &str) -> StatisticEntry {
        StatisticEntry {
            id: "test".to_string(),
            datetime: Utc::now(),
            volume,
            min_price: 10,
            max_price: 20,
            open_price: None,
            closed_price: None,
            avg_price,
            wa_price: avg_price,
            median: avg_price,
            moving_avg: None,
            donch_top: None,
            donch_bot: None,
            order_type: Some(order_type.to_string()),
            mod_rank: None,
        }
    }

    #[test]
    fn test_statistic_entry_is_empty() {
        let empty = make_closed_entry(0, 0.0);
        assert!(empty.is_empty());

        let with_volume = make_closed_entry(10, 15.0);
        assert!(!with_volume.is_empty());
    }

    #[test]
    fn test_statistic_entry_price_range() {
        let entry = make_closed_entry(10, 15.0);
        assert_eq!(entry.price_range(), 10); // 20 - 10
    }

    #[test]
    fn test_statistic_entry_donchian_width() {
        let closed = make_closed_entry(10, 15.0);
        assert_eq!(closed.donchian_width(), Some(20)); // 25 - 5

        let live = make_live_entry(10, 15.0, "sell");
        assert_eq!(live.donchian_width(), None);
    }

    #[test]
    fn test_statistic_entry_type_detection() {
        let closed = make_closed_entry(10, 15.0);
        assert!(closed.is_closed_trade());
        assert!(!closed.is_live_order());

        let live = make_live_entry(10, 15.0, "sell");
        assert!(!live.is_closed_trade());
        assert!(live.is_live_order());
    }

    #[test]
    fn test_timeframed_statistics_latest() {
        let stats = TimeframedStatistics {
            hours_48: vec![make_closed_entry(5, 10.0), make_closed_entry(10, 15.0)],
            days_90: vec![make_closed_entry(100, 12.0), make_closed_entry(150, 14.0)],
        };

        assert_eq!(stats.latest_hourly().unwrap().avg_price, 15.0);
        assert_eq!(stats.latest_daily().unwrap().avg_price, 14.0);
    }

    #[test]
    fn test_timeframed_statistics_avg_price() {
        let stats = TimeframedStatistics {
            hours_48: vec![make_closed_entry(5, 10.0), make_closed_entry(10, 20.0)],
            days_90: vec![make_closed_entry(0, 0.0), make_closed_entry(100, 15.0)], // first has no volume
        };

        assert_eq!(stats.avg_price_48h(), Some(15.0)); // (10 + 20) / 2
        assert_eq!(stats.avg_price_90d(), Some(15.0)); // only entry with volume
    }

    #[test]
    fn test_timeframed_statistics_total_volume() {
        let stats = TimeframedStatistics {
            hours_48: vec![make_closed_entry(5, 10.0), make_closed_entry(10, 15.0)],
            days_90: vec![make_closed_entry(100, 12.0), make_closed_entry(150, 14.0)],
        };

        assert_eq!(stats.total_volume_48h(), 15);
        assert_eq!(stats.total_volume_90d(), 250);
    }

    #[test]
    fn test_item_statistics_recent_price() {
        let closed = TimeframedStatistics {
            hours_48: vec![make_closed_entry(5, 10.0)],
            days_90: vec![make_closed_entry(100, 15.0)],
        };
        let live = TimeframedStatistics {
            hours_48: vec![make_live_entry(20, 18.0, "sell")],
            days_90: vec![make_live_entry(50, 16.0, "sell")],
        };

        let stats = ItemStatistics {
            statistics_closed: closed,
            statistics_live: live,
        };

        assert_eq!(stats.recent_avg_price(), Some(15.0));
        assert_eq!(stats.recent_median_price(), Some(15.0));
    }

    #[test]
    fn test_item_statistics_has_sufficient_data() {
        let mut entries = Vec::new();
        for i in 0..10 {
            entries.push(make_closed_entry(if i < 7 { 10 } else { 0 }, 15.0));
        }

        let stats = ItemStatistics {
            statistics_closed: TimeframedStatistics {
                hours_48: vec![],
                days_90: entries,
            },
            statistics_live: TimeframedStatistics {
                hours_48: vec![],
                days_90: vec![],
            },
        };

        assert!(stats.has_sufficient_data());
    }

    #[test]
    fn test_item_statistics_insufficient_data() {
        let entries = vec![
            make_closed_entry(10, 15.0),
            make_closed_entry(0, 0.0),
            make_closed_entry(10, 15.0),
        ];

        let stats = ItemStatistics {
            statistics_closed: TimeframedStatistics {
                hours_48: vec![],
                days_90: entries,
            },
            statistics_live: TimeframedStatistics {
                hours_48: vec![],
                days_90: vec![],
            },
        };

        assert!(!stats.has_sufficient_data()); // Only 2 days with trades
    }

    #[test]
    fn test_deserialize_closed_statistics() {
        let json = r#"{
            "id": "abc123",
            "datetime": "2026-01-10T12:00:00.000+00:00",
            "volume": 10,
            "min_price": 90,
            "max_price": 110,
            "open_price": 95,
            "closed_price": 105,
            "avg_price": 100.0,
            "wa_price": 99.5,
            "median": 100.0,
            "moving_avg": 98.0,
            "donch_top": 120,
            "donch_bot": 80,
            "mod_rank": 10
        }"#;

        let entry: StatisticEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, "abc123");
        assert_eq!(entry.volume, 10);
        assert_eq!(entry.open_price, Some(95));
        assert_eq!(entry.closed_price, Some(105));
        assert_eq!(entry.moving_avg, Some(98.0));
        assert_eq!(entry.donch_top, Some(120));
        assert_eq!(entry.donch_bot, Some(80));
        assert_eq!(entry.mod_rank, Some(10));
        assert!(entry.order_type.is_none());
        assert!(entry.is_closed_trade());
    }

    #[test]
    fn test_deserialize_live_statistics() {
        let json = r#"{
            "id": "def456",
            "datetime": "2026-01-10T12:00:00.000+00:00",
            "volume": 50,
            "min_price": 100,
            "max_price": 150,
            "avg_price": 125.0,
            "wa_price": 123.5,
            "median": 120.0,
            "order_type": "sell",
            "mod_rank": 0
        }"#;

        let entry: StatisticEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, "def456");
        assert_eq!(entry.volume, 50);
        assert!(entry.open_price.is_none());
        assert!(entry.closed_price.is_none());
        assert!(entry.moving_avg.is_none());
        assert!(entry.donch_top.is_none());
        assert!(entry.donch_bot.is_none());
        assert_eq!(entry.order_type, Some("sell".to_string()));
        assert_eq!(entry.mod_rank, Some(0));
        assert!(entry.is_live_order());
    }

    #[test]
    fn test_deserialize_full_response() {
        let json = r#"{
            "statistics_closed": {
                "48hours": [{
                    "id": "h1",
                    "datetime": "2026-01-10T12:00:00.000+00:00",
                    "volume": 5,
                    "min_price": 90,
                    "max_price": 110,
                    "open_price": 95,
                    "closed_price": 100,
                    "avg_price": 98.0,
                    "wa_price": 97.5,
                    "median": 98.0,
                    "moving_avg": 96.0,
                    "donch_top": 115,
                    "donch_bot": 85
                }],
                "90days": []
            },
            "statistics_live": {
                "48hours": [{
                    "id": "l1",
                    "datetime": "2026-01-10T12:00:00.000+00:00",
                    "volume": 20,
                    "min_price": 100,
                    "max_price": 130,
                    "avg_price": 115.0,
                    "wa_price": 112.0,
                    "median": 110.0,
                    "order_type": "sell"
                }],
                "90days": []
            }
        }"#;

        let stats: ItemStatistics = serde_json::from_str(json).unwrap();
        assert_eq!(stats.statistics_closed.hours_48.len(), 1);
        assert_eq!(stats.statistics_live.hours_48.len(), 1);

        let closed = &stats.statistics_closed.hours_48[0];
        assert!(closed.is_closed_trade());
        assert_eq!(closed.open_price, Some(95));

        let live = &stats.statistics_live.hours_48[0];
        assert!(live.is_live_order());
        assert_eq!(live.order_type, Some("sell".to_string()));
    }

    // ==================== Mod Rank Filtering Tests ====================

    /// Create a closed trade entry with a specific mod rank.
    fn make_closed_entry_with_rank(volume: i32, avg_price: f64, mod_rank: i32) -> StatisticEntry {
        StatisticEntry {
            id: format!("test_rank_{}", mod_rank),
            datetime: Utc::now(),
            volume,
            min_price: 10,
            max_price: 20,
            open_price: Some(12),
            closed_price: Some(18),
            avg_price,
            wa_price: avg_price,
            median: avg_price,
            moving_avg: Some(avg_price),
            donch_top: Some(25),
            donch_bot: Some(5),
            order_type: None,
            mod_rank: Some(mod_rank),
        }
    }

    /// Create a live order entry with a specific mod rank.
    fn make_live_entry_with_rank(
        volume: i32,
        avg_price: f64,
        order_type: &str,
        mod_rank: i32,
    ) -> StatisticEntry {
        StatisticEntry {
            id: format!("test_live_rank_{}", mod_rank),
            datetime: Utc::now(),
            volume,
            min_price: 10,
            max_price: 20,
            open_price: None,
            closed_price: None,
            avg_price,
            wa_price: avg_price,
            median: avg_price,
            moving_avg: None,
            donch_top: None,
            donch_bot: None,
            order_type: Some(order_type.to_string()),
            mod_rank: Some(mod_rank),
        }
    }

    /// Helper to create mod item statistics with rank 0 and rank 10 entries.
    fn make_mod_item_statistics() -> ItemStatistics {
        ItemStatistics {
            statistics_closed: TimeframedStatistics {
                hours_48: vec![
                    make_closed_entry_with_rank(5, 20.0, 0),   // unranked: 20p
                    make_closed_entry_with_rank(3, 100.0, 10), // max rank: 100p
                ],
                days_90: vec![
                    make_closed_entry_with_rank(50, 25.0, 0),   // unranked: 25p
                    make_closed_entry_with_rank(30, 110.0, 10), // max rank: 110p
                ],
            },
            statistics_live: TimeframedStatistics {
                hours_48: vec![
                    make_live_entry_with_rank(10, 22.0, "sell", 0),
                    make_live_entry_with_rank(8, 105.0, "sell", 10),
                ],
                days_90: vec![],
            },
        }
    }

    #[test]
    fn test_is_mod_item() {
        // Mod item with mod_rank
        let mod_stats = make_mod_item_statistics();
        assert!(mod_stats.is_mod_item());

        // Regular item without mod_rank
        let regular_stats = ItemStatistics {
            statistics_closed: TimeframedStatistics {
                hours_48: vec![make_closed_entry(10, 50.0)],
                days_90: vec![make_closed_entry(100, 55.0)],
            },
            statistics_live: TimeframedStatistics {
                hours_48: vec![],
                days_90: vec![],
            },
        };
        assert!(!regular_stats.is_mod_item());
    }

    #[test]
    fn test_available_mod_ranks() {
        let stats = make_mod_item_statistics();
        let ranks = stats.available_mod_ranks();

        assert_eq!(ranks, vec![0, 10]);
    }

    #[test]
    fn test_available_mod_ranks_empty_for_regular_items() {
        let stats = ItemStatistics {
            statistics_closed: TimeframedStatistics {
                hours_48: vec![make_closed_entry(10, 50.0)],
                days_90: vec![],
            },
            statistics_live: TimeframedStatistics {
                hours_48: vec![],
                days_90: vec![],
            },
        };

        assert!(stats.available_mod_ranks().is_empty());
    }

    #[test]
    fn test_stats_for_rank_success() {
        let stats = make_mod_item_statistics();

        // Filter to rank 10
        let rank10 = stats.stats_for_rank(10).unwrap();

        // Should only have rank 10 entries
        assert_eq!(rank10.statistics_closed.hours_48.len(), 1);
        assert_eq!(rank10.statistics_closed.hours_48[0].mod_rank, Some(10));
        assert_eq!(rank10.statistics_closed.hours_48[0].avg_price, 100.0);

        assert_eq!(rank10.statistics_closed.days_90.len(), 1);
        assert_eq!(rank10.statistics_closed.days_90[0].mod_rank, Some(10));
        assert_eq!(rank10.statistics_closed.days_90[0].avg_price, 110.0);

        // Filter to rank 0
        let rank0 = stats.stats_for_rank(0).unwrap();
        assert_eq!(rank0.statistics_closed.hours_48.len(), 1);
        assert_eq!(rank0.statistics_closed.hours_48[0].mod_rank, Some(0));
        assert_eq!(rank0.statistics_closed.hours_48[0].avg_price, 20.0);
    }

    #[test]
    fn test_stats_for_rank_not_found() {
        let stats = make_mod_item_statistics();

        // Try to filter to a rank that doesn't exist
        let result = stats.stats_for_rank(5);
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_str = format!("{}", err);
        assert!(err_str.contains("not found"));
        assert!(err_str.contains("mod rank 5"));
    }

    #[test]
    fn test_max_rank_stats() {
        let stats = make_mod_item_statistics();

        let max_rank = stats.max_rank_stats().unwrap();

        // Should be rank 10 stats
        assert_eq!(max_rank.statistics_closed.hours_48[0].mod_rank, Some(10));
        assert_eq!(max_rank.statistics_closed.hours_48[0].avg_price, 100.0);

        // recent_avg_price should work on the view
        assert_eq!(max_rank.recent_avg_price(), Some(110.0));
    }

    #[test]
    fn test_unranked_stats() {
        let stats = make_mod_item_statistics();

        let unranked = stats.unranked_stats().unwrap();

        // Should be rank 0 stats
        assert_eq!(unranked.statistics_closed.hours_48[0].mod_rank, Some(0));
        assert_eq!(unranked.statistics_closed.hours_48[0].avg_price, 20.0);

        // recent_avg_price should work on the view
        assert_eq!(unranked.recent_avg_price(), Some(25.0));
    }

    #[test]
    fn test_max_rank_stats_non_mod_item() {
        let stats = ItemStatistics {
            statistics_closed: TimeframedStatistics {
                hours_48: vec![make_closed_entry(10, 50.0)],
                days_90: vec![],
            },
            statistics_live: TimeframedStatistics {
                hours_48: vec![],
                days_90: vec![],
            },
        };

        let result = stats.max_rank_stats();
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_str = format!("{}", err);
        assert!(err_str.contains("mod item"));
    }

    #[test]
    fn test_unranked_stats_non_mod_item() {
        let stats = ItemStatistics {
            statistics_closed: TimeframedStatistics {
                hours_48: vec![make_closed_entry(10, 50.0)],
                days_90: vec![],
            },
            statistics_live: TimeframedStatistics {
                hours_48: vec![],
                days_90: vec![],
            },
        };

        let result = stats.unranked_stats();
        assert!(result.is_err());
    }

    #[test]
    fn test_view_helper_methods() {
        let stats = make_mod_item_statistics();
        let rank10 = stats.stats_for_rank(10).unwrap();

        // Test TimeframedStatisticsView methods
        assert_eq!(
            rank10.statistics_closed.latest_hourly().unwrap().avg_price,
            100.0
        );
        assert_eq!(
            rank10.statistics_closed.latest_daily().unwrap().avg_price,
            110.0
        );
        assert_eq!(rank10.statistics_closed.avg_price_48h(), Some(100.0));
        assert_eq!(rank10.statistics_closed.avg_price_90d(), Some(110.0));
        assert_eq!(rank10.statistics_closed.total_volume_48h(), 3);
        assert_eq!(rank10.statistics_closed.total_volume_90d(), 30);

        // Test ItemStatisticsView methods
        assert_eq!(rank10.recent_avg_price(), Some(110.0));
        assert_eq!(rank10.recent_median_price(), Some(110.0));
    }

    #[test]
    fn test_view_has_sufficient_data() {
        // Create mod stats with enough days of data for rank 10
        let mut days_90 = Vec::new();
        for i in 0..10 {
            days_90.push(make_closed_entry_with_rank(
                if i < 7 { 10 } else { 0 },
                100.0,
                10,
            ));
            days_90.push(make_closed_entry_with_rank(
                if i < 3 { 5 } else { 0 },
                20.0,
                0,
            ));
        }

        let stats = ItemStatistics {
            statistics_closed: TimeframedStatistics {
                hours_48: vec![],
                days_90,
            },
            statistics_live: TimeframedStatistics {
                hours_48: vec![],
                days_90: vec![],
            },
        };

        let rank10 = stats.stats_for_rank(10).unwrap();
        assert!(rank10.has_sufficient_data()); // 7 days with trades

        let rank0 = stats.stats_for_rank(0).unwrap();
        assert!(!rank0.has_sufficient_data()); // Only 3 days with trades
    }

    #[test]
    fn test_timeframed_statistics_available_mod_ranks() {
        let stats = TimeframedStatistics {
            hours_48: vec![
                make_closed_entry_with_rank(5, 20.0, 0),
                make_closed_entry_with_rank(3, 100.0, 10),
            ],
            days_90: vec![
                make_closed_entry_with_rank(50, 25.0, 0),
                make_closed_entry_with_rank(30, 110.0, 5), // Different rank in days_90
            ],
        };

        let ranks = stats.available_mod_ranks();
        assert_eq!(ranks, vec![0, 5, 10]);
    }
}
