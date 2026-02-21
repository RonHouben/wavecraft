use anyhow::{bail, Context, Result};
use console::style;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::SystemTime;
use walkdir::WalkDir;
use wavecraft_protocol::{ParameterInfo, ProcessorInfo};

use crate::project::{
    find_plugin_dylib,
    param_extract::{
        extract_params_subprocess, extract_processors_subprocess, DEFAULT_EXTRACT_TIMEOUT,
    },
    resolve_debug_dir,
    ts_codegen::{write_parameter_types, write_processor_types},
    ProjectMarkers,
};

const PARAM_SIDECAR_FILENAME: &str = "wavecraft-params.json";
const PROCESSOR_SIDECAR_FILENAME: &str = "wavecraft-processors.json";

enum SidecarDecision {
    Missing,
    Stale(&'static str),
    Fresh,
}

pub(super) fn refresh_generated_types(project: &ProjectMarkers, package_name: &str) -> Result<()> {
    println!(
        "{} Refreshing generated parameter/processor types...",
        style("→").cyan()
    );

    let (params, processors) = try_load_metadata_sidecars(&project.engine_dir)?.map_or_else(
        || discover_plugin_metadata(&project.engine_dir, package_name),
        Ok,
    )?;

    write_parameter_types(&project.ui_dir, &params)
        .context("Failed to write generated TypeScript parameter IDs")?;
    write_processor_types(&project.ui_dir, &processors)
        .context("Failed to write generated TypeScript processor IDs")?;

    println!(
        "{} Generated contract types synced ({} parameters, {} processors)",
        style("✓").green(),
        params.len(),
        processors.len()
    );

    Ok(())
}

fn try_load_metadata_sidecars(
    engine_dir: &Path,
) -> Result<Option<(Vec<ParameterInfo>, Vec<ProcessorInfo>)>> {
    let debug_dir = match resolve_debug_dir(engine_dir) {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };

    let params_path = debug_dir.join(PARAM_SIDECAR_FILENAME);
    let processors_path = debug_dir.join(PROCESSOR_SIDECAR_FILENAME);

    match decide_sidecar_usage(engine_dir, &params_path, &processors_path) {
        SidecarDecision::Missing => return Ok(None),
        SidecarDecision::Stale(reason) => {
            println!(
                "{} Metadata sidecars stale ({}); running discovery build...",
                style("→").cyan(),
                reason
            );
            return Ok(None);
        }
        SidecarDecision::Fresh => {}
    }

    let params_contents = read_sidecar_contents(&params_path, "parameter")?;
    let processors_contents = read_sidecar_contents(&processors_path, "processor")?;

    let params: Vec<ParameterInfo> = serde_json::from_str(&params_contents).with_context(|| {
        format!(
            "Failed to parse parameter sidecar JSON at {}",
            params_path.display()
        )
    })?;

    let processors: Vec<ProcessorInfo> =
        serde_json::from_str(&processors_contents).with_context(|| {
            format!(
                "Failed to parse processor sidecar JSON at {}",
                processors_path.display()
            )
        })?;

    println!(
        "{} Loaded metadata sidecars from {}",
        style("✓").green(),
        debug_dir.display()
    );

    Ok(Some((params, processors)))
}

fn decide_sidecar_usage(
    engine_dir: &Path,
    params_path: &Path,
    processors_path: &Path,
) -> SidecarDecision {
    if !params_path.is_file() || !processors_path.is_file() {
        return SidecarDecision::Missing;
    }

    match metadata_sidecars_stale_reason(engine_dir, params_path, processors_path) {
        Some(reason) => SidecarDecision::Stale(reason),
        None => SidecarDecision::Fresh,
    }
}

fn read_sidecar_contents(path: &Path, sidecar_kind: &str) -> Result<String> {
    fs::read_to_string(path).with_context(|| {
        format!(
            "Failed to read {} sidecar at {}",
            sidecar_kind,
            path.display()
        )
    })
}

fn metadata_sidecars_stale_reason(
    engine_dir: &Path,
    params_path: &Path,
    processors_path: &Path,
) -> Option<&'static str> {
    let params_mtime = file_mtime(params_path)?;
    let processors_mtime = file_mtime(processors_path)?;
    let sidecar_mtime = std::cmp::min(params_mtime, processors_mtime);

    let dylib_path = find_plugin_dylib(engine_dir).ok()?;
    let dylib_mtime = file_mtime(&dylib_path)?;
    if dylib_mtime > sidecar_mtime {
        return Some("plugin dylib newer than sidecars");
    }

    if let Some(src_mtime) = newest_file_mtime_under(&engine_dir.join("src")) {
        if src_mtime > sidecar_mtime {
            return Some("engine source newer than sidecars");
        }
    }

    if let Some(cli_mtime) = current_exe_mtime() {
        if cli_mtime > sidecar_mtime {
            return Some("CLI binary newer than sidecars");
        }
    }

    None
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
    file_mtime(&current_exe)
}

