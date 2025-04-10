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
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};
use std::net::SocketAddr;
use futures::stream::StreamExt;
use futures::sink::SinkExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
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

    info!("[STARTUP] Starting Crypto Index Collector...");

    // Load configuration
    let config = config_data.clone();
    info!("[CONFIG] Configuration loaded successfully with {} indices defined", config.indices.len());

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
            let mut interval = tokio::time::interval(Duration::from_secs(1));
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
                            // WebSocket handling is done in the write_to_websocket function
                        }
                    }
                    Err(e) => {
                        error!("Failed to calculate indices: {}", e);
                    }
                }
            }
        })
    };

    // Start WebSocket server: use configured address or fallback to default if missing
    let websocket_address = if config.websocket.address.trim().is_empty() {
        "127.0.0.1:8080".to_string()
    } else {
        config.websocket.address.clone()
    };
    if let Err(e) = start_websocket_server(websocket_address, index_calc.clone()).await {
        error!("Failed to start WebSocket server: {}", e);
    }

    // Setup graceful shutdown
    tokio::signal::ctrl_c().await?;
    info!("[SHUTDOWN] Shutting down Crypto Index Collector...");

    // Wait for tasks to complete
    for handle in feed_handles {
        handle.abort();
    }
    calc_handle.abort();

    info!("[SHUTDOWN] Shutdown complete");
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

                if let Err(e) = tx.send(feed_data).await {
                    error!("Failed to send price update: {}", e);
                } else {
                    info!("[INTERNAL] Sent price update for feed: {} to index calculator", feed_id);
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
    }
}

fn output_index(index: &IndexResult) {
    info!(
        "[CALCULATED INDEX] Name: {}, Value: {}, Time: {}",
        index.name, index.value, index.timestamp
    );
}

// WebSocket server
async fn start_websocket_server(address: String, index_calc: Arc<RwLock<IndexCalculator>>) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let try_socket = TcpListener::bind(&address).await;
    let listener = try_socket.map_err(|e| format!("Failed to bind: {}", e))?;
    info!("[WEBSOCKET SERVER] Listening on: {}", address);

    while let Ok((stream, addr)) = listener.accept().await {
        // SocketAddr implements Copy, so no need to clone
        let index_calc_clone = index_calc.clone();
        tokio::spawn(async move {
            let result = handle_connection(stream, addr, index_calc_clone).await;
            if let Err(e) = result {
                error!("Error handling connection: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, index_calc: Arc<RwLock<IndexCalculator>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    info!("[WEBSOCKET CONNECTION] Incoming connection from: {}", addr);

    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| {
            error!("Failed to accept WebSocket connection: {}", e);
            Box::new(e) as Box<dyn Error + Send + Sync>
        })?;

    info!("[WEBSOCKET ESTABLISHED] Connection established with: {}", addr);

    // Use a simpler approach without spawning a task
    handle_websocket(ws_stream, addr, index_calc).await;

    Ok(())
}

async fn handle_websocket(mut ws_stream: WebSocketStream<TcpStream>, addr: SocketAddr, index_calc: Arc<RwLock<IndexCalculator>>) {
    // Send a welcome message
    let welcome = format!("Connected to Crypto Index Collector. Client: {}", addr);
    info!("[WEBSOCKET WELCOME] Sending welcome message to: {}", addr);
    let _ = ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(welcome.into())).await;

    // Set up interval for periodic updates
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    // Main processing loop
    loop {
        tokio::select! {
            // Handle incoming messages
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        info!("[WEBSOCKET RECEIVED] From: {}, Message: {:?}", addr, msg);
                    }
                    Some(Err(e)) => {
                        error!("[WEBSOCKET ERROR] From: {}, Error: {}", addr, e);
                        break;
                    }
                    None => {
                        info!("[WEBSOCKET CLOSED] Connection closed by client: {}", addr);
                        break;
                    }
                }
            }

            // Send index updates periodically
            _ = interval.tick() => {
                // Calculate indices
                let indices = match index_calc.write().await.calculate_indices() {
                    Ok(indices) => indices,
                    Err(e) => {
                        error!("Failed to calculate indices: {}", e);
                        continue;
                    }
                };

                // Send each index
                for index in indices {
                    let message = format!("INDEX: {} | TIMESTAMP: {} | VALUE: {}",
                        index.name, index.timestamp, index.value);

                    info!("[WEBSOCKET SEND] Client: {}, Index: {}, Value: {}",
                         addr, index.name, index.value);

                    if let Err(e) = ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(message.into())).await {
                        error!("[WEBSOCKET ERROR] Failed to send to: {}, Error: {}", addr, e);
                        return;
                    }
                }
            }
        }
    }

    info!("[WEBSOCKET CLOSED] Connection terminated with: {}", addr);
}

// WebSocket handling is done through tokio::select in handle_websocket
