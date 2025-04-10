use std::error::Error;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{self, Duration};
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod config;
mod exchange;
mod index;
mod models;
mod smoothing;
mod db;
use db::Database;

use exchange::Exchange;
use exchange::coinbase::CoinbaseExchange;
use exchange::binance::BinanceExchange;
use index::{IndexCalculator, IndexResult};
use models::{FeedData, PriceFeed};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    // Initialize database if enabled
    let config_data = config::Config::from_file("config.toml")?;
    let database = if config_data.database.enabled {
        Some(Database::new(&config_data.database.url, true).await?)
    } else {
        None
    };
    
    if let Some(db) = &database {
        db.setup_retention_policy(config_data.database.retention_days).await?;
    }

    info!("Starting Crypto Index Collector...");

    // Load configuration
    let config = config_data.clone();
    info!("Configuration loaded successfully");
    
    // Set up channels for price updates
    let (tx, rx) = mpsc::channel::<FeedData>(100);
    
    // Initialize index calculator
    let index_calc = Arc::new(RwLock::new(IndexCalculator::new(
        config.indices.clone(),
        rx,
    )));
    
    // Spawn a task for each exchange to fetch prices
    let mut feed_handles = vec![];
    
    // Coinbase feeds
    for index_def in &config.indices {
        for feed in &index_def.feeds {
            if feed.exchange == "coinbase" {
                let feed_tx = tx.clone();
                let feed_clone = feed.clone();
                let db_clone = database.clone();
                let handle = tokio::spawn(async move {
                    let exchange = CoinbaseExchange::new();
                    fetch_price_loop(exchange, feed_clone, feed_tx, db_clone).await;
                });
                feed_handles.push(handle);
            }
        }
    }
    
    // Binance feeds
    for index_def in &config.indices {
        for feed in &index_def.feeds {
            if feed.exchange == "binance" {
                let feed_tx = tx.clone();
                let feed_clone = feed.clone();
                let db_clone = database.clone();
                let handle = tokio::spawn(async move {
                    let exchange = BinanceExchange::new();
                    fetch_price_loop(exchange, feed_clone, feed_tx, db_clone).await;
                });
                feed_handles.push(handle);
            }
        }
    }
    
    // Spawn index calculator task
    let calc_handle = {
        let index_calc = Arc::clone(&index_calc);
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                
                let calc_result = {
                    let mut calculator = index_calc.write().await;
                    calculator.calculate_indices()
                };
                
                match calc_result {
                    Ok(indices) => {
                        for index in indices {
                            output_index(&index);
                        }
                    }
                    Err(e) => {
                        error!("Failed to calculate indices: {}", e);
                    }
                }
            }
        })
    };
    
    // Setup graceful shutdown
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    
    // Wait for tasks to complete
    for handle in feed_handles {
        handle.abort();
    }
    calc_handle.abort();
    
    info!("Shutdown complete");
    Ok(())
}

async fn fetch_price_loop<E: Exchange>(
    exchange: E,
    feed: PriceFeed,
    tx: mpsc::Sender<FeedData>,
    database: Option<Database>,
) {
    let mut interval = time::interval(Duration::from_secs(5));
    let mut consecutive_failures = 0;
    
    loop {
        interval.tick().await;
        
        match exchange.fetch_price(&feed.symbol).await {
            Ok(price) => {
                consecutive_failures = 0;
                let feed_data = FeedData {
                    feed_id: feed.id.clone(),
                    timestamp: chrono::Utc::now(),
                    price,
                };
                
                // Save to database if enabled
                if let Some(db) = &database {
                    if let Err(e) = db.save_price_data(&feed_data).await {
                        error!("Failed to save price data to database: {}", e);
                    }
                }
                
                if let Err(e) = tx.send(feed_data).await {
                    error!("Failed to send price update: {}", e);
                }
            }
            Err(e) => {
                consecutive_failures += 1;
                if consecutive_failures >= 5 {
                    warn!(
                        "Failed to fetch price from {} for {} {} times consecutively: {}",
                        feed.exchange, feed.symbol, consecutive_failures, e
                    );
                } else {
                    error!("Failed to fetch price: {}", e);
                }
            }
        }
    }
}

fn output_index(index: &IndexResult) {
    info!(
        "INDEX: {} | TIMESTAMP: {} | VALUE: {}",
        index.name, index.timestamp, index.value
    );
}
