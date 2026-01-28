//! CSV data source.

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use csv::ReaderBuilder;
use serde::Deserialize;
use std::path::Path;
use trading_core::error::DataError;
use trading_core::types::{Bar, Timeframe};

/// CSV record format.
#[derive(Debug, Deserialize)]
struct CsvRecord {
    #[serde(alias = "Date", alias = "date", alias = "timestamp", alias = "Timestamp")]
    date: String,
    #[serde(alias = "Open", alias = "open")]
    open: f64,
    #[serde(alias = "High", alias = "high")]
    high: f64,
    #[serde(alias = "Low", alias = "low")]
    low: f64,
    #[serde(alias = "Close", alias = "close", alias = "Adj Close")]
    close: f64,
    #[serde(alias = "Volume", alias = "volume", default)]
    volume: f64,
}

/// CSV data source for historical data.
pub struct CsvDataSource {
    path: String,
}

impl CsvDataSource {
    /// Create a new CSV data source.
    pub fn new(path: &str) -> Result<Self, DataError> {
        if !Path::new(path).exists() {
            return Err(DataError::NoDataAvailable);
        }
        Ok(Self {
            path: path.to_string(),
        })
    }

    /// Load all bars from the CSV file.
    pub async fn load_all(&self, _symbol: &str, _timeframe: Timeframe) -> Result<Vec<Bar>, DataError> {
        self.load_from_path(&self.path)
    }

    /// Load bars from a specific path.
    fn load_from_path(&self, path: &str) -> Result<Vec<Bar>, DataError> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_path(path)
            .map_err(|e| DataError::ParseError(e.to_string()))?;

        let mut bars = Vec::new();

        for result in reader.deserialize() {
            let record: CsvRecord = result.map_err(|e| DataError::ParseError(e.to_string()))?;

            let timestamp = self.parse_timestamp(&record.date)?;

            bars.push(Bar::new(
                timestamp,
                record.open,
                record.high,
                record.low,
                record.close,
                record.volume,
            ));
        }

        // Sort by timestamp
        bars.sort_by_key(|b| b.timestamp);

        Ok(bars)
    }

    /// Parse various timestamp formats.
    fn parse_timestamp(&self, date_str: &str) -> Result<i64, DataError> {
        // Try various formats
        let formats = [
            "%Y-%m-%d",
            "%Y-%m-%d %H:%M:%S",
            "%Y/%m/%d",
            "%m/%d/%Y",
            "%d-%m-%Y",
        ];

        for format in formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, format) {
                return Ok(dt.and_utc().timestamp_millis());
            }
            if let Ok(d) = NaiveDate::parse_from_str(date_str, format) {
                let dt = d.and_hms_opt(0, 0, 0).unwrap();
                return Ok(dt.and_utc().timestamp_millis());
            }
        }

        // Try parsing as Unix timestamp
        if let Ok(ts) = date_str.parse::<i64>() {
            // Assume milliseconds if > 10 digits
            if ts > 10_000_000_000 {
                return Ok(ts);
            } else {
                return Ok(ts * 1000);
            }
        }

        Err(DataError::ParseError(format!(
            "Could not parse date: {}",
            date_str
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp() {
        let source = CsvDataSource {
            path: String::new(),
        };

        // Test various formats
        assert!(source.parse_timestamp("2024-01-15").is_ok());
        assert!(source.parse_timestamp("2024-01-15 10:30:00").is_ok());
        assert!(source.parse_timestamp("1705312800000").is_ok()); // Unix ms
        assert!(source.parse_timestamp("1705312800").is_ok()); // Unix sec
    }
}
