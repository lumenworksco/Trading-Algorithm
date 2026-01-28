//! Validate configuration command.

use anyhow::Result;
use std::path::Path;
use trading_config::load_config;

pub async fn run(config_path: &Path) -> Result<()> {
    println!("Validating configuration: {:?}", config_path);

    match load_config(config_path) {
        Ok(config) => {
            println!("Configuration is valid!");
            println!();
            println!("App: {}", config.app.name);
            println!("Environment: {}", config.app.environment);
            println!("Log level: {}", config.logging.level);
            println!("Alpaca paper mode: {}", config.alpaca.paper);
            println!("Max position: {}%", config.risk.max_position_pct);
            println!("Max exposure: {}%", config.risk.max_exposure_pct);
            println!("Daily loss limit: {}%", config.risk.daily_loss_limit_pct);
        }
        Err(e) => {
            println!("Configuration error: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
