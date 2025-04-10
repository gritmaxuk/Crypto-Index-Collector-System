use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use tracing::debug;

use super::Exchange;

pub struct BinanceExchange {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct BinanceTickerResponse {
    price: String,
}

impl BinanceExchange {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for BinanceExchange {
    async fn fetch_price(&self, symbol: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", symbol);
        
        debug!("Fetching price from Binance for {}", symbol);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("Binance API error: {}", response.status()).into());
        }
        
        let data: BinanceTickerResponse = response.json().await?;
        let price = data.price.parse::<f64>()?;
        
        Ok(price)
    }
}