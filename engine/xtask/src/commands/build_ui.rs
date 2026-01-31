//! Build UI command.
//!
//! Builds the React UI using npm.

use anyhow::{Context, Result};
use std::process::Command;
use xtask::output::*;
use xtask::paths;

/// Build the React UI.
pub fn run(verbose: bool) -> Result<()> {
    print_header("Build React UI");

    let ui_dir = paths::ui_dir()?;

    if verbose {
        println!("  UI directory: {}", ui_dir.display());
    }

    // Check if node_modules exists
    let node_modules = ui_dir.join("node_modules");
    if !node_modules.exists() {
        print_status("Installing npm dependencies...");

        let status = Command::new("npm")
            .arg("ci")
            .current_dir(&ui_dir)
            .status()
            .context("Failed to run npm ci")?;

        if !status.success() {
            anyhow::bail!("npm ci failed");
        }
        print_success_item("Dependencies installed");
    }

    // Run npm build
    print_status("Building UI...");

    if verbose {
        println!("  Running: npm run build (in ui/)");
    }

    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(&ui_dir)
        .status()
        .context("Failed to run npm build")?;

    if !status.success() {
        anyhow::bail!("npm build failed");
    }

    // Verify dist directory was created
    let dist_dir = ui_dir.join("dist");
    if !dist_dir.exists() {
        anyhow::bail!("npm build succeeded but dist directory not found at {}", dist_dir.display());
    }

    if verbose {
        println!("  Dist directory created: {}", dist_dir.display());
    }

    print_success("UI built successfully");

    Ok(())
}
