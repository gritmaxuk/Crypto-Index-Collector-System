use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use tracing::debug;

use super::Exchange;

pub struct CoinbaseExchange {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct CoinbaseResponse {
    data: CoinbaseData,
}

#[derive(Debug, Deserialize)]
struct CoinbaseData {
    amount: String,
}

impl CoinbaseExchange {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl Exchange for CoinbaseExchange {
    async fn fetch_price(&self, symbol: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
        let url = format!("https://api.coinbase.com/v2/prices/{}/spot", symbol);
        
        debug!("Fetching price from Coinbase for {}", symbol);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("Coinbase API error: {}", response.status()).into());
        }
        
        let data: CoinbaseResponse = response.json().await?;
        let price = data.data.amount.parse::<f64>()?;
        
        Ok(price)
    }
}