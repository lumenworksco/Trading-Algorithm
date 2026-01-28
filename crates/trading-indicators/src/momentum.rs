//! Momentum indicators.

use trading_core::traits::{Indicator, MultiOutputIndicator};
use serde::{Deserialize, Serialize};

/// Relative Strength Index (RSI).
///
/// Measures the speed and magnitude of recent price changes
/// to evaluate overbought or oversold conditions.
#[derive(Debug, Clone)]
pub struct Rsi {
    period: usize,
}

impl Rsi {
    /// Create a new RSI indicator.
    ///
    /// Common periods are 14 (default) or 9.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self { period }
    }

    /// Calculate using Wilder's smoothing method.
    fn wilder_smooth(values: &[f64], period: usize) -> Vec<f64> {
        if values.len() < period {
            return vec![];
        }

        let mut result = Vec::with_capacity(values.len() - period + 1);
        let period_f64 = period as f64;

        // Initial average
        let mut avg: f64 = values[..period].iter().sum::<f64>() / period_f64;
        result.push(avg);

        // Wilder's smoothing: avg = (prev_avg * (period-1) + value) / period
        for &value in &values[period..] {
            avg = (avg * (period_f64 - 1.0) + value) / period_f64;
            result.push(avg);
        }

        result
    }
}

impl Indicator for Rsi {
    type Output = f64;

    fn calculate(&self, data: &[f64]) -> Vec<f64> {
        if data.len() <= self.period {
            return vec![];
        }

        // Calculate price changes
        let mut gains = Vec::with_capacity(data.len() - 1);
        let mut losses = Vec::with_capacity(data.len() - 1);

        for i in 1..data.len() {
            let change = data[i] - data[i - 1];
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(-change);
            }
        }

        // Smooth gains and losses
        let avg_gains = Self::wilder_smooth(&gains, self.period);
        let avg_losses = Self::wilder_smooth(&losses, self.period);

        // Calculate RSI
        avg_gains
            .iter()
            .zip(avg_losses.iter())
            .map(|(&gain, &loss)| {
                if loss == 0.0 {
                    100.0
                } else {
                    100.0 - (100.0 / (1.0 + gain / loss))
                }
            })
            .collect()
    }

    fn period(&self) -> usize {
        self.period + 1 // Need period+1 data points
    }

    fn name(&self) -> &str {
        "RSI"
    }
}

/// MACD (Moving Average Convergence Divergence) output.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MacdOutput {
    /// MACD line (fast EMA - slow EMA)
    pub macd: f64,
    /// Signal line (EMA of MACD)
    pub signal: f64,
    /// Histogram (MACD - Signal)
    pub histogram: f64,
}

/// MACD indicator.
///
/// Uses two EMAs to identify trend direction and momentum.
#[derive(Debug, Clone)]
pub struct Macd {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
}

impl Macd {
    /// Create a new MACD with default parameters (12, 26, 9).
    pub fn new() -> Self {
        Self::with_periods(12, 26, 9)
    }

    /// Create a MACD with custom periods.
    pub fn with_periods(fast: usize, slow: usize, signal: usize) -> Self {
        assert!(fast > 0 && slow > 0 && signal > 0);
        assert!(fast < slow, "Fast period must be less than slow period");
        Self {
            fast_period: fast,
            slow_period: slow,
            signal_period: signal,
        }
    }

    fn calculate_ema(data: &[f64], period: usize) -> Vec<f64> {
        if data.len() < period {
            return vec![];
        }

        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut result = Vec::with_capacity(data.len() - period + 1);

        // Initial SMA
        let sma: f64 = data[..period].iter().sum::<f64>() / period as f64;
        result.push(sma);

        // EMA
        let mut ema = sma;
        for &price in &data[period..] {
            ema = price * multiplier + ema * (1.0 - multiplier);
            result.push(ema);
        }

        result
    }
}

impl Default for Macd {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiOutputIndicator for Macd {
    type Outputs = MacdOutput;

    fn calculate(&self, data: &[f64]) -> Vec<MacdOutput> {
        if data.len() < self.slow_period + self.signal_period {
            return vec![];
        }

        // Calculate EMAs
        let fast_ema = Self::calculate_ema(data, self.fast_period);
        let slow_ema = Self::calculate_ema(data, self.slow_period);

        // Align the EMAs (fast has more values)
        let offset = self.slow_period - self.fast_period;
        let fast_ema = &fast_ema[offset..];

        // Calculate MACD line
        let macd_line: Vec<f64> = fast_ema
            .iter()
            .zip(slow_ema.iter())
            .map(|(f, s)| f - s)
            .collect();

        if macd_line.len() < self.signal_period {
            return vec![];
        }

        // Calculate signal line (EMA of MACD)
        let signal_line = Self::calculate_ema(&macd_line, self.signal_period);

        // Align and create output
        let offset = self.signal_period - 1;
        macd_line[offset..]
            .iter()
            .zip(signal_line.iter())
            .map(|(&macd, &signal)| MacdOutput {
                macd,
                signal,
                histogram: macd - signal,
            })
            .collect()
    }

    fn period(&self) -> usize {
        self.slow_period + self.signal_period
    }

