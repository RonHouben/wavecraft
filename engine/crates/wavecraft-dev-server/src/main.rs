//! Wavecraft dev server entry point

use clap::Parser;
use std::sync::Arc;
use tracing::{debug, info};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

mod app;
mod assets;
mod webview;
mod ws_server;

use app::AppState;
use wavecraft_bridge::IpcHandler;
use ws_server::WsServer;

/// Wavecraft dev server - Audio plugin UI testing tool
#[derive(Parser, Debug)]
#[command(name = "wavecraft-dev-server")]
#[command(about = "Wavecraft development server for plugin UI testing")]
struct Args {
    /// Run in dev server mode (headless, WebSocket only)
    #[arg(long)]
    dev_server: bool,

    /// WebSocket server port (default: 9000)
    #[arg(long, default_value_t = 9000)]
    port: u16,

    /// Show verbose output (all JSON-RPC messages)
    #[arg(long)]
    verbose: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber
    // Use RUST_LOG env var for level control, default to info
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    let args = Args::parse();

    if args.dev_server {
        run_dev_server(args.port, args.verbose)
    } else {
        run_gui_app()
    }
}

/// Run headless dev server mode (WebSocket only, no GUI)
fn run_dev_server(port: u16, verbose: bool) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting Wavecraft dev server on port {}", port);
    if verbose {
        debug!("Verbose mode: showing all JSON-RPC messages");
    }
    info!("Press Ctrl+C to stop");

    // Create tokio runtime
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        // Create application state
        let state = AppState::new();

        // Wrap in IPC handler
        let handler = Arc::new(IpcHandler::new(state));

        // Start WebSocket server
        let server = WsServer::new(port, handler, verbose);
        server.start().await?;

        // Wait for Ctrl+C signal
        tokio::signal::ctrl_c().await?;
        info!("Shutting down...");

        Ok(())
    })
}

/// Run GUI app with native window
fn run_gui_app() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::new());

    info!("Starting Wavecraft Dev Server (GUI mode)...");
    webview::run_app(state)
}
