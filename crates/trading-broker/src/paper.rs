//! Paper trading broker for backtesting and simulation.

use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use trading_core::error::BrokerError;
use trading_core::traits::Broker;
use trading_core::types::{
    Fill, Order, OrderRequest, OrderStatus, OrderType, Portfolio, Position, Side,
};
use uuid::Uuid;

/// Paper trading broker for simulation.
pub struct PaperBroker {
    portfolio: Arc<Mutex<Portfolio>>,
    orders: Arc<Mutex<HashMap<Uuid, Order>>>,
    slippage_pct: Decimal,
    commission_per_share: Decimal,
}

impl PaperBroker {
    /// Create a new paper broker with initial capital.
    pub fn new(initial_capital: Decimal) -> Self {
        Self {
            portfolio: Arc::new(Mutex::new(Portfolio::new(initial_capital))),
            orders: Arc::new(Mutex::new(HashMap::new())),
            slippage_pct: dec!(0.05), // 0.05% slippage
            commission_per_share: Decimal::ZERO,
        }
    }

    /// Set slippage percentage.
    pub fn with_slippage(mut self, slippage_pct: Decimal) -> Self {
        self.slippage_pct = slippage_pct;
        self
    }

    /// Set commission per share.
    pub fn with_commission(mut self, commission: Decimal) -> Self {
        self.commission_per_share = commission;
        self
    }

    /// Simulate order execution at a given price.
    pub fn execute_at_price(&self, order_id: Uuid, market_price: Decimal) -> Result<Order, BrokerError> {
        let mut orders = self.orders.lock().unwrap();
        let order = orders.get_mut(&order_id)
            .ok_or_else(|| BrokerError::OrderNotFound(order_id.to_string()))?;

        if order.status.is_terminal() {
            return Ok(order.clone());
        }

        // Apply slippage
        let fill_price = match order.side {
            Side::Buy => market_price * (dec!(1) + self.slippage_pct / dec!(100)),
            Side::Sell => market_price * (dec!(1) - self.slippage_pct / dec!(100)),
        };

        // Check if limit order can be filled
        if order.order_type == OrderType::Limit {
            if let Some(limit) = order.limit_price {
                match order.side {
                    Side::Buy if fill_price > limit => {
                        return Ok(order.clone()); // Can't fill above limit
                    }
                    Side::Sell if fill_price < limit => {
                        return Ok(order.clone()); // Can't fill below limit
                    }
                    _ => {}
                }
            }
        }

        // Check buying power for buys
        if order.side == Side::Buy {
            let portfolio = self.portfolio.lock().unwrap();
            let cost = fill_price * order.quantity;
            if cost > portfolio.cash {
                return Err(BrokerError::InsufficientFunds {
                    required: cost,
                    available: portfolio.cash,
                });
            }
            drop(portfolio);
        }

        // Calculate commission
        let commission = self.commission_per_share * order.quantity;

        // Create fill
        let fill = Fill {
            id: Uuid::new_v4().to_string(),
            order_id,
            quantity: order.quantity,
            price: fill_price,
            commission,
            timestamp: Utc::now(),
        };

        order.add_fill(fill);
        order.status = OrderStatus::Filled;

        // Update portfolio
        let mut portfolio = self.portfolio.lock().unwrap();

        // Update cash
        let fill_value = fill_price * order.quantity;
        match order.side {
            Side::Buy => {
                portfolio.cash -= fill_value + commission;
            }
            Side::Sell => {
                portfolio.cash += fill_value - commission;
            }
        }

        // Update position
        let position = portfolio.positions.entry(order.symbol.clone())
            .or_insert_with(|| Position::new(&order.symbol, Decimal::ZERO, Decimal::ZERO));

        position.apply_fill(order.side, order.quantity, fill_price);

        if position.is_flat() {
            portfolio.positions.remove(&order.symbol);
        }

        portfolio.update_equity();
        portfolio.buying_power = portfolio.cash; // Simplified

        Ok(order.clone())
    }

    /// Update all position prices.
    pub fn update_prices(&self, prices: &HashMap<String, Decimal>) {
        let mut portfolio = self.portfolio.lock().unwrap();
        portfolio.update_prices(prices);
    }

    /// Get a snapshot of the portfolio.
    pub fn portfolio_snapshot(&self) -> Portfolio {
        self.portfolio.lock().unwrap().clone()
    }
}

#[async_trait]
impl Broker for PaperBroker {
    async fn get_account(&self) -> Result<Portfolio, BrokerError> {
        Ok(self.portfolio.lock().unwrap().clone())
    }

