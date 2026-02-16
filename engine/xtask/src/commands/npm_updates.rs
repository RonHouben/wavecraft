//! NPM update check command.
//!
//! Checks for outdated npm packages in:
//! - ui/ workspace root (workspace-aware)
//! - sdk-template/ui/ standalone project
//!
//! and can optionally perform upgrades across all npm areas.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use xtask::output::*;
use xtask::{command_exists, npm_command, paths};

/// Configuration for npm update checks.
#[derive(Debug, Clone)]
pub struct NpmUpdatesConfig {
    /// When true, return a non-zero exit code if updates are found.
    pub strict: bool,
    /// When true, run `npm update` across all npm areas.
    pub upgrade: bool,
    /// Show verbose command output.
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutdatedResult {
    UpToDate,
    Outdated,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NpmArea {
    UiWorkspace,
    UiComponents,
    UiCore,
    SdkTemplateUi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AreaSnapshot {
    files: Vec<FileSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileSnapshot {
    path: PathBuf,
    contents: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UpgradeAreaSuccess {
    updates_detected: bool,
    install_ran: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckModeSummary {
    NoUpdates,
    UpdatesAllowed,
}

/// Run npm update checks across monorepo npm projects.
pub fn run(config: NpmUpdatesConfig) -> Result<()> {
    if !command_exists("npm") {
        anyhow::bail!("npm is not installed or not on PATH");
    }

    if config.upgrade {
        run_upgrade_mode(config)
    } else {
        run_check_mode(config)
    }
}

fn run_check_mode(config: NpmUpdatesConfig) -> Result<()> {
    print_header("NPM Package Update Check");

    let ui_dir = paths::ui_dir()?;
    let sdk_template_ui_dir = paths::sdk_template_ui_dir()?;

    let ui_result = run_area(
        "Area 1/2: UI workspace (ui/)",
        &ui_dir,
        outdated_args_for_area(NpmArea::UiWorkspace),
        config.verbose,
    )
    .context("Failed while checking updates in ui/")?;

    let sdk_template_result = run_area(
        "Area 2/2: SDK template app (sdk-template/ui/)",
        &sdk_template_ui_dir,
        outdated_args_for_area(NpmArea::SdkTemplateUi),
        config.verbose,
    )
    .context("Failed while checking updates in sdk-template/ui/")?;

    println!();
    print_status("Summary:");

    print_summary_item("ui/", ui_result);
    print_summary_item("sdk-template/ui/", sdk_template_result);

    match finalize_check_results(ui_result, sdk_template_result, config.strict)? {
        CheckModeSummary::NoUpdates => {
            println!();
            print_success("No outdated npm packages found.");
            Ok(())
        }
        CheckModeSummary::UpdatesAllowed => {
            println!();
            print_warning(
                "Outdated packages found (informational mode). Returning success due to --allow-updates.",
            );
            Ok(())
        }
    }
}

fn run_upgrade_mode(config: NpmUpdatesConfig) -> Result<()> {
    print_header("NPM Dependency Upgrade");

    if !config.strict {
        print_info(
            "Ignoring --allow-updates in --upgrade mode (upgrade mode always succeeds/fails based on npm update results).",
        );
        println!();
    }

    let ui_dir = paths::ui_dir()?;
    let sdk_template_ui_dir = paths::sdk_template_ui_dir()?;

    let upgrade_areas = [
        (
            "Area 1/4: UI workspace root (ui/)",
            "ui/",
            ui_dir.clone(),
            NpmArea::UiWorkspace,
        ),
        (
            "Area 2/4: UI components workspace (ui/packages/components/)",
            "ui/packages/components/",
            ui_dir.join("packages/components"),
            NpmArea::UiComponents,
        ),
        (
            "Area 3/4: UI core workspace (ui/packages/core/)",
            "ui/packages/core/",
            ui_dir.join("packages/core"),
            NpmArea::UiCore,
        ),
        (
            "Area 4/4: SDK template app (sdk-template/ui/)",
            "sdk-template/ui/",
            sdk_template_ui_dir,
            NpmArea::SdkTemplateUi,
        ),
    ];

    let mut results: Vec<(&str, Result<UpgradeAreaSuccess>)> = Vec::new();
    for (header, summary_label, dir, area) in upgrade_areas {
        results.push((
            summary_label,
            run_upgrade_area(header, &dir, area, config.verbose),
        ));
    }

    println!();
    print_status("Summary:");
    for (area, result) in &results {
        print_upgrade_summary_item(area, result);
    }

    if results.iter().any(|(_, result)| result.is_err()) {
        anyhow::bail!("One or more npm upgrade commands failed");
    }

    println!();
    print_success("Dependency upgrades completed successfully in all npm areas.");
    Ok(())
}

fn run_upgrade_area(
    header: &str,
    dir: &Path,
    area: NpmArea,
    verbose: bool,
) -> Result<UpgradeAreaSuccess> {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status(header);
    println!("  Directory: {}", dir.display());
    println!("  Action: Run npm update, detect changes, then conditionally run npm install");

    let before = capture_area_snapshot(area)?;

    let update_args = upgrade_args_for_area(area);

    if verbose {
        println!("  Running: npm {}", update_args.join(" "));
    }

    let status = npm_command()
        .args(update_args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("Failed to run npm update in {}", dir.display()))?;

    if !status.success() {
        print_error_item("npm update command failed");
        if let Some(code) = status.code() {
            print_error(&format!("  Exit code: {}", code));
        }
        anyhow::bail!("npm update failed in {}", dir.display())
    }

    print_success_item("npm update command completed successfully");

    let after = capture_area_snapshot(area)?;
    let updates_detected = updates_detected_for_area(&before, &after);

    if updates_detected {
        print_success_item("Updates detected: yes");
    } else {
        print_success_item("Updates detected: no");
    }

    let install_ran = if updates_detected {
        let install_args = ["install"];
        if verbose {
            println!("  Running: npm {}", install_args.join(" "));
        }

        let install_status = npm_command()
            .args(install_args)
            .current_dir(dir)
            .status()
            .with_context(|| format!("Failed to run npm install in {}", dir.display()))?;

        if !install_status.success() {
            print_error_item("npm install command failed");
            if let Some(code) = install_status.code() {
                print_error(&format!("  Exit code: {}", code));
            }
            anyhow::bail!("npm install failed in {}", dir.display())
        }

        print_success_item("npm install run: yes");
        true
    } else {
        print_skip("  - npm install run: no (skipped, no updates detected)");
        false
    };

    Ok(UpgradeAreaSuccess {
        updates_detected,
        install_ran,
    })
}

fn capture_area_snapshot(area: NpmArea) -> Result<AreaSnapshot> {
    let files = tracked_files_for_area(area)?
        .into_iter()
        .map(|path| {
            let contents = read_file_if_exists(&path)?;
            Ok(FileSnapshot { path, contents })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(AreaSnapshot { files })
}

fn tracked_files_for_area(area: NpmArea) -> Result<Vec<PathBuf>> {
    let project_root = paths::project_root()?;
    let ui_dir = project_root.join("ui");
    let sdk_template_ui_dir = project_root.join("sdk-template/ui");

    let files = match area {
        NpmArea::UiWorkspace => vec![
            ui_dir.join("package-lock.json"),
            ui_dir.join("package.json"),
        ],
        NpmArea::UiComponents => vec![
            ui_dir.join("package-lock.json"),
            ui_dir.join("packages/components/package.json"),
        ],
        NpmArea::UiCore => vec![
            ui_dir.join("package-lock.json"),
            ui_dir.join("packages/core/package.json"),
        ],
        NpmArea::SdkTemplateUi => vec![
            sdk_template_ui_dir.join("package-lock.json"),
            sdk_template_ui_dir.join("package.json"),
        ],
    };

    Ok(files)
}

fn read_file_if_exists(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("Failed to read {}", path.display())),
    }
}

fn updates_detected_for_area(before: &AreaSnapshot, after: &AreaSnapshot) -> bool {
    before != after
}

fn print_upgrade_summary_item(area: &str, result: &Result<UpgradeAreaSuccess>) {
    match result {
        Ok(success) => {
            let install_status = if success.install_ran {
                "install ran"
            } else {
                "install skipped"
            };
            let updates_status = if success.updates_detected {
                "updates detected"
            } else {
                "no updates detected"
            };
            print_success_item(&format!(
                "{}: update ok, {}, {}",
                area, updates_status, install_status
            ));
        }
        Err(_) => print_error(&format!("  ✗ {}: upgrade failed", area)),
    }
}

fn outdated_args_for_area(area: NpmArea) -> &'static [&'static str] {
    match area {
        NpmArea::UiWorkspace => &["outdated", "--workspaces", "--include-workspace-root"],
        NpmArea::UiComponents => &["outdated", "--workspace", "@wavecraft/components"],
        NpmArea::UiCore => &["outdated", "--workspace", "@wavecraft/core"],
        NpmArea::SdkTemplateUi => &["outdated"],
    }
}

fn upgrade_args_for_area(area: NpmArea) -> &'static [&'static str] {
    match area {
        NpmArea::UiWorkspace => &["update", "--workspaces", "--include-workspace-root"],
        NpmArea::UiComponents => &["update", "--workspace", "@wavecraft/components"],
        NpmArea::UiCore => &["update", "--workspace", "@wavecraft/core"],
        NpmArea::SdkTemplateUi => &["update"],
    }
}

fn finalize_check_results(
    ui_result: OutdatedResult,
    sdk_template_result: OutdatedResult,
    strict: bool,
) -> Result<CheckModeSummary> {
    if ui_result == OutdatedResult::Failed || sdk_template_result == OutdatedResult::Failed {
        anyhow::bail!("One or more npm outdated checks failed to run");
    }

    let updates_found =
        ui_result == OutdatedResult::Outdated || sdk_template_result == OutdatedResult::Outdated;

    if updates_found && strict {
        anyhow::bail!(
            "Outdated npm packages found. Re-run with '--allow-updates' for informational mode."
        );
    }

    if updates_found {
        return Ok(CheckModeSummary::UpdatesAllowed);
    }

    Ok(CheckModeSummary::NoUpdates)
}

fn run_area(header: &str, dir: &Path, args: &[&str], verbose: bool) -> Result<OutdatedResult> {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status(header);
    println!("  Directory: {}", dir.display());

    if verbose {
        println!("  Running: npm {}", args.join(" "));
    }

    let status = npm_command()
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("Failed to run npm outdated in {}", dir.display()))?;

    let result = classify_outdated_status(status.code());

    match result {
        OutdatedResult::UpToDate => print_success_item("Up to date"),
        OutdatedResult::Outdated => print_warning("Outdated packages detected"),
        OutdatedResult::Failed => {
            print_error_item("npm outdated command failed");
            if let Some(code) = status.code() {
                print_error(&format!("  Exit code: {}", code));
            }
        }
    }

    Ok(result)
}

fn print_summary_item(area: &str, result: OutdatedResult) {
    match result {
        OutdatedResult::UpToDate => {
            print_success_item(&format!("{}: up to date", area));
        }
        OutdatedResult::Outdated => {
            print_warning(&format!("  ! {}: updates available", area));
        }
        OutdatedResult::Failed => {
            print_error(&format!("  ✗ {}: check failed", area));
        }
    }
}

fn classify_outdated_status(code: Option<i32>) -> OutdatedResult {
    match code {
        Some(0) => OutdatedResult::UpToDate,
        Some(1) => OutdatedResult::Outdated,
        _ => OutdatedResult::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AreaSnapshot, CheckModeSummary, FileSnapshot, NpmArea, OutdatedResult,
        classify_outdated_status, finalize_check_results, outdated_args_for_area,
        updates_detected_for_area, upgrade_args_for_area,
    };
    use std::path::PathBuf;

    #[test]
    fn classify_exit_code_zero_is_up_to_date() {
        assert_eq!(classify_outdated_status(Some(0)), OutdatedResult::UpToDate);
    }

    #[test]
    fn classify_exit_code_one_is_outdated() {
        assert_eq!(classify_outdated_status(Some(1)), OutdatedResult::Outdated);
    }

    #[test]
    fn classify_other_exit_codes_as_failed() {
        assert_eq!(classify_outdated_status(Some(2)), OutdatedResult::Failed);
        assert_eq!(classify_outdated_status(None), OutdatedResult::Failed);
    }

    #[test]
    fn check_mode_fails_when_any_area_fails() {
        let result = finalize_check_results(OutdatedResult::UpToDate, OutdatedResult::Failed, true);
        assert!(result.is_err());
    }

    #[test]
    fn check_mode_fails_when_outdated_and_strict() {
        let result =
            finalize_check_results(OutdatedResult::Outdated, OutdatedResult::UpToDate, true);
        assert!(result.is_err());
    }

    #[test]
    fn check_mode_allows_outdated_when_not_strict() {
        let result =
            finalize_check_results(OutdatedResult::Outdated, OutdatedResult::UpToDate, false)
                .expect("non-strict check mode should allow outdated dependencies");
        assert_eq!(result, CheckModeSummary::UpdatesAllowed);
    }

    #[test]
    fn check_mode_reports_no_updates_when_all_up_to_date() {
        let result =
            finalize_check_results(OutdatedResult::UpToDate, OutdatedResult::UpToDate, true)
                .expect("up-to-date dependencies should succeed");
        assert_eq!(result, CheckModeSummary::NoUpdates);
    }

    #[test]
    fn ui_workspace_uses_workspace_aware_outdated_args() {
        assert_eq!(
            outdated_args_for_area(NpmArea::UiWorkspace),
            &["outdated", "--workspaces", "--include-workspace-root"]
        );
    }

    #[test]
    fn sdk_template_uses_standard_outdated_args() {
        assert_eq!(
            outdated_args_for_area(NpmArea::SdkTemplateUi),
            &["outdated"]
        );
    }

    #[test]
    fn ui_workspace_uses_workspace_aware_upgrade_args() {
        assert_eq!(
            upgrade_args_for_area(NpmArea::UiWorkspace),
            &["update", "--workspaces", "--include-workspace-root"]
        );
    }

    #[test]
    fn ui_components_uses_workspace_scoped_upgrade_args() {
        assert_eq!(
            upgrade_args_for_area(NpmArea::UiComponents),
            &["update", "--workspace", "@wavecraft/components"]
        );
    }

    #[test]
    fn ui_core_uses_workspace_scoped_upgrade_args() {
        assert_eq!(
            upgrade_args_for_area(NpmArea::UiCore),
            &["update", "--workspace", "@wavecraft/core"]
        );
    }

    #[test]
    fn sdk_template_uses_standard_upgrade_args() {
        assert_eq!(upgrade_args_for_area(NpmArea::SdkTemplateUi), &["update"]);
    }

    #[test]
    fn update_detection_reports_no_changes_for_identical_snapshots() {
        let snapshot = AreaSnapshot {
            files: vec![FileSnapshot {
                path: PathBuf::from("ui/package-lock.json"),
                contents: Some(vec![1, 2, 3]),
            }],
        };

        assert!(!updates_detected_for_area(&snapshot, &snapshot));
    }

    #[test]
    fn update_detection_reports_changes_when_contents_differ() {
        let before = AreaSnapshot {
            files: vec![FileSnapshot {
                path: PathBuf::from("ui/package-lock.json"),
                contents: Some(vec![1, 2, 3]),
            }],
        };

        let after = AreaSnapshot {
            files: vec![FileSnapshot {
                path: PathBuf::from("ui/package-lock.json"),
                contents: Some(vec![9, 9, 9]),
            }],
        };

        assert!(updates_detected_for_area(&before, &after));
    }
}
