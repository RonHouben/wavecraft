//! CD dry-run command - Simulates continuous deployment publish steps locally.
//!
//! This command mirrors the package-publish validation portions of
//! `.github/workflows/continuous-deploy.yml` by:
//! 1. Detecting changed package areas from git diff
//! 2. Reporting local versions for changed packages
//! 3. Running publish dry-runs only for changed targets

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::validate_cli_deps::{self, ValidateCliDepsConfig};
use xtask::npm_command;
use xtask::output::*;
use xtask::paths;

/// CD dry-run configuration.
#[derive(Debug, Clone)]
pub struct CdDryRunConfig {
    /// Show verbose output.
    pub verbose: bool,
    /// Base git reference used for change detection (e.g., `main`).
    pub base_ref: String,
}

impl Default for CdDryRunConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            base_ref: "main".to_string(),
        }
    }
}

/// Change-set flags aligned with continuous-deploy.yml path filters.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChangeSet {
    /// CLI or sdk-template related files changed.
    pub cli: bool,
    /// Engine publishable crates related files changed.
    pub engine: bool,
    /// Dev server files changed.
    pub dev_server: bool,
    /// `@wavecraft/core` files changed.
    pub npm_core: bool,
    /// `@wavecraft/components` files changed.
    pub npm_components: bool,
}

impl ChangeSet {
    fn all() -> Self {
        Self {
            cli: true,
            engine: true,
            dev_server: true,
            npm_core: true,
            npm_components: true,
        }
    }
}

#[derive(Debug)]
struct RustCrate {
    name: String,
    manifest_path: PathBuf,
}

#[derive(Default)]
struct PhaseSummary {
    passed: Vec<String>,
    failed: Vec<String>,
    skipped: Vec<String>,
    skipped_labels: Vec<String>,
}

impl PhaseSummary {
    fn pass(&mut self, label: impl Into<String>) {
        self.passed.push(label.into());
    }

    fn fail(&mut self, label: impl Into<String>, error: impl Into<String>) {
        self.failed
            .push(format!("{} ({})", label.into(), error.into()));
    }

    fn skip(&mut self, label: impl Into<String>, reason: impl Into<String>) {
        let label = label.into();
        self.skipped.push(format!("{} ({})", label, reason.into()));
        self.skipped_labels.push(label);
    }

    fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }
}

/// Run the CD dry-run phase.
pub fn run(config: CdDryRunConfig) -> Result<()> {
    let project_root = paths::project_root()?;

    let (changes, used_fallback, fallback_error) =
        resolve_changes(&project_root, &config.base_ref, detect_changes);

    if let Some(err) = fallback_error {
        print_warning(&format!(
            "⚠️ Change detection failed against '{}': {}",
            config.base_ref, err
        ));
        print_warning("⚠️ Falling back to running dry-runs for all publish targets.");
    }

    print_status("Phase 5a: Change Detection");
    print_change_set(changes, used_fallback);
    println!();

    print_status("Phase 5b: Version Report");
    print_version_report(&project_root, changes)?;
    println!();

    let mut summary = PhaseSummary::default();

    print_status("Phase 5c: validate-cli-deps");
    if changes.cli || changes.engine {
        match validate_cli_deps::run(ValidateCliDepsConfig {
            verbose: config.verbose,
            check_registry: false,
        }) {
            Ok(()) => summary.pass("validate-cli-deps"),
            Err(err) => {
                print_error(&format!("validate-cli-deps failed: {:#}", err));
                summary.fail("validate-cli-deps", err.to_string());
            }
        }
    } else {
        summary.skip("validate-cli-deps", "no CLI/engine changes");
    }
    println!();

    print_status("Phase 5d: Rust Publish Dry-Run");
    run_rust_dry_runs(&project_root, &config, changes, &mut summary);
    println!();

    print_status("Phase 5e: NPM Publish Dry-Run");
    run_npm_dry_runs(&project_root, &config, changes, &mut summary);
    println!();

    print_status("Phase 5f: Summary");
    print_phase_summary(&summary, used_fallback);

    if summary.has_failures() {
        anyhow::bail!("CD dry-run failed");
    }

    Ok(())
}

fn resolve_changes<F>(
    project_root: &Path,
    base_ref: &str,
    detect_fn: F,
) -> (ChangeSet, bool, Option<String>)
where
    F: FnOnce(&Path, &str) -> Result<(ChangeSet, bool)>,
{
    match detect_fn(project_root, base_ref) {
        Ok((changes, used_fallback)) => (changes, used_fallback, None),
        Err(err) => (ChangeSet::all(), true, Some(err.to_string())),
    }
}

