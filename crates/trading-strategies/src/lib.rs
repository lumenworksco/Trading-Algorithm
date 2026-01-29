//! Trading strategy implementations.
//!
//! This crate provides implementations of common trading strategies:
//! - Moving Average Crossover
//! - Mean Reversion (Bollinger Bands)
//! - Momentum/Trend Following
//! - RSI-based trading

mod ma_crossover;
mod mean_reversion;
mod momentum;
mod registry;
mod rsi_strategy;

pub use ma_crossover::{MACrossoverConfig, MACrossoverStrategy};
pub use mean_reversion::{MeanReversionConfig, MeanReversionStrategy};
pub use momentum::{MomentumConfig, MomentumStrategy};
pub use registry::{StrategyInfo, StrategyRegistry};
pub use rsi_strategy::{RsiConfig, RsiStrategy};
