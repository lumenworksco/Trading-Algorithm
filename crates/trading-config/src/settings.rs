//! Configuration structures.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use trading_risk::{PositionSizingMethod, StopLossMethod};

/// Main application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub app: AppSettings,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub alpaca: AlpacaConfig,
    #[serde(default)]
    pub risk: RiskSettings,
    #[serde(default)]
    pub backtest: BacktestSettings,
}

/// General app settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub name: String,
    pub environment: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            name: "trading-system".to_string(),
            environment: "development".to_string(),
        }
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            file: None,
        }
    }
}

/// Alpaca API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaConfig {
    pub api_key_env: String,
    pub api_secret_env: String,
    pub base_url: String,
    pub paper: bool,
}

impl Default for AlpacaConfig {
    fn default() -> Self {
        Self {
            api_key_env: "ALPACA_API_KEY".to_string(),
            api_secret_env: "ALPACA_API_SECRET".to_string(),
            base_url: "https://paper-api.alpaca.markets".to_string(),
            paper: true,
        }
    }
}

/// Risk management settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSettings {
    pub max_position_pct: Decimal,
    pub max_exposure_pct: Decimal,
    pub daily_loss_limit_pct: Decimal,
    pub max_drawdown_pct: Decimal,
    pub position_sizing: PositionSizingMethod,
    pub stop_loss: StopLossMethod,
}

impl Default for RiskSettings {
    fn default() -> Self {
        use rust_decimal_macros::dec;
        Self {
            max_position_pct: dec!(10),
            max_exposure_pct: dec!(80),
            daily_loss_limit_pct: dec!(3),
            max_drawdown_pct: dec!(20),
            position_sizing: PositionSizingMethod::PercentEquity { percent: dec!(2) },
            stop_loss: StopLossMethod::FixedPercent { percent: dec!(2) },
        }
    }
}

/// Backtest settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestSettings {
    pub default_capital: Decimal,
    pub commission: Decimal,
    pub slippage_pct: Decimal,
}

impl Default for BacktestSettings {
    fn default() -> Self {
        use rust_decimal_macros::dec;
        Self {
            default_capital: dec!(100000),
            commission: Decimal::ZERO,
            slippage_pct: dec!(0.05),
        }
    }
}
