//! Core domain types for the scalper trader bot.
//!
//! This module defines the fundamental types used throughout the trading system,
//! following domain-driven design principles.

mod money;
mod order;
mod order_book;
mod position;
mod trade;

pub use money::{Currency, Money, Price, Quantity};
pub use order::{Order, OrderId, OrderSide, OrderStatus, OrderType};
pub use order_book::{OrderBook, OrderBookEntry, OrderBookLevel};
pub use position::Position;
pub use trade::{Trade, TradeHistory, TradeId};
