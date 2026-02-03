//! Notarization command for macOS bundles.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

use xtask::Platform;
use xtask::output::*;
use xtask::paths;

/// Notarization request state (persisted to disk).
#[derive(Debug, Serialize, Deserialize)]
struct NotarizationRequest {
    request_id: String,
    submitted_at: String,
    bundles: Vec<String>,
}

const REQUEST_FILE: &str = ".notarization-request";

/// Configuration for notarization.
pub struct NotarizeConfig {
    /// Apple ID for notarization
    pub apple_id: String,
    /// Team ID
    pub team_id: String,
    /// App-specific password (or keychain reference)
    pub password: String,
    /// Enable verbose output
    pub verbose: bool,
}

impl NotarizeConfig {
    /// Load configuration from environment.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            apple_id: std::env::var("APPLE_ID").context("APPLE_ID environment variable not set")?,
            team_id: std::env::var("APPLE_TEAM_ID")
                .context("APPLE_TEAM_ID environment variable not set")?,
            password: std::env::var("APPLE_APP_PASSWORD")
                .unwrap_or_else(|_| "@keychain:AC_PASSWORD".to_string()),
            verbose: false,
        })
    }
}

/// Notarization subcommand.
#[derive(Debug, Clone, Copy)]
pub enum NotarizeAction {
    Submit,
    Status,
    Staple,
    Full, // Submit, wait, staple (blocking)
}

/// Run the notarize command.
pub fn run(action: NotarizeAction, config: NotarizeConfig) -> Result<()> {
    if Platform::current() != Platform::MacOS {
        anyhow::bail!("Notarization is only supported on macOS");
    }

    match action {
        NotarizeAction::Submit => submit(&config),
        NotarizeAction::Status => status(&config),
        NotarizeAction::Staple => staple(&config),
        NotarizeAction::Full => full(&config),
    }
}

/// Submit bundles for notarization.
fn submit(config: &NotarizeConfig) -> Result<()> {
    let bundled_dir = paths::bundled_dir()?;
    let engine_dir = paths::engine_dir()?;

    // Create a ZIP of all bundles
    let zip_path = engine_dir.join("target").join("wavecraft-notarize.zip");

    print_status("Creating submission archive...");
    create_submission_zip(&bundled_dir, &zip_path)?;

    print_status("Submitting to Apple notary service...");

    let output = Command::new("xcrun")
        .arg("notarytool")
        .arg("submit")
        .arg(&zip_path)
        .arg("--apple-id")
        .arg(&config.apple_id)
        .arg("--team-id")
        .arg(&config.team_id)
        .arg("--password")
        .arg(&config.password)
        .arg("--output-format")
        .arg("json")
        .output()
        .context("Failed to run notarytool")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Notarization submission failed: {}", stderr);
    }

    // Parse response to get request ID
    let response: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse notarytool response")?;

    let request_id = response["id"]
        .as_str()
        .context("No request ID in response")?
        .to_string();

    // Save request state
    let request = NotarizationRequest {
        request_id: request_id.clone(),
        submitted_at: chrono::Utc::now().to_rfc3339(),
        bundles: vec!["wavecraft.vst3", "wavecraft.clap", "wavecraft.component"]
            .into_iter()
            .map(String::from)
            .collect(),
    };

    let request_file = engine_dir.join(REQUEST_FILE);
    fs::write(&request_file, serde_json::to_string_pretty(&request)?)
        .context("Failed to write request file")?;

    print_success(&format!(
        "Submission successful! Request ID: {}",
        request_id
    ));
    print_info(&format!("Request saved to: {}", request_file.display()));
    print_info("Run 'cargo xtask notarize --status' to check progress");
    print_info("Run 'cargo xtask notarize --staple' when complete");

    // Clean up ZIP
    fs::remove_file(&zip_path).ok();

    Ok(())
}

/// Check notarization status.
fn status(config: &NotarizeConfig) -> Result<()> {
    let request = load_request()?;

    print_status(&format!(
        "Checking status for request {}...",
        request.request_id
    ));

    let output = Command::new("xcrun")
        .arg("notarytool")
        .arg("info")
        .arg(&request.request_id)
        .arg("--apple-id")
        .arg(&config.apple_id)
        .arg("--team-id")
        .arg(&config.team_id)
        .arg("--password")
        .arg(&config.password)
        .arg("--output-format")
        .arg("json")
        .output()
        .context("Failed to run notarytool info")?;

    let response: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Failed to parse notarytool response")?;

    let status = response["status"].as_str().unwrap_or("unknown");

    match status {
        "Accepted" => {
            print_success("Notarization complete! Status: Accepted");
            print_info("Run 'cargo xtask notarize --staple' to staple the ticket");
        }
        "In Progress" => {
            print_info("Notarization in progress... Check again in a few minutes");
        }
        "Invalid" | "Rejected" => {
            print_error(&format!("Notarization failed! Status: {}", status));
            // Fetch log for details
            print_info("Fetching detailed log...");
            fetch_notarization_log(&request.request_id, config)?;
        }
        _ => {
            print_warning(&format!("Unknown status: {}", status));
        }
    }

    Ok(())
}

