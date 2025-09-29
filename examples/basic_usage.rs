//! Basic usage examples for the Hyperliquid Trading Bot
//! 
//! This file demonstrates how to use the trading bot with different configurations
//! and strategies. Run with: `cargo run --example basic_usage`

use hyperliquid_trading_bot::{
    config::Config,
    strategies::{DCAStrategy, GridStrategy, MomentumStrategy},
    trading_bot::TradingBot,
    utils::setup_logging,
};
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    setup_logging(false)?;
    
    info!("🚀 Hyperliquid Trading Bot - Basic Usage Examples");
    
    // Example 1: Load configuration from file
    example_load_config().await?;
    
    // Example 2: Create custom DCA strategy
    example_dca_strategy().await?;
    
    // Example 3: Create custom Grid strategy
    example_grid_strategy().await?;
    
    // Example 4: Create custom Momentum strategy
    example_momentum_strategy().await?;
    
    // Example 5: Run bot with custom configuration
    example_run_bot().await?;
    
    Ok(())
}

async fn example_load_config() -> Result<(), Box<dyn std::error::Error>> {
    info!("📋 Example 1: Loading Configuration");
    
    // Load default configuration
    let config = Config::load("config/default.toml")?;
    
    info!("Configuration loaded:");
    info!("  - Testnet: {}", config.hyperliquid.testnet);
    info!("  - Dry run: {}", config.trading.dry_run);
    info!("  - Max positions: {}", config.trading.max_positions);
    info!("  - Strategies: {}", config.strategies.len());
    
    Ok(())
}

async fn example_dca_strategy() -> Result<(), Box<dyn std::error::Error>> {
    info!("📈 Example 2: DCA Strategy");
    
    let mut dca = DCAStrategy::new("example_dca".to_string(), "BTC".to_string());
    
    // Configure DCA parameters
    let mut parameters = HashMap::new();
    parameters.insert("investment_amount".to_string(), serde_json::Value::String("50".to_string()));
    parameters.insert("interval_hours".to_string(), serde_json::Value::Number(12.into()));
    parameters.insert("max_investment".to_string(), serde_json::Value::String("1000".to_string()));
    
    dca.update_parameters(parameters).await?;
    
    info!("DCA Strategy configured:");
    info!("  - Investment amount: $50");
    info!("  - Interval: 12 hours");
    info!("  - Max investment: $1000");
    
    Ok(())
}

async fn example_grid_strategy() -> Result<(), Box<dyn std::error::Error>> {
    info!("🔲 Example 3: Grid Strategy");
    
    let mut grid = GridStrategy::new("example_grid".to_string(), "ETH".to_string());
    
    // Configure Grid parameters
    let mut parameters = HashMap::new();
    parameters.insert("grid_spacing".to_string(), serde_json::Value::String("2.0".to_string()));
    parameters.insert("position_size".to_string(), serde_json::Value::String("25".to_string()));
    parameters.insert("max_levels".to_string(), serde_json::Value::Number(5.into()));
    
    grid.update_parameters(parameters).await?;
    
    // Initialize grid with a base price
    grid.initialize_with_price(Decimal::from(2000)); // $2000 ETH
    
    info!("Grid Strategy configured:");
    info!("  - Grid spacing: 2%");
    info!("  - Position size: $25");
    info!("  - Max levels: 5");
    info!("  - Base price: $2000");
    
    Ok(())
}

async fn example_momentum_strategy() -> Result<(), Box<dyn std::error::Error>> {
    info!("📊 Example 4: Momentum Strategy");
    
    let mut momentum = MomentumStrategy::new("example_momentum".to_string(), "SOL".to_string());
    
    // Configure Momentum parameters
    let mut parameters = HashMap::new();
    parameters.insert("fast_period".to_string(), serde_json::Value::Number(10.into()));
    parameters.insert("slow_period".to_string(), serde_json::Value::Number(20.into()));
    parameters.insert("rsi_period".to_string(), serde_json::Value::Number(14.into()));
    parameters.insert("min_confidence".to_string(), serde_json::Value::Number(0.7.into()));
    
    momentum.update_parameters(parameters).await?;
    
    info!("Momentum Strategy configured:");
    info!("  - Fast period: 10");
    info!("  - Slow period: 20");
    info!("  - RSI period: 14");
    info!("  - Min confidence: 0.7");
    
    Ok(())
}

async fn example_run_bot() -> Result<(), Box<dyn std::error::Error>> {
    info!("🤖 Example 5: Running Trading Bot");
    
    // Load configuration
    let config = Config::load("config/default.toml")?;
    
    // Create trading bot
    let bot = TradingBot::new(config).await?;
    
    info!("Trading bot created successfully!");
    info!("Bot features:");
    info!("  - Multiple trading strategies");
    info!("  - Risk management");
    info!("  - Real-time market data");
    info!("  - Comprehensive logging");
    
    // Note: In a real scenario, you would call bot.start().await? here
    // For this example, we'll just show the bot was created successfully
    
    info!("✅ All examples completed successfully!");
    
    Ok(())
}
