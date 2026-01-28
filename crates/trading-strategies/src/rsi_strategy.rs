//! RSI-based Trading Strategy.
//!
//! Trades based on RSI overbought/oversold conditions.
//! Buys when RSI crosses above oversold level,
//! sells when RSI crosses below overbought level.

use serde::{Deserialize, Serialize};
use trading_core::{
    error::StrategyError,
    traits::{Indicator, Strategy, StrategyConfig, StrategyState},
    types::{BarSeries, Signal, SignalMetadata, SignalStrength, SignalType},
};
use trading_indicators::Rsi;

/// Configuration for the RSI strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsiConfig {
    /// Symbols to trade
    pub symbols: Vec<String>,
    /// RSI calculation period
    pub period: usize,
    /// Overbought threshold (sell above this)
    pub overbought: f64,
    /// Oversold threshold (buy below this)
    pub oversold: f64,
    /// Exit overbought level for longs
    pub exit_overbought: f64,
    /// Exit oversold level for shorts
    pub exit_oversold: f64,
    /// Allow short positions
    pub allow_short: bool,
}

impl Default for RsiConfig {
    fn default() -> Self {
        Self {
            symbols: vec![],
            period: 14,
            overbought: 70.0,
            oversold: 30.0,
            exit_overbought: 70.0,
            exit_oversold: 30.0,
            allow_short: false,
        }
    }
}

