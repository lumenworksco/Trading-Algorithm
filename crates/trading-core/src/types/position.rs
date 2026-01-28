//! Position and portfolio types.

use num_traits::Signed;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{Order, Side};

/// A position in a single security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Symbol
    pub symbol: String,
    /// Number of shares (positive for long, negative for short)
    pub quantity: Decimal,
    /// Average entry price
    pub avg_entry_price: Decimal,
    /// Current market price
    pub current_price: Decimal,
    /// Market value (quantity * current_price)
    pub market_value: Decimal,
    /// Cost basis (quantity * avg_entry_price)
    pub cost_basis: Decimal,
    /// Unrealized profit/loss
    pub unrealized_pnl: Decimal,
    /// Unrealized P&L as a percentage
    pub unrealized_pnl_percent: Decimal,
    /// Realized profit/loss from closed portions
    pub realized_pnl: Decimal,
}

impl Position {
    /// Create a new position.
    pub fn new(symbol: impl Into<String>, quantity: Decimal, avg_entry_price: Decimal) -> Self {
        let symbol = symbol.into();
        let current_price = avg_entry_price;
        let market_value = quantity * current_price;
        let cost_basis = quantity * avg_entry_price;

        Self {
            symbol,
            quantity,
            avg_entry_price,
            current_price,
            market_value,
            cost_basis,
            unrealized_pnl: Decimal::ZERO,
            unrealized_pnl_percent: Decimal::ZERO,
            realized_pnl: Decimal::ZERO,
        }
    }

    /// Check if this is a long position.
    pub fn is_long(&self) -> bool {
        self.quantity > Decimal::ZERO
    }

    /// Check if this is a short position.
    pub fn is_short(&self) -> bool {
        self.quantity < Decimal::ZERO
    }

    /// Check if the position is flat (no shares).
    pub fn is_flat(&self) -> bool {
        self.quantity == Decimal::ZERO
    }

    /// Get the absolute quantity.
    pub fn abs_quantity(&self) -> Decimal {
        self.quantity.abs()
    }

    /// Update the current market price and recalculate values.
    pub fn update_price(&mut self, price: Decimal) {
        self.current_price = price;
        self.market_value = self.quantity * price;
        self.unrealized_pnl = self.market_value - self.cost_basis;

        if self.cost_basis != Decimal::ZERO {
            self.unrealized_pnl_percent =
                (self.unrealized_pnl / self.cost_basis.abs()) * Decimal::from(100);
        }
    }

    /// Apply a fill to the position.
    /// Returns the realized P&L if the position is being reduced.
    pub fn apply_fill(&mut self, side: Side, quantity: Decimal, price: Decimal) -> Decimal {
        let fill_qty = match side {
            Side::Buy => quantity,
            Side::Sell => -quantity,
        };

        let mut realized = Decimal::ZERO;

        // Check if this is increasing or decreasing the position
        let same_direction = (self.quantity > Decimal::ZERO && fill_qty > Decimal::ZERO)
            || (self.quantity < Decimal::ZERO && fill_qty < Decimal::ZERO);

        if same_direction || self.quantity == Decimal::ZERO {
            // Adding to position - update average entry price
            let total_cost = self.quantity * self.avg_entry_price + fill_qty * price;
            let new_quantity = self.quantity + fill_qty;

            if new_quantity != Decimal::ZERO {
                self.avg_entry_price = total_cost / new_quantity;
            }
            self.quantity = new_quantity;
        } else {
            // Reducing or reversing position
            let close_qty = fill_qty.abs().min(self.quantity.abs());

            // Calculate realized P&L on the closed portion
            if self.quantity > Decimal::ZERO {
                // Was long, now selling
                realized = close_qty * (price - self.avg_entry_price);
            } else {
                // Was short, now buying
                realized = close_qty * (self.avg_entry_price - price);
            }
            self.realized_pnl += realized;

            // Update quantity
            let remaining = fill_qty.abs() - close_qty;
            if remaining > Decimal::ZERO {
                // Position reversed
                self.quantity = fill_qty.signum() * remaining;
                self.avg_entry_price = price;
            } else {
                self.quantity += fill_qty;
            }
        }

        // Update derived values
        self.cost_basis = self.quantity * self.avg_entry_price;
        self.update_price(self.current_price);

        realized
    }
}

/// Portfolio containing cash and positions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Portfolio {
    /// Available cash
    pub cash: Decimal,
    /// Buying power (may be different from cash due to margin)
    pub buying_power: Decimal,
    /// Total equity (cash + market value of positions)
    pub equity: Decimal,
    /// Map of symbol to position
    pub positions: HashMap<String, Position>,
    /// Total unrealized P&L across all positions
    pub total_unrealized_pnl: Decimal,
    /// Total realized P&L across all positions
    pub total_realized_pnl: Decimal,
    /// Initial capital (for calculating returns)
    pub initial_capital: Decimal,
    /// Highest equity reached (for drawdown calculation)
    pub peak_equity: Decimal,
}

impl Portfolio {
    /// Create a new portfolio with initial cash.
    pub fn new(initial_capital: Decimal) -> Self {
        Self {
            cash: initial_capital,
            buying_power: initial_capital,
            equity: initial_capital,
            positions: HashMap::new(),
            total_unrealized_pnl: Decimal::ZERO,
            total_realized_pnl: Decimal::ZERO,
            initial_capital,
            peak_equity: initial_capital,
        }
    }

