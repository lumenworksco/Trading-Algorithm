//! Technical indicators with SIMD optimization.
//!
//! This crate provides efficient implementations of common technical indicators:
//! - Moving averages (SMA, EMA, WMA)
//! - Momentum indicators (RSI, MACD, Stochastic)
//! - Volatility indicators (ATR, Bollinger Bands, Standard Deviation)
//!
//! Many indicators have SIMD-optimized implementations for improved performance
//! during backtesting over large datasets.

pub mod momentum;
pub mod moving_average;
pub mod simd;
pub mod volatility;

pub use momentum::{Macd, MacdOutput, Rsi, Stochastic, StochasticOutput};
pub use moving_average::{Ema, Sma, Wma};
pub use volatility::{Atr, BollingerBands, BollingerOutput, StdDev};
