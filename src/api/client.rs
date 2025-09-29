use crate::{
    error::{Error, Result},
    models::{AccountInfo, MarketData, Order, OrderSide, OrderType, Position, PositionSide, Trade},
    utils::log_error_with_context,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use rust_decimal::Decimal;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};

use super::types::*;

pub struct HyperliquidClient {
    client: Client,
    base_url: String,
    api_key: String,
    private_key: String,
    testnet: bool,
}

impl HyperliquidClient {
    pub fn new(base_url: String, api_key: String, private_key: String, testnet: bool) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url,
            api_key,
            private_key,
            testnet,
        }
    }
    
    fn create_signature(&self, data: &str) -> Result<String> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        let mut mac = Hmac::<Sha256>::new_from_slice(self.private_key.as_bytes())
            .map_err(|e| Error::Api(format!("Invalid private key: {}", e)))?;
        
        mac.update(data.as_bytes());
        let result = mac.finalize();
        Ok(hex::encode(result.into_bytes()))
    }
    
    async fn make_request<T>(&self, endpoint: &str, data: Option<serde_json::Value>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        
        let mut request_builder = self.client.post(&url);
        
        if let Some(data) = data {
            let data_str = serde_json::to_string(&data)?;
            let signature = self.create_signature(&data_str)?;
            
            request_builder = request_builder
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("X-Signature", signature)
                .body(data_str);
        }
        
        let response = request_builder.send().await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::Api(format!("HTTP {}: {}", response.status(), error_text)));
        }
        
        let response_data: HyperliquidResponse<T> = response.json().await?;
        
        if !response_data.success {
            return Err(Error::Api(
                response_data.error.unwrap_or_else(|| "Unknown API error".to_string())
            ));
        }
        
        response_data.data.ok_or_else(|| Error::Api("No data in response".to_string()))
    }
}

#[async_trait]
pub trait TradingClient {
    async fn get_market_data(&self, symbol: &str) -> Result<MarketData>;
    async fn get_account_info(&self) -> Result<AccountInfo>;
    async fn get_positions(&self) -> Result<Vec<Position>>;
    async fn get_open_orders(&self) -> Result<Vec<Order>>;
    async fn place_order(&self, order: &Order) -> Result<String>;
    async fn cancel_order(&self, order_id: &str) -> Result<bool>;
    async fn get_trade_history(&self, symbol: Option<&str>) -> Result<Vec<Trade>>;
}

#[async_trait]
impl TradingClient for HyperliquidClient {
    async fn get_market_data(&self, symbol: &str) -> Result<MarketData> {
        debug!("Fetching market data for {}", symbol);
        
        let data = json!({
            "type": "allMids"
        });
        
        let response: HashMap<String, Decimal> = self.make_request("info", Some(data)).await?;
        
        let price = response.get(symbol)
            .ok_or_else(|| Error::Api(format!("Symbol {} not found", symbol)))?;
        
        // For now, return basic market data. In production, you'd want to fetch
        // more detailed data including volume, 24h change, etc.
        Ok(MarketData {
            symbol: symbol.to_string(),
            price: *price,
            volume_24h: Decimal::ZERO, // Would need separate API call
            change_24h: Decimal::ZERO, // Would need separate API call
            high_24h: *price, // Would need separate API call
            low_24h: *price, // Would need separate API call
            timestamp: Utc::now(),
        })
    }
    
    async fn get_account_info(&self) -> Result<AccountInfo> {
        debug!("Fetching account info");
        
        let data = json!({
            "type": "clearinghouseState",
            "user": self.api_key
        });
        
        let response: UserState = self.make_request("info", Some(data)).await?;
        
        let mut positions = Vec::new();
        for asset_pos in response.asset_positions {
            if asset_pos.sz != Decimal::ZERO {
                positions.push(Position {
                    symbol: asset_pos.coin,
                    side: if asset_pos.sz > Decimal::ZERO {
                        PositionSide::Long
                    } else {
                        PositionSide::Short
                    },
                    size: asset_pos.sz.abs(),
                    entry_price: asset_pos.entry_px,
                    current_price: Decimal::ZERO, // Would need separate call
                    unrealized_pnl: asset_pos.unrealized_pnl,
                    realized_pnl: Decimal::ZERO, // Would need separate call
                    margin: asset_pos.position.margin_used,
                    timestamp: Utc::now(),
                });
            }
        }
        
        let balance = response.withdrawable;
        let total_pnl = positions.iter()
            .map(|p| p.unrealized_pnl)
            .sum();
        
        Ok(AccountInfo {
            balance,
            available_balance: balance,
            total_pnl,
            total_margin: positions.iter()
                .map(|p| p.margin)
                .sum(),
            positions,
            open_orders: Vec::new(), // Would need separate call
        })
    }
    
    async fn get_positions(&self) -> Result<Vec<Position>> {
        let account_info = self.get_account_info().await?;
        Ok(account_info.positions)
    }
    
    async fn get_open_orders(&self) -> Result<Vec<Order>> {
        // This would require a separate API call to get open orders
        // For now, return empty vector
        Ok(Vec::new())
    }
    
    async fn place_order(&self, order: &Order) -> Result<String> {
        debug!("Placing order: {:?}", order);
        
        // Convert our Order model to Hyperliquid's format
        let order_request = OrderRequest {
            a: 0, // asset_id - would need to map symbol to asset_id
            b: matches!(order.side, OrderSide::Buy),
            p: order.price.unwrap_or(Decimal::ZERO),
            s: order.quantity,
            r: false, // reduce_only
            t: match order.order_type {
                OrderType::Market => "Market".to_string(),
                OrderType::Limit => "Limit".to_string(),
                _ => "Limit".to_string(),
            },
            cid: 0, // client_order_id - would generate unique ID
        };
        
        let data = json!({
            "action": {
                "type": "order",
                "orders": [order_request]
            }
        });
        
        let response: OrderResponse = self.make_request("exchange", Some(data)).await?;
        
        if response.status == "ok" {
            info!("Order placed successfully");
            Ok("order_id".to_string()) // Would return actual order ID
        } else {
            Err(Error::Trading(format!("Failed to place order: {}", response.status)))
        }
    }
    
    async fn cancel_order(&self, order_id: &str) -> Result<bool> {
        debug!("Cancelling order: {}", order_id);
        
        let cancel_request = CancelRequest {
            coin: "".to_string(), // Would need to map order_id to coin
            oid: order_id.parse().unwrap_or(0),
        };
        
        let data = json!({
            "action": {
                "type": "cancel",
                "cancels": [cancel_request]
            }
        });
        
        let response: CancelResponse = self.make_request("exchange", Some(data)).await?;
        
        Ok(response.status == "ok")
    }
    
    async fn get_trade_history(&self, _symbol: Option<&str>) -> Result<Vec<Trade>> {
        // This would require a separate API call to get trade history
        // For now, return empty vector
        Ok(Vec::new())
    }
}
