//! Mean Reversion Strategy using Bollinger Bands.
//!
//! Buys when price touches the lower band (oversold),
//! sells when price touches the upper band (overbought).

use serde::{Deserialize, Serialize};
use trading_core::{
    error::StrategyError,
    traits::{Strategy, StrategyConfig, StrategyState, MultiOutputIndicator},
    types::{BarSeries, Signal, SignalMetadata, SignalStrength, SignalType},
};
use trading_indicators::BollingerBands;

/// Configuration for the Mean Reversion strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeanReversionConfig {
    /// Symbols to trade
    pub symbols: Vec<String>,
    /// Bollinger Bands period
    pub bb_period: usize,
    /// Bollinger Bands standard deviation multiplier
    pub bb_std_dev: f64,
    /// Entry threshold (%B value for entry, e.g., 0.05 = below 5%)
    pub entry_threshold: f64,
    /// Exit threshold (%B value for exit, e.g., 0.5 = at middle band)
    pub exit_threshold: f64,
    /// Use mean reversion for both long and short
    pub allow_short: bool,
}

impl Default for MeanReversionConfig {
    fn default() -> Self {
        Self {
            symbols: vec![],
            bb_period: 20,
            bb_std_dev: 2.0,
            entry_threshold: 0.05,
            exit_threshold: 0.5,
            allow_short: false,
        }
    }
}

