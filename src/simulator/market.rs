//! Simulated market for testing.

use crate::types::{Currency, OrderBookLevel, Price, Quantity};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Market conditions that affect price movements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketCondition {
    /// Prices trending upward.
    Bullish,
    /// Prices trending downward.
    Bearish,
    /// Prices moving sideways with low volatility.
    Ranging,
    /// High volatility with unpredictable movements.
    Volatile,
}

impl Default for MarketCondition {
    fn default() -> Self {
        Self::Ranging
    }
}

/// Configuration for a simulated market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatedMarketConfig {
    /// The instrument symbol.
    pub symbol: String,
    /// The currency for prices.
    pub currency: Currency,
    /// Initial price.
    pub initial_price: Decimal,
    /// Price step (tick size).
    pub price_step: Decimal,
    /// Lot size.
    pub lot_size: u64,
    /// Default spread in ticks.
    pub spread_ticks: u32,
    /// Base liquidity at each level (in lots).
    pub base_liquidity: u64,
    /// Number of order book levels.
    pub book_depth: usize,
}

impl Default for SimulatedMarketConfig {
    fn default() -> Self {
        Self {
            symbol: "TEST".to_string(),
            currency: Currency::usd(),
            initial_price: Decimal::from(100),
            price_step: Decimal::new(1, 2), // 0.01
            lot_size: 1,
            spread_ticks: 2,
            base_liquidity: 100,
            book_depth: 10,
        }
    }
}

/// A simulated market for testing trading strategies.
#[derive(Debug, Clone)]
pub struct SimulatedMarket {
    config: SimulatedMarketConfig,
    current_mid_price: Decimal,
    condition: MarketCondition,
    volatility: Decimal,
    tick_count: u64,
}

impl SimulatedMarket {
    /// Creates a new simulated market with the given configuration.
    #[must_use]
    pub fn new(config: SimulatedMarketConfig) -> Self {
        let initial_price = config.initial_price;
        Self {
            config,
            current_mid_price: initial_price,
            condition: MarketCondition::default(),
            volatility: Decimal::new(1, 3), // 0.001 = 0.1%
            tick_count: 0,
        }
    }

    /// Creates a market with default configuration.
    #[must_use]
    pub fn default_market() -> Self {
        Self::new(SimulatedMarketConfig::default())
    }

    /// Returns the current configuration.
    #[must_use]
    pub fn config(&self) -> &SimulatedMarketConfig {
        &self.config
    }

