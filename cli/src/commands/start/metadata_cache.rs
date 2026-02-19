use anyhow::{Context, Result};
use console::style;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::project::{find_plugin_dylib, read_engine_package_name, resolve_debug_dir};
use wavecraft_protocol::{ParameterInfo, ProcessorInfo};

/// Path to the sidecar parameter cache file.
const PARAM_SIDECAR_FILENAME: &str = "wavecraft-params.json";
const PROCESSOR_SIDECAR_FILENAME: &str = "wavecraft-processors.json";

#[derive(Debug, Clone)]
pub(super) struct PluginMetadata {
    pub(super) params: Vec<ParameterInfo>,
    pub(super) processors: Vec<ProcessorInfo>,
}

fn sidecar_json_path(engine_dir: &Path, file_name: &str) -> Result<PathBuf> {
    let debug_dir = resolve_debug_dir(engine_dir)?;
    Ok(debug_dir.join(file_name))
}

fn params_sidecar_json_path(engine_dir: &Path) -> Result<PathBuf> {
    sidecar_json_path(engine_dir, PARAM_SIDECAR_FILENAME)
}

fn processors_sidecar_json_path(engine_dir: &Path) -> Result<PathBuf> {
    sidecar_json_path(engine_dir, PROCESSOR_SIDECAR_FILENAME)
}

/// Try reading cached parameters from the sidecar JSON file.
///
/// Returns `Some(params)` if the file exists and is newer than the dylib
/// (i.e., no source changes since last extraction). Returns `None` otherwise.
pub(super) fn try_read_cached_params(engine_dir: &Path) -> Option<Vec<ParameterInfo>> {
    let sidecar_path = params_sidecar_json_path(engine_dir).ok()?;
    if !sidecar_path.exists() {
        return None;
    }

    // Check if sidecar is still valid.
    //
    // A sidecar is valid only when it is newer than:
    // - the compiled dylib currently used for extraction
    // - the newest file under engine/src (source edits before rebuild)
    // - the currently running CLI binary (cache format/logic migrations)
    let dylib_path = find_plugin_dylib(engine_dir).ok()?;
    let sidecar_mtime = std::fs::metadata(&sidecar_path).ok()?.modified().ok()?;
    let dylib_mtime = std::fs::metadata(&dylib_path).ok()?.modified().ok()?;

    if dylib_mtime > sidecar_mtime {
        println!("  Sidecar cache stale (dylib newer), rebuilding...");

        return None;
    }

    if let Some(src_mtime) = newest_file_mtime_under(&engine_dir.join("src")) {
        if src_mtime > sidecar_mtime {
            println!("  Sidecar cache stale (engine source newer), rebuilding...");

            return None;
        }
    }

    if let Some(cli_mtime) = current_exe_mtime() {
        if cli_mtime > sidecar_mtime {
            println!("  Sidecar cache stale (CLI binary newer), rebuilding...");

            return None;
        }
    }

    // Load parameters from JSON file (inline to avoid publish dep issues)
    let contents = std::fs::read_to_string(&sidecar_path).ok()?;
    serde_json::from_str(&contents).ok()
}

