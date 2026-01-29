//! Configuration management.

mod settings;

pub use settings::{AlpacaConfig, AppConfig, LoggingConfig, RiskSettings};

use config::{Config, ConfigError, Environment, File};
use std::path::Path;

/// Load configuration from file and environment.
pub fn load_config(path: &Path) -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .add_source(File::from(path).required(true))
        .add_source(
            Environment::with_prefix("TRADING")
                .separator("__")
                .try_parsing(true),
        )
        .build()?;

    config.try_deserialize()
}
