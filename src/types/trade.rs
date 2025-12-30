//! Trade types for recording executed transactions.

use super::{Currency, Money, OrderId, OrderSide, Price, Quantity};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a trade.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradeId(String);

impl TradeId {
    /// Creates a new trade ID from a string.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generates a new random trade ID.
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

impl fmt::Display for TradeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents an executed trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    id: TradeId,
    order_id: OrderId,
    instrument: String,
    side: OrderSide,
    quantity: Quantity,
    price: Price,
    commission: Money,
    executed_at: DateTime<Utc>,
}

impl Trade {
    /// Creates a new trade.
    #[must_use]
    pub fn new(
        order_id: OrderId,
        instrument: impl Into<String>,
        side: OrderSide,
        quantity: Quantity,
        price: Price,
        commission: Money,
    ) -> Self {
        Self {
            id: TradeId::generate(),
            order_id,
            instrument: instrument.into(),
            side,
            quantity,
            price,
            commission,
            executed_at: Utc::now(),
        }
    }

    /// Creates a trade with a specific timestamp (for testing/simulation).
    #[must_use]
    pub fn with_timestamp(
        order_id: OrderId,
        instrument: impl Into<String>,
        side: OrderSide,
        quantity: Quantity,
        price: Price,
        commission: Money,
        executed_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: TradeId::generate(),
            order_id,
            instrument: instrument.into(),
            side,
            quantity,
            price,
            commission,
            executed_at,
        }
    }

    /// Returns the trade ID.
    #[must_use]
    pub fn id(&self) -> &TradeId {
        &self.id
    }

    /// Returns the order ID this trade belongs to.
    #[must_use]
    pub fn order_id(&self) -> &OrderId {
        &self.order_id
    }

    /// Returns the instrument.
    #[must_use]
    pub fn instrument(&self) -> &str {
        &self.instrument
    }

    /// Returns the trade side.
    #[must_use]
    pub fn side(&self) -> OrderSide {
        self.side
    }

    /// Returns the quantity traded.
    #[must_use]
    pub fn quantity(&self) -> Quantity {
        self.quantity
    }

    /// Returns the execution price.
    #[must_use]
    pub fn price(&self) -> &Price {
        &self.price
    }

    /// Returns the commission paid.
    #[must_use]
    pub fn commission(&self) -> &Money {
        &self.commission
    }

    /// Returns the execution timestamp.
    #[must_use]
    pub fn executed_at(&self) -> DateTime<Utc> {
        self.executed_at
    }

    /// Calculates the gross value of the trade (price * quantity).
    #[must_use]
    pub fn gross_value(&self) -> Money {
        Money::new(
            self.price.value() * self.quantity.value(),
            self.price.currency().clone(),
        )
    }

    /// Calculates the net value of the trade (gross value - commission for buys, + commission for sells).
    pub fn net_value(&self) -> Result<Money, super::money::MoneyError> {
        let gross = self.gross_value();
        match self.side {
            OrderSide::Buy => gross.add(&self.commission),
            OrderSide::Sell => gross.subtract(&self.commission),
        }
    }

    /// Checks if this is a buy trade.
    #[must_use]
    pub fn is_buy(&self) -> bool {
        self.side == OrderSide::Buy
    }

    /// Checks if this is a sell trade.
    #[must_use]
    pub fn is_sell(&self) -> bool {
        self.side == OrderSide::Sell
    }
}

/// A collection of trades for calculating statistics.
#[derive(Debug, Clone, Default)]
pub struct TradeHistory {
    trades: Vec<Trade>,
}

impl TradeHistory {
    /// Creates a new empty trade history.
    #[must_use]
    pub fn new() -> Self {
        Self { trades: Vec::new() }
    }

    /// Adds a trade to the history.
    pub fn add(&mut self, trade: Trade) {
        self.trades.push(trade);
    }

    /// Returns all trades.
    #[must_use]
    pub fn trades(&self) -> &[Trade] {
        &self.trades
    }

    /// Returns the number of trades.
    #[must_use]
    pub fn count(&self) -> usize {
        self.trades.len()
    }

    /// Returns trades for a specific instrument.
    #[must_use]
    pub fn for_instrument(&self, instrument: &str) -> Vec<&Trade> {
        self.trades
            .iter()
            .filter(|t| t.instrument() == instrument)
            .collect()
    }

    /// Returns all buy trades.
    #[must_use]
    pub fn buys(&self) -> Vec<&Trade> {
        self.trades.iter().filter(|t| t.is_buy()).collect()
    }

    /// Returns all sell trades.
    #[must_use]
    pub fn sells(&self) -> Vec<&Trade> {
        self.trades.iter().filter(|t| t.is_sell()).collect()
    }

    /// Calculates total commission paid.
    #[must_use]
    pub fn total_commission(&self, currency: &Currency) -> Money {
        self.trades
            .iter()
            .filter(|t| t.commission.currency() == currency)
            .fold(Money::zero(currency.clone()), |acc, t| {
                acc.add(t.commission()).unwrap_or(acc)
            })
    }

