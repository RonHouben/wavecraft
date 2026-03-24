use anyhow::{Context, Result};
use console::style;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::project::{find_plugin_dylib, read_engine_package_name, resolve_debug_dir};
use wavecraft_protocol::{ParameterInfo, ProcessorInfo, SignalChainSlot};

/// Path to the sidecar parameter cache file.
const PARAM_SIDECAR_FILENAME: &str = "wavecraft-params.json";
const PROCESSOR_SIDECAR_FILENAME: &str = "wavecraft-processors.json";
const SIGNAL_CHAIN_SLOTS_SIDECAR_FILENAME: &str = "wavecraft-signal-chain-slots.json";

#[derive(Debug, Clone)]
pub(super) struct PluginMetadata {
    pub(super) params: Vec<ParameterInfo>,
    pub(super) processors: Vec<ProcessorInfo>,
    pub(super) signal_chain_slots: Vec<SignalChainSlot>,
}

fn legacy_soft_clip_schema_reason(params: &[ParameterInfo]) -> Option<&'static str> {
    let ids: HashSet<&str> = params.iter().map(|param| param.id.as_str()).collect();

    if !ids.iter().any(|id| id.starts_with("soft_clip_")) {
        return None;
    }

    if ids.contains("soft_clip_output_trim_db") {
        return Some("legacy soft_clip_output_trim_db parameter id");
    }

    let has_drive = ids.contains("soft_clip_drive_db");
    let has_output = ids.contains("soft_clip_output_db");
    let has_mix = ids.contains("soft_clip_mix");
    let has_tone = ids.contains("soft_clip_tone");

    if has_drive && (!has_output || !has_mix || !has_tone) {
        return Some("incomplete soft_clip parameter schema");
    }

    None
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

fn signal_chain_slots_sidecar_json_path(engine_dir: &Path) -> Result<PathBuf> {
    sidecar_json_path(engine_dir, SIGNAL_CHAIN_SLOTS_SIDECAR_FILENAME)
}

fn stale_sidecar_reason(
    engine_dir: &Path,
    dylib_path: &Path,
    sidecar_mtime: SystemTime,
) -> Option<&'static str> {
    let dylib_mtime = std::fs::metadata(dylib_path).ok()?.modified().ok()?;
    if dylib_mtime > sidecar_mtime {
        return Some("dylib newer");
    }

    if let Some(src_mtime) = newest_file_mtime_under(&engine_dir.join("src")) {
        if src_mtime > sidecar_mtime {
            return Some("engine source newer");
        }
    }

    if let Some(processors_inputs_mtime) = sdk_processors_inputs_mtime(engine_dir) {
        if processors_inputs_mtime > sidecar_mtime {
            return Some("wavecraft-processors inputs newer");
        }
    }

    if let Some(cli_mtime) = current_exe_mtime() {
        if cli_mtime > sidecar_mtime {
            return Some("CLI binary newer");
        }
    }

    None
}

fn sdk_processors_inputs_mtime(engine_dir: &Path) -> Option<SystemTime> {
    let sdk_template_dir = engine_dir.parent()?;
    if sdk_template_dir.file_name()?.to_str()? != "sdk-template" {
        return None;
    }

    let repo_root = sdk_template_dir.parent()?;
    let processors_crate = repo_root
        .join("engine")
        .join("crates")
        .join("wavecraft-processors");

    let processors_src = processors_crate.join("src");
    let processors_manifest = processors_crate.join("Cargo.toml");

    max_mtime(
        newest_file_mtime_under(&processors_src),
        file_mtime(&processors_manifest),
    )
}

fn try_read_cached_sidecar_json<T>(
    engine_dir: &Path,
    sidecar_path: &Path,
    stale_cache_name: &str,
    parse: impl FnOnce(&str) -> Option<Vec<T>>,
) -> Option<Vec<T>> {
    if !sidecar_path.exists() {
        return None;
    }

    let dylib_path = find_plugin_dylib(engine_dir).ok()?;
    let sidecar_mtime = std::fs::metadata(sidecar_path).ok()?.modified().ok()?;

    if let Some(reason) = stale_sidecar_reason(engine_dir, &dylib_path, sidecar_mtime) {
        println!("  {stale_cache_name} stale ({reason}), rebuilding...");
        return None;
    }

    let contents = std::fs::read_to_string(sidecar_path).ok()?;
    parse(&contents)
}

fn write_sidecar_json(
    engine_dir: &Path,
    file_name: &str,
    json: &str,
    write_error: &'static str,
) -> Result<()> {
    let sidecar_path = sidecar_json_path(engine_dir, file_name)?;
    std::fs::write(&sidecar_path, json).context(write_error)?;
    Ok(())
}

