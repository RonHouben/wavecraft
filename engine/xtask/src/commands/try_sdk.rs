//! Try SDK command - Generate and open a local test plugin project.

use anyhow::{Context, Result, bail};
use std::fs;
use std::process::Command;
use xtask::output::*;
use xtask::paths;
use xtask::run_command_checked;

const DEFAULT_PROJECT_NAME: &str = "TestPlugin";
const DEFAULT_OUTPUT_RELATIVE: &str = "target/tmp/test-plugin";

/// Run the `try-sdk` command.
///
/// Equivalent behavior:
/// 1) `cargo run --manifest-path cli/Cargo.toml -- create "TestPlugin" --output "target/tmp/test-plugin"`
/// 2) `code "target/tmp/test-plugin"`
pub fn run(verbose: bool) -> Result<()> {
    print_header("Try Wavecraft SDK");

    let workspace_root = paths::project_root()?;
    let output_rel = DEFAULT_OUTPUT_RELATIVE;
    let output_abs = workspace_root.join(output_rel);

    ensure_tmp_dir(&workspace_root, verbose)?;

    print_status("Step 1/2: Creating test plugin project...");
    run_create_command(&workspace_root, output_rel)
        .with_context(|| format!("Failed to generate project at {}", output_abs.display()))?;
    print_success_item(&format!("Created project at {}", output_abs.display()));

    print_status("Step 2/2: Opening project in VS Code...");
    run_code_open_command(&workspace_root, output_rel).with_context(|| {
        format!(
            "Failed to open project in VS Code. Ensure `code` CLI is installed and available in PATH. Path: {}",
            output_abs.display()
        )
    })?;
    print_success_item(&format!("Opened {}", output_abs.display()));

    print_success("try-sdk completed successfully.");
    Ok(())
}

fn ensure_tmp_dir(workspace_root: &std::path::Path, verbose: bool) -> Result<()> {
    let tmp_dir = workspace_root.join("target").join("tmp");
    if !tmp_dir.exists() {
        if verbose {
            print_status(&format!("Creating {}", tmp_dir.display()));
        }
        fs::create_dir_all(&tmp_dir)
            .with_context(|| format!("Failed to create {}", tmp_dir.display()))?;
    }
    Ok(())
}

fn run_create_command(workspace_root: &std::path::Path, output_rel: &str) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args([
        "run",
        "--manifest-path",
        "cli/Cargo.toml",
        "--",
        "create",
        DEFAULT_PROJECT_NAME,
        "--output",
        output_rel,
    ])
    .current_dir(workspace_root);

    run_command_checked(&mut cmd).context(
        "`cargo run --manifest-path cli/Cargo.toml -- create \"TestPlugin\" --output \"target/tmp/test-plugin\"` failed",
    )
}

fn run_code_open_command(workspace_root: &std::path::Path, output_rel: &str) -> Result<()> {
    if !xtask::command_exists("code") {
        bail!(
            "`code` command not found. Install the VS Code shell command (Command Palette: 'Shell Command: Install \'code\' command in PATH')."
        );
    }

    let mut cmd = Command::new("code");
    cmd.arg(output_rel).current_dir(workspace_root);

    run_command_checked(&mut cmd).context("`code \"target/tmp/test-plugin\"` failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_requested_values() {
        assert_eq!(DEFAULT_PROJECT_NAME, "TestPlugin");
        assert_eq!(DEFAULT_OUTPUT_RELATIVE, "target/tmp/test-plugin");
    }
}
