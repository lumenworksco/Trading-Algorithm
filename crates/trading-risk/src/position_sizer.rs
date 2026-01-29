//! Position sizing algorithms.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use trading_core::types::{Portfolio, Signal, SignalStrength};

/// Position sizing method.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionSizingMethod {
    /// Fixed number of shares
    Fixed { shares: Decimal },
    /// Fixed dollar amount
    FixedDollar { amount: Decimal },
    /// Percentage of equity
    PercentEquity { percent: Decimal },
    /// Risk-based (percentage of equity at risk per trade)
    RiskBased { risk_percent: Decimal },
    /// Kelly criterion
    Kelly {
        win_rate: Decimal,
        avg_win_loss_ratio: Decimal,
    },
}

impl Default for PositionSizingMethod {
    fn default() -> Self {
        PositionSizingMethod::PercentEquity { percent: dec!(2) }
    }
}

/// Position sizer calculates the appropriate position size.
#[derive(Debug, Clone)]
pub struct PositionSizer {
    method: PositionSizingMethod,
    max_shares: Option<Decimal>,
    max_position_value: Option<Decimal>,
    use_signal_strength: bool,
}

impl PositionSizer {
    /// Create a new position sizer.
    pub fn new(method: PositionSizingMethod) -> Self {
        Self {
            method,
            max_shares: None,
            max_position_value: None,
            use_signal_strength: true,
        }
    }

    /// Set maximum shares per position.
    pub fn with_max_shares(mut self, max: Decimal) -> Self {
        self.max_shares = Some(max);
        self
    }

    /// Set maximum position value.
    pub fn with_max_position_value(mut self, max: Decimal) -> Self {
        self.max_position_value = Some(max);
        self
    }

    /// Disable signal strength adjustment.
    pub fn without_signal_strength(mut self) -> Self {
        self.use_signal_strength = false;
        self
    }

