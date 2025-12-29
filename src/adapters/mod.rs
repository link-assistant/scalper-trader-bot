//! Exchange adapters for connecting to real trading APIs.
//!
//! This module provides adapters for various exchanges and brokers:
//! - T-Bank (formerly Tinkoff) - Russian broker
//! - Binance - Cryptocurrency exchange
//! - Interactive Brokers - International broker
//!
//! Each adapter implements the `Exchange` trait for unified interaction.

pub mod binance;
pub mod interactive_brokers;
pub mod tbank;

pub use binance::BinanceAdapter;
pub use interactive_brokers::InteractiveBrokersAdapter;
pub use tbank::TBankAdapter;
