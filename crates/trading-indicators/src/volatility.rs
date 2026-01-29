//! Volatility indicators.

use serde::{Deserialize, Serialize};
use trading_core::traits::{Indicator, MultiOutputIndicator};

/// Standard Deviation.
#[derive(Debug, Clone)]
pub struct StdDev {
    period: usize,
}

impl StdDev {
    /// Create a new standard deviation indicator.
    pub fn new(period: usize) -> Self {
        assert!(period > 1, "Period must be greater than 1");
        Self { period }
    }
}

impl Indicator for StdDev {
    type Output = f64;

    fn calculate(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < self.period {
            return vec![];
        }

        let period_f64 = self.period as f64;
        let mut result = Vec::with_capacity(data.len() - self.period + 1);

        for window in data.windows(self.period) {
            let mean: f64 = window.iter().sum::<f64>() / period_f64;
            let variance: f64 = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period_f64;
            result.push(variance.sqrt());
        }

        result
    }

    fn period(&self) -> usize {
        self.period
    }

    fn name(&self) -> &str {
        "StdDev"
    }
}

/// Average True Range (ATR).
///
/// Measures market volatility by decomposing the entire range
/// of an asset price for that period.
#[derive(Debug, Clone)]
pub struct Atr {
    period: usize,
}

impl Atr {
    /// Create a new ATR indicator.
    ///
    /// Common period is 14.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self { period }
    }

    /// Calculate ATR from OHLC data.
    pub fn calculate_ohlc(&self, high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> {
        let len = high.len().min(low.len()).min(close.len());
        if len < self.period + 1 {
            return vec![];
        }

        // Calculate True Range
        let mut tr = Vec::with_capacity(len - 1);

        for i in 1..len {
            let high_low = high[i] - low[i];
            let high_close = (high[i] - close[i - 1]).abs();
            let low_close = (low[i] - close[i - 1]).abs();
            tr.push(high_low.max(high_close).max(low_close));
        }

        if tr.len() < self.period {
            return vec![];
        }

        // Calculate ATR using Wilder's smoothing
        let period_f64 = self.period as f64;
        let mut result = Vec::with_capacity(tr.len() - self.period + 1);

        // Initial ATR is SMA of first 'period' true ranges
        let mut atr: f64 = tr[..self.period].iter().sum::<f64>() / period_f64;
        result.push(atr);

        // Wilder's smoothing
        for &tr_val in &tr[self.period..] {
            atr = (atr * (period_f64 - 1.0) + tr_val) / period_f64;
            result.push(atr);
        }

        result
    }
}

impl Indicator for Atr {
    type Output = f64;

    /// Calculate using close prices only (approximation using range).
    fn calculate(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < self.period + 1 {
            return vec![];
        }

        // Use close-to-close changes as approximation of true range
        let mut tr = Vec::with_capacity(data.len() - 1);
        for i in 1..data.len() {
            tr.push((data[i] - data[i - 1]).abs());
        }

        // Calculate ATR
        let period_f64 = self.period as f64;
        let mut result = Vec::with_capacity(tr.len() - self.period + 1);

        let mut atr: f64 = tr[..self.period].iter().sum::<f64>() / period_f64;
        result.push(atr);

        for &tr_val in &tr[self.period..] {
            atr = (atr * (period_f64 - 1.0) + tr_val) / period_f64;
            result.push(atr);
        }

        result
    }

    fn period(&self) -> usize {
        self.period + 1
    }

    fn name(&self) -> &str {
        "ATR"
    }
}

/// Bollinger Bands output.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BollingerOutput {
    /// Upper band
    pub upper: f64,
    /// Middle band (SMA)
    pub middle: f64,
    /// Lower band
    pub lower: f64,
    /// Bandwidth ((upper - lower) / middle)
    pub bandwidth: f64,
    /// %B ((price - lower) / (upper - lower))
    pub percent_b: f64,
}

impl BollingerOutput {
    /// Check if price is above upper band.
    pub fn is_overbought(&self, price: f64) -> bool {
        price > self.upper
    }

    /// Check if price is below lower band.
    pub fn is_oversold(&self, price: f64) -> bool {
        price < self.lower
    }
}

/// Bollinger Bands.
///
/// Consists of a middle band (SMA) with upper and lower bands
/// at a specified number of standard deviations.
#[derive(Debug, Clone)]
pub struct BollingerBands {
    period: usize,
    std_dev_multiplier: f64,
}

impl BollingerBands {
    /// Create new Bollinger Bands with default parameters (20, 2.0).
    pub fn new() -> Self {
        Self::with_params(20, 2.0)
    }

    /// Create Bollinger Bands with custom parameters.
    pub fn with_params(period: usize, std_dev_multiplier: f64) -> Self {
        assert!(period > 1, "Period must be greater than 1");
        assert!(
            std_dev_multiplier > 0.0,
            "Std dev multiplier must be positive"
        );
        Self {
            period,
            std_dev_multiplier,
        }
    }
}

impl Default for BollingerBands {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiOutputIndicator for BollingerBands {
    type Outputs = BollingerOutput;

    fn calculate(&self, data: &[f64]) -> Vec<BollingerOutput> {
        if data.len() < self.period {
            return vec![];
        }

        let period_f64 = self.period as f64;
        let mut result = Vec::with_capacity(data.len() - self.period + 1);

        for (i, window) in data.windows(self.period).enumerate() {
            let mean: f64 = window.iter().sum::<f64>() / period_f64;
            let variance: f64 = window.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / period_f64;
            let std_dev = variance.sqrt();

            let upper = mean + self.std_dev_multiplier * std_dev;
            let lower = mean - self.std_dev_multiplier * std_dev;

            let bandwidth = if mean != 0.0 {
                (upper - lower) / mean
            } else {
                0.0
            };

            let price = data[self.period - 1 + i];
            let percent_b = if upper != lower {
                (price - lower) / (upper - lower)
            } else {
                0.5
            };

            result.push(BollingerOutput {
                upper,
                middle: mean,
                lower,
                bandwidth,
                percent_b,
            });
        }

        result
    }

