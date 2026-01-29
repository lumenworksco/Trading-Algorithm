# Trading System

A high-performance algorithmic trading system written in Rust, featuring multiple trading strategies, backtesting capabilities, and real-time monitoring.

## Features

- **4 Trading Strategies**
  - **MA Crossover** - Fast/slow moving average crossover signals
  - **Mean Reversion** - Bollinger Band mean reversion trading
  - **Momentum** - Trend following with RSI confirmation
  - **RSI Strategy** - Overbought/oversold reversal trading

- **SIMD-Optimized Indicators** - High-performance technical indicators using SIMD instructions
- **Backtesting Engine** - Event-driven simulation with detailed performance metrics
- **Risk Management** - Position sizing, stop-loss, and portfolio limits
- **Paper Trading** - Real-time paper trading via Alpaca API
- **Live Trading** - Alpaca API integration for live markets *(planned — not yet implemented)*
- **TUI Dashboard** - Real-time monitoring with terminal UI

## Installation

### Prerequisites

- Rust 1.75 or later
- Cargo (comes with Rust)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/trading-system.git
cd trading-system

# Build in release mode
cargo build --release

# The binary will be at ./target/release/trading
```

## Quick Start

### 1. List Available Strategies

```bash
./target/release/trading strategies
```

### 2. Run a Backtest

The repository includes mock historical data in the `data/` directory for AAPL, GOOGL, MSFT, SPY, QQQ, and TSLA.

**Single symbol** (pass a CSV file directly):

```bash
./target/release/trading backtest \
  --strategy ma_crossover \
  --symbols AAPL \
  --start 2023-01-01 \
  --end 2024-01-01 \
  --capital 100000 \
  --data ./data/aapl_daily.csv
```

**Multiple symbols** (pass the data directory — it automatically finds `{symbol}_daily.csv` files):

```bash
./target/release/trading backtest \
  --strategy ma_crossover \
  --symbols AAPL,GOOGL,MSFT \
  --start 2023-01-01 \
  --end 2024-01-01 \
  --capital 100000 \
  --data ./data
```

**Save results as JSON:**

```bash
./target/release/trading backtest \
  --strategy rsi \
  --symbols AAPL \
  --start 2023-01-01 \
  --end 2024-01-01 \
  --data ./data \
  --output json \
  --save results.json
```

### 3. Paper Trading

Paper trading connects to the [Alpaca](https://alpaca.markets/) paper trading API. Add your API credentials to `config/default.toml`:

```toml
[alpaca]
api_key_env = "YOUR_ALPACA_API_KEY"
api_secret_env = "YOUR_ALPACA_API_SECRET"
paper = true
```

Then run:

```bash
./target/release/trading paper \
  --strategy rsi \
  --symbols SPY,QQQ \
  --capital 50000 \
  --timeframe 5m
```

Alternatively, you can set environment variables instead of editing the config file:

```bash
export ALPACA_API_KEY="your_key"
export ALPACA_API_SECRET="your_secret"
```

### 4. Validate Configuration

```bash
./target/release/trading validate-config
```

## Configuration

Configuration is stored in `config/default.toml`. You can customize:

- **Alpaca API** - API credentials for paper/live trading
- **Risk Management** - Position sizing, stop-loss methods, exposure limits
- **Backtest Settings** - Default capital, commission, slippage

Example configuration:

```toml
[alpaca]
api_key_env = "YOUR_API_KEY"
api_secret_env = "YOUR_API_SECRET"
paper = true

[risk]
max_position_pct = 10.0
max_exposure_pct = 80.0
daily_loss_limit_pct = 3.0

[risk.position_sizing.percent_equity]
percent = 2.0

[risk.stop_loss.fixed_percent]
percent = 2.0
```

## Project Structure

```
trading-system/
├── Cargo.toml              # Workspace configuration
├── config/
│   └── default.toml        # Default configuration
├── data/                   # Mock historical data (CSV)
├── crates/
│   ├── trading-core/       # Core types and traits
│   ├── trading-indicators/ # Technical indicators (SIMD)
│   ├── trading-strategies/ # Strategy implementations
│   ├── trading-risk/       # Risk management
│   ├── trading-data/       # Data sources
│   ├── trading-broker/     # Broker integrations
│   ├── trading-backtest/   # Backtesting engine
│   ├── trading-monitor/    # TUI dashboard
│   └── trading-config/     # Configuration management
└── src/
    ├── main.rs             # CLI entry point
    └── cli/                # Command implementations
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `backtest` | Run backtesting simulation |
| `live` | Start live trading *(not yet implemented)* |
| `paper` | Start paper trading |
| `strategies` | List available strategies |
| `validate-config` | Validate configuration file |

