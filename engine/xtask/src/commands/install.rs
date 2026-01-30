//! Install command - Copy built plugins to system directories.

use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use xtask::output::*;
use xtask::paths;
use xtask::Platform;
use xtask::PLUGIN_DISPLAY_NAME;
use xtask::PLUGIN_NAME;

/// Run the install command.
///
/// Copies built plugins to the appropriate system directories.
pub fn run(dry_run: bool, verbose: bool) -> Result<()> {
    print_status("Installing plugins...");

    let bundled_dir = paths::bundled_dir()?;

    // Install VST3
    let vst3_src = bundled_dir.join(format!("{}.vst3", PLUGIN_NAME));
    let vst3_dest_dir = paths::vst3_install_dir()?;
    let vst3_dest = vst3_dest_dir.join(format!("{}.vst3", PLUGIN_NAME));

    if vst3_src.exists() {
        if dry_run {
            println!(
                "  [dry-run] Would install: {} → {}",
                vst3_src.display(),
                vst3_dest.display()
            );
        } else {
            install_bundle(&vst3_src, &vst3_dest_dir, &vst3_dest, verbose)?;
            print_success_item(&format!("Installed {}.vst3 to {}", PLUGIN_NAME, vst3_dest_dir.display()));
        }
    } else if verbose {
        println!("VST3 bundle not found at {}", vst3_src.display());
    }

    // Install CLAP
    let clap_src = bundled_dir.join(format!("{}.clap", PLUGIN_NAME));
    let clap_dest_dir = paths::clap_install_dir()?;
    let clap_dest = clap_dest_dir.join(format!("{}.clap", PLUGIN_NAME));

    if clap_src.exists() {
        if dry_run {
            println!(
                "  [dry-run] Would install: {} → {}",
                clap_src.display(),
                clap_dest.display()
            );
        } else {
            install_bundle(&clap_src, &clap_dest_dir, &clap_dest, verbose)?;
            print_success_item(&format!("Installed {}.clap to {}", PLUGIN_NAME, clap_dest_dir.display()));
        }
    } else if verbose {
        println!("CLAP bundle not found at {}", clap_src.display());
    }

    // Install AU (macOS only)
    if Platform::current().is_macos() {
        let au_src = paths::au_wrapper_dir()?
            .join("build")
            .join(format!("{}.component", PLUGIN_DISPLAY_NAME));
        let au_dest_dir = paths::au_install_dir()?;
        let au_dest = au_dest_dir.join(format!("{}.component", PLUGIN_DISPLAY_NAME));

        if au_src.exists() {
            if dry_run {
                println!(
                    "  [dry-run] Would install: {} → {}",
                    au_src.display(),
                    au_dest.display()
                );
            } else {
                install_bundle(&au_src, &au_dest_dir, &au_dest, verbose)?;
                print_success_item(&format!(
                    "Installed {}.component to {}",
                    PLUGIN_DISPLAY_NAME,
                    au_dest_dir.display()
                ));

                // Refresh macOS AU cache
                refresh_au_cache(verbose)?;
            }
        } else if verbose {
            println!("AU component not found at {}", au_src.display());
        }
    }

    print_success("Installation complete.");
    Ok(())
}

/// Install a plugin bundle by removing existing and copying new.
fn install_bundle(
    src: &std::path::Path,
    dest_dir: &std::path::Path,
    dest: &std::path::Path,
    verbose: bool,
) -> Result<()> {
    // Create destination directory if it doesn't exist
    if !dest_dir.exists() {
        if verbose {
            println!("Creating directory: {}", dest_dir.display());
        }
        fs::create_dir_all(dest_dir)
            .with_context(|| format!("Failed to create directory: {}", dest_dir.display()))?;
    }

    // Remove existing bundle if present
    if dest.exists() {
        if verbose {
            println!("Removing existing: {}", dest.display());
        }
        fs::remove_dir_all(dest)
            .with_context(|| format!("Failed to remove existing bundle: {}", dest.display()))?;
    }

    // Copy the bundle
    if verbose {
        println!("Copying: {} → {}", src.display(), dest.display());
    }
    copy_dir_recursive(src, dest)
        .with_context(|| format!("Failed to copy {} to {}", src.display(), dest.display()))?;

    Ok(())
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &std::path::Path, dest: &std::path::Path) -> Result<()> {
    fs::create_dir_all(dest)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }

    Ok(())
}

/// Refresh the macOS AudioUnit cache.
fn refresh_au_cache(verbose: bool) -> Result<()> {
    if verbose {
        println!("Refreshing macOS AU cache...");
    }

    // Kill AudioComponentRegistrar to force AU cache refresh
    // This may fail if the process isn't running, which is fine
    let _ = Command::new("killall")
        .arg("-9")
        .arg("AudioComponentRegistrar")
        .status();

    Ok(())
}
