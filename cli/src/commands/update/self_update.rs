use super::CURRENT_VERSION;
use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

/// Result of the CLI self-update attempt.
///
/// Note: Uses unit variants rather than data-carrying variants (as described in LLD).
/// Version information is printed directly in `update_cli()` rather than stored in the
/// enum, which simplifies control flow without losing functionality.
pub(super) enum SelfUpdateResult {
    /// CLI binary was updated to a new version.
    Updated,
    /// Already at the latest version.
    AlreadyUpToDate,
    /// Self-update failed (non-fatal).
    Failed,
}

/// Progress phases during `cargo install`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum InstallPhase {
    Checking,
    Downloading,
    Compiling,
}

/// Perform CLI self-update via `cargo install wavecraft`.
///
/// Runs `cargo install wavecraft`, captures output, and determines whether
/// a new version was installed, the CLI is already up-to-date, or the
/// update failed. Failures are non-fatal — captured as `SelfUpdateResult::Failed`.
///
/// Note: No timeout is applied to the `cargo install` subprocess. Compilation
/// typically takes 30-60 seconds. A hang (network stall, compilation freeze) will
/// block `wavecraft update` indefinitely. This is acceptable for a CLI tool —
/// users can Ctrl-C to abort. A timeout may be added in a future version if needed.
pub(super) fn update_cli() -> SelfUpdateResult {
    println!("🔄 Checking for CLI updates...");

    let mut child = match Command::new("cargo")
        .args(["install", "wavecraft"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!(
                "⚠️  CLI self-update failed: Failed to run 'cargo install'. \
                 Is cargo installed? ({})",
                e
            );
            eprintln!("   Run 'cargo install wavecraft' manually to update the CLI.");
            return SelfUpdateResult::Failed;
        }
    };

    // Stream stderr in real time for progress feedback and capture full content
    let stderr_pipe = child
        .stderr
        .take()
        .expect("child stderr should be piped for progress parsing");
    let stderr_content = stream_install_progress(stderr_pipe);

    let status = match child.wait() {
        Ok(status) => status,
        Err(e) => {
            eprintln!("⚠️  CLI self-update failed: {}", e);
            return SelfUpdateResult::Failed;
        }
    };

    if !status.success() {
        eprintln!(
            "⚠️  CLI self-update failed: cargo install failed: {}",
            stderr_content.trim()
        );
        eprintln!("   Run 'cargo install wavecraft' manually to update the CLI.");
        return SelfUpdateResult::Failed;
    }

    // Detect whether a new version was installed vs already up-to-date
    if is_already_up_to_date(&stderr_content) {
        println!("✅ CLI is up to date ({})", CURRENT_VERSION);
        return SelfUpdateResult::AlreadyUpToDate;
    }

    // A new version was installed — query it
    match get_installed_version() {
        Ok(new_version) => {
            println!(
                "✅ CLI updated to {} (was {})",
                new_version, CURRENT_VERSION
            );
            SelfUpdateResult::Updated
        }
        Err(_) => {
            // Binary was updated but we couldn't determine the version
            println!("✅ CLI updated (was {})", CURRENT_VERSION);
            SelfUpdateResult::Updated
        }
    }
}

/// Stream stderr from `cargo install`, showing phase-appropriate progress messages.
///
/// Returns the full stderr content for later analysis.
fn stream_install_progress(stderr: impl std::io::Read) -> String {
    let reader = BufReader::new(stderr);
    let mut all_output = String::new();
    let mut current_phase = InstallPhase::Checking;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        all_output.push_str(&line);
        all_output.push('\n');

        if let Some(phase) = detect_phase(&line) {
            if phase != current_phase {
                match phase {
                    InstallPhase::Downloading => {
                        println!("📥 Downloading...");
                    }
                    InstallPhase::Compiling => {
                        println!("🔨 Compiling... this may take a minute.");
                    }
                    InstallPhase::Checking => {}
                }
                current_phase = phase;
            }
        }
    }

    all_output
}

/// Detect the install phase from a cargo stderr line.
fn detect_phase(line: &str) -> Option<InstallPhase> {
    let trimmed = line.trim();

    if trimmed.starts_with("Downloading") || trimmed.starts_with("Downloaded") {
        Some(InstallPhase::Downloading)
    } else if trimmed.starts_with("Compiling") {
        Some(InstallPhase::Compiling)
    } else {
        None
    }
}

