//! Simulator engine implementing the Exchange trait.

use super::market::{SimulatedMarket, SimulatedMarketConfig};
use crate::exchange::{
    Exchange, ExchangeError, ExchangeInfo, InstrumentInfo, MarketDataProvider, OrderExecutor,
};
use crate::types::{
    Currency, Money, Order, OrderBook, OrderId, OrderSide, OrderStatus, Position, Price, Quantity,
    Trade,
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// A simulated exchange for testing trading strategies.
#[derive(Debug)]
pub struct SimulatorEngine {
    info: ExchangeInfo,
    markets: HashMap<String, SimulatedMarket>,
    orders: HashMap<OrderId, Order>,
    positions: HashMap<String, Position>,
    balances: HashMap<Currency, Money>,
    trades: Vec<Trade>,
    connected: bool,
    commission_rate: Decimal,
}

impl SimulatorEngine {
    /// Creates a new simulator engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            info: ExchangeInfo {
                name: "Simulator".to_string(),
                id: "simulator".to_string(),
                connected: false,
                instruments: Vec::new(),
            },
            markets: HashMap::new(),
            orders: HashMap::new(),
            positions: HashMap::new(),
            balances: HashMap::new(),
            trades: Vec::new(),
            connected: false,
            commission_rate: Decimal::new(1, 4), // 0.01% commission
        }
    }

    /// Adds a market to the simulator.
    pub fn add_market(&mut self, config: SimulatedMarketConfig) {
        let symbol = config.symbol.clone();
        let currency = config.currency.clone();
        let price_step = config.price_step;
        let lot_size = config.lot_size;

        let market = SimulatedMarket::new(config);

        // Add instrument info
        self.info.instruments.push(InstrumentInfo {
            symbol: symbol.clone(),
            name: format!("{symbol} (Simulated)"),
            currency,
            min_quantity: Decimal::from(1),
            quantity_step: Decimal::from(1),
            price_step,
            lot_size,
            trading_enabled: true,
        });

        self.markets.insert(symbol, market);
    }

    /// Adds a balance to the simulator.
    pub fn add_balance(&mut self, currency: Currency, amount: Decimal) {
        self.balances
            .insert(currency.clone(), Money::new(amount, currency));
    }

    /// Sets the commission rate.
    pub fn set_commission_rate(&mut self, rate: Decimal) {
        self.commission_rate = rate;
    }

    /// Advances all markets by one tick.
    pub fn tick(&mut self) {
        for market in self.markets.values_mut() {
            market.tick();
        }
        self.process_pending_orders();
    }

    /// Advances all markets by multiple ticks.
    pub fn advance(&mut self, ticks: u64) {
        for _ in 0..ticks {
            self.tick();
        }
    }

    /// Gets a reference to a market.
    #[must_use]
    pub fn market(&self, symbol: &str) -> Option<&SimulatedMarket> {
        self.markets.get(symbol)
    }

    /// Gets a mutable reference to a market.
    pub fn market_mut(&mut self, symbol: &str) -> Option<&mut SimulatedMarket> {
        self.markets.get_mut(symbol)
    }

    /// Processes pending orders against current market prices.
    fn process_pending_orders(&mut self) {
        // Collect order info first to avoid borrow issues
        let order_info: Vec<(OrderId, String)> = self
            .orders
            .iter()
            .filter(|(_, o)| o.status().is_active())
            .map(|(id, o)| (id.clone(), o.instrument().to_string()))
            .collect();

        for (order_id, instrument) in order_info {
            // First, update pending orders to submitted
            if let Some(order) = self.orders.get_mut(&order_id) {
                if order.status() == OrderStatus::Pending {
                    order.submit();
                }
            }

            // Get market info (clone what we need to avoid borrow issues)
            let fill_info = self.calculate_fill_info_by_id(&order_id, &instrument);

            // Execute the fill if possible
            if let Some((price, quantity, commission_amount, side, instrument_str)) = fill_info {
                self.execute_fill_by_id(
                    &order_id,
                    &price,
                    quantity,
                    commission_amount,
                    side,
                    &instrument_str,
                );
            }
        }
    }

    /// Calculates fill information for an order without mutating state.
    fn calculate_fill_info_by_id(
        &self,
        order_id: &OrderId,
        instrument: &str,
    ) -> Option<(Price, Quantity, Decimal, OrderSide, String)> {
        let market = self.markets.get(instrument)?;
        let order = self.orders.get(order_id)?;

        let fill_price = match order.side() {
            OrderSide::Buy => {
                let ask = market.best_ask();
                match order.price() {
                    Some(limit) if limit.value() >= ask.value() => Some(ask),
                    None => Some(ask),
                    _ => None,
                }
            }
            OrderSide::Sell => {
                let bid = market.best_bid();
                match order.price() {
                    Some(limit) if limit.value() <= bid.value() => Some(bid),
                    None => Some(bid),
                    _ => None,
                }
            }
        };

        fill_price.and_then(|price| {
            let quantity = order.remaining_quantity();
            let value = price.value() * quantity.value();
            let commission_amount = value * self.commission_rate;
            let currency = price.currency().clone();
            let side = order.side();
            let instrument_str = order.instrument().to_string();

            let can_fill = match side {
                OrderSide::Buy => {
                    let required = value + commission_amount;
                    self.balances
                        .get(&currency)
                        .is_some_and(|b| b.amount() >= required)
                }
                OrderSide::Sell => self
                    .positions
                    .get(order.instrument())
                    .is_some_and(|p| p.quantity() >= quantity),
            };

            if can_fill {
                Some((price, quantity, commission_amount, side, instrument_str))
            } else {
                None
            }
        })
    }

    /// Executes a fill for an order by ID.
    fn execute_fill_by_id(
        &mut self,
        order_id: &OrderId,
        price: &Price,
        quantity: Quantity,
        commission_amount: Decimal,
        side: OrderSide,
        instrument: &str,
    ) {
        let currency = price.currency().clone();
        let value = price.value() * quantity.value();
        let commission = Money::new(commission_amount, currency.clone());

        // Update balances and positions
        match side {
            OrderSide::Buy => {
                if let Some(balance) = self.balances.get_mut(&currency) {
                    let total_cost = value + commission_amount;
                    *balance = Money::new(balance.amount() - total_cost, currency.clone());
                }

                let position = self
                    .positions
                    .entry(instrument.to_string())
                    .or_insert_with(|| Position::empty(instrument, currency.clone()));
                position.add(quantity, price);
            }
            OrderSide::Sell => {
                if let Some(balance) = self.balances.get_mut(&currency) {
                    let net_proceeds = value - commission_amount;
                    *balance = Money::new(balance.amount() + net_proceeds, currency.clone());
                }

                if let Some(position) = self.positions.get_mut(instrument) {
                    position.reduce(quantity);
                }
            }
        }

        // Record the trade
        let trade = Trade::new(
            order_id.clone(),
            instrument,
            side,
            quantity,
            price.clone(),
            commission,
        );
        self.trades.push(trade);

        // Update the order
        if let Some(order) = self.orders.get_mut(order_id) {
            order.fill(quantity);
        }
    }

    /// Returns the trade history.
    #[must_use]
    pub fn trade_history(&self) -> &[Trade] {
        &self.trades
    }
}

