//! Moving average indicators.

use trading_core::traits::Indicator;

/// Simple Moving Average (SMA).
///
/// Calculates the arithmetic mean of the last N values.
#[derive(Debug, Clone)]
pub struct Sma {
    period: usize,
}

impl Sma {
    /// Create a new SMA with the specified period.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self { period }
    }
}

impl Indicator for Sma {
    type Output = f64;

    fn calculate(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < self.period {
            return vec![];
        }

        let mut result = Vec::with_capacity(data.len() - self.period + 1);
        let period_f64 = self.period as f64;

        // Initial sum
        let mut sum: f64 = data[..self.period].iter().sum();
        result.push(sum / period_f64);

        // Sliding window
        for i in self.period..data.len() {
            sum = sum - data[i - self.period] + data[i];
            result.push(sum / period_f64);
        }

        result
    }

    fn period(&self) -> usize {
        self.period
    }

    fn name(&self) -> &str {
        "SMA"
    }
}

/// Exponential Moving Average (EMA).
///
/// Gives more weight to recent prices using an exponential decay.
#[derive(Debug, Clone)]
pub struct Ema {
    period: usize,
    multiplier: f64,
}

impl Ema {
    /// Create a new EMA with the specified period.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        let multiplier = 2.0 / (period as f64 + 1.0);
        Self { period, multiplier }
    }

    /// Create an EMA with a custom smoothing factor.
    pub fn with_multiplier(period: usize, multiplier: f64) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        assert!(
            (0.0..=1.0).contains(&multiplier),
            "Multiplier must be between 0 and 1"
        );
        Self { period, multiplier }
    }
}

impl Indicator for Ema {
    type Output = f64;

    fn calculate(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < self.period {
            return vec![];
        }

        let mut result = Vec::with_capacity(data.len() - self.period + 1);

        // Initialize with SMA
        let initial_sma: f64 = data[..self.period].iter().sum::<f64>() / self.period as f64;
        result.push(initial_sma);

        // Calculate EMA
        let mut ema = initial_sma;
        let one_minus_mult = 1.0 - self.multiplier;

        for &price in &data[self.period..] {
            ema = price * self.multiplier + ema * one_minus_mult;
            result.push(ema);
        }

        result
    }

    fn period(&self) -> usize {
        self.period
    }

    fn name(&self) -> &str {
        "EMA"
    }
}

/// Weighted Moving Average (WMA).
///
/// Gives linearly decreasing weights to older prices.
#[derive(Debug, Clone)]
pub struct Wma {
    period: usize,
    weights_sum: f64,
}

impl Wma {
    /// Create a new WMA with the specified period.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        // Sum of weights: 1 + 2 + ... + n = n(n+1)/2
        let weights_sum = (period * (period + 1)) as f64 / 2.0;
        Self { period, weights_sum }
    }
}

impl Indicator for Wma {
    type Output = f64;

    fn calculate(&self, data: &[f64]) -> Vec<f64> {
        if data.len() < self.period {
            return vec![];
        }

        let mut result = Vec::with_capacity(data.len() - self.period + 1);

        for window in data.windows(self.period) {
            let weighted_sum: f64 = window
                .iter()
                .enumerate()
                .map(|(i, &price)| price * (i + 1) as f64)
                .sum();
            result.push(weighted_sum / self.weights_sum);
        }

        result
    }

    fn period(&self) -> usize {
        self.period
    }

    fn name(&self) -> &str {
        "WMA"
    }
}

/// Streaming EMA that maintains state for incremental updates.
#[derive(Debug, Clone)]
pub struct StreamingEma {
    period: usize,
    multiplier: f64,
    current: Option<f64>,
    count: usize,
    sum: f64,
}

impl StreamingEma {
    /// Create a new streaming EMA.
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        let multiplier = 2.0 / (period as f64 + 1.0);
        Self {
            period,
            multiplier,
            current: None,
            count: 0,
            sum: 0.0,
        }
    }

    /// Update with a new value and return the current EMA.
    pub fn update(&mut self, value: f64) -> Option<f64> {
        self.count += 1;

        if self.count < self.period {
            // Accumulating for initial SMA
            self.sum += value;
            None
        } else if self.count == self.period {
            // First EMA value is the SMA
            self.sum += value;
            let sma = self.sum / self.period as f64;
            self.current = Some(sma);
            self.current
        } else {
            // Regular EMA calculation
            let ema = self.current.unwrap();
            let new_ema = value * self.multiplier + ema * (1.0 - self.multiplier);
            self.current = Some(new_ema);
            self.current
        }
    }

    /// Get the current EMA value.
    pub fn current(&self) -> Option<f64> {
        self.current
    }

    /// Reset the indicator.
    pub fn reset(&mut self) {
        self.current = None;
        self.count = 0;
        self.sum = 0.0;
    }

    /// Check if the indicator is ready.
    pub fn is_ready(&self) -> bool {
        self.count >= self.period
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let sma = Sma::new(3);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma.calculate(&data);

        assert_eq!(result.len(), 3);
        assert!((result[0] - 2.0).abs() < 1e-10); // (1+2+3)/3
        assert!((result[1] - 3.0).abs() < 1e-10); // (2+3+4)/3
        assert!((result[2] - 4.0).abs() < 1e-10); // (3+4+5)/3
    }

    #[test]
    fn test_sma_insufficient_data() {
        let sma = Sma::new(5);
        let data = vec![1.0, 2.0, 3.0];
        let result = sma.calculate(&data);

        assert!(result.is_empty());
    }

    #[test]
    fn test_ema() {
        let ema = Ema::new(3);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = ema.calculate(&data);

        assert_eq!(result.len(), 3);
        assert!((result[0] - 2.0).abs() < 1e-10); // Initial SMA
        // EMA = price * mult + prev_ema * (1 - mult)
        // mult = 2/(3+1) = 0.5
        // result[1] = 4 * 0.5 + 2 * 0.5 = 3.0
        assert!((result[1] - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_wma() {
        let wma = Wma::new(3);
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = wma.calculate(&data);

        assert_eq!(result.len(), 3);
        // Weights: 1, 2, 3; sum = 6
        // (1*1 + 2*2 + 3*3) / 6 = (1 + 4 + 9) / 6 = 14/6 ≈ 2.333
        assert!((result[0] - 14.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_streaming_ema() {
        let mut ema = StreamingEma::new(3);

        assert!(!ema.is_ready());
        assert!(ema.update(1.0).is_none());
        assert!(ema.update(2.0).is_none());

        // Third value triggers first output
        let first = ema.update(3.0).unwrap();
        assert!((first - 2.0).abs() < 1e-10); // SMA of first 3
        assert!(ema.is_ready());

        // Subsequent values use EMA formula
        let second = ema.update(4.0).unwrap();
        assert!((second - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_streaming_ema_reset() {
        let mut ema = StreamingEma::new(3);
        ema.update(1.0);
        ema.update(2.0);
        ema.update(3.0);

        assert!(ema.is_ready());
        ema.reset();
        assert!(!ema.is_ready());
        assert!(ema.current().is_none());
    }
}
