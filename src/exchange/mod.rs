//! Exchange abstraction for multi-exchange support.
//!
//! This module provides traits and types for interacting with different
//! trading exchanges in a unified manner.

mod error;
mod traits;

pub use error::ExchangeError;
pub use traits::{Exchange, ExchangeInfo, InstrumentInfo, MarketDataProvider, OrderExecutor};
