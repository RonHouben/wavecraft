//! Shared utilities for the xtask build system.
//!
//! This module provides common functionality used across all xtask commands:
//! - Platform detection
//! - Path resolution for project directories and plugin install locations
//! - Colored terminal output helpers
//! - Command execution utilities

use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

/// Build mode for compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildMode {
    /// Debug build (faster compile, no optimizations)
    Debug,
    /// Release build (optimized)
    #[default]
    Release,
    /// Release build with debug symbols (for profiling)
    ReleaseDebug,
}

impl BuildMode {
    /// Returns the cargo profile flag for this build mode.
    pub fn cargo_flag(&self) -> Option<&'static str> {
        match self {
            BuildMode::Debug => None,
            BuildMode::Release => Some("--release"),
            BuildMode::ReleaseDebug => Some("--profile=release-debug"),
        }
    }

    /// Returns the target directory name for this build mode.
    pub fn target_dir(&self) -> &'static str {
        match self {
            BuildMode::Debug => "debug",
            BuildMode::Release => "release",
            BuildMode::ReleaseDebug => "release-debug",
        }
    }
}

/// Detected operating system platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
}

impl Platform {
    /// Detect the current platform.
    pub fn current() -> Self {
        if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else {
            Platform::Linux
        }
    }

    /// Returns true if running on macOS.
    pub fn is_macos(&self) -> bool {
        matches!(self, Platform::MacOS)
    }
}

/// Plugin name constants.
pub const PLUGIN_NAME: &str = "vstkit";
pub const PLUGIN_DISPLAY_NAME: &str = "VstKit";

/// Read version from workspace Cargo.toml.
///
/// Extracts the version string from the `[workspace.package]` section.
///
/// # Returns
///
/// The SemVer version string (e.g., "0.1.0")
///
/// # Errors
///
/// Returns an error if:
/// - The workspace Cargo.toml cannot be read
/// - The TOML is malformed
/// - The `workspace.package.version` key is missing
pub fn read_workspace_version() -> Result<String> {
    let workspace_toml = paths::engine_dir()?.join("Cargo.toml");
    let content =
        std::fs::read_to_string(&workspace_toml).context("Failed to read workspace Cargo.toml")?;

    let toml: toml::Value = content.parse().context("Failed to parse Cargo.toml")?;

    let version = toml
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Version not found in workspace Cargo.toml"))?;

    Ok(version.to_string())
}

/// Path utilities for the project.
pub mod paths {
    use super::*;
    use std::env;

    /// Returns the project root directory (workspace root).
    pub fn project_root() -> Result<PathBuf> {
        // xtask is at engine/xtask, so we go up two levels to reach project root
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .or_else(|_| env::current_dir())?;

        // Check if we're in the xtask directory or engine directory
        if manifest_dir.ends_with("xtask") {
            Ok(manifest_dir
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.to_path_buf())
                .context("Failed to find project root from xtask directory")?)
        } else if manifest_dir.ends_with("engine") {
            Ok(manifest_dir
                .parent()
                .map(|p| p.to_path_buf())
                .context("Failed to find project root from engine directory")?)
        } else {
            // Assume we're at project root or search upward for Cargo.toml
            let mut current = manifest_dir.clone();
            loop {
                if current.join("engine").join("Cargo.toml").exists() {
                    return Ok(current);
                }
                if !current.pop() {
                    // If we can't find it, assume manifest_dir is close enough
                    return Ok(manifest_dir);
                }
            }
        }
    }

    /// Returns the engine directory.
    pub fn engine_dir() -> Result<PathBuf> {
        Ok(project_root()?.join("engine"))
    }

    /// Returns the UI directory.
    pub fn ui_dir() -> Result<PathBuf> {
        Ok(project_root()?.join("ui"))
    }

    /// Returns the target/bundled directory where built plugins are placed.
    pub fn bundled_dir() -> Result<PathBuf> {
        Ok(engine_dir()?.join("target").join("bundled"))
    }

    /// Returns the AU wrapper source directory.
    pub fn au_wrapper_dir() -> Result<PathBuf> {
        Ok(project_root()?
            .join("packaging")
            .join("macos")
            .join("au-wrapper"))
    }

    /// Returns the platform-specific VST3 installation directory.
    pub fn vst3_install_dir() -> Result<PathBuf> {
        match Platform::current() {
            Platform::MacOS => {
                let home = dirs::home_dir().context("Could not determine home directory")?;
                Ok(home.join("Library/Audio/Plug-Ins/VST3"))
            }
            Platform::Windows => Ok(PathBuf::from(r"C:\Program Files\Common Files\VST3")),
            Platform::Linux => {
                let home = dirs::home_dir().context("Could not determine home directory")?;
                Ok(home.join(".vst3"))
            }
        }
    }

    /// Returns the platform-specific CLAP installation directory.
    pub fn clap_install_dir() -> Result<PathBuf> {
        match Platform::current() {
            Platform::MacOS => {
                let home = dirs::home_dir().context("Could not determine home directory")?;
                Ok(home.join("Library/Audio/Plug-Ins/CLAP"))
            }
            Platform::Windows => Ok(PathBuf::from(r"C:\Program Files\Common Files\CLAP")),
            Platform::Linux => {
                let home = dirs::home_dir().context("Could not determine home directory")?;
                Ok(home.join(".clap"))
            }
        }
    }

    /// Returns the AU installation directory (macOS only).
    pub fn au_install_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join("Library/Audio/Plug-Ins/Components"))
    }
}

/// Colored output helpers for consistent terminal UX.
pub mod output {
    use super::*;

    /// Print a section header.
    pub fn print_header(text: &str) {
        println!("{}", "========================================".blue());
        println!("  {}", text.blue());
        println!("{}", "========================================".blue());
        println!();
    }

    /// Print a status message (action being taken).
    pub fn print_status(text: &str) {
        println!("{}", text.yellow());
    }

    /// Print a success message.
    pub fn print_success(text: &str) {
        println!("{}", text.green());
    }

    /// Print a success item with checkmark.
    pub fn print_success_item(text: &str) {
        println!("  {} {}", "✓".green(), text);
    }

    /// Print an error message.
    pub fn print_error(text: &str) {
        eprintln!("{}", text.red());
    }

    /// Print a warning message.
    pub fn print_warning(text: &str) {
        println!("{}", text.yellow());
    }

    /// Print a skip message (for optional steps).
    pub fn print_skip(text: &str) {
        println!("{}", text.yellow());
    }

    /// Print an informational message.
    pub fn print_info(text: &str) {
        println!("{}", text.cyan());
    }
}

/// Run a command and return its exit status.
///
/// This streams output to stdout/stderr in real-time.
pub fn run_command(cmd: &mut Command) -> Result<ExitStatus> {
    let program = cmd.get_program().to_string_lossy().to_string();

    let status = cmd
        .stdin(Stdio::null())
        .status()
        .with_context(|| format!("Failed to execute command: {}", program))?;

    Ok(status)
}

/// Run a command and check for success.
///
/// Returns an error if the command fails.
pub fn run_command_checked(cmd: &mut Command) -> Result<()> {
    let program = cmd.get_program().to_string_lossy().to_string();
    let status = run_command(cmd)?;

    if !status.success() {
        anyhow::bail!(
            "Command '{}' failed with exit code: {:?}",
            program,
            status.code()
        );
    }

    Ok(())
}

/// Check if a command/executable exists on the system.
pub fn command_exists(name: &str) -> bool {
    which::which(name).is_ok()
}

#[cfg(test)]
mod tests;
