//! Command implementations for the xtask build system.

pub mod au;
pub mod build_ui;
pub mod bundle;
pub mod cd_dry_run;
pub mod check;
pub mod clean;
pub mod desktop;
pub mod dev;
pub mod install;
pub mod lint;
pub mod notarize;
pub mod npm_updates;
pub mod release;
pub mod sign;
pub mod sync_ui_versions;
pub mod test;
pub mod validate_cli_deps;
pub mod validate_template;

use anyhow::Result;
use xtask::BuildMode;

/// Run the full build pipeline.
pub fn run_all(
    mode: BuildMode,
    skip_tests: bool,
    skip_au: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    use xtask::Platform;
    use xtask::output::*;

    print_header("Wavecraft Full Build Pipeline");

    // Step 1: Run tests
    if !skip_tests {
        print_status("Step 1/4: Running tests...");
        if dry_run {
            println!("  [dry-run] Would run: cargo test -p dsp -p protocol");
        } else {
            test::run(None, false, false, true, verbose)?; // ui=false, engine=true
        }
        print_success("Tests passed.");
        println!();
    } else {
        print_skip("Skipping tests (--skip-tests)");
        println!();
    }

    // Step 2: Bundle plugins
    print_status("Step 2/4: Building and bundling plugins...");
    if dry_run {
        println!(
            "  [dry-run] Would run: cargo xtask bundle wavecraft {}",
            mode.cargo_flag().unwrap_or("")
        );
    } else {
        bundle::run(mode, None, verbose)?;
    }
    print_success("VST3 and CLAP bundles created.");
    println!();

    // Step 3: Build AU (macOS only)
    if Platform::current().is_macos() && !skip_au {
        print_status("Step 3/4: Building AU wrapper...");
        if dry_run {
            println!("  [dry-run] Would run: cmake -B build && cmake --build build");
        } else {
            au::run(dry_run, verbose)?;
        }
        print_success("AU wrapper built.");
        println!();
    } else if skip_au {
        print_skip("Skipping AU build (--skip-au)");
        println!();
    } else {
        print_skip("Skipping AU build (macOS only)");
        println!();
    }

    // Step 4: Install plugins
    print_status("Step 4/4: Installing plugins...");
    if dry_run {
        println!("  [dry-run] Would install plugins to system directories");
    } else {
        install::run(dry_run, verbose)?;
    }
    print_success("Plugins installed.");
    println!();

    // Summary
    print_header("Build Complete");
    print_success("All steps completed successfully!");

    Ok(())
}
