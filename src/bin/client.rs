use std::error::Error;
use std::time::Duration;
use clap::Parser;
use futures::{StreamExt, SinkExt};
use tokio::{time, signal};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{info, error, warn, Level};
use tracing_subscriber::FmtSubscriber;

/// Crypto Index Client - WebSocket client for receiving crypto index updates
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// WebSocket server address
    #[arg(short, long, default_value = "ws://127.0.0.1:9000")]
    server: String,

    /// Reconnect automatically if connection is lost
    #[arg(short, long, default_value_t = true)]
    reconnect: bool,

    /// Reconnection delay in seconds
    #[arg(long, default_value_t = 5)]
    reconnect_delay: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Parse command line arguments
    let args = Args::parse();

    info!("[CLIENT] Crypto Index Client starting up");
    info!("[CLIENT] Connecting to WebSocket server at {}", args.server);

    let mut reconnect_attempts = 0;

    loop {
        match connect_to_server(&args.server).await {
            Ok(()) => {
                // Connection closed normally, reset reconnect attempts
                reconnect_attempts = 0;

                if !args.reconnect {
                    info!("[CLIENT] Connection closed and reconnect disabled. Exiting.");
                    break;
                }

                info!("[CLIENT] Connection closed. Reconnecting in {} seconds...", args.reconnect_delay);
                time::sleep(Duration::from_secs(args.reconnect_delay)).await;
            }
            Err(e) => {
                reconnect_attempts += 1;

                if !args.reconnect {
                    error!("[CLIENT] Connection error and reconnect disabled. Exiting: {}", e);
                    return Err(e);
                }

                let delay = calculate_backoff_delay(reconnect_attempts, args.reconnect_delay);
                warn!("[CLIENT] Connection error (attempt {}). Reconnecting in {} seconds: {}",
                      reconnect_attempts, delay, e);
                time::sleep(Duration::from_secs(delay)).await;
            }
        }
    }

    Ok(())
}

async fn connect_to_server(server_url: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Connect to the WebSocket server
    let (ws_stream, _) = connect_async(server_url).await?;
    info!("[CLIENT] Connected to the server successfully");

    // Split the WebSocket stream
    let (mut write, mut read) = ws_stream.split();

    // Process incoming messages with Ctrl+C handling
    loop {
        tokio::select! {
            // Handle WebSocket messages
            message = read.next() => {
                match message {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            process_message(msg);
                        } else if msg.is_close() {
                            info!("[CLIENT] Received close frame from server");
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        error!("[CLIENT] Error receiving message: {}", e);
                        return Err(Box::new(e));
                    }
                    None => {
                        info!("[CLIENT] WebSocket stream ended");
                        break;
                    }
                }
            }

            // Handle Ctrl+C signal
            _ = signal::ctrl_c() => {
                info!("[CLIENT] Received Ctrl+C, closing connection gracefully");
                // Send close frame
                if let Err(e) = write.send(Message::Close(None)).await {
                    warn!("[CLIENT] Error sending close frame: {}", e);
                }
                break;
            }
        }
    }

    info!("[CLIENT] WebSocket connection closed");
    Ok(())
}

fn process_message(msg: Message) {
    if let Message::Text(text) = msg {
        // Check if it's an index update message
        if text.starts_with("INDEX:") {
            // Parse the index data
            let parts: Vec<&str> = text.split('|').collect();
            if parts.len() >= 3 {
                let index_part = parts[0].trim();
                let timestamp_part = parts[1].trim();
                let value_part = parts[2].trim();

                // Extract the index name
                let index_name = index_part.strip_prefix("INDEX:").unwrap_or(index_part).trim();

                // Extract the timestamp
                let timestamp = timestamp_part.strip_prefix("TIMESTAMP:").unwrap_or(timestamp_part).trim();

                // Extract the value
                let value = value_part.strip_prefix("VALUE:").unwrap_or(value_part).trim();

                // Display the index update
                info!("[INDEX UPDATE] {} = {} ({})", index_name, value, timestamp);
            } else {
                warn!("[CLIENT] Received malformed index message: {}", text);
            }
        } else {
            // Just display the message as-is
            info!("[SERVER MESSAGE] {}", text);
        }
    }
}

fn calculate_backoff_delay(attempts: u64, base_delay: u64) -> u64 {
    // Exponential backoff with a maximum delay
    let max_delay = 60; // Maximum delay in seconds
    let delay = base_delay * (1 << attempts.saturating_sub(1));
    delay.min(max_delay)
}
