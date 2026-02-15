//! Development server command - runs `wavecraft start` via the CLI.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::SystemTime;
use xtask::npm_command;
use xtask::output::*;
use xtask::paths;

/// Run the development server via `wavecraft start` CLI command.
///
/// This command invokes the Wavecraft CLI which manages:
/// 1. WebSocket server (Rust) for IPC
/// 2. Vite UI dev server
/// 3. Hot-reload pipeline (file watcher + rebuild)
/// 4. Optional in-process audio
///
/// # Arguments
/// * `port` - WebSocket server port (default: 9000)
/// * `verbose` - Show detailed output
pub fn run(port: u16, verbose: bool) -> Result<()> {
    print_header("Wavecraft Development Server");

    run_preflight(verbose)?;

    // Locate the CLI manifest relative to the engine directory
    let engine_dir = paths::engine_dir()?;
    let cli_manifest = engine_dir.join("../cli/Cargo.toml");

    let mut args = vec!["run", "--manifest-path"];
    let cli_manifest_str = cli_manifest.to_string_lossy().to_string();
    args.push(&cli_manifest_str);
    args.push("--features");
    args.push("audio-dev");
    args.push("--");
    args.push("start");

    let port_str = port.to_string();
    args.push("--port");
    args.push(&port_str);

    if verbose {
        args.push("--verbose");
    }

    println!();
    print_status(&format!("Starting wavecraft start (port {})", port));
    println!();

    let mut child = Command::new("cargo")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .context("Failed to start wavecraft CLI")?;

    // Set up Ctrl+C handler
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel");
    })
    .context("Error setting Ctrl+C handler")?;

    // Wait for Ctrl+C or CLI process to exit
    let child_result = thread::spawn(move || child.wait());

    match rx.recv_timeout(std::time::Duration::from_millis(100)) {
        Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => {
            println!();
            print_status("Shutting down...");
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            if child_result.is_finished() {
                // CLI exited on its own
            } else {
                let _ = rx.recv();
                println!();
                print_status("Shutting down...");
            }
        }
    }

    print_success("Development server stopped");
    Ok(())
}

