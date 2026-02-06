//! Development server command - starts WebSocket + UI dev servers.
//!
//! This command provides the development experience for wavecraft plugins:
//! 1. Builds the plugin in debug mode
//! 2. Loads parameter metadata via FFI from the compiled dylib
//! 3. Starts an embedded WebSocket server for browser UI communication
//! 4. Starts the Vite dev server for UI hot-reloading

use anyhow::{Context, Result};
use console::style;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::dev_server::{DevServerHost, PluginLoader};
use crate::project::{has_node_modules, ProjectMarkers};
use standalone::ws_server::WsServer;
use wavecraft_bridge::IpcHandler;

/// Options for the `start` command.
#[derive(Debug)]
pub struct StartCommand {
    /// WebSocket server port
    pub port: u16,
    /// Vite UI server port
    pub ui_port: u16,
    /// Auto-install dependencies without prompting
    pub install: bool,
    /// Fail if dependencies are missing (no prompt)
    pub no_install: bool,
    /// Show verbose output
    pub verbose: bool,
}

impl StartCommand {
    pub fn execute(&self) -> Result<()> {
        // 1. Detect project
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let project = ProjectMarkers::detect(&cwd)?;

        // 2. Check dependencies
        if !has_node_modules(&project) {
            if self.no_install {
                anyhow::bail!(
                    "Dependencies not installed. Run `npm install` in the ui/ directory,\n\
                     or use `wavecraft start --install` to install automatically."
                );
            }

            let should_install = if self.install {
                true
            } else {
                prompt_install()?
            };

            if should_install {
                install_dependencies(&project)?;
            } else {
                anyhow::bail!("Cannot start without dependencies. Run `npm install` in ui/ first.");
            }
        }

        // 3. Start servers
        run_dev_servers(&project, self.port, self.ui_port, self.verbose)
    }
}

