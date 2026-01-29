//! Momentum/Trend Following Strategy.
//!
//! Uses price momentum and trend indicators to follow strong trends.
//! Buys when momentum is positive and accelerating,
//! sells when momentum turns negative.

use serde::{Deserialize, Serialize};
use trading_core::{
    error::StrategyError,
    traits::{Indicator, Strategy, StrategyConfig, StrategyState},
    types::{BarSeries, Signal, SignalMetadata, SignalStrength, SignalType},
};
use trading_indicators::{Ema, Rsi};

/// Configuration for the Momentum strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumConfig {
    /// Symbols to trade
    pub symbols: Vec<String>,
    /// Momentum lookback period
    pub momentum_period: usize,
    /// Fast EMA period for trend
    pub fast_ema_period: usize,
    /// Slow EMA period for trend
    pub slow_ema_period: usize,
    /// RSI period for confirmation
    pub rsi_period: usize,
    /// Minimum RSI for long entry
    pub rsi_long_threshold: f64,
    /// Maximum RSI for short entry
    pub rsi_short_threshold: f64,
    /// Minimum momentum percentage for entry
    pub min_momentum: f64,
    /// Allow short positions
    pub allow_short: bool,
}

impl Default for MomentumConfig {
    fn default() -> Self {
        Self {
            symbols: vec![],
            momentum_period: 10,
            fast_ema_period: 12,
            slow_ema_period: 26,
            rsi_period: 14,
            rsi_long_threshold: 50.0,
            rsi_short_threshold: 50.0,
            min_momentum: 0.02, // 2%
            allow_short: false,
        }
    }
}

