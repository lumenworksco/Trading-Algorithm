//! Indicator trait definitions.

use crate::error::IndicatorError;

/// Trait for technical indicators.
///
/// Indicators process price data and produce derived values
/// useful for trading decisions.
pub trait Indicator: Send + Sync {
    /// The output type of the indicator.
    type Output;

    /// Calculate indicator values for the given data.
    ///
    /// # Arguments
    /// * `data` - Input data (typically prices)
    ///
    /// # Returns
    /// A vector of indicator values
    fn calculate(&self, data: &[f64]) -> Vec<Self::Output>;

    /// Get the minimum data points required.
    fn period(&self) -> usize;

    /// Get the name of the indicator.
    fn name(&self) -> &str;

    /// Validate that there's enough data.
    fn validate_data(&self, data: &[f64]) -> Result<(), IndicatorError> {
        if data.len() < self.period() {
            return Err(IndicatorError::InsufficientData {
                required: self.period(),
                available: data.len(),
            });
        }
        Ok(())
    }
}

/// Streaming indicator that maintains internal state.
///
/// Unlike batch indicators, streaming indicators can be updated
/// incrementally with new data points.
pub trait StreamingIndicator: Send + Sync {
    /// The output type of the indicator.
    type Output;

    /// Update the indicator with a new value.
    ///
    /// # Arguments
    /// * `value` - New input value
    ///
    /// # Returns
    /// The current indicator value, or None if not yet ready
    fn update(&mut self, value: f64) -> Option<Self::Output>;

    /// Get the current value without adding new data.
    fn current(&self) -> Option<Self::Output>;

    /// Reset the indicator state.
    fn reset(&mut self);

    /// Check if the indicator has enough data to produce values.
    fn is_ready(&self) -> bool;

    /// Get the minimum data points required.
    fn period(&self) -> usize;

    /// Get the name of the indicator.
    fn name(&self) -> &str;
}

/// Multi-output indicator (e.g., Bollinger Bands, MACD).
///
/// Some indicators produce multiple related values.
pub trait MultiOutputIndicator: Send + Sync {
    /// The output type containing multiple values.
    type Outputs;

    /// Calculate indicator values for the given data.
    fn calculate(&self, data: &[f64]) -> Vec<Self::Outputs>;

    /// Get the minimum data points required.
    fn period(&self) -> usize;

    /// Get the name of the indicator.
    fn name(&self) -> &str;

    /// Validate that there's enough data.
    fn validate_data(&self, data: &[f64]) -> Result<(), IndicatorError> {
        if data.len() < self.period() {
            return Err(IndicatorError::InsufficientData {
                required: self.period(),
                available: data.len(),
            });
        }
        Ok(())
    }
}

/// OHLCV indicator that uses all bar data (not just close).
#[allow(dead_code)]
pub trait OhlcvIndicator: Send + Sync {
    /// The output type of the indicator.
    type Output;

    /// Calculate indicator values from OHLCV data.
    ///
    /// # Arguments
    /// * `open` - Open prices
    /// * `high` - High prices
    /// * `low` - Low prices
    /// * `close` - Close prices
    /// * `volume` - Volume data
    fn calculate(
        &self,
        open: &[f64],
        high: &[f64],
        low: &[f64],
        close: &[f64],
        volume: &[f64],
    ) -> Vec<Self::Output>;

    /// Get the minimum data points required.
    fn period(&self) -> usize;

    /// Get the name of the indicator.
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestIndicator {
        period: usize,
    }

    impl Indicator for TestIndicator {
        type Output = f64;

        fn calculate(&self, data: &[f64]) -> Vec<f64> {
            if data.len() < self.period {
                return vec![];
            }
            // Simple sum indicator for testing
            data.windows(self.period)
                .map(|w| w.iter().sum())
                .collect()
        }

        fn period(&self) -> usize {
            self.period
        }

        fn name(&self) -> &str {
            "test"
        }
    }

    #[test]
    fn test_indicator_validation() {
        let indicator = TestIndicator { period: 5 };

        assert!(indicator.validate_data(&[1.0, 2.0, 3.0]).is_err());
        assert!(indicator.validate_data(&[1.0, 2.0, 3.0, 4.0, 5.0]).is_ok());
    }

    #[test]
    fn test_indicator_calculate() {
        let indicator = TestIndicator { period: 3 };
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = indicator.calculate(&data);

        assert_eq!(result.len(), 3);
        assert!((result[0] - 6.0).abs() < 0.001); // 1+2+3
        assert!((result[1] - 9.0).abs() < 0.001); // 2+3+4
        assert!((result[2] - 12.0).abs() < 0.001); // 3+4+5
    }
}
