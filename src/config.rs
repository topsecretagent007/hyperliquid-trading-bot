use crate::error::{Error, Result};
use config::{Config as ConfigFile, File, FileFormat};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hyperliquid: HyperliquidConfig,
    pub trading: TradingConfig,
    pub strategies: HashMap<String, StrategyConfig>,
    pub risk_management: RiskManagementConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperliquidConfig {
    pub base_url: String,
    pub ws_url: String,
    pub api_key: String,
    pub private_key: String,
    pub testnet: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub dry_run: bool,
    pub max_positions: u32,
    pub default_slippage: Decimal,
    pub order_timeout_seconds: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub enabled: bool,
    pub strategy_type: String,
    pub symbol: String,
    pub position_size: Decimal,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    pub max_daily_loss: Decimal,
    pub max_position_size: Decimal,
    pub stop_loss_percentage: Decimal,
    pub take_profit_percentage: Decimal,
    pub max_drawdown_percentage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size_mb: u64,
    pub max_files: u32,
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let mut config = ConfigFile::new();
        
        // Load default configuration
        config = config.add_source(File::new("config/default", FileFormat::Toml).required(false));
        
        // Load custom configuration if provided
        if path != "config/default.toml" {
            config = config.add_source(File::new(path, FileFormat::Toml).required(true));
        }
        
        // Load environment variables
        config = config.add_source(config::Environment::with_prefix("HYPERLIQUID"));
        
        let config: Config = config.try_into()
            .map_err(|e| Error::Config(format!("Failed to load configuration: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    fn validate(&self) -> Result<()> {
        if self.hyperliquid.api_key.is_empty() {
            return Err(Error::Config("API key is required".to_string()));
        }
        
        if self.hyperliquid.private_key.is_empty() {
            return Err(Error::Config("Private key is required".to_string()));
        }
        
        if self.trading.max_positions == 0 {
            return Err(Error::Config("Max positions must be greater than 0".to_string()));
        }
        
        if self.risk_management.max_position_size <= Decimal::ZERO {
            return Err(Error::Config("Max position size must be greater than 0".to_string()));
        }
        
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hyperliquid: HyperliquidConfig {
                base_url: "https://api.hyperliquid.xyz".to_string(),
                ws_url: "wss://api.hyperliquid.xyz/ws".to_string(),
                api_key: String::new(),
                private_key: String::new(),
                testnet: true,
            },
            trading: TradingConfig {
                dry_run: true,
                max_positions: 10,
                default_slippage: Decimal::new(1, 2), // 1%
                order_timeout_seconds: 30,
                retry_attempts: 3,
                retry_delay_ms: 1000,
            },
            strategies: HashMap::new(),
            risk_management: RiskManagementConfig {
                max_daily_loss: Decimal::new(1000, 0), // $1000
                max_position_size: Decimal::new(10000, 0), // $10000
                stop_loss_percentage: Decimal::new(5, 0), // 5%
                take_profit_percentage: Decimal::new(10, 0), // 10%
                max_drawdown_percentage: Decimal::new(20, 0), // 20%
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("logs/bot.log".to_string()),
                max_file_size_mb: 100,
                max_files: 10,
            },
        }
    }
}
