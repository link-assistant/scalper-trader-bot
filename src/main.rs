//! Scalper Trader Bot - Command Line Interface
//!
//! This binary provides a CLI for running the scalper trading bot.

use rust_decimal::Decimal;
use scalper_trader_bot::exchange::MarketDataProvider;
use scalper_trader_bot::simulator::{SimulatedMarketConfig, SimulatorEngine};
use scalper_trader_bot::strategy::{ScalpingStrategy, Strategy, TradingSettings};
use scalper_trader_bot::types::Currency;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    info!("Scalper Trader Bot v{}", scalper_trader_bot::VERSION);
    info!("Starting simulation demo...");

    // Create and configure the simulator
    let mut engine = SimulatorEngine::new();

    // Add a test market
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

    // Add initial balance
    engine.add_balance(Currency::usd(), Decimal::from(10000));

    // Connect to the simulated exchange
    engine
        .connect()
        .await
        .expect("Failed to connect to simulator");

    // Create the scalping strategy
    let settings = TradingSettings::new("TEST/USD")
        .with_minimum_profit_steps(2)
        .with_price_step(Decimal::new(1, 2))
        .with_lot_size(1)
        .with_max_position(10)
        .with_min_order_size_to_buy(10)
        .with_min_order_size_to_sell(10)
        .with_order_book_depth(5);

    let strategy = ScalpingStrategy::new(settings);

    info!("Strategy: {}", strategy.name());
    info!("Trading settings configured for TEST/USD");

    // Display initial market state
    let order_book = engine
        .get_order_book("TEST/USD")
        .await
        .expect("Failed to get order book");

    info!("Initial market state:");
    if let Some(bid) = order_book.best_bid() {
        info!(
            "  Best bid: {} ({} lots)",
            bid.price(),
            bid.total_quantity()
        );
    }
    if let Some(ask) = order_book.best_ask() {
        info!(
            "  Best ask: {} ({} lots)",
            ask.price(),
            ask.total_quantity()
        );
    }
    if let Some(spread) = order_book.spread() {
        info!("  Spread: {}", spread);
    }

    info!("");
    info!("Demo complete. Use the library to build your own trading bot!");
    info!("");
    info!("Supported exchanges (adapters):");
    info!("  - T-Bank (Tinkoff) - Russian broker");
    info!("  - Binance - Cryptocurrency exchange");
    info!("  - Interactive Brokers - International broker");
    info!("");
    info!("For testing, use the SimulatorEngine which implements the full Exchange trait.");
}
