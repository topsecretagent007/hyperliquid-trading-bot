use crate::{
    error::Result,
    models::{MarketData, StrategySignal, SignalAction},
    strategies::base::Strategy,
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::{debug, info};

pub struct DCAStrategy {
    name: String,
    symbol: String,
    enabled: bool,
    parameters: HashMap<String, serde_json::Value>,
    
    // DCA specific parameters
    investment_amount: Decimal,
    interval_hours: u64,
    last_buy_time: Option<chrono::DateTime<chrono::Utc>>,
    max_investment: Decimal,
    current_investment: Decimal,
    price_history: Vec<Decimal>,
    lookback_period: usize,
}

impl DCAStrategy {
    pub fn new(name: String, symbol: String) -> Self {
        Self {
            name,
            symbol,
            enabled: true,
            parameters: HashMap::new(),
            investment_amount: Decimal::from(100), // $100 per interval
            interval_hours: 24, // Daily
            last_buy_time: None,
            max_investment: Decimal::from(10000), // $10,000 max
            current_investment: Decimal::ZERO,
            price_history: Vec::new(),
            lookback_period: 20,
        }
    }
    
    fn should_buy(&self, market_data: &MarketData) -> bool {
        // Check if enough time has passed since last buy
        if let Some(last_buy) = self.last_buy_time {
            let time_since_last = chrono::Utc::now() - last_buy;
            if time_since_last.num_hours() < self.interval_hours as i64 {
                return false;
            }
        }
        
        // Check if we haven't exceeded max investment
        if self.current_investment >= self.max_investment {
            debug!("DCA: Max investment reached for {}", self.symbol);
            return false;
        }
        
        // Check if we have enough price history for analysis
        if self.price_history.len() < self.lookback_period {
            return true; // Buy on first few intervals
        }
        
        // Simple trend analysis - buy if price is below recent average
        let recent_avg = self.calculate_recent_average();
        if let Some(avg) = recent_avg {
            market_data.price < avg
        } else {
            true
        }
    }
    
    fn calculate_recent_average(&self) -> Option<Decimal> {
        if self.price_history.len() < self.lookback_period {
            return None;
        }
        
        let recent_prices = &self.price_history[self.price_history.len() - self.lookback_period..];
        let sum: Decimal = recent_prices.iter().sum();
        Some(sum / Decimal::from(recent_prices.len()))
    }
    
    fn calculate_confidence(&self, market_data: &MarketData) -> f64 {
        if self.price_history.len() < self.lookback_period {
            return 0.5; // Medium confidence for early buys
        }
        
        let recent_avg = self.calculate_recent_average().unwrap_or(market_data.price);
        let price_ratio = market_data.price / recent_avg;
        
        // Higher confidence when price is significantly below average
        if price_ratio < Decimal::new(95, 2) { // 5% below average
            0.8
        } else if price_ratio < Decimal::new(98, 2) { // 2% below average
            0.6
        } else {
            0.4
        }
    }
}

#[async_trait]
impl Strategy for DCAStrategy {
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
        
        debug!("DCA analyzing {} at price {}", self.symbol, market_data.price);
        
        if self.should_buy(market_data) {
            let confidence = self.calculate_confidence(market_data);
            
            info!(
                "DCA signal: BUY {} at {} (confidence: {:.2})",
                self.symbol,
                market_data.price,
                confidence
            );
            
            Ok(Some(StrategySignal {
                strategy_name: self.name.clone(),
                symbol: self.symbol.clone(),
                action: SignalAction::Buy,
                quantity: self.investment_amount / market_data.price,
                price: Some(market_data.price),
                confidence,
                metadata: HashMap::from([
                    ("investment_amount".to_string(), serde_json::Value::String(self.investment_amount.to_string())),
                    ("interval_hours".to_string(), serde_json::Value::Number(self.interval_hours.into())),
                    ("current_investment".to_string(), serde_json::Value::String(self.current_investment.to_string())),
                ]),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> Result<()> {
        for (key, value) in parameters {
            match key.as_str() {
                "investment_amount" => {
                    if let Some(amount) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        self.investment_amount = amount;
                    }
                }
                "interval_hours" => {
                    if let Some(hours) = value.as_u64() {
                        self.interval_hours = hours;
                    }
                }
                "max_investment" => {
                    if let Some(max) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        self.max_investment = max;
                    }
                }
                "lookback_period" => {
                    if let Some(period) = value.as_u64() {
                        self.lookback_period = period as usize;
                    }
                }
                _ => {
                    debug!("Unknown DCA parameter: {}", key);
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
                "investment_amount" => {
                    if let Some(amount) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        if amount <= Decimal::ZERO {
                            return Err(crate::error::Error::Strategy(
                                "Investment amount must be positive".to_string()
                            ));
                        }
                    }
                }
                "interval_hours" => {
                    if let Some(hours) = value.as_u64() {
                        if hours == 0 {
                            return Err(crate::error::Error::Strategy(
                                "Interval hours must be greater than 0".to_string()
                            ));
                        }
                    }
                }
                "max_investment" => {
                    if let Some(max) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        if max <= Decimal::ZERO {
                            return Err(crate::error::Error::Strategy(
                                "Max investment must be positive".to_string()
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

impl DCAStrategy {
    pub fn update_price_history(&mut self, price: Decimal) {
        self.price_history.push(price);
        
        // Keep only recent prices to avoid memory growth
        if self.price_history.len() > self.lookback_period * 2 {
            self.price_history.drain(0..self.price_history.len() - self.lookback_period);
        }
    }
    
    pub fn record_buy(&mut self, amount: Decimal) {
        self.last_buy_time = Some(chrono::Utc::now());
        self.current_investment += amount;
    }
    
    pub fn reset_investment(&mut self) {
        self.current_investment = Decimal::ZERO;
        self.last_buy_time = None;
    }
}
