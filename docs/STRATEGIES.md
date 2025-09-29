# Trading Strategies Guide

This document provides detailed information about the trading strategies implemented in the Hyperliquid Trading Bot.

## Table of Contents

- [Strategy Overview](#strategy-overview)
- [DCA Strategy](#dca-strategy)
- [Grid Strategy](#grid-strategy)
- [Momentum Strategy](#momentum-strategy)
- [Custom Strategy Development](#custom-strategy-development)
- [Strategy Configuration](#strategy-configuration)
- [Performance Metrics](#performance-metrics)

## Strategy Overview

The Hyperliquid Trading Bot implements three main trading strategies:

1. **DCA (Dollar Cost Averaging)** - Systematic investment strategy
2. **Grid Trading** - Automated buy/sell orders at predetermined levels
3. **Momentum Trading** - Technical analysis-based strategy

Each strategy is designed to be modular, configurable, and can be run independently or in combination.

## DCA Strategy

### Overview

The Dollar Cost Averaging (DCA) strategy systematically invests a fixed amount at regular intervals, reducing the impact of market volatility by spreading purchases over time.

### How It Works

1. **Investment Schedule**: Invests a fixed amount at predetermined intervals
2. **Price Analysis**: Analyzes recent price trends to optimize timing
3. **Risk Management**: Includes maximum investment limits and stop conditions
4. **Trend Following**: Adjusts investment timing based on market conditions

### Configuration

```toml
[strategies.dca_btc]
enabled = true
strategy_type = "dca"
symbol = "BTC"
position_size = 100.0
parameters = {
    investment_amount = "100",    # $100 per interval
    interval_hours = "24",        # Daily investment
    max_investment = "5000",      # Maximum total investment
    lookback_period = "20"        # Price analysis period
}
```

### Parameters

| Parameter | Type | Description | Default | Range |
|-----------|------|-------------|---------|-------|
| `investment_amount` | Decimal | Amount to invest per interval | 100 | > 0 |
| `interval_hours` | u64 | Hours between investments | 24 | 1-168 |
| `max_investment` | Decimal | Maximum total investment | 10000 | > 0 |
| `lookback_period` | usize | Price analysis period | 20 | 5-100 |

### Signal Generation

The DCA strategy generates buy signals when:

1. **Time Condition**: Enough time has passed since the last investment
2. **Investment Limit**: Total investment hasn't exceeded the maximum
3. **Price Condition**: Current price is below recent average (optional)

### Example Usage

```rust
use hyperliquid_trading_bot::strategies::DCAStrategy;

let mut dca = DCAStrategy::new("dca_btc".to_string(), "BTC".to_string());

// Configure parameters
let mut parameters = HashMap::new();
parameters.insert("investment_amount".to_string(), serde_json::Value::String("100".to_string()));
parameters.insert("interval_hours".to_string(), serde_json::Value::Number(24.into()));

dca.update_parameters(parameters).await?;
```

## Grid Strategy

### Overview

The Grid Trading strategy places buy and sell orders at predetermined price levels to profit from market volatility. It creates a grid of orders above and below the current price.

### How It Works

1. **Grid Initialization**: Creates buy/sell levels around a base price
2. **Order Placement**: Places orders at each grid level
3. **Order Filling**: When orders are filled, places opposite orders
4. **Profit Taking**: Captures profits from price movements

### Configuration

```toml
[strategies.grid_eth]
enabled = true
strategy_type = "grid"
symbol = "ETH"
position_size = 50.0
parameters = {
    grid_spacing = "1.0",         # 1% between levels
    position_size = "50",         # $50 per grid level
    max_levels = "10",            # 10 levels above/below
    max_investment = "3000"       # Maximum total investment
}
```

### Parameters

| Parameter | Type | Description | Default | Range |
|-----------|------|-------------|---------|-------|
| `grid_spacing` | Decimal | Percentage between levels | 1.0 | 0.1-50.0 |
| `position_size` | Decimal | Size of each grid position | 100 | > 0 |
| `max_levels` | usize | Maximum number of levels | 10 | 1-50 |
| `max_investment` | Decimal | Maximum total investment | 5000 | > 0 |

### Signal Generation

The Grid strategy generates signals when:

1. **Buy Signal**: Price reaches a buy level above current price
2. **Sell Signal**: Price reaches a sell level below current price
3. **Grid Management**: Manages active orders and fills

### Example Usage

```rust
use hyperliquid_trading_bot::strategies::GridStrategy;

let mut grid = GridStrategy::new("grid_eth".to_string(), "ETH".to_string());

// Configure parameters
let mut parameters = HashMap::new();
parameters.insert("grid_spacing".to_string(), serde_json::Value::String("2.0".to_string()));
parameters.insert("position_size".to_string(), serde_json::Value::String("50".to_string()));

grid.update_parameters(parameters).await?;

// Initialize grid with base price
grid.initialize_with_price(Decimal::from(2000));
```

## Momentum Strategy

### Overview

The Momentum Trading strategy uses technical indicators to identify momentum and trend changes. It combines multiple indicators for robust signal generation.

### How It Works

1. **Technical Analysis**: Calculates MACD, RSI, and moving averages
2. **Signal Detection**: Identifies bullish and bearish signals
3. **Confidence Scoring**: Assigns confidence levels to signals
4. **Position Sizing**: Adjusts position size based on confidence

### Configuration

```toml
[strategies.momentum_sol]
enabled = true
strategy_type = "momentum"
symbol = "SOL"
position_size = 75.0
parameters = {
    fast_period = "12",           # Fast EMA period
    slow_period = "26",           # Slow EMA period
    rsi_period = "14",            # RSI calculation period
    min_confidence = "0.6"        # Minimum signal confidence
}
```

### Parameters

| Parameter | Type | Description | Default | Range |
|-----------|------|-------------|---------|-------|
| `fast_period` | usize | Fast EMA period | 12 | 1-100 |
| `slow_period` | usize | Slow EMA period | 26 | 1-100 |
| `signal_period` | usize | MACD signal period | 9 | 1-100 |
| `rsi_period` | usize | RSI calculation period | 14 | 1-100 |
| `rsi_oversold` | Decimal | RSI oversold threshold | 30 | 0-100 |
| `rsi_overbought` | Decimal | RSI overbought threshold | 70 | 0-100 |
| `min_confidence` | f64 | Minimum signal confidence | 0.6 | 0.0-1.0 |

### Technical Indicators

#### MACD (Moving Average Convergence Divergence)
- **Fast EMA**: 12-period exponential moving average
- **Slow EMA**: 26-period exponential moving average
- **Signal Line**: 9-period EMA of MACD line
- **Histogram**: Difference between MACD and signal line

#### RSI (Relative Strength Index)
- **Period**: 14-period RSI calculation
- **Oversold**: Below 30 (potential buy signal)
- **Overbought**: Above 70 (potential sell signal)

#### Moving Averages
- **Fast SMA**: Short-term simple moving average
- **Slow SMA**: Long-term simple moving average
- **Trend Analysis**: Price position relative to moving averages

### Signal Generation

The Momentum strategy generates signals based on:

1. **MACD Crossover**: Bullish when MACD crosses above signal line
2. **RSI Levels**: Oversold/overbought conditions
3. **Moving Average Position**: Price above/below moving averages
4. **Volume Confirmation**: High volume supporting the signal

### Example Usage

```rust
use hyperliquid_trading_bot::strategies::MomentumStrategy;

let mut momentum = MomentumStrategy::new("momentum_sol".to_string(), "SOL".to_string());

// Configure parameters
let mut parameters = HashMap::new();
parameters.insert("fast_period".to_string(), serde_json::Value::Number(12.into()));
parameters.insert("slow_period".to_string(), serde_json::Value::Number(26.into()));
parameters.insert("rsi_period".to_string(), serde_json::Value::Number(14.into()));

momentum.update_parameters(parameters).await?;
```

## Custom Strategy Development

### Strategy Trait

All strategies implement the `Strategy` trait:

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

### Creating a Custom Strategy

1. **Implement the Strategy trait**
2. **Define strategy-specific parameters**
3. **Implement signal generation logic**
4. **Add parameter validation**
5. **Register with the trading bot**

### Example: Mean Reversion Strategy

```rust
pub struct MeanReversionStrategy {
    name: String,
    symbol: String,
    enabled: bool,
    lookback_period: usize,
    deviation_threshold: Decimal,
    position_size: Decimal,
    price_history: Vec<Decimal>,
}

#[async_trait]
impl Strategy for MeanReversionStrategy {
    // ... implement trait methods
}
```

## Strategy Configuration

### Configuration File Structure

```toml
[strategies.strategy_name]
enabled = true
strategy_type = "strategy_type"
symbol = "SYMBOL"
position_size = 100.0
parameters = {
    param1 = "value1",
    param2 = "value2"
}
```

### Environment Variables

Strategies can be configured via environment variables:

```bash
HYPERLIQUID_STRATEGIES_DCA_BTC_ENABLED=true
HYPERLIQUID_STRATEGIES_DCA_BTC_INVESTMENT_AMOUNT=100
HYPERLIQUID_STRATEGIES_DCA_BTC_INTERVAL_HOURS=24
```

## Performance Metrics

### Strategy Performance Tracking

Each strategy tracks:

- **Signal Generation**: Number of signals generated
- **Signal Accuracy**: Percentage of profitable signals
- **Execution Success**: Percentage of successful executions
- **Risk Metrics**: Drawdown, volatility, Sharpe ratio

### Risk Management Integration

All strategies integrate with the risk management system:

- **Position Size Limits**: Maximum position size per strategy
- **Daily Loss Limits**: Maximum daily loss per strategy
- **Correlation Limits**: Maximum correlation between strategies
- **Overall Risk Limits**: Portfolio-level risk controls

### Monitoring and Alerts

- **Performance Alerts**: When strategy performance drops
- **Risk Alerts**: When risk limits are approached
- **System Alerts**: When strategy execution fails
- **Market Alerts**: When market conditions change

## Best Practices

### Strategy Development

1. **Start Simple**: Begin with basic strategies and add complexity
2. **Test Thoroughly**: Use backtesting and paper trading
3. **Monitor Performance**: Track metrics and adjust parameters
4. **Risk Management**: Always implement proper risk controls
5. **Documentation**: Document strategy logic and parameters

### Strategy Configuration

1. **Conservative Settings**: Start with conservative parameters
2. **Gradual Adjustment**: Adjust parameters gradually
3. **Multiple Timeframes**: Test on different timeframes
4. **Market Conditions**: Adapt to different market conditions
5. **Regular Review**: Regularly review and update strategies

### Strategy Monitoring

1. **Real-time Monitoring**: Monitor strategy performance in real-time
2. **Alert Systems**: Set up alerts for important events
3. **Log Analysis**: Analyze logs for patterns and issues
4. **Performance Review**: Regular performance reviews
5. **Continuous Improvement**: Continuously improve strategies

## Troubleshooting

### Common Issues

1. **No Signals Generated**: Check strategy parameters and market data
2. **Poor Performance**: Review strategy logic and parameters
3. **High Drawdown**: Adjust risk management settings
4. **Execution Failures**: Check API connectivity and limits
5. **Configuration Errors**: Validate configuration files

### Debugging

1. **Enable Debug Logging**: Use `--debug` flag for detailed logs
2. **Check Logs**: Review log files for errors and warnings
3. **Test Parameters**: Use dry-run mode to test strategies
4. **Monitor Metrics**: Track performance metrics
5. **Seek Help**: Contact support for complex issues

## Conclusion

The Hyperliquid Trading Bot provides a robust framework for implementing and managing trading strategies. By following the guidelines in this document, you can create effective trading strategies that integrate seamlessly with the bot's risk management and monitoring systems.

For more information, see the [API Reference](API.md) and [README](README.md).
