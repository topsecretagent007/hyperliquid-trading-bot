use crate::error::Result;
use rust_decimal::Decimal;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn setup_logging(debug: bool) -> Result<()> {
    let level = if debug { "debug" } else { "info" };
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    Ok(())
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn format_decimal(value: Decimal, precision: u32) -> String {
    format!("{:.prec$}", value, prec = precision as usize)
}

pub fn calculate_percentage_change(old_value: Decimal, new_value: Decimal) -> Decimal {
    if old_value.is_zero() {
        return Decimal::ZERO;
    }
    
    (new_value - old_value) / old_value * Decimal::from(100)
}

pub fn calculate_position_size(
    account_balance: Decimal,
    risk_percentage: Decimal,
    entry_price: Decimal,
    stop_loss_price: Decimal,
) -> Decimal {
    let risk_amount = account_balance * (risk_percentage / Decimal::from(100));
    let price_difference = (entry_price - stop_loss_price).abs();
    
    if price_difference.is_zero() {
        return Decimal::ZERO;
    }
    
    risk_amount / price_difference
}

pub fn calculate_pnl(
    entry_price: Decimal,
    current_price: Decimal,
    quantity: Decimal,
    side: crate::models::PositionSide,
) -> Decimal {
    let price_diff = match side {
        crate::models::PositionSide::Long => current_price - entry_price,
        crate::models::PositionSide::Short => entry_price - current_price,
    };
    
    price_diff * quantity
}

pub fn is_market_hours() -> bool {
    // Simple check - in production, you'd want to check actual market hours
    // and holidays for the specific exchange
    let now = chrono::Utc::now();
    let hour = now.hour();
    
    // Assume market is open 24/7 for crypto, but you can add specific hours here
    true
}

pub fn sleep_ms(ms: u64) -> tokio::time::Sleep {
    tokio::time::sleep(Duration::from_millis(ms))
}

pub fn sleep_seconds(seconds: u64) -> tokio::time::Sleep {
    tokio::time::sleep(Duration::from_secs(seconds))
}

pub fn log_trade_execution(
    symbol: &str,
    side: &crate::models::OrderSide,
    quantity: Decimal,
    price: Decimal,
    success: bool,
) {
    let action = match side {
        crate::models::OrderSide::Buy => "BUY",
        crate::models::OrderSide::Sell => "SELL",
    };
    
    let status = if success { "✅" } else { "❌" };
    
    info!(
        "{} {} {} {} @ {}",
        status,
        action,
        format_decimal(quantity, 6),
        symbol,
        format_decimal(price, 2)
    );
}

pub fn log_position_update(
    symbol: &str,
    side: &crate::models::PositionSide,
    size: Decimal,
    pnl: Decimal,
) {
    let position_type = match side {
        crate::models::PositionSide::Long => "LONG",
        crate::models::PositionSide::Short => "SHORT",
    };
    
    let pnl_emoji = if pnl >= Decimal::ZERO { "📈" } else { "📉" };
    
    info!(
        "{} {} {} {} PnL: {}",
        pnl_emoji,
        position_type,
        format_decimal(size, 6),
        symbol,
        format_decimal(pnl, 2)
    );
}

pub fn validate_symbol(symbol: &str) -> bool {
    // Basic validation - in production, you'd want to check against
    // the actual list of supported symbols from the exchange
    !symbol.is_empty() && symbol.len() <= 20 && symbol.chars().all(|c| c.is_alphanumeric() || c == '-')
}

pub fn normalize_symbol(symbol: &str) -> String {
    symbol.to_uppercase().replace("-", "")
}

pub fn calculate_slippage(expected_price: Decimal, actual_price: Decimal) -> Decimal {
    if expected_price.is_zero() {
        return Decimal::ZERO;
    }
    
    ((actual_price - expected_price) / expected_price * Decimal::from(100)).abs()
}

pub fn is_slippage_acceptable(slippage: Decimal, max_slippage: Decimal) -> bool {
    slippage <= max_slippage
}

pub fn format_currency(amount: Decimal) -> String {
    format!("${:.2}", amount)
}

pub fn format_percentage(percentage: Decimal) -> String {
    format!("{:.2}%", percentage)
}

pub fn log_error_with_context(error: &crate::error::Error, context: &str) {
    error!("{}: {}", context, error);
}

pub fn log_warning_with_context(message: &str, context: &str) {
    warn!("{}: {}", context, message);
}
