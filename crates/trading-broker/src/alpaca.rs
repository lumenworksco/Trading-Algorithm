//! Alpaca broker integration for paper and live trading.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, header};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use trading_core::error::BrokerError;
use trading_core::traits::Broker;
use trading_core::types::{
    Bar, Fill, Order, OrderRequest, OrderStatus, OrderType, Portfolio, Position, Side,
};
use tracing::{debug, info};
use uuid::Uuid;

/// Alpaca API configuration.
#[derive(Debug, Clone)]
pub struct AlpacaConfig {
    pub api_key: String,
    pub api_secret: String,
    pub paper: bool,
}

impl AlpacaConfig {
    /// Create config directly with key and secret.
    pub fn new(api_key: String, api_secret: String, paper: bool) -> Self {
        Self { api_key, api_secret, paper }
    }

    /// Load from environment variables.
    pub fn from_env() -> Result<Self, BrokerError> {
        let api_key = std::env::var("ALPACA_API_KEY")
            .map_err(|_| BrokerError::Configuration("ALPACA_API_KEY not set".into()))?;
        let api_secret = std::env::var("ALPACA_API_SECRET")
            .map_err(|_| BrokerError::Configuration("ALPACA_API_SECRET not set".into()))?;
        let paper = std::env::var("ALPACA_PAPER")
            .map(|v| v.to_lowercase() != "false")
            .unwrap_or(true);

        Ok(Self {
            api_key,
            api_secret,
            paper,
        })
    }

    pub fn base_url(&self) -> &str {
        if self.paper {
            "https://paper-api.alpaca.markets"
        } else {
            "https://api.alpaca.markets"
        }
    }

    pub fn data_url(&self) -> &str {
        "https://data.alpaca.markets"
    }
}

/// Alpaca API response types
#[derive(Debug, Deserialize)]
struct AlpacaAccount {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    currency: String,
    cash: String,
    #[allow(dead_code)]
    portfolio_value: String,
    buying_power: String,
    equity: String,
    #[allow(dead_code)]
    last_equity: String,
    #[allow(dead_code)]
    daytrade_count: i32,
    #[allow(dead_code)]
    pattern_day_trader: bool,
}

#[derive(Debug, Deserialize)]
struct AlpacaPosition {
    #[allow(dead_code)]
    asset_id: String,
    symbol: String,
    qty: String,
    avg_entry_price: String,
    market_value: String,
    cost_basis: String,
    unrealized_pl: String,
    unrealized_plpc: String,
    current_price: String,
    #[allow(dead_code)]
    side: String,
}

