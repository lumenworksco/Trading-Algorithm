//! Broker integrations.

mod paper;

pub use paper::PaperBroker;

use trading_core::types::{Order, OrderRequest, Position, Portfolio};
use trading_core::error::BrokerError;
use async_trait::async_trait;
