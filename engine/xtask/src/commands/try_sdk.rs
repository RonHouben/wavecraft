//! Try SDK command - Generate and open a local test plugin project.

use anyhow::{Context, Result, bail};
use std::fs;
use std::process::Command;
use xtask::output::*;
use xtask::paths;
use xtask::{npm_command, run_command_checked};

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
    let ui_dir = workspace_root.join("ui");
    let output_rel = DEFAULT_OUTPUT_RELATIVE;
    let output_abs = workspace_root.join(output_rel);

    ensure_tmp_dir(&workspace_root, verbose)?;

    print_status("Step 1/3: Refreshing local UI package artifacts...");
    run_ui_build_lib_preflight(&ui_dir, verbose)
        .with_context(|| format!("Failed preflight UI package build in {}", ui_dir.display()))?;
    print_success_item("UI package artifacts are up-to-date.");

    print_status("Step 2/3: Creating test plugin project...");
    run_create_command(&workspace_root, output_rel)
        .with_context(|| format!("Failed to generate project at {}", output_abs.display()))?;
    print_success_item(&format!("Created project at {}", output_abs.display()));

    print_status("Step 3/3: Opening project in VS Code...");
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

fn run_ui_build_lib_preflight(ui_dir: &std::path::Path, verbose: bool) -> Result<()> {
    if !ui_dir.is_dir() {
        bail!(
            "UI workspace directory not found at {}. Expected repository layout with `ui/` at workspace root.",
            ui_dir.display()
        );
    }

    if !xtask::command_exists("npm") {
        bail!(
            "`npm` command not found. Install Node.js/npm and ensure `npm` is available in PATH before running `cargo xtask try-sdk`."
        );
    }

    if verbose {
        print_info(&format!(
            "Preflight: running `npm run build:lib` in {}",
            ui_dir.display()
        ));
    }

    let status = npm_command()
        .args(["run", "build:lib"])
        .current_dir(ui_dir)
        .status()
        .context("Failed to launch npm for `npm run build:lib`")?;

    if !status.success() {
        bail!(
            "`npm run build:lib` failed in {} (exit code: {}). Ensure ui dependencies are installed (run npm install in ui/) and fix build errors before retrying.",
            ui_dir.display(),
            status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "terminated by signal".to_string())
        );
    }

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
