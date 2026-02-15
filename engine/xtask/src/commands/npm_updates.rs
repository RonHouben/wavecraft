//! NPM update check command.
//!
//! Checks for outdated npm packages in:
//! - ui/ workspace root (workspace-aware)
//! - sdk-template/ui/ standalone project

use anyhow::{Context, Result};
use std::path::Path;
use xtask::output::*;
use xtask::{command_exists, npm_command, paths};

/// Configuration for npm update checks.
#[derive(Debug, Clone)]
pub struct NpmUpdatesConfig {
    /// When true, return a non-zero exit code if updates are found.
    pub strict: bool,
    /// Show verbose command output.
    pub verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutdatedResult {
    UpToDate,
    Outdated,
    Failed,
}

/// Run npm update checks across monorepo npm projects.
pub fn run(config: NpmUpdatesConfig) -> Result<()> {
    print_header("NPM Package Update Check");

    if !command_exists("npm") {
        anyhow::bail!("npm is not installed or not on PATH");
    }

    let ui_dir = paths::ui_dir()?;
    let sdk_template_ui_dir = paths::sdk_template_ui_dir()?;

    let ui_result = run_area(
        "Area 1/2: UI workspace (ui/)",
        &ui_dir,
        &["outdated", "--workspaces", "--include-workspace-root"],
        config.verbose,
    )
    .context("Failed while checking updates in ui/")?;

    let sdk_template_result = run_area(
        "Area 2/2: SDK template app (sdk-template/ui/)",
        &sdk_template_ui_dir,
        &["outdated"],
        config.verbose,
    )
    .context("Failed while checking updates in sdk-template/ui/")?;

    println!();
    print_status("Summary:");

    print_summary_item("ui/", ui_result);
    print_summary_item("sdk-template/ui/", sdk_template_result);

    if ui_result == OutdatedResult::Failed || sdk_template_result == OutdatedResult::Failed {
        anyhow::bail!("One or more npm outdated checks failed to run");
    }

    let updates_found =
        ui_result == OutdatedResult::Outdated || sdk_template_result == OutdatedResult::Outdated;

    if updates_found {
        if config.strict {
            anyhow::bail!(
                "Outdated npm packages found. Re-run with '--allow-updates' for informational mode."
            );
        }

        println!();
        print_warning(
            "Outdated packages found (informational mode). Returning success due to --allow-updates.",
        );
        return Ok(());
    }

    println!();
    print_success("No outdated npm packages found.");
    Ok(())
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
    use super::{OutdatedResult, classify_outdated_status};

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
}
