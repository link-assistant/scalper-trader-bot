//! Trading strategies module.
//!
//! This module provides trading strategy implementations.

mod scalper;
mod settings;
mod traits;

pub use scalper::ScalpingStrategy;
pub use settings::TradingSettings;
pub use traits::{MarketState, Strategy, StrategyAction, StrategyDecision};
