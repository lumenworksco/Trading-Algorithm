//! Live trading command implementation.

use anyhow::Result;
use std::path::Path;
use tracing::info;

use crate::cli::LiveArgs;

pub async fn run(args: LiveArgs, _config_path: &Path) -> Result<()> {
    info!("Live trading is not yet implemented");
    info!("Strategy: {}", args.strategy);
    info!("Symbols: {:?}", args.symbols);
    info!("Timeframe: {}", args.timeframe);
    info!("Dry run: {}", args.dry_run);

    println!("Live trading requires Alpaca API credentials.");
    println!("Please set ALPACA_API_KEY and ALPACA_API_SECRET environment variables.");
    println!("\nThis feature will be available in a future release.");

    Ok(())
}