impl Default for SimulatorEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MarketDataProvider for SimulatorEngine {
    async fn connect(&mut self) -> Result<(), ExchangeError> {
        self.connected = true;
        self.info.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), ExchangeError> {
        self.connected = false;
        self.info.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn get_order_book(&self, instrument: &str) -> Result<OrderBook, ExchangeError> {
        let market = self
            .markets
            .get(instrument)
            .ok_or_else(|| ExchangeError::InstrumentNotFound(instrument.to_string()))?;

        let bids = market.generate_bids();
        let asks = market.generate_asks();

        Ok(OrderBook::with_levels(instrument, bids, asks))
    }

    async fn get_instrument_info(&self, instrument: &str) -> Result<InstrumentInfo, ExchangeError> {
        self.info
            .instruments
            .iter()
            .find(|i| i.symbol == instrument)
            .cloned()
            .ok_or_else(|| ExchangeError::InstrumentNotFound(instrument.to_string()))
    }

    async fn list_instruments(&self) -> Result<Vec<InstrumentInfo>, ExchangeError> {
        Ok(self.info.instruments.clone())
    }
}

#[async_trait]
impl OrderExecutor for SimulatorEngine {
    async fn submit_order(&mut self, mut order: Order) -> Result<Order, ExchangeError> {
        // Validate the instrument exists
        if !self.markets.contains_key(order.instrument()) {
            return Err(ExchangeError::InstrumentNotFound(
                order.instrument().to_string(),
            ));
        }

        // Validate resources for the order
        if let Some(market) = self.markets.get(order.instrument()) {
            let currency = market.config().currency.clone();

            match order.side() {
                OrderSide::Buy => {
                    let price = order
                        .price()
                        .map(|p| p.value())
                        .unwrap_or(market.best_ask().value());
                    let required = price * order.quantity().value();
                    let available = self
                        .balances
                        .get(&currency)
                        .map(Money::amount)
                        .unwrap_or_default();

                    if available < required {
                        return Err(ExchangeError::InsufficientFunds {
                            required: required.to_string(),
                            available: available.to_string(),
                        });
                    }
                }
                OrderSide::Sell => {
                    let required = order.quantity();
                    let available = self
                        .positions
                        .get(order.instrument())
                        .map(Position::quantity)
                        .unwrap_or(Quantity::zero());

                    if available < required {
                        return Err(ExchangeError::InsufficientPosition {
                            required: required.to_string(),
                            available: available.to_string(),
                        });
                    }
                }
            }
        }

        order.submit();
        self.orders.insert(order.id().clone(), order.clone());

        // Try to fill immediately
        self.process_pending_orders();

        // Return the updated order
        Ok(self.orders.get(order.id()).cloned().unwrap_or(order))
    }