### Global Options

| Option | Description |
|--------|-------------|
| `-c, --config` | Configuration file path (default: config/default.toml) |
| `-l, --log-level` | Log level: trace, debug, info, warn, error |
| `--json-logs` | Enable JSON log format |

## Strategies

### MA Crossover

Generates buy signals when the fast moving average crosses above the slow moving average, and sell signals on the opposite crossover.

**Parameters:**
- `fast_period` - Fast MA period (default: 10)
- `slow_period` - Slow MA period (default: 20)
- `use_ema` - Use EMA instead of SMA (default: true)

### Mean Reversion

Trades reversions to the mean using Bollinger Bands. Buys when price touches the lower band, sells at the upper band.

**Parameters:**
- `period` - Bollinger Band period (default: 20)
- `std_dev` - Standard deviation multiplier (default: 2.0)
- `entry_threshold` - Band touch threshold (default: 0.95)

### Momentum

Follows strong trends using price momentum with RSI confirmation.

**Parameters:**
- `momentum_period` - Lookback period (default: 10)
- `rsi_period` - RSI period (default: 14)
- `min_momentum` - Minimum momentum for entry (default: 2%)

### RSI Strategy

Trades overbought/oversold conditions based on RSI levels.

**Parameters:**
- `period` - RSI period (default: 14)
- `oversold` - Oversold threshold (default: 30)
- `overbought` - Overbought threshold (default: 70)

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p trading-strategies

# Run tests with output
cargo test -- --nocapture
```

### Running Benchmarks

```bash
cargo bench
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy --workspace
```

## Data

### Included Mock Data

The `data/` directory contains mock historical daily data (260 trading days, 2023-01-02 to 2024-01-01) for:

| File | Symbol |
|------|--------|
| `aapl_daily.csv` | Apple |
| `googl_daily.csv` | Alphabet |
| `msft_daily.csv` | Microsoft |
| `spy_daily.csv` | S&P 500 ETF |
| `qqq_daily.csv` | Nasdaq 100 ETF |
| `tsla_daily.csv` | Tesla |
| `multi_symbol_daily.csv` | All symbols combined |

### CSV Format

The backtest engine accepts CSV files with the following columns:

```csv
timestamp,open,high,low,close,volume
2023-01-03T09:30:00,130.28,130.90,129.89,130.15,1234567
2023-01-04T09:30:00,130.15,130.45,130.00,130.30,987654
```

Supported timestamp formats:
- ISO 8601: `2023-01-03T09:30:00`
- Date-time: `2023-01-03 09:30:00`
- Date only: `2023-01-03`
- Unix milliseconds: `1672746600000`
- Unix seconds: `1672746600`

### Data Directory Convention

When passing `--data ./data` (a directory), the engine looks for files matching:
- `{SYMBOL}_daily.csv` (e.g. `aapl_daily.csv` for `--symbols AAPL`)
- `{SYMBOL}.csv`
- Case-insensitive matching

## Environment Variables

| Variable | Description |
|----------|-------------|
| `ALPACA_API_KEY` | Alpaca API key (alternative to config file) |
| `ALPACA_API_SECRET` | Alpaca API secret (alternative to config file) |
| `RUST_LOG` | Log level override |

## Performance

The system is optimized for performance:

- **SIMD indicators** - 4x speedup for indicator calculations
- **Cache-aligned data structures** - Optimal memory access patterns
- **Zero-copy parsing** - Efficient data loading
- **Async I/O** - Non-blocking operations with Tokio

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Disclaimer

This software is for educational purposes only. Trading financial instruments involves substantial risk of loss. Past performance is not indicative of future results. Always do your own research and never trade with money you cannot afford to lose.
