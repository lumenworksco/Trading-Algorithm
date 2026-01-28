//! Trading system CLI application.

mod cli;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use trading_monitor::setup_logging;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = match cli.log_level {
        cli::LogLevel::Trace => "trace",
        cli::LogLevel::Debug => "debug",
        cli::LogLevel::Info => "info",
        cli::LogLevel::Warn => "warn",
        cli::LogLevel::Error => "error",
    };
    setup_logging(log_level, cli.json_logs);

    // Execute command
    match cli.command {
        Commands::Backtest(args) => cli::commands::backtest::run(args, &cli.config).await,
        Commands::Live(args) => cli::commands::live::run(args, &cli.config).await,
        Commands::Paper(args) => cli::commands::paper::run(args, &cli.config).await,
        Commands::Strategies => cli::commands::strategies::run().await,
        Commands::ValidateConfig => cli::commands::validate::run(&cli.config).await,
    }
}
