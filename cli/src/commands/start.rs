//! Development server command - starts WebSocket + UI dev servers.
//!
//! This command provides the development experience for wavecraft plugins:
//! 1. Builds the plugin in debug mode
//! 2. Loads parameter metadata via FFI from the compiled dylib
//! 3. Optionally loads the audio processor vtable and runs audio in-process
//! 4. Starts an embedded WebSocket server for browser UI communication
//! 5. Starts the Vite dev server for UI hot-reloading

use anyhow::{Context, Result};
use command_group::{CommandGroup, GroupChild};
use console::style;
use std::io::{self, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use tokio::sync::watch;

use crate::project::{
    find_plugin_dylib, has_node_modules, read_engine_package_name, resolve_debug_dir,
    ProjectMarkers,
};
use wavecraft_bridge::IpcHandler;
use wavecraft_dev_server::{DevServerHost, DevSession, RebuildCallbacks, WsServer};
use wavecraft_protocol::ParameterInfo;

#[cfg(not(feature = "audio-dev"))]
use crate::project::param_extract::{extract_params_subprocess, DEFAULT_EXTRACT_TIMEOUT};

/// Re-export PluginParamLoader for audio dev mode
use wavecraft_bridge::PluginParamLoader as PluginLoader;

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
    ws_handle: wavecraft_dev_server::WsHandle,
    param_bridge: std::sync::Arc<wavecraft_dev_server::AtomicParameterBridge>,
    verbose: bool,
) -> Option<wavecraft_dev_server::AudioHandle> {
    use wavecraft_dev_server::{AudioConfig, AudioServer, FfiProcessor};

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

    // Load parameters from JSON file (inline to avoid publish dep issues)
    let contents = std::fs::read_to_string(&sidecar_path).ok()?;
    serde_json::from_str(&contents).ok()
}

/// Write parameter metadata to the sidecar JSON cache.
pub(crate) fn write_sidecar_cache(engine_dir: &Path, params: &[ParameterInfo]) -> Result<()> {
    let sidecar_path = sidecar_json_path(engine_dir)?;
    let json = serde_json::to_string_pretty(params).context("Failed to serialize parameters")?;
    std::fs::write(&sidecar_path, json).context("Failed to write sidecar cache")?;
    Ok(())
}

