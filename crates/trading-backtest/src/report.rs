//! Backtest report generation.

use serde::{Deserialize, Serialize};
use trading_core::types::Portfolio;

use crate::{BacktestConfig, BacktestStats};

/// Complete backtest report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestReport {
    /// Configuration used
    pub config: BacktestConfig,
    /// Statistics
    pub stats: BacktestStats,
    /// Final portfolio state
    pub final_portfolio: Portfolio,
}

impl BacktestReport {
    /// Generate a text summary.
    pub fn summary(&self) -> String {
        let mut s = String::new();

        s.push_str("═══════════════════════════════════════════════════════════\n");
        s.push_str("                     BACKTEST REPORT                        \n");
        s.push_str("═══════════════════════════════════════════════════════════\n\n");

        s.push_str("PERFORMANCE\n");
        s.push_str("───────────────────────────────────────────────────────────\n");
        s.push_str(&format!(
            "  Initial Capital:     ${:.2}\n",
            self.stats.initial_capital
        ));
        s.push_str(&format!(
            "  Final Equity:        ${:.2}\n",
            self.stats.final_equity
        ));
        s.push_str(&format!(
            "  Total Return:        {:.2}%\n",
            self.stats.total_return_pct
        ));
        s.push_str(&format!(
            "  Annualized Return:   {:.2}%\n",
            self.stats.annualized_return_pct
        ));
        s.push_str(&format!(
            "  Max Drawdown:        {:.2}%\n",
            self.stats.max_drawdown_pct
        ));
        s.push('\n');

        s.push_str("RISK METRICS\n");
        s.push_str("───────────────────────────────────────────────────────────\n");
        s.push_str(&format!(
            "  Sharpe Ratio:        {:.2}\n",
            self.stats.sharpe_ratio
        ));
        s.push_str(&format!(
            "  Sortino Ratio:       {:.2}\n",
            self.stats.sortino_ratio
        ));
        s.push_str(&format!(
            "  Profit Factor:       {:.2}\n",
            self.stats.profit_factor
        ));
        s.push('\n');

        s.push_str("TRADE STATISTICS\n");
        s.push_str("───────────────────────────────────────────────────────────\n");
        s.push_str(&format!(
            "  Total Trades:        {}\n",
            self.stats.total_trades
        ));
        s.push_str(&format!(
            "  Winning Trades:      {}\n",
            self.stats.winning_trades
        ));
        s.push_str(&format!(
            "  Losing Trades:       {}\n",
            self.stats.losing_trades
        ));
        s.push_str(&format!(
            "  Breakeven Trades:    {}\n",
            self.stats.breakeven_trades
        ));
        s.push_str(&format!(
            "  Win Rate:            {:.2}%\n",
            self.stats.win_rate_pct
        ));
        s.push_str(&format!(
            "  Avg Win:             ${:.2}\n",
            self.stats.avg_win
        ));
        s.push_str(&format!(
            "  Avg Loss:            ${:.2}\n",
            self.stats.avg_loss
        ));
        s.push('\n');

        s.push_str("EXECUTION\n");
        s.push_str("───────────────────────────────────────────────────────────\n");
        s.push_str(&format!(
            "  Bars Processed:      {}\n",
            self.stats.bars_processed
        ));
        s.push_str(&format!(
            "  Equity Points:       {}\n",
            self.stats.equity_curve.len()
        ));
        s.push('\n');

        s.push_str("═══════════════════════════════════════════════════════════\n");

        s
    }

    /// Export to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export to CSV (equity curve only).
    pub fn equity_to_csv(&self) -> String {
        let mut csv = String::from("timestamp,equity\n");
        for (ts, equity) in &self.stats.equity_curve {
            csv.push_str(&format!("{},{}\n", ts, equity));
        }
        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_report_summary() {
        let config = BacktestConfig::default();
        let mut stats = BacktestStats::new(dec!(100000));
        stats.final_equity = dec!(110000);
        stats.total_return_pct = dec!(10);
        stats.max_drawdown_pct = dec!(5);
        stats.total_trades = 10;

        let report = BacktestReport {
            config,
            stats,
            final_portfolio: Portfolio::new(dec!(110000)),
        };

        let summary = report.summary();
        assert!(summary.contains("Total Return"));
        assert!(summary.contains("10.00%"));
    }
}
