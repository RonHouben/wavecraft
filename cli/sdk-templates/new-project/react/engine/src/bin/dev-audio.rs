//! Development audio server for testing plugins with real OS audio input.
//!
//! This binary processes live microphone input through the same DSP code
//! used in the production plugin, providing real-time testing during development.
//!
//! Usage: `wavecraft start` automatically runs this binary when available.

use {{plugin_name_snake}}::{{plugin_name_pascal}}Gain;
use anyhow::Result;
use wavecraft_dev_server::audio_server::{AudioConfig, AudioServer};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Get WebSocket URL from environment (set by CLI)
    let websocket_url = std::env::var("WAVECRAFT_WS_URL")
        .unwrap_or_else(|_| "ws://127.0.0.1:9000".to_string());

    // Configure audio server
    let config = AudioConfig {
        websocket_url,
        sample_rate: 44100.0,  // Will use system default if available
        buffer_size: 512,
    };

    // Create processor instance (same as used in plugin)
    let processor = {{plugin_name_pascal}}Gain::default();

    // Create and run audio server
    log::info!("Starting audio server...");
    let server = AudioServer::new(processor, config)?;
    
    log::info!("Audio server running. Press Ctrl+C to stop.");
    server.run().await?;

    Ok(())
}
