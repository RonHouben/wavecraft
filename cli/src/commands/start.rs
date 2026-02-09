//! Development server command - starts WebSocket + UI dev servers.
//!
//! This command provides the development experience for wavecraft plugins:
//! 1. Builds the plugin in debug mode
//! 2. Loads parameter metadata via FFI from the compiled dylib
//! 3. Optionally loads the audio processor vtable and runs audio in-process
//! 4. Starts an embedded WebSocket server for browser UI communication
//! 5. Starts the Vite dev server for UI hot-reloading

use anyhow::{Context, Result};
use console::style;
use std::io::{self, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::dev_server::{DevServerHost, PluginLoader};
use crate::project::{has_node_modules, ProjectMarkers};
use wavecraft_bridge::IpcHandler;
use wavecraft_dev_server::ws_server::WsServer;
use wavecraft_protocol::ParameterInfo;

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

/// Try to start audio processing in-process via FFI vtable.
///
/// - If the plugin exports the vtable symbol: creates an `FfiProcessor`,
///   starts audio capture via cpal, and returns the audio handle.
/// - If the symbol is missing (older SDK): logs info and returns `None`.
/// - If audio init fails (no microphone, etc.): logs warning and returns `None`.
#[cfg(feature = "audio-dev")]
fn try_start_audio_in_process(
    loader: &PluginLoader,
    ws_handle: wavecraft_dev_server::ws_server::WsHandle,
    param_bridge: std::sync::Arc<wavecraft_dev_server::atomic_params::AtomicParameterBridge>,
    verbose: bool,
) -> Option<wavecraft_dev_server::audio_server::AudioHandle> {
    use wavecraft_dev_server::audio_server::{AudioConfig, AudioServer};
    use wavecraft_dev_server::ffi_processor::FfiProcessor;

    println!();
    println!("{} Checking for audio processor...", style("→").cyan());

    let vtable = match loader.dev_processor_vtable() {
        Some(vt) => {
            println!("{} Audio processor vtable loaded", style("✓").green());
            vt
        }
        None => {
            println!(
                "{}",
                style("ℹ Audio processor not available (plugin may use older SDK)").blue()
            );
            println!("  Continuing without audio processing...");
            println!();
            return None;
        }
    };

    let processor = match FfiProcessor::new(vtable) {
        Some(p) => p,
        None => {
            println!(
                "{}",
                style("⚠ Failed to create audio processor (create returned null)").yellow()
            );
            println!("  Continuing without audio processing...");
            println!();
            return None;
        }
    };

    let config = AudioConfig {
        sample_rate: 44100.0,
        buffer_size: 512,
    };

    let server = match AudioServer::new(Box::new(processor), config, param_bridge) {
        Ok(s) => s,
        Err(e) => {
            if verbose {
                println!(
                    "{}",
                    style(format!("⚠ Audio init failed: {:#}", e)).yellow()
                );
            } else {
                println!("{}", style("⚠ No audio input device available").yellow());
            }
            println!("  Continuing without audio...");
            println!();
            return None;
        }
    };

    let has_output = server.has_output();

    // Start audio server. Returns a lock-free ring buffer consumer for
    // meter data (RT-safe: audio thread writes without allocations).
    let (handle, mut meter_consumer) = match server.start() {
        Ok((h, c)) => (h, c),
        Err(e) => {
            println!(
                "{}",
                style(format!("⚠ Failed to start audio: {}", e)).yellow()
            );
            println!("  Continuing without audio...");
            println!();
            return None;
        }
    };

    // Spawn a task that drains the lock-free meter ring buffer and
    // forwards updates to WebSocket clients.
    tokio::spawn(async move {
        use wavecraft_protocol::{IpcNotification, NOTIFICATION_METER_UPDATE};

        let mut interval = tokio::time::interval(std::time::Duration::from_millis(16));
        loop {
            interval.tick().await;
            // Drain all available meter frames, keeping only the latest.
            let mut latest = None;
            while let Ok(notification) = meter_consumer.pop() {
                latest = Some(notification);
            }
            if let Some(notification) = latest {
                if let Ok(json) = serde_json::to_string(&IpcNotification::new(
                    NOTIFICATION_METER_UPDATE,
                    notification,
                )) {
                    ws_handle.broadcast(&json).await;
                }
            }
        }
    });

    if has_output {
        println!(
            "{} Audio server started — full-duplex (input + output)",
            style("✓").green()
        );
    } else {
        println!(
            "{} Audio server started — input-only (metering)",
            style("✓").green()
        );
    }
    println!();
    Some(handle)
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

/// Path to the sidecar parameter cache file.
fn sidecar_json_path(engine_dir: &Path) -> Result<PathBuf> {
    let debug_dir = resolve_debug_dir(engine_dir)?;
    Ok(debug_dir.join("wavecraft-params.json"))
}

/// Try reading cached parameters from the sidecar JSON file.
///
/// Returns `Some(params)` if the file exists and is newer than the dylib
/// (i.e., no source changes since last extraction). Returns `None` otherwise.
fn try_read_cached_params(engine_dir: &Path, verbose: bool) -> Option<Vec<ParameterInfo>> {
    let sidecar_path = sidecar_json_path(engine_dir).ok()?;
    if !sidecar_path.exists() {
        return None;
    }

    // Check if sidecar is still valid (newer than any source file change)
    let dylib_path = find_plugin_dylib(engine_dir).ok()?;
    let sidecar_mtime = std::fs::metadata(&sidecar_path).ok()?.modified().ok()?;
    let dylib_mtime = std::fs::metadata(&dylib_path).ok()?.modified().ok()?;

    if dylib_mtime > sidecar_mtime {
        if verbose {
            println!("  Sidecar cache stale (dylib newer), rebuilding...");
        }
        return None;
    }

    PluginLoader::load_params_from_file(&sidecar_path).ok()
}

/// Write parameter metadata to the sidecar JSON cache.
fn write_sidecar_cache(engine_dir: &Path, params: &[ParameterInfo]) -> Result<()> {
    let sidecar_path = sidecar_json_path(engine_dir)?;
    let json =
        serde_json::to_string_pretty(params).context("Failed to serialize parameters")?;
    std::fs::write(&sidecar_path, json).context("Failed to write sidecar cache")?;
    Ok(())
}

/// Load plugin parameters using cached sidecar or feature-gated build.
///
/// Attempts in order:
/// 1. Read cached `wavecraft-params.json` (instant, no build)
/// 2. Build with `_param-discovery` feature (no nih-plug static init)
/// 3. Fall back to normal build + FFI load (for older plugins)
fn load_parameters(
    engine_dir: &Path,
    verbose: bool,
) -> Result<(Vec<ParameterInfo>, Option<PluginLoader>)> {
    // 1. Try cached sidecar
    if let Some(params) = try_read_cached_params(engine_dir, verbose) {
        println!(
            "{} Loaded {} parameters (cached)",
            style("✓").green(),
            params.len()
        );
        return Ok((params, None));
    }

    // 2. Build with _param-discovery feature (skip nih-plug exports)
    println!(
        "{} Building for parameter discovery...",
        style("→").cyan()
    );
    let build_result = Command::new("cargo")
        .args(["build", "--lib", "--features", "_param-discovery"])
        .current_dir(engine_dir)
        .stdout(if verbose {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
        .stderr(Stdio::inherit())
        .status();

    match build_result {
        Ok(status) if status.success() => {
            // Discovery build succeeded — load params from safe dylib
            let dylib_path = find_plugin_dylib(engine_dir)
                .context("Failed to find plugin library after discovery build")?;

            if verbose {
                println!("  Found dylib: {}", dylib_path.display());
            }

            println!(
                "{} Loading plugin parameters...",
                style("→").cyan()
            );
            let loader = PluginLoader::load(&dylib_path)
                .context("Failed to load plugin for parameter discovery")?;
            let params = loader.parameters().to_vec();

            // Write sidecar cache for next run
            if let Err(e) = write_sidecar_cache(engine_dir, &params) {
                if verbose {
                    println!("  Warning: failed to write param cache: {}", e);
                }
            }

            println!(
                "{} Loaded {} parameters",
                style("✓").green(),
                params.len()
            );
            Ok((params, Some(loader)))
        }
        _ => {
            // 3. Fallback: normal build (for older plugins without _param-discovery)
            if verbose {
                println!("  Discovery build failed, falling back to standard build...");
            }
            println!("{} Building plugin...", style("→").cyan());
            let fallback_status = Command::new("cargo")
                .args(["build", "--lib"])
                .current_dir(engine_dir)
                .stdout(if verbose {
                    Stdio::inherit()
                } else {
                    Stdio::null()
                })
                .stderr(Stdio::inherit())
                .status()
                .context("Failed to run cargo build")?;

            if !fallback_status.success() {
                anyhow::bail!("Plugin build failed. Please fix the errors above.");
            }

            let dylib_path = find_plugin_dylib(engine_dir)?;
            println!(
                "{} Loading plugin parameters...",
                style("→").cyan()
            );
            let loader = PluginLoader::load(&dylib_path)
                .context("Failed to load plugin")?;
            let params = loader.parameters().to_vec();
            println!(
                "{} Loaded {} parameters",
                style("✓").green(),
                params.len()
            );
            Ok((params, Some(loader)))
        }
    }
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

    ensure_port_available(ws_port, "WebSocket server", "--port")?;
    ensure_port_available(ui_port, "UI dev server", "--ui-port")?;

    // 1. Build the plugin and load parameters (two-phase or cached)
    let (params, loader) = load_parameters(&project.engine_dir, verbose)?;

    if verbose {
        for param in &params {
            println!(
                "  - {}: {} ({})",
                param.id,
                param.name,
                param.group.as_deref().unwrap_or("ungrouped")
            );
        }
    }

    // 2. Create AtomicParameterBridge for lock-free audio-thread param reads
    #[cfg(feature = "audio-dev")]
    let param_bridge = {
        use wavecraft_dev_server::atomic_params::AtomicParameterBridge;
        std::sync::Arc::new(AtomicParameterBridge::new(&params))
    };

    // 3. Start embedded WebSocket server
    println!(
        "{} Starting WebSocket server on port {}...",
        style("→").cyan(),
        ws_port
    );

    #[cfg(feature = "audio-dev")]
    let host = DevServerHost::with_param_bridge(params, std::sync::Arc::clone(&param_bridge));
    #[cfg(not(feature = "audio-dev"))]
    let host = DevServerHost::new(params);

    let handler = std::sync::Arc::new(IpcHandler::new(host));

    // Create tokio runtime for the WebSocket server
    let runtime = tokio::runtime::Runtime::new().context("Failed to create async runtime")?;

    let server = WsServer::new(ws_port, handler.clone(), verbose);
    runtime.block_on(async { server.start().await.map_err(|e| anyhow::anyhow!("{}", e)) })?;

    println!("{} WebSocket server running", style("✓").green());

    // 4. Try to start audio in-process via FFI (optional, graceful fallback)
    // Store the AudioHandle so the cpal stream stays alive until shutdown.
    // When this variable is dropped (reverse declaration order for locals),
    // the FfiProcessor inside the closure is dropped while the Library in
    // `loader` is still loaded — preserving vtable pointer validity.
    #[cfg(feature = "audio-dev")]
    let _audio_handle = runtime.block_on(async {
        let ws_handle = server.handle();
        match &loader {
            Some(l) => try_start_audio_in_process(l, ws_handle, param_bridge.clone(), verbose),
            None => {
                // Loaded from cache — no loader available, skip audio
                if verbose {
                    println!("  Skipping audio (params loaded from cache, no dylib loaded)");
                }
                None
            }
        }
    });
    #[cfg(feature = "audio-dev")]
    let has_audio = _audio_handle.is_some();
    #[cfg(not(feature = "audio-dev"))]
    let has_audio = false;

    // 5. Start UI dev server
    println!(
        "{} Starting UI dev server on port {}...",
        style("→").cyan(),
        ui_port
    );

    let ui_port_str = format!("--port={}", ui_port);
    let mut ui_server = Command::new("npm")
        .args(["run", "dev", "--", &ui_port_str, "--strictPort"])
        .current_dir(&project.ui_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start UI dev server")?;

    // Give the UI server a moment to fail fast (e.g., port already in use).
    thread::sleep(Duration::from_millis(300));
    if let Some(status) = ui_server
        .try_wait()
        .context("Failed to check UI dev server status")?
    {
        anyhow::bail!("UI dev server exited early with status {}", status);
    }

    // Print success message
    println!();
    println!("{}", style("✓ All servers running!").green().bold());
    println!();
    println!("  WebSocket: ws://127.0.0.1:{}", ws_port);
    println!("  UI:        http://localhost:{}", ui_port);
    if has_audio {
        println!("  Audio:     Real-time OS input (in-process FFI)");
    }
    println!();
    println!("{}", style("Press Ctrl+C to stop").dim());
    println!();

    // Wait for shutdown (keeps runtime alive)
    wait_for_shutdown(ui_server, runtime)
}

fn ensure_port_available(port: u16, label: &str, flag: &str) -> Result<()> {
    match TcpListener::bind(("0.0.0.0", port)) {
        Ok(listener) => {
            drop(listener);
            Ok(())
        }
        Err(err) => anyhow::bail!(
            "{} port {} is already in use ({}). Stop the process using it or run `wavecraft start {}` with a free port.",
            label,
            port,
            err,
            flag
        ),
    }
}

/// Find the plugin dylib in the target directory.
///
/// Searches for `.dylib` (macOS), `.so` (Linux), or `.dll` (Windows)
/// files in `engine/target/debug/`.
fn find_plugin_dylib(engine_dir: &Path) -> Result<PathBuf> {
    let debug_dir = resolve_debug_dir(engine_dir)?;

    // Look for library files with platform-specific extensions
    #[cfg(target_os = "macos")]
    let extension = "dylib";
    #[cfg(target_os = "linux")]
    let extension = "so";
    #[cfg(target_os = "windows")]
    let extension = "dll";

    // Find library files (skip deps/ subdirectory)
    let entries = std::fs::read_dir(&debug_dir).context("Failed to read debug directory")?;

    let candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|ext| ext == extension)
                && p.file_name().is_some_and(|n| {
                    let name = n.to_string_lossy();
                    if cfg!(target_os = "windows") {
                        !name.starts_with("lib")
                    } else {
                        name.starts_with("lib")
                    }
                })
        })
        .collect();

    if candidates.is_empty() {
        anyhow::bail!(
            "No plugin library found in {}.\n\
             Make sure the engine crate has `crate-type = [\"cdylib\"]` in Cargo.toml.",
            debug_dir.display()
        );
    }

    // Prefer the dylib that matches the engine crate name
    if let Some(crate_name) = read_engine_crate_name(engine_dir) {
        let expected_stem = crate_name.replace('-', "_");
        if let Some(matched) = candidates
            .iter()
            .find(|p| library_matches_name(p, &expected_stem, extension))
        {
            return Ok(matched.to_path_buf());
        }
    }

    if candidates.len() == 1 {
        return Ok(candidates.into_iter().next().unwrap());
    }

    // Multiple libraries - pick the one most recently modified
    let mut sorted = candidates;
    sorted.sort_by_key(|p| {
        std::fs::metadata(p)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    Ok(sorted.pop().unwrap())
}

fn resolve_debug_dir(engine_dir: &Path) -> Result<PathBuf> {
    let engine_debug = engine_dir.join("target").join("debug");
    if engine_debug.exists() {
        return Ok(engine_debug);
    }

    let workspace_debug = engine_dir.parent().map(|p| p.join("target").join("debug"));

    if let Some(debug_dir) = workspace_debug {
        if debug_dir.exists() {
            return Ok(debug_dir);
        }
    }

    anyhow::bail!(
        "Build output directory not found. Tried:\n  - {}\n  - {}\n\
         Run `cargo build` first.",
        engine_debug.display(),
        engine_dir
            .parent()
            .map(|p| p.join("target").join("debug").display().to_string())
            .unwrap_or_else(|| "<workspace root unavailable>".to_string())
    );
}

fn read_engine_crate_name(engine_dir: &Path) -> Option<String> {
    let cargo_toml_path = engine_dir.join("Cargo.toml");
    let contents = std::fs::read_to_string(cargo_toml_path).ok()?;
    let manifest: toml::Value = toml::from_str(&contents).ok()?;

    let lib_name = manifest
        .get("lib")
        .and_then(|lib| lib.get("name"))
        .and_then(|name| name.as_str())
        .map(|name| name.to_string());

    if lib_name.is_some() {
        return lib_name;
    }

    manifest
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(|name| name.as_str())
        .map(|name| name.to_string())
}

fn library_matches_name(path: &Path, expected_stem: &str, extension: &str) -> bool {
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    if cfg!(target_os = "windows") {
        file_name.eq_ignore_ascii_case(&format!("{}.{}", expected_stem, extension))
    } else {
        file_name.eq_ignore_ascii_case(&format!("lib{}.{}", expected_stem, extension))
    }
}

/// Set up Ctrl+C handler and wait for shutdown.
///
/// Audio runs in-process (via FFI) on the tokio runtime's thread pool,
/// so dropping the runtime is sufficient to stop audio. Only the UI
/// child process needs explicit cleanup.
fn wait_for_shutdown(mut ui_server: Child, _runtime: tokio::runtime::Runtime) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })
    .context("Failed to set Ctrl+C handler")?;

    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(_) => {
                println!();
                println!("{} Shutting down servers...", style("→").cyan());
                kill_process(ui_server)?;
                println!("{} Servers stopped", style("✓").green());
                return Ok(());
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Check UI server
                if let Some(status) = ui_server
                    .try_wait()
                    .context("Failed to check UI dev server status")?
                {
                    println!();
                    println!(
                        "{} UI dev server exited unexpectedly ({}).",
                        style("✗").red(),
                        status
                    );
                    println!("{} Shutting down servers...", style("→").cyan());
                    println!("{} Servers stopped", style("✓").green());
                    return Err(anyhow::anyhow!(
                        "UI dev server exited unexpectedly with status {}",
                        status
                    ));
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!();
                println!("{} Shutting down servers...", style("→").cyan());
                kill_process(ui_server)?;
                println!("{} Servers stopped", style("✓").green());
                return Ok(());
            }
        }
    }
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
