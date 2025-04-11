use std::net::SocketAddr;
use std::sync::Arc;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::time::Duration;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tracing::{info, error};

use crate::index::IndexCalculator;
use crate::error::AppResult;

/// Start a WebSocket server for streaming index updates
pub async fn start_websocket_server(address: &str, index_calc: Arc<RwLock<IndexCalculator>>) -> AppResult<()> {
    let addr: SocketAddr = address.parse()
        .map_err(|e| format!("Invalid WebSocket address: {}", e))?;
    
    let listener = TcpListener::bind(&addr).await
        .map_err(|e| format!("Failed to bind WebSocket server: {}", e))?;
    
    info!("[WEBSOCKET SERVER] Listening on: {}", address);
    
    while let Ok((stream, addr)) = listener.accept().await {
        let index_calc_clone = index_calc.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, addr, index_calc_clone).await {
                error!("Error handling WebSocket connection: {}", e);
            }
        });
    }
    
    Ok(())
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, index_calc: Arc<RwLock<IndexCalculator>>) -> AppResult<()> {
    info!("[WEBSOCKET CONNECTION] Incoming connection from: {}", addr);

    let ws_stream = accept_async(stream).await?;
    
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
