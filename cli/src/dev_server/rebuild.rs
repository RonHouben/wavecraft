//! Hot-reload rebuild pipeline for `wavecraft start`
//!
//! This module provides automatic rebuilding when Rust source files change.
//! The pipeline watches for file changes, triggers a Cargo build, and updates
//! the development server's parameter state without dropping WebSocket connections.

use anyhow::{Context, Result};
use console::style;
use std::any::Any;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::watch;
use wavecraft_bridge::ParameterHost;
use wavecraft_dev_server::ws_server::WsServer;
use wavecraft_protocol::ParameterInfo;

use super::host::DevServerHost;
use crate::dev_server::PluginLoader;
use crate::commands::start::write_sidecar_cache;

/// Concurrency control for rebuild operations.
///
/// Ensures only one build runs at a time, with at most one pending.
/// Uses atomics for lock-free coordination between watcher and builder.
pub struct BuildGuard {
    building: AtomicBool,
    pending: AtomicBool,
}

impl BuildGuard {
    pub fn new() -> Self {
        Self {
            building: AtomicBool::new(false),
            pending: AtomicBool::new(false),
        }
    }

    /// Try to start a build. Returns true if acquired.
    pub fn try_start(&self) -> bool {
        self.building
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// Mark a pending rebuild request (received during active build).
    pub fn mark_pending(&self) {
        self.pending.store(true, Ordering::SeqCst);
    }

    /// Complete current build. Returns true if a pending build should start.
    pub fn complete(&self) -> bool {
        self.building.store(false, Ordering::SeqCst);
        self.pending.swap(false, Ordering::SeqCst)
    }
}

impl Default for BuildGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// Rebuild pipeline for hot-reload.
///
/// Coordinates Cargo builds, parameter reloading, and WebSocket notifications.
pub struct RebuildPipeline {
    guard: Arc<BuildGuard>,
    engine_dir: PathBuf,
    host: Arc<DevServerHost>,
    ws_server: Arc<WsServer<Arc<DevServerHost>>>,
    shutdown_rx: watch::Receiver<bool>,
    #[cfg(feature = "audio-dev")]
    audio_reload_tx: Option<tokio::sync::mpsc::UnboundedSender<Vec<ParameterInfo>>>,
}

impl RebuildPipeline {
    /// Create a new rebuild pipeline.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        guard: Arc<BuildGuard>,
        engine_dir: PathBuf,
        host: Arc<DevServerHost>,
        ws_server: Arc<WsServer<Arc<DevServerHost>>>,
        shutdown_rx: watch::Receiver<bool>,
        #[cfg(feature = "audio-dev")] audio_reload_tx: Option<
            tokio::sync::mpsc::UnboundedSender<Vec<ParameterInfo>>,
        >,
    ) -> Self {
        Self {
            guard,
            engine_dir,
            host,
            ws_server,
            shutdown_rx,
            #[cfg(feature = "audio-dev")]
            audio_reload_tx,
        }
    }

