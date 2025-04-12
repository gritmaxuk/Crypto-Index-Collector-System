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
    pub symbol: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
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
            // Check that all referenced feeds exist
            for feed_ref in &index.feeds {
                if !config.feeds.contains_key(&feed_ref.id) {
                    return Err(format!("Feed '{}' referenced in index '{}' does not exist",
                                      feed_ref.id, index.name).into());
                }

                // Check if the feed is enabled
                if let Some(feed) = config.feeds.get(&feed_ref.id) {
                    if !feed.enabled {
                        return Err(format!("Feed '{}' referenced in index '{}' is disabled",
                                          feed_ref.id, index.name).into());
                    }
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
    pub fn to_internal_model(&self) -> Vec<crate::models::IndexDefinition> {
        self.indices.iter().map(|index_config| {
            let feeds = index_config.feeds.iter().map(|feed_ref| {
                let feed_config = self.feeds.get(&feed_ref.id).unwrap();
                crate::models::PriceFeed {
                    id: feed_ref.id.clone(),
                    exchange: feed_config.exchange.clone(),
                    symbol: feed_config.symbol.clone(),
                    weight: feed_ref.weight,
                }
            }).collect();

            crate::models::IndexDefinition {
                name: index_config.name.clone(),
                feeds,
                smoothing: index_config.smoothing.clone(),
            }
        }).collect()
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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WebsocketConfig {
    #[serde(default = "default_websocket_address")]
    pub address: String,
}

fn default_websocket_address() -> String {
    "127.0.0.1:8080".to_string()
}
