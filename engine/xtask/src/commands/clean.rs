//! Clean command - Remove build artifacts and optionally installed plugins.

use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use xtask::output::*;
use xtask::paths;
use xtask::Platform;
use xtask::PLUGIN_DISPLAY_NAME;
use xtask::PLUGIN_NAME;

/// Run the clean command.
///
/// Cleans build artifacts and optionally removes installed plugins.
pub fn run(include_installed: bool, force: bool, dry_run: bool, verbose: bool) -> Result<()> {
    let engine_dir = paths::engine_dir()?;

    print_status("Cleaning build artifacts...");

    // Run cargo clean
    if dry_run {
        println!("  [dry-run] Would run: cargo clean");
    } else {
        if verbose {
            println!("Running: cargo clean");
        }
        let status = Command::new("cargo")
            .current_dir(&engine_dir)
            .arg("clean")
            .status()
            .context("Failed to run cargo clean")?;

        if !status.success() {
            print_warning("cargo clean returned non-zero exit code");
        }
    }

    // Remove bundled directory
    let bundled_dir = paths::bundled_dir()?;
    if bundled_dir.exists() {
        if dry_run {
            println!("  [dry-run] Would remove: {}", bundled_dir.display());
        } else {
            if verbose {
                println!("Removing: {}", bundled_dir.display());
            }
            fs::remove_dir_all(&bundled_dir)
                .with_context(|| format!("Failed to remove {}", bundled_dir.display()))?;
            print_success_item(&format!("Removed {}", bundled_dir.display()));
        }
    }

    // Remove AU wrapper build directory (macOS)
    if Platform::current().is_macos() {
        let au_build_dir = paths::au_wrapper_dir()?.join("build");
        if au_build_dir.exists() {
            if dry_run {
                println!("  [dry-run] Would remove: {}", au_build_dir.display());
            } else {
                if verbose {
                    println!("Removing: {}", au_build_dir.display());
                }
                fs::remove_dir_all(&au_build_dir)
                    .with_context(|| format!("Failed to remove {}", au_build_dir.display()))?;
                print_success_item(&format!("Removed {}", au_build_dir.display()));
            }
        }
    }

    // Remove installed plugins if requested
    if include_installed {
        if !force && !dry_run {
            print_warning("Removing installed plugins requires --force flag for safety.");
            print_warning("Use: cargo xtask clean --installed --force");
            return Ok(());
        }

        print_status("Removing installed plugins...");

        // VST3
        let vst3_dir = paths::vst3_install_dir()?;
        let vst3_plugin = vst3_dir.join(format!("{}.vst3", PLUGIN_NAME));
        if vst3_plugin.exists() {
            if dry_run {
                println!("  [dry-run] Would remove: {}", vst3_plugin.display());
            } else {
                fs::remove_dir_all(&vst3_plugin)
                    .with_context(|| format!("Failed to remove {}", vst3_plugin.display()))?;
                print_success_item(&format!("Removed {}", vst3_plugin.display()));
            }
        }

        // CLAP
        let clap_dir = paths::clap_install_dir()?;
        let clap_plugin = clap_dir.join(format!("{}.clap", PLUGIN_NAME));
        if clap_plugin.exists() {
            if dry_run {
                println!("  [dry-run] Would remove: {}", clap_plugin.display());
            } else {
                fs::remove_dir_all(&clap_plugin)
                    .with_context(|| format!("Failed to remove {}", clap_plugin.display()))?;
                print_success_item(&format!("Removed {}", clap_plugin.display()));
            }
        }

        // AU (macOS only)
        if Platform::current().is_macos() {
            let au_dir = paths::au_install_dir()?;
            let au_plugin = au_dir.join(format!("{}.component", PLUGIN_DISPLAY_NAME));
            if au_plugin.exists() {
                if dry_run {
                    println!("  [dry-run] Would remove: {}", au_plugin.display());
                } else {
                    fs::remove_dir_all(&au_plugin)
                        .with_context(|| format!("Failed to remove {}", au_plugin.display()))?;
                    print_success_item(&format!("Removed {}", au_plugin.display()));
                }
            }
        }
    }

    print_success("Clean complete.");
    Ok(())
}