    /// Handle a file change event. Triggers rebuild if not already running.
    pub async fn handle_change(&self) -> Result<()> {
        if !self.guard.try_start() {
            self.guard.mark_pending();
            println!(
                "  {} Build already in progress, queuing rebuild...",
                style("→").dim()
            );
            return Ok(());
        }

        loop {
            let result = self.do_build().await;

            match result {
                Ok((params, param_count_change)) => {
                    let mut reload_ok = true;

                    if let Err(e) = write_sidecar_cache(&self.engine_dir, &params) {
                        println!("  Warning: failed to update param cache: {}", e);
                    }

                    println!("  {} Updating parameter host...", style("→").dim());
                    let replace_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        self.host.replace_parameters(params.clone())
                    }));

                    match replace_result {
                        Ok(Ok(())) => {
                            println!("  {} Updated {} parameters", style("→").dim(), params.len());
                        }
                        Ok(Err(e)) => {
                            reload_ok = false;
                            println!(
                                "  {} Failed to replace parameters: {:#}",
                                style("✗").red(),
                                e
                            );
                        }
                        Err(panic_payload) => {
                            reload_ok = false;
                            println!(
                                "  {} Panic while replacing parameters: {}",
                                style("✗").red(),
                                panic_message(panic_payload)
                            );
                        }
                    }

                    if reload_ok {
                        println!("  {} Notifying UI clients...", style("→").dim());
                        let broadcast_result = tokio::spawn({
                            let ws_server = Arc::clone(&self.ws_server);
                            async move { ws_server.broadcast_parameters_changed().await }
                        })
                        .await;

                        match broadcast_result {
                            Ok(Ok(())) => {
                                println!("  {} UI notified", style("→").dim());
                            }
                            Ok(Err(e)) => {
                                reload_ok = false;
                                println!(
                                    "  {} Failed to notify UI clients: {:#}",
                                    style("✗").red(),
                                    e
                                );
                            }
                            Err(join_err) => {
                                reload_ok = false;
                                println!(
                                    "  {} Panic while notifying UI clients: {}",
                                    style("✗").red(),
                                    join_err
                                );
                            }
                        }
                    }

                    if reload_ok {
                        let change_info = if param_count_change > 0 {
                            format!(" (+{} new)", param_count_change)
                        } else if param_count_change < 0 {
                            format!(" ({} removed)", -param_count_change)
                        } else {
                            String::new()
                        };

                        println!(
                            "  {} Hot-reload complete — {} parameters{}",
                            style("✓").green(),
                            params.len(),
                            change_info
                        );

                        // Trigger audio reload if audio-dev is enabled
                        #[cfg(feature = "audio-dev")]
                        if let Some(ref tx) = self.audio_reload_tx {
                            let _ = tx.send(params);
                        }
                    } else {
                        println!(
                            "  {} Hot-reload aborted — parameters not fully applied",
                            style("✗").red()
                        );
                    }
                }
                Err(e) => {
                    println!("  {} Build failed:\n{}", style("✗").red(), e);
                    // Preserve old state, don't update parameters
                }
            }

            if !self.guard.complete() {
                break; // No pending rebuild
            }
            println!(
                "  {} Pending changes detected, rebuilding...",
                style("→").cyan()
            );
        }

        Ok(())
    }

    /// Execute a Cargo build and load new parameters on success.
    async fn do_build(&self) -> Result<(Vec<ParameterInfo>, i32)> {
        use crate::project::read_engine_package_name;

        if *self.shutdown_rx.borrow() {
            anyhow::bail!("Build cancelled due to shutdown");
        }

        println!("  {} Rebuilding plugin...", style("🔄").cyan());
        let start = std::time::Instant::now();

        // Get old parameter count for change reporting
        let old_count = self.host.get_all_parameters().len() as i32;

        // Build command with optional --package flag
        let mut cmd = Command::new("cargo");
        cmd.args([
            "build",
            "--lib",
            "--features",
            "_param-discovery",
            "--message-format=json",
        ]);

        if let Some(package_name) = read_engine_package_name(&self.engine_dir) {
            cmd.args(["--package", &package_name]);
        }

        let mut child = cmd
            .current_dir(&self.engine_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn cargo build")?;

        let stdout = child
            .stdout
            .take()
            .context("Failed to capture cargo stdout")?;
        let stderr = child
            .stderr
            .take()
            .context("Failed to capture cargo stderr")?;

        let stdout_handle = tokio::spawn(read_to_end(stdout));
        let stderr_handle = tokio::spawn(read_to_end(stderr));

        let mut shutdown_rx = self.shutdown_rx.clone();
        let status = tokio::select! {
            status = child.wait() => status.context("Failed to wait for cargo build")?,
            _ = shutdown_rx.changed() => {
                self.kill_build_process(&mut child).await?;
                let _ = stdout_handle.await;
                let _ = stderr_handle.await;
                anyhow::bail!("Build cancelled due to shutdown");
            }
        };

        let stdout = stdout_handle
            .await
            .context("Failed to join cargo stdout task")??;
        let stderr = stderr_handle
            .await
            .context("Failed to join cargo stderr task")??;

        let elapsed = start.elapsed();

        if !status.success() {
            // Parse JSON output for errors
            let stderr = String::from_utf8_lossy(&stderr);
            let stdout = String::from_utf8_lossy(&stdout);

            // Try to extract compiler errors from JSON
            let mut error_lines = Vec::new();
            for line in stdout.lines().chain(stderr.lines()) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                    if json["reason"] == "compiler-message" {
                        if let Some(message) = json["message"]["rendered"].as_str() {
                            error_lines.push(message.to_string());
                        }
                    }
                }
            }

            if error_lines.is_empty() {
                error_lines.push(stderr.to_string());
            }

            anyhow::bail!("{}", error_lines.join("\n"));
        }

        println!(
            "  {} Build succeeded in {:.1}s",
            style("✓").green(),
            elapsed.as_secs_f64()
        );

        let engine_dir = self.engine_dir.clone();
        let load_result = tokio::time::timeout(
            Duration::from_secs(30),
            tokio::task::spawn_blocking(move || {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    load_parameters_from_dylib(engine_dir)
                }))
            }),
        )
        .await;

        let params = match load_result {
            Err(_) => {
                anyhow::bail!(
                    "Timeout loading parameters from dylib (30s). This may indicate a macOS library loading issue. Try restarting `wavecraft start`."
                );
            }
            Ok(join_result) => match join_result {
                Err(join_err) => {
                    anyhow::bail!("Failed to join parameter load task: {}", join_err);
                }
                Ok(Ok(Ok(params))) => params,
                Ok(Ok(Err(e))) => {
                    return Err(e).context("Failed to load parameters from dylib");
                }
                Ok(Err(panic_payload)) => {
                    anyhow::bail!(
                        "Panic while loading parameters from dylib: {}",
                        panic_message(panic_payload)
                    );
                }
            },
        };
        let param_count_change = params.len() as i32 - old_count;

        Ok((params, param_count_change))
    }

    async fn kill_build_process(&self, child: &mut tokio::process::Child) -> Result<()> {
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            if let Some(pid) = child.id() {
                let _ = kill(Pid::from_raw(-(pid as i32)), Signal::SIGTERM);
            }
        }

        let _ = child.kill().await;
        Ok(())
    }
}

