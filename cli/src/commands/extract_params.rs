//! Hidden subcommand for parameter extraction via subprocess isolation.
//!
//! This command is invoked by `wavecraft start` and hot-reload to extract
//! parameter metadata from a plugin dylib in a separate process. This isolates
//! the `dlopen` call, allowing the parent to kill the subprocess if it hangs
//! due to static initializers in nih-plug dependencies.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use wavecraft_bridge::PluginParamLoader;

/// Execute the extract-params subcommand.
///
/// Loads the plugin dylib via FFI, extracts parameters, and prints JSON
/// to stdout. Any errors are written to stderr with appropriate exit codes.
///
/// # Exit Codes
///
/// - 0: Success (JSON written to stdout)
/// - 1: General error (see stderr)
/// - 2: `dlopen` failed
/// - 3: Required FFI symbol not found
/// - 4: JSON serialization failed
/// - 5: Invalid UTF-8 in FFI response
pub fn execute(dylib_path: PathBuf) -> Result<()> {
    validate_dylib_path(&dylib_path)?;

    // Load parameters via FFI (this is the only place dlopen happens in subprocess)
    let params = PluginParamLoader::load_params_only(&dylib_path)
        .with_context(|| format!("Failed to load parameters from {}", dylib_path.display()))?;

    // Serialize to compact JSON and write to stdout (parent will parse this)
    let json = serde_json::to_string(&params).context("Failed to serialize parameters to JSON")?;
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
