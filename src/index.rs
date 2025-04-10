use std::collections::{HashMap, VecDeque};
use std::error::Error;
use chrono::{DateTime, Utc};
use tokio::sync::mpsc;
use tracing::error;

use crate::models::{FeedData, IndexDefinition};
use crate::smoothing::SmoothingAlgorithm;

const MAX_HISTORY_SIZE: usize = 20;

#[derive(Debug)]
pub struct IndexCalculator {
    indices: Vec<IndexDefinition>,
    feed_values: HashMap<String, f64>,
    feed_history: HashMap<String, VecDeque<f64>>,
    index_history: HashMap<String, VecDeque<f64>>,
    receiver: mpsc::Receiver<FeedData>,
}

#[derive(Debug, Clone)]
pub struct IndexResult {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

impl IndexCalculator {
    pub fn new(
        indices: Vec<IndexDefinition>,
        receiver: mpsc::Receiver<FeedData>,
    ) -> Self {
        let mut feed_values = HashMap::new();
        let mut feed_history = HashMap::new();
        let mut index_history = HashMap::new();
        
        // Initialize data structures
        for index in &indices {
            index_history.insert(index.name.clone(), VecDeque::with_capacity(MAX_HISTORY_SIZE));
            
            for feed in &index.feeds {
                feed_values.insert(feed.id.clone(), 0.0);
                feed_history.insert(feed.id.clone(), VecDeque::with_capacity(MAX_HISTORY_SIZE));
            }
        }
        
        Self {
            indices,
            feed_values,
            feed_history,
            index_history,
            receiver,
        }
    }
    
    pub fn calculate_indices(&mut self) -> Result<Vec<IndexResult>, Box<dyn Error>> {
        // Process any new feed updates
        self.process_feed_updates()?;
        
        let mut results = Vec::new();
        let timestamp = Utc::now();
        
        for index_def in &self.indices {
            let mut weighted_sum = 0.0;
            let mut total_weights = 0;
            let mut missing_feeds = false;
            
            for feed in &index_def.feeds {
                if let Some(&price) = self.feed_values.get(&feed.id) {
                    if price > 0.0 {
                        weighted_sum += price * (feed.weight as f64 / 100.0);
                        total_weights += feed.weight;
                    } else {
                        missing_feeds = true;
                        break;
                    }
                } else {
                    missing_feeds = true;
                    break;
                }
            }
            
            if missing_feeds || total_weights == 0 {
                continue;
            }
            
            let raw_index_value = weighted_sum / (total_weights as f64 / 100.0);
            
            // Apply smoothing algorithm
            let smoothing_algo = SmoothingAlgorithm::from(&index_def.smoothing);
            let index_history = self.index_history.entry(index_def.name.clone()).or_default();
            let smoothed_value = smoothing_algo.apply(index_history, raw_index_value);
            
            // Update history
            index_history.push_front(smoothed_value);
            if index_history.len() > MAX_HISTORY_SIZE {
                index_history.pop_back();
            }
            
            results.push(IndexResult {
                name: index_def.name.clone(),
                timestamp,
                value: smoothed_value,
            });
        }
        
        if results.is_empty() {
            error!("Failed to calculate any indices - missing price data");
        }
        
        Ok(results)
    }
    
    fn process_feed_updates(&mut self) -> Result<(), Box<dyn Error>> {
        // Process all available updates without blocking
        while let Ok(feed_data) = self.receiver.try_recv() {
            // Update current value
            self.feed_values.insert(feed_data.feed_id.clone(), feed_data.price);
            
            // Update history
            let history = self.feed_history.entry(feed_data.feed_id.clone()).or_default();
            history.push_front(feed_data.price);
            if history.len() > MAX_HISTORY_SIZE {
                history.pop_back();
            }
        }
        
        Ok(())
    }
}