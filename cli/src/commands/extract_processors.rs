//! Hidden subcommand for processor metadata extraction via subprocess isolation.
//!
//! This command is invoked by `wavecraft start` and hot-reload to extract
//! processor metadata from a plugin dylib in a separate process.

use anyhow::{Context, Result};
use std::path::PathBuf;
use wavecraft_bridge::PluginParamLoader;

/// Execute the extract-processors subcommand.
///
/// Loads the plugin dylib via FFI, extracts processor metadata, and prints JSON
/// to stdout. Any errors are written to stderr with appropriate exit codes.
pub fn execute(dylib_path: PathBuf) -> Result<()> {
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

    let processors = PluginParamLoader::load_processors_only(&dylib_path)
        .with_context(|| format!("Failed to load processors from {}", dylib_path.display()))?;

    let json =
        serde_json::to_string(&processors).context("Failed to serialize processors to JSON")?;

    println!("{}", json);

    Ok(())
}
