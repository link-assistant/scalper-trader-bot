//! Exchange traits for abstracting exchange operations.

use super::ExchangeError;
use crate::types::{Currency, Money, Order, OrderBook, OrderId, Position, Trade};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about an exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeInfo {
    /// Name of the exchange.
    pub name: String,
    /// Exchange identifier.
    pub id: String,
    /// Whether the exchange is currently connected.
    pub connected: bool,
    /// List of supported instruments.
    pub instruments: Vec<InstrumentInfo>,
}

/// Information about a tradeable instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentInfo {
    /// Symbol/ticker of the instrument.
    pub symbol: String,
    /// Human-readable name.
    pub name: String,
    /// Trading currency.
    pub currency: Currency,
    /// Minimum order quantity.
    pub min_quantity: rust_decimal::Decimal,
    /// Quantity step size.
    pub quantity_step: rust_decimal::Decimal,
    /// Price step size (tick size).
    pub price_step: rust_decimal::Decimal,
    /// Lot size (number of units per lot).
    pub lot_size: u64,
    /// Whether trading is currently enabled.
    pub trading_enabled: bool,
}

/// Provides market data from an exchange.
#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    /// Connects to the market data feed.
    async fn connect(&mut self) -> Result<(), ExchangeError>;

    /// Disconnects from the market data feed.
    async fn disconnect(&mut self) -> Result<(), ExchangeError>;

    /// Checks if connected to the market data feed.
    fn is_connected(&self) -> bool;

    /// Gets the current order book for an instrument.
    async fn get_order_book(&self, instrument: &str) -> Result<OrderBook, ExchangeError>;

    /// Gets information about an instrument.
    async fn get_instrument_info(&self, instrument: &str) -> Result<InstrumentInfo, ExchangeError>;

    /// Lists all available instruments.
    async fn list_instruments(&self) -> Result<Vec<InstrumentInfo>, ExchangeError>;
}

/// Executes orders on an exchange.
#[async_trait]
pub trait OrderExecutor: Send + Sync {
    /// Submits an order to the exchange.
    async fn submit_order(&mut self, order: Order) -> Result<Order, ExchangeError>;

    /// Cancels an order.
    async fn cancel_order(&mut self, order_id: &OrderId) -> Result<(), ExchangeError>;

    /// Gets the status of an order.
    async fn get_order(&self, order_id: &OrderId) -> Result<Order, ExchangeError>;

    /// Lists all active orders.
    async fn list_active_orders(&self) -> Result<Vec<Order>, ExchangeError>;

    /// Lists orders for a specific instrument.
    async fn list_orders_for_instrument(
        &self,
        instrument: &str,
    ) -> Result<Vec<Order>, ExchangeError>;

    /// Gets the current positions.
    async fn get_positions(&self) -> Result<HashMap<String, Position>, ExchangeError>;

    /// Gets the position for a specific instrument.
    async fn get_position(&self, instrument: &str) -> Result<Position, ExchangeError>;

    /// Gets the available cash balance.
    async fn get_balance(&self, currency: &Currency) -> Result<Money, ExchangeError>;

    /// Gets all cash balances.
    async fn get_balances(&self) -> Result<HashMap<Currency, Money>, ExchangeError>;

    /// Gets recent trades for the account.
    async fn get_trades(&self, limit: usize) -> Result<Vec<Trade>, ExchangeError>;

    /// Gets trades for a specific instrument.
    async fn get_trades_for_instrument(
        &self,
        instrument: &str,
        limit: usize,
    ) -> Result<Vec<Trade>, ExchangeError>;
}

/// A complete exchange interface combining market data and order execution.
#[async_trait]
pub trait Exchange: MarketDataProvider + OrderExecutor {
    /// Returns information about the exchange.
    fn info(&self) -> &ExchangeInfo;

    /// Returns the exchange name.
    fn name(&self) -> &str {
        &self.info().name
    }

    /// Returns the exchange ID.
    fn id(&self) -> &str {
        &self.info().id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_info() {
        let info = ExchangeInfo {
            name: "Test Exchange".to_string(),
            id: "test".to_string(),
            connected: false,
            instruments: vec![],
        };

        assert_eq!(info.name, "Test Exchange");
        assert_eq!(info.id, "test");
        assert!(!info.connected);
    }

    #[test]
    fn test_instrument_info() {
        let info = InstrumentInfo {
            symbol: "AAPL".to_string(),
            name: "Apple Inc.".to_string(),
            currency: Currency::usd(),
            min_quantity: rust_decimal::Decimal::from(1),
            quantity_step: rust_decimal::Decimal::from(1),
            price_step: rust_decimal::Decimal::new(1, 2), // 0.01
            lot_size: 1,
            trading_enabled: true,
        };

        assert_eq!(info.symbol, "AAPL");
        assert!(info.trading_enabled);
    }
}
