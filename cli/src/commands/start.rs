//! Development server command - starts WebSocket + UI dev servers.
//!
//! This command provides the development experience for wavecraft plugins:
//! 1. Builds the plugin in debug mode
//! 2. Loads parameter metadata via FFI from the compiled dylib
//! 3. Loads and validates the audio processor vtable, then runs audio in-process
//! 4. Starts an embedded WebSocket server for browser UI communication
//! 5. Starts the Vite dev server for UI hot-reloading

use anyhow::{Context, Result};

use crate::project::ProjectMarkers;

mod audio_runtime;
mod metadata_cache;
mod preflight;
mod reload_extractors;
mod shutdown;
mod startup_pipeline;
mod tsconfig_paths;

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
}

pub(super) const ALLOW_NO_AUDIO_ENV: &str = "WAVECRAFT_ALLOW_NO_AUDIO";

impl StartCommand {
    pub fn execute(&self) -> Result<()> {
        // 1. Detect project
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let project = ProjectMarkers::detect(&cwd)?;

        // 2. Check dependencies
        preflight::ensure_dependencies(&project, self.install, self.no_install)?;

        // 3. Start servers
        tsconfig_paths::ensure_sdk_ui_paths_for_typescript(&project)?;
        startup_pipeline::run_dev_servers(&project, self.port, self.ui_port)
    }
}
