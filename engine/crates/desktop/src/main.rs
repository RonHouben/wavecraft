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
        println!("    --list-assets    List all embedded UI assets");
        println!("    --help, -h       Show this help message");
        return Ok(());
    }

    if args.contains(&"--list-assets".to_string()) {
        println!("Embedded UI assets:");
        for asset in assets::list_assets() {
            println!("  {}", asset);
        }
        return Ok(());
    }

    // Create application state
    let state = Arc::new(AppState::new());

    // Run the app
    println!("Starting VstKit Desktop POC...");
    webview::run_app(state)
}