impl StrategyConfig for MeanReversionConfig {
    fn validate(&self) -> Result<(), StrategyError> {
        if self.bb_period < 2 {
            return Err(StrategyError::InvalidConfig(
                "BB period must be at least 2".into(),
            ));
        }
        if self.bb_std_dev <= 0.0 {
            return Err(StrategyError::InvalidConfig(
                "BB std dev must be positive".into(),
            ));
        }
        if self.entry_threshold < 0.0 || self.entry_threshold > 0.5 {
            return Err(StrategyError::InvalidConfig(
                "Entry threshold must be between 0 and 0.5".into(),
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

/// Position state for the strategy
#[derive(Debug, Clone, Copy, PartialEq)]
enum PositionState {
    Flat,
    Long,
    Short,
}

/// Mean Reversion Strategy using Bollinger Bands.
pub struct MeanReversionStrategy {
    config: MeanReversionConfig,
    bb: BollingerBands,
    position: PositionState,
    bars_processed: usize,
    signals_generated: usize,
    last_percent_b: Option<f64>,
    last_bandwidth: Option<f64>,
}

impl MeanReversionStrategy {
    /// Create a new Mean Reversion strategy.
    pub fn new(config: MeanReversionConfig) -> Self {
        let bb = BollingerBands::with_params(config.bb_period, config.bb_std_dev);
        Self {
            config,
            bb,
            position: PositionState::Flat,
            bars_processed: 0,
            signals_generated: 0,
            last_percent_b: None,
            last_bandwidth: None,
        }
    }

    fn classify_strength(&self, percent_b: f64) -> SignalStrength {
        // More extreme %B = stronger signal
        let distance_from_extreme = if percent_b < 0.5 {
            percent_b // Distance from 0
        } else {
            1.0 - percent_b // Distance from 1
        };

        if distance_from_extreme < 0.05 {
            SignalStrength::Strong
        } else if distance_from_extreme < 0.15 {
            SignalStrength::Moderate
        } else {
            SignalStrength::Weak
        }
    }
}

impl Strategy for MeanReversionStrategy {
    fn name(&self) -> &str {
        "Mean Reversion"
    }

    fn description(&self) -> &str {
        "Trades reversions to the mean using Bollinger Bands"
    }

    fn on_bar(&mut self, series: &BarSeries) -> Option<Signal> {
        self.bars_processed += 1;

        if series.len() < self.warmup_period() {
            return None;
        }

        let closes = series.closes();
        let bb_values = self.bb.calculate(&closes);

        if bb_values.is_empty() {
            return None;
        }

        let bb = bb_values.last()?;
        let bar = series.last()?;

        self.last_percent_b = Some(bb.percent_b);
        self.last_bandwidth = Some(bb.bandwidth);

        let signal = match self.position {
            PositionState::Flat => {
                // Look for entry signals
                if bb.percent_b <= self.config.entry_threshold {
                    // Oversold - potential long entry
                    self.position = PositionState::Long;
                    self.signals_generated += 1;
                    Some(Signal {
                        symbol: series.symbol.clone(),
                        signal_type: SignalType::Buy,
                        strength: self.classify_strength(bb.percent_b),
                        price: bar.close,
                        timestamp: bar.timestamp,
                        confidence: 1.0 - bb.percent_b, // Higher confidence when more oversold
                        metadata: SignalMetadata {
                            strategy_name: self.name().to_string(),
                            indicators: [
                                ("percent_b".to_string(), bb.percent_b),
                                ("upper_band".to_string(), bb.upper),
                                ("middle_band".to_string(), bb.middle),
                                ("lower_band".to_string(), bb.lower),
                                ("bandwidth".to_string(), bb.bandwidth),
                            ]
                            .into_iter()
                            .collect(),
                            reason: format!(
                                "Price near lower band (%B: {:.2}%), expecting reversion to mean",
                                bb.percent_b * 100.0
                            ),
                            stop_loss: Some(bb.lower - (bb.upper - bb.lower) * 0.1),
                            take_profit: Some(bb.middle),
                            ..Default::default()
                        },
                    })
                } else if self.config.allow_short && bb.percent_b >= 1.0 - self.config.entry_threshold {
                    // Overbought - potential short entry
                    self.position = PositionState::Short;
                    self.signals_generated += 1;
                    Some(Signal {
                        symbol: series.symbol.clone(),
                        signal_type: SignalType::Sell,
                        strength: self.classify_strength(bb.percent_b),
                        price: bar.close,
                        timestamp: bar.timestamp,
                        confidence: bb.percent_b,
                        metadata: SignalMetadata {
                            strategy_name: self.name().to_string(),
                            indicators: [
                                ("percent_b".to_string(), bb.percent_b),
                                ("upper_band".to_string(), bb.upper),
                                ("middle_band".to_string(), bb.middle),
                                ("lower_band".to_string(), bb.lower),
                                ("bandwidth".to_string(), bb.bandwidth),
                            ]
                            .into_iter()
                            .collect(),
                            reason: format!(
                                "Price near upper band (%B: {:.2}%), expecting reversion to mean",
                                bb.percent_b * 100.0
                            ),
                            stop_loss: Some(bb.upper + (bb.upper - bb.lower) * 0.1),
                            take_profit: Some(bb.middle),
                            ..Default::default()
                        },
                    })
                } else {
                    None
                }
            }
            PositionState::Long => {
                // Look for exit signal
                if bb.percent_b >= self.config.exit_threshold {
                    self.position = PositionState::Flat;
                    self.signals_generated += 1;
                    Some(Signal {
                        symbol: series.symbol.clone(),
                        signal_type: SignalType::CloseLong,
                        strength: SignalStrength::Moderate,
                        price: bar.close,
                        timestamp: bar.timestamp,
                        confidence: 0.8,
                        metadata: SignalMetadata {
                            strategy_name: self.name().to_string(),
                            indicators: [("percent_b".to_string(), bb.percent_b)]
                                .into_iter()
                                .collect(),
                            reason: format!(
                                "Price returned to mean (%B: {:.2}%)",
                                bb.percent_b * 100.0
                            ),
                            ..Default::default()
                        },
                    })
                } else {
                    None
                }
            }
            PositionState::Short => {
                // Look for exit signal
                if bb.percent_b <= self.config.exit_threshold {
                    self.position = PositionState::Flat;
                    self.signals_generated += 1;
                    Some(Signal {
                        symbol: series.symbol.clone(),
                        signal_type: SignalType::CloseShort,
                        strength: SignalStrength::Moderate,
                        price: bar.close,
                        timestamp: bar.timestamp,
                        confidence: 0.8,
                        metadata: SignalMetadata {
                            strategy_name: self.name().to_string(),
                            indicators: [("percent_b".to_string(), bb.percent_b)]
                                .into_iter()
                                .collect(),
                            reason: format!(
                                "Price returned to mean (%B: {:.2}%)",
                                bb.percent_b * 100.0
                            ),
                            ..Default::default()
                        },
                    })
                } else {
                    None
                }
            }
        };

        signal
    }

    fn reset(&mut self) {
        self.position = PositionState::Flat;
        self.bars_processed = 0;
        self.signals_generated = 0;
        self.last_percent_b = None;
        self.last_bandwidth = None;
    }

    fn state(&self) -> StrategyState {
        StrategyState {
            name: self.name().to_string(),
            is_warmed_up: self.bars_processed >= self.warmup_period(),
            bars_processed: self.bars_processed,
            signals_generated: self.signals_generated,
            indicators: [
                ("percent_b".to_string(), self.last_percent_b.unwrap_or(0.5)),
                ("bandwidth".to_string(), self.last_bandwidth.unwrap_or(0.0)),
            ]
            .into_iter()
            .collect(),
            custom: serde_json::json!({
                "position": format!("{:?}", self.position),
                "bb_period": self.config.bb_period,
                "bb_std_dev": self.config.bb_std_dev,
            }),
        }
    }

    fn warmup_period(&self) -> usize {
        self.config.bb_period
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
        let mut config = MeanReversionConfig::default();
        config.symbols = vec!["AAPL".to_string()];
        assert!(config.validate().is_ok());

        config.bb_period = 1;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_oversold_entry() {
        let config = MeanReversionConfig {
            symbols: vec!["TEST".to_string()],
            bb_period: 10,
            bb_std_dev: 2.0,
            entry_threshold: 0.1,
            exit_threshold: 0.5,
            allow_short: false,
        };

        let mut strategy = MeanReversionStrategy::new(config);

        // Create prices that go from stable to oversold
        let mut prices: Vec<f64> = vec![100.0; 10];
        prices.extend(vec![95.0, 90.0, 85.0]); // Sharp drop

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

        // Should have a buy signal when price drops below lower band
        let buy_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.signal_type == SignalType::Buy)
            .collect();
        assert!(!buy_signals.is_empty());
    }
}