fn file_mtime(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

fn discover_plugin_metadata(
    engine_dir: &Path,
    package_name: &str,
) -> Result<(Vec<ParameterInfo>, Vec<ProcessorInfo>)> {
    println!(
        "{} Metadata sidecars not found; running discovery build...",
        style("→").cyan()
    );

    let status = Command::new("cargo")
        .args([
            "build",
            "--lib",
            "--features",
            "_param-discovery",
            "-p",
            package_name,
        ])
        .current_dir(engine_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run metadata discovery build")?;

    if !status.success() {
        let code = status.code().map_or_else(
            || "terminated by signal".to_string(),
            |value| value.to_string(),
        );

        bail!("Metadata discovery build failed (exit: {}).", code);
    }

    let dylib_path =
        find_plugin_dylib(engine_dir).context("Failed to locate plugin library for metadata")?;

    let runtime = tokio::runtime::Runtime::new()
        .context("Failed to create runtime for metadata extraction")?;

    let params = runtime
        .block_on(extract_params_subprocess(
            &dylib_path,
            DEFAULT_EXTRACT_TIMEOUT,
        ))
        .with_context(|| format!("Failed to extract parameters from {}", dylib_path.display()))?;

    let processors = runtime
        .block_on(extract_processors_subprocess(
            &dylib_path,
            DEFAULT_EXTRACT_TIMEOUT,
        ))
        .with_context(|| format!("Failed to extract processors from {}", dylib_path.display()))?;

    Ok((params, processors))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn stale_sidecars_are_ignored_when_engine_source_is_newer() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let engine_dir = temp.path().join("engine");
        let src_dir = engine_dir.join("src");
        let debug_dir = engine_dir.join("target").join("debug");

        fs::create_dir_all(&src_dir).expect("src dir should be created");
        fs::create_dir_all(&debug_dir).expect("debug dir should be created");

        fs::write(
            engine_dir.join("Cargo.toml"),
            "[package]\nname = \"wavecraft-dev-template\"\n[lib]\nname = \"wavecraft_dev_template\"\n",
        )
        .expect("Cargo.toml should be written");

        #[cfg(target_os = "macos")]
        let dylib_name = "libwavecraft_dev_template.dylib";
        #[cfg(target_os = "linux")]
        let dylib_name = "libwavecraft_dev_template.so";
        #[cfg(target_os = "windows")]
        let dylib_name = "wavecraft_dev_template.dll";

        fs::write(debug_dir.join(dylib_name), b"test dylib")
            .expect("dylib placeholder should be written");

        fs::write(
            debug_dir.join(PARAM_SIDECAR_FILENAME),
            r#"[{"id":"stale_param","name":"Stale","type":"float","value":0.0,"default":0.0,"min":0.0,"max":1.0,"unit":null,"group":null,"variants":null}]"#,
        )
        .expect("write stale param sidecar");
        fs::write(
            debug_dir.join(PROCESSOR_SIDECAR_FILENAME),
            r#"[{"id":"stale"}]"#,
        )
        .expect("write stale processor sidecar");

        thread::sleep(std::time::Duration::from_millis(20));
        fs::write(src_dir.join("lib.rs"), "// newer source").expect("source should be written");

        let loaded = try_load_metadata_sidecars(&engine_dir).expect("sidecar load should not fail");
        assert!(
            loaded.is_none(),
            "stale sidecars should be ignored so bundle falls back to discovery"
        );
    }

    #[test]
    fn fresh_sidecars_are_used_for_contract_refresh() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let engine_dir = temp.path().join("engine");
        let src_dir = engine_dir.join("src");
        let debug_dir = engine_dir.join("target").join("debug");

        fs::create_dir_all(&src_dir).expect("src dir should be created");
        fs::create_dir_all(&debug_dir).expect("debug dir should be created");

        fs::write(
            engine_dir.join("Cargo.toml"),
            "[package]\nname = \"wavecraft-dev-template\"\n[lib]\nname = \"wavecraft_dev_template\"\n",
        )
        .expect("Cargo.toml should be written");
        fs::write(src_dir.join("lib.rs"), "// source").expect("source should be written");

        #[cfg(target_os = "macos")]
        let dylib_name = "libwavecraft_dev_template.dylib";
        #[cfg(target_os = "linux")]
        let dylib_name = "libwavecraft_dev_template.so";
        #[cfg(target_os = "windows")]
        let dylib_name = "wavecraft_dev_template.dll";

        fs::write(debug_dir.join(dylib_name), b"test dylib")
            .expect("dylib placeholder should be written");

        thread::sleep(std::time::Duration::from_millis(20));
        fs::write(
            debug_dir.join(PARAM_SIDECAR_FILENAME),
            r#"[{"id":"oscillator_enabled","name":"Oscillator Enabled","type":"bool","value":1.0,"default":1.0,"min":0.0,"max":1.0,"unit":null,"group":"Oscillator","variants":null}]"#,
        )
        .expect("write param sidecar");
        fs::write(
            debug_dir.join(PROCESSOR_SIDECAR_FILENAME),
            r#"[{"id":"oscillator"}]"#,
        )
        .expect("write processor sidecar");

        let loaded = try_load_metadata_sidecars(&engine_dir).expect("sidecar load should not fail");
        let (params, processors) = loaded.expect("fresh sidecars should be used");

        assert_eq!(params.len(), 1);
        assert_eq!(params[0].id, "oscillator_enabled");
        assert_eq!(processors.len(), 1);
        assert_eq!(processors[0].id, "oscillator");
    }
}
