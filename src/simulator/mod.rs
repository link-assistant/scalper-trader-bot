//! Market simulator for backtesting and testing trading strategies.
//!
//! This module provides a simulated exchange that can be used for:
//! - Unit testing trading logic without real API calls
//! - Backtesting strategies against historical data
//! - Integration testing the full trading system
//! - Evaluating strategy performance

mod engine;
mod market;

pub use engine::SimulatorEngine;
pub use market::{MarketCondition, SimulatedMarket, SimulatedMarketConfig};
