//! Broker trait definition.

use crate::error::BrokerError;
use crate::types::{Order, OrderRequest, Portfolio, Position};
use async_trait::async_trait;

/// Trait for broker integrations.
///
/// Brokers handle order execution, position management, and account information.
#[async_trait]
pub trait Broker: Send + Sync {
    /// Get account/portfolio information.
    async fn get_account(&self) -> Result<Portfolio, BrokerError>;

    /// Submit a new order.
    ///
    /// # Arguments
    /// * `request` - The order request to submit
    ///
    /// # Returns
    /// The created order with an ID and initial status
    async fn submit_order(&self, request: OrderRequest) -> Result<Order, BrokerError>;

    /// Cancel an existing order.
    ///
    /// # Arguments
    /// * `order_id` - The ID of the order to cancel
    async fn cancel_order(&self, order_id: &str) -> Result<(), BrokerError>;

    /// Get the status of an order.
    ///
    /// # Arguments
    /// * `order_id` - The ID of the order to check
    async fn get_order(&self, order_id: &str) -> Result<Order, BrokerError>;

    /// Get all open orders.
    async fn get_open_orders(&self) -> Result<Vec<Order>, BrokerError>;

    /// Get all positions.
    async fn get_positions(&self) -> Result<Vec<Position>, BrokerError>;

    /// Get position for a specific symbol.
    ///
    /// # Arguments
    /// * `symbol` - The symbol to look up
    ///
    /// # Returns
    /// The position if one exists, None otherwise
    async fn get_position(&self, symbol: &str) -> Result<Option<Position>, BrokerError>;

    /// Close a position.
    ///
    /// This will submit a market order to close the entire position.
    ///
    /// # Arguments
    /// * `symbol` - The symbol to close
    async fn close_position(&self, symbol: &str) -> Result<Order, BrokerError>;

    /// Close all positions.
    async fn close_all_positions(&self) -> Result<Vec<Order>, BrokerError>;

    /// Cancel all open orders.
    async fn cancel_all_orders(&self) -> Result<(), BrokerError>;

    /// Check if the market is currently open.
    async fn is_market_open(&self) -> Result<bool, BrokerError>;

    /// Get the current buying power.
    async fn get_buying_power(&self) -> Result<rust_decimal::Decimal, BrokerError> {
        let account = self.get_account().await?;
        Ok(account.buying_power)
    }

    /// Get the broker name.
    fn name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    // Broker tests would typically use mock implementations
}
