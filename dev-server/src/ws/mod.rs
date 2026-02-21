//! WebSocket server for browser-based UI development
//!
//! This module provides a WebSocket server that exposes the same IPC protocol
//! used by the native WKWebView transport, enabling real-time communication
//! between a browser-based UI and the Rust engine during development.

use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, broadcast};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};
use wavecraft_bridge::{IpcHandler, ParameterHost};
use wavecraft_protocol::{
    AudioRuntimeStatus, IpcNotification, IpcRequest, IpcResponse, METHOD_REGISTER_AUDIO,
    NOTIFICATION_AUDIO_STATUS_CHANGED, NOTIFICATION_METER_UPDATE, NOTIFICATION_PARAMETER_CHANGED,
    SetParameterParams,
};

const NOTIFICATION_PARAMETERS_CHANGED: &str = "parametersChanged";

type BrowserClientTx = tokio::sync::mpsc::Sender<String>;

/// Shared state for tracking connected clients
struct ServerState {
    /// Connected browser clients (for broadcasting meter updates)
    browser_clients: Arc<RwLock<Vec<BrowserClientTx>>>,
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

/// A lightweight, cloneable handle to the WebSocket server's broadcast
/// capability. Non-generic — can be passed across async task boundaries.
///
/// Constructed via [`WsServer::handle()`]. Used by the CLI to forward
/// meter updates from the in-process audio callback to browser clients.
#[derive(Clone)]
pub struct WsHandle {
    state: Arc<ServerState>,
}

impl WsHandle {
    /// Broadcast a JSON string to all connected browser clients.
    pub async fn broadcast(&self, json: &str) {
        broadcast_to_browser_clients(&self.state, json, None, "broadcast message").await;
    }

    /// Broadcast an audioStatusChanged notification to connected clients.
    pub async fn broadcast_audio_status_changed(
        &self,
        status: &AudioRuntimeStatus,
    ) -> Result<(), serde_json::Error> {
        let json = serde_json::to_string(&IpcNotification::new(
            NOTIFICATION_AUDIO_STATUS_CHANGED,
            status,
        ))?;

        self.broadcast(&json).await;
        Ok(())
    }
}

async fn broadcast_to_browser_clients(
    state: &Arc<ServerState>,
    json: &str,
    exclude_client_index: Option<usize>,
    warning_context: &str,
) {
    let clients = state.browser_clients.read().await;
    for (index, client) in clients.iter().enumerate() {
        if exclude_client_index.is_some_and(|excluded| index == excluded) {
            continue;
        }

        if let Err(error) = client.try_send(json.to_owned()) {
            warn!(
                "Failed to {} (client {}): {}",
                warning_context, index, error
            );
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
    /// Shared server state
    state: Arc<ServerState>,
}

fn build_set_parameter_notification(request: &IpcRequest, response: &str) -> Option<String> {
    if request.method != wavecraft_protocol::METHOD_SET_PARAMETER {
        return None;
    }

    let response_msg = serde_json::from_str::<IpcResponse>(response).ok()?;
    if response_msg.error.is_some() {
        return None;
    }

    let params = request.params.clone()?;
    let set_params = serde_json::from_value::<SetParameterParams>(params).ok()?;

    serde_json::to_string(&IpcNotification::new(
        NOTIFICATION_PARAMETER_CHANGED,
        serde_json::json!({
            "id": set_params.id,
            "value": set_params.value,
        }),
    ))
    .ok()
}

impl<H: ParameterHost + 'static> WsServer<H> {
    /// Create a new WebSocket server
    pub fn new(port: u16, handler: Arc<IpcHandler<H>>) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            port,
            handler,
            shutdown_tx,
            state: Arc::new(ServerState::new()),
        }
    }

    /// Get a lightweight handle for broadcasting to connected clients.
    ///
    /// The returned `WsHandle` is non-generic, `Clone`, and can be moved
    /// into async tasks (e.g., for forwarding meter updates from audio).
    pub fn handle(&self) -> WsHandle {
        WsHandle {
            state: Arc::clone(&self.state),
        }
    }

    /// Broadcast a parametersChanged notification to all connected clients.
    ///
    /// This is used by the hot-reload pipeline to notify the UI that
    /// parameters have been updated and should be re-fetched.
    pub async fn broadcast_parameters_changed(&self) -> Result<(), serde_json::Error> {
        let notification =
            IpcNotification::new(NOTIFICATION_PARAMETERS_CHANGED, serde_json::json!({}));
        let json = serde_json::to_string(&notification)?;

        broadcast_to_browser_clients(
            &self.state,
            &json,
            None,
            "send parametersChanged notification",
        )
        .await;

        Ok(())
    }

    /// Start the server (spawns async tasks, returns immediately)
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr: SocketAddr = format!("127.0.0.1:{}", self.port).parse()?;
        let listener = TcpListener::bind(&addr).await?;

        info!("Server listening on ws://{}", addr);

        let handler = Arc::clone(&self.handler);
        let mut shutdown_rx = self.shutdown_tx.subscribe();
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
                                tokio::spawn(handle_connection(handler, stream, addr, state));
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
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(128);

