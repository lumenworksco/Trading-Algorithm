//! Portfolio-level risk limits.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use trading_core::types::Portfolio;

/// Result of a limit check.
#[derive(Debug, Clone)]
pub enum LimitCheck {
    /// Trade allowed
    Allowed,
    /// Trade blocked with reason
    Blocked { reason: String },
    /// Trade allowed but with reduced size
    Reduced { max_size: Decimal, reason: String },
}

impl LimitCheck {
    pub fn is_allowed(&self) -> bool {
        matches!(self, LimitCheck::Allowed | LimitCheck::Reduced { .. })
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self, LimitCheck::Blocked { .. })
    }
}

/// Portfolio-level limits configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioLimits {
    /// Maximum position size as percentage of equity
    pub max_position_pct: Decimal,
    /// Maximum total exposure as percentage of equity
    pub max_exposure_pct: Decimal,
    /// Maximum number of open positions
    pub max_positions: usize,
    /// Maximum loss per day as percentage of equity
    pub daily_loss_limit_pct: Decimal,
    /// Maximum drawdown before stopping trading
    pub max_drawdown_pct: Decimal,
    /// Minimum cash to maintain
    pub min_cash: Decimal,
    /// Maximum concentration in any single position
    pub max_concentration_pct: Decimal,
}

impl Default for PortfolioLimits {
    fn default() -> Self {
        Self {
            max_position_pct: dec!(10), // 10% max per position
            max_exposure_pct: dec!(80), // 80% max invested
            max_positions: 10,
            daily_loss_limit_pct: dec!(3), // Stop if down 3% today
            max_drawdown_pct: dec!(20),    // Stop if 20% drawdown
            min_cash: dec!(1000),
            max_concentration_pct: dec!(25), // No position > 25% of portfolio
        }
    }
}

impl PortfolioLimits {
    /// Check if a new position is allowed.
    pub fn check_new_position(
        &self,
        portfolio: &Portfolio,
        position_value: Decimal,
        daily_pnl: Decimal,
    ) -> LimitCheck {
        // Check daily loss limit
        let daily_loss_pct = if portfolio.initial_capital > Decimal::ZERO {
            (daily_pnl / portfolio.initial_capital) * dec!(100)
        } else {
            Decimal::ZERO
        };

        if daily_loss_pct <= -self.daily_loss_limit_pct {
            return LimitCheck::Blocked {
                reason: format!(
                    "Daily loss limit reached: {:.2}% (limit: {:.2}%)",
                    daily_loss_pct, self.daily_loss_limit_pct
                ),
            };
        }

        // Check max drawdown
        let drawdown = portfolio.drawdown();
        if drawdown >= self.max_drawdown_pct {
            return LimitCheck::Blocked {
                reason: format!(
                    "Max drawdown exceeded: {:.2}% (limit: {:.2}%)",
                    drawdown, self.max_drawdown_pct
                ),
            };
        }

        // Check max positions
        if portfolio.position_count() >= self.max_positions {
            return LimitCheck::Blocked {
                reason: format!(
                    "Max positions reached: {} (limit: {})",
                    portfolio.position_count(),
                    self.max_positions
                ),
            };
        }

        // Check min cash
        if portfolio.cash - position_value < self.min_cash {
            let max_allowed = portfolio.cash - self.min_cash;
            if max_allowed <= Decimal::ZERO {
                return LimitCheck::Blocked {
                    reason: format!(
                        "Insufficient cash: ${:.2} (need ${:.2} minimum)",
                        portfolio.cash, self.min_cash
                    ),
                };
            }
            return LimitCheck::Reduced {
                max_size: max_allowed,
                reason: "Limited by minimum cash requirement".to_string(),
            };
        }

        // Check max exposure
        let current_exposure = portfolio.total_market_value();
        let new_exposure = current_exposure + position_value;
        let exposure_pct = (new_exposure / portfolio.equity) * dec!(100);

        if exposure_pct > self.max_exposure_pct {
            let max_additional =
                (portfolio.equity * self.max_exposure_pct / dec!(100)) - current_exposure;
            if max_additional <= Decimal::ZERO {
                return LimitCheck::Blocked {
                    reason: format!(
                        "Max exposure reached: {:.2}% (limit: {:.2}%)",
                        (current_exposure / portfolio.equity) * dec!(100),
                        self.max_exposure_pct
                    ),
                };
            }
            return LimitCheck::Reduced {
                max_size: max_additional,
                reason: format!("Limited by max exposure ({:.2}%)", self.max_exposure_pct),
            };
        }

        // Check position size limit
        let position_pct = (position_value / portfolio.equity) * dec!(100);
        if position_pct > self.max_position_pct {
            let max_position = portfolio.equity * self.max_position_pct / dec!(100);
            return LimitCheck::Reduced {
                max_size: max_position,
                reason: format!(
                    "Limited by max position size ({:.2}%)",
                    self.max_position_pct
                ),
            };
        }

        // Check concentration
        if position_pct > self.max_concentration_pct {
            let max_position = portfolio.equity * self.max_concentration_pct / dec!(100);
            return LimitCheck::Reduced {
                max_size: max_position,
                reason: format!(
                    "Limited by max concentration ({:.2}%)",
                    self.max_concentration_pct
                ),
            };
        }

        LimitCheck::Allowed
    }

