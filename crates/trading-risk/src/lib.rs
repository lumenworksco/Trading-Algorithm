//! Risk management for trading.
//!
//! Provides position sizing, stop-loss management, and portfolio limits.

mod portfolio_limits;
mod position_sizer;
mod risk_manager;
mod stop_loss;

pub use portfolio_limits::{LimitCheck, PortfolioLimits};
pub use position_sizer::{PositionSizer, PositionSizingMethod};
pub use risk_manager::{RiskConfig, RiskDecision, RiskManager};
pub use stop_loss::{StopLossManager, StopLossMethod, StopLossOrder};
