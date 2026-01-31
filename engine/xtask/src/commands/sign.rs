//! Code signing command for macOS bundles.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use xtask::output::*;
use xtask::paths;
use xtask::Platform;

/// Configuration for code signing.
pub struct SigningConfig {
    /// Developer ID identity (e.g., "Developer ID Application: Name (TEAM_ID)")
    pub identity: String,
    /// Path to entitlements file
    pub entitlements: Option<String>,
    /// Enable verbose output
    pub verbose: bool,
}

impl SigningConfig {
    /// Load signing configuration from environment.
    pub fn from_env() -> Result<Self> {
        let identity = std::env::var("APPLE_SIGNING_IDENTITY")
            .context("APPLE_SIGNING_IDENTITY environment variable not set")?;

        let entitlements = std::env::var("APPLE_ENTITLEMENTS").ok();

        Ok(Self {
            identity,
            entitlements,
            verbose: false,
        })
    }
}

/// Run the sign command.
pub fn run(config: SigningConfig) -> Result<()> {
    // Verify we're on macOS
    if Platform::current() != Platform::MacOS {
        anyhow::bail!("Code signing is only supported on macOS");
    }

    let bundled_dir = paths::bundled_dir()?;

    // Sign each bundle type
    let bundles = [
        ("vstkit.vst3", "VST3"),
        ("vstkit.clap", "CLAP"),
        ("vstkit.component", "AU"),
    ];

    for (bundle_name, format) in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            print_status(&format!("Signing {} bundle...", format));
            sign_bundle(&bundle_path, &config)?;
            print_success(&format!("{} bundle signed", format));
        } else if config.verbose {
            print_warning(&format!("{} bundle not found, skipping", format));
        }
    }

    // Verify signatures
    print_status("Verifying signatures...");
    for (bundle_name, format) in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            verify_signature(&bundle_path)?;
            print_success(&format!("{} signature valid", format));
        }
    }

    Ok(())
}

/// Sign a single bundle.
fn sign_bundle(bundle_path: &Path, config: &SigningConfig) -> Result<()> {
    let mut cmd = Command::new("codesign");

    cmd.arg("--deep")
        .arg("--force")
        .arg("--options")
        .arg("runtime") // Enable hardened runtime
        .arg("--timestamp"); // Include secure timestamp

    // Add entitlements if specified
    if let Some(ref entitlements) = config.entitlements {
        cmd.arg("--entitlements").arg(entitlements);
    } else {
        // Use default entitlements
        let default_entitlements = paths::engine_dir()?.join("signing").join("entitlements.plist");
        if default_entitlements.exists() {
            cmd.arg("--entitlements").arg(&default_entitlements);
        }
    }

    cmd.arg("--sign").arg(&config.identity);
    cmd.arg(bundle_path);

    if config.verbose {
        println!("Running: {:?}", cmd);
    }

    let status = cmd.status().context("Failed to run codesign")?;

    if !status.success() {
        anyhow::bail!(
            "codesign failed for {}: {}",
            bundle_path.display(),
            diagnose_signing_error(status.code().unwrap_or(-1))
        );
    }

    Ok(())
}

/// Verify a bundle's signature.
fn verify_signature(bundle_path: &Path) -> Result<()> {
    let status = Command::new("codesign")
        .arg("--verify")
        .arg("--deep")
        .arg("--strict")
        .arg(bundle_path)
        .status()
        .context("Failed to run codesign --verify")?;

    if !status.success() {
        anyhow::bail!("Signature verification failed for {}", bundle_path.display());
    }

    Ok(())
}

/// Check if ad-hoc signing is requested (for local development).
pub fn run_adhoc() -> Result<()> {
    // Verify we're on macOS
    if Platform::current() != Platform::MacOS {
        anyhow::bail!("Code signing is only supported on macOS");
    }

    let bundled_dir = paths::bundled_dir()?;

    let bundles = ["vstkit.vst3", "vstkit.clap", "vstkit.component"];

    for bundle_name in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            print_status(&format!("Ad-hoc signing {}...", bundle_name));

            let status = Command::new("codesign")
                .arg("--deep")
                .arg("--force")
                .arg("--sign")
                .arg("-") // Ad-hoc signature
                .arg(&bundle_path)
                .status()
                .context("Failed to run codesign")?;

            if !status.success() {
                anyhow::bail!("Ad-hoc signing failed for {}", bundle_path.display());
            }
        }
    }

    print_success("Ad-hoc signing complete");
    Ok(())
}

/// User-friendly error messages for common signing issues.
fn diagnose_signing_error(code: i32) -> &'static str {
    match code {
        1 => "Invalid identity or certificate not found. \
              Run 'security find-identity -v -p codesigning' to list available identities.",
        3 => "Bundle is malformed or has invalid structure.",
        _ => "Unknown error. Run with --verbose for details.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signing_config_from_env() {
        std::env::set_var("APPLE_SIGNING_IDENTITY", "Test Identity");
        let config = SigningConfig::from_env().unwrap();
        assert_eq!(config.identity, "Test Identity");
        std::env::remove_var("APPLE_SIGNING_IDENTITY");
    }

    #[test]
    fn test_signing_config_missing_env() {
        std::env::remove_var("APPLE_SIGNING_IDENTITY");
        let result = SigningConfig::from_env();
        assert!(result.is_err());
    }
}