#[derive(Debug, Deserialize)]
struct AlpacaOrder {
    id: String,
    #[allow(dead_code)]
    client_order_id: String,
    status: String,
    symbol: String,
    qty: String,
    filled_qty: String,
    #[serde(rename = "type")]
    order_type: String,
    side: String,
    #[allow(dead_code)]
    time_in_force: String,
    limit_price: Option<String>,
    stop_price: Option<String>,
    filled_avg_price: Option<String>,
    created_at: String,
    #[allow(dead_code)]
    updated_at: Option<String>,
    #[allow(dead_code)]
    submitted_at: Option<String>,
    filled_at: Option<String>,
    canceled_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateOrderRequest {
    symbol: String,
    qty: String,
    side: String,
    #[serde(rename = "type")]
    order_type: String,
    time_in_force: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlpacaBar {
    t: String,
    o: f64,
    h: f64,
    l: f64,
    c: f64,
    v: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AlpacaBarsResponse {
    bars: HashMap<String, Vec<AlpacaBar>>,
    #[allow(dead_code)]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlpacaSingleBarsResponse {
    bars: Vec<AlpacaBar>,
    #[allow(dead_code)]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlpacaLatestQuote {
    ap: f64,
    #[serde(rename = "as")]
    #[allow(dead_code)]
    ask_size: u64,
    bp: f64,
    #[allow(dead_code)]
    bs: u64,
    #[allow(dead_code)]
    t: String,
}

#[derive(Debug, Deserialize)]
struct AlpacaLatestQuotesResponse {
    quotes: HashMap<String, AlpacaLatestQuote>,
}

#[derive(Debug, Deserialize)]
struct AlpacaClock {
    #[allow(dead_code)]
    timestamp: String,
    is_open: bool,
    #[allow(dead_code)]
    next_open: String,
    #[allow(dead_code)]
    next_close: String,
}

/// Alpaca broker client.
pub struct AlpacaBroker {
    config: AlpacaConfig,
    client: Client,
}

impl AlpacaBroker {
    /// Create a new Alpaca broker client.
    pub fn new(config: AlpacaConfig) -> Result<Self, BrokerError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "APCA-API-KEY-ID",
            header::HeaderValue::from_str(&config.api_key)
                .map_err(|e| BrokerError::Configuration(e.to_string()))?,
        );
        headers.insert(
            "APCA-API-SECRET-KEY",
            header::HeaderValue::from_str(&config.api_secret)
                .map_err(|e| BrokerError::Configuration(e.to_string()))?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        Ok(Self { config, client })
    }

    /// Create from environment variables.
    pub fn from_env() -> Result<Self, BrokerError> {
        let config = AlpacaConfig::from_env()?;
        Self::new(config)
    }

    /// Get historical bars for a symbol.
    pub async fn get_bars(
        &self,
        symbol: &str,
        timeframe: &str,
        start: &str,
        end: &str,
        limit: Option<usize>,
    ) -> Result<Vec<Bar>, BrokerError> {
        let url = format!("{}/v2/stocks/{}/bars", self.config.data_url(), symbol);

        let mut params = vec![
            ("timeframe", timeframe.to_string()),
            ("start", start.to_string()),
            ("end", end.to_string()),
            ("feed", "iex".to_string()),
        ];

        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let resp = self.client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }

        let data: AlpacaSingleBarsResponse = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;

        let bars = data.bars.iter().map(|b| {
            let ts = DateTime::parse_from_rfc3339(&b.t)
                .map(|dt| dt.timestamp_millis())
                .unwrap_or(0);
            Bar::new(ts, b.o, b.h, b.l, b.c, b.v as f64)
        }).collect();

        Ok(bars)
    }

    /// Get latest quotes for symbols.
    pub async fn get_latest_quotes(&self, symbols: &[String]) -> Result<HashMap<String, Decimal>, BrokerError> {
        let url = format!("{}/v2/stocks/quotes/latest", self.config.data_url());
        let symbols_param = symbols.join(",");

        let resp = self.client
            .get(&url)
            .query(&[("symbols", &symbols_param), ("feed", &"iex".to_string())])
            .send()
            .await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }

        let data: AlpacaLatestQuotesResponse = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;

        let prices: HashMap<String, Decimal> = data.quotes
            .into_iter()
            .map(|(symbol, quote)| {
                let mid_price = (quote.ap + quote.bp) / 2.0;
                (symbol, Decimal::from_f64_retain(mid_price).unwrap_or(dec!(0)))
            })
            .collect();

        Ok(prices)
    }

