//! Stop-loss management.

use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use trading_core::types::{Position, Side};

/// Stop-loss calculation method.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopLossMethod {
    /// Fixed percentage below/above entry
    FixedPercent { percent: Decimal },
    /// ATR-based stop
    Atr { multiplier: Decimal },
    /// Fixed dollar amount
    FixedDollar { amount: Decimal },
    /// Trailing stop (percentage)
    TrailingPercent { percent: Decimal },
    /// Trailing stop (ATR-based)
    TrailingAtr { multiplier: Decimal },
}

impl Default for StopLossMethod {
    fn default() -> Self {
        StopLossMethod::FixedPercent { percent: dec!(2) }
    }
}

/// A stop-loss order to be placed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopLossOrder {
    /// Symbol
    pub symbol: String,
    /// Stop price
    pub stop_price: Decimal,
    /// Quantity
    pub quantity: Decimal,
    /// Is this a trailing stop
    pub is_trailing: bool,
    /// Trail amount (for trailing stops)
    pub trail_amount: Option<Decimal>,
}

/// Stop-loss manager.
#[derive(Debug, Clone)]
pub struct StopLossManager {
    method: StopLossMethod,
    current_atr: Option<Decimal>,
}

impl StopLossManager {
    /// Create a new stop-loss manager.
    pub fn new(method: StopLossMethod) -> Self {
        Self {
            method,
            current_atr: None,
        }
    }

    /// Update the current ATR value (needed for ATR-based stops).
    pub fn update_atr(&mut self, atr: Decimal) {
        self.current_atr = Some(atr);
    }

    /// Calculate stop-loss price for a new position.
    pub fn calculate_stop_price(&self, entry_price: Decimal, side: Side) -> Option<Decimal> {
        match &self.method {
            StopLossMethod::FixedPercent { percent } => {
                let offset = entry_price * (*percent / dec!(100));
                match side {
                    Side::Buy => Some(entry_price - offset),  // Long: stop below
                    Side::Sell => Some(entry_price + offset), // Short: stop above
                }
            }

            StopLossMethod::Atr { multiplier } => self.current_atr.map(|atr| {
                let offset = atr * *multiplier;
                match side {
                    Side::Buy => entry_price - offset,
                    Side::Sell => entry_price + offset,
                }
            }),

            StopLossMethod::FixedDollar { amount } => match side {
                Side::Buy => Some(entry_price - *amount),
                Side::Sell => Some(entry_price + *amount),
            },

            StopLossMethod::TrailingPercent { percent } => {
                // Initial stop same as fixed percent
                let offset = entry_price * (*percent / dec!(100));
                match side {
                    Side::Buy => Some(entry_price - offset),
                    Side::Sell => Some(entry_price + offset),
                }
            }

            StopLossMethod::TrailingAtr { multiplier } => self.current_atr.map(|atr| {
                let offset = atr * *multiplier;
                match side {
                    Side::Buy => entry_price - offset,
                    Side::Sell => entry_price + offset,
                }
            }),
        }
    }

    /// Update trailing stop based on current price.
    pub fn update_trailing_stop(
        &self,
        current_stop: Decimal,
        current_price: Decimal,
        side: Side,
    ) -> Decimal {
        match &self.method {
            StopLossMethod::TrailingPercent { percent } => {
                let offset = current_price * (*percent / dec!(100));
                match side {
                    Side::Buy => {
                        // Long: move stop up if price moved up
                        let new_stop = current_price - offset;
                        new_stop.max(current_stop)
                    }
                    Side::Sell => {
                        // Short: move stop down if price moved down
                        let new_stop = current_price + offset;
                        new_stop.min(current_stop)
                    }
                }
            }

            StopLossMethod::TrailingAtr { multiplier } => {
                if let Some(atr) = self.current_atr {
                    let offset = atr * *multiplier;
                    match side {
                        Side::Buy => {
                            let new_stop = current_price - offset;
                            new_stop.max(current_stop)
                        }
                        Side::Sell => {
                            let new_stop = current_price + offset;
                            new_stop.min(current_stop)
                        }
                    }
                } else {
                    current_stop
                }
            }

            _ => current_stop, // Non-trailing stops don't update
        }
    }

    /// Check if stop-loss is triggered.
    pub fn is_triggered(&self, stop_price: Decimal, current_price: Decimal, side: Side) -> bool {
        match side {
            Side::Buy => current_price <= stop_price, // Long: triggered if price falls to stop
            Side::Sell => current_price >= stop_price, // Short: triggered if price rises to stop
        }
    }

    /// Create a stop-loss order for a position.
    pub fn create_stop_order(&self, position: &Position) -> Option<StopLossOrder> {
        let side = if position.is_long() {
            Side::Buy
        } else {
            Side::Sell
        };
        let stop_price = self.calculate_stop_price(position.avg_entry_price, side)?;

        let is_trailing = matches!(
            self.method,
            StopLossMethod::TrailingPercent { .. } | StopLossMethod::TrailingAtr { .. }
        );

        let trail_amount = match &self.method {
            StopLossMethod::TrailingPercent { percent } => {
                Some(position.avg_entry_price * (*percent / dec!(100)))
            }
            StopLossMethod::TrailingAtr { multiplier } => {
                self.current_atr.map(|atr| atr * *multiplier)
            }
            _ => None,
        };

        Some(StopLossOrder {
            symbol: position.symbol.clone(),
            stop_price,
            quantity: position.quantity.abs(),
            is_trailing,
            trail_amount,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_percent_stop() {
        let manager = StopLossManager::new(StopLossMethod::FixedPercent { percent: dec!(5) });

        // Long position
        let stop = manager.calculate_stop_price(dec!(100), Side::Buy).unwrap();
        assert_eq!(stop, dec!(95)); // 5% below

        // Short position
        let stop = manager.calculate_stop_price(dec!(100), Side::Sell).unwrap();
        assert_eq!(stop, dec!(105)); // 5% above
    }

    #[test]
    fn test_atr_stop() {
        let mut manager = StopLossManager::new(StopLossMethod::Atr {
            multiplier: dec!(2),
        });
        manager.update_atr(dec!(5)); // ATR = 5

        let stop = manager.calculate_stop_price(dec!(100), Side::Buy).unwrap();
        assert_eq!(stop, dec!(90)); // 2 * 5 = 10 below
    }

    #[test]
    fn test_trailing_stop_update() {
        let manager = StopLossManager::new(StopLossMethod::TrailingPercent { percent: dec!(5) });

        // Long position, price moved up
        let current_stop = dec!(95);
        let new_stop = manager.update_trailing_stop(current_stop, dec!(110), Side::Buy);
        assert_eq!(new_stop, dec!(104.5)); // 5% below 110

        // Price moved down - stop shouldn't move down
        let new_stop2 = manager.update_trailing_stop(new_stop, dec!(105), Side::Buy);
        assert_eq!(new_stop2, dec!(104.5)); // Stays at higher level
    }

    #[test]
    fn test_stop_triggered() {
        let manager = StopLossManager::new(StopLossMethod::FixedPercent { percent: dec!(5) });

        // Long position
        assert!(manager.is_triggered(dec!(95), dec!(94), Side::Buy));
        assert!(manager.is_triggered(dec!(95), dec!(95), Side::Buy));
        assert!(!manager.is_triggered(dec!(95), dec!(96), Side::Buy));

        // Short position
        assert!(manager.is_triggered(dec!(105), dec!(106), Side::Sell));
        assert!(!manager.is_triggered(dec!(105), dec!(104), Side::Sell));
    }
}
