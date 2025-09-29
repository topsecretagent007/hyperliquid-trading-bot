use anyhow::Result;
use clap::Parser;
use hyperliquid_trading_bot::{
    config::Config,
    trading_bot::TradingBot,
    utils::setup_logging,
};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "hyperliquid-trading-bot")]
#[command(about = "A high-performance Hyperliquid trading bot")]
#[command(version)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "config/default.toml")]
    config: String,
    
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    /// Dry run mode (no actual trades)
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Setup logging
    setup_logging(cli.debug)?;
    
    info!("🚀 Starting Hyperliquid Trading Bot");
    info!("📊 GitHub: https://github.com/topsecretagent007/hyperliquid-trading-bot");
    info!("📱 Telegram: @topsecretagent_007");
    
    // Load configuration
    let config = Config::load(&cli.config)?;
    
    // Override dry run if specified
    let mut config = config;
    if cli.dry_run {
        config.trading.dry_run = true;
        info!("🔍 Running in DRY RUN mode - no actual trades will be executed");
    }
    
    // Create trading bot
    let bot = Arc::new(TradingBot::new(config).await?);
    
    // Start the bot
    let bot_handle = {
        let bot = bot.clone();
        tokio::spawn(async move {
            if let Err(e) = bot.start().await {
                error!("Bot error: {}", e);
            }
        })
    };
    
    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("🛑 Received shutdown signal");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }
    
    // Graceful shutdown
    info!("🔄 Shutting down gracefully...");
    bot.stop().await;
    bot_handle.abort();
    
    info!("✅ Shutdown complete");
    Ok(())
}
