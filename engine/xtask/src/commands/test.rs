//! Test command - Run unit tests for specified crates.

use anyhow::{Context, Result};
use std::process::Command;

use xtask::cargo_command;
use xtask::output::*;
use xtask::paths;

/// Default crates to test when no specific package is requested.
const DEFAULT_TEST_CRATES: &[&str] = &["dsp", "protocol"];

/// Run the test command.
///
/// Runs tests for engine (Rust) and/or UI (npm), based on flags.
pub fn run(
    packages: Option<Vec<String>>,
    test_all: bool,
    ui_only: bool,
    engine_only: bool,
    verbose: bool,
) -> Result<()> {
    // Determine what to test
    let run_ui = ui_only || !engine_only;
    let run_engine = engine_only || !ui_only;

    let mut all_passed = true;

    if run_engine {
        print_status("Running engine tests...");
        if let Err(e) = run_engine_tests(packages, test_all, verbose) {
            print_error(&format!("Engine tests failed: {:#}", e));
            all_passed = false;
        }
    }

    if run_ui {
        print_status("Running UI tests...");
        if let Err(e) = run_ui_tests(verbose) {
            print_error(&format!("UI tests failed: {:#}", e));
            all_passed = false;
        }
    }

    if all_passed {
        print_success("All tests passed.");
        Ok(())
    } else {
        anyhow::bail!("Some tests failed");
    }
}

/// Run engine (Rust) tests.
fn run_engine_tests(packages: Option<Vec<String>>, test_all: bool, verbose: bool) -> Result<()> {
    let engine_dir = paths::engine_dir()?;

    let mut cmd = cargo_command();
    cmd.current_dir(&engine_dir);
    cmd.arg("test");

    if test_all {
        // Test entire workspace
        cmd.arg("--workspace");
        if verbose {
            println!("Running: cargo test --workspace");
        }
    } else if let Some(pkgs) = packages {
        // Test specified packages
        for pkg in &pkgs {
            cmd.arg("-p").arg(pkg);
        }
        if verbose {
            println!("Running: cargo test -p {}", pkgs.join(" -p "));
        }
    } else {
        // Test default crates
        for pkg in DEFAULT_TEST_CRATES {
            cmd.arg("-p").arg(*pkg);
        }
        if verbose {
            println!(
                "Running: cargo test -p {}",
                DEFAULT_TEST_CRATES.join(" -p ")
            );
        }
    }

    let status = cmd.status().context("Failed to run cargo test")?;

    if !status.success() {
        anyhow::bail!("Engine tests failed with exit code: {:?}", status.code());
    }

    print_success("Engine tests passed.");
    Ok(())
}

/// Run UI (npm) tests.
fn run_ui_tests(verbose: bool) -> Result<()> {
    let ui_dir = paths::ui_dir()?;

    if verbose {
        println!("Running: npm test (in {:?})", ui_dir);
    }

    let mut cmd = Command::new("npm");
    cmd.current_dir(&ui_dir);
    cmd.arg("test");

    let status = cmd.status().context("Failed to run npm test")?;

    if !status.success() {
        anyhow::bail!("UI tests failed with exit code: {:?}", status.code());
    }

    print_success("UI tests passed.");
    Ok(())
}
