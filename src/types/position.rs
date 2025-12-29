//! Position types for tracking holdings.

use super::{Currency, Money, Price, Quantity};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents a position in an instrument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    instrument: String,
    quantity: Quantity,
    average_price: Price,
    currency: Currency,
}

impl Position {
    /// Creates a new position.
    #[must_use]
    pub fn new(instrument: impl Into<String>, quantity: Quantity, average_price: Price) -> Self {
        let currency = average_price.currency().clone();
        Self {
            instrument: instrument.into(),
            quantity,
            average_price,
            currency,
        }
    }

    /// Creates an empty position for an instrument.
    #[must_use]
    pub fn empty(instrument: impl Into<String>, currency: Currency) -> Self {
        Self {
            instrument: instrument.into(),
            quantity: Quantity::zero(),
            average_price: Price::new(Decimal::ZERO, currency.clone()).unwrap_or_else(|_| {
                // This should never fail with ZERO
                panic!("Failed to create zero price")
            }),
            currency,
        }
    }

    /// Returns the instrument.
    #[must_use]
    pub fn instrument(&self) -> &str {
        &self.instrument
    }

    /// Returns the quantity.
    #[must_use]
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Returns the average price.
    #[must_use]
    pub fn average_price(&self) -> &Price {
        &self.average_price
    }

    /// Returns the currency.
    #[must_use]
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Checks if the position is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.quantity.is_zero()
    }

    /// Returns the total cost basis.
    #[must_use]
    pub fn cost_basis(&self) -> Money {
        Money::new(
            self.average_price.value() * self.quantity.value(),
            self.currency.clone(),
        )
    }

    /// Calculates unrealized profit/loss at a given current price.
    #[must_use]
    pub fn unrealized_pnl(&self, current_price: &Price) -> Money {
        if self.quantity.is_zero() {
            return Money::zero(self.currency.clone());
        }

        let current_value = current_price.value() * self.quantity.value();
        let cost = self.average_price.value() * self.quantity.value();
        Money::new(current_value - cost, self.currency.clone())
    }

    /// Calculates unrealized profit/loss percentage.
    #[must_use]
    pub fn unrealized_pnl_percent(&self, current_price: &Price) -> Decimal {
        if self.average_price.value().is_zero() {
            return Decimal::ZERO;
        }

        ((current_price.value() - self.average_price.value()) / self.average_price.value())
            * Decimal::from(100)
    }

    /// Adds to the position (buying more).
    pub fn add(&mut self, quantity: Quantity, price: &Price) {
        if quantity.is_zero() {
            return;
        }

        let current_cost = self.average_price.value() * self.quantity.value();
        let additional_cost = price.value() * quantity.value();
        let new_quantity = self.quantity.add(quantity);

        let new_average = if new_quantity.is_zero() {
            Decimal::ZERO
        } else {
            (current_cost + additional_cost) / new_quantity.value()
        };

        self.quantity = new_quantity;
        self.average_price = Price::new(new_average, self.currency.clone())
            .unwrap_or_else(|_| self.average_price.clone());
    }

    /// Reduces the position (selling).
    pub fn reduce(&mut self, quantity: Quantity) {
        self.quantity = self.quantity.subtract(quantity);

        if self.quantity.is_zero() {
            self.average_price = Price::new(Decimal::ZERO, self.currency.clone())
                .unwrap_or_else(|_| self.average_price.clone());
        }
    }

    /// Calculates the market value at a given price.
    #[must_use]
    pub fn market_value(&self, current_price: &Price) -> Money {
        Money::new(
            current_price.value() * self.quantity.value(),
            self.currency.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_position() -> Position {
        let price = Price::from_f64(100.0, Currency::usd()).unwrap();
        Position::new("AAPL", Quantity::from_lots(10), price)
    }

    #[test]
    fn test_position_creation() {
        let pos = create_test_position();
        assert_eq!(pos.instrument(), "AAPL");
        assert_eq!(pos.quantity().as_lots(), 10);
        assert_eq!(pos.average_price().value(), Decimal::from(100));
    }

    #[test]
    fn test_empty_position() {
        let pos = Position::empty("AAPL", Currency::usd());
        assert!(pos.is_empty());
        assert_eq!(pos.quantity().as_lots(), 0);
    }

    #[test]
    fn test_cost_basis() {
        let pos = create_test_position();
        let cost = pos.cost_basis();
        assert_eq!(cost.amount(), Decimal::from(1000)); // 10 * 100
    }

    #[test]
    fn test_unrealized_pnl_profit() {
        let pos = create_test_position();
        let current = Price::from_f64(110.0, Currency::usd()).unwrap();
        let pnl = pos.unrealized_pnl(&current);
        assert_eq!(pnl.amount(), Decimal::from(100)); // (110 - 100) * 10
        assert!(pnl.is_positive());
    }

    #[test]
    fn test_unrealized_pnl_loss() {
        let pos = create_test_position();
        let current = Price::from_f64(90.0, Currency::usd()).unwrap();
        let pnl = pos.unrealized_pnl(&current);
        assert_eq!(pnl.amount(), Decimal::from(-100)); // (90 - 100) * 10
    }

    #[test]
    fn test_add_to_position() {
        let mut pos = create_test_position();
        let new_price = Price::from_f64(120.0, Currency::usd()).unwrap();
        pos.add(Quantity::from_lots(10), &new_price);

        assert_eq!(pos.quantity().as_lots(), 20);
        // New average: (10 * 100 + 10 * 120) / 20 = 110
        assert_eq!(pos.average_price().value(), Decimal::from(110));
    }

    #[test]
    fn test_reduce_position() {
        let mut pos = create_test_position();
        pos.reduce(Quantity::from_lots(5));

        assert_eq!(pos.quantity().as_lots(), 5);
        // Average price remains the same
        assert_eq!(pos.average_price().value(), Decimal::from(100));
    }

    #[test]
    fn test_close_position() {
        let mut pos = create_test_position();
        pos.reduce(Quantity::from_lots(10));

        assert!(pos.is_empty());
    }

    #[test]
    fn test_market_value() {
        let pos = create_test_position();
        let current = Price::from_f64(105.0, Currency::usd()).unwrap();
        let value = pos.market_value(&current);
        assert_eq!(value.amount(), Decimal::from(1050)); // 10 * 105
    }
}
