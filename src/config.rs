use std::error::Error;
use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::models::IndexDefinition;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub indices: Vec<IndexDefinition>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
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