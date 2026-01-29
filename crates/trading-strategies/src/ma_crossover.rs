//! Moving Average Crossover Strategy.
//!
//! Generates buy signals when the fast MA crosses above the slow MA,
//! and sell signals when the fast MA crosses below the slow MA.

use serde::{Deserialize, Serialize};
use trading_core::traits::Indicator;
use trading_core::{
    error::StrategyError,
    traits::{Strategy, StrategyConfig, StrategyState},
    types::{BarSeries, Signal, SignalMetadata, SignalStrength, SignalType},
};
use trading_indicators::{Ema, Sma};

/// Configuration for the MA Crossover strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MACrossoverConfig {
    /// Symbols to trade
    pub symbols: Vec<String>,
    /// Fast moving average period
    pub fast_period: usize,
    /// Slow moving average period
    pub slow_period: usize,
    /// Use EMA instead of SMA
    pub use_ema: bool,
    /// Minimum crossover magnitude to generate signal (as percentage)
    pub signal_threshold: f64,
}

impl Default for MACrossoverConfig {
    fn default() -> Self {
        Self {
            symbols: vec![],
            fast_period: 12,
            slow_period: 26,
            use_ema: true,
            signal_threshold: 0.001, // 0.1%
        }
    }
}

impl StrategyConfig for MACrossoverConfig {
    fn validate(&self) -> Result<(), StrategyError> {
        if self.fast_period >= self.slow_period {
            return Err(StrategyError::InvalidConfig(
                "Fast period must be less than slow period".into(),
            ));
        }
        if self.fast_period == 0 {
            return Err(StrategyError::InvalidConfig(
                "Fast period must be greater than 0".into(),
            ));
        }
        if self.symbols.is_empty() {
            return Err(StrategyError::InvalidConfig(
                "At least one symbol required".into(),
            ));
        }
        Ok(())
    }
}

/// Moving Average Crossover Strategy.
pub struct MACrossoverStrategy {
    config: MACrossoverConfig,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
    bars_processed: usize,
    signals_generated: usize,
}

impl MACrossoverStrategy {
    /// Create a new MA Crossover strategy.
    pub fn new(config: MACrossoverConfig) -> Self {
        Self {
            config,
            prev_fast: None,
            prev_slow: None,
            bars_processed: 0,
            signals_generated: 0,
        }
    }

    fn classify_strength(magnitude: f64) -> SignalStrength {
        if magnitude > 0.02 {
            SignalStrength::Strong
        } else if magnitude > 0.01 {
            SignalStrength::Moderate
        } else {
            SignalStrength::Weak
        }
    }

    fn calculate_ma(&self, closes: &[f64], period: usize) -> Vec<f64> {
        if self.config.use_ema {
            Ema::new(period).calculate(closes)
        } else {
            Sma::new(period).calculate(closes)
        }
    }
}

impl Strategy for MACrossoverStrategy {
    fn name(&self) -> &str {
        "MA Crossover"
    }

    fn description(&self) -> &str {
        "Generates signals based on fast/slow moving average crossovers"
    }

