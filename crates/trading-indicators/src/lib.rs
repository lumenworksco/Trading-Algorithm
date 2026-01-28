//! Technical indicators with SIMD optimization.
//!
//! This crate provides efficient implementations of common technical indicators:
//! - Moving averages (SMA, EMA, WMA)
//! - Momentum indicators (RSI, MACD, Stochastic)
//! - Volatility indicators (ATR, Bollinger Bands, Standard Deviation)
//!
//! Many indicators have SIMD-optimized implementations for improved performance
//! during backtesting over large datasets.

pub mod moving_average;
pub mod momentum;
pub mod volatility;
pub mod simd;

pub use moving_average::{Sma, Ema, Wma};
pub use momentum::{Rsi, Macd, MacdOutput, Stochastic, StochasticOutput};
pub use volatility::{Atr, BollingerBands, BollingerOutput, StdDev};
