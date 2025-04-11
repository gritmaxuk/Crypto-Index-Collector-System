// Re-export PriceFeed from models
pub use crate::models::PriceFeed;

// Modules
pub mod coinbase;
pub mod binance;
pub mod traits;

// Re-export the Exchange trait
pub use traits::Exchange;

// Factory function to create exchange instances
pub fn create_exchange(name: &str) -> Option<Box<dyn Exchange>> {
    match name.to_lowercase().as_str() {
        "coinbase" => Some(Box::new(coinbase::CoinbaseExchange::new())),
        "binance" => Some(Box::new(binance::BinanceExchange::new())),
        _ => None,
    }
}