/// Try reading cached parameters from the sidecar JSON file.
///
/// Returns `Some(params)` if the file exists and is newer than the dylib
/// (i.e., no source changes since last extraction). Returns `None` otherwise.
pub(super) fn try_read_cached_params(engine_dir: &Path) -> Option<Vec<ParameterInfo>> {
    let sidecar_path = params_sidecar_json_path(engine_dir).ok()?;
    // A sidecar is valid only when it is newer than:
    // - the compiled dylib currently used for extraction
    // - the newest file under engine/src (source edits before rebuild)
    // - the currently running CLI binary (cache format/logic migrations)
    try_read_cached_sidecar_json(engine_dir, &sidecar_path, "Sidecar cache", |contents| {
        serde_json::from_str(contents).ok()
    })
}

/// Try reading cached processors from sidecar JSON file.
fn try_read_cached_processors(engine_dir: &Path) -> Option<Vec<ProcessorInfo>> {
    let sidecar_path = processors_sidecar_json_path(engine_dir).ok()?;
    try_read_cached_sidecar_json(
        engine_dir,
        &sidecar_path,
        "Processor sidecar cache",
        |contents| serde_json::from_str(contents).ok(),
    )
}

fn try_read_cached_signal_chain_slots(engine_dir: &Path) -> Option<Vec<SignalChainSlot>> {
    let sidecar_path = signal_chain_slots_sidecar_json_path(engine_dir).ok()?;
    try_read_cached_sidecar_json(
        engine_dir,
        &sidecar_path,
        "Signal-chain sidecar cache",
        |contents| serde_json::from_str(contents).ok(),
    )
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

fn file_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

fn max_mtime(a: Option<SystemTime>, b: Option<SystemTime>) -> Option<SystemTime> {
    match (a, b) {
        (Some(a), Some(b)) => Some(std::cmp::max(a, b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Write parameter metadata to the sidecar JSON cache.
pub(crate) fn write_sidecar_cache(engine_dir: &Path, params: &[ParameterInfo]) -> Result<()> {
    let json = serde_json::to_string_pretty(params).context("Failed to serialize parameters")?;
    write_sidecar_json(
        engine_dir,
        PARAM_SIDECAR_FILENAME,
        &json,
        "Failed to write sidecar cache",
    )
}

fn write_processors_sidecar_cache(engine_dir: &Path, processors: &[ProcessorInfo]) -> Result<()> {
    let json =
        serde_json::to_string_pretty(processors).context("Failed to serialize processors")?;
    write_sidecar_json(
        engine_dir,
        PROCESSOR_SIDECAR_FILENAME,
        &json,
        "Failed to write processor sidecar cache",
    )
}

fn write_signal_chain_slots_sidecar_cache(
    engine_dir: &Path,
    signal_chain_slots: &[SignalChainSlot],
) -> Result<()> {
    let json = serde_json::to_string_pretty(signal_chain_slots)
        .context("Failed to serialize signal-chain slots")?;
    write_sidecar_json(
        engine_dir,
        SIGNAL_CHAIN_SLOTS_SIDECAR_FILENAME,
        &json,
        "Failed to write signal-chain sidecar cache",
    )
}

/// Load plugin metadata (parameters + processors) using cached sidecars or
/// feature-gated discovery build.
pub(super) async fn load_plugin_metadata(engine_dir: &Path) -> Result<PluginMetadata> {
    // 1. Try cached sidecars
    if let (Some(params), Some(processors), Some(signal_chain_slots)) = (
        try_read_cached_params(engine_dir),
        try_read_cached_processors(engine_dir),
        try_read_cached_signal_chain_slots(engine_dir),
    ) {
        if let Some(reason) = legacy_soft_clip_schema_reason(&params) {
            println!("  Sidecar cache stale ({reason}), rebuilding...");
        } else {
            println!(
                "{} Loaded {} parameters, {} processors, and {} signal-chain slots (cached)",
                style("✓").green(),
                params.len(),
                processors.len(),
                signal_chain_slots.len()
            );
            return Ok(PluginMetadata {
                params,
                processors,
                signal_chain_slots,
            });
        }
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
    let (params, processors, signal_chain_slots) = {
        let loader = super::PluginLoader::load(&dylib_path)
            .context("Failed to load plugin for metadata discovery")?;
        (
            loader.parameters().to_vec(),
            loader.processors().to_vec(),
            loader.signal_chain_slots().to_vec(),
        )
    };
    #[cfg(not(feature = "audio-dev"))]
    let (params, processors, signal_chain_slots) = {
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
        (params, processors, Vec::new())
    };

    if let Err(e) = write_sidecar_cache(engine_dir, &params) {
        println!("  Warning: failed to write param cache: {}", e);
    }
    if let Err(e) = write_processors_sidecar_cache(engine_dir, &processors) {
        println!("  Warning: failed to write processor cache: {}", e);
    }
    if let Err(e) = write_signal_chain_slots_sidecar_cache(engine_dir, &signal_chain_slots) {
        println!("  Warning: failed to write signal-chain slot cache: {}", e);
    }

    println!(
        "{} Loaded {} parameters, {} processors, and {} signal-chain slots",
        style("✓").green(),
        params.len(),
        processors.len(),
        signal_chain_slots.len()
    );

    Ok(PluginMetadata {
        params,
        processors,
        signal_chain_slots,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::thread;
    use std::time::Duration;

    use super::{legacy_soft_clip_schema_reason, try_read_cached_params, write_sidecar_cache};
    use wavecraft_protocol::{ParameterInfo, ParameterType};

    fn soft_clip_param(id: &str) -> ParameterInfo {
        ParameterInfo {
            id: id.to_string(),
            name: id.to_string(),
            param_type: ParameterType::Float,
            value: 0.0,
            default: 0.0,
            min: 0.0,
            max: 1.0,
            unit: None,
            group: Some("Saturator".to_string()),
            variants: None,
        }
    }

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

        let params = vec![
            ParameterInfo {
                id: "test_tone_enabled".to_string(),
                name: "Enabled".to_string(),
                param_type: ParameterType::Bool,
                value: 0.0,
                default: 0.0,
                min: 0.0,
                max: 1.0,
                unit: None,
                group: Some("Test Tone".to_string()),
                variants: None,
            },
            ParameterInfo {
                id: "test_tone_frequency".to_string(),
                name: "Frequency".to_string(),
                param_type: ParameterType::Float,
                value: 440.0,
                default: 440.0,
                min: 20.0,
                max: 20_000.0,
                unit: Some("Hz".to_string()),
                group: Some("Test Tone".to_string()),
                variants: None,
            },
        ];

        write_sidecar_cache(&engine_dir, &params).expect("sidecar cache should be written");

        let cached = try_read_cached_params(&engine_dir)
            .expect("cached sidecar should be used in start path");

        let frequency = cached
            .iter()
            .find(|param| param.id == "test_tone_frequency")
            .expect("frequency parameter should exist");
        let enabled = cached
            .iter()
            .find(|param| param.id == "test_tone_enabled")
            .expect("enabled parameter should exist");

        assert_eq!(enabled.param_type, ParameterType::Bool);
        assert!(enabled.default.abs() <= f32::EPSILON);

        assert!((frequency.min - 20.0).abs() < f32::EPSILON);
        assert!((frequency.max - 20_000.0).abs() < f32::EPSILON);
        assert!((frequency.value - 440.0).abs() < f32::EPSILON);
    }

    #[test]
    fn sdk_sidecar_cache_invalidates_when_wavecraft_processors_manifest_is_newer() {
        let temp = tempfile::tempdir().expect("temp dir should be created");
        let repo_root = temp.path();

        let engine_dir = repo_root.join("sdk-template").join("engine");
        let src_dir = engine_dir.join("src");
        let debug_dir = engine_dir.join("target").join("debug");

        fs::create_dir_all(&src_dir).expect("src dir should be created");
        fs::create_dir_all(&debug_dir).expect("debug dir should be created");

        let processors_crate = repo_root
            .join("engine")
            .join("crates")
            .join("wavecraft-processors");
        fs::create_dir_all(processors_crate.join("src")).expect("processors src dir");
        let processors_manifest = processors_crate.join("Cargo.toml");
        fs::write(
            &processors_manifest,
            "[package]\nname = \"wavecraft-processors\"\n",
        )
        .expect("processors manifest should be written");

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
            id: "test_tone_enabled".to_string(),
            name: "Enabled".to_string(),
            param_type: ParameterType::Bool,
            value: 0.0,
            default: 0.0,
            min: 0.0,
            max: 1.0,
            unit: None,
            group: Some("Test Tone".to_string()),
            variants: None,
        }];

        write_sidecar_cache(&engine_dir, &params).expect("sidecar cache should be written");
        let cached_before = try_read_cached_params(&engine_dir);
        assert!(
            cached_before.is_some(),
            "cache should be valid before processors manifest changes"
        );

        thread::sleep(Duration::from_millis(20));
        fs::write(
            &processors_manifest,
            "[package]\nname = \"wavecraft-processors\"\nversion = \"0.0.1\"\n",
        )
        .expect("processors manifest should be updated");

        let cached_after = try_read_cached_params(&engine_dir);
        assert!(
            cached_after.is_none(),
            "cache should be invalidated when wavecraft-processors/Cargo.toml is newer"
        );
    }

    #[test]
    fn soft_clip_schema_guard_rejects_legacy_output_trim_id() {
        let params = vec![
            soft_clip_param("soft_clip_bypass"),
            soft_clip_param("soft_clip_drive_db"),
            soft_clip_param("soft_clip_output_trim_db"),
        ];

        let reason = legacy_soft_clip_schema_reason(&params);
        assert_eq!(reason, Some("legacy soft_clip_output_trim_db parameter id"));
    }

    #[test]
    fn soft_clip_schema_guard_accepts_expanded_schema() {
        let params = vec![
            soft_clip_param("soft_clip_bypass"),
            soft_clip_param("soft_clip_drive_db"),
            soft_clip_param("soft_clip_output_db"),
            soft_clip_param("soft_clip_mix"),
            soft_clip_param("soft_clip_tone"),
        ];

        assert_eq!(legacy_soft_clip_schema_reason(&params), None);
    }
}
