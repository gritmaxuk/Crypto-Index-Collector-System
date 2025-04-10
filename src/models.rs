use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct IndexDefinition {
    pub name: String,
    pub feeds: Vec<PriceFeed>,
    pub smoothing: SmoothingType,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceFeed {
    pub id: String,
    pub exchange: String,
    pub symbol: String,
    pub weight: u32,  // Percentage (1-100)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmoothingType {
    None,
    Sma,
    Ema,
}

#[derive(Debug, Clone)]
pub struct FeedData {
    pub feed_id: String,
    pub timestamp: DateTime<Utc>,
    pub price: f64,
}