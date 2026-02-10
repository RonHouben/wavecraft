//! File watching for Rust hot-reload
//!
//! Watches engine source files and triggers rebuild events on changes.
//! Uses debouncing to handle rapid file saves and editor temp files.

use anyhow::Result;
use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, watch};

/// Events emitted by the file watcher
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// Rust source files changed (list of changed paths)
    RustFilesChanged(Vec<PathBuf>),
}

/// File watcher with debouncing for Rust source files
pub struct FileWatcher {
    #[allow(dead_code)] // Kept alive for the lifetime of the watcher
    debouncer: Debouncer<notify::RecommendedWatcher, FileIdMap>,
    #[allow(dead_code)] // Kept alive for the lifetime of the watcher
    _shutdown_rx: watch::Receiver<bool>,
}

impl FileWatcher {
    /// Create a new file watcher.
    ///
    /// Watches `engine/src/**/*.rs` and `engine/Cargo.toml` for changes.
    /// Events are debounced with a 500ms timeout to handle rapid multi-file saves.
    ///
    /// # Arguments
    ///
    /// * `engine_dir` - Path to the engine directory (containing src/ and Cargo.toml)
    /// * `tx` - Channel to send watch events to
    pub fn new(
        engine_dir: &Path,
        tx: mpsc::UnboundedSender<WatchEvent>,
        shutdown_rx: watch::Receiver<bool>,
    ) -> Result<Self> {
        let engine_dir_clone = engine_dir.to_path_buf();
        let engine_dir = Arc::new(engine_dir_clone.clone());
        let shutdown_rx_for_events = shutdown_rx.clone();

        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None, // no tick rate override
            move |result: DebounceEventResult| {
                Self::handle_events(result, Arc::clone(&engine_dir), &tx, &shutdown_rx_for_events);
            },
        )?;

        // Watch engine/src recursively
        let src_path = engine_dir_clone.join("src");
        debouncer.watch(&src_path, RecursiveMode::Recursive)?;

        // Watch engine/Cargo.toml non-recursively
        let cargo_toml = engine_dir_clone.join("Cargo.toml");
        if cargo_toml.exists() {
            debouncer.watch(&cargo_toml, RecursiveMode::NonRecursive)?;
        }

        Ok(Self {
            debouncer,
            _shutdown_rx: shutdown_rx,
        })
    }

    /// Handle debounced file events
    fn handle_events(
        result: DebounceEventResult,
        engine_dir: Arc<PathBuf>,
        tx: &mpsc::UnboundedSender<WatchEvent>,
        shutdown_rx: &watch::Receiver<bool>,
    ) {
        if *shutdown_rx.borrow() {
            return;
        }

        let events = match result {
            Ok(events) => events,
            Err(errors) => {
                for error in errors {
                    eprintln!("File watcher error: {:?}", error);
                }
                return;
            }
        };

        // Filter to relevant files
        let mut changed_paths = Vec::new();
        for event in events {
            for path in &event.paths {
                if Self::is_relevant_file(path, &engine_dir) {
                    changed_paths.push(path.clone());
                }
            }
        }

        if !changed_paths.is_empty() {
            // Deduplicate paths
            changed_paths.sort();
            changed_paths.dedup();

            if let Err(e) = tx.send(WatchEvent::RustFilesChanged(changed_paths)) {
                eprintln!("Warning: File watcher failed to send event (channel closed): {:?}", e);
            }
        }
    }

    /// Check if a file path is relevant for hot-reload
    fn is_relevant_file(path: &Path, engine_dir: &Path) -> bool {
        // Ignore target/ directory
        if path.starts_with(engine_dir.join("target")) {
            return false;
        }

        // Ignore hidden files and directories
        if path
            .components()
            .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
        {
            return false;
        }

        let file_name = match path.file_name() {
            Some(name) => name.to_string_lossy(),
            None => return false,
        };

        // Ignore editor temp files
        if file_name.ends_with(".swp")
            || file_name.ends_with(".swo")
            || file_name.ends_with('~')
            || file_name.starts_with(".#")
        {
            return false;
        }

        // Accept .rs files and Cargo.toml
        file_name.ends_with(".rs") || file_name == "Cargo.toml"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_relevant_file() {
        let engine_dir = PathBuf::from("/project/engine");

        // Should accept .rs files
        assert!(FileWatcher::is_relevant_file(
            &engine_dir.join("src/lib.rs"),
            &engine_dir
        ));
        assert!(FileWatcher::is_relevant_file(
            &engine_dir.join("src/dsp/oscillator.rs"),
            &engine_dir
        ));

        // Should accept Cargo.toml
        assert!(FileWatcher::is_relevant_file(
            &engine_dir.join("Cargo.toml"),
            &engine_dir
        ));

        // Should reject target/ files
        assert!(!FileWatcher::is_relevant_file(
            &engine_dir.join("target/debug/libfoo.dylib"),
            &engine_dir
        ));

        // Should reject hidden files
        assert!(!FileWatcher::is_relevant_file(
            &engine_dir.join("src/.hidden.rs"),
            &engine_dir
        ));

        // Should reject editor temp files
        assert!(!FileWatcher::is_relevant_file(
            &engine_dir.join("src/lib.rs.swp"),
            &engine_dir
        ));
        assert!(!FileWatcher::is_relevant_file(
            &engine_dir.join("src/lib.rs~"),
            &engine_dir
        ));
        assert!(!FileWatcher::is_relevant_file(
            &engine_dir.join("src/.#lib.rs"),
            &engine_dir
        ));

        // Should reject non-Rust files
        assert!(!FileWatcher::is_relevant_file(
            &engine_dir.join("src/data.json"),
            &engine_dir
        ));
    }
}