fn detect_changes(project_root: &Path, base_ref: &str) -> Result<(ChangeSet, bool)> {
    let output = Command::new("git")
        .arg("diff")
        .arg(base_ref)
        .arg("--name-only")
        .current_dir(project_root)
        .output()
        .with_context(|| format!("Failed to execute git diff {base_ref} --name-only"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "git diff command failed with status {:?}: {}",
            output.status.code(),
            stderr.trim()
        );
    }

    let changed_stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let changed_files: Vec<String> = changed_stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect();

    let mut changes = ChangeSet::default();
    for path in &changed_files {
        if is_cli_change(path) {
            changes.cli = true;
        }
        if is_engine_change(path) {
            changes.engine = true;
        }
        if is_dev_server_change(path) {
            changes.dev_server = true;
        }
        if is_npm_core_change(path) {
            changes.npm_core = true;
        }
        if is_npm_components_change(path) {
            changes.npm_components = true;
        }
    }

    Ok((changes, false))
}

fn is_cli_change(path: &str) -> bool {
    path.starts_with("cli/src/") || is_cli_root_file(path) || path.starts_with("sdk-template/")
}

fn is_cli_root_file(path: &str) -> bool {
    let Some(relative) = path.strip_prefix("cli/") else {
        return false;
    };

    !relative.is_empty() && !relative.contains('/')
}

fn is_engine_change(path: &str) -> bool {
    path == "engine/Cargo.toml"
        || path.starts_with("engine/crates/wavecraft-core/")
        || path.starts_with("engine/crates/wavecraft-dsp/")
        || path.starts_with("engine/crates/wavecraft-protocol/")
        || path.starts_with("engine/crates/wavecraft-bridge/")
        || path.starts_with("engine/crates/wavecraft-metering/")
        || path.starts_with("engine/crates/wavecraft-macros/")
}

fn is_dev_server_change(path: &str) -> bool {
    path.starts_with("dev-server/")
}

fn is_npm_core_change(path: &str) -> bool {
    path.starts_with("ui/packages/core/")
}

fn is_npm_components_change(path: &str) -> bool {
    path.starts_with("ui/packages/components/")
}

fn print_change_set(changes: ChangeSet, used_fallback: bool) {
    if used_fallback {
        print_warning("Using fallback changeset (all targets enabled)");
    }
    print_success_item(&format!("cli: {}", yes_no(changes.cli)));
    print_success_item(&format!("engine: {}", yes_no(changes.engine)));
    print_success_item(&format!("dev-server: {}", yes_no(changes.dev_server)));
    print_success_item(&format!("npm-core: {}", yes_no(changes.npm_core)));
    print_success_item(&format!(
        "npm-components: {}",
        yes_no(changes.npm_components)
    ));
}

fn yes_no(flag: bool) -> &'static str {
    if flag { "yes" } else { "no" }
}

fn print_version_report(project_root: &Path, changes: ChangeSet) -> Result<()> {
    if changes.cli {
        print_info(&format!(
            "CLI (wavecraft): {}",
            read_toml_package_version(&project_root.join("cli/Cargo.toml"))?
        ));
    }

    if changes.engine {
        print_info(&format!(
            "Engine workspace: {}",
            read_engine_workspace_version(&project_root.join("engine/Cargo.toml"))?
        ));
    }

    if changes.dev_server {
        print_info(&format!(
            "dev-server (wavecraft-dev-server): {}",
            read_toml_package_version(&project_root.join("dev-server/Cargo.toml"))?
        ));
    }

    if changes.npm_core {
        print_info(&format!(
            "@wavecraft/core: {}",
            read_package_json_version(&project_root.join("ui/packages/core/package.json"))?
        ));
    }

    if changes.npm_components {
        print_info(&format!(
            "@wavecraft/components: {}",
            read_package_json_version(&project_root.join("ui/packages/components/package.json"))?
        ));
    }

    if !changes.cli
        && !changes.engine
        && !changes.dev_server
        && !changes.npm_core
        && !changes.npm_components
    {
        print_skip("No package-related changes detected.");
    }

    Ok(())
}

fn read_toml_package_version(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read TOML file: {}", path.display()))?;
    let toml: toml::Value = content
        .parse()
        .with_context(|| format!("Failed to parse TOML file: {}", path.display()))?;

    toml.get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow::anyhow!("No package.version found in {}", path.display()))
}

