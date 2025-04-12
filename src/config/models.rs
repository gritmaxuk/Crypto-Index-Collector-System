use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::models::SmoothingType;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub feeds: HashMap<String, FeedConfig>,
    pub indices: Vec<IndexConfig>,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub websocket: WebsocketConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeedConfig {
    pub exchange: String,
    pub base_currency: String,
    pub quote_currency: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(skip)]
    pub symbol: String,
}

impl FeedConfig {
    // Build the exchange-specific symbol format based on base and quote currencies
    pub fn get_symbol(&self) -> String {
        match self.exchange.as_str() {
            "coinbase" => format!("{}-{}", self.base_currency, self.quote_currency),
            "binance" => {
                // Binance requires USDT for USD pairs
                if self.quote_currency == "USD" {
                    format!("{}{}", self.base_currency, "USDT")
                } else {
                    format!("{}{}", self.base_currency, self.quote_currency)
                }
            },
            _ => format!("{}-{}", self.base_currency, self.quote_currency) // Default format
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexConfig {
    pub name: String,
    pub smoothing: SmoothingType,
    pub feeds: Vec<IndexFeedReference>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexFeedReference {
    pub id: String,
    pub weight: u32,
}

fn default_enabled() -> bool {
    true
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;

        // Validate configuration
        for index in &config.indices {
            // Extract the base and quote currencies from index name (e.g., "BTC" and "USD" from "BTC-USD-INDEX")
            let parts: Vec<&str> = index.name.split('-').collect();
            if parts.len() < 2 {
                return Err(format!("Invalid index name format: {}, expected format like 'BTC-USD-INDEX'", index.name).into());
            }

            let index_base_currency = parts[0];
            let index_quote_currency = parts[1];

            // Check that all referenced feeds exist and match the index currency
            for feed_ref in &index.feeds {
                // Check if the feed exists
                let feed = config.feeds.get(&feed_ref.id)
                    .ok_or_else(|| format!("Feed '{}' referenced in index '{}' does not exist",
                                          feed_ref.id, index.name))?;

                // Check if the feed is enabled
                if !feed.enabled {
                    return Err(format!("Feed '{}' referenced in index '{}' is disabled",
                                      feed_ref.id, index.name).into());
                }

                // Check if the feed's base and quote currencies match the index's currencies
                if feed.base_currency != index_base_currency {
                    return Err(format!(
                        "Feed '{}' with base currency '{}' cannot be used in index '{}' with base currency '{}'",
                        feed_ref.id, feed.base_currency, index.name, index_base_currency
                    ).into());
                }

                if feed.quote_currency != index_quote_currency {
                    return Err(format!(
                        "Feed '{}' with quote currency '{}' cannot be used in index '{}' with quote currency '{}'",
                        feed_ref.id, feed.quote_currency, index.name, index_quote_currency
                    ).into());
                }
            }

            // Validate weights
            let total_weight: u32 = index.feeds.iter().map(|f| f.weight).sum();
            if total_weight != 100 {
                return Err(format!("Weights for index {} must sum to 100, got {}",
                                  index.name, total_weight).into());
            }
        }

        Ok(config)
    }

    // Convert to the internal model format used by the application
    pub fn to_internal_model(&self) -> Result<Vec<crate::models::IndexDefinition>, String> {
        let mut result = Vec::with_capacity(self.indices.len());

        for index_config in &self.indices {
            let mut feeds = Vec::with_capacity(index_config.feeds.len());

            for feed_ref in &index_config.feeds {
                let feed_config = self.feeds.get(&feed_ref.id)
                    .ok_or_else(|| format!("Feed '{}' referenced in index '{}' not found",
                                          feed_ref.id, index_config.name))?;

                feeds.push(crate::models::PriceFeed {
                    id: feed_ref.id.clone(),
                    exchange: feed_config.exchange.clone(),
                    symbol: feed_config.get_symbol(),
                    weight: feed_ref.weight,
                });
            }

            result.push(crate::models::IndexDefinition {
                name: index_config.name.clone(),
                feeds,
                smoothing: index_config.smoothing.clone(),
            });
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_db_url")]
    pub url: String,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: default_db_url(),
            retention_days: default_retention_days(),
        }
    }
}

fn default_db_url() -> String {
    "postgres://postgres:password@localhost:5432/crypto_indices".to_string()
}

fn default_retention_days() -> u32 {
    30
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebsocketConfig {
    #[serde(default = "default_websocket_address")]
    pub address: String,
}

impl Default for WebsocketConfig {
    fn default() -> Self {
        Self {
            address: default_websocket_address(),
        }
    }
}

fn default_websocket_address() -> String {
    "127.0.0.1:8080".to_string()
}
