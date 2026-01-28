//! Core traits for the trading system.

mod strategy;
mod indicator;
mod broker;
mod data_source;

pub use strategy::{Strategy, StrategyConfig, StrategyState};
pub use indicator::{Indicator, StreamingIndicator, MultiOutputIndicator};
pub use broker::Broker;
pub use data_source::{DataSource, QuoteSource, Quote};
