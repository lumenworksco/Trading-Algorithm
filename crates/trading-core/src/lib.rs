//! Core types and traits for the trading system.
//!
//! This crate provides the foundational building blocks including:
//! - Market data types (Bar, BarSeries)
//! - Order and position management types
//! - Trading signals
//! - Core traits for strategies, indicators, brokers, and data sources

pub mod error;
pub mod traits;
pub mod types;

pub use error::{TradingError, TradingResult};
pub use traits::*;
pub use types::*;
