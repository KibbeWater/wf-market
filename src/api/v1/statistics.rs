//! Item statistics endpoint (V1 API).
//!
//! This module implements the `/v1/items/{slug}/statistics` endpoint
//! for retrieving historical trading statistics.

use crate::client::{AuthState, Client};
use crate::error::{ApiErrorResponse, Error, Result};
use crate::internal::V1_API_URL;
use crate::models::ItemStatistics;

use super::V1ApiResponse;

impl<S: AuthState> Client<S> {
    /// Get trading statistics for an item.
    ///
    /// Returns historical price and volume statistics for the specified item,
    /// including both completed trades and live order data.
    ///
    /// # Data Timeframes
    ///
    /// - **48 hours**: Hourly data points (`statistics.hours_48`)
    /// - **90 days**: Daily data points (`statistics.days_90`)
    ///
    /// # Statistics Types
    ///
    /// - `statistics_closed`: Data from completed/closed trades (actual prices paid)
    /// - `statistics_live`: Data from live/pending orders (current asking/bidding prices)
    ///
    /// # Price Indicators
    ///
    /// Each data point includes:
    /// - OHLC-style pricing (open, high, low, close)
    /// - Various averages (simple, weighted, moving)
    /// - Median price
    /// - Donchian channel bounds
    ///
    /// # Note
    ///
    /// This endpoint uses the **V1 API** which is deprecated. It will be removed
    /// when a V2 equivalent becomes available.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use wf_market::Client;
    ///
    /// async fn example() -> wf_market::Result<()> {
    ///     let client = Client::builder().build().await?;
    ///     let stats = client.get_item_statistics("nikana_prime_set").await?;
    ///
    ///     // Get recent price from closed trades
    ///     if let Some(price) = stats.recent_avg_price() {
    ///         println!("Recent average price: {:.0}p", price);
    ///     }
    ///
    ///     // Analyze 48-hour trend
    ///     for entry in &stats.statistics_closed.hours_48 {
    ///         if entry.volume > 0 {
    ///             println!("{}: {:.0}p ({} trades)",
    ///                 entry.datetime.format("%H:%M"),
    ///                 entry.avg_price,
    ///                 entry.volume
    ///             );
    ///         }
    ///     }
    ///
    ///     // Check 90-day volume
    ///     let total_volume = stats.statistics_closed.total_volume_90d();
    ///     println!("Total trades (90d): {}", total_volume);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_item_statistics(&self, slug: &str) -> Result<ItemStatistics> {
        self.wait_for_rate_limit().await;

        let response = self
            .http
            .get(format!("{}/items/{}/statistics", V1_API_URL, slug))
            .send()
            .await
            .map_err(Error::Network)?;

        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(Error::not_found(format!("Item not found: {}", slug)));
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();

            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                return Err(Error::api_with_response(
                    status,
                    format!("Failed to fetch statistics for item: {}", slug),
                    error_response,
                ));
            }

            return Err(Error::api(
                status,
                format!("Failed to fetch statistics for {}: {}", slug, body),
            ));
        }

        let body = response.text().await.map_err(Error::Network)?;

        let api_response: V1ApiResponse<ItemStatistics> =
            serde_json::from_str(&body).map_err(|e| Error::parse_with_body(e.to_string(), body))?;

        Ok(api_response.payload)
    }
}