impl StrategyConfig for MomentumConfig {
    fn validate(&self) -> Result<(), StrategyError> {
        if self.momentum_period == 0 {
            return Err(StrategyError::InvalidConfig(
                "Momentum period must be greater than 0".into(),
            ));
        }
        if self.fast_ema_period >= self.slow_ema_period {
            return Err(StrategyError::InvalidConfig(
                "Fast EMA period must be less than slow EMA period".into(),
            ));
        }
        if self.rsi_period == 0 {
            return Err(StrategyError::InvalidConfig(
                "RSI period must be greater than 0".into(),
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

/// Momentum/Trend Following Strategy.
pub struct MomentumStrategy {
    config: MomentumConfig,
    position: PositionState,
    bars_processed: usize,
    signals_generated: usize,
    last_momentum: Option<f64>,
    last_rsi: Option<f64>,
    last_trend: Option<f64>,
}

impl MomentumStrategy {
    /// Create a new Momentum strategy.
    pub fn new(config: MomentumConfig) -> Self {
        Self {
            config,
            position: PositionState::Flat,
            bars_processed: 0,
            signals_generated: 0,
            last_momentum: None,
            last_rsi: None,
            last_trend: None,
        }
    }

    /// Calculate momentum as rate of change.
    fn calculate_momentum(&self, closes: &[f64]) -> Option<f64> {
        if closes.len() < self.config.momentum_period + 1 {
            return None;
        }

        let current = *closes.last()?;
        let past = closes[closes.len() - self.config.momentum_period - 1];

        if past != 0.0 {
            Some((current - past) / past)
        } else {
            None
        }
    }

    /// Calculate trend strength (fast EMA - slow EMA) / slow EMA.
    fn calculate_trend(&self, closes: &[f64]) -> Option<f64> {
        let fast_ema = Ema::new(self.config.fast_ema_period);
        let slow_ema = Ema::new(self.config.slow_ema_period);

        let fast_values = fast_ema.calculate(closes);
        let slow_values = slow_ema.calculate(closes);

        if fast_values.is_empty() || slow_values.is_empty() {
            return None;
        }

        // Use the most recent values from each EMA
        let fast_val = fast_values.last()?;
        let slow_val = slow_values.last()?;

        if *slow_val != 0.0 {
            Some((fast_val - slow_val) / slow_val)
        } else {
            None
        }
    }

    fn classify_strength(&self, momentum: f64, rsi: f64) -> SignalStrength {
        let momentum_abs = momentum.abs();
        let rsi_extreme = if rsi > 50.0 { rsi - 50.0 } else { 50.0 - rsi };

        if momentum_abs > 0.05 && rsi_extreme > 20.0 {
            SignalStrength::Strong
        } else if momentum_abs > 0.03 && rsi_extreme > 10.0 {
            SignalStrength::Moderate
        } else {
            SignalStrength::Weak
        }
    }
}

impl Strategy for MomentumStrategy {
    fn name(&self) -> &str {
        "Momentum"
    }

    fn description(&self) -> &str {
        "Follows strong trends using momentum and RSI confirmation"
    }

    fn on_bar(&mut self, series: &BarSeries) -> Option<Signal> {
        self.bars_processed += 1;

        if series.len() < self.warmup_period() {
            return None;
        }

        let closes = series.closes();
        let bar = series.last()?;

        // Calculate indicators
        let momentum = self.calculate_momentum(&closes)?;
        let trend = self.calculate_trend(&closes)?;

        let rsi_indicator = Rsi::new(self.config.rsi_period);
        let rsi_values = rsi_indicator.calculate(&closes);
        let rsi = *rsi_values.last()?;

        self.last_momentum = Some(momentum);
        self.last_rsi = Some(rsi);
        self.last_trend = Some(trend);

        let signal = match self.position {
            PositionState::Flat => {
                // Long entry: positive momentum, uptrend, RSI above threshold
                if momentum >= self.config.min_momentum
                    && trend > 0.0
                    && rsi >= self.config.rsi_long_threshold
                {
                    self.position = PositionState::Long;
                    self.signals_generated += 1;
                    Some(Signal {
                        symbol: series.symbol.clone(),
                        signal_type: SignalType::Buy,
                        strength: self.classify_strength(momentum, rsi),
                        price: bar.close,
                        timestamp: bar.timestamp,
                        confidence: (momentum / 0.1).clamp(0.0, 1.0),
                        metadata: SignalMetadata {
                            strategy_name: self.name().to_string(),
                            indicators: [
                                ("momentum".to_string(), momentum),
                                ("trend".to_string(), trend),
                                ("rsi".to_string(), rsi),
                            ]
                            .into_iter()
                            .collect(),
                            reason: format!(
                                "Strong upward momentum ({:.2}%) with RSI at {:.1}",
                                momentum * 100.0,
                                rsi
                            ),
                            ..Default::default()
                        },
                    })
                }
                // Short entry: negative momentum, downtrend, RSI below threshold
                else if self.config.allow_short
                    && momentum <= -self.config.min_momentum
                    && trend < 0.0
                    && rsi <= self.config.rsi_short_threshold
                {
                    self.position = PositionState::Short;
                    self.signals_generated += 1;
                    Some(Signal {
                        symbol: series.symbol.clone(),
                        signal_type: SignalType::Sell,
                        strength: self.classify_strength(momentum, rsi),
                        price: bar.close,
                        timestamp: bar.timestamp,
                        confidence: (momentum.abs() / 0.1).clamp(0.0, 1.0),
                        metadata: SignalMetadata {
                            strategy_name: self.name().to_string(),
                            indicators: [
                                ("momentum".to_string(), momentum),
                                ("trend".to_string(), trend),
                                ("rsi".to_string(), rsi),
                            ]
                            .into_iter()
                            .collect(),
                            reason: format!(
                                "Strong downward momentum ({:.2}%) with RSI at {:.1}",
                                momentum * 100.0,
                                rsi
                            ),
                            ..Default::default()
                        },
                    })
                } else {
                    None
                }
            }
            PositionState::Long => {
                // Exit long: momentum turns negative or trend reverses
                if momentum < 0.0 || trend < 0.0 {
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
                            indicators: [
                                ("momentum".to_string(), momentum),
                                ("trend".to_string(), trend),
                            ]
                            .into_iter()
                            .collect(),
                            reason: "Momentum or trend reversed".to_string(),
                            ..Default::default()
                        },
                    })
                } else {
                    None
                }
            }
            PositionState::Short => {
                // Exit short: momentum turns positive or trend reverses
                if momentum > 0.0 || trend > 0.0 {
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
                            indicators: [
                                ("momentum".to_string(), momentum),
                                ("trend".to_string(), trend),
                            ]
                            .into_iter()
                            .collect(),
                            reason: "Momentum or trend reversed".to_string(),
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
        self.last_momentum = None;
        self.last_rsi = None;
        self.last_trend = None;
    }

    fn state(&self) -> StrategyState {
        StrategyState {
            name: self.name().to_string(),
            is_warmed_up: self.bars_processed >= self.warmup_period(),
            bars_processed: self.bars_processed,
            signals_generated: self.signals_generated,
            indicators: [
                ("momentum".to_string(), self.last_momentum.unwrap_or(0.0)),
                ("rsi".to_string(), self.last_rsi.unwrap_or(50.0)),
                ("trend".to_string(), self.last_trend.unwrap_or(0.0)),
            ]
            .into_iter()
            .collect(),
            custom: serde_json::json!({
                "position": format!("{:?}", self.position),
                "momentum_period": self.config.momentum_period,
                "min_momentum": self.config.min_momentum,
            }),
        }
    }

    fn warmup_period(&self) -> usize {
        self.config
            .slow_ema_period
            .max(self.config.momentum_period + 1)
            .max(self.config.rsi_period + 1)
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
        let mut config = MomentumConfig {
            symbols: vec!["AAPL".to_string()],
            ..Default::default()
        };
        assert!(config.validate().is_ok());

        config.fast_ema_period = 30;
        config.slow_ema_period = 20;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_uptrend_entry() {
        let config = MomentumConfig {
            symbols: vec!["TEST".to_string()],
            momentum_period: 5,
            fast_ema_period: 5,
            slow_ema_period: 10,
            rsi_period: 7,
            rsi_long_threshold: 40.0,
            rsi_short_threshold: 60.0,
            min_momentum: 0.01,
            allow_short: false,
        };

        let mut strategy = MomentumStrategy::new(config);

        // Create prices with consolidation then breakout - produces stronger RSI
        // and clearer momentum signals
        let prices: Vec<f64> = vec![
            100.0, 99.0, 101.0, 100.0, 99.5, 100.5, 100.0, 99.0, 100.0, 99.5, // consolidation
            101.0, 103.0, 105.0, 108.0, 112.0, 115.0, 119.0, 124.0, 128.0, 133.0, // breakout
            138.0, 143.0, 148.0, 153.0, 158.0, 163.0, 168.0, 173.0, 178.0,
            183.0, // strong trend
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

        // Should have a buy signal in the uptrend
        let buy_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.signal_type == SignalType::Buy)
            .collect();
        assert!(!buy_signals.is_empty());
    }
}