    async fn cancel_order(&mut self, order_id: &OrderId) -> Result<(), ExchangeError> {
        let order = self
            .orders
            .get_mut(order_id)
            .ok_or_else(|| ExchangeError::OrderNotFound(order_id.clone()))?;

        if order.status().is_terminal() {
            return Err(ExchangeError::OrderRejected(
                "Cannot cancel a completed order".to_string(),
            ));
        }

        order.cancel();
        Ok(())
    }

    async fn get_order(&self, order_id: &OrderId) -> Result<Order, ExchangeError> {
        self.orders
            .get(order_id)
            .cloned()
            .ok_or_else(|| ExchangeError::OrderNotFound(order_id.clone()))
    }

    async fn list_active_orders(&self) -> Result<Vec<Order>, ExchangeError> {
        Ok(self
            .orders
            .values()
            .filter(|o| o.status().is_active())
            .cloned()
            .collect())
    }

    async fn list_orders_for_instrument(
        &self,
        instrument: &str,
    ) -> Result<Vec<Order>, ExchangeError> {
        Ok(self
            .orders
            .values()
            .filter(|o| o.instrument() == instrument)
            .cloned()
            .collect())
    }

    async fn get_positions(&self) -> Result<HashMap<String, Position>, ExchangeError> {
        Ok(self.positions.clone())
    }

    async fn get_position(&self, instrument: &str) -> Result<Position, ExchangeError> {
        self.positions
            .get(instrument)
            .cloned()
            .ok_or_else(|| ExchangeError::InstrumentNotFound(instrument.to_string()))
    }

    async fn get_balance(&self, currency: &Currency) -> Result<Money, ExchangeError> {
        self.balances
            .get(currency)
            .cloned()
            .ok_or_else(|| ExchangeError::InstrumentNotFound(currency.code().to_string()))
    }

    async fn get_balances(&self) -> Result<HashMap<Currency, Money>, ExchangeError> {
        Ok(self.balances.clone())
    }

    async fn get_trades(&self, limit: usize) -> Result<Vec<Trade>, ExchangeError> {
        let start = self.trades.len().saturating_sub(limit);
        Ok(self.trades[start..].to_vec())
    }

