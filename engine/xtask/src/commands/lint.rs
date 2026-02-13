//! Lint command - Run linters for UI and/or engine code.

use anyhow::{Context, Result};
use std::process::Command;

use xtask::cargo_command;
use xtask::output::*;
use xtask::paths;

/// Lint target selection.
#[derive(Debug, Clone, Copy)]
pub struct LintTargets {
    pub ui: bool,
    pub engine: bool,
    pub fix: bool,
}

impl Default for LintTargets {
    fn default() -> Self {
        Self {
            ui: true,
            engine: true,
            fix: false,
        }
    }
}

/// Run the lint command.
pub fn run(targets: LintTargets, verbose: bool) -> Result<()> {
    print_header("Wavecraft Linting");

    let mut ui_result = Ok(());
    let mut engine_result = Ok(());

    // Run engine linting
    if targets.engine {
        engine_result = run_engine_lint(targets.fix, verbose);
    }

    // Run UI linting
    if targets.ui {
        ui_result = run_ui_lint(targets.fix, verbose);
    }

    // Print summary
    println!();
    print_status("Summary:");

    if targets.engine {
        match &engine_result {
            Ok(()) => print_success_item("Engine (Rust): PASSED"),
            Err(e) => print_error(&format!("  ✗ Engine (Rust): FAILED - {}", e)),
        }
    }

    if targets.ui {
        match &ui_result {
            Ok(()) => print_success_item("UI (TypeScript): PASSED"),
            Err(e) => print_error(&format!("  ✗ UI (TypeScript): FAILED - {}", e)),
        }
    }

    // Return error if any failed
    engine_result?;
    ui_result?;

    println!();
    print_success("All linting checks passed!");
    Ok(())
}

/// Run engine linting (cargo fmt + clippy).
fn run_engine_lint(fix: bool, verbose: bool) -> Result<()> {
    let engine_dir = paths::engine_dir()?;

    println!();
    print_status("Checking Rust formatting...");

    // Step 1: cargo fmt
    let mut fmt_cmd = cargo_command();
    fmt_cmd.current_dir(&engine_dir);

    if fix {
        fmt_cmd.args(["fmt"]);
        if verbose {
            println!("Running: cargo fmt");
        }
    } else {
        fmt_cmd.args(["fmt", "--check"]);
        if verbose {
            println!("Running: cargo fmt --check");
        }
    }

    let fmt_status = fmt_cmd.status().context("Failed to run cargo fmt")?;
    if !fmt_status.success() {
        if fix {
            anyhow::bail!("cargo fmt failed");
        } else {
            anyhow::bail!(
                "Formatting issues found. Run 'cargo xtask lint --engine --fix' or 'cargo fmt' to fix."
            );
        }
    }
    print_success_item("Formatting OK");

    // Step 2: cargo clippy
    print_status("Running Clippy...");

    let mut clippy_cmd = cargo_command();
    clippy_cmd.current_dir(&engine_dir);

    if fix {
        clippy_cmd.args([
            "clippy",
            "--workspace",
            "--fix",
            "--allow-dirty",
            "--allow-staged",
            "--",
            "-D",
            "warnings",
        ]);
        if verbose {
            println!(
                "Running: cargo clippy --workspace --fix --allow-dirty --allow-staged -- -D warnings"
            );
        }
    } else {
        clippy_cmd.args(["clippy", "--workspace", "--", "-D", "warnings"]);
        if verbose {
            println!("Running: cargo clippy --workspace -- -D warnings");
        }
    }

    let clippy_status = clippy_cmd.status().context("Failed to run cargo clippy")?;
    if !clippy_status.success() {
        anyhow::bail!(
            "Clippy found issues. Run 'cargo xtask lint --engine --fix' or 'cargo clippy --fix' to auto-fix."
        );
    }
    print_success_item("Clippy OK");

    Ok(())
}

/// Run UI linting (ESLint + Prettier + TypeScript type-check).
fn run_ui_lint(fix: bool, verbose: bool) -> Result<()> {
    let ui_dir = paths::ui_dir()?;

    // Check if node_modules exists
    if !ui_dir.join("node_modules").exists() {
        anyhow::bail!(
            "node_modules not found in ui/. Run 'npm install' in the ui/ directory first."
        );
    }

    println!();
    print_status("Running ESLint...");

    // Step 1: ESLint
    let mut eslint_cmd = Command::new("npm");
    eslint_cmd.current_dir(&ui_dir);

    if fix {
        eslint_cmd.args(["run", "lint:fix"]);
        if verbose {
            println!("Running: npm run lint:fix");
        }
    } else {
        eslint_cmd.args(["run", "lint"]);
        if verbose {
            println!("Running: npm run lint");
        }
    }

    let eslint_status = eslint_cmd.status().context("Failed to run ESLint")?;
    if !eslint_status.success() {
        anyhow::bail!(
            "ESLint found issues. Run 'cargo xtask lint --ui --fix' or 'npm run lint:fix' to auto-fix."
        );
    }
    print_success_item("ESLint OK");

    // Step 2: Prettier
    print_status("Checking Prettier formatting...");

    let mut prettier_cmd = Command::new("npm");
    prettier_cmd.current_dir(&ui_dir);

    if fix {
        prettier_cmd.args(["run", "format"]);
        if verbose {
            println!("Running: npm run format");
        }
    } else {
        prettier_cmd.args(["run", "format:check"]);
        if verbose {
            println!("Running: npm run format:check");
        }
    }

    let prettier_status = prettier_cmd.status().context("Failed to run Prettier")?;
    if !prettier_status.success() {
        if fix {
            anyhow::bail!("Prettier formatting failed");
        } else {
            anyhow::bail!(
                "Formatting issues found. Run 'cargo xtask lint --ui --fix' or 'npm run format' to fix."
            );
        }
    }
    print_success_item("Prettier OK");

    // Step 3: TypeScript type-check
    print_status("Running TypeScript type-check...");

    let mut typecheck_cmd = Command::new("npm");
    typecheck_cmd.current_dir(&ui_dir);
    typecheck_cmd.args(["run", "typecheck"]);

    if verbose {
        println!("Running: npm run typecheck");
    }

    let typecheck_status = typecheck_cmd
        .status()
        .context("Failed to run TypeScript type-check")?;
    if !typecheck_status.success() {
        anyhow::bail!(
            "TypeScript type-check failed. Run 'npm run typecheck' in ui/ to inspect errors."
        );
    }
    print_success_item("TypeScript type-check OK");

    Ok(())
}
