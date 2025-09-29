use crate::{
    error::Result,
    models::{MarketData, StrategySignal},
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use std::collections::HashMap;

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

pub struct StrategyConfig {
    pub name: String,
    pub symbol: String,
    pub enabled: bool,
    pub parameters: HashMap<String, serde_json::Value>,
}

impl StrategyConfig {
    pub fn new(name: String, symbol: String) -> Self {
        Self {
            name,
            symbol,
            enabled: true,
            parameters: HashMap::new(),
        }
    }
    
    pub fn with_parameter(mut self, key: &str, value: serde_json::Value) -> Self {
        self.parameters.insert(key.to_string(), value);
        self
    }
    
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

pub fn calculate_sma(prices: &[Decimal], period: usize) -> Option<Decimal> {
    if prices.len() < period {
        return None;
    }
    
    let sum: Decimal = prices.iter().rev().take(period).sum();
    Some(sum / Decimal::from(period))
}

pub fn calculate_ema(prices: &[Decimal], period: usize, alpha: Option<Decimal>) -> Option<Decimal> {
    if prices.is_empty() {
        return None;
    }
    
    let alpha = alpha.unwrap_or_else(|| Decimal::from(2) / (Decimal::from(period) + Decimal::from(1)));
    let mut ema = prices[0];
    
    for &price in prices.iter().skip(1) {
        ema = alpha * price + (Decimal::from(1) - alpha) * ema;
    }
    
    Some(ema)
}

pub fn calculate_rsi(prices: &[Decimal], period: usize) -> Option<Decimal> {
    if prices.len() < period + 1 {
        return None;
    }
    
    let mut gains = Vec::new();
    let mut losses = Vec::new();
    
    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > Decimal::ZERO {
            gains.push(change);
            losses.push(Decimal::ZERO);
        } else {
            gains.push(Decimal::ZERO);
            losses.push(-change);
        }
    }
    
    if gains.len() < period {
        return None;
    }
    
    let avg_gain = gains.iter().rev().take(period).sum::<Decimal>() / Decimal::from(period);
    let avg_loss = losses.iter().rev().take(period).sum::<Decimal>() / Decimal::from(period);
    
    if avg_loss == Decimal::ZERO {
        return Some(Decimal::from(100));
    }
    
    let rs = avg_gain / avg_loss;
    let rsi = Decimal::from(100) - (Decimal::from(100) / (Decimal::from(1) + rs));
    
    Some(rsi)
}

pub fn calculate_bollinger_bands(
    prices: &[Decimal],
    period: usize,
    std_dev: Decimal,
) -> Option<(Decimal, Decimal, Decimal)> {
    if prices.len() < period {
        return None;
    }
    
    let sma = calculate_sma(prices, period)?;
    let recent_prices = &prices[prices.len() - period..];
    
    let variance = recent_prices
        .iter()
        .map(|&price| (price - sma).powi(2))
        .sum::<Decimal>()
        / Decimal::from(period);
    
    let std_deviation = variance.sqrt().unwrap_or(Decimal::ZERO);
    
    let upper_band = sma + (std_deviation * std_dev);
    let lower_band = sma - (std_deviation * std_dev);
    
    Some((upper_band, sma, lower_band))
}

pub fn calculate_macd(
    prices: &[Decimal],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> Option<(Decimal, Decimal, Decimal)> {
    if prices.len() < slow_period {
        return None;
    }
    
    let fast_ema = calculate_ema(prices, fast_period, None)?;
    let slow_ema = calculate_ema(prices, slow_period, None)?;
    let macd_line = fast_ema - slow_ema;
    
    // For signal line, we'd need to calculate EMA of MACD line
    // This is simplified - in practice, you'd maintain MACD history
    let signal_line = macd_line; // Simplified
    let histogram = macd_line - signal_line;
    
    Some((macd_line, signal_line, histogram))
}

pub fn is_oversold(rsi: Decimal) -> bool {
    rsi < Decimal::from(30)
}

pub fn is_overbought(rsi: Decimal) -> bool {
    rsi > Decimal::from(70)
}

pub fn is_bullish_divergence(prices: &[Decimal], rsi_values: &[Decimal]) -> bool {
    if prices.len() < 2 || rsi_values.len() < 2 {
        return false;
    }
    
    let price_trend = prices[prices.len() - 1] > prices[prices.len() - 2];
    let rsi_trend = rsi_values[rsi_values.len() - 1] < rsi_values[rsi_values.len() - 2];
    
    price_trend && rsi_trend
}

pub fn is_bearish_divergence(prices: &[Decimal], rsi_values: &[Decimal]) -> bool {
    if prices.len() < 2 || rsi_values.len() < 2 {
        return false;
    }
    
    let price_trend = prices[prices.len() - 1] < prices[prices.len() - 2];
    let rsi_trend = rsi_values[rsi_values.len() - 1] > rsi_values[rsi_values.len() - 2];
    
    price_trend && rsi_trend
}
