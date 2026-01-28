//! Backtesting engine.

mod engine;
mod statistics;
mod report;

pub use engine::{BacktestEngine, BacktestConfig};
pub use statistics::{BacktestStats, TradeRecord};
pub use report::BacktestReport;
