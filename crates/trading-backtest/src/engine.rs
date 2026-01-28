//! Backtesting engine.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use trading_broker::PaperBroker;
use trading_core::traits::{Broker, Strategy};
use trading_core::types::{Bar, BarSeries, Portfolio, SignalType, Timeframe};
use trading_risk::{RiskConfig, RiskManager};

use crate::statistics::{BacktestStats, TradeRecord};
use crate::report::BacktestReport;

/// Backtest configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// Initial capital
    pub initial_capital: Decimal,
    /// Commission per share
    pub commission: Decimal,
    /// Slippage percentage
    pub slippage_pct: Decimal,
    /// Risk configuration
    pub risk_config: RiskConfig,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: dec!(100000),
            commission: Decimal::ZERO,
            slippage_pct: dec!(0.05),
            risk_config: RiskConfig::default(),
        }
    }
}

/// Backtesting engine.
pub struct BacktestEngine {
    config: BacktestConfig,
}

impl BacktestEngine {
    /// Create a new backtest engine.
    pub fn new(config: BacktestConfig) -> Self {
        Self { config }
    }

    /// Run a backtest.
    pub async fn run(
        &self,
        strategy: &mut dyn Strategy,
        data: HashMap<String, Vec<Bar>>,
    ) -> BacktestReport {
        let broker = PaperBroker::new(self.config.initial_capital)
            .with_slippage(self.config.slippage_pct)
            .with_commission(self.config.commission);

        let risk_manager = RiskManager::new(self.config.risk_config.clone());

        let mut stats = BacktestStats::new(self.config.initial_capital);
        let mut series_map: HashMap<String, BarSeries> = HashMap::new();

        // Initialize bar series
        for symbol in data.keys() {
            series_map.insert(
                symbol.clone(),
                BarSeries::with_capacity(symbol.clone(), Timeframe::Daily, 500),
            );
        }

        // Get all timestamps and sort them
        let mut all_timestamps: Vec<(i64, String, Bar)> = Vec::new();
        for (symbol, bars) in &data {
            for bar in bars {
                all_timestamps.push((bar.timestamp, symbol.clone(), *bar));
            }
        }
        all_timestamps.sort_by_key(|(ts, _, _)| *ts);

        // Process bars in chronological order
        for (timestamp, symbol, bar) in all_timestamps {
            // Add bar to series
            if let Some(series) = series_map.get_mut(&symbol) {
                series.push(bar);

                // Get signal from strategy
                if let Some(signal) = strategy.on_bar(series) {
                    // Evaluate with risk manager
                    let current_price = Decimal::try_from(bar.close).unwrap_or(dec!(0));
                    let portfolio = broker.get_account().await.unwrap();
                    let decision = risk_manager.evaluate_signal(&portfolio, &signal, current_price);

                    if let Some(order_request) = decision.order() {
                        // Submit and execute order
                        if let Ok(order) = broker.submit_order(order_request.clone()).await {
                            if let Ok(filled) = broker.execute_at_price(order.id, current_price) {
                                // Record trade
                                let trade = TradeRecord {
                                    symbol: symbol.clone(),
                                    side: order_request.side,
                                    quantity: filled.filled_quantity,
                                    price: filled.filled_avg_price.unwrap_or(current_price),
                                    timestamp: DateTime::from_timestamp_millis(timestamp)
                                        .unwrap_or_else(|| Utc::now()),
                                    signal_type: signal.signal_type,
                                    pnl: None, // Calculated later
                                };
                                stats.add_trade(trade);
                            }
                        }
                    }
                }
            }

            // Update prices for all positions
            let mut prices = HashMap::new();
            for (sym, bars) in &data {
                if let Some(b) = bars.iter().find(|b| b.timestamp == timestamp) {
                    prices.insert(sym.clone(), Decimal::try_from(b.close).unwrap_or(dec!(0)));
                }
            }
            broker.update_prices(&prices);

            // Record equity
            let portfolio = broker.get_account().await.unwrap();
            stats.record_equity(timestamp, portfolio.equity);
        }

        // Final statistics
        let final_portfolio = broker.get_account().await.unwrap();
        stats.finalize(&final_portfolio);

        BacktestReport {
            config: self.config.clone(),
            stats,
            final_portfolio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trading_strategies::{MACrossoverConfig, MACrossoverStrategy};

    fn generate_test_data() -> HashMap<String, Vec<Bar>> {
        let mut data = HashMap::new();
        let bars: Vec<Bar> = (0..100)
            .map(|i| {
                let price = 100.0 + (i as f64 * 0.5).sin() * 10.0;
                Bar::new(
                    i as i64 * 86400000,
                    price,
                    price + 2.0,
                    price - 2.0,
                    price + 1.0,
                    1000000.0,
                )
            })
            .collect();
        data.insert("TEST".to_string(), bars);
        data
    }

    #[tokio::test]
    async fn test_backtest_runs() {
        let config = BacktestConfig::default();
        let engine = BacktestEngine::new(config);

        let strategy_config = MACrossoverConfig {
            symbols: vec!["TEST".to_string()],
            fast_period: 5,
            slow_period: 10,
            use_ema: true,
            signal_threshold: 0.0,
        };
        let mut strategy = MACrossoverStrategy::new(strategy_config);

        let data = generate_test_data();
        let report = engine.run(&mut strategy, data).await;

        assert!(report.stats.bars_processed > 0);
    }
}