fn run_preflight(verbose: bool) -> Result<()> {
    print_status("Preflight: checking dev artifacts and caches...");

    let project_root = paths::project_root()?;

    let ui_result = refresh_ui_package_artifacts_if_needed(&project_root, verbose)?;
    log_preflight_result("UI package artifacts", &ui_result);

    let cache_result = refresh_param_codegen_caches_if_needed(&project_root, verbose)?;
    log_preflight_result("parameter/typegen caches", &cache_result);

    println!();
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PreflightResult {
    refreshed: bool,
    detail: String,
}

impl PreflightResult {
    fn refreshed(detail: impl Into<String>) -> Self {
        Self {
            refreshed: true,
            detail: detail.into(),
        }
    }

    fn skipped(detail: impl Into<String>) -> Self {
        Self {
            refreshed: false,
            detail: detail.into(),
        }
    }
}

fn log_preflight_result(label: &str, result: &PreflightResult) {
    if result.refreshed {
        print_success_item(&format!(
            "Preflight: refreshed {} ({})",
            label, result.detail
        ));
    } else {
        print_info(&format!("Preflight: skipped {} ({})", label, result.detail));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefreshReason {
    MissingArtifact,
    InputsNewer,
    UpToDate,
    NoInputs,
}

fn decide_refresh(
    inputs_latest: Option<SystemTime>,
    outputs_latest: Option<SystemTime>,
) -> RefreshReason {
    match (inputs_latest, outputs_latest) {
        (_, None) => RefreshReason::MissingArtifact,
        (None, Some(_)) => RefreshReason::NoInputs,
        (Some(input), Some(output)) if input > output => RefreshReason::InputsNewer,
        (Some(_), Some(_)) => RefreshReason::UpToDate,
    }
}

fn refresh_ui_package_artifacts_if_needed(
    project_root: &Path,
    verbose: bool,
) -> Result<PreflightResult> {
    let ui_dir = project_root.join("ui");
    let core_dir = ui_dir.join("packages").join("core");
    let components_dir = ui_dir.join("packages").join("components");

    if !core_dir.exists() || !components_dir.exists() {
        return Ok(PreflightResult::skipped(
            "ui/packages/core or ui/packages/components missing",
        ));
    }

    let input_roots = vec![
        ui_dir.join("package.json"),
        core_dir.join("package.json"),
        core_dir.join("vite.lib.config.ts"),
        core_dir.join("src"),
        components_dir.join("package.json"),
        components_dir.join("vite.lib.config.ts"),
        components_dir.join("src"),
    ];

    let inputs_latest = latest_mtime_for_paths(&input_roots)?;
    let core_dist_latest = latest_mtime_for_path(&core_dir.join("dist"))?;
    let components_dist_latest = latest_mtime_for_path(&components_dir.join("dist"))?;
    let outputs_latest = min_time(core_dist_latest, components_dist_latest);

    match decide_refresh(inputs_latest, outputs_latest) {
        RefreshReason::MissingArtifact => {
            run_ui_build_lib(&ui_dir, verbose)?;
            Ok(PreflightResult::refreshed(
                "at least one package dist artifact is missing",
            ))
        }
        RefreshReason::InputsNewer => {
            run_ui_build_lib(&ui_dir, verbose)?;
            Ok(PreflightResult::refreshed(
                "ui package sources changed since last build",
            ))
        }
        RefreshReason::UpToDate => Ok(PreflightResult::skipped("up-to-date")),
        RefreshReason::NoInputs => Ok(PreflightResult::skipped("no package source inputs found")),
    }
}

fn run_ui_build_lib(ui_dir: &Path, verbose: bool) -> Result<()> {
    if !ui_dir.join("node_modules").is_dir() {
        anyhow::bail!(
            "Preflight requires UI dependencies. Missing {}. Run `npm install` in ui/ first.",
            ui_dir.join("node_modules").display()
        );
    }

    if verbose {
        print_status("Preflight: running npm run build:lib (ui packages)");
    }

    let status = npm_command()
        .args(["run", "build:lib"])
        .current_dir(ui_dir)
        .status()
        .context("Failed to run `npm run build:lib` for preflight")?;

    if !status.success() {
        anyhow::bail!("Preflight UI package artifact refresh failed");
    }

    Ok(())
}

fn refresh_param_codegen_caches_if_needed(
    project_root: &Path,
    verbose: bool,
) -> Result<PreflightResult> {
    let sdk_engine_dir = project_root.join("sdk-template").join("engine");
    let sdk_ui_dir = project_root.join("sdk-template").join("ui");

    if !sdk_engine_dir.exists() || !sdk_ui_dir.exists() {
        return Ok(PreflightResult::skipped(
            "sdk-template engine/ui not detected",
        ));
    }

    let mut sidecar_candidates = sidecar_cache_candidates(&sdk_engine_dir, "wavecraft-params.json");
    sidecar_candidates.extend(sidecar_cache_candidates(
        &sdk_engine_dir,
        "wavecraft-processors.json",
    ));
    let existing_sidecars: Vec<PathBuf> = sidecar_candidates
        .into_iter()
        .filter(|path| path.is_file())
        .collect();

    if existing_sidecars.is_empty() {
        return Ok(PreflightResult::skipped(
            "no existing parameter sidecar cache",
        ));
    }

    let dependency_paths = vec![
        sdk_engine_dir.join("Cargo.toml"),
        sdk_engine_dir.join("build.rs"),
        sdk_engine_dir.join("src"),
        project_root
            .join("engine")
            .join("crates")
            .join("wavecraft-protocol")
            .join("src"),
        project_root
            .join("engine")
            .join("crates")
            .join("wavecraft-bridge")
            .join("src"),
        project_root
            .join("engine")
            .join("crates")
            .join("wavecraft-macros")
            .join("src"),
        project_root.join("cli").join("Cargo.toml"),
        project_root
            .join("cli")
            .join("src")
            .join("commands")
            .join("start.rs"),
        project_root
            .join("cli")
            .join("src")
            .join("commands")
            .join("extract_params.rs"),
        project_root
            .join("cli")
            .join("src")
            .join("project")
            .join("detection.rs"),
        project_root
            .join("cli")
            .join("src")
            .join("project")
            .join("dylib.rs"),
        project_root
            .join("cli")
            .join("src")
            .join("project")
            .join("param_extract.rs"),
        project_root
            .join("cli")
            .join("src")
            .join("project")
            .join("ts_codegen.rs"),
    ];

    let dependencies_latest = latest_mtime_for_paths(&dependency_paths)?;

    let mut removed_sidecars = Vec::new();
    for sidecar in &existing_sidecars {
        let sidecar_mtime = latest_mtime_for_path(sidecar)?;
        match decide_refresh(dependencies_latest, sidecar_mtime) {
            RefreshReason::InputsNewer => {
                if verbose {
                    print_status(&format!(
                        "Preflight: removing stale sidecar cache {}",
                        sidecar.display()
                    ));
                }
                fs::remove_file(sidecar).with_context(|| {
                    format!("Failed to remove stale sidecar cache {}", sidecar.display())
                })?;
                removed_sidecars.push(sidecar.clone());
            }
            RefreshReason::MissingArtifact | RefreshReason::NoInputs | RefreshReason::UpToDate => {}
        }
    }

    if removed_sidecars.is_empty() {
        return Ok(PreflightResult::skipped("up-to-date"));
    }

    let generated_params = sdk_ui_dir
        .join("src")
        .join("generated")
        .join("parameters.ts");
    if generated_params.is_file() {
        if verbose {
            print_status(&format!(
                "Preflight: removing stale generated parameter types {}",
                generated_params.display()
            ));
        }
        fs::remove_file(&generated_params).with_context(|| {
            format!(
                "Failed to remove stale generated parameter types {}",
                generated_params.display()
            )
        })?;
    }

    let generated_processors = sdk_ui_dir
        .join("src")
        .join("generated")
        .join("processors.ts");
    if generated_processors.is_file() {
        if verbose {
            print_status(&format!(
                "Preflight: removing stale generated processor types {}",
                generated_processors.display()
            ));
        }
        fs::remove_file(&generated_processors).with_context(|| {
            format!(
                "Failed to remove stale generated processor types {}",
                generated_processors.display()
            )
        })?;
    }

    Ok(PreflightResult::refreshed(format!(
        "invalidated {} stale sidecar cache file(s)",
        removed_sidecars.len()
    )))
}

fn sidecar_cache_candidates(engine_dir: &Path, file_name: &str) -> Vec<PathBuf> {
    let mut caches = Vec::with_capacity(3);
    caches.push(engine_dir.join("target").join("debug").join(file_name));

    if let Some(parent) = engine_dir.parent() {
        caches.push(parent.join("target").join("debug").join(file_name));

        if let Some(grand_parent) = parent.parent() {
            caches.push(grand_parent.join("target").join("debug").join(file_name));
        }
    }

    caches
}

fn min_time(a: Option<SystemTime>, b: Option<SystemTime>) -> Option<SystemTime> {
    match (a, b) {
        (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
        _ => None,
    }
}

fn latest_mtime_for_paths(paths: &[PathBuf]) -> Result<Option<SystemTime>> {
    let mut latest = None;

    for path in paths {
        let mtime = latest_mtime_for_path(path)?;
        latest = max_time(latest, mtime);
    }

    Ok(latest)
}

fn latest_mtime_for_path(path: &Path) -> Result<Option<SystemTime>> {
    if !path.exists() {
        return Ok(None);
    }

    if path.is_file() {
        return Ok(Some(
            fs::metadata(path)
                .with_context(|| format!("Failed to stat {}", path.display()))?
                .modified()
                .with_context(|| format!("Failed to read mtime for {}", path.display()))?,
        ));
    }

    if !path.is_dir() {
        return Ok(None);
    }

    let mut latest = None;
    collect_latest_mtime(path, &mut latest)?;
    Ok(latest)
}

fn collect_latest_mtime(dir: &Path, latest: &mut Option<SystemTime>) -> Result<()> {
    let entries =
        fs::read_dir(dir).with_context(|| format!("Failed to read directory {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            collect_latest_mtime(&path, latest)?;
            continue;
        }

        let mtime = entry
            .metadata()
            .with_context(|| format!("Failed to stat {}", path.display()))?
            .modified()
            .with_context(|| format!("Failed to read mtime for {}", path.display()))?;

        *latest = max_time(*latest, Some(mtime));
    }

    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                "target" | "node_modules" | "dist" | ".git" | ".idea" | ".vscode"
            )
        })
}

fn max_time(a: Option<SystemTime>, b: Option<SystemTime>) -> Option<SystemTime> {
    match (a, b) {
        (Some(a), Some(b)) => Some(std::cmp::max(a, b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RefreshReason, collect_latest_mtime, decide_refresh, latest_mtime_for_path,
        sidecar_cache_candidates,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use std::time::{Duration, SystemTime};

    #[test]
    fn decide_refresh_identifies_missing_outputs() {
        let now = SystemTime::now();
        assert_eq!(
            decide_refresh(Some(now), None),
            RefreshReason::MissingArtifact
        );
    }

    #[test]
    fn decide_refresh_identifies_newer_inputs() {
        let output = SystemTime::now();
        let input = output + Duration::from_secs(1);
        assert_eq!(
            decide_refresh(Some(input), Some(output)),
            RefreshReason::InputsNewer
        );
    }

    #[test]
    fn decide_refresh_identifies_up_to_date_outputs() {
        let now = SystemTime::now();
        assert_eq!(
            decide_refresh(Some(now), Some(now)),
            RefreshReason::UpToDate
        );
    }

    #[test]
    fn decide_refresh_identifies_missing_inputs() {
        assert_eq!(
            decide_refresh(None, Some(SystemTime::now())),
            RefreshReason::NoInputs
        );
    }

    #[test]
    fn latest_mtime_skips_ignored_directories() {
        let temp = tempfile::tempdir().expect("temp dir");
        let root = temp.path();

        let src_dir = root.join("src");
        let dist_dir = root.join("dist");
        fs::create_dir_all(&src_dir).expect("create src");
        fs::create_dir_all(&dist_dir).expect("create dist");

        let src_file = src_dir.join("input.txt");
        fs::write(&src_file, "src").expect("write src file");

        thread::sleep(Duration::from_millis(20));

        let dist_file = dist_dir.join("output.txt");
        fs::write(&dist_file, "dist").expect("write dist file");

        let latest = latest_mtime_for_path(root)
            .expect("read latest mtime")
            .expect("expected mtime");
        let src_mtime = fs::metadata(&src_file)
            .expect("src metadata")
            .modified()
            .expect("src modified");
        let dist_mtime = fs::metadata(&dist_file)
            .expect("dist metadata")
            .modified()
            .expect("dist modified");

        assert!(
            latest >= src_mtime,
            "latest mtime should include source file"
        );
        assert!(
            latest < dist_mtime,
            "latest mtime should ignore dist directory files"
        );

        // Also smoke-test direct recursion helper.
        let mut direct_latest = None;
        collect_latest_mtime(root, &mut direct_latest).expect("collect latest");
        assert_eq!(direct_latest, Some(latest));
    }

    #[test]
    fn sidecar_cache_candidates_cover_all_expected_locations() {
        let engine_dir = PathBuf::from("/tmp/wavecraft/sdk-template/engine");
        let candidates = sidecar_cache_candidates(&engine_dir, "wavecraft-params.json");

        assert_eq!(candidates.len(), 3);
        assert!(candidates[0].ends_with("sdk-template/engine/target/debug/wavecraft-params.json"));
        assert!(candidates[1].ends_with("sdk-template/target/debug/wavecraft-params.json"));
        assert!(candidates[2].ends_with("target/debug/wavecraft-params.json"));
    }
}
