//! Strategy registry for dynamic strategy loading.

use crate::{
    MACrossoverConfig, MACrossoverStrategy, MeanReversionConfig, MeanReversionStrategy,
    MomentumConfig, MomentumStrategy, RsiConfig, RsiStrategy,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use trading_core::{error::StrategyError, traits::Strategy, traits::StrategyConfig};

/// Information about a registered strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyInfo {
    /// Strategy name
    pub name: String,
    /// Strategy description
    pub description: String,
    /// Default configuration as JSON
    pub default_config: serde_json::Value,
}

/// Registry for available trading strategies.
pub struct StrategyRegistry {
    strategies: HashMap<String, StrategyInfo>,
}

impl StrategyRegistry {
    /// Create a new strategy registry with all built-in strategies.
    pub fn new() -> Self {
        let mut strategies = HashMap::new();

        strategies.insert(
            "ma_crossover".to_string(),
            StrategyInfo {
                name: "MA Crossover".to_string(),
                description: "Generates signals based on fast/slow moving average crossovers"
                    .to_string(),
                default_config: serde_json::to_value(MACrossoverConfig::default()).unwrap(),
            },
        );

        strategies.insert(
            "mean_reversion".to_string(),
            StrategyInfo {
                name: "Mean Reversion".to_string(),
                description: "Trades reversions to the mean using Bollinger Bands".to_string(),
                default_config: serde_json::to_value(MeanReversionConfig::default()).unwrap(),
            },
        );

        strategies.insert(
            "momentum".to_string(),
            StrategyInfo {
                name: "Momentum".to_string(),
                description: "Follows strong trends using momentum and RSI confirmation"
                    .to_string(),
                default_config: serde_json::to_value(MomentumConfig::default()).unwrap(),
            },
        );

        strategies.insert(
            "rsi".to_string(),
            StrategyInfo {
                name: "RSI Strategy".to_string(),
                description: "Trades RSI overbought/oversold reversals".to_string(),
                default_config: serde_json::to_value(RsiConfig::default()).unwrap(),
            },
        );

        Self { strategies }
    }

    /// List all available strategies.
    pub fn list(&self) -> Vec<&StrategyInfo> {
        self.strategies.values().collect()
    }

    /// Get strategy info by name.
    pub fn get(&self, name: &str) -> Option<&StrategyInfo> {
        self.strategies.get(name)
    }

    /// Check if a strategy exists.
    pub fn exists(&self, name: &str) -> bool {
        self.strategies.contains_key(name)
    }

    /// Get all strategy names.
    pub fn names(&self) -> Vec<&String> {
        self.strategies.keys().collect()
    }

    /// Create a strategy instance from configuration.
    pub fn create(
        &self,
        name: &str,
        config: serde_json::Value,
        symbols: Vec<String>,
    ) -> Result<Box<dyn Strategy>, StrategyError> {
        match name {
            "ma_crossover" => {
                let mut config: MACrossoverConfig = serde_json::from_value(config)
                    .map_err(|e| StrategyError::InvalidConfig(e.to_string()))?;
                config.symbols = symbols;
                config.validate()?;
                Ok(Box::new(MACrossoverStrategy::new(config)))
            }
            "mean_reversion" => {
                let mut config: MeanReversionConfig = serde_json::from_value(config)
                    .map_err(|e| StrategyError::InvalidConfig(e.to_string()))?;
                config.symbols = symbols;
                config.validate()?;
                Ok(Box::new(MeanReversionStrategy::new(config)))
            }
            "momentum" => {
                let mut config: MomentumConfig = serde_json::from_value(config)
                    .map_err(|e| StrategyError::InvalidConfig(e.to_string()))?;
                config.symbols = symbols;
                config.validate()?;
                Ok(Box::new(MomentumStrategy::new(config)))
            }
            "rsi" => {
                let mut config: RsiConfig = serde_json::from_value(config)
                    .map_err(|e| StrategyError::InvalidConfig(e.to_string()))?;
                config.symbols = symbols;
                config.validate()?;
                Ok(Box::new(RsiStrategy::new(config)))
            }
            _ => Err(StrategyError::NotFound(name.to_string())),
        }
    }

    /// Create a strategy with default configuration.
    pub fn create_default(
        &self,
        name: &str,
        symbols: Vec<String>,
    ) -> Result<Box<dyn Strategy>, StrategyError> {
        let info = self
            .get(name)
            .ok_or_else(|| StrategyError::NotFound(name.to_string()))?;
        self.create(name, info.default_config.clone(), symbols)
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_list() {
        let registry = StrategyRegistry::new();
        let strategies = registry.list();

        assert_eq!(strategies.len(), 4);
    }

    #[test]
    fn test_registry_get() {
        let registry = StrategyRegistry::new();

        assert!(registry.get("ma_crossover").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_create_default() {
        let registry = StrategyRegistry::new();

        let strategy = registry.create_default("ma_crossover", vec!["AAPL".to_string()]);
        assert!(strategy.is_ok());

        let strategy = strategy.unwrap();
        assert_eq!(strategy.name(), "MA Crossover");
        assert_eq!(strategy.symbols(), &["AAPL".to_string()]);
    }

    #[test]
    fn test_create_with_config() {
        let registry = StrategyRegistry::new();

        let config = serde_json::json!({
            "symbols": [],
            "fast_period": 5,
            "slow_period": 10,
            "use_ema": true,
            "signal_threshold": 0.001
        });

        let strategy = registry.create("ma_crossover", config, vec!["GOOGL".to_string()]);
        assert!(strategy.is_ok());
    }

    #[test]
    fn test_create_unknown_strategy() {
        let registry = StrategyRegistry::new();

        let result = registry.create_default("unknown", vec!["AAPL".to_string()]);
        assert!(result.is_err());
    }
}
