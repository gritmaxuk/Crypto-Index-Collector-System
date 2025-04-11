use std::error::Error;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::models::IndexDefinition;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub indices: Vec<IndexDefinition>,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub websocket: WebsocketConfig,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;

        // Validate configuration
        for index in &config.indices {
            let total_weight: u32 = index.feeds.iter().map(|f| f.weight).sum();
            if total_weight != 100 {
                return Err(format!("Weights for index {} must sum to 100, got {}", index.name, total_weight).into());
            }
        }

        Ok(config)
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
