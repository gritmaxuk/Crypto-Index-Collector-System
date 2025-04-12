use std::net::SocketAddr;
use std::sync::Arc;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, broadcast};
use tokio::time::Duration;
use tokio_tungstenite::{accept_async, WebSocketStream, tungstenite::Message};

use tracing::{info, error, warn};

use crate::index::IndexCalculator;
use crate::error::AppResult;

/// Start a WebSocket server for streaming index updates
pub async fn start_websocket_server(
    address: &str,
    index_calc: Arc<RwLock<IndexCalculator>>,
    mut shutdown: broadcast::Receiver<()>,
) -> AppResult<()> {
    let addr: SocketAddr = address.parse()
        .map_err(|e| format!("Invalid WebSocket address: {}", e))?;

    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                let port = addr.port();
                return Err(format!("WebSocket port {} is already in use. This could be due to:\n\
                1. Another instance of the collector is already running\n\
                2. Another application is using this port\n\
                Try running 'lsof -i :{}' to identify the process, then terminate it with 'kill <PID>'.",
                port, port).into());
            } else {
                return Err(format!("Failed to bind WebSocket server: {}", e).into());
            }
        }
    };

    info!("[WEBSOCKET SERVER] Listening on: {}", address);

    loop {
        tokio::select! {
            accept_result = listener.accept() => {
                match accept_result {
                    Ok((stream, addr)) => {
                        let index_calc_clone = index_calc.clone();
                        let shutdown_rx = shutdown.resubscribe();

                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(stream, addr, index_calc_clone, shutdown_rx).await {
                                error!("Error handling WebSocket connection: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
            _ = shutdown.recv() => {
                info!("[WEBSOCKET SERVER] Shutdown signal received, stopping server");
                break;
            }
        }
    }

    info!("[WEBSOCKET SERVER] Server stopped gracefully");
    Ok(())
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    index_calc: Arc<RwLock<IndexCalculator>>,
    shutdown: broadcast::Receiver<()>,
) -> AppResult<()> {
    info!("[WEBSOCKET CONNECTION] Incoming connection from: {}", addr);

    let ws_stream = accept_async(stream).await?;

    info!("[WEBSOCKET ESTABLISHED] Connection established with: {}", addr);

    handle_websocket(ws_stream, addr, index_calc, shutdown).await;

    Ok(())
}

async fn handle_websocket(
    mut ws_stream: WebSocketStream<TcpStream>,
    addr: SocketAddr,
    index_calc: Arc<RwLock<IndexCalculator>>,
    mut shutdown: broadcast::Receiver<()>,
) {
    // Send welcome message
    let welcome = format!("Connected to Crypto Index Collector. Client: {}", addr);
    info!("[WEBSOCKET WELCOME] Sending welcome message to: {}", addr);

    // Try to send a close frame when shutting down
    if shutdown.try_recv().is_ok() {
        info!("[WEBSOCKET] Closing connection to client: {}", addr);
        // Send close frame
        if let Err(e) = ws_stream.send(Message::Close(None)).await {
            warn!("[WEBSOCKET] Error sending close frame to {}: {}", addr, e);
        }
        return;
    }

    let _ = ws_stream.send(Message::Text(welcome.into())).await;

    // Start a heartbeat task
    let heartbeat_interval = Duration::from_secs(30);
    let mut heartbeat_timer = tokio::time::interval(heartbeat_interval);

    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        tokio::select! {
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

            _ = interval.tick() => {
                match index_calc.write().await.calculate_indices() {
                    Ok(indices) => {
                        for index in indices {
                            let message = format!("INDEX: {} | TIMESTAMP: {} | VALUE: {}",
                                index.name, index.timestamp, index.value);

                            if let Err(e) = ws_stream.send(Message::Text(message.into())).await {
                                error!("[WEBSOCKET ERROR] Failed to send to: {}, Error: {}", addr, e);
                                return;
                            }
                        }
                    }
                    Err(e) => error!("Failed to calculate indices: {}", e)
                }
            }

            _ = shutdown.recv() => {
                info!("[WEBSOCKET CONNECTION] Shutdown signal received, closing connection with: {}", addr);
                let _ = ws_stream.send(Message::Close(None)).await;
                break;
            }

            _ = heartbeat_timer.tick() => {
                // Send ping frame as heartbeat
                info!("[WEBSOCKET HEARTBEAT] Sending ping to: {}", addr);
                if let Err(e) = ws_stream.send(Message::Ping(vec![].into())).await {
                    error!("[WEBSOCKET ERROR] Failed to send ping to: {}, Error: {}", addr, e);
                    break;
                }
            }
        }
    }

    info!("[WEBSOCKET CLOSED] Connection terminated with: {}", addr);
}
