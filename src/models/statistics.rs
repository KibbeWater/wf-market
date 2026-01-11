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
/// - `open_price` / `closed_price`: First and last trade prices in the period
/// - `min_price` / `max_price`: Price range for the period
/// - `avg_price`: Simple average of all trades
/// - `wa_price`: Volume-weighted average price
/// - `median`: Median price
/// - `moving_avg`: Moving average (smoothed trend)
///
/// # Technical Indicators
///
/// - `donch_top` / `donch_bot`: Donchian channel bounds (highest high / lowest low)
#[derive(Debug, Clone, Deserialize)]
pub struct StatisticEntry {
    /// Unique identifier for this data point.
    pub id: String,

    /// Timestamp for this data point.
    ///
    /// For 48-hour data: hourly intervals.
    /// For 90-day data: daily intervals.
    pub datetime: DateTime<Utc>,

    /// Number of trades in this period.
    pub volume: i32,

    /// Lowest price in this period.
    pub min_price: i32,

    /// Highest price in this period.
    pub max_price: i32,

    /// First trade price in this period (open).
    pub open_price: i32,

    /// Last trade price in this period (close).
    pub closed_price: i32,

    /// Simple average price.
    pub avg_price: f64,

    /// Volume-weighted average price.
    pub wa_price: f64,

    /// Median price.
    pub median: f64,

    /// Moving average price.
    pub moving_avg: f64,

    /// Donchian channel top (highest high over lookback period).
    pub donch_top: i32,

    /// Donchian channel bottom (lowest low over lookback period).
    pub donch_bot: i32,
}

impl StatisticEntry {
    /// Returns `true` if no trades occurred in this period.
    pub fn is_empty(&self) -> bool {
        self.volume == 0
    }

    /// Returns the price range (spread) for this period.
    pub fn price_range(&self) -> i32 {
        self.max_price - self.min_price
    }

    /// Returns the Donchian channel width.
    pub fn donchian_width(&self) -> i32 {
        self.donch_top - self.donch_bot
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
    pub fn latest_hourly(&self) -> Option<&StatisticEntry> {
        self.hours_48.last()
    }

    /// Get the most recent daily data point.
    pub fn latest_daily(&self) -> Option<&StatisticEntry> {
        self.days_90.last()
    }

    /// Calculate the average price over the last 48 hours.
    ///
    /// Returns `None` if there are no data points with volume.
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
    pub fn avg_price_90d(&self) -> Option<f64> {
        let entries: Vec<_> = self.days_90.iter().filter(|e| e.volume > 0).collect();
        if entries.is_empty() {
            return None;
        }
        let sum: f64 = entries.iter().map(|e| e.avg_price).sum();
        Some(sum / entries.len() as f64)
    }

    /// Calculate total volume over the last 48 hours.
    pub fn total_volume_48h(&self) -> i32 {
        self.hours_48.iter().map(|e| e.volume).sum()
    }

    /// Calculate total volume over the last 90 days.
    pub fn total_volume_90d(&self) -> i32 {
        self.days_90.iter().map(|e| e.volume).sum()
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
    pub fn recent_avg_price(&self) -> Option<f64> {
        self.statistics_closed
            .latest_daily()
            .filter(|e| e.volume > 0)
            .map(|e| e.avg_price)
    }

    /// Get the most recent median price from closed trades (daily).
    pub fn recent_median_price(&self) -> Option<f64> {
        self.statistics_closed
            .latest_daily()
            .filter(|e| e.volume > 0)
            .map(|e| e.median)
    }

    /// Check if there's enough trading activity for reliable statistics.
    ///
    /// Returns `true` if there were trades in at least 7 of the last 90 days.
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

    fn make_entry(volume: i32, avg_price: f64) -> StatisticEntry {
        StatisticEntry {
            id: "test".to_string(),
            datetime: Utc::now(),
            volume,
            min_price: 10,
            max_price: 20,
            open_price: 12,
            closed_price: 18,
            avg_price,
            wa_price: avg_price,
            median: avg_price,
            moving_avg: avg_price,
            donch_top: 25,
            donch_bot: 5,
        }
    }

    #[test]
    fn test_statistic_entry_is_empty() {
        let empty = make_entry(0, 0.0);
        assert!(empty.is_empty());

        let with_volume = make_entry(10, 15.0);
        assert!(!with_volume.is_empty());
    }

    #[test]
    fn test_statistic_entry_price_range() {
        let entry = make_entry(10, 15.0);
        assert_eq!(entry.price_range(), 10); // 20 - 10
    }

    #[test]
    fn test_statistic_entry_donchian_width() {
        let entry = make_entry(10, 15.0);
        assert_eq!(entry.donchian_width(), 20); // 25 - 5
    }

    #[test]
    fn test_timeframed_statistics_latest() {
        let stats = TimeframedStatistics {
            hours_48: vec![make_entry(5, 10.0), make_entry(10, 15.0)],
            days_90: vec![make_entry(100, 12.0), make_entry(150, 14.0)],
        };

        assert_eq!(stats.latest_hourly().unwrap().avg_price, 15.0);
        assert_eq!(stats.latest_daily().unwrap().avg_price, 14.0);
    }

    #[test]
    fn test_timeframed_statistics_avg_price() {
        let stats = TimeframedStatistics {
            hours_48: vec![make_entry(5, 10.0), make_entry(10, 20.0)],
            days_90: vec![make_entry(0, 0.0), make_entry(100, 15.0)], // first has no volume
        };

        assert_eq!(stats.avg_price_48h(), Some(15.0)); // (10 + 20) / 2
        assert_eq!(stats.avg_price_90d(), Some(15.0)); // only entry with volume
    }

    #[test]
    fn test_timeframed_statistics_total_volume() {
        let stats = TimeframedStatistics {
            hours_48: vec![make_entry(5, 10.0), make_entry(10, 15.0)],
            days_90: vec![make_entry(100, 12.0), make_entry(150, 14.0)],
        };

        assert_eq!(stats.total_volume_48h(), 15);
        assert_eq!(stats.total_volume_90d(), 250);
    }

    #[test]
    fn test_item_statistics_recent_price() {
        let closed = TimeframedStatistics {
            hours_48: vec![make_entry(5, 10.0)],
            days_90: vec![make_entry(100, 15.0)],
        };
        let live = TimeframedStatistics {
            hours_48: vec![make_entry(20, 18.0)],
            days_90: vec![make_entry(50, 16.0)],
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
            entries.push(make_entry(if i < 7 { 10 } else { 0 }, 15.0));
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
            make_entry(10, 15.0),
            make_entry(0, 0.0),
            make_entry(10, 15.0),
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
}
