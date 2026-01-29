//! Backtest command implementation.

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::path::Path;
use trading_backtest::{BacktestConfig, BacktestEngine};
use trading_data::CsvDataSource;
use trading_risk::RiskConfig;
use trading_strategies::StrategyRegistry;
use tracing::info;

use crate::cli::BacktestArgs;

pub async fn run(args: BacktestArgs, _config_path: &Path) -> Result<()> {
    info!("Starting backtest for strategy: {}", args.strategy);

    // Create strategy
    let registry = StrategyRegistry::new();
    let mut strategy = registry
        .create_default(&args.strategy, args.symbols.clone())
        .context("Failed to create strategy")?;

    // Load data
    let data = if let Some(data_path) = &args.data {
        if !data_path.exists() {
            anyhow::bail!(
                "Data path '{}' does not exist. Provide a CSV file or directory containing CSV files (e.g. --data ./data)",
                data_path.display()
            );
        }
        load_data_from_csv(data_path, &args.symbols).await?
    } else {
        anyhow::bail!("Please provide a data file or directory with --data (e.g. --data ./data)");
    };

    // Create backtest config
    let capital = Decimal::try_from(args.capital).unwrap_or_default();
    let backtest_config = BacktestConfig {
        initial_capital: capital,
        commission: Decimal::ZERO,
        slippage_pct: Decimal::try_from(0.05).unwrap(),
        risk_config: RiskConfig::default(),
    };

    // Run backtest
    let engine = BacktestEngine::new(backtest_config);
    let report = engine.run(strategy.as_mut(), data).await;

    // Output results
    match args.output.as_str() {
        "json" => {
            let json = report.to_json()?;
            println!("{}", json);
        }
        _ => {
            println!("{}", report.summary());
        }
    }

    // Save if requested
    if let Some(save_path) = &args.save {
        let json = report.to_json()?;
        std::fs::write(save_path, json)?;
        info!("Results saved to {:?}", save_path);
    }

    Ok(())
}

async fn load_data_from_csv(
    path: &Path,
    symbols: &[String],
) -> Result<HashMap<String, Vec<trading_core::types::Bar>>> {
    let mut data = HashMap::new();

    // If path is a file, load it for the first symbol
    if path.is_file() {
        let source = CsvDataSource::new(path.to_str().unwrap())?;
        let symbol = symbols.first().cloned().unwrap_or_else(|| "DATA".to_string());
        let bars = source
            .load_all(&symbol, trading_core::types::Timeframe::Daily)
            .await?;
        data.insert(symbol, bars);
    } else {
        // If path is a directory, look for files named {symbol}.csv or {symbol}_daily.csv
        for symbol in symbols {
            let lower = symbol.to_lowercase();
            let candidates = [
                path.join(format!("{}.csv", symbol)),
                path.join(format!("{}.csv", lower)),
                path.join(format!("{}_daily.csv", symbol)),
                path.join(format!("{}_daily.csv", lower)),
            ];
            for file_path in &candidates {
                if file_path.exists() {
                    let source = CsvDataSource::new(file_path.to_str().unwrap())?;
                    let bars = source
                        .load_all(symbol, trading_core::types::Timeframe::Daily)
                        .await?;
                    data.insert(symbol.clone(), bars);
                    break;
                }
            }
        }
    }

    if data.is_empty() {
        anyhow::bail!("No data loaded");
    }

    info!("Loaded data for {} symbols", data.len());
    Ok(data)
}
