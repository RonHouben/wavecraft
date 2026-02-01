//! Release command - full build, sign, and notarize workflow.

use anyhow::Result;

use super::{bundle, notarize, sign};
use xtask::BuildMode;
use xtask::output::*;

/// Run the release command.
pub fn run(skip_notarize: bool, verbose: bool) -> Result<()> {
    print_header("VstKit Release Build");

    // Step 1: Build release bundles
    print_status("Building release bundles...");
    bundle::run(BuildMode::Release, None, verbose)?;

    // Step 2: Sign
    print_status("Signing bundles...");
    let mut sign_config = sign::SigningConfig::from_env()?;
    sign_config.verbose = verbose;
    sign::run(sign_config)?;

    // Step 3: Notarize (optional)
    if !skip_notarize {
        print_status("Notarizing bundles...");
        let mut notarize_config = notarize::NotarizeConfig::from_env()?;
        notarize_config.verbose = verbose;
        notarize::run(notarize::NotarizeAction::Full, notarize_config)?;
    } else {
        print_warning("Skipping notarization (--skip-notarize)");
    }

    print_success("Release build complete!");

    Ok(())
}
