//! WebSocket server for browser-based UI development
//!
//! This module provides a WebSocket server that exposes the same IPC protocol
//! used by the native WKWebView transport, enabling real-time communication
//! between a browser-based UI and the Rust engine during development.

use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};
use wavecraft_bridge::{IpcHandler, ParameterHost};

/// Shared state for tracking connected clients
struct ServerState {
    /// Connected browser clients (for broadcasting meter updates)
    browser_clients: Arc<RwLock<Vec<tokio::sync::mpsc::UnboundedSender<String>>>>,
    /// Audio client ID (if connected)
    audio_client: Arc<RwLock<Option<String>>>,
}

impl ServerState {
    fn new() -> Self {
        Self {
            browser_clients: Arc::new(RwLock::new(Vec::new())),
            audio_client: Arc::new(RwLock::new(None)),
        }
    }
}

/// WebSocket server for browser-based UI development
pub struct WsServer<H: ParameterHost + 'static> {
    /// Port the server listens on
    port: u16,
    /// Shared IPC handler
    handler: Arc<IpcHandler<H>>,
    /// Shutdown signal
    shutdown_tx: broadcast::Sender<()>,
    /// Enable verbose logging (all JSON-RPC messages)
    verbose: bool,
    /// Shared server state
    state: Arc<ServerState>,
}

impl<H: ParameterHost + 'static> WsServer<H> {
    /// Create a new WebSocket server
    pub fn new(port: u16, handler: Arc<IpcHandler<H>>, verbose: bool) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            port,
            handler,
            shutdown_tx,
            verbose,
            state: Arc::new(ServerState::new()),
        }
    }

    /// Start the server (spawns async tasks, returns immediately)
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("127.0.0.1:{}", self.port).parse()?;
        let listener = TcpListener::bind(&addr).await?;

        info!("Server listening on ws://{}", addr);

        let handler = Arc::clone(&self.handler);
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let verbose = self.verbose;
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                info!("Client connected: {}", addr);
                                let handler = Arc::clone(&handler);
                                let state = Arc::clone(&state);
                                tokio::spawn(handle_connection(handler, stream, addr, verbose, state));
                            }
                            Err(e) => {
                                error!("Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Server shutting down");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Shutdown the server gracefully.
    ///
    /// Note: Not currently called but kept for future graceful shutdown support.
    #[allow(dead_code)]
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

/// Handle a single WebSocket connection
async fn handle_connection<H: ParameterHost>(
    handler: Arc<IpcHandler<H>>,
    stream: TcpStream,
    addr: SocketAddr,
    verbose: bool,
    state: Arc<ServerState>,
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("Error during handshake with {}: {}", addr, e);
            return;
        }
    };

    info!("WebSocket connection established: {}", addr);

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    
    // Track this client for broadcasting
    let mut is_audio_client = false;
    state.browser_clients.write().await.push(tx.clone());
    let client_index = state.browser_clients.read().await.len() - 1;

    // Spawn task to send messages from channel to WebSocket
    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = write.send(Message::Text(msg)).await {
                error!("Error sending to {}: {}", addr, e);
                break;
            }
        }
    });

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(json)) => {
                // Log incoming message (verbose only)
                if verbose {
                    debug!("Received from {}: {}", addr, json);
                }

                // Check if this is an audio client registration
                if json.contains("\"method\":\"registerAudio\"") {
                    is_audio_client = true;
                    info!("Audio client registered: {}", addr);
                    
                    // Parse to extract client_id
                    if let Ok(req) = serde_json::from_str::<wavecraft_protocol::IpcRequest>(&json) {
                        if let Some(params) = req.params {
                            if let Ok(audio_params) = serde_json::from_value::<wavecraft_protocol::RegisterAudioParams>(params) {
                                *state.audio_client.write().await = Some(audio_params.client_id.clone());
                            }
                        }
                    }
                    
                    // Send success response
                    let response = wavecraft_protocol::IpcResponse::success(
                        wavecraft_protocol::RequestId::Number(1),
                        wavecraft_protocol::RegisterAudioResult {
                            status: "registered".to_string(),
                        },
                    );
                    let response_json = serde_json::to_string(&response).unwrap();
                    if let Err(e) = tx.send(response_json) {
                        error!("Error sending response: {}", e);
                        break;
                    }
                    continue;
                }

                // Check if this is a meter update notification from audio client
                if is_audio_client && json.contains("\"method\":\"meterUpdate\"") {
                    // Broadcast to all browser clients
                    let clients = state.browser_clients.read().await;
                    for (idx, client) in clients.iter().enumerate() {
                        if idx != client_index {  // Don't send back to audio client
                            let _ = client.send(json.clone());
                        }
                    }
                    continue;
                }

                // Route through existing IpcHandler
                let response = handler.handle_json(&json);

                // Log outgoing response (verbose only)
                if verbose {
                    debug!("Sending to {}: {}", addr, response);
                }

                // Send response
                if let Err(e) = tx.send(response) {
                    error!("Error queueing response: {}", e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                info!("Client closed connection: {}", addr);
                break;
            }
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                // Ignore ping/pong frames (automatically handled)
            }
            Ok(Message::Binary(_)) => {
                warn!("Unexpected binary message from {}", addr);
            }
            Ok(Message::Frame(_)) => {
                // Raw frames shouldn't appear at this level
            }
            Err(e) => {
                error!("Error receiving from {}: {}", addr, e);
                break;
            }
        }
    }

    // Cleanup: remove client from broadcast list
    state.browser_clients.write().await.retain(|c| !c.is_closed());
    if is_audio_client {
        *state.audio_client.write().await = None;
        info!("Audio client disconnected: {}", addr);
    }
    
    write_task.abort();
    info!("Connection closed: {}", addr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppState;

    #[tokio::test]
    async fn test_server_creation() {
        let state = AppState::new();
        let handler = Arc::new(IpcHandler::new(state));
        let server = WsServer::new(9001, handler, false);

        // Just verify we can create a server without panicking
        assert_eq!(server.port, 9001);
    }
}
