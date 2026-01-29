//! OHLCV (Open, High, Low, Close, Volume) data types.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::Timeframe;

/// Compact OHLCV bar optimized for performance.
/// Uses f64 for fast indicator calculations.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Bar {
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
    /// Opening price
    pub open: f64,
    /// Highest price
    pub high: f64,
    /// Lowest price
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Trading volume
    pub volume: f64,
    /// Volume-weighted average price (optional)
    pub vwap: Option<f64>,
}

impl Bar {
    /// Create a new bar.
    pub fn new(timestamp: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            vwap: None,
        }
    }

    /// Create a new bar with VWAP.
    pub fn with_vwap(mut self, vwap: f64) -> Self {
        self.vwap = Some(vwap);
        self
    }

    /// Calculate the typical price (HLC average).
    #[inline]
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// Calculate the bar's range (high - low).
    #[inline]
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// Calculate the bar's body size (absolute difference between open and close).
    #[inline]
    pub fn body(&self) -> f64 {
        (self.close - self.open).abs()
    }

    /// Check if the bar is bullish (close > open).
    #[inline]
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Check if the bar is bearish (close < open).
    #[inline]
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Get the timestamp as a DateTime.
    pub fn datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self.timestamp)
            .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
    }

    /// Calculate the true range (used for ATR).
    pub fn true_range(&self, prev_close: Option<f64>) -> f64 {
        match prev_close {
            Some(pc) => {
                let hl = self.high - self.low;
                let hc = (self.high - pc).abs();
                let lc = (self.low - pc).abs();
                hl.max(hc).max(lc)
            }
            None => self.high - self.low,
        }
    }
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            timestamp: 0,
            open: 0.0,
            high: 0.0,
            low: 0.0,
            close: 0.0,
            volume: 0.0,
            vwap: None,
        }
    }
}

/// High-precision bar for order calculations.
/// Uses Decimal for exact arithmetic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreciseBar {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Opening price
    pub open: Decimal,
    /// Highest price
    pub high: Decimal,
    /// Lowest price
    pub low: Decimal,
    /// Closing price
    pub close: Decimal,
    /// Trading volume
    pub volume: Decimal,
}

impl From<Bar> for PreciseBar {
    fn from(bar: Bar) -> Self {
        Self {
            timestamp: bar.datetime(),
            open: Decimal::try_from(bar.open).unwrap_or_default(),
            high: Decimal::try_from(bar.high).unwrap_or_default(),
            low: Decimal::try_from(bar.low).unwrap_or_default(),
            close: Decimal::try_from(bar.close).unwrap_or_default(),
            volume: Decimal::try_from(bar.volume).unwrap_or_default(),
        }
    }
}

/// Time-series container for bars, optimized for sequential access.
#[derive(Debug, Clone)]
pub struct BarSeries {
    /// Symbol identifier
    pub symbol: String,
    /// Timeframe of the bars
    pub timeframe: Timeframe,
    /// Bars stored in a deque for efficient push/pop
    bars: VecDeque<Bar>,
    /// Maximum capacity (0 = unlimited)
    capacity: usize,
}

impl BarSeries {
    /// Create a new empty bar series.
    pub fn new(symbol: String, timeframe: Timeframe) -> Self {
        Self {
            symbol,
            timeframe,
            bars: VecDeque::new(),
            capacity: 0,
        }
    }

