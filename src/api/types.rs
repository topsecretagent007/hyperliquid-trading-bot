use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperliquidResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketInfo {
    pub name: String,
    pub only_isolated: bool,
    pub sz_decimals: u32,
    pub wei_decimals: u32,
    pub is_inverse: bool,
    pub min_order_size: Decimal,
    pub max_leverage: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub name: String,
    pub sz_decimals: u32,
    pub wei_decimals: u32,
    pub only_isolated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub universe: Vec<AssetInfo>,
    pub amms: Vec<serde_json::Value>,
    pub open_interest: HashMap<String, Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L2Book {
    pub coin: String,
    pub levels: Vec<[Decimal; 2]>,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub coin: String,
    pub px: Decimal,
    pub sz: Decimal,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub t: u64,
    pub o: Decimal,
    pub h: Decimal,
    pub l: Decimal,
    pub c: Decimal,
    pub v: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserState {
    pub asset_positions: Vec<AssetPosition>,
    pub cross_margin_summary: Option<CrossMarginSummary>,
    pub margin_summary: Option<MarginSummary>,
    pub time: u64,
    pub withdrawable: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPosition {
    pub position: Position,
    pub type_: String,
    pub coin: String,
    pub pnl: Decimal,
    pub value: Decimal,
    pub entry_px: Decimal,
    pub leverage: Decimal,
    pub sz: Decimal,
    pub unrealized_pnl: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub coin: String,
    pub entry_px: Decimal,
    pub leverage: Decimal,
    pub liquidation_px: Option<Decimal>,
    pub margin_used: Decimal,
    pub max_leverage: Decimal,
    pub position_value: Decimal,
    pub return_on_equity: Decimal,
    pub szi: Decimal,
    pub unrealized_pnl: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossMarginSummary {
    pub account_value: Decimal,
    pub total_margin_used: Decimal,
    pub total_ntl_pos: Decimal,
    pub total_raw_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginSummary {
    pub account_value: Decimal,
    pub total_margin_used: Decimal,
    pub total_ntl_pos: Decimal,
    pub total_raw_usd: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub a: u32, // asset_id
    pub b: bool, // is_buy
    pub p: Decimal, // price
    pub s: Decimal, // size
    pub r: bool, // reduce_only
    pub t: String, // order_type
    pub cid: u64, // client_order_id
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub status: String,
    pub response: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelRequest {
    pub coin: String,
    pub oid: u64, // order_id
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    pub status: String,
    pub response: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub channel: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketSubscription {
    pub method: String,
    pub subscription: serde_json::Value,
}
