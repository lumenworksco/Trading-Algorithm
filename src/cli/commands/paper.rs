//! Paper trading command implementation.

use anyhow::Result;
use std::path::Path;
use tracing::info;

use crate::cli::PaperArgs;

pub async fn run(args: PaperArgs, _config_path: &Path) -> Result<()> {
    info!("Paper trading is not yet implemented");
    info!("Strategy: {}", args.strategy);
    info!("Symbols: {:?}", args.symbols);
    info!("Capital: ${}", args.capital);
    info!("Timeframe: {}", args.timeframe);

    println!("Paper trading with real-time data requires Alpaca API credentials.");
    println!("Please set ALPACA_API_KEY and ALPACA_API_SECRET environment variables.");
    println!("\nFor now, use the 'backtest' command with historical CSV data.");

    Ok(())
}
