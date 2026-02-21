//! Development session lifecycle management
//!
//! The DevSession struct owns all components of the development server
//! and ensures proper initialization and cleanup ordering.

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};

use crate::host::DevServerHost;
use crate::reload::guard::BuildGuard;
use crate::reload::rebuild::{RebuildCallbacks, RebuildPipeline};
use crate::reload::watcher::{FileWatcher, WatchEvent};
use crate::ws::WsServer;

#[cfg(feature = "audio")]
use crate::audio::server::AudioHandle;

fn log_rust_file_change(paths: &[PathBuf]) {
    let timestamp = chrono::Local::now().format("%H:%M:%S");
    if paths.len() == 1 {
        println!(
            "[{}] File changed: {}",
            timestamp,
            paths[0].file_name().unwrap_or_default().to_string_lossy()
        );
    } else {
        println!("[{}] {} files changed", timestamp, paths.len());
    }
}

fn report_rebuild_result(result: Result<Result<()>, tokio::task::JoinError>) {
    match result {
        Ok(Ok(())) => {
            // Success - continue watching
        }
        Ok(Err(e)) => {
            eprintln!("  {} Rebuild failed: {:#}", console::style("✗").red(), e);
        }
        Err(join_err) => {
            eprintln!(
                "  {} Hot-reload pipeline panicked: {}\n  {} Continuing to watch for changes...",
                console::style("✗").red(),
                join_err,
                console::style("→").cyan()
            );
        }
    }
}

async fn handle_rust_files_changed(paths: Vec<PathBuf>, pipeline: &Arc<RebuildPipeline>) {
    log_rust_file_change(&paths);

    // Trigger rebuild with panic recovery
    let pipeline = Arc::clone(pipeline);
    let result = tokio::spawn(async move { pipeline.handle_change().await }).await;
    report_rebuild_result(result);
}

/// Development session managing WebSocket server, file watcher, and rebuild pipeline.
///
/// Components are owned in the correct order for proper drop semantics:
/// 1. Watcher (drops first, stops sending events)
/// 2. Pipeline (processes pending events, then stops)
/// 3. Server (drops last, closes connections)
pub struct DevSession {
    /// File watcher for hot-reload triggers
    #[allow(dead_code)] // Kept alive for the lifetime of the session
    watcher: FileWatcher,
    /// Rebuild pipeline handle (for graceful shutdown)
    #[allow(dead_code)] // Kept alive for the lifetime of the session
    _pipeline_handle: tokio::task::JoinHandle<()>,
    /// WebSocket server (kept alive for the lifetime of the session)
    #[allow(dead_code)] // Kept alive for the lifetime of the session
    ws_server: Arc<WsServer<Arc<DevServerHost>>>,
    /// Shutdown signal receiver (kept alive for the lifetime of the session)
    #[allow(dead_code)]
    _shutdown_rx: watch::Receiver<bool>,
    /// Audio processing handle (if audio is enabled)
    #[cfg(feature = "audio")]
    #[allow(dead_code)] // Kept alive for the lifetime of the session
    _audio_handle: Option<AudioHandle>,
}

impl DevSession {
    /// Create a new development session.
    ///
    /// # Arguments
    ///
    /// * `engine_dir` - Path to the engine directory
    /// * `host` - Parameter host (shared with IPC handler and pipeline)
    /// * `ws_server` - WebSocket server (shared with pipeline)
    /// * `shutdown_rx` - Shutdown signal receiver
    /// * `callbacks` - CLI-specific callbacks for rebuild pipeline
    /// * `audio_handle` - Optional audio processing handle (audio feature only)
    ///
    /// # Returns
    ///
    /// A `DevSession` that manages the lifecycle of all components.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        engine_dir: PathBuf,
        host: Arc<DevServerHost>,
        ws_server: Arc<WsServer<Arc<DevServerHost>>>,
        shutdown_rx: watch::Receiver<bool>,
        callbacks: RebuildCallbacks,
        #[cfg(feature = "audio")] audio_handle: Option<AudioHandle>,
    ) -> Result<Self> {
        // Create channel for watch events
        let (watch_tx, mut watch_rx) = mpsc::unbounded_channel::<WatchEvent>();

        // Create file watcher
        let watcher = FileWatcher::new(&engine_dir, watch_tx, shutdown_rx.clone())?;

        // Create build guard and pipeline
        let guard = Arc::new(BuildGuard::new());
        let pipeline = Arc::new(RebuildPipeline::new(
            guard,
            engine_dir,
            Arc::clone(&host),
            Arc::clone(&ws_server),
            shutdown_rx.clone(),
            callbacks,
            #[cfg(feature = "audio")]
            None, // Audio reload will be handled separately if needed
        ));

        // Spawn rebuild pipeline task with panic recovery
        let shutdown_rx_for_task = shutdown_rx.clone();
        let pipeline_handle = tokio::spawn(async move {
            let mut shutdown_rx = shutdown_rx_for_task.clone();
            loop {
                if *shutdown_rx.borrow() {
                    break;
                }

                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                    maybe_event = watch_rx.recv() => {
                        let event = match maybe_event {
                            Some(event) => event,
                            None => break,
                        };

                        match event {
                            WatchEvent::RustFilesChanged(paths) => {
                                handle_rust_files_changed(paths, &pipeline).await;
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            watcher,
            _pipeline_handle: pipeline_handle,
            ws_server,
            _shutdown_rx: shutdown_rx,
            #[cfg(feature = "audio")]
            _audio_handle: audio_handle,
        })
    }
}
