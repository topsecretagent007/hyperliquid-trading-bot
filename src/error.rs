use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Trading error: {0}")]
    Trading(String),
    
    #[error("Strategy error: {0}")]
    Strategy(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Order not found")]
    OrderNotFound,
    
    #[error("Market closed")]
    MarketClosed,
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;