/// Prompt user to install dependencies.
fn prompt_install() -> Result<bool> {
    print!(
        "{} Dependencies not installed. Run npm install? [Y/n] ",
        style("?").cyan().bold()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let response = input.trim().to_lowercase();
    Ok(response.is_empty() || response == "y" || response == "yes")
}

/// Install npm dependencies in the ui/ directory.
fn install_dependencies(project: &ProjectMarkers) -> Result<()> {
    println!("{} Installing dependencies...", style("→").cyan());

    let status = Command::new("npm")
        .args(["install"])
        .current_dir(&project.ui_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run npm install. Is npm installed and in your PATH?")?;

    if !status.success() {
        anyhow::bail!("npm install failed. Please check the output above and try again.");
    }

    println!("{} Dependencies installed", style("✓").green());
    Ok(())
}

/// Run both development servers.
fn run_dev_servers(
    project: &ProjectMarkers,
    ws_port: u16,
    ui_port: u16,
    verbose: bool,
) -> Result<()> {
    println!();
    println!(
        "{}",
        style("Starting Wavecraft Development Servers")
            .cyan()
            .bold()
    );
    println!();

    // 1. Build the plugin in debug mode
    println!("{} Building plugin...", style("→").cyan());
    let build_status = Command::new("cargo")
        .args(["build", "--lib"])
        .current_dir(&project.engine_dir)
        .stdout(if verbose {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run cargo build. Is Rust installed?")?;

    if !build_status.success() {
        anyhow::bail!("Plugin build failed. Please fix the errors above.");
    }
    println!("{} Plugin built", style("✓").green());

    // 2. Find the plugin dylib
    let dylib_path = find_plugin_dylib(&project.engine_dir)
        .context("Failed to find compiled plugin library")?;

    if verbose {
        println!("  Found dylib: {}", dylib_path.display());
    }

    // 3. Load parameters via FFI
    println!("{} Loading plugin parameters...", style("→").cyan());
    let loader = PluginLoader::load(&dylib_path)
        .context("Failed to load plugin. Make sure it's compiled with wavecraft SDK.")?;

    let params = loader.parameters().to_vec();
    println!(
        "{} Loaded {} parameters",
        style("✓").green(),
        params.len()
    );

    if verbose {
        for param in &params {
            println!("  - {}: {} ({})", param.id, param.name, param.group.as_deref().unwrap_or("ungrouped"));
        }
    }

    // 4. Start embedded WebSocket server
    println!(
        "{} Starting WebSocket server on port {}...",
        style("→").cyan(),
        ws_port
    );

    let host = DevServerHost::new(params);
    let handler = Arc::new(IpcHandler::new(host));

    // Create tokio runtime for the WebSocket server
    let runtime = tokio::runtime::Runtime::new()
        .context("Failed to create async runtime")?;

    let server = WsServer::new(ws_port, handler.clone(), verbose);
    runtime.block_on(async {
        server.start().await.map_err(|e| anyhow::anyhow!("{}", e))
    })?;

    println!("{} WebSocket server running", style("✓").green());

    // 5. Start UI dev server
    println!(
        "{} Starting UI dev server on port {}...",
        style("→").cyan(),
        ui_port
    );

    let ui_port_str = format!("--port={}", ui_port);
    let ui_server = Command::new("npm")
        .args(["run", "dev", "--", &ui_port_str])
        .current_dir(&project.ui_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start UI dev server")?;

    // Print success message
    println!();
    println!("{}", style("✓ Both servers running!").green().bold());
    println!();
    println!("  WebSocket: ws://127.0.0.1:{}", ws_port);
    println!("  UI:        http://localhost:{}", ui_port);
    println!();
    println!("{}", style("Press Ctrl+C to stop").dim());
    println!();

    // Wait for shutdown (keeps runtime alive)
    wait_for_shutdown(ui_server, runtime)
}

/// Find the plugin dylib in the target directory.
///
/// Searches for `.dylib` (macOS), `.so` (Linux), or `.dll` (Windows)
/// files in `engine/target/debug/`.
fn find_plugin_dylib(engine_dir: &std::path::Path) -> Result<PathBuf> {
    let debug_dir = engine_dir.join("target").join("debug");

    if !debug_dir.exists() {
        anyhow::bail!(
            "Build output directory not found: {}\nRun `cargo build` first.",
            debug_dir.display()
        );
    }

    // Look for library files with platform-specific extensions
    #[cfg(target_os = "macos")]
    let extension = "dylib";
    #[cfg(target_os = "linux")]
    let extension = "so";
    #[cfg(target_os = "windows")]
    let extension = "dll";

    // Find .dylib files (skip deps/ subdirectory)
    let entries = std::fs::read_dir(&debug_dir)
        .context("Failed to read debug directory")?;

    let candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|ext| ext == extension)
                && p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("lib"))
        })
        .collect();

    match candidates.len() {
        0 => anyhow::bail!(
            "No plugin library found in {}.\n\
             Make sure the engine crate has `crate-type = [\"cdylib\"]` in Cargo.toml.",
            debug_dir.display()
        ),
        1 => Ok(candidates.into_iter().next().unwrap()),
        _ => {
            // Multiple libraries - pick the one most recently modified
            let mut sorted = candidates;
            sorted.sort_by_key(|p| {
                std::fs::metadata(p)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            });
            Ok(sorted.pop().unwrap())
        }
    }
}

/// Set up Ctrl+C handler and wait for shutdown.
fn wait_for_shutdown(ui_server: Child, _runtime: tokio::runtime::Runtime) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .context("Failed to set Ctrl+C handler")?;

    // Wait for Ctrl+C
    let _ = rx.recv();

    println!();
    println!("{} Shutting down servers...", style("→").cyan());

    // Kill UI server
    kill_process(ui_server)?;

    // Runtime is dropped when it goes out of scope, which stops the WebSocket server
    println!("{} Servers stopped", style("✓").green());
    Ok(())
}

/// Kill a child process gracefully.
#[cfg(unix)]
fn kill_process(mut child: Child) -> Result<()> {
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;

    let pid = child.id();
    // Send SIGTERM to process group (negative PID kills the group)
    let _ = kill(Pid::from_raw(-(pid as i32)), Signal::SIGTERM);
    thread::sleep(Duration::from_millis(500));
    // Force kill if still running
    let _ = child.kill();
    Ok(())
}

/// Kill a child process on Windows.
#[cfg(windows)]
fn kill_process(mut child: Child) -> Result<()> {
    let _ = child.kill();
    Ok(())
}
