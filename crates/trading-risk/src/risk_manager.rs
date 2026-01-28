//! Unified risk manager.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use trading_core::types::{OrderRequest, Portfolio, Signal, Side};

use crate::{
    PortfolioLimits, PositionSizer, PositionSizingMethod,
    StopLossManager, StopLossMethod, LimitCheck,
};

/// Risk management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Position sizing method
    pub position_sizing: PositionSizingMethod,
    /// Stop-loss method
    pub stop_loss: StopLossMethod,
    /// Portfolio limits
    pub limits: PortfolioLimits,
    /// Maximum shares per order
    pub max_shares: Option<Decimal>,
    /// Use signal strength for sizing
    pub use_signal_strength: bool,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            position_sizing: PositionSizingMethod::PercentEquity { percent: dec!(2) },
            stop_loss: StopLossMethod::FixedPercent { percent: dec!(2) },
            limits: PortfolioLimits::default(),
            max_shares: Some(dec!(1000)),
            use_signal_strength: true,
        }
    }
}

/// Decision from the risk manager.
#[derive(Debug, Clone)]
pub enum RiskDecision {
    /// Order approved with calculated parameters
    Approved {
        order: OrderRequest,
        stop_loss_price: Option<Decimal>,
    },
    /// Order rejected with reason
    Rejected { reason: String },
    /// Order modified (size reduced)
    Modified {
        order: OrderRequest,
        stop_loss_price: Option<Decimal>,
        reason: String,
    },
}

impl RiskDecision {
    pub fn is_approved(&self) -> bool {
        matches!(self, RiskDecision::Approved { .. } | RiskDecision::Modified { .. })
    }

    pub fn order(&self) -> Option<&OrderRequest> {
        match self {
            RiskDecision::Approved { order, .. } => Some(order),
            RiskDecision::Modified { order, .. } => Some(order),
            RiskDecision::Rejected { .. } => None,
        }
    }
}

/// Unified risk manager that combines position sizing, stop-loss, and limits.
pub struct RiskManager {
    config: RiskConfig,
    position_sizer: PositionSizer,
    stop_loss_manager: StopLossManager,
    daily_pnl: Decimal,
}

impl RiskManager {
    /// Create a new risk manager.
    pub fn new(config: RiskConfig) -> Self {
        let mut position_sizer = PositionSizer::new(config.position_sizing.clone());
        if let Some(max) = config.max_shares {
            position_sizer = position_sizer.with_max_shares(max);
        }
        if !config.use_signal_strength {
            position_sizer = position_sizer.without_signal_strength();
        }

        let stop_loss_manager = StopLossManager::new(config.stop_loss.clone());

        Self {
            config,
            position_sizer,
            stop_loss_manager,
            daily_pnl: Decimal::ZERO,
        }
    }

    /// Update the daily P&L tracking.
    pub fn update_daily_pnl(&mut self, pnl: Decimal) {
        self.daily_pnl = pnl;
    }

    /// Reset daily P&L (call at start of trading day).
    pub fn reset_daily_pnl(&mut self) {
        self.daily_pnl = Decimal::ZERO;
    }

    /// Update ATR for stop-loss calculations.
    pub fn update_atr(&mut self, atr: Decimal) {
        self.stop_loss_manager.update_atr(atr);
    }

    /// Evaluate a signal and produce a risk decision.
    pub fn evaluate_signal(
        &self,
        portfolio: &Portfolio,
        signal: &Signal,
        current_price: Decimal,
    ) -> RiskDecision {
        // Determine side based on signal
        let side = match signal.signal_type {
            trading_core::types::SignalType::Buy => Side::Buy,
            trading_core::types::SignalType::Sell => Side::Sell,
            trading_core::types::SignalType::CloseLong => Side::Sell,
            trading_core::types::SignalType::CloseShort => Side::Buy,
            trading_core::types::SignalType::Hold => {
                return RiskDecision::Rejected {
                    reason: "Hold signal - no action needed".to_string(),
                };
            }
        };

        // Calculate stop-loss price
        let stop_loss_price = self.stop_loss_manager.calculate_stop_price(current_price, side);

        // Calculate position size
        let quantity = self.position_sizer.calculate(
            portfolio,
            signal,
            current_price,
            stop_loss_price,
        );

        if quantity <= Decimal::ZERO {
            return RiskDecision::Rejected {
                reason: "Calculated position size is zero or negative".to_string(),
            };
        }

        // Calculate position value
        let position_value = quantity * current_price;

        // Check portfolio limits
        let limit_check = self.config.limits.check_new_position(
            portfolio,
            position_value,
            self.daily_pnl,
        );

        match limit_check {
            LimitCheck::Blocked { reason } => {
                RiskDecision::Rejected { reason }
            }

            LimitCheck::Reduced { max_size, reason } => {
                let reduced_quantity = (max_size / current_price).floor();
                if reduced_quantity <= Decimal::ZERO {
                    return RiskDecision::Rejected {
                        reason: format!("Position too small after reduction: {}", reason),
                    };
                }

                let order = OrderRequest::market(&signal.symbol, side, reduced_quantity);

                RiskDecision::Modified {
                    order,
                    stop_loss_price,
                    reason,
                }
            }

            LimitCheck::Allowed => {
                let order = OrderRequest::market(&signal.symbol, side, quantity);

                RiskDecision::Approved {
                    order,
                    stop_loss_price,
                }
            }
        }
    }

    /// Check if trading should be halted.
    pub fn should_halt(&self, portfolio: &Portfolio) -> Option<String> {
        self.config.limits.should_halt_trading(portfolio, self.daily_pnl)
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RiskConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trading_core::types::{SignalMetadata, SignalStrength, SignalType};

    fn create_portfolio() -> Portfolio {
        let mut portfolio = Portfolio::new(dec!(100000));
        portfolio.cash = dec!(100000);
        portfolio.buying_power = dec!(100000);
        portfolio
    }

    fn create_signal(signal_type: SignalType) -> Signal {
        Signal {
            symbol: "TEST".to_string(),
            signal_type,
            strength: SignalStrength::Moderate,
            price: 100.0,
            timestamp: 0,
            confidence: 1.0,
            metadata: SignalMetadata::default(),
        }
    }

    #[test]
    fn test_approved_signal() {
        let config = RiskConfig::default();
        let manager = RiskManager::new(config);
        let portfolio = create_portfolio();
        let signal = create_signal(SignalType::Buy);

        let decision = manager.evaluate_signal(&portfolio, &signal, dec!(100));
        assert!(decision.is_approved());

        if let RiskDecision::Approved { order, stop_loss_price } = decision {
            assert_eq!(order.symbol, "TEST");
            assert_eq!(order.side, Side::Buy);
            assert!(order.quantity > Decimal::ZERO);
            assert!(stop_loss_price.is_some());
        }
    }

    #[test]
    fn test_hold_signal_rejected() {
        let config = RiskConfig::default();
        let manager = RiskManager::new(config);
        let portfolio = create_portfolio();
        let signal = create_signal(SignalType::Hold);

        let decision = manager.evaluate_signal(&portfolio, &signal, dec!(100));
        assert!(!decision.is_approved());
    }

    #[test]
    fn test_daily_loss_halt() {
        let config = RiskConfig::default();
        let mut manager = RiskManager::new(config);
        let portfolio = create_portfolio();

        // No halt initially
        assert!(manager.should_halt(&portfolio).is_none());

        // Update with big loss
        manager.update_daily_pnl(dec!(-5000)); // 5% loss

        // Should halt now
        assert!(manager.should_halt(&portfolio).is_some());
    }
}
