//! Build UI command.
//!
//! Builds the React UI using npm.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use xtask::npm_command;
use xtask::output::*;
use xtask::paths;

/// Build the React UI.
///
/// When `strict_install` is true, dependency installation always runs to mirror
/// CI behavior and avoid stale local `node_modules` state.
pub fn run(verbose: bool, strict_install: bool) -> Result<()> {
    print_header("Build React UI");

    let ui_dir = paths::ui_dir()?;
    let sdk_template_ui_dir = paths::sdk_template_ui_dir()?;

    if verbose {
        println!("  UI workspace directory: {}", ui_dir.display());
        println!(
            "  SDK template UI directory: {}",
            sdk_template_ui_dir.display()
        );
    }

    // Step 1: Build package workspace libraries in ui/
    ensure_npm_deps(&ui_dir, &["ci"], "npm ci", verbose, strict_install)?;
    run_npm(&ui_dir, &["run", "build:lib"], "npm run build:lib", verbose)?;
    print_success_item("UI packages built");

    // Step 2: Build template app in sdk-template/ui/
    ensure_npm_deps(
        &sdk_template_ui_dir,
        &["install"],
        "npm install",
        verbose,
        strict_install,
    )?;
    run_npm(
        &sdk_template_ui_dir,
        &["run", "build"],
        "npm run build",
        verbose,
    )?;

    // Step 3: Copy sdk-template/ui/dist to ui/dist for engine embedding
    let template_dist_dir = sdk_template_ui_dir.join("dist");
    if !template_dist_dir.exists() {
        anyhow::bail!(
            "Template UI build succeeded but dist directory not found at {}",
            template_dist_dir.display()
        );
    }

    let dist_dir = ui_dir.join("dist");
    copy_dist_dir(&template_dist_dir, &dist_dir)?;

    if verbose {
        println!(
            "  Dist directory populated for engine embedding: {}",
            dist_dir.display()
        );
    }

    print_success("UI built successfully");

    Ok(())
}

fn ensure_npm_deps(
    dir: &Path,
    install_args: &[&str],
    install_cmd_name: &str,
    verbose: bool,
    strict_install: bool,
) -> Result<()> {
    let node_modules = dir.join("node_modules");
    if !strict_install && node_modules.exists() {
        if verbose {
            println!(
                "  node_modules exists, skipping {} in {}",
                install_cmd_name,
                dir.display()
            );
        }
        return Ok(());
    }

    if strict_install && verbose && node_modules.exists() {
        println!(
            "  strict install mode: running {} even though node_modules exists in {}",
            install_cmd_name,
            dir.display()
        );
    }

    run_npm(dir, install_args, install_cmd_name, verbose)
}

fn run_npm(dir: &Path, args: &[&str], command_name: &str, verbose: bool) -> Result<()> {
    print_status(&format!("Running {}...", command_name));

    if verbose {
        println!("  Running: npm {} (in {})", args.join(" "), dir.display());
    }

    let status = npm_command()
        .args(args)
        .current_dir(dir)
        .status()
        .with_context(|| format!("Failed to run {}", command_name))?;

    if !status.success() {
        anyhow::bail!("{} failed", command_name);
    }

    Ok(())
}

fn copy_dist_dir(source_dist: &Path, target_dist: &Path) -> Result<()> {
    if target_dist.exists() {
        fs::remove_dir_all(target_dist).with_context(|| {
            format!(
                "Failed to remove existing dist directory {}",
                target_dist.display()
            )
        })?;
    }

    fs::create_dir_all(target_dist).with_context(|| {
        format!(
            "Failed to create target dist directory {}",
            target_dist.display()
        )
    })?;

    for entry in fs::read_dir(source_dist).with_context(|| {
        format!(
            "Failed to read source dist directory {}",
            source_dist.display()
        )
    })? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target_dist.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "Failed to copy file from {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }

    Ok(())
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)
        .with_context(|| format!("Failed to create target directory {}", target.display()))?;

    for entry in fs::read_dir(source)
        .with_context(|| format!("Failed to read source directory {}", source.display()))?
    {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "Failed to copy file from {} to {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }

    Ok(())
}