    async fn submit_order(&self, request: OrderRequest) -> Result<Order, BrokerError> {
        // Note: buying power check for market orders happens in execute_at_price
        // since we don't know the fill price at submission time.

        let order = Order::from_request(&request);
        let order_id = order.id;

        let mut orders = self.orders.lock().unwrap();
        orders.insert(order_id, order.clone());

        Ok(order)
    }

    async fn cancel_order(&self, order_id: &str) -> Result<(), BrokerError> {
        let uuid = Uuid::parse_str(order_id)
            .map_err(|_| BrokerError::OrderNotFound(order_id.to_string()))?;

        let mut orders = self.orders.lock().unwrap();
        let order = orders.get_mut(&uuid)
            .ok_or_else(|| BrokerError::OrderNotFound(order_id.to_string()))?;

        if order.status.is_terminal() {
            return Err(BrokerError::OrderRejected("Order already terminal".to_string()));
        }

        order.status = OrderStatus::Canceled;
        order.canceled_at = Some(Utc::now());

        Ok(())
    }

    async fn get_order(&self, order_id: &str) -> Result<Order, BrokerError> {
        let uuid = Uuid::parse_str(order_id)
            .map_err(|_| BrokerError::OrderNotFound(order_id.to_string()))?;

        let orders = self.orders.lock().unwrap();
        orders.get(&uuid)
            .cloned()
            .ok_or_else(|| BrokerError::OrderNotFound(order_id.to_string()))
    }

    async fn get_open_orders(&self) -> Result<Vec<Order>, BrokerError> {
        let orders = self.orders.lock().unwrap();
        Ok(orders.values()
            .filter(|o| o.status.is_active())
            .cloned()
            .collect())
    }

    async fn get_positions(&self) -> Result<Vec<Position>, BrokerError> {
        let portfolio = self.portfolio.lock().unwrap();
        Ok(portfolio.positions.values().cloned().collect())
    }

    async fn get_position(&self, symbol: &str) -> Result<Option<Position>, BrokerError> {
        let portfolio = self.portfolio.lock().unwrap();
        Ok(portfolio.positions.get(symbol).cloned())
    }

    async fn close_position(&self, symbol: &str) -> Result<Order, BrokerError> {
        let (side, quantity) = {
            let portfolio = self.portfolio.lock().unwrap();
            let position = portfolio.positions.get(symbol)
                .ok_or_else(|| BrokerError::PositionNotFound(symbol.to_string()))?;
            let side = if position.is_long() { Side::Sell } else { Side::Buy };
            let quantity = position.quantity.abs();
            (side, quantity)
        }; // MutexGuard dropped here before await

        let request = OrderRequest::market(symbol, side, quantity);
        self.submit_order(request).await
    }

    async fn close_all_positions(&self) -> Result<Vec<Order>, BrokerError> {
        let symbols: Vec<String> = {
            let portfolio = self.portfolio.lock().unwrap();
            portfolio.positions.keys().cloned().collect()
        };

        let mut orders = Vec::new();
        for symbol in symbols {
            orders.push(self.close_position(&symbol).await?);
        }

        Ok(orders)
    }

    async fn cancel_all_orders(&self) -> Result<(), BrokerError> {
        let order_ids: Vec<String> = {
            let orders = self.orders.lock().unwrap();
            orders.values()
                .filter(|o| o.status.is_active())
                .map(|o| o.id.to_string())
                .collect()
        };

        for order_id in order_ids {
            self.cancel_order(&order_id).await?;
        }

        Ok(())
    }

    async fn is_market_open(&self) -> Result<bool, BrokerError> {
        Ok(true) // Paper trading is always open
    }

    fn name(&self) -> &str {
        "Paper Broker"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_paper_broker_buy() {
        let broker = PaperBroker::new(dec!(100000));

        let request = OrderRequest::market("AAPL", Side::Buy, dec!(100));
        let order = broker.submit_order(request).await.unwrap();

        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.symbol, "AAPL");

        // Execute at price
        let filled = broker.execute_at_price(order.id, dec!(150)).unwrap();
        assert_eq!(filled.status, OrderStatus::Filled);

        // Check portfolio
        let portfolio = broker.get_account().await.unwrap();
        assert!(portfolio.positions.contains_key("AAPL"));
    }

    #[tokio::test]
    async fn test_paper_broker_close_position() {
        let broker = PaperBroker::new(dec!(100000));

        // Buy
        let buy = OrderRequest::market("AAPL", Side::Buy, dec!(100));
        let order = broker.submit_order(buy).await.unwrap();
        broker.execute_at_price(order.id, dec!(150)).unwrap();

        // Close
        let close_order = broker.close_position("AAPL").await.unwrap();
        broker.execute_at_price(close_order.id, dec!(155)).unwrap();

        // Check position closed
        let pos = broker.get_position("AAPL").await.unwrap();
        assert!(pos.is_none());
    }
}
