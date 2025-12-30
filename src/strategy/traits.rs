//! Strategy traits and types.

use crate::types::{Order, OrderBook, OrderId, Position, Price, Quantity};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// An action the strategy wants to take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyAction {
    /// Place a new buy order.
    Buy {
        quantity: Quantity,
        price: Option<Price>,
    },

    /// Place a new sell order.
    Sell {
        quantity: Quantity,
        price: Option<Price>,
    },

    /// Cancel an existing order.
    Cancel { order_id: OrderId },

    /// Modify an existing order's price.
    ModifyPrice { order_id: OrderId, new_price: Price },

    /// Do nothing.
    Hold,
}

impl StrategyAction {
    /// Creates a market buy action.
    #[must_use]
    pub fn market_buy(quantity: Quantity) -> Self {
        Self::Buy {
            quantity,
            price: None,
        }
    }

    /// Creates a limit buy action.
    #[must_use]
    pub fn limit_buy(quantity: Quantity, price: Price) -> Self {
        Self::Buy {
            quantity,
            price: Some(price),
        }
    }

    /// Creates a market sell action.
    #[must_use]
    pub fn market_sell(quantity: Quantity) -> Self {
        Self::Sell {
            quantity,
            price: None,
        }
    }

    /// Creates a limit sell action.
    #[must_use]
    pub fn limit_sell(quantity: Quantity, price: Price) -> Self {
        Self::Sell {
            quantity,
            price: Some(price),
        }
    }

    /// Creates a cancel action.
    #[must_use]
    pub fn cancel(order_id: OrderId) -> Self {
        Self::Cancel { order_id }
    }

    /// Creates a hold action.
    #[must_use]
    pub fn hold() -> Self {
        Self::Hold
    }

    /// Checks if this is a hold action.
    #[must_use]
    pub fn is_hold(&self) -> bool {
        matches!(self, Self::Hold)
    }
}

/// The decision made by a strategy, containing multiple possible actions.
#[derive(Debug, Clone, Default)]
pub struct StrategyDecision {
    actions: Vec<StrategyAction>,
    reason: Option<String>,
}

impl StrategyDecision {
    /// Creates a new empty decision.
    #[must_use]
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            reason: None,
        }
    }

    /// Creates a decision with a single action.
    #[must_use]
    pub fn single(action: StrategyAction) -> Self {
        Self {
            actions: vec![action],
            reason: None,
        }
    }

    /// Creates a hold decision.
    #[must_use]
    pub fn hold() -> Self {
        Self::single(StrategyAction::Hold)
    }

    /// Creates a hold decision with a reason.
    #[must_use]
    pub fn hold_with_reason(reason: impl Into<String>) -> Self {
        Self {
            actions: vec![StrategyAction::Hold],
            reason: Some(reason.into()),
        }
    }

    /// Adds an action to the decision.
    pub fn add_action(&mut self, action: StrategyAction) {
        self.actions.push(action);
    }

    /// Sets the reason for the decision.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Returns the actions.
    #[must_use]
    pub fn actions(&self) -> &[StrategyAction] {
        &self.actions
    }

    /// Returns the reason for the decision.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    /// Checks if this is a hold decision (no actions or only hold actions).
    #[must_use]
    pub fn is_hold(&self) -> bool {
        self.actions.is_empty() || self.actions.iter().all(StrategyAction::is_hold)
    }
}

/// Market state provided to the strategy for decision making.
#[derive(Debug, Clone)]
pub struct MarketState {
    /// The instrument being traded.
    pub instrument: String,
    /// Current order book.
    pub order_book: OrderBook,
    /// Current position.
    pub position: Option<Position>,
    /// Active orders.
    pub active_orders: Vec<Order>,
    /// Available cash for trading.
    pub available_cash: rust_decimal::Decimal,
    /// Current timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// A trading strategy that makes decisions based on market state.
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Returns the name of the strategy.
    fn name(&self) -> &str;

    /// Makes a decision based on the current market state.
    async fn decide(&self, state: &MarketState) -> StrategyDecision;

    /// Called when an order is filled.
    async fn on_order_filled(&mut self, order: &Order);

    /// Called when an order is cancelled.
    async fn on_order_cancelled(&mut self, order: &Order);

    /// Resets the strategy state.
    async fn reset(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_action_creation() {
        let buy = StrategyAction::market_buy(Quantity::from_lots(10));
        assert!(matches!(buy, StrategyAction::Buy { price: None, .. }));

        let sell = StrategyAction::market_sell(Quantity::from_lots(5));
        assert!(matches!(sell, StrategyAction::Sell { price: None, .. }));
    }

    #[test]
    fn test_strategy_decision() {
        let mut decision = StrategyDecision::new();
        decision.add_action(StrategyAction::market_buy(Quantity::from_lots(10)));
        decision.add_action(StrategyAction::market_sell(Quantity::from_lots(5)));

        assert_eq!(decision.actions().len(), 2);
        assert!(!decision.is_hold());
    }

    #[test]
    fn test_hold_decision() {
        let decision = StrategyDecision::hold_with_reason("Market closed");
        assert!(decision.is_hold());
        assert_eq!(decision.reason(), Some("Market closed"));
    }
}
