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

/// Verify and validate signatures on all bundles.
pub fn run_verify(verbose: bool) -> Result<()> {
    // Verify we're on macOS
    if Platform::current() != Platform::MacOS {
        anyhow::bail!("Code signing verification is only supported on macOS");
    }

    let bundled_dir = paths::bundled_dir()?;

    let bundles = [
        ("vstkit.vst3", "VST3"),
        ("vstkit.clap", "CLAP"),
        ("vstkit.component", "AU"),
    ];

    print_status("Verifying bundle signatures...");

    let mut verified_count = 0;

    for (bundle_name, format) in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            verify_signature(&bundle_path)?;
            
            if verbose {
                inspect_signature(&bundle_path)?;
            }
            
            // Validate expected properties
            validate_signature_properties(&bundle_path, verbose)?;
            
            print_success(&format!("{} signature valid", format));
            verified_count += 1;
        }
    }

    if verified_count == 0 {
        anyhow::bail!("No plugin bundles found to verify");
    }

    print_success(&format!("All {} signatures verified successfully", verified_count));
    Ok(())
}

/// Inspect signature details (for verbose output).
fn inspect_signature(bundle_path: &Path) -> Result<()> {
    let output = Command::new("codesign")
        .arg("-dv")
        .arg("--verbose=4")
        .arg(bundle_path)
        .output()
        .context("Failed to inspect signature")?;

    // codesign outputs to stderr
    if !output.stderr.is_empty() {
        println!("\n{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(())
}

/// Validate that signature has expected properties.
fn validate_signature_properties(bundle_path: &Path, verbose: bool) -> Result<()> {
    // Get signature info
    let output = Command::new("codesign")
        .arg("-d")
        .arg("--entitlements")
        .arg(":-") // Output to stdout
        .arg("--verbose=2")
        .arg(bundle_path)
        .output()
        .context("Failed to get signature info")?;

    let info = String::from_utf8_lossy(&output.stderr);

    // Check for hardened runtime
    if !info.contains("runtime") {
        anyhow::bail!(
            "Bundle {} is missing hardened runtime flag",
            bundle_path.display()
        );
    }

    // Parse and validate entitlements
    let entitlements_output = String::from_utf8_lossy(&output.stdout);
    
    if !entitlements_output.is_empty() {
        // Check for required JIT entitlement
        if !entitlements_output.contains("com.apple.security.cs.allow-jit") {
            anyhow::bail!(
                "Bundle {} is missing required JIT entitlement (needed for WebView JavaScript)",
                bundle_path.display()
            );
        }
        
        if verbose {
            println!("✓ Hardened runtime enabled");
            println!("✓ JIT entitlement present");
            
            if entitlements_output.contains("com.apple.security.cs.allow-unsigned-executable-memory") {
                println!("✓ Unsigned executable memory allowed");
            }
            if entitlements_output.contains("com.apple.security.cs.disable-library-validation") {
                println!("✓ Library validation disabled");
            }
        }
    } else if verbose {
        println!("⚠ No entitlements found (ad-hoc signature may not include entitlements)");
    }

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