/// Load plugin parameters using cached sidecar or feature-gated build.
///
/// Returns parameter metadata and optionally a PluginLoader (when audio-dev
/// feature is enabled and the vtable is available).
///
/// Two-phase process:
/// 1. Try reading from cached sidecar (fast path)
/// 2. Otherwise: build with _param-discovery, load via subprocess (or in-process for audio-dev), write sidecar
async fn load_parameters(
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
    println!("{} Building for parameter discovery...", style("→").cyan());

    // Get package name for --package flag (targets correct crate with path deps)
    let mut build_cmd = Command::new("cargo");
    build_cmd.args(["build", "--lib", "--features", "_param-discovery"]);

    if let Some(package_name) = read_engine_package_name(engine_dir) {
        build_cmd.args(["--package", &package_name]);
    }

    let build_result = build_cmd
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

            println!("{} Loading plugin parameters...", style("→").cyan());
            #[cfg(feature = "audio-dev")]
            let (params, loader) = {
                let loader = PluginLoader::load(&dylib_path)
                    .context("Failed to load plugin for parameter discovery")?;
                let params = loader.parameters().to_vec();
                (params, Some(loader))
            };
            #[cfg(not(feature = "audio-dev"))]
            let (params, loader) = {
                let params = extract_params_subprocess(&dylib_path, DEFAULT_EXTRACT_TIMEOUT)
                    .await
                    .context("Failed to extract parameters from plugin")?;
                (params, None)
            };

            // Write sidecar cache for next run
            if let Err(e) = write_sidecar_cache(engine_dir, &params) {
                if verbose {
                    println!("  Warning: failed to write param cache: {}", e);
                }
            }

            println!("{} Loaded {} parameters", style("✓").green(), params.len());
            Ok((params, loader))
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
            println!("{} Loading plugin parameters...", style("→").cyan());
            #[cfg(feature = "audio-dev")]
            let (params, loader) = {
                let loader = PluginLoader::load(&dylib_path).context("Failed to load plugin")?;
                let params = loader.parameters().to_vec();
                (params, Some(loader))
            };
            #[cfg(not(feature = "audio-dev"))]
            let (params, loader) = {
                let params = extract_params_subprocess(&dylib_path, DEFAULT_EXTRACT_TIMEOUT)
                    .await
                    .context("Failed to extract parameters from plugin")?;
                (params, None)
            };
            println!("{} Loaded {} parameters", style("✓").green(), params.len());
            Ok((params, loader))
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
    // Create tokio runtime for async parameter loading
    let runtime = tokio::runtime::Runtime::new().context("Failed to create async runtime")?;
    #[cfg_attr(not(feature = "audio-dev"), allow(unused_variables))]
    let (params, loader) = runtime.block_on(load_parameters(&project.engine_dir, verbose))?;

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
        use wavecraft_dev_server::AtomicParameterBridge;
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

    let host = std::sync::Arc::new(host);
    let handler = std::sync::Arc::new(IpcHandler::new(host.clone()));

    // Start WebSocket server (runtime already created above for param loading)
    let server = std::sync::Arc::new(WsServer::new(ws_port, handler.clone(), verbose));
    runtime.block_on(async { server.start().await.map_err(|e| anyhow::anyhow!("{}", e)) })?;

    println!("{} WebSocket server running", style("✓").green());

    // Create shutdown broadcast channel
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // 4. Initialize hot-reload development session
    println!("{} Setting up hot-reload...", style("→").cyan());

    // Create rebuild callbacks: wire CLI-specific functions into the dev-server pipeline
    let callbacks = RebuildCallbacks {
        package_name: read_engine_package_name(&project.engine_dir),
        write_sidecar: Some(std::sync::Arc::new(
            |engine_dir: &Path, params: &[ParameterInfo]| write_sidecar_cache(engine_dir, params),
        )),
        param_loader: std::sync::Arc::new(move |engine_dir: PathBuf| {
            Box::pin(load_parameters_from_dylib(engine_dir))
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<Vec<ParameterInfo>>> + Send>,
                >
        }),
    };

    let dev_session = runtime.block_on(async {
        DevSession::new(
            project.engine_dir.clone(),
            host.clone(),
            server.clone(),
            shutdown_rx,
            callbacks,
            #[cfg(feature = "audio-dev")]
            None, // Audio handle will be added if audio starts
        )
    })?;
    let watched_path = project.engine_dir.join("src");
    let relative_path = watched_path
        .strip_prefix(std::env::current_dir().unwrap_or_default())
        .unwrap_or(&watched_path);
    println!(
        "{} Watching {} for changes",
        style("👀").cyan(),
        relative_path.display()
    );
    println!();

    // 5. Try to start audio in-process via FFI (optional, graceful fallback)
    // Store the AudioHandle so the cpal stream stays alive until shutdown.
    // When this variable is dropped (reverse declaration order for locals),
    // the FfiProcessor inside the closure is dropped while the Library in
    // `loader` is still loaded — preserving vtable pointer validity.
    #[cfg(feature = "audio-dev")]
    let audio_handle = runtime.block_on(async {
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
    let has_audio = audio_handle.is_some();
    #[cfg(not(feature = "audio-dev"))]
    let has_audio = false;

    // 6. Start UI dev server
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
        .group_spawn()
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
    let shutdown_reason = wait_for_shutdown(ui_server, shutdown_tx)?;

    #[cfg(feature = "audio-dev")]
    drop(audio_handle);
    drop(dev_session);
    drop(runtime);

    match shutdown_reason {
        ShutdownReason::UiExited(status) => Err(anyhow::anyhow!(
            "UI dev server exited unexpectedly with status {}",
            status
        )),
        ShutdownReason::UiExitedUnknown => {
            Err(anyhow::anyhow!("UI dev server exited unexpectedly"))
        }
        ShutdownReason::CtrlC | ShutdownReason::ChannelClosed => Ok(()),
    }
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

/// Set up Ctrl+C handler and wait for shutdown.
///
/// Audio runs in-process (via FFI) on the tokio runtime's thread pool,
/// so dropping the runtime is sufficient to stop audio. Only the UI
/// child process needs explicit cleanup.
#[derive(Debug)]
enum ShutdownReason {
    CtrlC,
    UiExited(i32),
    UiExitedUnknown,
    ChannelClosed,
}

fn wait_for_shutdown(
    mut ui_server: GroupChild,
    shutdown_tx: watch::Sender<bool>,
) -> Result<ShutdownReason> {
    let (tx, rx) = mpsc::channel();
    let shutdown_tx_for_handler = shutdown_tx.clone();

    ctrlc::set_handler(move || {
        let _ = shutdown_tx_for_handler.send(true);
        let _ = tx.send(());
    })
    .context("Failed to set Ctrl+C handler")?;

    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(_) => {
                println!();
                println!("{} Shutting down servers...", style("→").cyan());
                let _ = shutdown_tx.send(true);
                kill_process(&mut ui_server)?;
                println!("{} Servers stopped", style("✓").green());
                return Ok(ShutdownReason::CtrlC);
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
                    let _ = shutdown_tx.send(true);
                    println!("{} Servers stopped", style("✓").green());
                    if let Some(code) = status.code() {
                        return Ok(ShutdownReason::UiExited(code));
                    }
                    return Ok(ShutdownReason::UiExitedUnknown);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!();
                println!("{} Shutting down servers...", style("→").cyan());
                let _ = shutdown_tx.send(true);
                kill_process(&mut ui_server)?;
                println!("{} Servers stopped", style("✓").green());
                return Ok(ShutdownReason::ChannelClosed);
            }
        }
    }
}

/// Kill a child process group gracefully.
fn kill_process(child: &mut GroupChild) -> Result<()> {
    let _ = child.kill();
    let _ = child.wait();
    Ok(())
}

/// Load parameters from the rebuilt dylib via subprocess isolation.
///
/// To avoid dylib caching issues on macOS, we copy the dylib to a unique
/// temporary location before loading. The subprocess extracts parameters
/// and exits cleanly, with the temp file deleted after.
///
/// This function stays in the CLI because it depends on CLI-specific
/// infrastructure: `find_plugin_dylib`, `extract_params_subprocess`,
/// and `create_temp_dylib_copy`.
async fn load_parameters_from_dylib(engine_dir: PathBuf) -> Result<Vec<ParameterInfo>> {
    use crate::project::param_extract::{extract_params_subprocess, DEFAULT_EXTRACT_TIMEOUT};

    println!("  {} Finding plugin dylib...", style("→").dim());
    let lib_path =
        find_plugin_dylib(&engine_dir).context("Failed to find plugin dylib after rebuild")?;
    println!("  {} Found: {}", style("→").dim(), lib_path.display());

    println!("  {} Copying to temp location...", style("→").dim());
    let temp_path = create_temp_dylib_copy(&lib_path)?;
    println!("  {} Temp: {}", style("→").dim(), temp_path.display());

    println!(
        "  {} Loading parameters via subprocess...",
        style("→").dim()
    );
    let params = extract_params_subprocess(&temp_path, DEFAULT_EXTRACT_TIMEOUT)
        .await
        .with_context(|| format!("Failed to extract parameters from: {}", temp_path.display()))?;
    println!(
        "  {} Loaded {} parameters via subprocess",
        style("→").dim(),
        params.len()
    );

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    Ok(params)
}

/// Create a temporary copy of the dylib with a unique name.
///
/// This ensures libloading loads a fresh dylib rather than returning
/// a cached handle from a previous load of the same path.
fn create_temp_dylib_copy(dylib_path: &Path) -> Result<PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let extension = dylib_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("dylib");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    let temp_name = format!("wavecraft_hotreload_{}.{}", timestamp, extension);
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(temp_name);

    std::fs::copy(dylib_path, &temp_path).with_context(|| {
        format!(
            "Failed to copy dylib to temp location: {}",
            temp_path.display()
        )
    })?;

    Ok(temp_path)
}