    /// Create a bar series with a maximum capacity.
    /// When capacity is reached, oldest bars are removed.
    pub fn with_capacity(symbol: String, timeframe: Timeframe, capacity: usize) -> Self {
        Self {
            symbol,
            timeframe,
            bars: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Push a new bar, removing the oldest if at capacity.
    pub fn push(&mut self, bar: Bar) {
        if self.capacity > 0 && self.bars.len() >= self.capacity {
            self.bars.pop_front();
        }
        self.bars.push_back(bar);
    }

    /// Push multiple bars.
    pub fn extend(&mut self, bars: impl IntoIterator<Item = Bar>) {
        for bar in bars {
            self.push(bar);
        }
    }

    /// Get the number of bars.
    #[inline]
    pub fn len(&self) -> usize {
        self.bars.len()
    }

    /// Check if the series is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }

    /// Get all bars as a slice.
    pub fn bars(&self) -> &VecDeque<Bar> {
        &self.bars
    }

    /// Get the last N bars.
    pub fn last_n(&self, n: usize) -> Vec<&Bar> {
        let start = self.bars.len().saturating_sub(n);
        self.bars.iter().skip(start).collect()
    }

    /// Get the last bar.
    pub fn last(&self) -> Option<&Bar> {
        self.bars.back()
    }

    /// Get a bar by index (0 = oldest).
    pub fn get(&self, index: usize) -> Option<&Bar> {
        self.bars.get(index)
    }

    /// Extract close prices as a vector.
    pub fn closes(&self) -> Vec<f64> {
        self.bars.iter().map(|b| b.close).collect()
    }

    /// Extract open prices as a vector.
    pub fn opens(&self) -> Vec<f64> {
        self.bars.iter().map(|b| b.open).collect()
    }

    /// Extract high prices as a vector.
    pub fn highs(&self) -> Vec<f64> {
        self.bars.iter().map(|b| b.high).collect()
    }

    /// Extract low prices as a vector.
    pub fn lows(&self) -> Vec<f64> {
        self.bars.iter().map(|b| b.low).collect()
    }

    /// Extract volumes as a vector.
    pub fn volumes(&self) -> Vec<f64> {
        self.bars.iter().map(|b| b.volume).collect()
    }

    /// Extract typical prices as a vector.
    pub fn typical_prices(&self) -> Vec<f64> {
        self.bars.iter().map(|b| b.typical_price()).collect()
    }

    /// Clear all bars.
    pub fn clear(&mut self) {
        self.bars.clear();
    }

    /// Get an iterator over the bars.
    pub fn iter(&self) -> impl Iterator<Item = &Bar> {
        self.bars.iter()
    }
}

impl FromIterator<Bar> for BarSeries {
    fn from_iter<T: IntoIterator<Item = Bar>>(iter: T) -> Self {
        let bars: VecDeque<Bar> = iter.into_iter().collect();
        Self {
            symbol: String::new(),
            timeframe: Timeframe::Daily,
            bars,
            capacity: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_calculations() {
        let bar = Bar::new(1000, 100.0, 110.0, 95.0, 105.0, 1000000.0);

        assert!((bar.typical_price() - 103.333333).abs() < 0.001);
        assert!((bar.range() - 15.0).abs() < 0.001);
        assert!((bar.body() - 5.0).abs() < 0.001);
        assert!(bar.is_bullish());
        assert!(!bar.is_bearish());
    }

    #[test]
    fn test_bar_true_range() {
        let bar = Bar::new(1000, 100.0, 110.0, 95.0, 105.0, 1000000.0);

        // Without previous close
        assert!((bar.true_range(None) - 15.0).abs() < 0.001);

        // With previous close that creates gap
        assert!((bar.true_range(Some(90.0)) - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_bar_series_capacity() {
        let mut series = BarSeries::with_capacity("AAPL".to_string(), Timeframe::Daily, 3);

        series.push(Bar::new(1, 100.0, 101.0, 99.0, 100.5, 1000.0));
        series.push(Bar::new(2, 100.5, 102.0, 100.0, 101.5, 1000.0));
        series.push(Bar::new(3, 101.5, 103.0, 101.0, 102.5, 1000.0));
        assert_eq!(series.len(), 3);

        // Should remove oldest when at capacity
        series.push(Bar::new(4, 102.5, 104.0, 102.0, 103.5, 1000.0));
        assert_eq!(series.len(), 3);
        assert_eq!(series.get(0).unwrap().timestamp, 2);
    }

    #[test]
    fn test_bar_series_extractions() {
        let mut series = BarSeries::new("AAPL".to_string(), Timeframe::Daily);
        series.push(Bar::new(1, 100.0, 101.0, 99.0, 100.5, 1000.0));
        series.push(Bar::new(2, 100.5, 102.0, 100.0, 101.5, 2000.0));

        let closes = series.closes();
        assert_eq!(closes, vec![100.5, 101.5]);

        let volumes = series.volumes();
        assert_eq!(volumes, vec![1000.0, 2000.0]);
    }
}