    async fn get_trades_for_instrument(
        &self,
        instrument: &str,
        limit: usize,
    ) -> Result<Vec<Trade>, ExchangeError> {
        let trades: Vec<Trade> = self
            .trades
            .iter()
            .filter(|t| t.instrument() == instrument)
            .cloned()
            .collect();
        let start = trades.len().saturating_sub(limit);
        Ok(trades[start..].to_vec())
    }
}

#[async_trait]
impl Exchange for SimulatorEngine {
    fn info(&self) -> &ExchangeInfo {
        &self.info
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_engine() -> SimulatorEngine {
        let mut engine = SimulatorEngine::new();

        engine.add_market(SimulatedMarketConfig {
            symbol: "TEST".to_string(),
            currency: Currency::usd(),
            initial_price: Decimal::from(100),
            ..Default::default()
        });

        engine.add_balance(Currency::usd(), Decimal::from(10000));
        engine.connect().await.unwrap();

        engine
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let engine = create_test_engine().await;
        assert!(engine.is_connected());
        assert_eq!(engine.info().name, "Simulator");
    }

    #[tokio::test]
    async fn test_get_order_book() {
        let engine = create_test_engine().await;
        let book = engine.get_order_book("TEST").await.unwrap();

        assert_eq!(book.instrument(), "TEST");
        assert!(!book.bids().is_empty());
        assert!(!book.asks().is_empty());
    }

    #[tokio::test]
    async fn test_submit_market_buy_order() {
        let mut engine = create_test_engine().await;

        let order = Order::market("TEST", OrderSide::Buy, Quantity::from_lots(10));
        let submitted = engine.submit_order(order).await.unwrap();

        assert_eq!(submitted.status(), OrderStatus::Filled);
        assert_eq!(submitted.filled_quantity().as_lots(), 10);

        // Check position was created
        let position = engine.get_position("TEST").await.unwrap();
        assert_eq!(position.quantity().as_lots(), 10);

        // Check balance was reduced
        let balance = engine.get_balance(&Currency::usd()).await.unwrap();
        assert!(balance.amount() < Decimal::from(10000));
    }

    #[tokio::test]
    async fn test_submit_limit_order() {
        let mut engine = create_test_engine().await;

        // Place a limit buy order below market price
        let low_price = Price::from_f64(90.0, Currency::usd()).unwrap();
        let order = Order::limit("TEST", OrderSide::Buy, Quantity::from_lots(10), low_price);
        let submitted = engine.submit_order(order).await.unwrap();

        // Should still be pending (not filled)
        assert_eq!(submitted.status(), OrderStatus::Submitted);

        // Move the market price down to trigger the order
        engine
            .market_mut("TEST")
            .unwrap()
            .set_price(Decimal::from(89));
        engine.tick();

        // Now check if order was filled
        let updated = engine.get_order(submitted.id()).await.unwrap();
        assert_eq!(updated.status(), OrderStatus::Filled);
    }

    #[tokio::test]
    async fn test_sell_order() {
        let mut engine = create_test_engine().await;

        // First buy some
        let buy = Order::market("TEST", OrderSide::Buy, Quantity::from_lots(10));
        engine.submit_order(buy).await.unwrap();

        // Then sell
        let sell = Order::market("TEST", OrderSide::Sell, Quantity::from_lots(5));
        let sold = engine.submit_order(sell).await.unwrap();

        assert_eq!(sold.status(), OrderStatus::Filled);

        // Check position was reduced
        let position = engine.get_position("TEST").await.unwrap();
        assert_eq!(position.quantity().as_lots(), 5);
    }

    #[tokio::test]
    async fn test_insufficient_funds() {
        let mut engine = create_test_engine().await;

        // Try to buy more than we can afford
        let order = Order::market("TEST", OrderSide::Buy, Quantity::from_lots(1000));
        let result = engine.submit_order(order).await;

        assert!(matches!(
            result,
            Err(ExchangeError::InsufficientFunds { .. })
        ));
    }

    #[tokio::test]
    async fn test_insufficient_position() {
        let mut engine = create_test_engine().await;

        // Try to sell without a position
        let order = Order::market("TEST", OrderSide::Sell, Quantity::from_lots(10));
        let result = engine.submit_order(order).await;

        assert!(matches!(
            result,
            Err(ExchangeError::InsufficientPosition { .. })
        ));
    }

    #[tokio::test]
    async fn test_cancel_order() {
        let mut engine = create_test_engine().await;

        // Place a limit order that won't fill
        let low_price = Price::from_f64(50.0, Currency::usd()).unwrap();
        let order = Order::limit("TEST", OrderSide::Buy, Quantity::from_lots(10), low_price);
        let submitted = engine.submit_order(order).await.unwrap();

        // Cancel it
        engine.cancel_order(submitted.id()).await.unwrap();

        // Verify it's cancelled
        let cancelled = engine.get_order(submitted.id()).await.unwrap();
        assert_eq!(cancelled.status(), OrderStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_trade_history() {
        let mut engine = create_test_engine().await;

        let order = Order::market("TEST", OrderSide::Buy, Quantity::from_lots(10));
        engine.submit_order(order).await.unwrap();

        let trades = engine.get_trades(10).await.unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity().as_lots(), 10);
    }
}