/// Load parameters from the rebuilt dylib using FFI.
///
/// To avoid dylib caching issues on macOS, we copy the dylib to a unique
/// temporary location before loading. This ensures libloading reads the
/// fresh file from disk rather than returning a cached library handle.
fn load_parameters_from_dylib(engine_dir: PathBuf) -> Result<Vec<ParameterInfo>> {
    use crate::project::find_plugin_dylib;

    println!("  {} Finding plugin dylib...", style("→").dim());
    let lib_path =
        find_plugin_dylib(&engine_dir).context("Failed to find plugin dylib after rebuild")?;
    println!("  {} Found: {}", style("→").dim(), lib_path.display());

    println!("  {} Copying to temp location...", style("→").dim());
    let temp_path = create_temp_dylib_copy(&lib_path)?;
    println!("  {} Temp: {}", style("→").dim(), temp_path.display());

    println!("  {} Loading dylib via FFI...", style("→").dim());
    let params = PluginLoader::load_params_only(&temp_path)
        .with_context(|| format!("Failed to load dylib: {}", temp_path.display()))?;
    println!(
        "  {} Loaded {} parameters via FFI",
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
fn create_temp_dylib_copy(dylib_path: &std::path::Path) -> Result<PathBuf> {
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

async fn read_to_end(mut reader: impl tokio::io::AsyncRead + Unpin) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    reader
        .read_to_end(&mut buffer)
        .await
        .context("Failed to read cargo output")?;
    Ok(buffer)
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(msg) = payload.downcast_ref::<String>() {
        msg.clone()
    } else if let Some(msg) = payload.downcast_ref::<&str>() {
        msg.to_string()
    } else {
        "Unknown panic".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_guard_single_build() {
        let guard = BuildGuard::new();

        // First try_start should succeed
        assert!(guard.try_start());

        // Second try_start should fail (build in progress)
        assert!(!guard.try_start());

        // Complete without pending should return false
        assert!(!guard.complete());

        // After complete, try_start should succeed again
        assert!(guard.try_start());
        guard.complete();
    }

    #[test]
    fn test_build_guard_pending() {
        let guard = BuildGuard::new();

        // Start a build
        assert!(guard.try_start());

        // Try to start another (should fail)
        assert!(!guard.try_start());

        // Mark as pending
        guard.mark_pending();

        // Complete should return true (pending build)
        assert!(guard.complete());

        // Now try_start should succeed (for the pending build)
        assert!(guard.try_start());
        assert!(!guard.complete());
    }

    #[test]
    fn test_build_guard_multiple_pending() {
        let guard = BuildGuard::new();

        // Start a build
        assert!(guard.try_start());

        // Mark pending multiple times (only one pending should be stored)
        guard.mark_pending();
        guard.mark_pending();
        guard.mark_pending();

        // Complete should return true once
        assert!(guard.complete());

        // Second complete should return false (no more pending)
        assert!(!guard.complete());
    }
}
