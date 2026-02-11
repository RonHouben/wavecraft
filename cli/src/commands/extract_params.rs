//! Hidden subcommand for parameter extraction via subprocess isolation.
//!
//! This command is invoked by `wavecraft start` and hot-reload to extract
//! parameter metadata from a plugin dylib in a separate process. This isolates
//! the `dlopen` call, allowing the parent to kill the subprocess if it hangs
//! due to static initializers in nih-plug dependencies.

use anyhow::{Context, Result};
use std::path::PathBuf;
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
    // Validate the dylib path exists
    if !dylib_path.exists() {
        anyhow::bail!(
            "Plugin dylib not found: {}\nEnsure the plugin was built successfully.",
            dylib_path.display()
        );
    }

    // Validate the file extension
    let ext = dylib_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    
    #[cfg(target_os = "macos")]
    let valid_ext = ext == "dylib";
    #[cfg(target_os = "linux")]
    let valid_ext = ext == "so";
    #[cfg(target_os = "windows")]
    let valid_ext = ext == "dll";
    
    if !valid_ext {
        #[cfg(target_os = "macos")]
        let expected = ".dylib";
        #[cfg(target_os = "linux")]
        let expected = ".so";
        #[cfg(target_os = "windows")]
        let expected = ".dll";
        
        anyhow::bail!(
            "Invalid dylib extension: expected '{}', got '{}'",
            expected,
            ext
        );
    }

    // Load parameters via FFI (this is the only place dlopen happens in subprocess)
    let params = PluginParamLoader::load_params_only(&dylib_path)
        .with_context(|| format!("Failed to load parameters from {}", dylib_path.display()))?;

    // Serialize to compact JSON (minimize pipe overhead)
    let json = serde_json::to_string(&params).context("Failed to serialize parameters to JSON")?;

    // Write to stdout (parent will parse this)
    println!("{}", json);

    Ok(())
}
