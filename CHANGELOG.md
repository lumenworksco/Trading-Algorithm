# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of the trading system

## [0.1.0] - 2024-01-28

### Added

#### Core Features
- Modular Rust workspace with 9 specialized crates
- High-performance SIMD-optimized technical indicators
- Event-driven backtesting engine
- Paper trading broker simulation
- Risk management system with position sizing and stop-loss

#### Trading Strategies
- **MA Crossover** - Moving average crossover strategy with configurable periods
- **Mean Reversion** - Bollinger Band-based mean reversion trading
- **Momentum** - Trend following with momentum and RSI confirmation
- **RSI Strategy** - Overbought/oversold reversal trading

#### Technical Indicators
- Simple Moving Average (SMA)
- Exponential Moving Average (EMA)
- Weighted Moving Average (WMA)
- Relative Strength Index (RSI)
- MACD (Moving Average Convergence Divergence)
- Stochastic Oscillator
- Bollinger Bands
- Average True Range (ATR)
- Standard Deviation

#### Risk Management
- Position sizing methods: Fixed shares, Percent equity, Risk-based, Kelly criterion
- Stop-loss types: Fixed percent, ATR-based, Trailing percent, Trailing ATR
- Portfolio limits: Max position size, Max exposure, Daily loss limit, Max drawdown

#### CLI Interface
- `backtest` - Run backtesting simulations
- `paper` - Paper trading mode
- `live` - Live trading mode (placeholder)
- `strategies` - List available strategies
- `validate-config` - Configuration validation

#### Data Support
- CSV data loader for historical data
- Multiple timestamp format support
- Data caching layer

#### Configuration
- TOML-based configuration system
- Environment variable support for API credentials
- Per-strategy configuration files

### Technical Details
- Rust 1.75+ required
- Async runtime with Tokio
- SIMD optimizations using `wide` crate
- Precise decimal arithmetic with `rust_decimal`
- Comprehensive test suite (92 tests)

### Documentation
- README with usage examples
- Contributing guidelines
- MIT License

[Unreleased]: https://github.com/yourusername/trading-system/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/trading-system/releases/tag/v0.1.0
