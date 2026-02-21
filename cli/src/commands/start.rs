//! Development server command - starts WebSocket + UI dev servers.
//!
//! This command provides the development experience for wavecraft plugins:
//! 1. Builds the plugin in debug mode
//! 2. Loads parameter metadata via FFI from the compiled dylib
//! 3. Loads and validates the audio processor vtable, then runs audio in-process
//! 4. Starts an embedded WebSocket server for browser UI communication
//! 5. Starts the Vite dev server for UI hot-reloading

use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};

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
pub(super) const START_XTASK_DELEGATION_GUARD_ENV: &str = "WAVECRAFT_START_FROM_XTASK";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartExecutionMode {
    Direct,
    DelegateToSdkXtask,
}

impl StartCommand {
    pub fn execute(&self) -> Result<()> {
        // 1. Detect project
        let cwd = std::env::current_dir().context("Failed to get current directory")?;
        let project = ProjectMarkers::detect(&cwd)?;

        if resolve_execution_mode(&project, self) == StartExecutionMode::DelegateToSdkXtask {
            return run_sdk_xtask_dev(&cwd, self.port);
        }

        // 2. Check dependencies
        preflight::ensure_dependencies(&project, self.install, self.no_install)?;

        // 3. Start servers
        tsconfig_paths::ensure_sdk_ui_paths_for_typescript(&project)?;
        startup_pipeline::run_dev_servers(&project, self.port, self.ui_port)
    }
}

fn resolve_execution_mode(project: &ProjectMarkers, command: &StartCommand) -> StartExecutionMode {
    resolve_execution_mode_with_guard(project, command, xtask_delegation_guard_enabled())
}

fn resolve_execution_mode_with_guard(
    project: &ProjectMarkers,
    command: &StartCommand,
    delegation_guard_enabled: bool,
) -> StartExecutionMode {
    if !project.sdk_mode {
        return StartExecutionMode::Direct;
    }

    if delegation_guard_enabled {
        return StartExecutionMode::Direct;
    }

    if !is_xtask_compatible_start(command) {
        return StartExecutionMode::Direct;
    }

    StartExecutionMode::DelegateToSdkXtask
}

fn is_xtask_compatible_start(command: &StartCommand) -> bool {
    command.ui_port == 5173 && !command.install && !command.no_install
}

fn xtask_delegation_guard_enabled() -> bool {
    std::env::var(START_XTASK_DELEGATION_GUARD_ENV)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn run_sdk_xtask_dev(sdk_root: &Path, port: u16) -> Result<()> {
    let port_str = port.to_string();
    let status = Command::new("cargo")
        .args(["xtask", "dev", "--port", &port_str])
        .current_dir(sdk_root)
        .env(START_XTASK_DELEGATION_GUARD_ENV, "1")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run `cargo xtask dev` from SDK repository root")?;

    if !status.success() {
        anyhow::bail!("`cargo xtask dev` exited with status {}", status);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn default_command() -> StartCommand {
        StartCommand {
            port: 9000,
            ui_port: 5173,
            install: false,
            no_install: false,
        }
    }

    fn create_plugin_project(root: &Path) {
        fs::create_dir_all(root.join("ui")).expect("ui dir");
        fs::create_dir_all(root.join("engine/src")).expect("engine dir");
        fs::write(root.join("ui/package.json"), "{}\n").expect("ui package");
        fs::write(
            root.join("engine/Cargo.toml"),
            "[package]\nname = \"test-plugin\"\nversion = \"0.1.0\"\n",
        )
        .expect("engine cargo");
    }

    fn create_sdk_repo_root(root: &Path) {
        fs::create_dir_all(root.join("ui")).expect("ui dir");
        fs::create_dir_all(root.join("engine/crates/wavecraft-core")).expect("core crate dir");
        fs::create_dir_all(root.join("sdk-template/ui")).expect("template ui dir");
        fs::create_dir_all(root.join("sdk-template/engine/src")).expect("template engine dir");
        fs::create_dir_all(root.join("cli")).expect("cli dir");

        fs::write(root.join("ui/package.json"), "{}\n").expect("root ui package");
        fs::write(
            root.join("engine/Cargo.toml"),
            "[workspace]\nmembers = [\"crates/*\"]\n",
        )
        .expect("workspace cargo");
        fs::write(
            root.join("engine/crates/wavecraft-core/Cargo.toml"),
            "[package]\nname = \"wavecraft-core\"\nversion = \"0.1.0\"\n",
        )
        .expect("core cargo");
        fs::write(
            root.join("cli/Cargo.toml"),
            "[package]\nname = \"wavecraft\"\nversion = \"0.1.0\"\n",
        )
        .expect("cli cargo");
        fs::write(root.join("sdk-template/ui/package.json"), "{}\n").expect("template ui package");
        fs::write(
            root.join("sdk-template/engine/Cargo.toml"),
            "[package]\nname = \"wavecraft-dev-template\"\nversion = \"0.1.0\"\n",
        )
        .expect("template engine cargo");
    }

    #[test]
    fn generated_plugin_project_stays_on_direct_start_flow() {
        let temp = TempDir::new().expect("temp dir");
        create_plugin_project(temp.path());

        let project = ProjectMarkers::detect(temp.path()).expect("plugin markers");
        assert!(!project.sdk_mode);

        let mode = resolve_execution_mode_with_guard(&project, &default_command(), false);
        assert_eq!(mode, StartExecutionMode::Direct);
    }

    #[test]
    fn sdk_root_uses_xtask_dev_mode_for_default_start() {
        let temp = TempDir::new().expect("temp dir");
        create_sdk_repo_root(temp.path());

        let project = ProjectMarkers::detect(temp.path()).expect("sdk markers");
        assert!(project.sdk_mode);

        let mode = resolve_execution_mode_with_guard(&project, &default_command(), false);
        assert_eq!(mode, StartExecutionMode::DelegateToSdkXtask);
    }

    #[test]
    fn unrelated_directory_still_fails_project_detection() {
        let temp = TempDir::new().expect("temp dir");

        let error = ProjectMarkers::detect(temp.path()).expect_err("should fail detection");
        let message = error.to_string();

        assert!(message.contains("Not a Wavecraft project"));
        assert!(message.contains("wavecraft create"));
    }
}