    fn parse_order(&self, order: AlpacaOrder) -> Result<Order, BrokerError> {
        let id = Uuid::parse_str(&order.id).unwrap_or_else(|_| Uuid::new_v4());

        let side = match order.side.as_str() {
            "buy" => Side::Buy,
            "sell" => Side::Sell,
            _ => return Err(BrokerError::ApiError(format!("Unknown side: {}", order.side))),
        };

        let order_type = match order.order_type.as_str() {
            "market" => OrderType::Market,
            "limit" => OrderType::Limit,
            "stop" => OrderType::Stop,
            "stop_limit" => OrderType::StopLimit,
            _ => OrderType::Market,
        };

        let status = match order.status.as_str() {
            "new" | "accepted" | "pending_new" => OrderStatus::Pending,
            "partially_filled" => OrderStatus::PartiallyFilled,
            "filled" => OrderStatus::Filled,
            "canceled" | "expired" | "rejected" => OrderStatus::Canceled,
            _ => OrderStatus::Pending,
        };

        let quantity: Decimal = order.qty.parse().unwrap_or(dec!(0));
        let filled_qty: Decimal = order.filled_qty.parse().unwrap_or(dec!(0));
        let limit_price = order.limit_price.as_ref().and_then(|p| p.parse().ok());
        let stop_price = order.stop_price.as_ref().and_then(|p| p.parse().ok());

        let created_at = DateTime::parse_from_rfc3339(&order.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let filled_at = order.filled_at.as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let canceled_at = order.canceled_at.as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let filled_avg_price = order.filled_avg_price.as_ref().and_then(|p| p.parse().ok());

        let mut result = Order {
            id,
            client_order_id: order.client_order_id,
            symbol: order.symbol,
            side,
            order_type,
            quantity,
            limit_price,
            stop_price,
            trail_amount: None,
            time_in_force: trading_core::types::TimeInForce::Day,
            status,
            filled_quantity: filled_qty,
            filled_avg_price,
            fills: vec![],
            created_at,
            updated_at: created_at,
            submitted_at: None,
            filled_at,
            expired_at: None,
            canceled_at,
            extended_hours: false,
        };

        if status == OrderStatus::Filled || status == OrderStatus::PartiallyFilled {
            if let Some(price) = filled_avg_price {
                let fill = Fill {
                    id: Uuid::new_v4().to_string(),
                    order_id: id,
                    quantity: filled_qty,
                    price,
                    commission: dec!(0),
                    timestamp: filled_at.unwrap_or_else(Utc::now),
                };
                result.fills.push(fill);
            }
        }

        Ok(result)
    }

    fn parse_position(&self, p: AlpacaPosition) -> Position {
        let quantity: Decimal = p.qty.parse().unwrap_or(dec!(0));
        let avg_price: Decimal = p.avg_entry_price.parse().unwrap_or(dec!(0));
        let current_price: Decimal = p.current_price.parse().unwrap_or(dec!(0));
        let market_value: Decimal = p.market_value.parse().unwrap_or(dec!(0));
        let cost_basis: Decimal = p.cost_basis.parse().unwrap_or(dec!(0));
        let unrealized_pnl: Decimal = p.unrealized_pl.parse().unwrap_or(dec!(0));
        let unrealized_pnl_percent: Decimal = p.unrealized_plpc.parse().unwrap_or(dec!(0));

        Position {
            symbol: p.symbol,
            quantity,
            avg_entry_price: avg_price,
            current_price,
            market_value,
            cost_basis,
            unrealized_pnl,
            unrealized_pnl_percent,
            realized_pnl: dec!(0),
        }
    }
}

#[async_trait]
impl Broker for AlpacaBroker {
    async fn get_account(&self) -> Result<Portfolio, BrokerError> {
        let url = format!("{}/v2/account", self.config.base_url());

        let resp = self.client.get(&url).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }

        let account: AlpacaAccount = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;

        let cash: Decimal = account.cash.parse().unwrap_or(dec!(0));
        let equity: Decimal = account.equity.parse().unwrap_or(dec!(0));
        let buying_power: Decimal = account.buying_power.parse().unwrap_or(dec!(0));

        let positions = self.get_positions().await?;
        let positions_map: HashMap<String, Position> = positions
            .into_iter()
            .map(|p| (p.symbol.clone(), p))
            .collect();

        Ok(Portfolio {
            cash,
            buying_power,
            equity,
            positions: positions_map,
            total_unrealized_pnl: dec!(0),
            total_realized_pnl: dec!(0),
            initial_capital: equity,
            peak_equity: equity,
        })
    }

