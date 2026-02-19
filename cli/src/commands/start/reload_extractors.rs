use anyhow::{Context, Result};
use console::style;
use std::path::{Path, PathBuf};

use crate::project::find_plugin_dylib;
use wavecraft_protocol::{ParameterInfo, ProcessorInfo};

/// Load parameters from the rebuilt dylib via subprocess isolation.
///
/// To avoid dylib caching issues on macOS, we copy the dylib to a unique
/// temporary location before loading. The subprocess extracts parameters
/// and exits cleanly, with the temp file deleted after.
///
/// This function stays in the CLI because it depends on CLI-specific
/// infrastructure: `find_plugin_dylib`, `extract_params_subprocess`,
/// and `create_temp_dylib_copy`.
pub(super) async fn load_parameters_from_dylib(engine_dir: PathBuf) -> Result<Vec<ParameterInfo>> {
    println!("  {} Finding plugin dylib...", style("→").dim());
    let lib_path =
        find_plugin_dylib(&engine_dir).context("Failed to find plugin dylib after rebuild")?;
    println!("  {} Found: {}", style("→").dim(), lib_path.display());

    println!("  {} Copying to temp location...", style("→").dim());
    let temp_path = create_temp_dylib_copy(&lib_path)?;
    println!("  {} Temp: {}", style("→").dim(), temp_path.display());

    println!(
        "  {} Loading parameters via subprocess...",
        style("→").dim()
    );
    let params = crate::project::param_extract::extract_params_subprocess(
        &temp_path,
        crate::project::param_extract::DEFAULT_EXTRACT_TIMEOUT,
    )
    .await
    .with_context(|| format!("Failed to extract parameters from: {}", temp_path.display()))?;
    println!(
        "  {} Loaded {} parameters via subprocess",
        style("→").dim(),
        params.len()
    );

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    Ok(params)
}

/// Load processors from the rebuilt dylib via subprocess isolation.
pub(super) async fn load_processors_from_dylib(engine_dir: PathBuf) -> Result<Vec<ProcessorInfo>> {
    println!("  {} Finding plugin dylib...", style("→").dim());
    let lib_path =
        find_plugin_dylib(&engine_dir).context("Failed to find plugin dylib after rebuild")?;
    println!("  {} Found: {}", style("→").dim(), lib_path.display());

    println!("  {} Copying to temp location...", style("→").dim());
    let temp_path = create_temp_dylib_copy(&lib_path)?;
    println!("  {} Temp: {}", style("→").dim(), temp_path.display());

    println!(
        "  {} Loading processors via subprocess...",
        style("→").dim()
    );
    let processors = crate::project::param_extract::extract_processors_subprocess(
        &temp_path,
        crate::project::param_extract::DEFAULT_EXTRACT_TIMEOUT,
    )
    .await
    .with_context(|| format!("Failed to extract processors from: {}", temp_path.display()))?;

    let _ = std::fs::remove_file(&temp_path);

    println!(
        "  {} Loaded {} processors via subprocess",
        style("→").dim(),
        processors.len()
    );

    Ok(processors)
}

/// Create a temporary copy of the dylib with a unique name.
///
/// This ensures libloading loads a fresh dylib rather than returning
/// a cached handle from a previous load of the same path.
fn create_temp_dylib_copy(dylib_path: &Path) -> Result<PathBuf> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let extension = dylib_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("dylib");

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);

    let temp_name = format!("wavecraft_hotreload_{}.{}", timestamp, extension);
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(temp_name);

    std::fs::copy(dylib_path, &temp_path).with_context(|| {
        format!(
            "Failed to copy dylib to temp location: {}",
            temp_path.display()
        )
    })?;

    Ok(temp_path)
}
