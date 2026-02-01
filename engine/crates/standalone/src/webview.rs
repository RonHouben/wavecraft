//! WebView setup and event loop
//!
//! This module creates the desktop window, configures the WebView with
//! embedded assets, sets up IPC communication, and runs the event loop.

use crate::app::AppState;
use crate::assets;
use vstkit_bridge::IpcHandler;
use std::borrow::Cow;
use std::sync::Arc;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

/// IPC primitives JavaScript (injected before React loads)
const IPC_PRIMITIVES_JS: &str = include_str!("js/ipc-primitives.js");

/// Run the desktop application
///
/// Creates a window, embeds the React UI via WebView, and handles IPC communication.
pub fn run_app(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    // Create event loop
    let event_loop = EventLoop::new();

    // Create window
    let window = WindowBuilder::new()
        .with_title("VstKit Desktop POC")
        .with_inner_size(tao::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .map_err(|e| format!("Failed to create window: {}", e))?;

    // Create IPC handler (unwrap Arc to pass AppState directly)
    let handler = IpcHandler::new((*state).clone());

    // Keep a reference to state for potential future use
    let _state = state;

    // Create channel for sending responses back to event loop
    let (response_tx, response_rx) = std::sync::mpsc::channel::<String>();

    // Create WebView
    let webview = WebViewBuilder::new()
        // Inject IPC primitives before any other scripts
        .with_initialization_script(IPC_PRIMITIVES_JS)
        // Register custom protocol for embedded assets
        .with_custom_protocol("vstkit".to_string(), move |_webview, request| {
            handle_asset_request(request)
        })
        // Handle IPC messages from UI
        .with_ipc_handler(move |request: wry::http::Request<String>| {
            let message = request.body();
            let response = handler.handle_json(message);

            // Log for debugging
            eprintln!("[IPC] Request: {}", message);
            eprintln!("[IPC] Response: {}", response);

            // Send response through channel to be delivered in event loop
            let _ = response_tx.send(response);
        })
        .build(&window)?;

    // Navigate to the React app
    webview.load_url("vstkit://localhost/index.html")?;

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        // Poll continuously to process IPC responses
        *control_flow = ControlFlow::Poll;

        // Process any pending IPC responses
        while let Ok(response) = response_rx.try_recv() {
            // Escape the JSON response for JavaScript
            let escaped_response = response
                .replace('\\', "\\\\")
                .replace('\'', "\\'")
                .replace('\n', "\\n")
                .replace('\r', "\\r");

            // Call the internal _receive method that the primitives expose
            let js_code = format!(
                "globalThis.__VSTKIT_IPC__._receive('{}');",
                escaped_response
            );

            if let Err(e) = webview.evaluate_script(&js_code) {
                eprintln!("[IPC] Failed to send response to UI: {}", e);
            }
        }

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}

/// Handle requests for embedded assets via custom protocol
fn handle_asset_request(
    request: wry::http::Request<Vec<u8>>,
) -> wry::http::Response<Cow<'static, [u8]>> {
    let path = request.uri().path();

    match assets::get_asset(path) {
        Some((content, mime_type)) => wry::http::Response::builder()
            .status(200)
            .header("Content-Type", mime_type)
            .header("Access-Control-Allow-Origin", "*")
            .body(Cow::Borrowed(content))
            .unwrap(),
        None => {
            eprintln!("[Assets] Not found: {}", path);
            wry::http::Response::builder()
                .status(404)
                .body(Cow::Borrowed(b"Not Found" as &[u8]))
                .unwrap()
        }
    }
}