    async fn submit_order(&self, request: OrderRequest) -> Result<Order, BrokerError> {
        let url = format!("{}/v2/orders", self.config.base_url());

        let side = match request.side { Side::Buy => "buy", Side::Sell => "sell" };
        let order_type = match request.order_type {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
            OrderType::Stop => "stop",
            OrderType::StopLimit => "stop_limit",
            OrderType::TrailingStop => "trailing_stop",
        };

        let create_req = CreateOrderRequest {
            symbol: request.symbol.clone(),
            qty: request.quantity.to_string(),
            side: side.to_string(),
            order_type: order_type.to_string(),
            time_in_force: "day".to_string(),
            limit_price: request.limit_price.map(|p| p.to_string()),
            stop_price: request.stop_price.map(|p| p.to_string()),
        };

        debug!("Submitting order: {:?}", create_req);

        let resp = self.client.post(&url).json(&create_req).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::OrderRejected(format!("{}: {}", status, text)));
        }

        let order: AlpacaOrder = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;

        info!("Order submitted: {} {} {} @ {:?}", order.side, order.qty, order.symbol, order.limit_price);
        self.parse_order(order)
    }

    async fn cancel_order(&self, order_id: &str) -> Result<(), BrokerError> {
        let url = format!("{}/v2/orders/{}", self.config.base_url(), order_id);
        let resp = self.client.delete(&url).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }
        info!("Order canceled: {}", order_id);
        Ok(())
    }

    async fn get_order(&self, order_id: &str) -> Result<Order, BrokerError> {
        let url = format!("{}/v2/orders/{}", self.config.base_url(), order_id);
        let resp = self.client.get(&url).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::OrderNotFound(format!("{}: {}", status, text)));
        }

        let order: AlpacaOrder = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;
        self.parse_order(order)
    }

    async fn get_open_orders(&self) -> Result<Vec<Order>, BrokerError> {
        let url = format!("{}/v2/orders", self.config.base_url());
        let resp = self.client.get(&url).query(&[("status", "open")]).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }

        let orders: Vec<AlpacaOrder> = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;
        orders.into_iter().map(|o| self.parse_order(o)).collect()
    }

    async fn get_positions(&self) -> Result<Vec<Position>, BrokerError> {
        let url = format!("{}/v2/positions", self.config.base_url());
        let resp = self.client.get(&url).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }

        let positions: Vec<AlpacaPosition> = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;
        Ok(positions.into_iter().map(|p| self.parse_position(p)).collect())
    }

    async fn get_position(&self, symbol: &str) -> Result<Option<Position>, BrokerError> {
        let url = format!("{}/v2/positions/{}", self.config.base_url(), symbol);
        let resp = self.client.get(&url).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }

        let p: AlpacaPosition = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;
        Ok(Some(self.parse_position(p)))
    }

    async fn close_position(&self, symbol: &str) -> Result<Order, BrokerError> {
        let url = format!("{}/v2/positions/{}", self.config.base_url(), symbol);
        let resp = self.client.delete(&url).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }

        let order: AlpacaOrder = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;
        info!("Position closed: {}", symbol);
        self.parse_order(order)
    }

    async fn close_all_positions(&self) -> Result<Vec<Order>, BrokerError> {
        let url = format!("{}/v2/positions", self.config.base_url());
        let resp = self.client.delete(&url).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }

        let orders: Vec<AlpacaOrder> = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;
        info!("All positions closed");
        orders.into_iter().map(|o| self.parse_order(o)).collect()
    }

    async fn cancel_all_orders(&self) -> Result<(), BrokerError> {
        let url = format!("{}/v2/orders", self.config.base_url());
        let resp = self.client.delete(&url).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }
        info!("All orders canceled");
        Ok(())
    }

    async fn is_market_open(&self) -> Result<bool, BrokerError> {
        let url = format!("{}/v2/clock", self.config.base_url());
        let resp = self.client.get(&url).send().await
            .map_err(|e| BrokerError::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(BrokerError::ApiError(format!("{}: {}", status, text)));
        }

        let clock: AlpacaClock = resp.json().await
            .map_err(|e| BrokerError::ApiError(e.to_string()))?;
        Ok(clock.is_open)
    }

    fn name(&self) -> &str {
        if self.config.paper { "Alpaca Paper" } else { "Alpaca Live" }
    }
}
