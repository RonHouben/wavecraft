//! Standalone application entry point

use clap::Parser;
use std::sync::Arc;

mod app;
mod assets;
mod webview;
mod ws_server;

use app::AppState;
use bridge::IpcHandler;
use ws_server::WsServer;

/// VstKit Standalone - Audio plugin UI testing tool
#[derive(Parser, Debug)]
#[command(name = "standalone")]
#[command(about = "VstKit standalone app for UI development and testing")]
struct Args {
    /// Run in dev server mode (headless, WebSocket only)
    #[arg(long)]
    dev_server: bool,

    /// WebSocket server port (default: 9000)
    #[arg(long, default_value_t = 9000)]
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.dev_server {
        run_dev_server(args.port)
    } else {
        run_gui_app()
    }
}

/// Run headless dev server mode (WebSocket only, no GUI)
fn run_dev_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting VstKit dev server on port {}...", port);
    println!("Press Ctrl+C to stop");

    // Create tokio runtime
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        // Create application state
        let state = AppState::new();

        // Wrap in IPC handler
        let handler = Arc::new(IpcHandler::new(state));

        // Start WebSocket server
        let server = WsServer::new(port, handler);
        server.start().await
    })
}

/// Run GUI app with native window
fn run_gui_app() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new());

    println!("Starting VstKit Standalone...");
    webview::run_app(state)
}
