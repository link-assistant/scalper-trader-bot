//! Basic usage example for scalper-trader-bot.
//!
//! This example demonstrates how to use the simulator to test trading strategies.
//!
//! Run with: `cargo run --example basic_usage`

use rust_decimal::Decimal;
use scalper_trader_bot::exchange::{MarketDataProvider, OrderExecutor};
use scalper_trader_bot::simulator::{SimulatedMarketConfig, SimulatorEngine};
use scalper_trader_bot::strategy::{MarketState, ScalpingStrategy, Strategy, TradingSettings};
use scalper_trader_bot::types::{Currency, Order, OrderSide, Quantity};

#[tokio::main]
async fn main() {
    println!("Scalper Trader Bot - Basic Usage Example");
    println!("=========================================\n");

    // Create a simulated exchange
    let mut engine = SimulatorEngine::new();

    // Configure a market for ETH/USDT
    engine.add_market(SimulatedMarketConfig {
        symbol: "ETH/USDT".to_string(),
        currency: Currency::usdt(),
        initial_price: Decimal::from(2000),
        price_step: Decimal::new(1, 1), // 0.1
        lot_size: 1,
        spread_ticks: 2,
        base_liquidity: 50,
        book_depth: 10,
    });

    // Add initial balance
    engine.add_balance(Currency::usdt(), Decimal::from(10000));

    // Connect to the simulator
    engine.connect().await.expect("Failed to connect");

    println!("Step 1: Connected to simulated exchange\n");

    // Display initial market state
    let order_book = engine.get_order_book("ETH/USDT").await.unwrap();
    println!("Initial Order Book:");
    if let Some(bid) = order_book.best_bid() {
        println!(
            "  Best Bid: {} USDT ({} lots)",
            bid.price(),
            bid.total_quantity()
        );
    }
    if let Some(ask) = order_book.best_ask() {
        println!(
            "  Best Ask: {} USDT ({} lots)",
            ask.price(),
            ask.total_quantity()
        );
    }
    println!();

    // Create a scalping strategy
    let settings = TradingSettings::new("ETH/USDT")
        .with_minimum_profit_steps(2)
        .with_price_step(Decimal::new(1, 1))
        .with_lot_size(1)
        .with_max_position(5);

    let strategy = ScalpingStrategy::new(settings);
    println!("Step 2: Created {} strategy\n", strategy.name());

    // Get strategy decision
    let balance = engine.get_balance(&Currency::usdt()).await.unwrap();
    let state = MarketState {
        instrument: "ETH/USDT".to_string(),
        order_book: engine.get_order_book("ETH/USDT").await.unwrap(),
        position: None,
        active_orders: Vec::new(),
        available_cash: balance.amount(),
        timestamp: chrono::Utc::now(),
    };

    let decision = strategy.decide(&state).await;
    println!("Step 3: Strategy Decision");
    println!("  Actions: {:?}", decision.actions().len());
    if let Some(reason) = decision.reason() {
        println!("  Reason: {}", reason);
    }
    println!();

    // Execute some trades manually
    println!("Step 4: Executing manual trades");

    // Buy 2 ETH
    let buy_order = Order::market("ETH/USDT", OrderSide::Buy, Quantity::from_lots(2));
    let filled_buy = engine.submit_order(buy_order).await.expect("Buy failed");
    println!("  Bought {} ETH", filled_buy.filled_quantity());

    // Check position
    let position = engine.get_position("ETH/USDT").await.unwrap();
    println!(
        "  Position: {} ETH at avg price {}",
        position.quantity(),
        position.average_price()
    );

    // Advance the market to simulate time passing
    engine.advance(50);

    // Get new order book
    let new_book = engine.get_order_book("ETH/USDT").await.unwrap();
    println!("\nAfter 50 ticks:");
    if let Some(bid) = new_book.best_bid() {
        println!("  Best Bid: {} USDT", bid.price());
    }
    if let Some(ask) = new_book.best_ask() {
        println!("  Best Ask: {} USDT", ask.price());
    }

    // Sell the position
    let sell_order = Order::market("ETH/USDT", OrderSide::Sell, Quantity::from_lots(2));
    let filled_sell = engine.submit_order(sell_order).await.expect("Sell failed");
    println!("  Sold {} ETH", filled_sell.filled_quantity());

    // Check final balance
    let final_balance = engine.get_balance(&Currency::usdt()).await.unwrap();
    println!("\nStep 5: Final Results");
    println!("  Starting Balance: 10000 USDT");
    println!("  Final Balance: {} USDT", final_balance.amount());

    // Display trade history
    let trades = engine.get_trades(10).await.unwrap();
    println!("\nTrade History:");
    for trade in &trades {
        println!(
            "  {} {} {} @ {}",
            trade.side(),
            trade.quantity(),
            trade.instrument(),
            trade.price()
        );
    }

    println!("\nExample complete!");
}
