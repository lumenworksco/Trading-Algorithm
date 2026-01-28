//! Strategy trait definitions.

use crate::error::StrategyError;
use crate::types::{BarSeries, Order, Signal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration trait for strategies.
pub trait StrategyConfig: Send + Sync + Clone + 'static {
    /// Validate the configuration.
    fn validate(&self) -> Result<(), StrategyError>;
}

/// State of a strategy for monitoring and serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyState {
    /// Strategy name
    pub name: String,
    /// Whether the strategy has processed enough bars to generate signals
    pub is_warmed_up: bool,
    /// Number of bars processed
    pub bars_processed: usize,
    /// Number of signals generated
    pub signals_generated: usize,
    /// Current indicator values
    pub indicators: HashMap<String, f64>,
    /// Custom strategy-specific state
    pub custom: serde_json::Value,
}

impl Default for StrategyState {
    fn default() -> Self {
        Self {
            name: String::new(),
            is_warmed_up: false,
            bars_processed: 0,
            signals_generated: 0,
            indicators: HashMap::new(),
            custom: serde_json::Value::Null,
        }
    }
}

/// Core strategy trait.
///
/// All trading strategies must implement this trait to integrate with
/// the trading system. Strategies receive bar data and emit trading signals.
pub trait Strategy: Send + Sync {
    /// Get the unique name of this strategy.
    fn name(&self) -> &str;

    /// Process a new bar and optionally generate a signal.
    ///
    /// This is called for each new bar received. The strategy should
    /// analyze the bar series and return a signal if conditions are met.
    ///
    /// # Arguments
    /// * `series` - The bar series containing historical and current bars
    ///
    /// # Returns
    /// * `Some(Signal)` if a trading action should be taken
    /// * `None` if no action is needed
    fn on_bar(&mut self, series: &BarSeries) -> Option<Signal>;

    /// Called when an order is filled.
    ///
    /// Strategies can use this to track positions and update internal state.
    fn on_fill(&mut self, _order: &Order) {}

    /// Reset the strategy state.
    ///
    /// This is called before backtesting to ensure a clean state.
    fn reset(&mut self);

    /// Get the current strategy state for monitoring.
    fn state(&self) -> StrategyState;

    /// Get the warmup period (number of bars needed before generating signals).
    fn warmup_period(&self) -> usize;

    /// Get the symbols this strategy trades.
    fn symbols(&self) -> &[String];

    /// Check if the strategy is warmed up (has enough data).
    fn is_warmed_up(&self, bars_available: usize) -> bool {
        bars_available >= self.warmup_period()
    }

    /// Get a description of the strategy.
    fn description(&self) -> &str {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStrategy {
        name: String,
        symbols: Vec<String>,
        warmup: usize,
        bars_seen: usize,
    }

    impl Strategy for TestStrategy {
        fn name(&self) -> &str {
            &self.name
        }

        fn on_bar(&mut self, _series: &BarSeries) -> Option<Signal> {
            self.bars_seen += 1;
            None
        }

        fn reset(&mut self) {
            self.bars_seen = 0;
        }

        fn state(&self) -> StrategyState {
            StrategyState {
                name: self.name.clone(),
                is_warmed_up: self.bars_seen >= self.warmup,
                bars_processed: self.bars_seen,
                ..Default::default()
            }
        }

        fn warmup_period(&self) -> usize {
            self.warmup
        }

        fn symbols(&self) -> &[String] {
            &self.symbols
        }
    }

    #[test]
    fn test_strategy_warmup() {
        let strategy = TestStrategy {
            name: "test".to_string(),
            symbols: vec!["AAPL".to_string()],
            warmup: 20,
            bars_seen: 0,
        };

        assert!(!strategy.is_warmed_up(10));
        assert!(!strategy.is_warmed_up(19));
        assert!(strategy.is_warmed_up(20));
        assert!(strategy.is_warmed_up(100));
    }
}
