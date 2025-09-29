use crate::{
    error::Result,
    models::{MarketData, StrategySignal, SignalAction},
    strategies::base::{Strategy, calculate_sma, calculate_ema, calculate_rsi, calculate_macd},
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tracing::{debug, info};

pub struct MomentumStrategy {
    name: String,
    symbol: String,
    enabled: bool,
    parameters: HashMap<String, serde_json::Value>,
    
    // Momentum specific parameters
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    rsi_period: usize,
    rsi_oversold: Decimal,
    rsi_overbought: Decimal,
    price_history: Vec<Decimal>,
    volume_history: Vec<Decimal>,
    min_confidence: f64,
}

impl MomentumStrategy {
    pub fn new(name: String, symbol: String) -> Self {
        Self {
            name,
            symbol,
            enabled: true,
            parameters: HashMap::new(),
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            rsi_period: 14,
            rsi_oversold: Decimal::from(30),
            rsi_overbought: Decimal::from(70),
            price_history: Vec::new(),
            volume_history: Vec::new(),
            min_confidence: 0.6,
        }
    }
    
    fn update_history(&mut self, market_data: &MarketData) {
        self.price_history.push(market_data.price);
        self.volume_history.push(market_data.volume_24h);
        
        // Keep only recent data to avoid memory growth
        let max_history = self.slow_period * 2;
        if self.price_history.len() > max_history {
            self.price_history.drain(0..self.price_history.len() - max_history);
        }
        if self.volume_history.len() > max_history {
            self.volume_history.drain(0..self.volume_history.len() - max_history);
        }
    }
    
    fn analyze_momentum(&self) -> Option<(SignalAction, f64)> {
        if self.price_history.len() < self.slow_period {
            return None;
        }
        
        // Calculate MACD
        let (macd_line, signal_line, histogram) = calculate_macd(
            &self.price_history,
            self.fast_period,
            self.slow_period,
            self.signal_period,
        )?;
        
        // Calculate RSI
        let rsi = calculate_rsi(&self.price_history, self.rsi_period)?;
        
        // Calculate moving averages
        let fast_sma = calculate_sma(&self.price_history, self.fast_period)?;
        let slow_sma = calculate_sma(&self.price_history, self.slow_period)?;
        
        // Momentum signals
        let mut signals = Vec::new();
        let mut confidence = 0.0;
        
        // MACD bullish crossover
        if macd_line > signal_line && histogram > Decimal::ZERO {
            signals.push("MACD_BULLISH");
            confidence += 0.3;
        }
        
        // MACD bearish crossover
        if macd_line < signal_line && histogram < Decimal::ZERO {
            signals.push("MACD_BEARISH");
            confidence += 0.3;
        }
        
        // RSI oversold (potential buy)
        if rsi < self.rsi_oversold {
            signals.push("RSI_OVERSOLD");
            confidence += 0.2;
        }
        
        // RSI overbought (potential sell)
        if rsi > self.rsi_overbought {
            signals.push("RSI_OVERBOUGHT");
            confidence += 0.2;
        }
        
        // Price above/below moving averages
        let current_price = self.price_history.last().unwrap();
        if current_price > fast_sma && fast_sma > slow_sma {
            signals.push("PRICE_ABOVE_MA");
            confidence += 0.2;
        } else if current_price < fast_sma && fast_sma < slow_sma {
            signals.push("PRICE_BELOW_MA");
            confidence += 0.2;
        }
        
        // Volume confirmation
        if self.volume_history.len() >= 2 {
            let current_volume = self.volume_history.last().unwrap();
            let avg_volume = self.volume_history.iter().sum::<Decimal>() / Decimal::from(self.volume_history.len());
            
            if current_volume > avg_volume * Decimal::new(15, 1) { // 1.5x average volume
                signals.push("HIGH_VOLUME");
                confidence += 0.1;
            }
        }
        
        // Determine action based on signals
        let bullish_signals = signals.iter().filter(|s| s.contains("BULLISH") || s.contains("OVERSOLD") || s.contains("ABOVE")).count();
        let bearish_signals = signals.iter().filter(|s| s.contains("BEARISH") || s.contains("OVERBOUGHT") || s.contains("BELOW")).count();
        
        if confidence >= self.min_confidence {
            if bullish_signals > bearish_signals {
                Some((SignalAction::Buy, confidence))
            } else if bearish_signals > bullish_signals {
                Some((SignalAction::Sell, confidence))
            } else {
                None
            }
        } else {
            None
        }
    }
    
    fn calculate_position_size(&self, market_data: &MarketData, confidence: f64) -> Decimal {
        // Base position size scaled by confidence
        let base_size = Decimal::from(100); // $100 base
        let confidence_multiplier = Decimal::from_f64_retain(confidence).unwrap_or(Decimal::ONE);
        base_size * confidence_multiplier / market_data.price
    }
}

#[async_trait]
impl Strategy for MomentumStrategy {
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
        
        debug!("Momentum analyzing {} at price {}", self.symbol, market_data.price);
        
        // Update price history
        let mut strategy = self.clone();
        strategy.update_history(market_data);
        
        if let Some((action, confidence)) = strategy.analyze_momentum() {
            let quantity = strategy.calculate_position_size(market_data, confidence);
            
            info!(
                "Momentum signal: {:?} {} at {} (confidence: {:.2})",
                action,
                self.symbol,
                market_data.price,
                confidence
            );
            
            Ok(Some(StrategySignal {
                strategy_name: self.name.clone(),
                symbol: self.symbol.clone(),
                action,
                quantity,
                price: Some(market_data.price),
                confidence,
                metadata: HashMap::from([
                    ("fast_period".to_string(), serde_json::Value::Number(self.fast_period.into())),
                    ("slow_period".to_string(), serde_json::Value::Number(self.slow_period.into())),
                    ("rsi_period".to_string(), serde_json::Value::Number(self.rsi_period.into())),
                    ("signals".to_string(), serde_json::Value::String(format!("{:?}", strategy.analyze_momentum()))),
                ]),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn update_parameters(&mut self, parameters: HashMap<String, serde_json::Value>) -> Result<()> {
        for (key, value) in parameters {
            match key.as_str() {
                "fast_period" => {
                    if let Some(period) = value.as_u64() {
                        self.fast_period = period as usize;
                    }
                }
                "slow_period" => {
                    if let Some(period) = value.as_u64() {
                        self.slow_period = period as usize;
                    }
                }
                "signal_period" => {
                    if let Some(period) = value.as_u64() {
                        self.signal_period = period as usize;
                    }
                }
                "rsi_period" => {
                    if let Some(period) = value.as_u64() {
                        self.rsi_period = period as usize;
                    }
                }
                "rsi_oversold" => {
                    if let Some(level) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        self.rsi_oversold = level;
                    }
                }
                "rsi_overbought" => {
                    if let Some(level) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        self.rsi_overbought = level;
                    }
                }
                "min_confidence" => {
                    if let Some(conf) = value.as_f64() {
                        self.min_confidence = conf;
                    }
                }
                _ => {
                    debug!("Unknown Momentum parameter: {}", key);
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
                "fast_period" | "slow_period" | "signal_period" | "rsi_period" => {
                    if let Some(period) = value.as_u64() {
                        if period == 0 || period > 100 {
                            return Err(crate::error::Error::Strategy(
                                format!("{} must be between 1 and 100", key)
                            ));
                        }
                    }
                }
                "rsi_oversold" | "rsi_overbought" => {
                    if let Some(level) = value.as_str().and_then(|s| s.parse::<Decimal>().ok()) {
                        if level < Decimal::ZERO || level > Decimal::from(100) {
                            return Err(crate::error::Error::Strategy(
                                format!("{} must be between 0 and 100", key)
                            ));
                        }
                    }
                }
                "min_confidence" => {
                    if let Some(conf) = value.as_f64() {
                        if conf < 0.0 || conf > 1.0 {
                            return Err(crate::error::Error::Strategy(
                                "Min confidence must be between 0 and 1".to_string()
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

// Implement Clone for MomentumStrategy
impl Clone for MomentumStrategy {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            symbol: self.symbol.clone(),
            enabled: self.enabled,
            parameters: self.parameters.clone(),
            fast_period: self.fast_period,
            slow_period: self.slow_period,
            signal_period: self.signal_period,
            rsi_period: self.rsi_period,
            rsi_oversold: self.rsi_oversold,
            rsi_overbought: self.rsi_overbought,
            price_history: self.price_history.clone(),
            volume_history: self.volume_history.clone(),
            min_confidence: self.min_confidence,
        }
    }
}
