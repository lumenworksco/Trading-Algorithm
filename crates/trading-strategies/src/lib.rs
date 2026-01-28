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
mod rsi_strategy;
mod registry;

pub use ma_crossover::{MACrossoverStrategy, MACrossoverConfig};
pub use mean_reversion::{MeanReversionStrategy, MeanReversionConfig};
pub use momentum::{MomentumStrategy, MomentumConfig};
pub use rsi_strategy::{RsiStrategy, RsiConfig};
pub use registry::{StrategyRegistry, StrategyInfo};