    // Track this client for broadcasting
    let mut is_audio_client = false;
    let client_index = {
        let mut clients = state.browser_clients.write().await;
        clients.push(tx.clone());
        clients.len() - 1
    };

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
                debug!("Received from {}: {}", addr, json);

                // Try to parse as IPC request for structured routing
                let parsed_req = serde_json::from_str::<IpcRequest>(&json);

                if let Ok(ref req) = parsed_req {
                    // Handle registerAudio
                    if req.method == METHOD_REGISTER_AUDIO {
                        is_audio_client = true;
                        info!("Audio client registered: {}", addr);

                        // Parse to extract client_id
                        if let Some(params) = req.params.clone()
                            && let Ok(audio_params) = serde_json::from_value::<
                                wavecraft_protocol::RegisterAudioParams,
                            >(params)
                        {
                            *state.audio_client.write().await =
                                Some(audio_params.client_id.clone());
                        }

                        // Send success response using the request's id
                        let response = wavecraft_protocol::IpcResponse::success(
                            req.id.clone(),
                            wavecraft_protocol::RegisterAudioResult {
                                status: "registered".to_string(),
                            },
                        );
                        let response_json = match serde_json::to_string(&response) {
                            Ok(json) => json,
                            Err(e) => {
                                error!("Failed to serialize registerAudio response: {}", e);
                                break;
                            }
                        };
                        if let Err(e) = tx.try_send(response_json) {
                            error!("Error sending response: {}", e);
                            break;
                        }
                        continue;
                    }

                    // Handle meterUpdate from audio client
                    if is_audio_client && req.method == NOTIFICATION_METER_UPDATE {
                        // Broadcast to all browser clients
                        broadcast_to_browser_clients(
                            &state,
                            &json,
                            Some(client_index),
                            "broadcast meter update",
                        )
                        .await;
                        continue;
                    }
                }

                // Route through existing IpcHandler
                let response = handler.handle_json(&json);

                // Mirror native editor behavior in dev mode: after successful
                // setParameter, emit parameterChanged so hooks relying on
                // notifications stay in sync with backend-confirmed state.
                if let Ok(req) = &parsed_req
                    && let Some(notification_json) =
                        build_set_parameter_notification(req, &response)
                {
                    broadcast_to_browser_clients(
                        &state,
                        &notification_json,
                        None,
                        "send parameterChanged notification",
                    )
                    .await;
                }

                // Log outgoing response
                debug!("Sending to {}: {}", addr, response);

                // Send response
                if let Err(e) = tx.try_send(response) {
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
    state
        .browser_clients
        .write()
        .await
        .retain(|c| !c.is_closed());
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
    use wavecraft_bridge::InMemoryParameterHost;
    use wavecraft_protocol::{IpcRequest, IpcResponse, ParameterInfo, ParameterType, RequestId};

    /// Simple test host for unit tests
    fn test_host() -> InMemoryParameterHost {
        InMemoryParameterHost::new(vec![ParameterInfo {
            id: "gain".to_string(),
            name: "Gain".to_string(),
            param_type: ParameterType::Float,
            value: 0.5,
            default: 0.5,
            min: 0.0,
            max: 1.0,
            unit: Some("dB".to_string()),
            group: Some("Input".to_string()),
            variants: None,
        }])
    }

    #[tokio::test]
    async fn test_server_creation() {
        let host = test_host();
        let handler = Arc::new(IpcHandler::new(host));
        let server = WsServer::new(9001, handler);

        // Just verify we can create a server without panicking
        assert_eq!(server.port, 9001);
    }

    #[test]
    fn build_set_parameter_notification_from_success_response() {
        let request = IpcRequest::new(
            RequestId::Number(1),
            wavecraft_protocol::METHOD_SET_PARAMETER,
            Some(serde_json::json!({ "id": "gain", "value": 0.8 })),
        );
        let response = serde_json::to_string(&IpcResponse::success(
            RequestId::Number(1),
            serde_json::json!({}),
        ))
        .expect("serialize response");

        let notification = build_set_parameter_notification(&request, &response)
            .expect("should create parameterChanged notification");
        let json: serde_json::Value =
            serde_json::from_str(&notification).expect("notification should parse");

        assert_eq!(
            json.get("method"),
            Some(&serde_json::json!(
                wavecraft_protocol::NOTIFICATION_PARAMETER_CHANGED
            ))
        );
        assert_eq!(json.pointer("/params/id"), Some(&serde_json::json!("gain")));
        let Some(value) = json
            .pointer("/params/value")
            .and_then(serde_json::Value::as_f64)
        else {
            panic!("notification should contain numeric params.value");
        };
        assert!(
            (value - 0.8).abs() < 1e-5,
            "expected approx 0.8, got {value}"
        );
    }

    #[test]
    fn build_set_parameter_notification_ignores_error_response() {
        let request = IpcRequest::new(
            RequestId::Number(1),
            wavecraft_protocol::METHOD_SET_PARAMETER,
            Some(serde_json::json!({ "id": "gain", "value": 10.0 })),
        );

        let response = serde_json::to_string(&IpcResponse::error(
            RequestId::Number(1),
            wavecraft_protocol::IpcError::invalid_params("out of range"),
        ))
        .expect("serialize error response");

        assert!(build_set_parameter_notification(&request, &response).is_none());
    }
}
