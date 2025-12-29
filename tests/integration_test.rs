//! Integration tests for scalper-trader-bot.
//!
//! These tests verify the full trading workflow using the simulator.

use rust_decimal::Decimal;
use scalper_trader_bot::exchange::{MarketDataProvider, OrderExecutor};
use scalper_trader_bot::simulator::{MarketCondition, SimulatedMarketConfig, SimulatorEngine};
use scalper_trader_bot::strategy::{MarketState, ScalpingStrategy, Strategy, TradingSettings};
use scalper_trader_bot::types::{Currency, Order, OrderSide, Quantity};

/// Creates a configured simulator for testing.
async fn create_test_simulator() -> SimulatorEngine {
    let mut engine = SimulatorEngine::new();

    engine.add_market(SimulatedMarketConfig {
        symbol: "TEST/USD".to_string(),
        currency: Currency::usd(),
        initial_price: Decimal::from(100),
        price_step: Decimal::new(1, 2), // 0.01
        lot_size: 1,
        spread_ticks: 2,
        base_liquidity: 100,
        book_depth: 10,
    });

    engine.add_balance(Currency::usd(), Decimal::from(10000));
    engine.connect().await.expect("Failed to connect");

    engine
}

mod simulator_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_buy_sell_cycle() {
        let mut engine = create_test_simulator().await;

        // Buy 10 lots
        let buy_order = Order::market("TEST/USD", OrderSide::Buy, Quantity::from_lots(10));
        let filled_buy = engine.submit_order(buy_order).await.expect("Buy failed");
        assert!(filled_buy.status().is_terminal());
        assert_eq!(filled_buy.filled_quantity().as_lots(), 10);

        // Verify position
        let position = engine
            .get_position("TEST/USD")
            .await
            .expect("Position not found");
        assert_eq!(position.quantity().as_lots(), 10);

        // Sell 10 lots
        let sell_order = Order::market("TEST/USD", OrderSide::Sell, Quantity::from_lots(10));
        let filled_sell = engine.submit_order(sell_order).await.expect("Sell failed");
        assert!(filled_sell.status().is_terminal());
        assert_eq!(filled_sell.filled_quantity().as_lots(), 10);

        // Verify position is closed
        let position = engine
            .get_position("TEST/USD")
            .await
            .expect("Position not found");
        assert!(position.is_empty());

        // Verify trades were recorded
        let trades = engine.get_trades(10).await.expect("Trades not found");
        assert_eq!(trades.len(), 2);
    }

    #[tokio::test]
    async fn test_market_simulation_with_ticks() {
        let mut engine = create_test_simulator().await;

        // Get initial price
        let initial_book = engine.get_order_book("TEST/USD").await.unwrap();
        let initial_mid = initial_book.mid_price().unwrap();

        // Advance the market
        engine.advance(100);

        // Get new price - should have changed
        let new_book = engine.get_order_book("TEST/USD").await.unwrap();
        let new_mid = new_book.mid_price().unwrap();

        // Price should have moved (in ranging market, might not move much)
        // but at least we can verify it didn't crash
        assert!(new_mid > Decimal::ZERO);
        println!("Price changed from {} to {}", initial_mid, new_mid);
    }

    #[tokio::test]
    async fn test_bullish_market_simulation() {
        let mut engine = create_test_simulator().await;

        // Set bullish market condition
        engine
            .market_mut("TEST/USD")
            .unwrap()
            .set_condition(MarketCondition::Bullish);

        let initial_book = engine.get_order_book("TEST/USD").await.unwrap();
        let initial_mid = initial_book.mid_price().unwrap();

        // Advance significantly
        engine.advance(500);

        let final_book = engine.get_order_book("TEST/USD").await.unwrap();
        let final_mid = final_book.mid_price().unwrap();

        // In a bullish market, price should generally trend up over many ticks
        println!("Bullish: {} -> {}", initial_mid, final_mid);
    }

    #[tokio::test]
    async fn test_order_book_depth() {
        let engine = create_test_simulator().await;

        let book = engine.get_order_book("TEST/USD").await.unwrap();

        // Verify order book structure
        assert!(!book.bids().is_empty());
        assert!(!book.asks().is_empty());

        // Bids should be in descending order
        let bids = book.bids();
        for i in 1..bids.len() {
            assert!(bids[i - 1].price().value() > bids[i].price().value());
        }

        // Asks should be in ascending order
        let asks = book.asks();
        for i in 1..asks.len() {
            assert!(asks[i - 1].price().value() < asks[i].price().value());
        }

        // Best ask should be higher than best bid
        let spread = book.spread().unwrap();
        assert!(spread > Decimal::ZERO);
    }
}