    /// Calculate position size.
    pub fn calculate(
        &self,
        portfolio: &Portfolio,
        signal: &Signal,
        current_price: Decimal,
        stop_loss_price: Option<Decimal>,
    ) -> Decimal {
        if current_price <= Decimal::ZERO {
            return Decimal::ZERO;
        }

        let base_size = match &self.method {
            PositionSizingMethod::Fixed { shares } => *shares,

            PositionSizingMethod::FixedDollar { amount } => *amount / current_price,

            PositionSizingMethod::PercentEquity { percent } => {
                let position_value = portfolio.equity * (*percent / dec!(100));
                position_value / current_price
            }

            PositionSizingMethod::RiskBased { risk_percent } => {
                if let Some(stop_price) = stop_loss_price {
                    let risk_per_share = (current_price - stop_price).abs();
                    if risk_per_share > Decimal::ZERO {
                        let risk_amount = portfolio.equity * (*risk_percent / dec!(100));
                        risk_amount / risk_per_share
                    } else {
                        Decimal::ZERO
                    }
                } else {
                    // Fallback to percent equity if no stop loss
                    let position_value = portfolio.equity * (*risk_percent / dec!(100));
                    position_value / current_price
                }
            }

            PositionSizingMethod::Kelly {
                win_rate,
                avg_win_loss_ratio,
            } => {
                // Kelly fraction = W - (1-W)/R
                // where W = win rate, R = avg win/loss ratio
                let kelly_fraction = *win_rate - (dec!(1) - *win_rate) / *avg_win_loss_ratio;
                let kelly_fraction = kelly_fraction.max(Decimal::ZERO).min(dec!(0.25)); // Cap at 25%

                let position_value = portfolio.equity * kelly_fraction;
                position_value / current_price
            }
        };

        // Apply signal strength multiplier
        let adjusted_size = if self.use_signal_strength {
            let multiplier = match signal.strength {
                SignalStrength::Weak => dec!(0.5),
                SignalStrength::Moderate => dec!(1.0),
                SignalStrength::Strong => dec!(1.5),
            };
            base_size * multiplier
        } else {
            base_size
        };

        // Apply limits
        let mut final_size = adjusted_size;

        if let Some(max) = self.max_shares {
            final_size = final_size.min(max);
        }

        if let Some(max_value) = self.max_position_value {
            let max_shares = max_value / current_price;
            final_size = final_size.min(max_shares);
        }

        // Check buying power
        let max_affordable = portfolio.buying_power / current_price;
        final_size = final_size.min(max_affordable);

        // Round down to whole shares
        final_size.floor()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trading_core::types::{SignalMetadata, SignalType};

    fn create_portfolio(equity: Decimal, buying_power: Decimal) -> Portfolio {
        let mut portfolio = Portfolio::new(equity);
        portfolio.buying_power = buying_power;
        portfolio
    }

    fn create_signal() -> Signal {
        Signal {
            symbol: "TEST".to_string(),
            signal_type: SignalType::Buy,
            strength: SignalStrength::Moderate,
            price: 100.0,
            timestamp: 0,
            confidence: 1.0,
            metadata: SignalMetadata::default(),
        }
    }

    #[test]
    fn test_fixed_shares() {
        let sizer = PositionSizer::new(PositionSizingMethod::Fixed { shares: dec!(100) })
            .without_signal_strength();
        let portfolio = create_portfolio(dec!(100000), dec!(100000));
        let signal = create_signal();

        let size = sizer.calculate(&portfolio, &signal, dec!(50), None);
        assert_eq!(size, dec!(100));
    }

    #[test]
    fn test_percent_equity() {
        let sizer = PositionSizer::new(PositionSizingMethod::PercentEquity { percent: dec!(5) })
            .without_signal_strength();
        let portfolio = create_portfolio(dec!(100000), dec!(100000));
        let signal = create_signal();

        let size = sizer.calculate(&portfolio, &signal, dec!(100), None);
        // 5% of 100000 = 5000, at $100/share = 50 shares
        assert_eq!(size, dec!(50));
    }

    #[test]
    fn test_risk_based() {
        let sizer = PositionSizer::new(PositionSizingMethod::RiskBased {
            risk_percent: dec!(1),
        })
        .without_signal_strength();
        let portfolio = create_portfolio(dec!(100000), dec!(100000));
        let signal = create_signal();

        // Risk 1% = $1000, stop loss $5 away = 200 shares
        let size = sizer.calculate(&portfolio, &signal, dec!(100), Some(dec!(95)));
        assert_eq!(size, dec!(200));
    }

    #[test]
    fn test_signal_strength_adjustment() {
        let sizer = PositionSizer::new(PositionSizingMethod::Fixed { shares: dec!(100) });
        let portfolio = create_portfolio(dec!(100000), dec!(100000));

        let mut weak_signal = create_signal();
        weak_signal.strength = SignalStrength::Weak;
        let weak_size = sizer.calculate(&portfolio, &weak_signal, dec!(50), None);

        let mut strong_signal = create_signal();
        strong_signal.strength = SignalStrength::Strong;
        let strong_size = sizer.calculate(&portfolio, &strong_signal, dec!(50), None);

        assert_eq!(weak_size, dec!(50)); // 100 * 0.5
        assert_eq!(strong_size, dec!(150)); // 100 * 1.5
    }

    #[test]
    fn test_max_shares_limit() {
        let sizer = PositionSizer::new(PositionSizingMethod::Fixed { shares: dec!(1000) })
            .with_max_shares(dec!(100))
            .without_signal_strength();
        let portfolio = create_portfolio(dec!(1000000), dec!(1000000));
        let signal = create_signal();

        let size = sizer.calculate(&portfolio, &signal, dec!(50), None);
        assert_eq!(size, dec!(100));
    }

    #[test]
    fn test_buying_power_limit() {
        let sizer = PositionSizer::new(PositionSizingMethod::Fixed { shares: dec!(1000) })
            .without_signal_strength();
        let portfolio = create_portfolio(dec!(100000), dec!(5000)); // Only $5000 buying power
        let signal = create_signal();

        let size = sizer.calculate(&portfolio, &signal, dec!(100), None);
        assert_eq!(size, dec!(50)); // Can only afford 50 shares
    }
}
