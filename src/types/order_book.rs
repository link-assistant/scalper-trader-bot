//! Order book types for market data.

use super::{Price, Quantity};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// A single entry in the order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    price: Price,
    quantity: Quantity,
}

impl OrderBookEntry {
    /// Creates a new order book entry.
    #[must_use]
    pub fn new(price: Price, quantity: Quantity) -> Self {
        Self { price, quantity }
    }

    /// Returns the price.
    #[must_use]
    pub fn price(&self) -> &Price {
        &self.price
    }

    /// Returns the quantity.
    #[must_use]
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }
}

/// A price level in the order book (aggregated entries at same price).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookLevel {
    price: Price,
    total_quantity: Quantity,
    order_count: u32,
}

impl OrderBookLevel {
    /// Creates a new order book level.
    #[must_use]
    pub fn new(price: Price, total_quantity: Quantity, order_count: u32) -> Self {
        Self {
            price,
            total_quantity,
            order_count,
        }
    }

    /// Returns the price.
    #[must_use]
    pub fn price(&self) -> &Price {
        &self.price
    }

    /// Returns the total quantity at this level.
    #[must_use]
    pub fn total_quantity(&self) -> Quantity {
        self.total_quantity
    }

    /// Returns the number of orders at this level.
    #[must_use]
    pub fn order_count(&self) -> u32 {
        self.order_count
    }
}

