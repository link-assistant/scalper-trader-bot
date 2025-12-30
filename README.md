# Scalper Trader Bot

A Rust-based automated trading bot with multi-exchange support, market simulator, and comprehensive testing capabilities.

[![CI/CD Pipeline](https://github.com/link-assistant/scalper-trader-bot/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/link-assistant/scalper-trader-bot/actions)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](http://unlicense.org/)

## Overview

This project is a Rust reimplementation inspired by [linksplatform/Bot/TraderBot](https://github.com/linksplatform/Bot/tree/main/csharp/TraderBot) and [suenot/tinkoff-invest-etf-balancer-bot](https://github.com/suenot/tinkoff-invest-etf-balancer-bot), designed with the [code-architecture-principles](https://github.com/link-foundation/code-architecture-principles) in mind.

## Features

- **Scalping Strategy**: Automated buy-low, sell-high trading with configurable profit targets
- **Multi-Exchange Support**: Unified abstraction layer for different exchanges
  - T-Bank (formerly Tinkoff) - Russian broker
  - Binance - Cryptocurrency exchange
  - Interactive Brokers - International broker
- **Market Simulator**: Full-featured simulator for backtesting and testing
  - Configurable market conditions (bullish, bearish, ranging, volatile)
  - Realistic order book simulation
  - Trade execution with commissions
- **Comprehensive Testing**: Unit tests, integration tests, and simulation tests
- **Type Safety**: Strongly-typed domain model with precise decimal arithmetic

## Architecture

```
src/
├── types/           # Core domain types (Money, Price, Order, Position, Trade)
├── exchange/        # Exchange abstraction traits and error types
├── simulator/       # Market simulator for testing and backtesting
├── strategy/        # Trading strategies (Scalping, etc.)
└── adapters/        # Exchange-specific implementations
    ├── tbank.rs     # T-Bank (Tinkoff) adapter
    ├── binance.rs   # Binance adapter
    └── interactive_brokers.rs  # Interactive Brokers adapter
```

## Quick Start

### Prerequisites

- Rust 1.70 or later
- Cargo

### Installation

```bash
git clone https://github.com/link-assistant/scalper-trader-bot.git
cd scalper-trader-bot
cargo build
```

### Running Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run with verbose output
cargo test --verbose

# Run only unit tests
cargo test --lib

# Run only integration tests
cargo test --test integration_test
```

### Running the Demo

```bash
cargo run
```

### Running the Example

```bash
cargo run --example basic_usage
```

## Usage

### Basic Example

```rust
use scalper_trader_bot::prelude::*;
use scalper_trader_bot::simulator::SimulatedMarketConfig;
use rust_decimal::Decimal;

#[tokio::main]
async fn main() {
    // Create a simulated exchange
    let mut engine = SimulatorEngine::new();

    engine.add_market(SimulatedMarketConfig {
        symbol: "ETH/USDT".to_string(),
        currency: Currency::usdt(),
        initial_price: Decimal::from(2000),
        ..Default::default()
    });

    engine.add_balance(Currency::usdt(), Decimal::from(10000));
    engine.connect().await.unwrap();

    // Create a scalping strategy
    let settings = TradingSettings::new("ETH/USDT")
        .with_minimum_profit_steps(2)
        .with_max_position(10);

    let strategy = ScalpingStrategy::new(settings);

    // Execute trades
    let order = Order::market("ETH/USDT", OrderSide::Buy, Quantity::from_lots(5));
    let filled = engine.submit_order(order).await.unwrap();

    println!("Filled: {} lots at {}", filled.filled_quantity(), filled.price().unwrap());
}
```

## Configuration

Trading settings can be configured through the `TradingSettings` builder:

```rust
let settings = TradingSettings::new("SYMBOL")
    .with_minimum_profit_steps(2)      // Minimum profit in price steps
    .with_price_step(Decimal::new(1, 2)) // Price step (tick size)
    .with_lot_size(1)                   // Lot size for orders
    .with_max_position(100)             // Maximum position size
    .with_order_book_depth(5)           // Order book depth to analyze
    .with_trading_hours(                // Trading time window
        NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
    );
```

## Exchange Adapters

### Simulator (Ready for Use)

The simulator is fully implemented and ready for testing:

```rust
let mut engine = SimulatorEngine::new();
engine.add_market(config);
engine.connect().await?;
```

### T-Bank, Binance, Interactive Brokers (Placeholders)

These adapters are placeholder implementations. To complete them:

1. Add the appropriate SDK dependency to `Cargo.toml`
2. Implement the `connect()` method with authentication
3. Implement market data methods (`get_order_book`, etc.)
4. Implement order execution methods (`submit_order`, etc.)

## Testing Strategy

The codebase follows a comprehensive testing approach:

1. **Unit Tests**: Every module includes unit tests for isolated functionality
2. **Integration Tests**: Tests in `tests/` verify the full trading workflow
3. **Simulator Tests**: Use the market simulator for realistic scenario testing
4. **Example Scripts**: Runnable examples in `examples/` serve as documentation

## Code Quality

This project uses:

- **rustfmt**: Standard Rust code formatting
- **Clippy**: Linting with pedantic and nursery lints
- **Pre-commit hooks**: Automated quality checks

```bash
# Format code
cargo fmt

# Run lints
cargo clippy --all-targets --all-features
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes and add tests
4. Run quality checks: `cargo fmt && cargo clippy && cargo test`
5. Add a changelog fragment
6. Commit and push
7. Create a Pull Request

## License

[Unlicense](LICENSE) - Public Domain

## Acknowledgments

- [linksplatform/Bot](https://github.com/linksplatform/Bot) - Original C# implementation
- [suenot/tinkoff-invest-etf-balancer-bot](https://github.com/suenot/tinkoff-invest-etf-balancer-bot) - ETF balancer inspiration
- [link-foundation/code-architecture-principles](https://github.com/link-foundation/code-architecture-principles) - Architecture guidelines
