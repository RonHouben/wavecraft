//! NPM update check command.
//!
//! Checks for outdated npm packages in:
//! - ui/ workspace root (workspace-aware)
//! - sdk-template/ui/ standalone project
//!
//! and can optionally perform safe upgrades in both areas.

use anyhow::{Context, Result};
use std::path::Path;
use xtask::output::*;
use xtask::{command_exists, npm_command, paths};

/// Configuration for npm update checks.
#[derive(Debug, Clone)]
pub struct NpmUpdatesConfig {
    /// When true, return a non-zero exit code if updates are found.
    pub strict: bool,
    /// When true, run `npm update` in both npm areas.
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
    SdkTemplateUi,
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

    let ui_result = run_upgrade_area(
        "Area 1/2: UI workspace (ui/)",
        &ui_dir,
        upgrade_args_for_area(NpmArea::UiWorkspace),
        config.verbose,
    );

    let sdk_template_result = run_upgrade_area(
        "Area 2/2: SDK template app (sdk-template/ui/)",
        &sdk_template_ui_dir,
        upgrade_args_for_area(NpmArea::SdkTemplateUi),
        config.verbose,
    );

    println!();
    print_status("Summary:");
    print_upgrade_summary_item("ui/", &ui_result);
    print_upgrade_summary_item("sdk-template/ui/", &sdk_template_result);

    if ui_result.is_err() || sdk_template_result.is_err() {
        anyhow::bail!("One or more npm upgrade commands failed");
    }

    println!();
    print_success("Dependency upgrades completed successfully in all npm areas.");
    Ok(())
}

fn run_upgrade_area(header: &str, dir: &Path, args: &[&str], verbose: bool) -> Result<()> {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status(header);
    println!("  Directory: {}", dir.display());
    println!("  Action: Upgrade dependencies with npm update");

    if verbose {
        println!("  Running: npm {}", args.join(" "));
    }

    let status = npm_command()
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("Failed to run npm update in {}", dir.display()))?;

    if status.success() {
        print_success_item("Upgrade command completed successfully");
        return Ok(());
    }

    print_error_item("npm update command failed");
    if let Some(code) = status.code() {
        print_error(&format!("  Exit code: {}", code));
    }
    anyhow::bail!("npm update failed in {}", dir.display())
}

fn print_upgrade_summary_item(area: &str, result: &Result<()>) {
    match result {
        Ok(()) => print_success_item(&format!("{}: upgrade completed", area)),
        Err(_) => print_error(&format!("  ✗ {}: upgrade failed", area)),
    }
}

fn outdated_args_for_area(area: NpmArea) -> &'static [&'static str] {
    match area {
        NpmArea::UiWorkspace => &["outdated", "--workspaces", "--include-workspace-root"],
        NpmArea::SdkTemplateUi => &["outdated"],
    }
}

fn upgrade_args_for_area(area: NpmArea) -> &'static [&'static str] {
    match area {
        NpmArea::UiWorkspace => &["update", "--workspaces", "--include-workspace-root"],
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
        CheckModeSummary, NpmArea, OutdatedResult, classify_outdated_status,
        finalize_check_results, outdated_args_for_area, upgrade_args_for_area,
    };

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
    fn sdk_template_uses_standard_upgrade_args() {
        assert_eq!(upgrade_args_for_area(NpmArea::SdkTemplateUi), &["update"]);
    }
}
