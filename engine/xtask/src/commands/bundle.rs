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

    // Build the command: cargo xtask bundle <package> [--release]
    // We shell out to cargo xtask bundle because nih_plug_xtask expects to be called as main()
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&engine_dir);
    cmd.arg("xtask");
    cmd.arg("bundle");
    cmd.arg(package_name);

    if let Some(flag) = mode.cargo_flag() {
        cmd.arg(flag);
    }

    if verbose {
        println!("Running: cargo xtask bundle {} {:?}", package_name, mode.cargo_flag());
    }

    let status = cmd
        .status()
        .context("Failed to run cargo xtask bundle")?;

    if !status.success() {
        anyhow::bail!("Bundle command failed with exit code: {:?}", status.code());
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
