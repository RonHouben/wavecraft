//! Test command - Run unit tests for specified crates.

use anyhow::{Context, Result};
use std::process::Command;

use xtask::output::*;
use xtask::paths;

/// Default crates to test when no specific package is requested.
const DEFAULT_TEST_CRATES: &[&str] = &["dsp", "protocol"];

/// Run the test command.
///
/// Runs `cargo test` for the specified crates, or the default crates if none specified.
pub fn run(packages: Option<Vec<String>>, test_all: bool, verbose: bool) -> Result<()> {
    let engine_dir = paths::engine_dir()?;

    print_status("Running unit tests...");

    let mut cmd = Command::new("cargo");
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
            println!("Running: cargo test -p {}", DEFAULT_TEST_CRATES.join(" -p "));
        }
    }

    let status = cmd
        .status()
        .context("Failed to run cargo test")?;

    if !status.success() {
        anyhow::bail!("Tests failed with exit code: {:?}", status.code());
    }

    print_success("All tests passed.");
    Ok(())
}
