//! Order types and structures.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Order side (buy or sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    /// Get the opposite side.
    pub fn opposite(&self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }

    /// Get the sign for position calculations (+1 for buy, -1 for sell).
    pub fn sign(&self) -> Decimal {
        match self {
            Side::Buy => Decimal::ONE,
            Side::Sell => -Decimal::ONE,
        }
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::Buy => write!(f, "BUY"),
            Side::Sell => write!(f, "SELL"),
        }
    }
}

/// Order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    /// Market order - execute immediately at best available price
    Market,
    /// Limit order - execute at specified price or better
    Limit,
    /// Stop order - becomes market order when stop price is reached
    Stop,
    /// Stop-limit order - becomes limit order when stop price is reached
    StopLimit,
    /// Trailing stop order - stop price trails the market
    TrailingStop,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Market => write!(f, "MARKET"),
            OrderType::Limit => write!(f, "LIMIT"),
            OrderType::Stop => write!(f, "STOP"),
            OrderType::StopLimit => write!(f, "STOP_LIMIT"),
            OrderType::TrailingStop => write!(f, "TRAILING_STOP"),
        }
    }
}

/// Time in force for orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TimeInForce {
    /// Valid for the trading day only
    #[default]
    Day,
    /// Good til canceled
    #[serde(rename = "gtc")]
    GTC,
    /// Immediate or cancel
    #[serde(rename = "ioc")]
    IOC,
    /// Fill or kill
    #[serde(rename = "fok")]
    FOK,
    /// At market open
    #[serde(rename = "opg")]
    OPG,
    /// At market close
    #[serde(rename = "cls")]
    CLS,
}

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Order created but not yet submitted
    Pending,
    /// Order submitted to broker
    Submitted,
    /// Order accepted by broker/exchange
    Accepted,
    /// Order partially filled
    PartiallyFilled,
    /// Order completely filled
    Filled,
    /// Order canceled
    Canceled,
    /// Order rejected
    Rejected,
    /// Order expired
    Expired,
}

impl OrderStatus {
    /// Check if the order is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OrderStatus::Filled
                | OrderStatus::Canceled
                | OrderStatus::Rejected
                | OrderStatus::Expired
        )
    }

    /// Check if the order is active (can still be filled).
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            OrderStatus::Pending
                | OrderStatus::Submitted
                | OrderStatus::Accepted
                | OrderStatus::PartiallyFilled
        )
    }
}

/// Order request for submitting new orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    /// Symbol to trade
    pub symbol: String,
    /// Buy or sell
    pub side: Side,
    /// Type of order
    pub order_type: OrderType,
    /// Quantity to trade
    pub quantity: Decimal,
    /// Limit price (for limit and stop-limit orders)
    pub limit_price: Option<Decimal>,
    /// Stop price (for stop and stop-limit orders)
    pub stop_price: Option<Decimal>,
    /// Trailing amount (for trailing stop orders)
    pub trail_amount: Option<Decimal>,
    /// Time in force
    pub time_in_force: TimeInForce,
    /// Client-provided order ID
    pub client_order_id: Option<String>,
    /// Extended hours trading
    pub extended_hours: bool,
}

impl OrderRequest {
    /// Create a market order request.
    pub fn market(symbol: impl Into<String>, side: Side, quantity: Decimal) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type: OrderType::Market,
            quantity,
            limit_price: None,
            stop_price: None,
            trail_amount: None,
            time_in_force: TimeInForce::Day,
            client_order_id: None,
            extended_hours: false,
        }
    }

    /// Create a limit order request.
    pub fn limit(
        symbol: impl Into<String>,
        side: Side,
        quantity: Decimal,
        limit_price: Decimal,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type: OrderType::Limit,
            quantity,
            limit_price: Some(limit_price),
            stop_price: None,
            trail_amount: None,
            time_in_force: TimeInForce::Day,
            client_order_id: None,
            extended_hours: false,
        }
    }

    /// Create a stop order request.
    pub fn stop(
        symbol: impl Into<String>,
        side: Side,
        quantity: Decimal,
        stop_price: Decimal,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type: OrderType::Stop,
            quantity,
            limit_price: None,
            stop_price: Some(stop_price),
            trail_amount: None,
            time_in_force: TimeInForce::GTC,
            client_order_id: None,
            extended_hours: false,
        }
    }

    /// Create a stop-limit order request.
    pub fn stop_limit(
        symbol: impl Into<String>,
        side: Side,
        quantity: Decimal,
        stop_price: Decimal,
        limit_price: Decimal,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            order_type: OrderType::StopLimit,
            quantity,
            limit_price: Some(limit_price),
            stop_price: Some(stop_price),
            trail_amount: None,
            time_in_force: TimeInForce::GTC,
            client_order_id: None,
            extended_hours: false,
        }
    }

    /// Set the time in force.
    pub fn with_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = tif;
        self
    }

    /// Set a client order ID.
    pub fn with_client_order_id(mut self, id: impl Into<String>) -> Self {
        self.client_order_id = Some(id.into());
        self
    }

    /// Enable extended hours trading.
    pub fn with_extended_hours(mut self) -> Self {
        self.extended_hours = true;
        self
    }
}

/// A fill represents a partial or complete execution of an order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    /// Fill ID
    pub id: String,
    /// Order ID this fill belongs to
    pub order_id: Uuid,
    /// Quantity filled
    pub quantity: Decimal,
    /// Price at which the fill occurred
    pub price: Decimal,
    /// Commission charged
    pub commission: Decimal,
    /// Timestamp of the fill
    pub timestamp: DateTime<Utc>,
}

