//! Scalper Trader Bot - A Rust-based automated trading bot.
//!
//! This library provides a comprehensive trading bot framework with:
//! - Multi-exchange support through a unified abstraction layer
//! - Market simulator for backtesting and testing strategies
//! - Scalping trading strategy implementation
//! - Support for T-Bank, Binance, and Interactive Brokers
//!
//! # Architecture
//!
//! The crate is organized into several modules:
//!
//! - [`types`]: Core domain types (Money, Price, Order, Position, Trade)
//! - [`exchange`]: Exchange abstraction traits and error types
//! - [`simulator`]: Market simulator for testing and backtesting
//! - [`strategy`]: Trading strategies including the scalping strategy
//! - [`adapters`]: Exchange-specific implementations
//!
//! # Example
//!
//! ```rust
//! use scalper_trader_bot::simulator::{SimulatorEngine, SimulatedMarketConfig};
//! use scalper_trader_bot::strategy::{ScalpingStrategy, Strategy, TradingSettings};
//! use scalper_trader_bot::types::Currency;
//! use rust_decimal::Decimal;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a simulated market
//!     let mut engine = SimulatorEngine::new();
//!     engine.add_market(SimulatedMarketConfig {
//!         symbol: "TEST".to_string(),
//!         currency: Currency::usd(),
//!         initial_price: Decimal::from(100),
//!         ..Default::default()
//!     });
//!     engine.add_balance(Currency::usd(), Decimal::from(10000));
//!
//!     // Create a scalping strategy
//!     let settings = TradingSettings::new("TEST")
//!         .with_minimum_profit_steps(2)
//!         .with_max_position(10);
//!     let strategy = ScalpingStrategy::new(settings);
//!
//!     println!("Strategy: {}", strategy.name());
//! }
//! ```
//!
//! # Testing
//!
//! The library is designed for comprehensive testing:
//!
//! - Unit tests for all modules
//! - Integration tests using the market simulator
//! - Support for backtesting against historical data
//!
//! # Code Architecture Principles
//!
//! This crate follows the [code-architecture-principles](https://github.com/link-foundation/code-architecture-principles):
//!
//! - Small, focused modules for maintainability
//! - Pure functions separated from I/O operations
//! - Immutable data structures where possible
//! - Explicit error handling with Result types
//! - Comprehensive test coverage

pub mod adapters;
pub mod exchange;
pub mod simulator;
pub mod strategy;
pub mod types;

/// Package version (matches Cargo.toml version).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Re-exports for convenient access.
pub mod prelude {
    pub use crate::exchange::{
        Exchange, ExchangeError, ExchangeInfo, MarketDataProvider, OrderExecutor,
    };
    pub use crate::simulator::{SimulatedMarket, SimulatedMarketConfig, SimulatorEngine};
    pub use crate::strategy::{
        ScalpingStrategy, Strategy, StrategyAction, StrategyDecision, TradingSettings,
    };
    pub use crate::types::{
        Currency, Money, Order, OrderBook, OrderId, OrderSide, OrderStatus, OrderType, Position,
        Price, Quantity, Trade, TradeId,
    };
}
