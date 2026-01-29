//! Error types for the trading system.

use thiserror::Error;

/// Top-level trading system error.
#[derive(Error, Debug)]
pub enum TradingError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Strategy error: {0}")]
    Strategy(#[from] StrategyError),

    #[error("Broker error: {0}")]
    Broker(#[from] BrokerError),

    #[error("Data error: {0}")]
    Data(#[from] DataError),

    #[error("Indicator error: {0}")]
    Indicator(#[from] IndicatorError),

    #[error("Risk management blocked order: {reason}")]
    RiskBlocked { reason: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Strategy-specific errors.
#[derive(Error, Debug)]
pub enum StrategyError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Insufficient data: need {required} bars, have {available}")]
    InsufficientData { required: usize, available: usize },

    #[error("Strategy not found: {0}")]
    NotFound(String),

    #[error("Strategy initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Strategy error: {0}")]
    Internal(String),
}

/// Broker-specific errors.
#[derive(Error, Debug)]
pub enum BrokerError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Order rejected: {0}")]
    OrderRejected(String),

    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds {
        required: rust_decimal::Decimal,
        available: rust_decimal::Decimal,
    },

    #[error("Position not found: {0}")]
    PositionNotFound(String),

    #[error("Order not found: {0}")]
    OrderNotFound(String),

    #[error("Rate limited: retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    #[error("Market closed")]
    MarketClosed,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

/// Data source errors.
#[derive(Error, Debug)]
pub enum DataError {
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("No data available for the requested range")]
    NoDataAvailable,

    #[error("Invalid timeframe: {0}")]
    InvalidTimeframe(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Data source error: {0}")]
    Internal(String),
}

/// Indicator calculation errors.
#[derive(Error, Debug)]
pub enum IndicatorError {
    #[error("Insufficient data: need {required} points, have {available}")]
    InsufficientData { required: usize, available: usize },

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),
}

/// Result type alias for trading operations.
pub type TradingResult<T> = Result<T, TradingError>;
