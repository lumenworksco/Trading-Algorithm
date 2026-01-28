# Contributing to Trading System

Thank you for your interest in contributing to Trading System! This document provides guidelines and information for contributors.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates.

When reporting a bug, include:

1. **Description** - Clear description of the issue
2. **Steps to Reproduce** - Detailed steps to reproduce the behavior
3. **Expected Behavior** - What you expected to happen
4. **Actual Behavior** - What actually happened
5. **Environment** - OS, Rust version, etc.
6. **Logs/Screenshots** - Any relevant output or screenshots

### Suggesting Features

Feature requests are welcome! Please include:

1. **Problem Statement** - What problem does this solve?
2. **Proposed Solution** - How would you like it to work?
3. **Alternatives Considered** - Other solutions you've thought about
4. **Additional Context** - Any other relevant information

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following the coding standards below
3. **Add tests** for any new functionality
4. **Ensure all tests pass** with `cargo test --workspace`
5. **Update documentation** if needed
6. **Submit a pull request** with a clear description

## Development Setup

### Prerequisites

- Rust 1.75 or later
- Git

### Getting Started

```bash
# Clone your fork
git clone https://github.com/yourusername/trading-system.git
cd trading-system

# Add upstream remote
git remote add upstream https://github.com/originalowner/trading-system.git

# Create a feature branch
git checkout -b feature/your-feature-name

# Build the project
cargo build

# Run tests
cargo test --workspace
```

## Coding Standards

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Write documentation for public APIs

### Code Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy --workspace -- -D warnings
```

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `style` - Code style changes (formatting, etc.)
- `refactor` - Code refactoring
- `test` - Adding or modifying tests
- `chore` - Maintenance tasks

Examples:
```
feat(strategies): add MACD crossover strategy
fix(backtest): correct drawdown calculation
docs(readme): add installation instructions
```

### Testing

- Write unit tests for new functionality
- Ensure existing tests pass
- Add integration tests for complex features

```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test -p trading-strategies test_name

# Run tests with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### Documentation

- Document all public APIs with doc comments
- Include examples where helpful
- Update README.md for user-facing changes

```rust
/// Calculates the Simple Moving Average (SMA) for a given period.
///
/// # Arguments
///
/// * `data` - Slice of price data
/// * `period` - Number of periods for the average
///
/// # Returns
///
/// Vector of SMA values, or empty if insufficient data
///
/// # Example
///
/// ```
/// let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let sma = calculate_sma(&prices, 3);
/// ```
pub fn calculate_sma(data: &[f64], period: usize) -> Vec<f64> {
    // implementation
}
```

## Project Structure

Understanding the crate structure helps you find where to make changes:

| Crate | Purpose |
|-------|---------|
| `trading-core` | Core types, traits, and errors |
| `trading-indicators` | Technical indicators (add new indicators here) |
| `trading-strategies` | Strategy implementations (add new strategies here) |
| `trading-risk` | Risk management components |
| `trading-data` | Data sources and loading |
| `trading-broker` | Broker integrations |
| `trading-backtest` | Backtesting engine |
| `trading-monitor` | TUI dashboard |
| `trading-config` | Configuration management |

## Adding a New Strategy

1. Create a new file in `crates/trading-strategies/src/`
2. Implement the `Strategy` trait from `trading-core`
3. Add configuration struct with `StrategyConfig` trait
4. Register in `registry.rs`
5. Add tests
6. Update documentation

Example skeleton:

```rust
use trading_core::{
    traits::{Strategy, StrategyConfig},
    types::{BarSeries, Signal},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyStrategyConfig {
    pub symbols: Vec<String>,
    pub param1: f64,
}

impl StrategyConfig for MyStrategyConfig {
    fn validate(&self) -> Result<(), StrategyError> {
        // validation logic
        Ok(())
    }
}

pub struct MyStrategy {
    config: MyStrategyConfig,
}

impl Strategy for MyStrategy {
    fn name(&self) -> &str { "My Strategy" }

    fn on_bar(&mut self, series: &BarSeries) -> Option<Signal> {
        // strategy logic
        None
    }

    // ... other trait methods
}
```

## Adding a New Indicator

1. Create a new file in `crates/trading-indicators/src/`
2. Implement the `Indicator` trait
3. Add SIMD optimization if applicable
4. Add tests and benchmarks
5. Export from `lib.rs`

## Review Process

1. All PRs require at least one review
2. CI must pass (tests, formatting, clippy)
3. Documentation must be updated if applicable
4. Breaking changes require discussion

## Getting Help

- Open an issue for questions
- Check existing documentation
- Look at similar implementations in the codebase

## Recognition

Contributors will be recognized in:
- Git commit history
- CHANGELOG.md for significant contributions
- README.md contributors section (for major contributions)

Thank you for contributing!
