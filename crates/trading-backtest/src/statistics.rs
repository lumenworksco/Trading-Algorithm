//! Backtest statistics.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use trading_core::types::{Portfolio, Side, SignalType};

/// Record of a single trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeRecord {
    pub symbol: String,
    pub side: Side,
    pub quantity: Decimal,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub signal_type: SignalType,
    pub pnl: Option<Decimal>,
}

/// Backtest statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestStats {
    /// Initial capital
    pub initial_capital: Decimal,
    /// Final equity
    pub final_equity: Decimal,
    /// Total return percentage
    pub total_return_pct: Decimal,
    /// Annualized return percentage
    pub annualized_return_pct: Decimal,
    /// Maximum drawdown percentage
    pub max_drawdown_pct: Decimal,
    /// Sharpe ratio (assuming risk-free rate of 0)
    pub sharpe_ratio: f64,
    /// Sortino ratio
    pub sortino_ratio: f64,
    /// Total number of trades
    pub total_trades: usize,
    /// Number of winning trades
    pub winning_trades: usize,
    /// Number of losing trades
    pub losing_trades: usize,
    /// Win rate percentage
    pub win_rate_pct: Decimal,
    /// Average profit per winning trade
    pub avg_win: Decimal,
    /// Average loss per losing trade
    pub avg_loss: Decimal,
    /// Profit factor (gross profit / gross loss)
    pub profit_factor: Decimal,
    /// Number of bars processed
    pub bars_processed: usize,
    /// Equity curve
    pub equity_curve: Vec<(i64, Decimal)>,
    /// All trades
    pub trades: Vec<TradeRecord>,
    /// Peak equity (for drawdown)
    peak_equity: Decimal,
    /// Daily returns for Sharpe calculation
    daily_returns: Vec<f64>,
}

impl BacktestStats {
    /// Create new stats tracker.
    pub fn new(initial_capital: Decimal) -> Self {
        Self {
            initial_capital,
            final_equity: initial_capital,
            total_return_pct: Decimal::ZERO,
            annualized_return_pct: Decimal::ZERO,
            max_drawdown_pct: Decimal::ZERO,
            sharpe_ratio: 0.0,
            sortino_ratio: 0.0,
            total_trades: 0,
            winning_trades: 0,
            losing_trades: 0,
            win_rate_pct: Decimal::ZERO,
            avg_win: Decimal::ZERO,
            avg_loss: Decimal::ZERO,
            profit_factor: Decimal::ZERO,
            bars_processed: 0,
            equity_curve: Vec::new(),
            trades: Vec::new(),
            peak_equity: initial_capital,
            daily_returns: Vec::new(),
        }
    }

    /// Record equity at a timestamp.
    pub fn record_equity(&mut self, timestamp: i64, equity: Decimal) {
        // Track daily return
        if let Some((_, prev_equity)) = self.equity_curve.last() {
            if *prev_equity > Decimal::ZERO {
                let ret = ((equity - *prev_equity) / *prev_equity)
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0);
                self.daily_returns.push(ret);
            }
        }

        self.equity_curve.push((timestamp, equity));

        // Update peak and drawdown
        if equity > self.peak_equity {
            self.peak_equity = equity;
        }

        if self.peak_equity > Decimal::ZERO {
            let drawdown = (self.peak_equity - equity) / self.peak_equity * dec!(100);
            if drawdown > self.max_drawdown_pct {
                self.max_drawdown_pct = drawdown;
            }
        }

        self.bars_processed += 1;
    }

    /// Add a trade record.
    pub fn add_trade(&mut self, trade: TradeRecord) {
        self.trades.push(trade);
        self.total_trades += 1;
    }

    /// Calculate final statistics.
    pub fn finalize(&mut self, portfolio: &Portfolio) {
        self.final_equity = portfolio.equity;

        // Total return
        if self.initial_capital > Decimal::ZERO {
            self.total_return_pct =
                (self.final_equity - self.initial_capital) / self.initial_capital * dec!(100);
        }

        // Annualized return (assuming daily bars)
        if !self.equity_curve.is_empty() {
            let days = self.equity_curve.len() as f64;
            let total_return = self.total_return_pct.to_string().parse::<f64>().unwrap_or(0.0) / 100.0;
            let annualized = ((1.0 + total_return).powf(252.0 / days) - 1.0) * 100.0;
            self.annualized_return_pct = Decimal::try_from(annualized).unwrap_or(Decimal::ZERO);
        }

        // Calculate trade statistics
        let mut total_profit = Decimal::ZERO;
        let mut total_loss = Decimal::ZERO;

        for trade in &self.trades {
            if let Some(pnl) = trade.pnl {
                if pnl > Decimal::ZERO {
                    self.winning_trades += 1;
                    total_profit += pnl;
                } else if pnl < Decimal::ZERO {
                    self.losing_trades += 1;
                    total_loss += pnl.abs();
                }
            }
        }

        // Win rate
        if self.total_trades > 0 {
            self.win_rate_pct = Decimal::from(self.winning_trades * 100)
                / Decimal::from(self.total_trades);
        }

        // Average win/loss
        if self.winning_trades > 0 {
            self.avg_win = total_profit / Decimal::from(self.winning_trades);
        }
        if self.losing_trades > 0 {
            self.avg_loss = total_loss / Decimal::from(self.losing_trades);
        }

        // Profit factor
        if total_loss > Decimal::ZERO {
            self.profit_factor = total_profit / total_loss;
        }

        // Sharpe ratio
        if !self.daily_returns.is_empty() {
            let mean: f64 = self.daily_returns.iter().sum::<f64>() / self.daily_returns.len() as f64;
            let variance: f64 = self.daily_returns.iter()
                .map(|r| (r - mean).powi(2))
                .sum::<f64>() / self.daily_returns.len() as f64;
            let std_dev = variance.sqrt();

            if std_dev > 0.0 {
                self.sharpe_ratio = (mean * 252.0_f64.sqrt()) / std_dev;
            }

            // Sortino ratio (only downside deviation)
            let negative_returns: Vec<f64> = self.daily_returns.iter()
                .filter(|&&r| r < 0.0)
                .copied()
                .collect();

            if !negative_returns.is_empty() {
                let downside_variance: f64 = negative_returns.iter()
                    .map(|r| r.powi(2))
                    .sum::<f64>() / negative_returns.len() as f64;
                let downside_dev = downside_variance.sqrt();

                if downside_dev > 0.0 {
                    self.sortino_ratio = (mean * 252.0_f64.sqrt()) / downside_dev;
                }
            }
        }
    }
}
