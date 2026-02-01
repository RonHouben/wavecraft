//! WebSocket server for browser-based UI development
//!
//! This module provides a WebSocket server that exposes the same IPC protocol
//! used by the native WKWebView transport, enabling real-time communication
//! between a browser-based UI and the Rust engine during development.

use bridge::{IpcHandler, ParameterHost};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

/// WebSocket server for browser-based UI development
pub struct WsServer<H: ParameterHost + 'static> {
    /// Port the server listens on
    port: u16,
    /// Shared IPC handler
    handler: Arc<IpcHandler<H>>,
    /// Shutdown signal
    shutdown_tx: broadcast::Sender<()>,
}

impl<H: ParameterHost + 'static> WsServer<H> {
    /// Create a new WebSocket server
    pub fn new(port: u16, handler: Arc<IpcHandler<H>>) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            port,
            handler,
            shutdown_tx,
        }
    }

    /// Start the server (spawns async tasks, returns immediately)
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("127.0.0.1:{}", self.port).parse()?;
        let listener = TcpListener::bind(&addr).await?;

        println!("[WebSocket] Server listening on ws://{}", addr);

        let handler = Arc::clone(&self.handler);
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                println!("[WebSocket] Client connected: {}", addr);
                                let handler = Arc::clone(&handler);
                                tokio::spawn(handle_connection(handler, stream, addr));
                            }
                            Err(e) => {
                                eprintln!("[WebSocket] Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        println!("[WebSocket] Server shutting down");
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
) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("[WebSocket] Error during handshake with {}: {}", addr, e);
            return;
        }
    };

    println!("[WebSocket] WebSocket connection established: {}", addr);

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(json)) => {
                // Log incoming message
                println!("[WebSocket] Received from {}: {}", addr, json);

                // Route through existing IpcHandler
                let response = handler.handle_json(&json);

                // Log outgoing response
                println!("[WebSocket] Sending to {}: {}", addr, response);

                // Send response back to client
                if let Err(e) = write.send(Message::Text(response)).await {
                    eprintln!("[WebSocket] Error sending response to {}: {}", addr, e);
                    break;
                }
            }
            Ok(Message::Close(_)) => {
                println!("[WebSocket] Client closed connection: {}", addr);
                break;
            }
            Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                // Ignore ping/pong frames (automatically handled)
            }
            Ok(Message::Binary(_)) => {
                eprintln!("[WebSocket] Unexpected binary message from {}", addr);
            }
            Ok(Message::Frame(_)) => {
                // Raw frames shouldn't appear at this level
            }
            Err(e) => {
                eprintln!("[WebSocket] Error receiving from {}: {}", addr, e);
                break;
            }
        }
    }

    println!("[WebSocket] Connection closed: {}", addr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppState;

    #[tokio::test]
    async fn test_server_creation() {
        let state = AppState::new();
        let handler = Arc::new(IpcHandler::new(state));
        let server = WsServer::new(9001, handler);

        // Just verify we can create a server without panicking
        assert_eq!(server.port, 9001);
    }
}
