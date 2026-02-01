//! Bundle command - Build and bundle VST3/CLAP plugins using nih_plug_xtask.

use anyhow::{Context, Result};
use std::process::Command;

use xtask::BuildMode;
use xtask::PLUGIN_NAME;
use xtask::cargo_command;
use xtask::output::*;
use xtask::paths;

/// Run the bundle command.
///
/// This builds the plugin library and packages it into VST3 and CLAP bundles.
pub fn run(mode: BuildMode, package: Option<&str>, verbose: bool) -> Result<()> {
    run_with_features(mode, package, &[], verbose)
}

/// Run the bundle command with specific features.
pub fn run_with_features(
    mode: BuildMode,
    package: Option<&str>,
    features: &[&str],
    verbose: bool,
) -> Result<()> {
    let package_name = package.unwrap_or(PLUGIN_NAME);
    let engine_dir = paths::engine_dir()?;

    // Build the React UI assets
    print_status("Building React UI assets...");
    let ui_dir = engine_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not find workspace root"))?
        .join("ui");

    if !ui_dir.exists() {
        anyhow::bail!("UI directory not found: {}", ui_dir.display());
    }

    // Read version from workspace Cargo.toml
    let version = xtask::read_workspace_version().context("Failed to read workspace version")?;

    if verbose {
        println!("  Plugin version: {}", version);
    }

    let npm_build = Command::new("npm")
        .current_dir(&ui_dir)
        .env("VITE_APP_VERSION", &version)
        .args(["run", "build"])
        .status()
        .context("Failed to run npm build")?;

    if !npm_build.success() {
        anyhow::bail!("UI build failed");
    }

    print_success("React UI built successfully");

    print_status(&format!("Building {} plugin...", package_name));

    // Build the command: cargo build --release -p <package>
    let mut build_cmd = cargo_command();
    build_cmd.current_dir(&engine_dir);
    build_cmd.arg("build");
    build_cmd.arg("-p").arg(package_name);

    if let Some(flag) = mode.cargo_flag() {
        build_cmd.arg(flag);
    }

    if !features.is_empty() {
        build_cmd.arg("--features").arg(features.join(","));
    }

    if verbose {
        println!(
            "Running: cargo build -p {} {:?} --features {:?}",
            package_name,
            mode.cargo_flag(),
            features
        );
    }

    let build_status = build_cmd.status().context("Failed to run cargo build")?;

    if !build_status.success() {
        anyhow::bail!(
            "Build command failed with exit code: {:?}",
            build_status.code()
        );
    }

    print_status("Bundling plugins...");

    // Use nih_plug_xtask's bundler
    // We need to pass features through to ensure the right binary is bundled
    let mut bundle_args = vec!["bundle".to_string(), package_name.to_string()];
    if let Some(flag) = mode.cargo_flag() {
        bundle_args.push(flag.to_string());
    }

    // Pass features to nih_plug_xtask so it rebuilds with the right features
    if !features.is_empty() {
        bundle_args.push("--features".to_string());
        bundle_args.push(features.join(","));
    }

    if verbose {
        println!("Bundle args: {:?}", bundle_args);
    }

    // Call nih_plug_xtask::main_with_args with features
    if let Err(e) = nih_plug_xtask::main_with_args(package_name, bundle_args) {
        anyhow::bail!("Bundle command failed: {}", e);
    }

    // Verify bundles were created
    let bundled_dir = paths::bundled_dir()?;
    let vst3_bundle = bundled_dir.join(format!("{}.vst3", package_name));
    let clap_bundle = bundled_dir.join(format!("{}.clap", package_name));

    if vst3_bundle.exists() {
        print_success_item(&format!("VST3: {}", vst3_bundle.display()));
    }
    if clap_bundle.exists() {
        print_success_item(&format!("CLAP: {}", clap_bundle.display()));
    }

    Ok(())
}