/// Represents a snapshot of the order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    instrument: String,
    bids: Vec<OrderBookLevel>,
    asks: Vec<OrderBookLevel>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl OrderBook {
    /// Creates a new order book.
    #[must_use]
    pub fn new(instrument: impl Into<String>) -> Self {
        Self {
            instrument: instrument.into(),
            bids: Vec::new(),
            asks: Vec::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Creates an order book with given bids and asks.
    #[must_use]
    pub fn with_levels(
        instrument: impl Into<String>,
        bids: Vec<OrderBookLevel>,
        asks: Vec<OrderBookLevel>,
    ) -> Self {
        let mut book = Self {
            instrument: instrument.into(),
            bids,
            asks,
            timestamp: chrono::Utc::now(),
        };
        book.sort_levels();
        book
    }

    /// Returns the instrument.
    #[must_use]
    pub fn instrument(&self) -> &str {
        &self.instrument
    }

    /// Returns the bid levels (buy orders), sorted by price descending.
    #[must_use]
    pub fn bids(&self) -> &[OrderBookLevel] {
        &self.bids
    }

    /// Returns the ask levels (sell orders), sorted by price ascending.
    #[must_use]
    pub fn asks(&self) -> &[OrderBookLevel] {
        &self.asks
    }

    /// Returns the timestamp.
    #[must_use]
    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        self.timestamp
    }

    /// Returns the best bid (highest buy price).
    #[must_use]
    pub fn best_bid(&self) -> Option<&OrderBookLevel> {
        self.bids.first()
    }

    /// Returns the best ask (lowest sell price).
    #[must_use]
    pub fn best_ask(&self) -> Option<&OrderBookLevel> {
        self.asks.first()
    }

    /// Calculates the spread (difference between best ask and best bid).
    #[must_use]
    pub fn spread(&self) -> Option<rust_decimal::Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => ask.price().difference(bid.price()).ok(),
            _ => None,
        }
    }

    /// Returns the mid-price.
    #[must_use]
    pub fn mid_price(&self) -> Option<rust_decimal::Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => {
                Some((bid.price().value() + ask.price().value()) / rust_decimal::Decimal::from(2))
            }
            _ => None,
        }
    }

    /// Adds a bid level.
    pub fn add_bid(&mut self, level: OrderBookLevel) {
        self.bids.push(level);
        self.sort_levels();
    }

    /// Adds an ask level.
    pub fn add_ask(&mut self, level: OrderBookLevel) {
        self.asks.push(level);
        self.sort_levels();
    }

    /// Sets the bids.
    pub fn set_bids(&mut self, bids: Vec<OrderBookLevel>) {
        self.bids = bids;
        self.sort_levels();
    }

    /// Sets the asks.
    pub fn set_asks(&mut self, asks: Vec<OrderBookLevel>) {
        self.asks = asks;
        self.sort_levels();
    }

    /// Updates the timestamp.
    pub fn update_timestamp(&mut self) {
        self.timestamp = chrono::Utc::now();
    }

    /// Sorts the levels: bids descending by price, asks ascending by price.
    fn sort_levels(&mut self) {
        // Bids: highest price first
        self.bids.sort_by(|a, b| {
            b.price()
                .value()
                .partial_cmp(&a.price().value())
                .unwrap_or(Ordering::Equal)
        });
        // Asks: lowest price first
        self.asks.sort_by(|a, b| {
            a.price()
                .value()
                .partial_cmp(&b.price().value())
                .unwrap_or(Ordering::Equal)
        });
    }

    /// Gets total quantity available at bids up to a given depth.
    #[must_use]
    pub fn bid_depth(&self, levels: usize) -> Quantity {
        self.bids
            .iter()
            .take(levels)
            .fold(Quantity::zero(), |acc, level| {
                acc.add(level.total_quantity())
            })
    }

    /// Gets total quantity available at asks up to a given depth.
    #[must_use]
    pub fn ask_depth(&self, levels: usize) -> Quantity {
        self.asks
            .iter()
            .take(levels)
            .fold(Quantity::zero(), |acc, level| {
                acc.add(level.total_quantity())
            })
    }

    /// Checks if the order book has sufficient liquidity for a given quantity.
    #[must_use]
    pub fn has_liquidity(&self, side: super::OrderSide, quantity: Quantity) -> bool {
        match side {
            super::OrderSide::Buy => {
                self.asks.iter().fold(Quantity::zero(), |acc, level| {
                    acc.add(level.total_quantity())
                }) >= quantity
            }
            super::OrderSide::Sell => {
                self.bids.iter().fold(Quantity::zero(), |acc, level| {
                    acc.add(level.total_quantity())
                }) >= quantity
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Currency;

    fn create_test_order_book() -> OrderBook {
        let bids = vec![
            OrderBookLevel::new(
                Price::from_f64(99.0, Currency::usd()).unwrap(),
                Quantity::from_lots(100),
                5,
            ),
            OrderBookLevel::new(
                Price::from_f64(98.0, Currency::usd()).unwrap(),
                Quantity::from_lots(200),
                10,
            ),
        ];

        let asks = vec![
            OrderBookLevel::new(
                Price::from_f64(101.0, Currency::usd()).unwrap(),
                Quantity::from_lots(150),
                7,
            ),
            OrderBookLevel::new(
                Price::from_f64(102.0, Currency::usd()).unwrap(),
                Quantity::from_lots(250),
                12,
            ),
        ];

        OrderBook::with_levels("TEST", bids, asks)
    }

    #[test]
    fn test_order_book_creation() {
        let book = create_test_order_book();
        assert_eq!(book.instrument(), "TEST");
        assert_eq!(book.bids().len(), 2);
        assert_eq!(book.asks().len(), 2);
    }

    #[test]
    fn test_best_bid_ask() {
        let book = create_test_order_book();

        let best_bid = book.best_bid().unwrap();
        assert_eq!(best_bid.price().value(), rust_decimal::Decimal::from(99));

        let best_ask = book.best_ask().unwrap();
        assert_eq!(best_ask.price().value(), rust_decimal::Decimal::from(101));
    }

    #[test]
    fn test_spread() {
        let book = create_test_order_book();
        let spread = book.spread().unwrap();
        assert_eq!(spread, rust_decimal::Decimal::from(2)); // 101 - 99 = 2
    }

    #[test]
    fn test_mid_price() {
        let book = create_test_order_book();
        let mid = book.mid_price().unwrap();
        assert_eq!(mid, rust_decimal::Decimal::from(100)); // (99 + 101) / 2 = 100
    }

    #[test]
    fn test_bid_depth() {
        let book = create_test_order_book();
        assert_eq!(book.bid_depth(1).as_lots(), 100);
        assert_eq!(book.bid_depth(2).as_lots(), 300);
    }

    #[test]
    fn test_ask_depth() {
        let book = create_test_order_book();
        assert_eq!(book.ask_depth(1).as_lots(), 150);
        assert_eq!(book.ask_depth(2).as_lots(), 400);
    }

    #[test]
    fn test_has_liquidity() {
        let book = create_test_order_book();
        use crate::types::OrderSide;

        // Should have liquidity for buying 100 lots
        assert!(book.has_liquidity(OrderSide::Buy, Quantity::from_lots(100)));
        // Should not have liquidity for buying 500 lots (only 400 available)
        assert!(!book.has_liquidity(OrderSide::Buy, Quantity::from_lots(500)));

        // Should have liquidity for selling 200 lots
        assert!(book.has_liquidity(OrderSide::Sell, Quantity::from_lots(200)));
        // Should not have liquidity for selling 400 lots (only 300 available)
        assert!(!book.has_liquidity(OrderSide::Sell, Quantity::from_lots(400)));
    }
}