fn read_engine_workspace_version(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read TOML file: {}", path.display()))?;
    let toml: toml::Value = content
        .parse()
        .with_context(|| format!("Failed to parse TOML file: {}", path.display()))?;

    toml.get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|pkg| pkg.get("version"))
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow::anyhow!("No workspace.package.version found in {}", path.display()))
}

fn read_package_json_version(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read package.json: {}", path.display()))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse package.json: {}", path.display()))?;

    json.get("version")
        .and_then(|v| v.as_str())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow::anyhow!("No version found in {}", path.display()))
}

fn run_rust_dry_runs(
    project_root: &Path,
    config: &CdDryRunConfig,
    changes: ChangeSet,
    summary: &mut PhaseSummary,
) {
    if changes.engine {
        print_info(
            "Note: crates.io 'already exists' and 'aborting upload due to dry run' warnings are expected in this phase.",
        );

        if cargo_workspaces_installed(project_root) {
            match run_cargo_ws_publish_dry_run(project_root, config.verbose) {
                Ok(()) => summary.pass("engine/workspace"),
                Err(err) => {
                    print_error(&format!("engine workspace dry-run failed: {:#}", err));
                    summary.fail("engine/workspace", err.to_string());
                }
            }
        } else {
            print_warning(
                "cargo-workspaces not found; using per-crate cargo publish --dry-run for engine crates.",
            );

            match discover_publishable_engine_crates(project_root) {
                Ok(crates) if crates.is_empty() => {
                    summary.skip("engine crates", "no publishable crates found");
                }
                Ok(crates) => {
                    for krate in crates {
                        let label = format!("engine/{}", krate.name);
                        match run_cargo_publish_dry_run_for_manifest(
                            &krate.manifest_path,
                            project_root,
                            config.verbose,
                        ) {
                            Ok(()) => summary.pass(label),
                            Err(err) => {
                                print_error(&format!("{} failed: {:#}", krate.name, err));
                                summary.fail(format!("engine/{}", krate.name), err.to_string());
                            }
                        }
                    }
                }
                Err(err) => {
                    print_error(&format!("Failed to discover engine crates: {:#}", err));
                    summary.fail("engine crate discovery", err.to_string());
                }
            }
        }
    } else {
        summary.skip("engine crates", "no changes");
    }

    if changes.dev_server {
        if let Some(reason) = dev_server_skip_reason(changes) {
            summary.skip("dev-server", reason);
        } else {
            let dev_server_dir = project_root.join("dev-server");
            match run_cargo_publish_dry_run_in_dir(&dev_server_dir, config.verbose) {
                Ok(()) => summary.pass("dev-server"),
                Err(err) => {
                    print_error(&format!("dev-server failed: {:#}", err));
                    summary.fail("dev-server", err.to_string());
                }
            }
        }
    } else {
        summary.skip("dev-server", "no changes");
    }

    if changes.cli {
        summary.skip(
            "cli",
            "expected: include_dir! references files outside crate — not publishable via dry-run",
        );
    } else {
        summary.skip("cli", "no changes");
    }
}

fn dev_server_skip_reason(changes: ChangeSet) -> Option<&'static str> {
    if changes.dev_server && changes.engine {
        Some(
            "engine changes detected; local dry-run cannot replay CI version sync + crates.io propagation",
        )
    } else {
        None
    }
}

fn run_npm_dry_runs(
    project_root: &Path,
    config: &CdDryRunConfig,
    changes: ChangeSet,
    summary: &mut PhaseSummary,
) {
    if changes.npm_core {
        let core_dir = project_root.join("ui/packages/core");
        match run_npm_pack_dry_run(&core_dir, config.verbose) {
            Ok(()) => summary.pass("@wavecraft/core"),
            Err(err) => {
                print_error(&format!("@wavecraft/core failed: {:#}", err));
                summary.fail("@wavecraft/core", err.to_string());
            }
        }
    } else {
        summary.skip("@wavecraft/core", "no changes");
    }

    if changes.npm_components {
        let components_dir = project_root.join("ui/packages/components");
        match run_npm_pack_dry_run(&components_dir, config.verbose) {
            Ok(()) => summary.pass("@wavecraft/components"),
            Err(err) => {
                print_error(&format!("@wavecraft/components failed: {:#}", err));
                summary.fail("@wavecraft/components", err.to_string());
            }
        }
    } else {
        summary.skip("@wavecraft/components", "no changes");
    }
}

