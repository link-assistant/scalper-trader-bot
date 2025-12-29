//! Monetary types for precise financial calculations.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a currency code (e.g., USD, RUB, BTC).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Currency(String);

impl Currency {
    /// Creates a new currency from a code string.
    #[must_use]
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into().to_uppercase())
    }

    /// Returns the currency code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.0
    }

    /// US Dollar.
    #[must_use]
    pub fn usd() -> Self {
        Self::new("USD")
    }

    /// Russian Ruble.
    #[must_use]
    pub fn rub() -> Self {
        Self::new("RUB")
    }

    /// Bitcoin.
    #[must_use]
    pub fn btc() -> Self {
        Self::new("BTC")
    }

    /// Ethereum.
    #[must_use]
    pub fn eth() -> Self {
        Self::new("ETH")
    }

    /// Tether (USDT).
    #[must_use]
    pub fn usdt() -> Self {
        Self::new("USDT")
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a precise monetary amount.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    amount: Decimal,
    currency: Currency,
}

impl Money {
    /// Creates a new money value.
    #[must_use]
    pub fn new(amount: Decimal, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Creates a money value from a floating-point amount.
    #[must_use]
    pub fn from_f64(amount: f64, currency: Currency) -> Self {
        Self {
            amount: Decimal::try_from(amount).unwrap_or_default(),
            currency,
        }
    }

    /// Returns the amount.
    #[must_use]
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    /// Returns the currency.
    #[must_use]
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Creates a zero amount for the given currency.
    #[must_use]
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency,
        }
    }

    /// Checks if the amount is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    /// Checks if the amount is positive.
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.amount.is_sign_positive() && !self.amount.is_zero()
    }

    /// Adds another money value (must be same currency).
    pub fn add(&self, other: &Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.clone(),
                actual: other.currency.clone(),
            });
        }
        Ok(Self {
            amount: self.amount + other.amount,
            currency: self.currency.clone(),
        })
    }

    /// Subtracts another money value (must be same currency).
    pub fn subtract(&self, other: &Self) -> Result<Self, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.clone(),
                actual: other.currency.clone(),
            });
        }
        Ok(Self {
            amount: self.amount - other.amount,
            currency: self.currency.clone(),
        })
    }

    /// Multiplies by a quantity.
    #[must_use]
    pub fn multiply(&self, quantity: Quantity) -> Self {
        Self {
            amount: self.amount * quantity.value(),
            currency: self.currency.clone(),
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

/// Represents a price (always positive).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price {
    value: Decimal,
    currency: Currency,
}

impl Price {
    /// Creates a new price.
    pub fn new(value: Decimal, currency: Currency) -> Result<Self, MoneyError> {
        if value.is_sign_negative() {
            return Err(MoneyError::NegativePrice);
        }
        Ok(Self { value, currency })
    }

    /// Creates a price from a floating-point value.
    pub fn from_f64(value: f64, currency: Currency) -> Result<Self, MoneyError> {
        let decimal = Decimal::try_from(value).map_err(|_| MoneyError::InvalidDecimal)?;
        Self::new(decimal, currency)
    }

    /// Returns the price value.
    #[must_use]
    pub fn value(&self) -> Decimal {
        self.value
    }

    /// Returns the currency.
    #[must_use]
    pub fn currency(&self) -> &Currency {
        &self.currency
    }

    /// Adds a step to the price.
    #[must_use]
    pub fn add_step(&self, step: Decimal) -> Self {
        Self {
            value: self.value + step,
            currency: self.currency.clone(),
        }
    }

    /// Subtracts a step from the price.
    #[must_use]
    pub fn subtract_step(&self, step: Decimal) -> Self {
        let new_value = self.value - step;
        Self {
            value: if new_value.is_sign_negative() {
                Decimal::ZERO
            } else {
                new_value
            },
            currency: self.currency.clone(),
        }
    }

    /// Calculates the difference from another price.
    pub fn difference(&self, other: &Self) -> Result<Decimal, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.clone(),
                actual: other.currency.clone(),
            });
        }
        Ok(self.value - other.value)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}

/// Represents a quantity of an asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Quantity(Decimal);