/// Re-execute the newly installed CLI binary to continue with project deps.
///
/// Uses `exec()` on Unix to replace the process image. The new binary
/// runs `wavecraft update --skip-self`, which skips phase 1 and runs
/// phase 2 using the updated code.
pub(super) fn reexec_with_new_binary() -> Result<()> {
    println!();
    println!("🔄 Continuing with updated CLI...");

    let binary = which_wavecraft()?;

    let mut cmd = Command::new(&binary);
    cmd.args(["update", "--skip-self"]);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let err = cmd.exec();
        // exec() only returns on error
        bail!("Failed to re-exec CLI: {}", err);
    }

    #[cfg(not(unix))]
    {
        let status = cmd.status().context("Failed to re-exec CLI")?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

/// Find the wavecraft binary path.
fn which_wavecraft() -> Result<std::path::PathBuf> {
    which::which("wavecraft").context(
        "Could not find 'wavecraft' binary after update. \
         Re-run 'wavecraft update' manually.",
    )
}

/// Detect if `cargo install` output indicates the package is already at the latest version.
///
/// `cargo install` writes to stderr. When the package is already installed at
/// the latest version, it outputs a line matching:
///   "package `wavecraft vX.Y.Z` is already installed"
fn is_already_up_to_date(stderr: &str) -> bool {
    stderr
        .lines()
        .any(|line| line.contains("is already installed"))
}

/// Query the version of the wavecraft binary currently on disk.
///
/// Invokes `wavecraft --version` which outputs `wavecraft X.Y.Z\n` via clap.
/// This runs the **disk binary** (not the currently running process), so after
/// `cargo install` completes it reflects the newly installed version.
fn get_installed_version() -> Result<String> {
    let output = Command::new("wavecraft")
        .arg("--version")
        .output()
        .context("Failed to run 'wavecraft --version'")?;

    if !output.status.success() {
        bail!("'wavecraft --version' exited with status {}", output.status);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_version_output(&stdout))
}

/// Parse the version string from `wavecraft --version` output.
///
/// clap outputs: "wavecraft X.Y.Z\n" — this strips the prefix and whitespace.
fn parse_version_output(stdout: &str) -> String {
    stdout
        .trim()
        .strip_prefix("wavecraft ")
        .unwrap_or(stdout.trim())
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- InstallPhase detection tests ---

    #[test]
    fn test_detect_phase_downloading() {
        assert_eq!(
            detect_phase("  Downloading crates ..."),
            Some(InstallPhase::Downloading)
        );
    }

    #[test]
    fn test_detect_phase_downloaded() {
        assert_eq!(
            detect_phase("  Downloaded wavecraft v0.9.2"),
            Some(InstallPhase::Downloading)
        );
    }

    #[test]
    fn test_detect_phase_compiling() {
        assert_eq!(
            detect_phase("   Compiling wavecraft v0.9.2"),
            Some(InstallPhase::Compiling)
        );
    }

    #[test]
    fn test_detect_phase_updating_index() {
        assert_eq!(detect_phase("   Updating crates.io index"), None);
    }

    #[test]
    fn test_detect_phase_installing() {
        assert_eq!(detect_phase("  Installing /path/to/bin"), None);
    }

    #[test]
    fn test_detect_phase_empty_line() {
        assert_eq!(detect_phase(""), None);
    }

    // --- stream_install_progress tests ---

    #[test]
    fn test_stream_collects_all_output() {
        let input = "  Downloading crates ...\n   Compiling wavecraft v0.9.2\n";
        let output = stream_install_progress(std::io::Cursor::new(input));
        assert!(output.contains("Downloading"));
        assert!(output.contains("Compiling"));
    }

    #[test]
    fn test_stream_handles_already_installed() {
        let input =
            "  Ignored package `wavecraft v0.9.1` is already installed, use --force to override\n";
        let output = stream_install_progress(std::io::Cursor::new(input));
        assert!(output.contains("is already installed"));
    }

    #[test]
    fn test_stream_empty_input() {
        let output = stream_install_progress(std::io::Cursor::new(""));
        assert!(output.is_empty());
    }

    // --- is_already_up_to_date tests ---

    #[test]
    fn test_is_already_up_to_date_true() {
        let stderr = "    Updating crates.io index\n     \
            Ignored package `wavecraft v0.9.1` is already installed, \
            use --force to override\n";
        assert!(is_already_up_to_date(stderr));
    }

    #[test]
    fn test_is_already_up_to_date_false_new_install() {
        let stderr = "    Updating crates.io index\n  \
            Installing wavecraft v0.9.2\n   \
            Compiling wavecraft v0.9.2\n";
        assert!(!is_already_up_to_date(stderr));
    }

    #[test]
    fn test_is_already_up_to_date_empty() {
        assert!(!is_already_up_to_date(""));
    }

    #[test]
    fn test_is_already_up_to_date_with_prefix() {
        let stderr =
            "     Ignored package `wavecraft v0.9.1` is already installed, use --force to override";
        assert!(is_already_up_to_date(stderr));
    }

    // --- parse_version_output tests (QA-L-002) ---

    #[test]
    fn test_parse_version_output_standard() {
        assert_eq!(parse_version_output("wavecraft 1.2.3\n"), "1.2.3");
    }

    #[test]
    fn test_parse_version_output_no_prefix() {
        assert_eq!(parse_version_output("1.2.3\n"), "1.2.3");
    }

    #[test]
    fn test_parse_version_output_whitespace() {
        assert_eq!(parse_version_output("  wavecraft 0.9.1  \n"), "0.9.1");
    }

    #[test]
    fn test_parse_version_output_empty() {
        assert_eq!(parse_version_output(""), "");
    }
}
