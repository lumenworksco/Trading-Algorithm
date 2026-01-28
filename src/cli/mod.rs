//! CLI definitions.

pub mod commands;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "trading")]
#[command(author, version, about = "High-performance algorithmic trading system")]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config/default.toml")]
    pub config: PathBuf,

    /// Log level
    #[arg(short, long, default_value = "info")]
    pub log_level: LogLevel,

    /// Enable JSON log format
    #[arg(long)]
    pub json_logs: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run backtesting simulation
    Backtest(BacktestArgs),
    /// Start live trading
    Live(LiveArgs),
    /// Start paper trading
    Paper(PaperArgs),
    /// List available strategies
    Strategies,
    /// Validate configuration
    ValidateConfig,
}

#[derive(clap::Args)]
pub struct BacktestArgs {
    /// Strategy to backtest
    #[arg(short, long)]
    pub strategy: String,

    /// Symbols to trade (comma-separated)
    #[arg(short = 'S', long, value_delimiter = ',')]
    pub symbols: Vec<String>,

    /// Start date (YYYY-MM-DD)
    #[arg(long)]
    pub start: String,

    /// End date (YYYY-MM-DD)
    #[arg(long)]
    pub end: String,

    /// Initial capital
    #[arg(long, default_value = "100000")]
    pub capital: f64,

    /// Timeframe
    #[arg(short, long, default_value = "1d")]
    pub timeframe: String,

    /// Strategy configuration file
    #[arg(long)]
    pub strategy_config: Option<PathBuf>,

    /// Output format (text, json)
    #[arg(long, default_value = "text")]
    pub output: String,

    /// Save results to file
    #[arg(long)]
    pub save: Option<PathBuf>,

    /// Data file (CSV)
    #[arg(long)]
    pub data: Option<PathBuf>,
}

#[derive(clap::Args)]
pub struct LiveArgs {
    /// Strategy to run
    #[arg(short, long)]
    pub strategy: String,

    /// Symbols to trade (comma-separated)
    #[arg(short = 'S', long, value_delimiter = ',')]
    pub symbols: Vec<String>,

    /// Timeframe
    #[arg(short, long, default_value = "1m")]
    pub timeframe: String,

    /// Enable dry run (no real orders)
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(clap::Args)]
pub struct PaperArgs {
    /// Strategy to run
    #[arg(short, long)]
    pub strategy: String,

    /// Symbols to trade (comma-separated)
    #[arg(short = 'S', long, value_delimiter = ',')]
    pub symbols: Vec<String>,

    /// Initial capital
    #[arg(long, default_value = "100000")]
    pub capital: f64,

    /// Timeframe
    #[arg(short, long, default_value = "1m")]
    pub timeframe: String,
}