    /// Calculates the realized profit/loss from completed round trips.
    /// A round trip is a buy followed by a sell (or vice versa).
    #[must_use]
    pub fn realized_pnl(&self, instrument: &str, currency: &Currency) -> Decimal {
        let trades = self.for_instrument(instrument);
        let mut total_buy_cost = Decimal::ZERO;
        let mut total_buy_qty = Decimal::ZERO;
        let mut total_sell_revenue = Decimal::ZERO;
        let mut total_sell_qty = Decimal::ZERO;
        let mut total_commission = Decimal::ZERO;

        for trade in trades {
            if trade.price().currency() != currency {
                continue;
            }

            let value = trade.price().value() * trade.quantity().value();
            total_commission += trade.commission().amount();

            match trade.side() {
                OrderSide::Buy => {
                    total_buy_cost += value;
                    total_buy_qty += trade.quantity().value();
                }
                OrderSide::Sell => {
                    total_sell_revenue += value;
                    total_sell_qty += trade.quantity().value();
                }
            }
        }

        // Realized P&L is based on matched quantities
        let matched_qty = total_buy_qty.min(total_sell_qty);
        if matched_qty.is_zero() {
            return Decimal::ZERO;
        }

        let avg_buy_price = if total_buy_qty.is_zero() {
            Decimal::ZERO
        } else {
            total_buy_cost / total_buy_qty
        };

        let avg_sell_price = if total_sell_qty.is_zero() {
            Decimal::ZERO
        } else {
            total_sell_revenue / total_sell_qty
        };

        (avg_sell_price - avg_buy_price) * matched_qty - total_commission
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_trade(side: OrderSide, price: f64, quantity: u64) -> Trade {
        Trade::new(
            OrderId::generate(),
            "AAPL",
            side,
            Quantity::from_lots(quantity),
            Price::from_f64(price, Currency::usd()).unwrap(),
            Money::from_f64(1.0, Currency::usd()),
        )
    }

    mod trade_tests {
        use super::*;

        #[test]
        fn test_trade_creation() {
            let trade = create_test_trade(OrderSide::Buy, 100.0, 10);
            assert_eq!(trade.instrument(), "AAPL");
            assert_eq!(trade.side(), OrderSide::Buy);
            assert_eq!(trade.quantity().as_lots(), 10);
        }

        #[test]
        fn test_gross_value() {
            let trade = create_test_trade(OrderSide::Buy, 100.0, 10);
            let gross = trade.gross_value();
            assert_eq!(gross.amount(), Decimal::from(1000));
        }

        #[test]
        fn test_net_value_buy() {
            let trade = create_test_trade(OrderSide::Buy, 100.0, 10);
            let net = trade.net_value().unwrap();
            // Buy: gross + commission = 1000 + 1 = 1001
            assert_eq!(net.amount(), Decimal::from(1001));
        }

        #[test]
        fn test_net_value_sell() {
            let trade = create_test_trade(OrderSide::Sell, 100.0, 10);
            let net = trade.net_value().unwrap();
            // Sell: gross - commission = 1000 - 1 = 999
            assert_eq!(net.amount(), Decimal::from(999));
        }
    }

    mod trade_history_tests {
        use super::*;

        #[test]
        fn test_trade_history() {
            let mut history = TradeHistory::new();
            history.add(create_test_trade(OrderSide::Buy, 100.0, 10));
            history.add(create_test_trade(OrderSide::Sell, 110.0, 10));

            assert_eq!(history.count(), 2);
            assert_eq!(history.buys().len(), 1);
            assert_eq!(history.sells().len(), 1);
        }

        #[test]
        fn test_total_commission() {
            let mut history = TradeHistory::new();
            history.add(create_test_trade(OrderSide::Buy, 100.0, 10));
            history.add(create_test_trade(OrderSide::Sell, 110.0, 10));

            let commission = history.total_commission(&Currency::usd());
            assert_eq!(commission.amount(), Decimal::from(2)); // 1 + 1
        }

        #[test]
        fn test_realized_pnl() {
            let mut history = TradeHistory::new();
            history.add(create_test_trade(OrderSide::Buy, 100.0, 10));
            history.add(create_test_trade(OrderSide::Sell, 110.0, 10));

            let pnl = history.realized_pnl("AAPL", &Currency::usd());
            // (110 - 100) * 10 - 2 (commission) = 98
            assert_eq!(pnl, Decimal::from(98));
        }

        #[test]
        fn test_realized_pnl_loss() {
            let mut history = TradeHistory::new();
            history.add(create_test_trade(OrderSide::Buy, 100.0, 10));
            history.add(create_test_trade(OrderSide::Sell, 95.0, 10));

            let pnl = history.realized_pnl("AAPL", &Currency::usd());
            // (95 - 100) * 10 - 2 (commission) = -52
            assert_eq!(pnl, Decimal::from(-52));
        }
    }
}
