//! Hot-reload rebuild pipeline
//!
//! This module provides automatic rebuilding when Rust source files change.
//! The pipeline watches for file changes, triggers a Cargo build, and updates
//! the development server's parameter state without dropping WebSocket connections.
//!
//! CLI-specific functions (dylib discovery, subprocess param extraction, sidecar
//! caching) are injected via [`RebuildCallbacks`] to keep this crate CLI-agnostic.

use anyhow::{Context, Result};
use console::style;
use std::any::Any;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio::sync::watch;
use wavecraft_bridge::ParameterHost;
use wavecraft_protocol::ParameterInfo;

use crate::host::DevServerHost;
use crate::reload::guard::BuildGuard;
use crate::ws::WsServer;

/// Callback type for loading parameters from a rebuilt dylib.
///
/// The closure receives the engine directory and must return the loaded
/// parameters. This is injected by the CLI to handle dylib discovery,
/// temp copying, and subprocess extraction.
pub type ParamLoaderFn = Arc<
    dyn Fn(PathBuf) -> Pin<Box<dyn Future<Output = Result<Vec<ParameterInfo>>> + Send>>
        + Send
        + Sync,
>;

/// Callback type for writing parameter cache to a sidecar file.
pub type SidecarWriterFn = Arc<dyn Fn(&Path, &[ParameterInfo]) -> Result<()> + Send + Sync>;

/// Callback type for writing generated TypeScript parameter ID types.
pub type TsTypesWriterFn = Arc<dyn Fn(&[ParameterInfo]) -> Result<()> + Send + Sync>;

/// Callbacks for CLI-specific operations.
///
/// The rebuild pipeline needs to perform operations that depend on CLI
/// infrastructure (dylib discovery, subprocess parameter extraction,
/// sidecar caching). These are injected as callbacks to keep the
/// dev-server crate independent of CLI internals.
pub struct RebuildCallbacks {
    /// Engine package name for `cargo build --package` flag.
    /// `None` means no `--package` flag (single-crate project).
    pub package_name: Option<String>,
    /// Optional sidecar cache writer (writes params to JSON file).
    pub write_sidecar: Option<SidecarWriterFn>,
    /// Optional TypeScript types writer (writes generated parameter ID typings).
    pub write_ts_types: Option<TsTypesWriterFn>,
    /// Loads parameters from the rebuilt dylib (async).
    /// Receives the engine directory and returns parsed parameters.
    pub param_loader: ParamLoaderFn,
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
    callbacks: RebuildCallbacks,
    #[cfg(feature = "audio")]
    audio_reload_tx: Option<tokio::sync::mpsc::UnboundedSender<Vec<ParameterInfo>>>,
    /// Channel for canceling the current parameter extraction when superseded.
    cancel_param_load_tx: watch::Sender<bool>,
    cancel_param_load_rx: watch::Receiver<bool>,
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
        callbacks: RebuildCallbacks,
        #[cfg(feature = "audio")] audio_reload_tx: Option<
            tokio::sync::mpsc::UnboundedSender<Vec<ParameterInfo>>,
        >,
    ) -> Self {
        let (cancel_param_load_tx, cancel_param_load_rx) = watch::channel(false);
        Self {
            guard,
            engine_dir,
            host,
            ws_server,
            shutdown_rx,
            callbacks,
            #[cfg(feature = "audio")]
            audio_reload_tx,
            cancel_param_load_tx,
            cancel_param_load_rx,
        }
    }

    /// Handle a file change event. Triggers rebuild if not already running.
    pub async fn handle_change(&self) -> Result<()> {
        if !self.guard.try_start() {
            self.guard.mark_pending();
            // Cancel any ongoing parameter extraction - it will be superseded
            let _ = self.cancel_param_load_tx.send(true);
            println!(
                "  {} Build already in progress, queuing rebuild...",
                style("→").dim()
            );
            return Ok(());
        }

        // Reset cancellation flag at the start of a new build cycle
        let _ = self.cancel_param_load_tx.send(false);

        loop {
            let result = self.do_build().await;

            match result {
                Ok((params, param_count_change)) => {
                    let mut reload_ok = true;
                    let mut ts_types_ok = true;

                    if let Some(ref writer) = self.callbacks.write_sidecar
                        && let Err(e) = writer(&self.engine_dir, &params)
                    {
                        println!("  Warning: failed to update param cache: {}", e);
                    }

                    if let Some(ref writer) = self.callbacks.write_ts_types
                        && let Err(e) = writer(&params)
                    {
                        ts_types_ok = false;
                        println!(
                            "  {} Failed to regenerate TypeScript parameter types: {}",
                            style("⚠").yellow(),
                            e
                        );
                        println!(
                            "  {} Save a Rust source file to retry, or restart the dev server",
                            style("  ↳").dim(),
                        );
                    }

                    println!("  {} Updating parameter host...", style("→").dim());
                    let replace_result =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
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
                            "  {} Hot-reload complete — {} parameters{}{}",
                            style("✓").green(),
                            params.len(),
                            change_info,
                            if ts_types_ok {
                                ""
                            } else {
                                " (⚠ TypeScript types stale)"
                            }
                        );

                        // Trigger audio reload if audio is enabled
                        #[cfg(feature = "audio")]
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

        if let Some(ref package_name) = self.callbacks.package_name {
            cmd.args(["--package", package_name]);
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
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(line)
                    && json["reason"] == "compiler-message"
                    && let Some(message) = json["message"]["rendered"].as_str()
                {
                    error_lines.push(message.to_string());
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

        // Load parameters via the injected callback, but race against cancellation
        let loader = Arc::clone(&self.callbacks.param_loader);
        let engine_dir = self.engine_dir.clone();
        let mut cancel_rx = self.cancel_param_load_rx.clone();

        let params = tokio::select! {
            result = loader(engine_dir) => {
                result.context("Failed to load parameters from rebuilt dylib")?
            }
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow_and_update() {
                    println!(
                        "  {} Parameter extraction cancelled — superseded by newer change",
                        style("⚠").yellow()
                    );
                    anyhow::bail!("Parameter extraction cancelled due to newer file change");
                }
                // If cancellation was reset to false, continue waiting
                loader(self.engine_dir.clone())
                    .await
                    .context("Failed to load parameters from rebuilt dylib")?
            }
        };

        let param_count_change = params.len() as i32 - old_count;

        Ok((params, param_count_change))
    }

    async fn kill_build_process(&self, child: &mut tokio::process::Child) -> Result<()> {
        #[cfg(unix)]
        {
            use nix::sys::signal::{Signal, kill};
            use nix::unistd::Pid;

            if let Some(pid) = child.id() {
                let _ = kill(Pid::from_raw(-(pid as i32)), Signal::SIGTERM);
            }
        }

        let _ = child.kill().await;
        Ok(())
    }
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