    fn name(&self) -> &str {
        "MACD"
    }
}

/// Stochastic oscillator output.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StochasticOutput {
    /// %K (fast stochastic)
    pub k: f64,
    /// %D (slow stochastic / signal)
    pub d: f64,
}

/// Stochastic oscillator.
///
/// Compares closing price to the price range over a period.
#[derive(Debug, Clone)]
pub struct Stochastic {
    k_period: usize,
    d_period: usize,
}

impl Stochastic {
    /// Create a new stochastic oscillator with default parameters (14, 3).
    pub fn new() -> Self {
        Self::with_periods(14, 3)
    }

    /// Create with custom periods.
    pub fn with_periods(k_period: usize, d_period: usize) -> Self {
        assert!(k_period > 0 && d_period > 0);
        Self { k_period, d_period }
    }

    /// Calculate stochastic from OHLC data.
    pub fn calculate_ohlc(
        &self,
        high: &[f64],
        low: &[f64],
        close: &[f64],
    ) -> Vec<StochasticOutput> {
        let len = high.len().min(low.len()).min(close.len());
        if len < self.k_period + self.d_period - 1 {
            return vec![];
        }

        // Calculate raw %K values
        let mut k_values = Vec::with_capacity(len - self.k_period + 1);

        for i in (self.k_period - 1)..len {
            let start = i + 1 - self.k_period;
            let highest = high[start..=i]
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max);
            let lowest = low[start..=i]
                .iter()
                .cloned()
                .fold(f64::INFINITY, f64::min);

            let range = highest - lowest;
            let k = if range == 0.0 {
                50.0 // Undefined, use midpoint
            } else {
                ((close[i] - lowest) / range) * 100.0
            };
            k_values.push(k);
        }

        if k_values.len() < self.d_period {
            return vec![];
        }

        // Calculate %D (SMA of %K)
        let mut result = Vec::with_capacity(k_values.len() - self.d_period + 1);
        let d_period_f64 = self.d_period as f64;

        for i in (self.d_period - 1)..k_values.len() {
            let k = k_values[i];
            let d: f64 = k_values[(i + 1 - self.d_period)..=i].iter().sum::<f64>() / d_period_f64;
            result.push(StochasticOutput { k, d });
        }

        result
    }
}

impl Default for Stochastic {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for Stochastic {
    type Output = StochasticOutput;

    /// Calculate using close prices only (uses close as high/low approximation).
    fn calculate(&self, data: &[f64]) -> Vec<StochasticOutput> {
        // For close-only data, we use close as both high and low
        // This is a simplified version; prefer calculate_ohlc for accurate results
        self.calculate_ohlc(data, data, data)
    }

    fn period(&self) -> usize {
        self.k_period + self.d_period - 1
    }

    fn name(&self) -> &str {
        "Stochastic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_basic() {
        let rsi = Rsi::new(14);
        // Generate test data with alternating up/down moves
        let data: Vec<f64> = (0..30)
            .map(|i| 100.0 + (i as f64 * 0.5).sin() * 5.0)
            .collect();

        let result = rsi.calculate(&data);
        assert!(!result.is_empty());

        // All RSI values should be between 0 and 100
        for value in &result {
            assert!(*value >= 0.0 && *value <= 100.0);
        }
    }

    #[test]
    fn test_rsi_all_gains() {
        let rsi = Rsi::new(5);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        let result = rsi.calculate(&data);

        assert!(!result.is_empty());
        // All gains = RSI should be 100
        assert!((result[0] - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_rsi_all_losses() {
        let rsi = Rsi::new(5);
        let data = vec![7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
        let result = rsi.calculate(&data);

        assert!(!result.is_empty());
        // All losses = RSI should be 0
        assert!(result[0].abs() < 1e-10);
    }

    #[test]
    fn test_macd_basic() {
        let macd = Macd::new();
        let data: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let result = macd.calculate(&data);

        assert!(!result.is_empty());
        // In an uptrend, MACD should be positive
        assert!(result.last().unwrap().macd > 0.0);
    }

    #[test]
    fn test_macd_custom_periods() {
        let macd = Macd::with_periods(5, 10, 3);
        let data: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let result = macd.calculate(&data);

        assert!(!result.is_empty());
    }

    #[test]
    fn test_stochastic_basic() {
        let stoch = Stochastic::new();
        let high: Vec<f64> = (0..30).map(|i| 105.0 + i as f64).collect();
        let low: Vec<f64> = (0..30).map(|i| 95.0 + i as f64).collect();
        let close: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();

        let result = stoch.calculate_ohlc(&high, &low, &close);
        assert!(!result.is_empty());

        // All values should be between 0 and 100
        for output in &result {
            assert!(output.k >= 0.0 && output.k <= 100.0);
            assert!(output.d >= 0.0 && output.d <= 100.0);
        }
    }

    #[test]
    fn test_stochastic_at_high() {
        let stoch = Stochastic::with_periods(5, 3);
        // Close at highs
        let high = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0];
        let low = vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let close = high.clone();

        let result = stoch.calculate_ohlc(&high, &low, &close);
        assert!(!result.is_empty());

        // Close at high = %K should be 100
        assert!((result.last().unwrap().k - 100.0).abs() < 1e-10);
    }
}
