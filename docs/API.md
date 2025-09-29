# Hyperliquid Trading Bot API Reference

This document provides detailed information about the Hyperliquid Trading Bot API and its components.

## Table of Contents

- [Core Components](#core-components)
- [Trading Strategies](#trading-strategies)
- [Risk Management](#risk-management)
- [Configuration](#configuration)
- [Error Handling](#error-handling)

## Core Components

### TradingBot

The main trading bot class that orchestrates all trading activities.

```rust
pub struct TradingBot {
    config: Config,
    api_client: Arc<HyperliquidClient>,
    ws_client: Arc<Mutex<WebSocketClient>>,
    strategies: HashMap<String, Box<dyn Strategy + Send + Sync>>,
    risk_manager: RiskManager,
    // ... other fields
}
```

#### Methods

- `new(config: Config) -> Result<Self>` - Create a new trading bot instance
- `start() -> Result<()>` - Start the trading bot
- `stop() -> Result<()>` - Stop the trading bot
- `get_status() -> BotStatus` - Get current bot status

### HyperliquidClient

HTTP client for interacting with the Hyperliquid API.

```rust
pub struct HyperliquidClient {
    client: Client,
    base_url: String,
    api_key: String,
    private_key: String,
    testnet: bool,
}
```

#### Methods

- `get_market_data(symbol: &str) -> Result<MarketData>` - Get market data for a symbol
- `get_account_info() -> Result<AccountInfo>` - Get account information
- `place_order(order: &Order) -> Result<String>` - Place a trading order
- `cancel_order(order_id: &str) -> Result<bool>` - Cancel an order

### WebSocketClient

WebSocket client for real-time market data streaming.

```rust
pub struct WebSocketClient {
    ws_url: String,
    sender: Option<mpsc::UnboundedSender<Message>>,
    receiver: Option<mpsc::UnboundedReceiver<Message>>,
}
```

#### Methods

- `connect() -> Result<()>` - Connect to WebSocket
- `subscribe_to_ticker(symbol: &str) -> Result<()>` - Subscribe to ticker data
- `subscribe_to_l2_book(symbol: &str) -> Result<()>` - Subscribe to order book data
- `disconnect() -> Result<()>` - Disconnect from WebSocket

## Trading Strategies

### Strategy Trait

All trading strategies implement the `Strategy` trait:

```rust
#[async_trait]
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;
    fn symbol(&self) -> &str;
    fn is_enabled(&self) -> bool;
    
    async fn analyze(&self, market_data: &MarketData) -> Result<Option<StrategySignal>>;
    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> Result<()>;
    
    fn get_parameters(&self) -> HashMap<String, serde_json::Value>;
    fn validate_parameters(&self, parameters: &HashMap<String, serde_json::Value>) -> Result<()>;
}
```

### DCA Strategy

Dollar Cost Averaging strategy for systematic investment.

```rust
pub struct DCAStrategy {
    name: String,
    symbol: String,
    enabled: bool,
    investment_amount: Decimal,
    interval_hours: u64,
    max_investment: Decimal,
    // ... other fields
}
```

#### Parameters

- `investment_amount`: Amount to invest per interval
- `interval_hours`: Hours between investments
- `max_investment`: Maximum total investment
- `lookback_period`: Price analysis period

### Grid Strategy

Grid trading strategy for automated buy/sell orders.

```rust
pub struct GridStrategy {
    name: String,
    symbol: String,
    enabled: bool,
    grid_spacing: Decimal,
    position_size: Decimal,
    max_levels: usize,
    // ... other fields
}
```

#### Parameters

- `grid_spacing`: Percentage spacing between grid levels
- `position_size`: Size of each grid position
- `max_levels`: Maximum number of grid levels
- `max_investment`: Maximum total investment

### Momentum Strategy

Technical analysis-based momentum trading strategy.

```rust
pub struct MomentumStrategy {
    name: String,
    symbol: String,
    enabled: bool,
    fast_period: usize,
    slow_period: usize,
    rsi_period: usize,
    // ... other fields
}
```

#### Parameters

- `fast_period`: Fast moving average period
- `slow_period`: Slow moving average period
- `rsi_period`: RSI calculation period
- `min_confidence`: Minimum signal confidence threshold

## Risk Management

### RiskManager

Manages risk limits and position sizing.

```rust
pub struct RiskManager {
    config: RiskManagementConfig,
}
```

#### Methods

- `check_risk_limits(account_info: &AccountInfo) -> Result<bool>` - Check if risk limits are exceeded
- `check_signal_risk(signal: &StrategySignal, account_info: &AccountInfo) -> Result<bool>` - Validate signal risk

### Risk Limits

- `max_daily_loss`: Maximum daily loss limit
- `max_position_size`: Maximum position size per asset
- `stop_loss_percentage`: Stop loss percentage
- `take_profit_percentage`: Take profit percentage
- `max_drawdown_percentage`: Maximum drawdown limit

## Configuration

### Config Structure

```rust
pub struct Config {
    pub hyperliquid: HyperliquidConfig,
    pub trading: TradingConfig,
    pub strategies: HashMap<String, StrategyConfig>,
    pub risk_management: RiskManagementConfig,
    pub logging: LoggingConfig,
}
```

### HyperliquidConfig

```rust
pub struct HyperliquidConfig {
    pub base_url: String,
    pub ws_url: String,
    pub api_key: String,
    pub private_key: String,
    pub testnet: bool,
}
```

### TradingConfig

```rust
pub struct TradingConfig {
    pub dry_run: bool,
    pub max_positions: u32,
    pub default_slippage: Decimal,
    pub order_timeout_seconds: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}
```

## Error Handling

### Error Types

```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Trading error: {0}")]
    Trading(String),
    
    #[error("Strategy error: {0}")]
    Strategy(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    // ... other error types
}
```

### Error Handling Best Practices

1. Always handle errors gracefully
2. Log errors with appropriate context
3. Implement retry logic for transient errors
4. Use specific error types for different failure modes
5. Provide meaningful error messages

## Data Models

### MarketData

```rust
pub struct MarketData {
    pub symbol: String,
    pub price: Decimal,
    pub volume_24h: Decimal,
    pub change_24h: Decimal,
    pub high_24h: Decimal,
    pub low_24h: Decimal,
    pub timestamp: DateTime<Utc>,
}
```

### Order

```rust
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub filled_quantity: Decimal,
    pub average_price: Option<Decimal>,
}
```

### StrategySignal

```rust
pub struct StrategySignal {
    pub strategy_name: String,
    pub symbol: String,
    pub action: SignalAction,
    pub quantity: Decimal,
    pub price: Option<Decimal>,
    pub confidence: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

## Usage Examples

### Basic Bot Setup

```rust
use hyperliquid_trading_bot::{config::Config, trading_bot::TradingBot};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::load("config/default.toml")?;
    
    // Create trading bot
    let bot = TradingBot::new(config).await?;
    
    // Start trading
    bot.start().await?;
    
    Ok(())
}
```

### Custom Strategy Implementation

```rust
use hyperliquid_trading_bot::strategies::base::Strategy;

pub struct MyCustomStrategy {
    // ... fields
}

#[async_trait]
impl Strategy for MyCustomStrategy {
    // ... implement trait methods
}
```

### Error Handling

```rust
use hyperliquid_trading_bot::error::{Error, Result};

async fn handle_trading_error() -> Result<()> {
    match some_trading_operation().await {
        Ok(result) => Ok(result),
        Err(Error::InsufficientBalance) => {
            // Handle insufficient balance
            Ok(())
        }
        Err(Error::RateLimit) => {
            // Handle rate limiting
            tokio::time::sleep(Duration::from_secs(60)).await;
            Ok(())
        }
        Err(e) => Err(e),
    }
}
```

## Performance Considerations

- Use async/await for non-blocking operations
- Implement proper error handling and retry logic
- Monitor memory usage and avoid memory leaks
- Use appropriate data structures for performance
- Implement proper logging without impacting performance

## Security Best Practices

- Never commit API keys or private keys
- Use environment variables for sensitive data
- Validate all inputs and parameters
- Implement proper authentication and authorization
- Use secure communication protocols
- Regular security audits and updates
