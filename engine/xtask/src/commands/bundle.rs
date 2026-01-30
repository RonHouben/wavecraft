//! Bundle command - Build and bundle VST3/CLAP plugins using nih_plug_xtask.

use anyhow::{Context, Result};
use std::process::Command;

use xtask::output::*;
use xtask::paths;
use xtask::BuildMode;
use xtask::PLUGIN_NAME;

/// Run the bundle command.
///
/// This builds the plugin library and packages it into VST3 and CLAP bundles.
pub fn run(mode: BuildMode, package: Option<&str>, verbose: bool) -> Result<()> {
    let package_name = package.unwrap_or(PLUGIN_NAME);
    let engine_dir = paths::engine_dir()?;

    print_status(&format!("Building {} plugin...", package_name));

    // Build the command: cargo build --release -p <package>
    let mut build_cmd = Command::new("cargo");
    build_cmd.current_dir(&engine_dir);
    build_cmd.arg("build");
    build_cmd.arg("-p").arg(package_name);

    if let Some(flag) = mode.cargo_flag() {
        build_cmd.arg(flag);
    }

    if verbose {
        println!("Running: cargo build -p {} {:?}", package_name, mode.cargo_flag());
    }

    let build_status = build_cmd
        .status()
        .context("Failed to run cargo build")?;

    if !build_status.success() {
        anyhow::bail!("Build command failed with exit code: {:?}", build_status.code());
    }

    print_status("Bundling plugins...");

    // Use nih_plug_xtask's main function directly via the CLI wrapper
    // We need to call it as a separate binary to avoid conflict with our xtask
    let mut bundle_args = vec!["bundle".to_string(), package_name.to_string()];
    if let Some(flag) = mode.cargo_flag() {
        bundle_args.push(flag.to_string());
    }

    // Call nih_plug_xtask::main_with_args directly
    if let Err(e) = nih_plug_xtask::main_with_args(package_name, bundle_args.into_iter()) {
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