impl Quantity {
    /// Creates a new quantity.
    pub fn new(value: Decimal) -> Result<Self, MoneyError> {
        if value.is_sign_negative() {
            return Err(MoneyError::NegativeQuantity);
        }
        Ok(Self(value))
    }

    /// Creates a quantity from a u64.
    #[must_use]
    pub fn from_lots(lots: u64) -> Self {
        Self(Decimal::from(lots))
    }

    /// Creates a quantity from a floating-point value.
    pub fn from_f64(value: f64) -> Result<Self, MoneyError> {
        if value < 0.0 {
            return Err(MoneyError::NegativeQuantity);
        }
        Ok(Self(
            Decimal::try_from(value).map_err(|_| MoneyError::InvalidDecimal)?,
        ))
    }

    /// Returns the quantity value.
    #[must_use]
    pub fn value(&self) -> Decimal {
        self.0
    }

    /// Returns the quantity as lots (whole number).
    #[must_use]
    pub fn as_lots(&self) -> u64 {
        self.0.to_string().parse().unwrap_or(0)
    }

    /// Checks if the quantity is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Creates a zero quantity.
    #[must_use]
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    /// Adds another quantity.
    #[must_use]
    pub fn add(&self, other: Self) -> Self {
        Self(self.0 + other.0)
    }

    /// Subtracts another quantity (returns zero if would be negative).
    #[must_use]
    pub fn subtract(&self, other: Self) -> Self {
        let result = self.0 - other.0;
        if result.is_sign_negative() {
            Self::zero()
        } else {
            Self(result)
        }
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error type for money operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum MoneyError {
    #[error("Currency mismatch: expected {expected}, got {actual}")]
    CurrencyMismatch {
        expected: Currency,
        actual: Currency,
    },

    #[error("Price cannot be negative")]
    NegativePrice,

    #[error("Quantity cannot be negative")]
    NegativeQuantity,

    #[error("Invalid decimal value")]
    InvalidDecimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod currency_tests {
        use super::*;

        #[test]
        fn test_currency_creation() {
            let usd = Currency::new("usd");
            assert_eq!(usd.code(), "USD");
        }

        #[test]
        fn test_predefined_currencies() {
            assert_eq!(Currency::usd().code(), "USD");
            assert_eq!(Currency::rub().code(), "RUB");
            assert_eq!(Currency::btc().code(), "BTC");
        }
    }

    mod money_tests {
        use super::*;

        #[test]
        fn test_money_creation() {
            let money = Money::from_f64(100.50, Currency::usd());
            assert_eq!(money.currency().code(), "USD");
        }

        #[test]
        fn test_money_add() {
            let a = Money::from_f64(100.0, Currency::usd());
            let b = Money::from_f64(50.0, Currency::usd());
            let result = a.add(&b).unwrap();
            assert_eq!(result.amount(), Decimal::from(150));
        }

        #[test]
        fn test_money_currency_mismatch() {
            let a = Money::from_f64(100.0, Currency::usd());
            let b = Money::from_f64(50.0, Currency::rub());
            assert!(a.add(&b).is_err());
        }
    }

    mod price_tests {
        use super::*;

        #[test]
        fn test_price_creation() {
            let price = Price::from_f64(100.50, Currency::usd()).unwrap();
            assert_eq!(price.currency().code(), "USD");
        }

        #[test]
        fn test_negative_price_rejected() {
            let result = Price::from_f64(-100.0, Currency::usd());
            assert!(result.is_err());
        }

        #[test]
        fn test_price_step_operations() {
            let price = Price::from_f64(100.0, Currency::usd()).unwrap();
            let higher = price.add_step(Decimal::from(1));
            assert_eq!(higher.value(), Decimal::from(101));
        }
    }

    mod quantity_tests {
        use super::*;

        #[test]
        fn test_quantity_from_lots() {
            let qty = Quantity::from_lots(10);
            assert_eq!(qty.as_lots(), 10);
        }

        #[test]
        fn test_negative_quantity_rejected() {
            let result = Quantity::from_f64(-1.0);
            assert!(result.is_err());
        }

        #[test]
        fn test_quantity_operations() {
            let a = Quantity::from_lots(10);
            let b = Quantity::from_lots(3);
            assert_eq!(a.add(b).as_lots(), 13);
            assert_eq!(a.subtract(b).as_lots(), 7);
        }
    }
}
