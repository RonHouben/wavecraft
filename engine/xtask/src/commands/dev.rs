//! Development server command - runs both WebSocket and UI dev servers.

use anyhow::{Context, Result};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use xtask::output::*;
use xtask::paths;

/// Run both WebSocket and UI dev servers concurrently.
///
/// This command:
/// 1. Starts the WebSocket server (Rust) on the specified port
/// 2. Starts the Vite UI dev server (npm)
/// 3. Handles Ctrl+C to shut down both servers cleanly
///
/// # Arguments
/// * `port` - WebSocket server port (default: 9000)
/// * `verbose` - Show detailed output
pub fn run(port: u16, verbose: bool) -> Result<()> {
    print_header("VstKit Development Servers");

    // Get paths to engine and UI directories
    let engine_dir = paths::engine_dir()?;
    let ui_dir = paths::ui_dir()?;

    println!();
    print_status(&format!("Starting WebSocket server on port {}", port));

    // Start WebSocket server in background
    let port_str = port.to_string();
    let mut ws_args = vec![
        "run",
        "-p",
        "standalone",
        "--release",
        "--",
        "--dev-server",
        "--port",
        &port_str,
    ];
    if verbose {
        ws_args.push("--verbose");
    }
    let ws_server = Command::new("cargo")
        .args(&ws_args)
        .current_dir(&engine_dir)
        .stdout(Stdio::inherit()) // Always show connection messages
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start WebSocket server")?;

    // Give the server a moment to start
    thread::sleep(std::time::Duration::from_millis(500));

    println!();
    print_status("Starting UI dev server on http://localhost:5173");

    // Start UI dev server (this will be in foreground)
    let mut ui_server = Command::new("npm")
        .args(["run", "dev"])
        .current_dir(&ui_dir)
        .spawn()
        .context("Failed to start UI dev server")?;

    println!();
    print_success("Both servers running!");
    println!();
    println!("  WebSocket: ws://127.0.0.1:{}", port);
    println!("  UI:        http://localhost:5173");
    println!();
    println!("Press Ctrl+C to stop both servers");
    println!();

    // Set up Ctrl+C handler
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel");
    })
    .context("Error setting Ctrl+C handler")?;

    // Wait for Ctrl+C or UI server to exit
    let ui_result = thread::spawn(move || ui_server.wait());

    // Block until either Ctrl+C or UI server exits
    match rx.recv_timeout(std::time::Duration::from_millis(100)) {
        Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => {
            // Ctrl+C received, kill both servers
            println!();
            print_status("Shutting down servers...");
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // No Ctrl+C yet, check if UI server is still running
            if ui_result.is_finished() {
                print_status("UI server exited, stopping WebSocket server...");
            } else {
                // Wait indefinitely for Ctrl+C
                let _ = rx.recv();
                println!();
                print_status("Shutting down servers...");
            }
        }
    }

    // Kill both servers
    kill_process_group(ws_server)?;

    print_success("Servers stopped");
    Ok(())
}

/// Kill a process and its children on Unix systems
#[cfg(unix)]
fn kill_process_group(mut child: Child) -> Result<()> {
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;

    let pid = child.id();
    // Kill the process group (negative PID)
    let _ = kill(Pid::from_raw(-(pid as i32)), Signal::SIGTERM);
    thread::sleep(std::time::Duration::from_millis(500));
    let _ = child.kill();
    Ok(())
}

/// Kill a process on Windows
#[cfg(windows)]
fn kill_process_group(mut child: Child) -> Result<()> {
    let _ = child.kill();
    Ok(())
}