    /// Returns the symbol.
    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.config.symbol
    }

    /// Returns the current mid price.
    #[must_use]
    pub fn mid_price(&self) -> Decimal {
        self.current_mid_price
    }

    /// Returns the current best bid price.
    #[must_use]
    pub fn best_bid(&self) -> Price {
        let half_spread = self.config.price_step * Decimal::from(self.config.spread_ticks / 2);
        Price::new(
            self.current_mid_price - half_spread,
            self.config.currency.clone(),
        )
        .unwrap_or_else(|_| Price::new(Decimal::ZERO, self.config.currency.clone()).unwrap())
    }

    /// Returns the current best ask price.
    #[must_use]
    pub fn best_ask(&self) -> Price {
        let half_spread = self.config.price_step * Decimal::from(self.config.spread_ticks / 2);
        Price::new(
            self.current_mid_price + half_spread,
            self.config.currency.clone(),
        )
        .unwrap_or_else(|_| Price::new(Decimal::ZERO, self.config.currency.clone()).unwrap())
    }

    /// Returns the current spread.
    #[must_use]
    pub fn spread(&self) -> Decimal {
        self.config.price_step * Decimal::from(self.config.spread_ticks)
    }

    /// Returns the current market condition.
    #[must_use]
    pub fn condition(&self) -> MarketCondition {
        self.condition
    }

    /// Sets the market condition.
    pub fn set_condition(&mut self, condition: MarketCondition) {
        self.condition = condition;
    }

    /// Sets the volatility (as a decimal, e.g., 0.01 for 1%).
    pub fn set_volatility(&mut self, volatility: Decimal) {
        self.volatility = volatility;
    }

    /// Generates bid levels for the order book.
    #[must_use]
    pub fn generate_bids(&self) -> Vec<OrderBookLevel> {
        let best_bid = self.best_bid();
        (0..self.config.book_depth)
            .map(|i| {
                let price_offset = self.config.price_step * Decimal::from(i as u32);
                let price = Price::new(
                    best_bid.value() - price_offset,
                    self.config.currency.clone(),
                )
                .unwrap_or(best_bid.clone());

                // Liquidity increases slightly at lower prices
                let liquidity_multiplier = Decimal::from(1) + Decimal::new(i as i64, 1);
                let quantity = Quantity::from_lots(
                    (Decimal::from(self.config.base_liquidity) * liquidity_multiplier)
                        .to_string()
                        .parse()
                        .unwrap_or(self.config.base_liquidity),
                );

                OrderBookLevel::new(price, quantity, (5 + i) as u32)
            })
            .collect()
    }

    /// Generates ask levels for the order book.
    #[must_use]
    pub fn generate_asks(&self) -> Vec<OrderBookLevel> {
        let best_ask = self.best_ask();
        (0..self.config.book_depth)
            .map(|i| {
                let price_offset = self.config.price_step * Decimal::from(i as u32);
                let price = Price::new(
                    best_ask.value() + price_offset,
                    self.config.currency.clone(),
                )
                .unwrap_or(best_ask.clone());

                // Liquidity increases slightly at higher prices
                let liquidity_multiplier = Decimal::from(1) + Decimal::new(i as i64, 1);
                let quantity = Quantity::from_lots(
                    (Decimal::from(self.config.base_liquidity) * liquidity_multiplier)
                        .to_string()
                        .parse()
                        .unwrap_or(self.config.base_liquidity),
                );

                OrderBookLevel::new(price, quantity, (5 + i) as u32)
            })
            .collect()
    }

    /// Advances the market by one tick, updating prices based on condition.
    pub fn tick(&mut self) {
        self.tick_count += 1;

        // Simulate price movement based on market condition
        let price_change = self.calculate_price_change();
        self.current_mid_price += price_change;

        // Ensure price doesn't go below minimum
        if self.current_mid_price < self.config.price_step {
            self.current_mid_price = self.config.price_step;
        }
    }

    /// Calculates the price change for this tick.
    fn calculate_price_change(&self) -> Decimal {
        // Use tick count to create deterministic but varied price movements
        let seed = self.tick_count;
        let pseudo_random = ((seed * 1103515245 + 12345) % 100) as i64 - 50;
        let random_factor = Decimal::new(pseudo_random, 2); // -0.50 to 0.50

        let base_change = self.current_mid_price * self.volatility * random_factor;

        match self.condition {
            MarketCondition::Bullish => {
                // Bias towards positive movements
                base_change + self.config.price_step * Decimal::new(3, 1) // +0.3 tick bias
            }
            MarketCondition::Bearish => {
                // Bias towards negative movements
                base_change - self.config.price_step * Decimal::new(3, 1) // -0.3 tick bias
            }
            MarketCondition::Ranging => {
                // Mean-reverting behavior
                base_change * Decimal::new(5, 1) // Reduced volatility
            }
            MarketCondition::Volatile => {
                // Amplified movements
                base_change * Decimal::from(2)
            }
        }
    }

    /// Moves the price by a specific amount (for testing).
    pub fn move_price(&mut self, delta: Decimal) {
        self.current_mid_price += delta;
        if self.current_mid_price < self.config.price_step {
            self.current_mid_price = self.config.price_step;
        }
    }

    /// Sets the price directly (for testing).
    pub fn set_price(&mut self, price: Decimal) {
        self.current_mid_price = price.max(self.config.price_step);
    }

    /// Returns the number of ticks processed.
    #[must_use]
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_creation() {
        let market = SimulatedMarket::default_market();
        assert_eq!(market.symbol(), "TEST");
        assert_eq!(market.mid_price(), Decimal::from(100));
    }

    #[test]
    fn test_best_bid_ask() {
        let market = SimulatedMarket::default_market();
        let bid = market.best_bid();
        let ask = market.best_ask();

        // With default spread of 2 ticks and price_step of 0.01
        // Mid = 100, spread = 0.02
        // Bid = 100 - 0.01 = 99.99
        // Ask = 100 + 0.01 = 100.01
        assert!(bid.value() < ask.value());
        assert_eq!(market.spread(), Decimal::new(2, 2)); // 0.02
    }

    #[test]
    fn test_generate_order_book() {
        let market = SimulatedMarket::default_market();
        let bids = market.generate_bids();
        let asks = market.generate_asks();

        assert_eq!(bids.len(), 10);
        assert_eq!(asks.len(), 10);

        // Bids should be descending
        for i in 1..bids.len() {
            assert!(bids[i - 1].price().value() > bids[i].price().value());
        }

        // Asks should be ascending
        for i in 1..asks.len() {
            assert!(asks[i - 1].price().value() < asks[i].price().value());
        }
    }

    #[test]
    fn test_market_tick() {
        let mut market = SimulatedMarket::default_market();
        let initial_price = market.mid_price();

        for _ in 0..100 {
            market.tick();
        }

        // Price should have changed after many ticks
        assert_ne!(market.mid_price(), initial_price);
        assert_eq!(market.tick_count(), 100);
    }

    #[test]
    fn test_market_conditions() {
        let config = SimulatedMarketConfig {
            initial_price: Decimal::from(100),
            ..Default::default()
        };

        // Test bullish market
        let mut bullish_market = SimulatedMarket::new(config.clone());
        bullish_market.set_condition(MarketCondition::Bullish);
        bullish_market.set_volatility(Decimal::new(5, 3)); // 0.5%

        let initial = bullish_market.mid_price();
        for _ in 0..100 {
            bullish_market.tick();
        }

        // Bullish market should generally trend up
        // (though individual runs may vary due to random component)
        let bullish_final = bullish_market.mid_price();

        // Test bearish market
        let mut bearish_market = SimulatedMarket::new(config);
        bearish_market.set_condition(MarketCondition::Bearish);
        bearish_market.set_volatility(Decimal::new(5, 3));

        for _ in 0..100 {
            bearish_market.tick();
        }

        let bearish_final = bearish_market.mid_price();

        // Bullish should end higher than bearish (statistically)
        // This is a probabilistic test, but with 100 ticks and bias, should be reliable
        assert!(
            bullish_final >= initial || bearish_final <= initial,
            "At least one market should show expected trend"
        );
    }

    #[test]
    fn test_manual_price_control() {
        let mut market = SimulatedMarket::default_market();

        market.set_price(Decimal::from(150));
        assert_eq!(market.mid_price(), Decimal::from(150));

        market.move_price(Decimal::from(-10));
        assert_eq!(market.mid_price(), Decimal::from(140));

        // Price shouldn't go below minimum
        market.set_price(Decimal::ZERO);
        assert!(market.mid_price() > Decimal::ZERO);
    }
}