    /// Check if trading should be halted.
    pub fn should_halt_trading(&self, portfolio: &Portfolio, daily_pnl: Decimal) -> Option<String> {
        // Check daily loss limit
        let daily_loss_pct = if portfolio.initial_capital > Decimal::ZERO {
            (daily_pnl / portfolio.initial_capital) * dec!(100)
        } else {
            Decimal::ZERO
        };

        if daily_loss_pct <= -self.daily_loss_limit_pct {
            return Some(format!(
                "Daily loss limit reached: {:.2}%",
                daily_loss_pct.abs()
            ));
        }

        // Check max drawdown
        let drawdown = portfolio.drawdown();
        if drawdown >= self.max_drawdown_pct {
            return Some(format!("Max drawdown exceeded: {:.2}%", drawdown));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_portfolio(equity: Decimal, cash: Decimal, positions: usize) -> Portfolio {
        let mut portfolio = Portfolio::new(equity);
        portfolio.cash = cash;
        portfolio.equity = equity;
        // Simulate positions by adjusting market value
        for i in 0..positions {
            let mut pos =
                trading_core::types::Position::new(format!("SYM{}", i), dec!(10), dec!(100));
            pos.market_value = dec!(1000);
            portfolio.positions.insert(format!("SYM{}", i), pos);
        }
        portfolio
    }

    #[test]
    fn test_allowed_position() {
        let limits = PortfolioLimits::default();
        let portfolio = create_portfolio(dec!(100000), dec!(50000), 2);

        let check = limits.check_new_position(&portfolio, dec!(5000), Decimal::ZERO);
        assert!(check.is_allowed());
    }

    #[test]
    fn test_max_positions_blocked() {
        let limits = PortfolioLimits {
            max_positions: 3,
            ..Default::default()
        };
        let portfolio = create_portfolio(dec!(100000), dec!(50000), 3);

        let check = limits.check_new_position(&portfolio, dec!(5000), Decimal::ZERO);
        assert!(check.is_blocked());
    }

    #[test]
    fn test_daily_loss_limit() {
        let limits = PortfolioLimits::default();
        let portfolio = create_portfolio(dec!(100000), dec!(50000), 0);

        // Down 5% today (exceeds 3% limit)
        let check = limits.check_new_position(&portfolio, dec!(5000), dec!(-5000));
        assert!(check.is_blocked());
    }

    #[test]
    fn test_position_size_reduced() {
        let limits = PortfolioLimits {
            max_position_pct: dec!(5), // 5% max
            ..Default::default()
        };
        let portfolio = create_portfolio(dec!(100000), dec!(100000), 0);

        // Trying to buy 10% position
        let check = limits.check_new_position(&portfolio, dec!(10000), Decimal::ZERO);
        match check {
            LimitCheck::Reduced { max_size, .. } => {
                assert_eq!(max_size, dec!(5000)); // Reduced to 5%
            }
            _ => panic!("Expected Reduced"),
        }
    }
}
