use crate::{
    error::Result,
    models::{MarketData, StrategySignal, SignalAction},
    strategies::base::Strategy,
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::{debug, info};

pub struct GridStrategy {
    name: String,
    symbol: String,
    enabled: bool,
    parameters: HashMap<String, serde_json::Value>,
    
    // Grid specific parameters
    grid_levels: Vec<Decimal>,
    grid_spacing: Decimal, // Percentage between levels
    base_price: Option<Decimal>,
    position_size: Decimal,
    max_levels: usize,
    active_orders: HashMap<Decimal, bool>, // price -> is_buy_order
    total_investment: Decimal,
    max_investment: Decimal,
}

impl GridStrategy {
    pub fn new(name: String, symbol: String) -> Self {
        Self {
            name,
            symbol,
            enabled: true,
            parameters: HashMap::new(),
            grid_levels: Vec::new(),
            grid_spacing: Decimal::new(1, 0), // 1% spacing
            base_price: None,
            position_size: Decimal::from(100), // $100 per grid level
            max_levels: 10,
            active_orders: HashMap::new(),
            total_investment: Decimal::ZERO,
            max_investment: Decimal::from(5000), // $5000 max
        }
    }
    
    fn initialize_grid(&mut self, base_price: Decimal) {
        self.base_price = Some(base_price);
        self.grid_levels.clear();
        self.active_orders.clear();
        
        // Create buy levels below base price
        for i in 1..=self.max_levels {
            let level = base_price * (Decimal::from(1) - (self.grid_spacing * Decimal::from(i)) / Decimal::from(100));
            self.grid_levels.push(level);
            self.active_orders.insert(level, true); // Buy order
        }
        
        // Create sell levels above base price
        for i in 1..=self.max_levels {
            let level = base_price * (Decimal::from(1) + (self.grid_spacing * Decimal::from(i)) / Decimal::from(100));
            self.grid_levels.push(level);
            self.active_orders.insert(level, false); // Sell order
        }
        
        info!(
            "Grid initialized for {} with {} levels around {}",
            self.symbol,
            self.grid_levels.len(),
            base_price
        );
    }
    
    fn should_place_buy_order(&self, market_data: &MarketData) -> Option<Decimal> {
        if self.total_investment >= self.max_investment {
            return None;
        }
        
        // Find the highest buy level that's above current price
        for &level in &self.grid_levels {
            if level > market_data.price && self.active_orders.get(&level) == Some(&true) {
                return Some(level);
            }
        }
        
        None
    }
    
    fn should_place_sell_order(&self, market_data: &MarketData) -> Option<Decimal> {
        // Find the lowest sell level that's below current price
        for &level in &self.grid_levels {
            if level < market_data.price && self.active_orders.get(&level) == Some(&false) {
                return Some(level);
            }
        }
        
        None
    }
    
    fn calculate_confidence(&self, action: &SignalAction, price: Decimal) -> f64 {
        match action {
            SignalAction::Buy => {
                // Higher confidence when price is further below base price
                if let Some(base) = self.base_price {
                    let deviation = (base - price) / base * Decimal::from(100);
                    if deviation > Decimal::from(5) {
                        0.9
                    } else if deviation > Decimal::from(2) {
                        0.7
                    } else {
                        0.5
                    }
                } else {
                    0.5
                }
            }
            SignalAction::Sell => {
                // Higher confidence when price is further above base price
                if let Some(base) = self.base_price {
                    let deviation = (price - base) / base * Decimal::from(100);
                    if deviation > Decimal::from(5) {
                        0.9
                    } else if deviation > Decimal::from(2) {
                        0.7
                    } else {
                        0.5
                    }
                } else {
                    0.5
                }
            }
            _ => 0.5,
        }
    }
}

#[async_trait]
impl Strategy for GridStrategy {
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
        
        debug!("Grid analyzing {} at price {}", self.symbol, market_data.price);
        
        // Initialize grid if not done yet
        if self.base_price.is_none() {
            // This would need to be handled by the strategy manager
            // For now, we'll skip analysis until grid is initialized
            return Ok(None);
        }
        
        // Check for buy opportunities
        if let Some(buy_price) = self.should_place_buy_order(market_data) {
            let confidence = self.calculate_confidence(&SignalAction::Buy, buy_price);
            
            info!(
                "Grid signal: BUY {} at {} (confidence: {:.2})",
                self.symbol,
                buy_price,
                confidence
            );
            
            return Ok(Some(StrategySignal {
                strategy_name: self.name.clone(),
                symbol: self.symbol.clone(),
                action: SignalAction::Buy,
                quantity: self.position_size / buy_price,
                price: Some(buy_price),
                confidence,
                metadata: HashMap::from([
                    ("grid_level".to_string(), serde_json::Value::String(buy_price.to_string())),
                    ("position_size".to_string(), serde_json::Value::String(self.position_size.to_string())),
                    ("total_investment".to_string(), serde_json::Value::String(self.total_investment.to_string())),
                ]),
            }));
        }
        
        // Check for sell opportunities
        if let Some(sell_price) = self.should_place_sell_order(market_data) {
            let confidence = self.calculate_confidence(&SignalAction::Sell, sell_price);
            
            info!(
                "Grid signal: SELL {} at {} (confidence: {:.2})",
                self.symbol,
                sell_price,
                confidence
            );
            
            return Ok(Some(StrategySignal {
                strategy_name: self.name.clone(),
                symbol: self.symbol.clone(),
                action: SignalAction::Sell,
                quantity: self.position_size / sell_price,
                price: Some(sell_price),
                confidence,
                metadata: HashMap::from([
                    ("grid_level".to_string(), serde_json::Value::String(sell_price.to_string())),
                    ("position_size".to_string(), serde_json::Value::String(self.position_size.to_string())),
                    ("total_investment".to_string(), serde_json::Value::String(self.total_investment.to_string())),
                ]),
            }));
        }
        
        Ok(None)
    }
    
    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> Result<()> {
        for (key, value) in parameters {
            match key.as_str() {
                "grid_spacing" => {
                    if let Some(spacing) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        self.grid_spacing = spacing;
                    }
                }
                "position_size" => {
                    if let Some(size) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        self.position_size = size;
                    }
                }
                "max_levels" => {
                    if let Some(levels) = value.as_u64() {
                        self.max_levels = levels as usize;
                    }
                }
                "max_investment" => {
                    if let Some(max) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        self.max_investment = max;
                    }
                }
                _ => {
                    debug!("Unknown Grid parameter: {}", key);
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
                "grid_spacing" => {
                    if let Some(spacing) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        if spacing <= Decimal::ZERO || spacing > Decimal::from(50) {
                            return Err(crate::error::Error::Strategy(
                                "Grid spacing must be between 0 and 50".to_string()
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
                "max_levels" => {
                    if let Some(levels) = value.as_u64() {
                        if levels == 0 || levels > 50 {
                            return Err(crate::error::Error::Strategy(
                                "Max levels must be between 1 and 50".to_string()
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

impl GridStrategy {
    pub fn initialize_with_price(&mut self, price: Decimal) {
        self.initialize_grid(price);
    }
    
    pub fn mark_order_filled(&mut self, price: Decimal, is_buy: bool) {
        if let Some(&was_buy) = self.active_orders.get(&price) {
            if was_buy == is_buy {
                self.active_orders.remove(&price);
                if is_buy {
                    self.total_investment += self.position_size;
                }
            }
        }
    }
    
    pub fn reset_grid(&mut self) {
        self.base_price = None;
        self.grid_levels.clear();
        self.active_orders.clear();
        self.total_investment = Decimal::ZERO;
    }
    
    pub fn get_active_orders(&self) -> &HashMap<Decimal, bool> {
        &self.active_orders
    }
}