/// Try reading cached processors from sidecar JSON file.
fn try_read_cached_processors(engine_dir: &Path) -> Option<Vec<ProcessorInfo>> {
    let sidecar_path = processors_sidecar_json_path(engine_dir).ok()?;
    if !sidecar_path.exists() {
        return None;
    }

    let dylib_path = find_plugin_dylib(engine_dir).ok()?;
    let sidecar_mtime = std::fs::metadata(&sidecar_path).ok()?.modified().ok()?;
    let dylib_mtime = std::fs::metadata(&dylib_path).ok()?.modified().ok()?;

    if dylib_mtime > sidecar_mtime {
        println!("  Processor sidecar cache stale (dylib newer), rebuilding...");
        return None;
    }

    if let Some(src_mtime) = newest_file_mtime_under(&engine_dir.join("src")) {
        if src_mtime > sidecar_mtime {
            println!("  Processor sidecar cache stale (engine source newer), rebuilding...");
            return None;
        }
    }

    if let Some(cli_mtime) = current_exe_mtime() {
        if cli_mtime > sidecar_mtime {
            println!("  Processor sidecar cache stale (CLI binary newer), rebuilding...");
            return None;
        }
    }

    let contents = std::fs::read_to_string(&sidecar_path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn newest_file_mtime_under(root: &Path) -> Option<SystemTime> {
    if !root.is_dir() {
        return None;
    }

    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| entry.metadata().ok())
        .filter_map(|metadata| metadata.modified().ok())
        .max()
}

fn current_exe_mtime() -> Option<SystemTime> {
    let current_exe = std::env::current_exe().ok()?;
    std::fs::metadata(current_exe).ok()?.modified().ok()
}

/// Write parameter metadata to the sidecar JSON cache.
pub(crate) fn write_sidecar_cache(engine_dir: &Path, params: &[ParameterInfo]) -> Result<()> {
    let sidecar_path = params_sidecar_json_path(engine_dir)?;
    let json = serde_json::to_string_pretty(params).context("Failed to serialize parameters")?;
    std::fs::write(&sidecar_path, json).context("Failed to write sidecar cache")?;
    Ok(())
}

fn write_processors_sidecar_cache(engine_dir: &Path, processors: &[ProcessorInfo]) -> Result<()> {
    let sidecar_path = processors_sidecar_json_path(engine_dir)?;
    let json =
        serde_json::to_string_pretty(processors).context("Failed to serialize processors")?;
    std::fs::write(&sidecar_path, json).context("Failed to write processor sidecar cache")?;
    Ok(())
}

/// Load plugin metadata (parameters + processors) using cached sidecars or
/// feature-gated discovery build.
pub(super) async fn load_plugin_metadata(engine_dir: &Path) -> Result<PluginMetadata> {
    // 1. Try cached sidecars
    if let (Some(params), Some(processors)) = (
        try_read_cached_params(engine_dir),
        try_read_cached_processors(engine_dir),
    ) {
        println!(
            "{} Loaded {} parameters and {} processors (cached)",
            style("✓").green(),
            params.len(),
            processors.len()
        );
        return Ok(PluginMetadata { params, processors });
    }

    // 2. Build with _param-discovery feature (skip nih-plug exports)
    println!("{} Building for metadata discovery...", style("→").cyan());

    let mut build_cmd = Command::new("cargo");
    build_cmd.args(["build", "--lib", "--features", "_param-discovery"]);

    if let Some(package_name) = read_engine_package_name(engine_dir) {
        build_cmd.args(["--package", &package_name]);
    }

    let build_result = build_cmd
        .current_dir(engine_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    let status = build_result.context("Failed to run cargo build for metadata discovery")?;
    if !status.success() {
        anyhow::bail!(
            "Metadata discovery build failed. This project must support --features _param-discovery in SDK dev mode."
        );
    }

    let dylib_path = find_plugin_dylib(engine_dir)
        .context("Failed to find plugin library after discovery build")?;

    println!("  Found dylib: {}", dylib_path.display());

    println!("{} Loading plugin metadata...", style("→").cyan());
    #[cfg(feature = "audio-dev")]
    let (params, processors) = {
        let loader = super::PluginLoader::load(&dylib_path)
            .context("Failed to load plugin for metadata discovery")?;
        (loader.parameters().to_vec(), loader.processors().to_vec())
    };
    #[cfg(not(feature = "audio-dev"))]
    let (params, processors) = {
        let params = crate::project::param_extract::extract_params_subprocess(
            &dylib_path,
            crate::project::param_extract::DEFAULT_EXTRACT_TIMEOUT,
        )
        .await
        .context("Failed to extract parameters from plugin")?;
        let processors = crate::project::param_extract::extract_processors_subprocess(
            &dylib_path,
            crate::project::param_extract::DEFAULT_EXTRACT_TIMEOUT,
        )
        .await
        .context("Failed to extract processors from plugin")?;
        (params, processors)
    };

    if let Err(e) = write_sidecar_cache(engine_dir, &params) {
        println!("  Warning: failed to write param cache: {}", e);
    }
    if let Err(e) = write_processors_sidecar_cache(engine_dir, &processors) {
        println!("  Warning: failed to write processor cache: {}", e);
    }

    println!(
        "{} Loaded {} parameters and {} processors",
        style("✓").green(),
        params.len(),
        processors.len()
    );

    Ok(PluginMetadata { params, processors })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{try_read_cached_params, write_sidecar_cache};
    use wavecraft_protocol::{ParameterInfo, ParameterType};

    #[test]
    fn cached_sidecar_path_preserves_full_frequency_range_for_browser_dev_mode() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let engine_dir = temp.path().join("engine");
        let src_dir = engine_dir.join("src");
        let debug_dir = engine_dir.join("target").join("debug");

        fs::create_dir_all(&src_dir).expect("src dir should be created");
        fs::create_dir_all(&debug_dir).expect("debug dir should be created");

        // Build output discovery depends on Cargo.toml + dylib naming convention.
        fs::write(
            engine_dir.join("Cargo.toml"),
            "[package]\nname = \"wavecraft-dev-template\"\n[lib]\nname = \"wavecraft_dev_template\"\n",
        )
        .expect("Cargo.toml should be written");

        fs::write(src_dir.join("lib.rs"), "// test source").expect("source file should be written");

        #[cfg(target_os = "macos")]
        let dylib_name = "libwavecraft_dev_template.dylib";
        #[cfg(target_os = "linux")]
        let dylib_name = "libwavecraft_dev_template.so";
        #[cfg(target_os = "windows")]
        let dylib_name = "wavecraft_dev_template.dll";

        fs::write(debug_dir.join(dylib_name), b"test dylib")
            .expect("dylib placeholder should be written");

        let params = vec![ParameterInfo {
            id: "oscillator_frequency".to_string(),
            name: "Frequency".to_string(),
            param_type: ParameterType::Float,
            value: 440.0,
            default: 440.0,
            min: 20.0,
            max: 5_000.0,
            unit: Some("Hz".to_string()),
            group: Some("Oscillator".to_string()),
            variants: None,
        }];

        write_sidecar_cache(&engine_dir, &params).expect("sidecar cache should be written");

        let cached = try_read_cached_params(&engine_dir)
            .expect("cached sidecar should be used in start path");

        let frequency = cached
            .iter()
            .find(|param| param.id == "oscillator_frequency")
            .expect("frequency parameter should exist");

        assert!((frequency.min - 20.0).abs() < f32::EPSILON);
        assert!((frequency.max - 5_000.0).abs() < f32::EPSILON);
        assert!((frequency.value - 440.0).abs() < f32::EPSILON);
    }
}
