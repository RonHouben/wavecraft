use anyhow::{Context, Result};
use console::style;
use std::future::Future;
use std::path::{Path, PathBuf};

use crate::project::find_plugin_dylib;
use wavecraft_protocol::{ParameterInfo, ProcessorInfo};

fn prepare_temp_dylib(engine_dir: &Path) -> Result<PathBuf> {
    println!("  {} Finding plugin dylib...", style("→").dim());
    let lib_path =
        find_plugin_dylib(engine_dir).context("Failed to find plugin dylib after rebuild")?;
    println!("  {} Found: {}", style("→").dim(), lib_path.display());

    println!("  {} Copying to temp location...", style("→").dim());
    let temp_path = create_temp_dylib_copy(&lib_path)?;
    println!("  {} Temp: {}", style("→").dim(), temp_path.display());

    Ok(temp_path)
}

async fn load_from_temp_dylib<T, F, Fut>(
    engine_dir: PathBuf,
    noun: &str,
    extract: F,
) -> Result<Vec<T>>
where
    F: FnOnce(PathBuf) -> Fut,
    Fut: Future<Output = Result<Vec<T>>>,
{
    let temp_path = prepare_temp_dylib(&engine_dir)?;

    println!("  {} Loading {} via subprocess...", style("→").dim(), noun);
    let result = extract(temp_path.clone())
        .await
        .with_context(|| format!("Failed to extract {} from: {}", noun, temp_path.display()));

    // Clean up temp file regardless of extract result.
    let _ = std::fs::remove_file(&temp_path);

    let values = result?;

    println!(
        "  {} Loaded {} {} via subprocess",
        style("→").dim(),
        values.len(),
        noun
    );

    Ok(values)
}

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
    load_from_temp_dylib(engine_dir, "parameters", |temp_path| async move {
        crate::project::param_extract::extract_params_subprocess(
            &temp_path,
            crate::project::param_extract::DEFAULT_EXTRACT_TIMEOUT,
        )
        .await
    })
    .await
}

/// Load processors from the rebuilt dylib via subprocess isolation.
pub(super) async fn load_processors_from_dylib(engine_dir: PathBuf) -> Result<Vec<ProcessorInfo>> {
    load_from_temp_dylib(engine_dir, "processors", |temp_path| async move {
        crate::project::param_extract::extract_processors_subprocess(
            &temp_path,
            crate::project::param_extract::DEFAULT_EXTRACT_TIMEOUT,
        )
        .await
    })
    .await
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
