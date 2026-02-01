//! Desktop POC command.
//!
//! Builds and runs the standalone desktop application that demonstrates
//! WebView ↔ Rust IPC communication.

use anyhow::{Context, Result};
use std::process::Command;
use xtask::cargo_command;
use xtask::output::*;

/// Run the desktop POC application.
///
/// This builds the React UI (if needed), then builds and launches the desktop app.
pub fn run(release: bool, build_ui: bool, verbose: bool) -> Result<()> {
    print_header("Desktop POC");

    // Step 1: Build React UI if requested
    if build_ui {
        print_status("Building React UI...");
        build_react_ui(verbose)?;
        print_success("React UI built");
    }

    // Step 2: Build desktop application
    print_status("Building desktop application...");
    let profile = if release { "release" } else { "debug" };

    let mut cmd = cargo_command();
    cmd.arg("build").arg("-p").arg("desktop");

    if release {
        cmd.arg("--release");
    }

    if verbose {
        println!(
            "  Running: cargo build -p desktop{}",
            if release { " --release" } else { "" }
        );
    }

    let status = cmd.status().context("Failed to execute cargo build")?;

    if !status.success() {
        anyhow::bail!("Desktop build failed");
    }

    print_success(&format!("Desktop built ({})", profile));

    // Step 3: Run the application
    print_status("Launching desktop application...");

    let binary_path = format!("target/{}/desktop", profile);

    if verbose {
        println!("  Running: {}", binary_path);
    }

    let mut cmd = Command::new(&binary_path);

    let status = cmd
        .status()
        .context("Failed to launch desktop application")?;

    if !status.success() {
        anyhow::bail!("Desktop application exited with error");
    }

    Ok(())
}

/// Build the React UI using npm.
fn build_react_ui(verbose: bool) -> Result<()> {
    let ui_dir = std::path::Path::new("../ui");

    if !ui_dir.exists() {
        anyhow::bail!("UI directory not found at ../ui");
    }

    // Check if node_modules exists
    let node_modules = ui_dir.join("node_modules");
    if !node_modules.exists() {
        print_status("Installing npm dependencies...");

        let status = Command::new("npm")
            .arg("install")
            .current_dir(ui_dir)
            .status()
            .context("Failed to run npm install")?;

        if !status.success() {
            anyhow::bail!("npm install failed");
        }
    }

    // Run npm build
    if verbose {
        println!("  Running: npm run build (in ui/)");
    }

    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir(ui_dir)
        .status()
        .context("Failed to run npm build")?;

    if !status.success() {
        anyhow::bail!("npm build failed");
    }

    Ok(())
}