/// Complete order with status and fill information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Unique order ID
    pub id: Uuid,
    /// Client-provided order ID
    pub client_order_id: String,
    /// Symbol traded
    pub symbol: String,
    /// Buy or sell
    pub side: Side,
    /// Type of order
    pub order_type: OrderType,
    /// Original quantity
    pub quantity: Decimal,
    /// Limit price
    pub limit_price: Option<Decimal>,
    /// Stop price
    pub stop_price: Option<Decimal>,
    /// Trail amount
    pub trail_amount: Option<Decimal>,
    /// Time in force
    pub time_in_force: TimeInForce,
    /// Current status
    pub status: OrderStatus,
    /// Quantity filled so far
    pub filled_quantity: Decimal,
    /// Average fill price
    pub filled_avg_price: Option<Decimal>,
    /// List of fills
    pub fills: Vec<Fill>,
    /// When the order was created
    pub created_at: DateTime<Utc>,
    /// When the order was last updated
    pub updated_at: DateTime<Utc>,
    /// When the order was submitted
    pub submitted_at: Option<DateTime<Utc>>,
    /// When the order was filled
    pub filled_at: Option<DateTime<Utc>>,
    /// When the order expired
    pub expired_at: Option<DateTime<Utc>>,
    /// When the order was canceled
    pub canceled_at: Option<DateTime<Utc>>,
    /// Extended hours flag
    pub extended_hours: bool,
}

impl Order {
    /// Create a new order from a request.
    pub fn from_request(request: &OrderRequest) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            client_order_id: request
                .client_order_id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            symbol: request.symbol.clone(),
            side: request.side,
            order_type: request.order_type,
            quantity: request.quantity,
            limit_price: request.limit_price,
            stop_price: request.stop_price,
            trail_amount: request.trail_amount,
            time_in_force: request.time_in_force,
            status: OrderStatus::Pending,
            filled_quantity: Decimal::ZERO,
            filled_avg_price: None,
            fills: Vec::new(),
            created_at: now,
            updated_at: now,
            submitted_at: None,
            filled_at: None,
            expired_at: None,
            canceled_at: None,
            extended_hours: request.extended_hours,
        }
    }

    /// Get the remaining quantity to be filled.
    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    /// Check if the order is completely filled.
    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled
    }

    /// Check if the order can be canceled.
    pub fn is_cancelable(&self) -> bool {
        self.status.is_active()
    }

    /// Calculate the total value of the order.
    pub fn value(&self) -> Option<Decimal> {
        self.filled_avg_price
            .map(|price| price * self.filled_quantity)
    }

    /// Add a fill to the order.
    pub fn add_fill(&mut self, fill: Fill) {
        let total_qty = self.filled_quantity + fill.quantity;
        let total_value = self.filled_avg_price.unwrap_or(Decimal::ZERO) * self.filled_quantity
            + fill.price * fill.quantity;

        self.filled_avg_price = Some(total_value / total_qty);
        self.filled_quantity = total_qty;
        self.fills.push(fill);
        self.updated_at = Utc::now();

        if self.filled_quantity >= self.quantity {
            self.status = OrderStatus::Filled;
            self.filled_at = Some(Utc::now());
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_order_request_market() {
        let request = OrderRequest::market("AAPL", Side::Buy, dec!(100));
        assert_eq!(request.symbol, "AAPL");
        assert_eq!(request.side, Side::Buy);
        assert_eq!(request.order_type, OrderType::Market);
        assert_eq!(request.quantity, dec!(100));
    }

    #[test]
    fn test_order_request_limit() {
        let request = OrderRequest::limit("AAPL", Side::Sell, dec!(50), dec!(150.00));
        assert_eq!(request.order_type, OrderType::Limit);
        assert_eq!(request.limit_price, Some(dec!(150.00)));
    }

    #[test]
    fn test_order_from_request() {
        let request = OrderRequest::market("AAPL", Side::Buy, dec!(100));
        let order = Order::from_request(&request);

        assert_eq!(order.symbol, "AAPL");
        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.filled_quantity, Decimal::ZERO);
    }

    #[test]
    fn test_order_add_fill() {
        let request = OrderRequest::market("AAPL", Side::Buy, dec!(100));
        let mut order = Order::from_request(&request);

        let fill = Fill {
            id: "fill1".to_string(),
            order_id: order.id,
            quantity: dec!(50),
            price: dec!(150.00),
            commission: Decimal::ZERO,
            timestamp: Utc::now(),
        };

        order.add_fill(fill);
        assert_eq!(order.filled_quantity, dec!(50));
        assert_eq!(order.filled_avg_price, Some(dec!(150.00)));
        assert_eq!(order.status, OrderStatus::PartiallyFilled);

        let fill2 = Fill {
            id: "fill2".to_string(),
            order_id: order.id,
            quantity: dec!(50),
            price: dec!(151.00),
            commission: Decimal::ZERO,
            timestamp: Utc::now(),
        };

        order.add_fill(fill2);
        assert_eq!(order.filled_quantity, dec!(100));
        assert_eq!(order.status, OrderStatus::Filled);
    }

    #[test]
    fn test_side_opposite() {
        assert_eq!(Side::Buy.opposite(), Side::Sell);
        assert_eq!(Side::Sell.opposite(), Side::Buy);
    }
}
