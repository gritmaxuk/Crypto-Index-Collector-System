use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock, broadcast};
use tokio::signal;
use tracing::{info, error, warn};

use crypto_index_collector::config;
use crypto_index_collector::exchange;
use crypto_index_collector::index::IndexCalculator;
use crypto_index_collector::models::FeedData;
use crypto_index_collector::storage::Database;
use crypto_index_collector::websocket;
use crypto_index_collector::logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Set up logging
    logging::setup_logging()?;

    info!("[STARTUP] Starting Crypto Index Collector...");

    // Load configuration
    let config = config::load_config("config.toml")?;

    info!("[CONFIG] Configuration loaded successfully with {} indices defined", config.indices.len());

    // Set up database connection if enabled
    let database = if config.database.enabled {
        Some(Database::new(&config.database.url, true).await?)
    } else {
        None
    };

    // Set up retention policy if database is enabled
    if let Some(db) = &database {
        db.setup_retention_policy(config.database.retention_days).await?;
    }

    // Create channel for price updates
    let (tx, rx) = mpsc::channel(100);

    // Create index calculator
    let index_calc = Arc::new(RwLock::new(IndexCalculator::new(
        config.indices.clone(),
        rx,
    )));

    // Create a shutdown channel
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    // Start WebSocket server with shutdown channel
    let websocket_address = config.websocket.address.clone();
    let ws_shutdown_rx = shutdown_tx.subscribe();
    let ws_handle = tokio::spawn(async move {
        if let Err(e) = websocket::start_websocket_server(&websocket_address, index_calc.clone(), ws_shutdown_rx).await {
            error!("WebSocket server error: {}", e);
        }
    });

    // Start price feed tasks
    let mut feed_handles = Vec::new();

    for index in &config.indices {
        for feed in &index.feeds {
            let feed = feed.clone();
            let tx = tx.clone();
            let db_clone = database.clone();
            let feed_shutdown_rx = shutdown_tx.subscribe();

            let handle = tokio::spawn(async move {
                fetch_price_loop(feed, tx, db_clone, feed_shutdown_rx).await;
            });

            feed_handles.push(handle);
        }
    }

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("[SHUTDOWN] Shutting down Crypto Index Collector...");

            // Notify all components to shut down
            if let Err(e) = shutdown_tx.send(()) {
                error!("[SHUTDOWN] Failed to send shutdown signal: {}", e);
            }

            // Wait for WebSocket server to shut down
            if let Err(e) = ws_handle.await {
                error!("[SHUTDOWN] Error waiting for WebSocket server to shut down: {}", e);
            }

            // Wait for all price feed tasks to complete
            for handle in feed_handles {
                if let Err(e) = handle.await {
                    error!("[SHUTDOWN] Error waiting for price feed task to complete: {}", e);
                }
            }

            info!("[SHUTDOWN] Graceful shutdown complete");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    Ok(())
}

async fn fetch_price_loop(
    feed: crypto_index_collector::models::PriceFeed,
    tx: mpsc::Sender<FeedData>,
    database: Option<Database>,
    mut shutdown: broadcast::Receiver<()>,
) {
    let mut consecutive_failures = 0;

    loop {
        // Check for shutdown signal
        if shutdown.try_recv().is_ok() {
            info!("[SHUTDOWN] Received shutdown signal in price feed loop for {}", feed.id);
            return;
        }
        match fetch_price(&feed).await {
            Ok(price) => {
                consecutive_failures = 0;

                let timestamp = chrono::Utc::now();
                let feed_data = FeedData {
                    feed_id: feed.id.clone(),
                    timestamp,
                    price,
                };

                info!("[RAW DATA] Exchange: {}, Symbol: {}, Price: {}, Time: {}",
                      feed.exchange, feed.symbol, price, timestamp);

                // Save to database if enabled
                if let Some(db) = &database {
                    if let Err(e) = db.save_price_data(&feed_data).await {
                        error!("Failed to save price data to database: {}", e);
                    } else {
                        info!("[DATABASE] Saved price data for feed: {}", feed_data.feed_id);
                    }
                }

                // Store feed_id before sending feed_data since send() moves the value
                let feed_id = feed_data.feed_id.clone();

                match tx.send(feed_data).await {
                    Ok(_) => {
                        info!("[INTERNAL] Sent price update for feed: {} to index calculator", feed_id);
                    },
                    Err(e) => {
                        if e.to_string().contains("channel closed") {
                            warn!("[CHANNEL] Channel to index calculator closed. This is normal during shutdown.");
                            // During normal shutdown, the receiver might be dropped
                            // We can continue running to collect data for the database
                            if database.is_none() {
                                // If no database is configured, there's no point in continuing
                                info!("[SHUTDOWN] No database configured and channel closed. Exiting feed loop.");
                                return;
                            }
                        } else {
                            error!("Failed to send price update: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                consecutive_failures += 1;

                if consecutive_failures >= 5 {
                    warn!(
                        "[EXCHANGE ERROR] Failed to fetch price from {} for {} {} times consecutively: {}",
                        feed.exchange, feed.symbol, consecutive_failures, e
                    );
                } else {
                    error!("[EXCHANGE ERROR] Failed to fetch price from {} for {}: {}",
                           feed.exchange, feed.symbol, e);
                }
            }
        }

        // Sleep before next fetch
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn fetch_price(feed: &crypto_index_collector::models::PriceFeed) -> Result<f64, Box<dyn Error + Send + Sync>> {
    // Get the exchange implementation
    let exchange = exchange::create_exchange(&feed.exchange)
        .ok_or_else(|| format!("Unsupported exchange: {}", feed.exchange))?;

    // Fetch the price
    let price = exchange.fetch_price(&feed.symbol).await?;

    Ok(price)
}

// Removed unused function
