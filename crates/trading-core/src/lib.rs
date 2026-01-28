//! Core types and traits for the trading system.
//!
//! This crate provides the foundational building blocks including:
//! - Market data types (Bar, BarSeries)
//! - Order and position management types
//! - Trading signals
//! - Core traits for strategies, indicators, brokers, and data sources

pub mod types;
pub mod traits;
pub mod error;

pub use error::{TradingError, TradingResult};
pub use types::*;
pub use traits::*;
