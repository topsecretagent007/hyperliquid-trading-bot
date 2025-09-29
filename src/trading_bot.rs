use crate::{
    api::{HyperliquidClient, WebSocketClient},
    config::Config,
    error::{Error, Result},
    models::{AccountInfo, BotStatus, MarketData, Order, OrderSide, OrderType, Position, RiskMetrics, StrategySignal},
    strategies::{DCAStrategy, GridStrategy, MomentumStrategy, Strategy},
    utils::{log_trade_execution, log_position_update, sleep_seconds},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct TradingBot {
    config: Config,
    api_client: Arc<HyperliquidClient>,
    ws_client: Arc<Mutex<WebSocketClient>>,
    strategies: HashMap<String, Box<dyn Strategy + Send + Sync>>,
    risk_manager: RiskManager,
    is_running: Arc<Mutex<bool>>,
    start_time: DateTime<Utc>,
    trade_stats: Arc<Mutex<TradeStats>>,
}

struct TradeStats {
    total_trades: u64,
    successful_trades: u64,
    failed_trades: u64,
    total_pnl: Decimal,
    daily_pnl: Decimal,
    last_reset_date: DateTime<Utc>,
}

impl TradingBot {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Hyperliquid Trading Bot");
        
        // Create API client
        let api_client = Arc::new(HyperliquidClient::new(
            config.hyperliquid.base_url.clone(),
            config.hyperliquid.api_key.clone(),
            config.hyperliquid.private_key.clone(),
            config.hyperliquid.testnet,
        ));
        
        // Create WebSocket client
        let ws_client = Arc::new(Mutex::new(WebSocketClient::new(
            config.hyperliquid.ws_url.clone(),
        )));
        
        // Initialize strategies
        let mut strategies: HashMap<String, Box<dyn Strategy + Send + Sync>> = HashMap::new();
        
        for (name, strategy_config) in &config.strategies {
            if strategy_config.enabled {
                let strategy: Box<dyn Strategy + Send + Sync> = match strategy_config.strategy_type.as_str() {
                    "dca" => {
                        let mut dca = DCAStrategy::new(name.clone(), strategy_config.symbol.clone());
                        dca.update_parameters(strategy_config.parameters.clone()).await?;
                        Box::new(dca)
                    }
                    "grid" => {
                        let mut grid = GridStrategy::new(name.clone(), strategy_config.symbol.clone());
                        grid.update_parameters(strategy_config.parameters.clone()).await?;
                        Box::new(grid)
                    }
                    "momentum" => {
                        let mut momentum = MomentumStrategy::new(name.clone(), strategy_config.symbol.clone());
                        momentum.update_parameters(strategy_config.parameters.clone()).await?;
                        Box::new(momentum)
                    }
                    _ => {
                        warn!("Unknown strategy type: {}", strategy_config.strategy_type);
                        continue;
                    }
                };
                
                strategies.insert(name.clone(), strategy);
                info!("Initialized strategy: {} ({})", name, strategy_config.strategy_type);
            }
        }
        
        // Initialize risk manager
        let risk_manager = RiskManager::new(config.risk_management.clone());
        
        // Initialize trade stats
        let trade_stats = Arc::new(Mutex::new(TradeStats {
            total_trades: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_pnl: Decimal::ZERO,
            daily_pnl: Decimal::ZERO,
            last_reset_date: Utc::now().date_naive(),
        }));
        
        Ok(Self {
            config,
            api_client,
            ws_client,
            strategies,
            risk_manager,
            is_running: Arc::new(Mutex::new(false)),
            start_time: Utc::now(),
            trade_stats,
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting trading bot");
        
        // Set running flag
        {
            let mut is_running = self.is_running.lock().await;
            *is_running = true;
        }
        
        // Connect to WebSocket
        {
            let mut ws_client = self.ws_client.lock().await;
            ws_client.connect().await?;
        }
        
        // Main trading loop
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        
        while *self.is_running.lock().await {
            interval.tick().await;
            
            if let Err(e) = self.trading_cycle().await {
                error!("Error in trading cycle: {}", e);
                sleep_seconds(10).await; // Wait before retrying
            }
        }
        
        info!("Trading bot stopped");
        Ok(())
    }
    
    pub async fn stop(&self) {
        info!("🛑 Stopping trading bot");
        
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        
        // Disconnect WebSocket
        if let Ok(mut ws_client) = self.ws_client.try_lock() {
            let _ = ws_client.disconnect().await;
        }
    }
    
    async fn trading_cycle(&self) -> Result<()> {
        debug!("Starting trading cycle");
        
        // Get account info
        let account_info = self.api_client.get_account_info().await?;
        
        // Check risk limits
        if !self.risk_manager.check_risk_limits(&account_info).await? {
            warn!("Risk limits exceeded, skipping trading cycle");
            return Ok(());
        }
        
        // Update trade stats
        self.update_trade_stats(&account_info).await;
        
        // Run strategies
        for (name, strategy) in &self.strategies {
            if !strategy.is_enabled() {
                continue;
            }
            
            debug!("Running strategy: {}", name);
            
            // Get market data for strategy symbol
            let market_data = self.api_client.get_market_data(strategy.symbol()).await?;
            
            // Analyze with strategy
            if let Some(signal) = strategy.analyze(&market_data).await? {
                info!("Strategy {} generated signal: {:?}", name, signal.action);
                
                // Check if we should execute the signal
                if self.should_execute_signal(&signal, &account_info).await? {
                    if let Err(e) = self.execute_signal(&signal).await {
                        error!("Failed to execute signal from {}: {}", name, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn should_execute_signal(&self, signal: &StrategySignal, account_info: &AccountInfo) -> Result<bool> {
        // Check if we have enough balance
        if signal.quantity * signal.price.unwrap_or(Decimal::ZERO) > account_info.available_balance {
            warn!("Insufficient balance for signal execution");
            return Ok(false);
        }
        
        // Check risk limits
        if !self.risk_manager.check_signal_risk(signal, account_info).await? {
            warn!("Signal rejected by risk manager");
            return Ok(false);
        }
        
        // Check confidence threshold
        if signal.confidence < 0.5 {
            warn!("Signal confidence too low: {:.2}", signal.confidence);
            return Ok(false);
        }
        
        Ok(true)
    }
    
    async fn execute_signal(&self, signal: &StrategySignal) -> Result<()> {
        info!("Executing signal: {:?} {} {} at {:?}", 
              signal.action, signal.quantity, signal.symbol, signal.price);
        
        if self.config.trading.dry_run {
            info!("DRY RUN: Would execute trade");
            return Ok(());
        }
        
        // Create order
        let order = Order {
            id: Uuid::new_v4().to_string(),
            symbol: signal.symbol.clone(),
            side: match signal.action {
                crate::models::SignalAction::Buy => OrderSide::Buy,
                crate::models::SignalAction::Sell => OrderSide::Sell,
                _ => return Ok(()), // Skip hold/close signals
            },
            order_type: if signal.price.is_some() { OrderType::Limit } else { OrderType::Market },
            quantity: signal.quantity,
            price: signal.price,
            status: crate::models::OrderStatus::Pending,
            created_at: Utc::now(),
            updated_at: None,
            filled_quantity: Decimal::ZERO,
            average_price: None,
        };
        
        // Place order
        match self.api_client.place_order(&order).await {
            Ok(order_id) => {
                log_trade_execution(&order.symbol, &order.side, order.quantity, order.price.unwrap_or(Decimal::ZERO), true);
                
                // Update trade stats
                let mut stats = self.trade_stats.lock().await;
                stats.total_trades += 1;
                stats.successful_trades += 1;
            }
            Err(e) => {
                log_trade_execution(&order.symbol, &order.side, order.quantity, order.price.unwrap_or(Decimal::ZERO), false);
                
                // Update trade stats
                let mut stats = self.trade_stats.lock().await;
                stats.total_trades += 1;
                stats.failed_trades += 1;
                
                return Err(e);
            }
        }
        
        Ok(())
    }
    
    async fn update_trade_stats(&self, account_info: &AccountInfo) {
        let mut stats = self.trade_stats.lock().await;
        
        // Reset daily PnL if new day
        let today = Utc::now().date_naive();
        if today > stats.last_reset_date {
            stats.daily_pnl = Decimal::ZERO;
            stats.last_reset_date = today;
        }
        
        // Update PnL
        stats.total_pnl = account_info.total_pnl;
        stats.daily_pnl = account_info.total_pnl; // Simplified - would need proper daily tracking
    }
    
    pub async fn get_status(&self) -> BotStatus {
        let is_running = *self.is_running.lock().await;
        let uptime = Utc::now() - self.start_time;
        let stats = self.trade_stats.lock().await;
        
        BotStatus {
            is_running,
            start_time: self.start_time,
            uptime_seconds: uptime.num_seconds() as u64,
            total_trades: stats.total_trades,
            successful_trades: stats.successful_trades,
            failed_trades: stats.failed_trades,
            current_positions: 0, // Would get from account info
            risk_metrics: RiskMetrics {
                current_drawdown: Decimal::ZERO, // Would calculate from historical data
                max_drawdown: Decimal::ZERO,
                daily_pnl: stats.daily_pnl,
                total_pnl: stats.total_pnl,
                win_rate: if stats.total_trades > 0 {
                    stats.successful_trades as f64 / stats.total_trades as f64
                } else {
                    0.0
                },
                profit_factor: 1.0, // Would calculate from trade history
                sharpe_ratio: 0.0, // Would calculate from returns
                max_position_risk: Decimal::ZERO,
            },
        }
    }
}

pub struct RiskManager {
    config: crate::config::RiskManagementConfig,
}

impl RiskManager {
    pub fn new(config: crate::config::RiskManagementConfig) -> Self {
        Self { config }
    }
    
    pub async fn check_risk_limits(&self, account_info: &AccountInfo) -> Result<bool> {
        // Check daily loss limit
        if account_info.total_pnl < -self.config.max_daily_loss {
            warn!("Daily loss limit exceeded: {} < {}", account_info.total_pnl, -self.config.max_daily_loss);
            return Ok(false);
        }
        
        // Check position size limits
        for position in &account_info.positions {
            let position_value = position.size * position.current_price;
            if position_value > self.config.max_position_size {
                warn!("Position size limit exceeded: {} > {}", position_value, self.config.max_position_size);
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    pub async fn check_signal_risk(&self, signal: &StrategySignal, account_info: &AccountInfo) -> Result<bool> {
        // Check if signal would exceed position size limit
        if let Some(price) = signal.price {
            let position_value = signal.quantity * price;
            if position_value > self.config.max_position_size {
                warn!("Signal would exceed position size limit");
                return Ok(false);
            }
        }
        
        // Additional risk checks can be added here
        Ok(true)
    }
}
