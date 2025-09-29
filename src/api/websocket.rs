use crate::error::{Error, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

use super::types::*;

pub struct WebSocketClient {
    ws_url: String,
    sender: Option<mpsc::UnboundedSender<Message>>,
    receiver: Option<mpsc::UnboundedReceiver<Message>>,
}

impl WebSocketClient {
    pub fn new(ws_url: String) -> Self {
        Self {
            ws_url,
            sender: None,
            receiver: None,
        }
    }
    
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to WebSocket: {}", self.ws_url);
        
        let (ws_stream, _) = connect_async(&self.ws_url).await?;
        let (mut write, read) = ws_stream.split();
        
        let (tx, rx) = mpsc::unbounded_channel();
        self.sender = Some(tx);
        self.receiver = Some(rx);
        
        // Spawn task to handle incoming messages
        let read_task = tokio::spawn(async move {
            let mut read = read;
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        debug!("Received WebSocket message: {}", text);
                        // Handle incoming messages here
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket connection closed");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        // Spawn task to handle outgoing messages
        let write_task = tokio::spawn(async move {
            let mut write = write;
            let mut rx = self.receiver.take().unwrap();
            
            while let Some(msg) = rx.recv().await {
                if let Err(e) = write.send(msg).await {
                    error!("Failed to send WebSocket message: {}", e);
                    break;
                }
            }
        });
        
        info!("WebSocket connected successfully");
        Ok(())
    }
    
    pub async fn subscribe_to_ticker(&self, symbol: &str) -> Result<()> {
        if let Some(sender) = &self.sender {
            let subscription = json!({
                "method": "subscribe",
                "subscription": {
                    "type": "ticker",
                    "coin": symbol
                }
            });
            
            let message = Message::Text(serde_json::to_string(&subscription)?);
            sender.send(message).map_err(|e| Error::WebSocket(e.into()))?;
            
            info!("Subscribed to ticker for {}", symbol);
        }
        
        Ok(())
    }
    
    pub async fn subscribe_to_l2_book(&self, symbol: &str) -> Result<()> {
        if let Some(sender) = &self.sender {
            let subscription = json!({
                "method": "subscribe",
                "subscription": {
                    "type": "l2Book",
                    "coin": symbol
                }
            });
            
            let message = Message::Text(serde_json::to_string(&subscription)?);
            sender.send(message).map_err(|e| Error::WebSocket(e.into()))?;
            
            info!("Subscribed to L2 book for {}", symbol);
        }
        
        Ok(())
    }
    
    pub async fn subscribe_to_candles(&self, symbol: &str, interval: &str) -> Result<()> {
        if let Some(sender) = &self.sender {
            let subscription = json!({
                "method": "subscribe",
                "subscription": {
                    "type": "candle",
                    "coin": symbol,
                    "interval": interval
                }
            });
            
            let message = Message::Text(serde_json::to_string(&subscription)?);
            sender.send(message).map_err(|e| Error::WebSocket(e.into()))?;
            
            info!("Subscribed to candles for {} ({})", symbol, interval);
        }
        
        Ok(())
    }
    
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(sender) = &self.sender {
            let close_message = Message::Close(None);
            sender.send(close_message).map_err(|e| Error::WebSocket(e.into()))?;
        }
        
        info!("WebSocket disconnected");
        Ok(())
    }
}
