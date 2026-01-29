//! Paper trading command implementation.

use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::path::Path;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use trading_broker::{AlpacaBroker, AlpacaConfig};
use trading_core::traits::Broker;
use trading_core::types::{BarSeries, OrderRequest, Side, SignalType, Timeframe};
use trading_strategies::StrategyRegistry;

use crate::cli::PaperArgs;

pub async fn run(args: PaperArgs, config_path: &Path) -> Result<()> {
    println!("Starting paper trading...");
    println!("Strategy: {}", args.strategy);
    println!("Symbols: {:?}", args.symbols);
    println!("Capital: ${}", args.capital);
    println!("Timeframe: {}", args.timeframe);
    println!();

    // Parse timeframe
    let timeframe: Timeframe = args.timeframe.parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    // Load Alpaca credentials: try config file first, then environment variables
    let config = if config_path.exists() {
        let app_config = trading_config::load_config(config_path)
            .context("Failed to load config file")?;
        let alpaca = &app_config.alpaca;
        // The config fields contain the actual keys (not env var names)
        AlpacaConfig::new(
            alpaca.api_key_env.clone(),
            alpaca.api_secret_env.clone(),
            alpaca.paper,
        )
    } else {
        AlpacaConfig::from_env()
            .context("Failed to load Alpaca credentials. Set ALPACA_API_KEY and ALPACA_API_SECRET environment variables, or provide a config file.")?
    };

    if !config.paper {
        warn!("Running in LIVE mode! Set ALPACA_PAPER=true for paper trading.");
    }

    let broker = AlpacaBroker::new(config)
        .context("Failed to create Alpaca broker")?;

    // Verify connection
    let account = broker.get_account().await
        .context("Failed to connect to Alpaca API. Check your credentials.")?;

    println!("Connected to {}!", broker.name());
    println!("Account equity: ${}", account.equity);
    println!("Buying power: ${}", account.buying_power);
    println!();

    // Check market status
    let market_open = broker.is_market_open().await
        .context("Failed to check market status")?;

    if !market_open {
        println!("Note: Market is currently CLOSED. Orders will be queued.");
    } else {
        println!("Market is OPEN.");
    }
    println!();

    // Create strategy
    let registry = StrategyRegistry::new();
    let symbols: Vec<String> = args.symbols.clone();

    let mut strategy = registry
        .create_default(&args.strategy, symbols.clone())
        .context("Failed to create strategy")?;

    info!("Strategy initialized: {}", strategy.name());

    // Initialize bar series for each symbol
    let mut series_map: std::collections::HashMap<String, BarSeries> = symbols
        .iter()
        .map(|s| (s.clone(), BarSeries::new(s.clone(), timeframe)))
        .collect();

    // Calculate polling interval based on timeframe
    let poll_interval = match timeframe {
        Timeframe::Minute1 => Duration::from_secs(60),
        Timeframe::Minute5 => Duration::from_secs(60),
        Timeframe::Minute15 => Duration::from_secs(60),
        Timeframe::Minute30 => Duration::from_secs(60),
        Timeframe::Hour1 => Duration::from_secs(300),
        Timeframe::Hour4 => Duration::from_secs(600),
        Timeframe::Daily => Duration::from_secs(3600),
        _ => Duration::from_secs(60),
    };

    println!("Loading historical data for warmup...");

    // Load historical bars for warmup
    let warmup_period = strategy.warmup_period();
    let timeframe_str = match timeframe {
        Timeframe::Minute1 => "1Min",
        Timeframe::Minute5 => "5Min",
        Timeframe::Minute15 => "15Min",
        Timeframe::Minute30 => "30Min",
        Timeframe::Hour1 => "1Hour",
        Timeframe::Hour4 => "4Hour",
        Timeframe::Daily => "1Day",
        _ => "1Day",
    };

    // Get historical data for each symbol
    let end = chrono::Utc::now();
    let start = end - chrono::Duration::days(30); // Get 30 days of data

    for symbol in &symbols {
        match broker.get_bars(
            symbol,
            timeframe_str,
            &start.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            &end.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            Some(warmup_period * 2),
        ).await {
            Ok(bars) => {
                info!("Loaded {} bars for {}", bars.len(), symbol);
                if let Some(series) = series_map.get_mut(symbol) {
                    for bar in bars {
                        series.push(bar);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to load historical data for {}: {}", symbol, e);
            }
        }
    }

    println!("Warmup complete. Starting trading loop...");
    println!("Press Ctrl+C to stop.");
    println!();

    // Trading loop
    let mut interval_timer = interval(poll_interval);
    let mut iteration = 0;

    loop {
        interval_timer.tick().await;
        iteration += 1;

        // Get latest quotes
        let prices = match broker.get_latest_quotes(&symbols).await {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to get quotes: {}", e);
                continue;
            }
        };

        // Update series with latest prices and check for signals
        for symbol in &symbols {
            if let (Some(series), Some(&price)) = (series_map.get_mut(symbol), prices.get(symbol)) {
                // Create a synthetic bar from the latest quote
                let now = chrono::Utc::now().timestamp_millis();
                let price_f64 = price.to_string().parse::<f64>().unwrap_or(0.0);
                let bar = trading_core::types::Bar::new(
                    now,
                    price_f64,
                    price_f64,
                    price_f64,
                    price_f64,
                    0.0,
                );
                series.push(bar);

                // Check for signals
                if let Some(signal) = strategy.on_bar(series) {
                    info!("Signal: {:?} {} @ ${}", signal.signal_type, symbol, signal.price);

                    // Execute signal
                    let result = match signal.signal_type {
                        SignalType::Buy => {
                            // Calculate position size (simplified: use 10% of buying power)
                            let account = broker.get_account().await?;
                            let position_value = account.buying_power * Decimal::from_str_exact("0.1").unwrap();
                            let quantity = (position_value / price).round();

                            if quantity > Decimal::ZERO {
                                let request = OrderRequest::market(symbol, Side::Buy, quantity);
                                broker.submit_order(request).await
                            } else {
                                continue;
                            }
                        }
                        SignalType::Sell | SignalType::CloseLong => {
                            // Close existing position
                            if let Ok(Some(_pos)) = broker.get_position(symbol).await {
                                broker.close_position(symbol).await
                            } else {
                                continue;
                            }
                        }
                        SignalType::CloseShort => {
                            if let Ok(Some(_pos)) = broker.get_position(symbol).await {
                                broker.close_position(symbol).await
                            } else {
                                continue;
                            }
                        }
                        SignalType::Hold => continue,
                    };

                    match result {
                        Ok(order) => {
                            info!("Order submitted: {} {} {} @ {:?}",
                                order.side, order.quantity, order.symbol, order.limit_price);
                        }
                        Err(e) => {
                            error!("Failed to submit order: {}", e);
                        }
                    }
                }
            }
        }

        // Print status every 10 iterations
        if iteration % 10 == 0 {
            match broker.get_account().await {
                Ok(account) => {
                    println!("[{}] Equity: ${:.2} | Positions: {}",
                        chrono::Utc::now().format("%H:%M:%S"),
                        account.equity,
                        account.positions.len()
                    );
                }
                Err(e) => {
                    error!("Failed to get account: {}", e);
                }
            }
        }
    }
}
