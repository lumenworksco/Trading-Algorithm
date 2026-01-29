//! Core traits for the trading system.

mod broker;
mod data_source;
mod indicator;
mod strategy;

pub use broker::Broker;
pub use data_source::{DataSource, Quote, QuoteSource};
pub use indicator::{Indicator, MultiOutputIndicator, StreamingIndicator};
pub use strategy::{Strategy, StrategyConfig, StrategyState};
