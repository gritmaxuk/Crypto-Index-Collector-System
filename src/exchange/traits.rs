use async_trait::async_trait;
use crate::error::AppResult;

/// Trait for cryptocurrency exchange APIs
#[async_trait]
pub trait Exchange: Send + Sync {
    /// Fetch the current price for a symbol
    async fn fetch_price(&self, symbol: &str) -> AppResult<f64>;
}
