//! Data source trait definitions.

use crate::error::DataError;
use crate::types::{Bar, Timeframe};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// A real-time quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// Symbol
    pub symbol: String,
    /// Best bid price
    pub bid: f64,
    /// Best ask price
    pub ask: f64,
    /// Bid size
    pub bid_size: f64,
    /// Ask size
    pub ask_size: f64,
    /// Timestamp (Unix milliseconds)
    pub timestamp: i64,
}

impl Quote {
    /// Get the mid price.
    pub fn mid(&self) -> f64 {
        (self.bid + self.ask) / 2.0
    }

    /// Get the spread.
    pub fn spread(&self) -> f64 {
        self.ask - self.bid
    }

    /// Get the spread as a percentage of the mid price.
    pub fn spread_percent(&self) -> f64 {
        let mid = self.mid();
        if mid == 0.0 {
            0.0
        } else {
            (self.spread() / mid) * 100.0
        }
    }
}

/// Trait for historical data sources.
#[async_trait]
pub trait DataSource: Send + Sync {
    /// Fetch historical bars.
    ///
    /// # Arguments
    /// * `symbol` - The symbol to fetch
    /// * `timeframe` - The bar timeframe
    /// * `start` - Start of the date range
    /// * `end` - End of the date range
    ///
    /// # Returns
    /// A vector of bars ordered from oldest to newest
    async fn get_historical_bars(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Bar>, DataError>;

    /// Subscribe to real-time bar updates.
    ///
    /// # Arguments
    /// * `symbols` - Symbols to subscribe to
    /// * `timeframe` - The bar timeframe
    ///
    /// # Returns
    /// A channel receiver that will receive (symbol, bar) tuples
    async fn subscribe_bars(
        &self,
        symbols: &[String],
        timeframe: Timeframe,
    ) -> Result<mpsc::Receiver<(String, Bar)>, DataError>;

    /// Unsubscribe from bar updates.
    async fn unsubscribe(&self, symbols: &[String]) -> Result<(), DataError>;

    /// Get the latest bar for a symbol.
    async fn get_latest_bar(
        &self,
        symbol: &str,
        timeframe: Timeframe,
    ) -> Result<Option<Bar>, DataError>;

    /// Check if a symbol is valid/tradeable.
    async fn is_valid_symbol(&self, symbol: &str) -> Result<bool, DataError>;

    /// Get the data source name.
    fn name(&self) -> &str;
}

/// Trait for real-time quote sources.
#[async_trait]
pub trait QuoteSource: Send + Sync {
    /// Subscribe to real-time quotes.
    ///
    /// # Arguments
    /// * `symbols` - Symbols to subscribe to
    ///
    /// # Returns
    /// A channel receiver that will receive quotes
    async fn subscribe_quotes(
        &self,
        symbols: &[String],
    ) -> Result<mpsc::Receiver<Quote>, DataError>;

    /// Unsubscribe from quote updates.
    async fn unsubscribe_quotes(&self, symbols: &[String]) -> Result<(), DataError>;

    /// Get the latest quote for a symbol.
    async fn get_latest_quote(&self, symbol: &str) -> Result<Option<Quote>, DataError>;

    /// Get the source name.
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_calculations() {
        let quote = Quote {
            symbol: "AAPL".to_string(),
            bid: 149.95,
            ask: 150.05,
            bid_size: 100.0,
            ask_size: 200.0,
            timestamp: 1000,
        };

        assert!((quote.mid() - 150.0).abs() < 0.001);
        assert!((quote.spread() - 0.10).abs() < 0.001);
        assert!((quote.spread_percent() - 0.0667).abs() < 0.01);
    }
}