    fn on_bar(&mut self, series: &BarSeries) -> Option<Signal> {
        self.bars_processed += 1;

        if series.len() < self.warmup_period() {
            return None;
        }

        let closes = series.closes();

        // Calculate MAs
        let fast = self.calculate_ma(&closes, self.config.fast_period);
        let slow = self.calculate_ma(&closes, self.config.slow_period);

        if fast.is_empty() || slow.is_empty() {
            return None;
        }

        let current_fast = *fast.last()?;
        let current_slow = *slow.last()?;

        let signal = match (self.prev_fast, self.prev_slow) {
            (Some(prev_f), Some(prev_s)) => {
                let crossover_magnitude = if current_slow != 0.0 {
                    ((current_fast - current_slow) / current_slow).abs()
                } else {
                    0.0
                };

                let bar = series.last()?;

                // Bullish crossover: fast crosses above slow
                if prev_f <= prev_s
                    && current_fast > current_slow
                    && crossover_magnitude >= self.config.signal_threshold
                {
                    self.signals_generated += 1;
                    Some(Signal {
                        symbol: series.symbol.clone(),
                        signal_type: SignalType::Buy,
                        strength: Self::classify_strength(crossover_magnitude),
                        price: bar.close,
                        timestamp: bar.timestamp,
                        confidence: crossover_magnitude.min(1.0),
                        metadata: SignalMetadata {
                            strategy_name: self.name().to_string(),
                            indicators: [
                                ("fast_ma".to_string(), current_fast),
                                ("slow_ma".to_string(), current_slow),
                                ("crossover_magnitude".to_string(), crossover_magnitude),
                            ]
                            .into_iter()
                            .collect(),
                            reason: format!(
                                "Bullish crossover: fast MA ({:.2}) crossed above slow MA ({:.2})",
                                current_fast, current_slow
                            ),
                            ..Default::default()
                        },
                    })
                }
                // Bearish crossover: fast crosses below slow
                else if prev_f >= prev_s
                    && current_fast < current_slow
                    && crossover_magnitude >= self.config.signal_threshold
                {
                    self.signals_generated += 1;
                    Some(Signal {
                        symbol: series.symbol.clone(),
                        signal_type: SignalType::Sell,
                        strength: Self::classify_strength(crossover_magnitude),
                        price: bar.close,
                        timestamp: bar.timestamp,
                        confidence: crossover_magnitude.min(1.0),
                        metadata: SignalMetadata {
                            strategy_name: self.name().to_string(),
                            indicators: [
                                ("fast_ma".to_string(), current_fast),
                                ("slow_ma".to_string(), current_slow),
                                ("crossover_magnitude".to_string(), crossover_magnitude),
                            ]
                            .into_iter()
                            .collect(),
                            reason: format!(
                                "Bearish crossover: fast MA ({:.2}) crossed below slow MA ({:.2})",
                                current_fast, current_slow
                            ),
                            ..Default::default()
                        },
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        self.prev_fast = Some(current_fast);
        self.prev_slow = Some(current_slow);

        signal
    }

    fn reset(&mut self) {
        self.prev_fast = None;
        self.prev_slow = None;
        self.bars_processed = 0;
        self.signals_generated = 0;
    }

    fn state(&self) -> StrategyState {
        StrategyState {
            name: self.name().to_string(),
            is_warmed_up: self.bars_processed >= self.warmup_period(),
            bars_processed: self.bars_processed,
            signals_generated: self.signals_generated,
            indicators: [
                ("fast_ma".to_string(), self.prev_fast.unwrap_or(0.0)),
                ("slow_ma".to_string(), self.prev_slow.unwrap_or(0.0)),
            ]
            .into_iter()
            .collect(),
            custom: serde_json::json!({
                "fast_period": self.config.fast_period,
                "slow_period": self.config.slow_period,
                "use_ema": self.config.use_ema,
            }),
        }
    }

    fn warmup_period(&self) -> usize {
        self.config.slow_period + 1
    }

    fn symbols(&self) -> &[String] {
        &self.config.symbols
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trading_core::types::{Bar, Timeframe};

    fn create_test_series(prices: &[f64]) -> BarSeries {
        let mut series = BarSeries::new("TEST".to_string(), Timeframe::Daily);
        for (i, &price) in prices.iter().enumerate() {
            series.push(Bar::new(
                i as i64 * 86400000,
                price,
                price + 1.0,
                price - 1.0,
                price,
                1000.0,
            ));
        }
        series
    }

    #[test]
    fn test_config_validation() {
        let mut config = MACrossoverConfig {
            symbols: vec!["AAPL".to_string()],
            ..Default::default()
        };
        assert!(config.validate().is_ok());

        config.fast_period = 30;
        config.slow_period = 20;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_bullish_crossover() {
        let config = MACrossoverConfig {
            symbols: vec!["TEST".to_string()],
            fast_period: 3,
            slow_period: 5,
            use_ema: false,
            signal_threshold: 0.0,
        };

        let mut strategy = MACrossoverStrategy::new(config);

        // Prices that create a bullish crossover
        // First: fast below slow, then fast above slow
        let prices = vec![
            100.0, 99.0, 98.0, 97.0, 96.0, // Downtrend
            97.0, 99.0, 102.0, 105.0, 108.0, // Uptrend starts
        ];

        let series = create_test_series(&prices);

        let mut signals = Vec::new();
        for i in 0..prices.len() {
            let mut temp_series = BarSeries::new("TEST".to_string(), Timeframe::Daily);
            for bar in series.bars().iter().take(i + 1) {
                temp_series.push(*bar);
            }
            if let Some(signal) = strategy.on_bar(&temp_series) {
                signals.push(signal);
            }
        }

        // Should have at least one buy signal when trend reverses
        let buy_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.signal_type == SignalType::Buy)
            .collect();
        assert!(!buy_signals.is_empty());
    }

    #[test]
    fn test_reset() {
        let config = MACrossoverConfig {
            symbols: vec!["TEST".to_string()],
            fast_period: 3,
            slow_period: 5,
            use_ema: true,
            signal_threshold: 0.0,
        };

        let mut strategy = MACrossoverStrategy::new(config);

        let series = create_test_series(&[100.0, 101.0, 102.0, 103.0, 104.0, 105.0]);
        strategy.on_bar(&series);

        assert!(strategy.prev_fast.is_some());
        assert!(strategy.bars_processed > 0);

        strategy.reset();

        assert!(strategy.prev_fast.is_none());
        assert_eq!(strategy.bars_processed, 0);
    }
}