fn cargo_workspaces_installed(project_root: &Path) -> bool {
    Command::new("cargo")
        .args(["ws", "--version"])
        .current_dir(project_root)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn discover_publishable_engine_crates(project_root: &Path) -> Result<Vec<RustCrate>> {
    let crates_root = project_root.join("engine/crates");
    let mut crates = Vec::new();

    for entry in fs::read_dir(&crates_root)
        .with_context(|| format!("Failed to read {}", crates_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let Some(crate_dir_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !crate_dir_name.starts_with("wavecraft-") {
            continue;
        }

        let manifest_path = path.join("Cargo.toml");
        if !manifest_path.exists() {
            continue;
        }

        if !is_publishable_crate(&manifest_path)? {
            continue;
        }

        let crate_name = read_crate_name(&manifest_path)?;
        crates.push(RustCrate {
            name: crate_name,
            manifest_path,
        });
    }

    crates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(crates)
}

fn is_publishable_crate(manifest_path: &Path) -> Result<bool> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;
    let toml: toml::Value = content
        .parse()
        .with_context(|| format!("Failed to parse {}", manifest_path.display()))?;

    let publish = toml
        .get("package")
        .and_then(|pkg| pkg.get("publish"))
        .and_then(|value| value.as_bool());

    Ok(publish != Some(false))
}

fn read_crate_name(manifest_path: &Path) -> Result<String> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;
    let toml: toml::Value = content
        .parse()
        .with_context(|| format!("Failed to parse {}", manifest_path.display()))?;

    toml.get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow::anyhow!("No package.name in {}", manifest_path.display()))
}

fn run_cargo_publish_dry_run_for_manifest(
    manifest_path: &Path,
    project_root: &Path,
    verbose: bool,
) -> Result<()> {
    let mut command = Command::new("cargo");
    command
        .arg("publish")
        .arg("--dry-run")
        .arg("--allow-dirty")
        .arg("--manifest-path")
        .arg(manifest_path)
        .current_dir(project_root);

    if verbose {
        print_info(&format!(
            "Running: cargo publish --dry-run --allow-dirty --manifest-path {}",
            manifest_path.display()
        ));
    }

    let status = command
        .status()
        .context("Failed to execute cargo publish")?;
    if !status.success() {
        anyhow::bail!(
            "cargo publish --dry-run failed for {}",
            manifest_path.display()
        );
    }

    Ok(())
}

fn run_cargo_ws_publish_dry_run(project_root: &Path, verbose: bool) -> Result<()> {
    let engine_dir = project_root.join("engine");

    let mut command = Command::new("cargo");
    command
        .arg("ws")
        .arg("publish")
        .arg("--yes")
        .arg("--no-git-push")
        .arg("--allow-branch")
        .arg("main")
        .arg("--from-git")
        .arg("--dry-run")
        .current_dir(&engine_dir);

    if verbose {
        print_info(&format!(
            "Running in {}: cargo ws publish --yes --no-git-push --allow-branch main --from-git --dry-run",
            engine_dir.display()
        ));
    }

    let status = command
        .status()
        .context("Failed to execute cargo ws publish")?;
    if !status.success() {
        anyhow::bail!(
            "cargo ws publish --dry-run failed in {}",
            engine_dir.display()
        );
    }

    Ok(())
}

fn run_cargo_publish_dry_run_in_dir(dir: &Path, verbose: bool) -> Result<()> {
    let mut command = Command::new("cargo");
    command
        .arg("publish")
        .arg("--dry-run")
        .arg("--allow-dirty")
        .current_dir(dir);

    if verbose {
        print_info(&format!(
            "Running in {}: cargo publish --dry-run --allow-dirty",
            dir.display()
        ));
    }

    let status = command
        .status()
        .context("Failed to execute cargo publish")?;
    if !status.success() {
        anyhow::bail!("cargo publish --dry-run failed in {}", dir.display());
    }

    Ok(())
}

fn run_npm_pack_dry_run(dir: &Path, verbose: bool) -> Result<()> {
    let mut build_command = npm_command();
    build_command.arg("run").arg("build:lib").current_dir(dir);

    if verbose {
        print_info(&format!("Running in {}: npm run build:lib", dir.display()));
    }

    let build_status = build_command
        .status()
        .context("Failed to run npm build command")?;
    if !build_status.success() {
        anyhow::bail!("npm run build:lib failed in {}", dir.display());
    }

    let mut pack_command = npm_command();
    pack_command.arg("pack").arg("--dry-run").current_dir(dir);

    if verbose {
        print_info(&format!("Running in {}: npm pack --dry-run", dir.display()));
    }

    let pack_status = pack_command
        .status()
        .context("Failed to run npm pack command")?;
    if !pack_status.success() {
        anyhow::bail!("npm pack --dry-run failed in {}", dir.display());
    }

    Ok(())
}