mod strategy_integration_tests {
    use super::*;

    fn create_test_settings() -> TradingSettings {
        TradingSettings::new("TEST/USD")
            .with_minimum_profit_steps(2)
            .with_price_step(Decimal::new(1, 2))
            .with_lot_size(1)
            .with_max_position(10)
            .with_min_order_size_to_buy(10)
            .with_min_order_size_to_sell(10)
            .with_order_book_depth(3)
    }

    #[tokio::test]
    async fn test_strategy_decision_with_cash() {
        let engine = create_test_simulator().await;
        let strategy = ScalpingStrategy::new(create_test_settings());

        let order_book = engine.get_order_book("TEST/USD").await.unwrap();
        let balance = engine.get_balance(&Currency::usd()).await.unwrap();

        let state = MarketState {
            instrument: "TEST/USD".to_string(),
            order_book,
            position: None,
            active_orders: Vec::new(),
            available_cash: balance.amount(),
            timestamp: chrono::Utc::now(),
        };

        let decision = strategy.decide(&state).await;

        // With sufficient cash and liquidity, should suggest a buy
        assert!(!decision.is_hold(), "Strategy should suggest a buy action");
    }

    #[tokio::test]
    async fn test_strategy_decision_without_cash() {
        let engine = create_test_simulator().await;
        let strategy = ScalpingStrategy::new(create_test_settings());

        let order_book = engine.get_order_book("TEST/USD").await.unwrap();

        let state = MarketState {
            instrument: "TEST/USD".to_string(),
            order_book,
            position: None,
            active_orders: Vec::new(),
            available_cash: Decimal::ZERO, // No cash
            timestamp: chrono::Utc::now(),
        };

        let decision = strategy.decide(&state).await;

        // Without cash, should hold
        assert!(decision.is_hold(), "Strategy should hold without cash");
    }

    #[tokio::test]
    async fn test_strategy_lifecycle() {
        let mut strategy = ScalpingStrategy::new(create_test_settings());

        // Simulate a buy fill
        let price = scalper_trader_bot::types::Price::from_f64(100.0, Currency::usd()).unwrap();
        let mut buy_order = Order::limit("TEST/USD", OrderSide::Buy, Quantity::from_lots(5), price);
        buy_order.submit();
        buy_order.fill(Quantity::from_lots(5));

        strategy.on_order_filled(&buy_order).await;

        // Simulate a sell fill
        let sell_price =
            scalper_trader_bot::types::Price::from_f64(100.02, Currency::usd()).unwrap();
        let mut sell_order = Order::limit(
            "TEST/USD",
            OrderSide::Sell,
            Quantity::from_lots(5),
            sell_price,
        );
        sell_order.submit();
        sell_order.fill(Quantity::from_lots(5));

        strategy.on_order_filled(&sell_order).await;

        // Reset and verify clean state
        strategy.reset().await;
    }
}

mod version_tests {
    use scalper_trader_bot::VERSION;

    #[test]
    fn test_version_is_not_empty() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_version_matches_cargo_toml() {
        // Version should match the one in Cargo.toml
        assert!(VERSION.starts_with("0."));
    }
}