/// Staple notarization ticket to bundles.
fn staple(_config: &NotarizeConfig) -> Result<()> {
    let bundled_dir = paths::bundled_dir()?;

    let bundles = ["wavecraft.vst3", "wavecraft.clap", "wavecraft.component"];

    for bundle_name in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            print_status(&format!("Stapling ticket to {}...", bundle_name));

            let status = Command::new("xcrun")
                .arg("stapler")
                .arg("staple")
                .arg(&bundle_path)
                .status()
                .context("Failed to run stapler")?;

            if !status.success() {
                anyhow::bail!("Stapling failed for {}", bundle_name);
            }

            print_success(&format!("{} stapled", bundle_name));
        }
    }

    // Verify with spctl
    print_status("Verifying notarization...");
    for bundle_name in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            let status = Command::new("spctl")
                .arg("--assess")
                .arg("--type")
                .arg("install")
                .arg("--verbose")
                .arg(&bundle_path)
                .status()
                .context("Failed to run spctl")?;

            if status.success() {
                print_success(&format!("{} passed Gatekeeper check", bundle_name));
            } else {
                print_warning(&format!("{} may have Gatekeeper issues", bundle_name));
            }
        }
    }

    // Clean up request file
    let engine_dir = paths::engine_dir()?;
    fs::remove_file(engine_dir.join(REQUEST_FILE)).ok();

    print_success("Notarization complete! Bundles are ready for distribution.");

    Ok(())
}

/// Full notarization workflow (blocking).
fn full(config: &NotarizeConfig) -> Result<()> {
    submit(config)?;

    print_status("Waiting for notarization to complete...");

    // Poll for completion
    let max_attempts = 60; // 30 minutes at 30-second intervals
    for attempt in 1..=max_attempts {
        std::thread::sleep(std::time::Duration::from_secs(30));

        let request = load_request()?;

        let output = Command::new("xcrun")
            .arg("notarytool")
            .arg("info")
            .arg(&request.request_id)
            .arg("--apple-id")
            .arg(&config.apple_id)
            .arg("--team-id")
            .arg(&config.team_id)
            .arg("--password")
            .arg(&config.password)
            .arg("--output-format")
            .arg("json")
            .output()
            .context("Failed to run notarytool info")?;

        let response: serde_json::Value = serde_json::from_slice(&output.stdout)
            .context("Failed to parse notarytool response")?;

        let status = response["status"].as_str().unwrap_or("unknown");

        match status {
            "Accepted" => {
                print_success("Notarization accepted!");
                return staple(config);
            }
            "Invalid" | "Rejected" => {
                print_error(&format!("Notarization failed: {}", status));
                fetch_notarization_log(&request.request_id, config)?;
                anyhow::bail!("Notarization rejected");
            }
            _ => {
                print_info(&format!(
                    "Attempt {}/{}: Still processing...",
                    attempt, max_attempts
                ));
            }
        }
    }

    anyhow::bail!("Notarization timed out after 30 minutes");
}

/// Load the saved notarization request.
fn load_request() -> Result<NotarizationRequest> {
    let engine_dir = paths::engine_dir()?;
    let request_file = engine_dir.join(REQUEST_FILE);

    let content = fs::read_to_string(&request_file)
        .context("No pending notarization request. Run 'cargo xtask notarize --submit' first")?;

    serde_json::from_str(&content).context("Failed to parse request file")
}

/// Create a ZIP archive for notarization submission.
fn create_submission_zip(bundled_dir: &Path, zip_path: &Path) -> Result<()> {
    // Remove existing ZIP if present
    fs::remove_file(zip_path).ok();

    let status = Command::new("ditto")
        .arg("-c")
        .arg("-k")
        .arg("--keepParent")
        .arg(bundled_dir)
        .arg(zip_path)
        .status()
        .context("Failed to create ZIP archive")?;

    if !status.success() {
        anyhow::bail!("Failed to create submission ZIP");
    }

    Ok(())
}

/// Fetch and display notarization log for failed submissions.
fn fetch_notarization_log(request_id: &str, config: &NotarizeConfig) -> Result<()> {
    let output = Command::new("xcrun")
        .arg("notarytool")
        .arg("log")
        .arg(request_id)
        .arg("--apple-id")
        .arg(&config.apple_id)
        .arg("--team-id")
        .arg(&config.team_id)
        .arg("--password")
        .arg(&config.password)
        .output()
        .context("Failed to fetch notarization log")?;

    let log = String::from_utf8_lossy(&output.stdout);
    print_error("Notarization log:");
    println!("{}", log);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notarization_request_serialization() {
        let request = NotarizationRequest {
            request_id: "abc-123".to_string(),
            submitted_at: "2026-01-31T00:00:00Z".to_string(),
            bundles: vec!["wavecraft.vst3".to_string()],
        };

        let json = serde_json::to_string(&request).unwrap();
        let parsed: NotarizationRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.request_id, "abc-123");
    }

    #[test]
    fn test_notarize_config_from_env() {
        // SAFETY: This is a test that runs in isolation. Environment variable
        // modification is acceptable in single-threaded test contexts.
        unsafe {
            std::env::set_var("APPLE_ID", "test@example.com");
            std::env::set_var("APPLE_TEAM_ID", "ABC123XYZ");
            std::env::set_var("APPLE_APP_PASSWORD", "test-password");
        }

        let config = NotarizeConfig::from_env().unwrap();
        assert_eq!(config.apple_id, "test@example.com");
        assert_eq!(config.team_id, "ABC123XYZ");
        assert_eq!(config.password, "test-password");

        // SAFETY: Cleanup after test
        unsafe {
            std::env::remove_var("APPLE_ID");
            std::env::remove_var("APPLE_TEAM_ID");
            std::env::remove_var("APPLE_APP_PASSWORD");
        }
    }
}
