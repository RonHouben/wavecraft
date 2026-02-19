//! Hidden subcommand for processor metadata extraction via subprocess isolation.
//!
//! This command is invoked by `wavecraft start` and hot-reload to extract
//! processor metadata from a plugin dylib in a separate process.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use wavecraft_bridge::PluginParamLoader;

/// Execute the extract-processors subcommand.
///
/// Loads the plugin dylib via FFI, extracts processor metadata, and prints JSON
/// to stdout. Any errors are written to stderr with appropriate exit codes.
pub fn execute(dylib_path: PathBuf) -> Result<()> {
    validate_dylib_path(&dylib_path)?;

    let processors = PluginParamLoader::load_processors_only(&dylib_path)
        .with_context(|| format!("Failed to load processors from {}", dylib_path.display()))?;

    let json =
        serde_json::to_string(&processors).context("Failed to serialize processors to JSON")?;
    print_compact_json(&json);

    Ok(())
}

fn validate_dylib_path(dylib_path: &Path) -> Result<()> {
    if !dylib_path.exists() {
        anyhow::bail!(
            "Plugin dylib not found: {}\nEnsure the plugin was built successfully.",
            dylib_path.display()
        );
    }

    let ext = dylib_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    if !has_expected_dylib_extension(ext) {
        anyhow::bail!(
            "Invalid dylib extension: expected '{}', got '{}'",
            expected_dylib_extension(),
            ext
        );
    }

    Ok(())
}

fn has_expected_dylib_extension(ext: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        ext == "dylib"
    }
    #[cfg(target_os = "linux")]
    {
        ext == "so"
    }
    #[cfg(target_os = "windows")]
    {
        ext == "dll"
    }
}

fn expected_dylib_extension() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        ".dylib"
    }
    #[cfg(target_os = "linux")]
    {
        ".so"
    }
    #[cfg(target_os = "windows")]
    {
        ".dll"
    }
}

fn print_compact_json(json: &str) {
    println!("{json}");
}
