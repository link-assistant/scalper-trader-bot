//! Order types for trading operations.

use super::{Price, Quantity};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for an order.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(String);

impl OrderId {
    /// Creates a new order ID from a string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generates a new random order ID.
    #[must_use]
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Returns the ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OrderId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Direction of the order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    /// Buy order - acquiring an asset.
    Buy,
    /// Sell order - disposing of an asset.
    Sell,
}

impl fmt::Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Buy => write!(f, "BUY"),
            Self::Sell => write!(f, "SELL"),
        }
    }
}

/// Type of order execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// Market order - execute immediately at current market price.
    Market,
    /// Limit order - execute only at specified price or better.
    Limit,
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Market => write!(f, "MARKET"),
            Self::Limit => write!(f, "LIMIT"),
        }
    }
}

/// Current status of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order is pending submission.
    Pending,
    /// Order has been submitted to the exchange.
    Submitted,
    /// Order is partially filled.
    PartiallyFilled,
    /// Order is completely filled.
    Filled,
    /// Order has been cancelled.
    Cancelled,
    /// Order was rejected by the exchange.
    Rejected,
    /// Order has expired.
    Expired,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "PENDING"),
            Self::Submitted => write!(f, "SUBMITTED"),
            Self::PartiallyFilled => write!(f, "PARTIALLY_FILLED"),
            Self::Filled => write!(f, "FILLED"),
            Self::Cancelled => write!(f, "CANCELLED"),
            Self::Rejected => write!(f, "REJECTED"),
            Self::Expired => write!(f, "EXPIRED"),
        }
    }
}

impl OrderStatus {
    /// Checks if the order is in a terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Filled | Self::Cancelled | Self::Rejected | Self::Expired
        )
    }

    /// Checks if the order is still active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Pending | Self::Submitted | Self::PartiallyFilled
        )
    }
}