fn print_phase_summary(summary: &PhaseSummary, used_fallback: bool) {
    for item in &summary.passed {
        print_success_item(&format!("{}: PASSED", item));
    }

    for item in &summary.skipped {
        println!("  ⊘ {}: SKIPPED", item);
    }

    for item in &summary.failed {
        print_error(&format!("  ✗ {}: FAILED", item));
    }

    for note in confidence_boundary_notes(summary, used_fallback) {
        print_warning(&note);
    }
}

fn confidence_boundary_notes(summary: &PhaseSummary, used_fallback: bool) -> Vec<String> {
    let mut notes = Vec::new();

    if used_fallback {
        notes.push(
            "Confidence boundary: change detection fell back to all targets; local pass is conservative and not a full CI-equivalent signal.".to_string(),
        );
    }

    if summary.skipped_labels.iter().any(|label| label == "cli") {
        notes.push(
            "Confidence boundary: CLI publish dry-run is intentionally skipped locally; rely on CI + release workflow checks for full CLI publish confidence.".to_string(),
        );
    }

    if summary
        .skipped_labels
        .iter()
        .any(|label| label == "dev-server")
    {
        notes.push(
            "Confidence boundary: dev-server publish dry-run may be skipped when engine changed; local pass does not cover full cross-job CI publish sequencing.".to_string(),
        );
    }

    notes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_path_detection() {
        assert!(is_cli_change("cli/src/main.rs"));
        assert!(is_cli_change("cli/build.rs"));
        assert!(is_cli_change("cli/Cargo.toml"));
        assert!(is_cli_change("cli/README.md"));
        assert!(is_cli_change("sdk-template/engine/Cargo.toml.template"));
        assert!(!is_cli_change("engine/Cargo.toml"));
    }

    #[test]
    fn test_engine_path_detection() {
        assert!(is_engine_change("engine/Cargo.toml"));
        assert!(is_engine_change("engine/crates/wavecraft-core/src/lib.rs"));
        assert!(is_engine_change(
            "engine/crates/wavecraft-protocol/src/messages.rs"
        ));
        assert!(!is_engine_change(
            "engine/crates/wavecraft-nih_plug/src/lib.rs"
        ));
    }

    #[test]
    fn test_dev_server_path_detection() {
        assert!(is_dev_server_change("dev-server/src/lib.rs"));
        assert!(!is_dev_server_change("cli/src/main.rs"));
    }

    #[test]
    fn test_npm_path_detection() {
        assert!(is_npm_core_change("ui/packages/core/src/index.ts"));
        assert!(is_npm_components_change(
            "ui/packages/components/src/index.tsx"
        ));
        assert!(!is_npm_core_change("ui/src/main.ts"));
        assert!(!is_npm_components_change("ui/src/App.tsx"));
    }

    #[test]
    fn test_default_config() {
        let config = CdDryRunConfig::default();
        assert!(!config.verbose);
        assert_eq!(config.base_ref, "main");
    }

    #[test]
    fn test_dev_server_skip_reason_when_engine_and_dev_server_changed() {
        let changes = ChangeSet {
            engine: true,
            dev_server: true,
            ..Default::default()
        };

        assert!(dev_server_skip_reason(changes).is_some());
    }

    #[test]
    fn test_dev_server_skip_reason_none_when_only_dev_server_changed() {
        let changes = ChangeSet {
            engine: false,
            dev_server: true,
            ..Default::default()
        };

        assert!(dev_server_skip_reason(changes).is_none());
    }

    #[test]
    fn test_resolve_changes_falls_back_on_detection_error() {
        let root = Path::new(".");
        let (changes, used_fallback, fallback_error) =
            resolve_changes(root, "main", |_project_root, _base_ref| {
                anyhow::bail!("simulated git diff failure")
            });

        assert!(used_fallback);
        assert!(fallback_error.is_some());
        assert!(changes.cli);
        assert!(changes.engine);
        assert!(changes.dev_server);
        assert!(changes.npm_core);
        assert!(changes.npm_components);
    }

    #[test]
    fn test_confidence_boundary_notes_include_fallback_and_skips() {
        let mut summary = PhaseSummary::default();
        summary.skip("cli", "intentionally skipped");
        summary.skip("dev-server", "engine changed");

        let notes = confidence_boundary_notes(&summary, true);

        assert_eq!(notes.len(), 3);
        assert!(notes.iter().any(|n| n.contains("fell back to all targets")));
        assert!(notes.iter().any(|n| n.contains("CLI publish dry-run")));
        assert!(
            notes
                .iter()
                .any(|n| n.contains("dev-server publish dry-run"))
        );
    }
}