    /// Get a position by symbol.
    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// Get a mutable position by symbol.
    pub fn get_position_mut(&mut self, symbol: &str) -> Option<&mut Position> {
        self.positions.get_mut(symbol)
    }

    /// Check if we have a position in a symbol.
    pub fn has_position(&self, symbol: &str) -> bool {
        self.positions
            .get(symbol)
            .map(|p| !p.is_flat())
            .unwrap_or(false)
    }

    /// Get the total market value of all positions.
    pub fn total_market_value(&self) -> Decimal {
        self.positions.values().map(|p| p.market_value).sum()
    }

    /// Update the equity and related calculations.
    pub fn update_equity(&mut self) {
        let market_value: Decimal = self.positions.values().map(|p| p.market_value).sum();
        self.equity = self.cash + market_value;

        self.total_unrealized_pnl = self.positions.values().map(|p| p.unrealized_pnl).sum();

        // Update peak equity for drawdown calculation
        if self.equity > self.peak_equity {
            self.peak_equity = self.equity;
        }
    }

    /// Apply a filled order to the portfolio.
    pub fn apply_order(&mut self, order: &Order) {
        if !order.is_filled() {
            return;
        }

        let fill_value = order.filled_avg_price.unwrap_or(Decimal::ZERO) * order.filled_quantity;

        // Update cash
        match order.side {
            Side::Buy => self.cash -= fill_value,
            Side::Sell => self.cash += fill_value,
        }

        // Update or create position
        let position = self
            .positions
            .entry(order.symbol.clone())
            .or_insert_with(|| Position::new(&order.symbol, Decimal::ZERO, Decimal::ZERO));

        let realized = position.apply_fill(
            order.side,
            order.filled_quantity,
            order.filled_avg_price.unwrap_or(Decimal::ZERO),
        );

        self.total_realized_pnl += realized;

        // Remove flat positions
        if position.is_flat() {
            self.positions.remove(&order.symbol);
        }

        self.update_equity();
    }

    /// Update all positions with current market prices.
    pub fn update_prices(&mut self, prices: &HashMap<String, Decimal>) {
        for (symbol, position) in self.positions.iter_mut() {
            if let Some(&price) = prices.get(symbol) {
                position.update_price(price);
            }
        }
        self.update_equity();
    }

    /// Calculate current drawdown from peak.
    pub fn drawdown(&self) -> Decimal {
        if self.peak_equity == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (self.peak_equity - self.equity) / self.peak_equity * Decimal::from(100)
    }

    /// Calculate total return percentage.
    pub fn total_return(&self) -> Decimal {
        if self.initial_capital == Decimal::ZERO {
            return Decimal::ZERO;
        }
        (self.equity - self.initial_capital) / self.initial_capital * Decimal::from(100)
    }

    /// Get the number of open positions.
    pub fn position_count(&self) -> usize {
        self.positions.len()
    }

    /// Get all symbols with open positions.
    pub fn symbols(&self) -> Vec<&String> {
        self.positions.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_position_long() {
        let mut position = Position::new("AAPL", dec!(100), dec!(150.00));
        assert!(position.is_long());
        assert_eq!(position.cost_basis, dec!(15000.00));

        position.update_price(dec!(160.00));
        assert_eq!(position.market_value, dec!(16000.00));
        assert_eq!(position.unrealized_pnl, dec!(1000.00));
    }

    #[test]
    fn test_position_apply_fill_increase() {
        let mut position = Position::new("AAPL", dec!(100), dec!(150.00));

        // Add more shares at a different price
        let realized = position.apply_fill(Side::Buy, dec!(100), dec!(160.00));
        assert_eq!(realized, Decimal::ZERO);
        assert_eq!(position.quantity, dec!(200));
        assert_eq!(position.avg_entry_price, dec!(155.00)); // Average of 150 and 160
    }

    #[test]
    fn test_position_apply_fill_close() {
        let mut position = Position::new("AAPL", dec!(100), dec!(150.00));
        position.update_price(dec!(160.00));

        // Sell all shares at 160
        let realized = position.apply_fill(Side::Sell, dec!(100), dec!(160.00));
        assert_eq!(realized, dec!(1000.00)); // 100 shares * $10 profit
        assert!(position.is_flat());
    }

    #[test]
    fn test_portfolio_creation() {
        let portfolio = Portfolio::new(dec!(100000));
        assert_eq!(portfolio.cash, dec!(100000));
        assert_eq!(portfolio.equity, dec!(100000));
        assert_eq!(portfolio.position_count(), 0);
    }

    #[test]
    fn test_portfolio_drawdown() {
        let mut portfolio = Portfolio::new(dec!(100000));
        portfolio.peak_equity = dec!(110000);
        portfolio.equity = dec!(99000);

        let dd = portfolio.drawdown();
        assert!((dd - dec!(10)).abs() < dec!(0.01)); // 10% drawdown
    }

    #[test]
    fn test_portfolio_total_return() {
        let mut portfolio = Portfolio::new(dec!(100000));
        portfolio.equity = dec!(120000);

        let ret = portfolio.total_return();
        assert_eq!(ret, dec!(20)); // 20% return
    }
}
