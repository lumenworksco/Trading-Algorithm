//! Backtesting engine.

mod engine;
mod report;
mod statistics;

pub use engine::{BacktestConfig, BacktestEngine};
pub use report::BacktestReport;
pub use statistics::{BacktestStats, TradeRecord};