    fn period(&self) -> usize {
        self.period
    }

    fn name(&self) -> &str {
        "Bollinger Bands"
    }
}

/// Keltner Channels.
///
/// Similar to Bollinger Bands but uses ATR instead of standard deviation.
#[derive(Debug, Clone)]
pub struct KeltnerChannels {
    ema_period: usize,
    atr_period: usize,
    atr_multiplier: f64,
}

impl KeltnerChannels {
    /// Create new Keltner Channels with default parameters (20, 10, 2.0).
    pub fn new() -> Self {
        Self::with_params(20, 10, 2.0)
    }

    /// Create Keltner Channels with custom parameters.
    pub fn with_params(ema_period: usize, atr_period: usize, atr_multiplier: f64) -> Self {
        assert!(ema_period > 0 && atr_period > 0);
        assert!(atr_multiplier > 0.0);
        Self {
            ema_period,
            atr_period,
            atr_multiplier,
        }
    }

    /// Calculate from OHLC data.
    pub fn calculate_ohlc(&self, high: &[f64], low: &[f64], close: &[f64]) -> Vec<BollingerOutput> {
        let atr = Atr::new(self.atr_period);
        let atr_values = atr.calculate_ohlc(high, low, close);

        if atr_values.is_empty() || close.len() < self.ema_period {
            return vec![];
        }

        // Calculate EMA
        let multiplier = 2.0 / (self.ema_period as f64 + 1.0);
        let mut ema_values = Vec::with_capacity(close.len() - self.ema_period + 1);

        let initial_sma: f64 =
            close[..self.ema_period].iter().sum::<f64>() / self.ema_period as f64;
        ema_values.push(initial_sma);

        let mut ema = initial_sma;
        for &price in &close[self.ema_period..] {
            ema = price * multiplier + ema * (1.0 - multiplier);
            ema_values.push(ema);
        }

        // Align EMA and ATR
        let offset = self.atr_period.saturating_sub(self.ema_period);

        let ema_slice = if offset > 0 && offset < ema_values.len() {
            &ema_values[offset..]
        } else {
            &ema_values
        };

        let min_len = ema_slice.len().min(atr_values.len());
        let ema_offset = ema_slice.len() - min_len;
        let atr_offset = atr_values.len() - min_len;

        let mut result = Vec::with_capacity(min_len);

        for i in 0..min_len {
            let ema_val = ema_slice[ema_offset + i];
            let atr_val = atr_values[atr_offset + i];
            let band_width = self.atr_multiplier * atr_val;

            let upper = ema_val + band_width;
            let lower = ema_val - band_width;

            let price_idx = close.len() - min_len + i;
            let price = close[price_idx];

            let bandwidth = if ema_val != 0.0 {
                (upper - lower) / ema_val
            } else {
                0.0
            };

            let percent_b = if upper != lower {
                (price - lower) / (upper - lower)
            } else {
                0.5
            };

            result.push(BollingerOutput {
                upper,
                middle: ema_val,
                lower,
                bandwidth,
                percent_b,
            });
        }

        result
    }
}

impl Default for KeltnerChannels {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_std_dev() {
        let std_dev = StdDev::new(3);
        let data = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let result = std_dev.calculate(&data);

        assert_eq!(result.len(), 3);
        // First window: [2, 4, 6], mean = 4, variance = (4+0+4)/3 = 8/3
        // std_dev = sqrt(8/3) ≈ 1.633
        assert!((result[0] - 1.633).abs() < 0.01);
    }

    #[test]
    fn test_atr_ohlc() {
        let atr = Atr::new(3);
        let high = vec![10.0, 11.0, 12.0, 11.0, 13.0, 14.0];
        let low = vec![8.0, 9.0, 10.0, 9.0, 11.0, 12.0];
        let close = vec![9.0, 10.0, 11.0, 10.0, 12.0, 13.0];

        let result = atr.calculate_ohlc(&high, &low, &close);
        assert!(!result.is_empty());

        // All ATR values should be positive
        for value in &result {
            assert!(*value > 0.0);
        }
    }

    #[test]
    fn test_bollinger_bands() {
        let bb = BollingerBands::new();
        let data: Vec<f64> = (0..30)
            .map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0)
            .collect();

        let result = bb.calculate(&data);
        assert!(!result.is_empty());

        for output in &result {
            // Upper > Middle > Lower
            assert!(output.upper > output.middle);
            assert!(output.middle > output.lower);
            // Bandwidth should be positive
            assert!(output.bandwidth > 0.0);
        }
    }

    #[test]
    fn test_bollinger_percent_b() {
        let bb = BollingerBands::with_params(5, 2.0);
        let data = vec![100.0, 100.0, 100.0, 100.0, 100.0]; // Constant price

        let result = bb.calculate(&data);
        assert_eq!(result.len(), 1);

        // With constant price, bands collapse, percent_b = 0.5
        assert!((result[0].percent_b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_bollinger_overbought_oversold() {
        let output = BollingerOutput {
            upper: 110.0,
            middle: 100.0,
            lower: 90.0,
            bandwidth: 0.2,
            percent_b: 0.5,
        };

        assert!(output.is_overbought(115.0));
        assert!(!output.is_overbought(105.0));
        assert!(output.is_oversold(85.0));
        assert!(!output.is_oversold(95.0));
    }
}
