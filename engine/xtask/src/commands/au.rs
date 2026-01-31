//! AU command - Build AU wrapper using CMake (macOS only).

use anyhow::{Context, Result, bail};
use std::process::Command;

use xtask::Platform;
use xtask::command_exists;
use xtask::output::*;
use xtask::paths;

/// Run the AU build command.
///
/// Builds the Audio Unit wrapper using CMake. Only works on macOS.
pub fn run(dry_run: bool, verbose: bool) -> Result<()> {
    // Check platform
    if !Platform::current().is_macos() {
        print_skip("AU wrapper build is only supported on macOS.");
        return Ok(());
    }

    // Check CMake is available
    if !command_exists("cmake") {
        print_error("CMake not found.");
        print_warning("AU wrapper requires CMake. Install with:");
        println!("  brew install cmake");
        bail!("CMake is required to build the AU wrapper");
    }

    let au_wrapper_dir = paths::au_wrapper_dir()?;

    if !au_wrapper_dir.exists() {
        bail!(
            "AU wrapper directory not found: {}",
            au_wrapper_dir.display()
        );
    }

    print_status("Building AU wrapper...");

    // Configure with CMake
    if dry_run {
        println!("  [dry-run] Would run: cmake -B build");
    } else {
        if verbose {
            println!("Running: cmake -B build in {}", au_wrapper_dir.display());
        }

        let status = Command::new("cmake")
            .current_dir(&au_wrapper_dir)
            .arg("-B")
            .arg("build")
            .status()
            .context("Failed to run cmake configure")?;

        if !status.success() {
            bail!("CMake configure failed with exit code: {:?}", status.code());
        }
    }

    // Build with CMake
    if dry_run {
        println!("  [dry-run] Would run: cmake --build build");
    } else {
        if verbose {
            println!("Running: cmake --build build");
        }

        let status = Command::new("cmake")
            .current_dir(&au_wrapper_dir)
            .arg("--build")
            .arg("build")
            .status()
            .context("Failed to run cmake build")?;

        if !status.success() {
            bail!("CMake build failed with exit code: {:?}", status.code());
        }
    }

    // Verify the component was built
    let component_path = au_wrapper_dir
        .join("build")
        .join(format!("{}.component", xtask::PLUGIN_DISPLAY_NAME));

    if !dry_run && component_path.exists() {
        print_success_item(&format!("AU: {}", component_path.display()));
    }

    print_success("AU build complete.");
    Ok(())
}