/// Represents a trading order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    id: OrderId,
    instrument: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: Quantity,
    filled_quantity: Quantity,
    price: Option<Price>,
    status: OrderStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Order {
    /// Creates a new limit order.
    #[must_use]
    pub fn limit(
        instrument: impl Into<String>,
        side: OrderSide,
        quantity: Quantity,
        price: Price,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: OrderId::generate(),
            instrument: instrument.into(),
            side,
            order_type: OrderType::Limit,
            quantity,
            filled_quantity: Quantity::zero(),
            price: Some(price),
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates a new market order.
    #[must_use]
    pub fn market(instrument: impl Into<String>, side: OrderSide, quantity: Quantity) -> Self {
        let now = Utc::now();
        Self {
            id: OrderId::generate(),
            instrument: instrument.into(),
            side,
            order_type: OrderType::Market,
            quantity,
            filled_quantity: Quantity::zero(),
            price: None,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    /// Returns the order ID.
    #[must_use]
    pub fn id(&self) -> &OrderId {
        &self.id
    }

    /// Returns the instrument.
    #[must_use]
    pub fn instrument(&self) -> &str {
        &self.instrument
    }

    /// Returns the order side.
    #[must_use]
    pub fn side(&self) -> OrderSide {
        self.side
    }

    /// Returns the order type.
    #[must_use]
    pub fn order_type(&self) -> OrderType {
        self.order_type
    }

    /// Returns the requested quantity.
    #[must_use]
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Returns the filled quantity.
    #[must_use]
    pub fn filled_quantity(&self) -> Quantity {
        self.filled_quantity
    }

    /// Returns the remaining quantity to be filled.
    #[must_use]
    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity.subtract(self.filled_quantity)
    }

    /// Returns the limit price (if any).
    #[must_use]
    pub fn price(&self) -> Option<&Price> {
        self.price.as_ref()
    }

    /// Returns the current status.
    #[must_use]
    pub fn status(&self) -> OrderStatus {
        self.status
    }

    /// Returns the creation timestamp.
    #[must_use]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns the last update timestamp.
    #[must_use]
    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    /// Updates the order status.
    pub fn set_status(&mut self, status: OrderStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Records a partial fill.
    pub fn fill(&mut self, quantity: Quantity) {
        self.filled_quantity = self.filled_quantity.add(quantity);
        self.updated_at = Utc::now();

        if self.filled_quantity >= self.quantity {
            self.status = OrderStatus::Filled;
        } else {
            self.status = OrderStatus::PartiallyFilled;
        }
    }

    /// Marks the order as submitted.
    pub fn submit(&mut self) {
        if self.status == OrderStatus::Pending {
            self.status = OrderStatus::Submitted;
            self.updated_at = Utc::now();
        }
    }

    /// Cancels the order.
    pub fn cancel(&mut self) {
        if self.status.is_active() {
            self.status = OrderStatus::Cancelled;
            self.updated_at = Utc::now();
        }
    }

    /// Checks if this is a buy order.
    #[must_use]
    pub fn is_buy(&self) -> bool {
        self.side == OrderSide::Buy
    }

    /// Checks if this is a sell order.
    #[must_use]
    pub fn is_sell(&self) -> bool {
        self.side == OrderSide::Sell
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Currency;

    mod order_id_tests {
        use super::*;

        #[test]
        fn test_order_id_generation() {
            let id1 = OrderId::generate();
            let id2 = OrderId::generate();
            assert_ne!(id1, id2);
        }

        #[test]
        fn test_order_id_from_string() {
            let id = OrderId::new("test-order-123");
            assert_eq!(id.as_str(), "test-order-123");
        }
    }

    mod order_tests {
        use super::*;

        #[test]
        fn test_limit_order_creation() {
            let price = Price::from_f64(100.0, Currency::usd()).unwrap();
            let qty = Quantity::from_lots(10);
            let order = Order::limit("AAPL", OrderSide::Buy, qty, price);

            assert_eq!(order.instrument(), "AAPL");
            assert_eq!(order.side(), OrderSide::Buy);
            assert_eq!(order.order_type(), OrderType::Limit);
            assert_eq!(order.status(), OrderStatus::Pending);
        }

        #[test]
        fn test_market_order_creation() {
            let qty = Quantity::from_lots(5);
            let order = Order::market("BTC/USDT", OrderSide::Sell, qty);

            assert_eq!(order.instrument(), "BTC/USDT");
            assert_eq!(order.side(), OrderSide::Sell);
            assert_eq!(order.order_type(), OrderType::Market);
            assert!(order.price().is_none());
        }

        #[test]
        fn test_order_fill() {
            let price = Price::from_f64(100.0, Currency::usd()).unwrap();
            let qty = Quantity::from_lots(10);
            let mut order = Order::limit("AAPL", OrderSide::Buy, qty, price);

            order.submit();
            assert_eq!(order.status(), OrderStatus::Submitted);

            order.fill(Quantity::from_lots(5));
            assert_eq!(order.status(), OrderStatus::PartiallyFilled);
            assert_eq!(order.filled_quantity().as_lots(), 5);
            assert_eq!(order.remaining_quantity().as_lots(), 5);

            order.fill(Quantity::from_lots(5));
            assert_eq!(order.status(), OrderStatus::Filled);
            assert_eq!(order.remaining_quantity().as_lots(), 0);
        }

        #[test]
        fn test_order_cancel() {
            let qty = Quantity::from_lots(10);
            let mut order = Order::market("AAPL", OrderSide::Buy, qty);

            order.submit();
            order.cancel();

            assert_eq!(order.status(), OrderStatus::Cancelled);
        }

        #[test]
        fn test_status_is_terminal() {
            assert!(OrderStatus::Filled.is_terminal());
            assert!(OrderStatus::Cancelled.is_terminal());
            assert!(OrderStatus::Rejected.is_terminal());
            assert!(!OrderStatus::Pending.is_terminal());
            assert!(!OrderStatus::Submitted.is_terminal());
        }
    }
}
