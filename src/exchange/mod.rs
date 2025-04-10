use async_trait::async_trait;
use std::error::Error;

// Re-export PriceFeed from models
pub use crate::models::PriceFeed;

// Re-export submodules
pub mod coinbase;
pub mod binance;

#[async_trait]
pub trait Exchange {
    async fn fetch_price(&self, symbol: &str) -> Result<f64, Box<dyn Error + Send + Sync>>;
}