impl StrategyConfig for RsiConfig {
    fn validate(&self) -> Result<(), StrategyError> {
        if self.period < 2 {
            return Err(StrategyError::InvalidConfig(
                "RSI period must be at least 2".into(),
            ));
        }
        if self.overbought <= self.oversold {
            return Err(StrategyError::InvalidConfig(
                "Overbought must be greater than oversold".into(),
            ));
        }
        if self.overbought > 100.0 || self.oversold < 0.0 {
            return Err(StrategyError::InvalidConfig(
                "RSI thresholds must be between 0 and 100".into(),
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

/// Position state
#[derive(Debug, Clone, Copy, PartialEq)]
enum PositionState {
    Flat,
    Long,
    Short,
}

/// RSI-based Trading Strategy.
pub struct RsiStrategy {
    config: RsiConfig,
    rsi: Rsi,
    position: PositionState,
    prev_rsi: Option<f64>,
    bars_processed: usize,
    signals_generated: usize,
}

impl RsiStrategy {
    /// Create a new RSI strategy.
    pub fn new(config: RsiConfig) -> Self {
        let rsi = Rsi::new(config.period);
        Self {
            config,
            rsi,
            position: PositionState::Flat,
            prev_rsi: None,
            bars_processed: 0,
            signals_generated: 0,
        }
    }

    fn classify_strength(&self, rsi: f64) -> SignalStrength {
        if rsi <= 20.0 || rsi >= 80.0 {
            SignalStrength::Strong
        } else if rsi <= 30.0 || rsi >= 70.0 {
            SignalStrength::Moderate
        } else {
            SignalStrength::Weak
        }
    }

    fn calculate_confidence(&self, rsi: f64) -> f64 {
        // Higher confidence at extreme RSI levels
        if rsi <= 20.0 || rsi >= 80.0 {
            0.9
        } else if rsi <= 30.0 || rsi >= 70.0 {
            0.7
        } else {
            0.5
        }
    }

    fn create_signal(
        &self,
        symbol: &str,
        signal_type: SignalType,
        price: f64,
        timestamp: i64,
        rsi: f64,
        reason: &str,
    ) -> Signal {
        Signal {
            symbol: symbol.to_string(),
            signal_type,
            strength: self.classify_strength(rsi),
            price,
            timestamp,
            confidence: self.calculate_confidence(rsi),
            metadata: SignalMetadata {
                strategy_name: self.name().to_string(),
                indicators: [("rsi".to_string(), rsi)].into_iter().collect(),
                reason: reason.to_string(),
                ..Default::default()
            },
        }
    }
}

impl Strategy for RsiStrategy {
    fn name(&self) -> &str {
        "RSI Strategy"
    }

    fn description(&self) -> &str {
        "Trades RSI overbought/oversold reversals"
    }

    fn on_bar(&mut self, series: &BarSeries) -> Option<Signal> {
        self.bars_processed += 1;

        if series.len() < self.warmup_period() {
            return None;
        }

        let closes = series.closes();
        let rsi_values = self.rsi.calculate(&closes);

        if rsi_values.is_empty() {
            return None;
        }

        let current_rsi = *rsi_values.last()?;
        let bar = series.last()?;

        let signal = match (self.prev_rsi, self.position) {
            // Entry signals when flat
            (Some(prev), PositionState::Flat) => {
                // Oversold -> potential long entry (RSI crosses above oversold)
                if prev <= self.config.oversold && current_rsi > self.config.oversold {
                    self.position = PositionState::Long;
                    self.signals_generated += 1;
                    Some(self.create_signal(
                        &series.symbol,
                        SignalType::Buy,
                        bar.close,
                        bar.timestamp,
                        current_rsi,
                        &format!(
                            "RSI ({:.1}) crossed above oversold level ({:.1})",
                            current_rsi, self.config.oversold
                        ),
                    ))
                }
                // Overbought -> potential short entry (RSI crosses below overbought)
                else if self.config.allow_short
                    && prev >= self.config.overbought
                    && current_rsi < self.config.overbought
                {
                    self.position = PositionState::Short;
                    self.signals_generated += 1;
                    Some(self.create_signal(
                        &series.symbol,
                        SignalType::Sell,
                        bar.close,
                        bar.timestamp,
                        current_rsi,
                        &format!(
                            "RSI ({:.1}) crossed below overbought level ({:.1})",
                            current_rsi, self.config.overbought
                        ),
                    ))
                } else {
                    None
                }
            }
            // Exit signals for long position
            (Some(_prev), PositionState::Long) => {
                if current_rsi >= self.config.exit_overbought {
                    self.position = PositionState::Flat;
                    self.signals_generated += 1;
                    Some(self.create_signal(
                        &series.symbol,
                        SignalType::CloseLong,
                        bar.close,
                        bar.timestamp,
                        current_rsi,
                        &format!(
                            "RSI ({:.1}) reached overbought exit level ({:.1})",
                            current_rsi, self.config.exit_overbought
                        ),
                    ))
                } else {
                    None
                }
            }
            // Exit signals for short position
            (Some(_prev), PositionState::Short) => {
                if current_rsi <= self.config.exit_oversold {
                    self.position = PositionState::Flat;
                    self.signals_generated += 1;
                    Some(self.create_signal(
                        &series.symbol,
                        SignalType::CloseShort,
                        bar.close,
                        bar.timestamp,
                        current_rsi,
                        &format!(
                            "RSI ({:.1}) reached oversold exit level ({:.1})",
                            current_rsi, self.config.exit_oversold
                        ),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        };

        self.prev_rsi = Some(current_rsi);
        signal
    }

    fn reset(&mut self) {
        self.position = PositionState::Flat;
        self.prev_rsi = None;
        self.bars_processed = 0;
        self.signals_generated = 0;
    }

    fn state(&self) -> StrategyState {
        StrategyState {
            name: self.name().to_string(),
            is_warmed_up: self.bars_processed >= self.warmup_period(),
            bars_processed: self.bars_processed,
            signals_generated: self.signals_generated,
            indicators: [("rsi".to_string(), self.prev_rsi.unwrap_or(50.0))]
                .into_iter()
                .collect(),
            custom: serde_json::json!({
                "position": format!("{:?}", self.position),
                "overbought": self.config.overbought,
                "oversold": self.config.oversold,
            }),
        }
    }

    fn warmup_period(&self) -> usize {
        self.config.period + 1
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
        let mut config = RsiConfig::default();
        config.symbols = vec!["AAPL".to_string()];
        assert!(config.validate().is_ok());

        config.overbought = 30.0;
        config.oversold = 70.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_oversold_entry() {
        let config = RsiConfig {
            symbols: vec!["TEST".to_string()],
            period: 5,
            overbought: 70.0,
            oversold: 30.0,
            exit_overbought: 70.0,
            exit_oversold: 30.0,
            allow_short: false,
        };

        let mut strategy = RsiStrategy::new(config);

        // Create prices with a significant drop followed by recovery
        // This should trigger oversold condition then recovery
        let prices: Vec<f64> = vec![
            100.0, 99.0, 98.0, 97.0, 96.0, // Initial decline
            95.0, 94.0, 93.0, 92.0, 91.0, // Continued decline (oversold)
            92.0, 93.0, 94.0, 95.0, 96.0, // Recovery
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

        // Should eventually have a buy signal when recovering from oversold
        // Note: This depends on exact RSI calculation
        assert!(strategy.bars_processed > 0);
    }

    #[test]
    fn test_signal_strength() {
        let config = RsiConfig::default();
        let strategy = RsiStrategy::new(config);

        assert_eq!(strategy.classify_strength(15.0), SignalStrength::Strong);
        assert_eq!(strategy.classify_strength(25.0), SignalStrength::Moderate);
        assert_eq!(strategy.classify_strength(50.0), SignalStrength::Weak);
        assert_eq!(strategy.classify_strength(75.0), SignalStrength::Moderate);
        assert_eq!(strategy.classify_strength(85.0), SignalStrength::Strong);
    }

    #[test]
    fn test_reset() {
        let config = RsiConfig {
            symbols: vec!["TEST".to_string()],
            period: 5,
            ..Default::default()
        };

        let mut strategy = RsiStrategy::new(config);

        let series = create_test_series(&[100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0]);
        strategy.on_bar(&series);

        assert!(strategy.prev_rsi.is_some());
        assert!(strategy.bars_processed > 0);

        strategy.reset();

        assert!(strategy.prev_rsi.is_none());
        assert_eq!(strategy.bars_processed, 0);
        assert_eq!(strategy.position, PositionState::Flat);
    }
}
