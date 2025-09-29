//! Custom strategy implementation example
//! 
//! This example shows how to create a custom trading strategy by implementing
//! the Strategy trait. Run with: `cargo run --example strategy_custom`

use hyperliquid_trading_bot::{
    error::Result,
    models::{MarketData, StrategySignal, SignalAction},
    strategies::base::Strategy,
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::info;

/// Custom Mean Reversion Strategy
/// 
/// This strategy identifies when an asset's price has deviated significantly
/// from its moving average and signals a trade in the opposite direction.
pub struct MeanReversionStrategy {
    name: String,
    symbol: String,
    enabled: bool,
    parameters: HashMap<String, serde_json::Value>,
    
    // Strategy-specific parameters
    lookback_period: usize,
    deviation_threshold: Decimal,
    position_size: Decimal,
    price_history: Vec<Decimal>,
}

impl MeanReversionStrategy {
    pub fn new(name: String, symbol: String) -> Self {
        Self {
            name,
            symbol,
            enabled: true,
            parameters: HashMap::new(),
            lookback_period: 20,
            deviation_threshold: Decimal::new(2, 0), // 2%
            position_size: Decimal::from(100),
            price_history: Vec::new(),
        }
    }
    
    fn calculate_moving_average(&self) -> Option<Decimal> {
        if self.price_history.len() < self.lookback_period {
            return None;
        }
        
        let recent_prices = &self.price_history[self.price_history.len() - self.lookback_period..];
        let sum: Decimal = recent_prices.iter().sum();
        Some(sum / Decimal::from(recent_prices.len()))
    }
    
    fn calculate_deviation(&self, price: Decimal, ma: Decimal) -> Decimal {
        ((price - ma) / ma * Decimal::from(100)).abs()
    }
    
    fn should_buy(&self, price: Decimal, ma: Decimal) -> bool {
        let deviation = self.calculate_deviation(price, ma);
        price < ma && deviation > self.deviation_threshold
    }
    
    fn should_sell(&self, price: Decimal, ma: Decimal) -> bool {
        let deviation = self.calculate_deviation(price, ma);
        price > ma && deviation > self.deviation_threshold
    }
    
    fn calculate_confidence(&self, price: Decimal, ma: Decimal) -> f64 {
        let deviation = self.calculate_deviation(price, ma);
        let max_deviation = Decimal::new(10, 0); // 10%
        
        // Higher confidence with higher deviation
        let confidence_ratio = deviation / max_deviation;
        confidence_ratio.to_f64().unwrap_or(0.5).min(0.95)
    }
}

#[async_trait]
impl Strategy for MeanReversionStrategy {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn symbol(&self) -> &str {
        &self.symbol
    }
    
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    async fn analyze(&self, market_data: &MarketData) -> Result<Option<StrategySignal>> {
        if !self.enabled {
            return Ok(None);
        }
        
        // Update price history
        let mut strategy = self.clone();
        strategy.price_history.push(market_data.price);
        
        // Keep only recent prices
        if strategy.price_history.len() > self.lookback_period * 2 {
            strategy.price_history.drain(0..strategy.price_history.len() - self.lookback_period);
        }
        
        // Calculate moving average
        let ma = match strategy.calculate_moving_average() {
            Some(ma) => ma,
            None => return Ok(None), // Not enough data
        };
        
        let price = market_data.price;
        
        // Check for buy signal (price below MA with significant deviation)
        if strategy.should_buy(price, ma) {
            let confidence = strategy.calculate_confidence(price, ma);
            
            info!(
                "Mean Reversion BUY signal: {} at {} (MA: {}, deviation: {:.2}%)",
                self.symbol,
                price,
                ma,
                strategy.calculate_deviation(price, ma)
            );
            
            return Ok(Some(StrategySignal {
                strategy_name: self.name.clone(),
                symbol: self.symbol.clone(),
                action: SignalAction::Buy,
                quantity: self.position_size / price,
                price: Some(price),
                confidence,
                metadata: HashMap::from([
                    ("moving_average".to_string(), serde_json::Value::String(ma.to_string())),
                    ("deviation_percent".to_string(), serde_json::Value::String(
                        strategy.calculate_deviation(price, ma).to_string()
                    )),
                    ("lookback_period".to_string(), serde_json::Value::Number(self.lookback_period.into())),
                ]),
            }));
        }
        
        // Check for sell signal (price above MA with significant deviation)
        if strategy.should_sell(price, ma) {
            let confidence = strategy.calculate_confidence(price, ma);
            
            info!(
                "Mean Reversion SELL signal: {} at {} (MA: {}, deviation: {:.2}%)",
                self.symbol,
                price,
                ma,
                strategy.calculate_deviation(price, ma)
            );
            
            return Ok(Some(StrategySignal {
                strategy_name: self.name.clone(),
                symbol: self.symbol.clone(),
                action: SignalAction::Sell,
                quantity: self.position_size / price,
                price: Some(price),
                confidence,
                metadata: HashMap::from([
                    ("moving_average".to_string(), serde_json::Value::String(ma.to_string())),
                    ("deviation_percent".to_string(), serde_json::Value::String(
                        strategy.calculate_deviation(price, ma).to_string()
                    )),
                    ("lookback_period".to_string(), serde_json::Value::String(self.lookback_period.to_string())),
                ]),
            }));
        }
        
        Ok(None)
    }
    
    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> Result<()> {
        for (key, value) in parameters {
            match key.as_str() {
                "lookback_period" => {
                    if let Some(period) = value.as_u64() {
                        self.lookback_period = period as usize;
                    }
                }
                "deviation_threshold" => {
                    if let Some(threshold) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        self.deviation_threshold = threshold;
                    }
                }
                "position_size" => {
                    if let Some(size) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        self.position_size = size;
                    }
                }
                _ => {
                    info!("Unknown parameter: {}", key);
                }
            }
        }
        
        self.parameters = parameters;
        Ok(())
    }
    
    fn get_parameters(&self) -> HashMap<String, serde_json::Value> {
        self.parameters.clone()
    }
    
    fn validate_parameters(&self, parameters: &HashMap<String, serde_json::Value>) -> Result<()> {
        for (key, value) in parameters {
            match key.as_str() {
                "lookback_period" => {
                    if let Some(period) = value.as_u64() {
                        if period == 0 || period > 100 {
                            return Err(crate::error::Error::Strategy(
                                "Lookback period must be between 1 and 100".to_string()
                            ));
                        }
                    }
                }
                "deviation_threshold" => {
                    if let Some(threshold) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        if threshold <= Decimal::ZERO || threshold > Decimal::from(50) {
                            return Err(crate::error::Error::Strategy(
                                "Deviation threshold must be between 0 and 50".to_string()
                            ));
                        }
                    }
                }
                "position_size" => {
                    if let Some(size) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        if size <= Decimal::ZERO {
                            return Err(crate::error::Error::Strategy(
                                "Position size must be positive".to_string()
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        
        Ok(())
    }
}

// Implement Clone for MeanReversionStrategy
impl Clone for MeanReversionStrategy {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            enabled: self.enabled,
            parameters: self.parameters.clone(),
            lookback_period: self.lookback_period,
            deviation_threshold: self.deviation_threshold,
            position_size: self.position_size,
            price_history: self.price_history.clone(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logging
    hyperliquid_trading_bot::utils::setup_logging(false)?;
    
    info!("🎯 Custom Strategy Example: Mean Reversion");
    
    // Create custom strategy
    let mut strategy = MeanReversionStrategy::new(
        "mean_reversion_btc".to_string(),
        "BTC".to_string(),
    );
    
    // Configure strategy
    let mut parameters = HashMap::new();
    parameters.insert("lookback_period".to_string(), serde_json::Value::Number(30.into()));
    parameters.insert("deviation_threshold".to_string(), serde_json::Value::String("3.0".to_string()));
    parameters.insert("position_size".to_string(), serde_json::Value::String("200".to_string()));
    
    strategy.update_parameters(parameters).await?;
    
    info!("Custom strategy configured:");
    info!("  - Lookback period: 30");
    info!("  - Deviation threshold: 3%");
    info!("  - Position size: $200");
    
    // Simulate market data
    let market_data = MarketData {
        symbol: "BTC".to_string(),
        price: Decimal::from(45000),
        volume_24h: Decimal::from(1000000),
        change_24h: Decimal::from(-500),
        high_24h: Decimal::from(46000),
        low_24h: Decimal::from(44000),
        timestamp: chrono::Utc::now(),
    };
    
    // Analyze market data
    if let Some(signal) = strategy.analyze(&market_data).await? {
        info!("Signal generated: {:?}", signal.action);
        info!("Confidence: {:.2}", signal.confidence);
        info!("Quantity: {}", signal.quantity);
    } else {
        info!("No signal generated");
    }
    
    info!("✅ Custom strategy example completed!");
    
    Ok(())
}
