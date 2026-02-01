//! Desktop application entry point

use std::sync::Arc;

mod app;
mod assets;
mod webview;

use app::AppState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("VstKit Desktop POC");
        println!();
        println!("USAGE:");
        println!("    desktop [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("    --help, -h       Show this help message");
        return Ok(());
    }

    // Create application state
    let state = Arc::new(AppState::new());

    // Run the app
    println!("Starting VstKit Desktop POC...");
    webview::run_app(state)
}
