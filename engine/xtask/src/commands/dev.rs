//! Development server command - runs `wavecraft start` via the CLI.

use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use xtask::output::*;
use xtask::paths;

/// Run the development server via `wavecraft start` CLI command.
///
/// This command invokes the Wavecraft CLI which manages:
/// 1. WebSocket server (Rust) for IPC
/// 2. Vite UI dev server
/// 3. Hot-reload pipeline (file watcher + rebuild)
/// 4. Optional in-process audio
///
/// # Arguments
/// * `port` - WebSocket server port (default: 9000)
/// * `verbose` - Show detailed output
pub fn run(port: u16, verbose: bool) -> Result<()> {
    print_header("Wavecraft Development Server");

    // Locate the CLI manifest relative to the engine directory
    let engine_dir = paths::engine_dir()?;
    let cli_manifest = engine_dir.join("../cli/Cargo.toml");

    let mut args = vec!["run", "--manifest-path"];
    let cli_manifest_str = cli_manifest.to_string_lossy().to_string();
    args.push(&cli_manifest_str);
    args.push("--features");
    args.push("audio-dev");
    args.push("--");
    args.push("start");

    let port_str = port.to_string();
    args.push("--port");
    args.push(&port_str);

    if verbose {
        args.push("--verbose");
    }

    println!();
    print_status(&format!("Starting wavecraft start (port {})", port));
    println!();

    let mut child = Command::new("cargo")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start wavecraft CLI")?;

    // Set up Ctrl+C handler
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel");
    })
    .context("Error setting Ctrl+C handler")?;

    // Wait for Ctrl+C or CLI process to exit
    let child_result = thread::spawn(move || child.wait());

    match rx.recv_timeout(std::time::Duration::from_millis(100)) {
        Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => {
            println!();
            print_status("Shutting down...");
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            if child_result.is_finished() {
                // CLI exited on its own
            } else {
                let _ = rx.recv();
                println!();
                print_status("Shutting down...");
            }
        }
    }

    print_success("Development server stopped");
    Ok(())
}
