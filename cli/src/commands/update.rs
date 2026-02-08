use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Current CLI version, known at compile time.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result of the CLI self-update attempt.
enum SelfUpdateResult {
    /// CLI binary was updated to a new version.
    Updated,
    /// Already at the latest version.
    AlreadyUpToDate,
    /// Self-update failed (non-fatal).
    Failed,
}

/// Result of project dependency updates.
enum ProjectUpdateResult {
    /// Not in a project directory — deps skipped.
    NotInProject,
    /// Project deps updated (may include partial failures).
    Updated { errors: Vec<String> },
}

/// Update the CLI and (if in a project) project dependencies.
pub fn run() -> Result<()> {
    // Phase 1: CLI self-update (always runs)
    let self_update_result = update_cli();

    // Phase 2: Project dependency update (context-dependent)
    let project_result = update_project_deps();

    // Summary and exit code
    print_summary(&self_update_result, &project_result)
}

/// Perform CLI self-update via `cargo install wavecraft`.
///
/// Runs `cargo install wavecraft`, captures output, and determines whether
/// a new version was installed, the CLI is already up-to-date, or the
/// update failed. Failures are non-fatal — captured as `SelfUpdateResult::Failed`.
fn update_cli() -> SelfUpdateResult {
    println!("🔄 Checking for CLI updates...");

    let output = match Command::new("cargo")
        .args(["install", "wavecraft"])
        .output()
    {
        Ok(output) => output,
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

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "⚠️  CLI self-update failed: cargo install failed: {}",
            stderr.trim()
        );
        eprintln!("   Run 'cargo install wavecraft' manually to update the CLI.");
        return SelfUpdateResult::Failed;
    }

    // Detect whether a new version was installed vs already up-to-date
    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_already_up_to_date(&stderr) {
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    // clap outputs: "wavecraft X.Y.Z\n"
    let version = stdout
        .trim()
        .strip_prefix("wavecraft ")
        .unwrap_or(stdout.trim())
        .to_string();

    Ok(version)
}

/// Update project dependencies (Rust crates + npm packages) if in a project directory.
fn update_project_deps() -> ProjectUpdateResult {
    let has_engine = Path::new("engine/Cargo.toml").exists();
    let has_ui = Path::new("ui/package.json").exists();

    if !has_engine && !has_ui {
        println!();
        println!("ℹ️  Not in a Wavecraft plugin project — skipping dependency updates.");
        println!(
            "   Run this command from a project root to also update Rust and npm dependencies."
        );
        return ProjectUpdateResult::NotInProject;
    }

    let mut errors = Vec::new();

    if has_engine {
        println!("📦 Updating Rust dependencies...");
        match update_rust_deps() {
            Ok(()) => println!("✅ Rust dependencies updated"),
            Err(e) => {
                eprintln!("❌ Rust update failed: {}", e);
                errors.push(format!("Rust: {}", e));
            }
        }
    }

    if has_ui {
        println!("📦 Updating npm dependencies...");
        match update_npm_deps() {
            Ok(()) => println!("✅ npm dependencies updated"),
            Err(e) => {
                eprintln!("❌ npm update failed: {}", e);
                errors.push(format!("npm: {}", e));
            }
        }
    }

    ProjectUpdateResult::Updated { errors }
}

/// Print a summary of both update phases and determine the exit code.
fn print_summary(self_update: &SelfUpdateResult, project: &ProjectUpdateResult) -> Result<()> {
    let cli_updated = matches!(self_update, SelfUpdateResult::Updated);
    let cli_failed = matches!(self_update, SelfUpdateResult::Failed);
    let in_project = matches!(project, ProjectUpdateResult::Updated { .. });

    let project_errors = match project {
        ProjectUpdateResult::Updated { errors } => errors.clone(),
        ProjectUpdateResult::NotInProject => vec![],
    };

    // Show re-run hint if CLI was updated and project deps were also run
    if cli_updated && in_project {
        println!();
        println!("💡 Note: Project dependencies were updated using the previous CLI version.");
        println!("   Re-run `wavecraft update` to use the new CLI for dependency updates.");
    }

    // Check for project dependency failures
    if !project_errors.is_empty() {
        bail!(
            "Failed to update some dependencies:\n  {}",
            project_errors.join("\n  ")
        );
    }

    // Final summary based on outcomes
    if cli_failed && in_project {
        // CLI failed but project deps succeeded
        println!();
        println!("✨ Project dependencies updated (CLI self-update skipped)");
    } else if !cli_failed {
        // Everything that was attempted succeeded
        println!();
        println!("✨ All updates complete");
    }
    // If cli_failed && !in_project: warning + not-in-project messages already shown

    Ok(())
}

fn update_rust_deps() -> Result<()> {
    let status = Command::new("cargo")
        .arg("update")
        .current_dir("engine")
        .status()
        .context("Failed to run 'cargo update'. Is cargo installed?")?;

    if !status.success() {
        bail!("cargo update exited with status {}", status);
    }

    Ok(())
}

fn update_npm_deps() -> Result<()> {
    let status = Command::new("npm")
        .arg("update")
        .current_dir("ui")
        .status()
        .context("Failed to run 'npm update'. Is npm installed?")?;

    if !status.success() {
        bail!("npm update exited with status {}", status);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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

    #[test]
    fn test_detects_engine_only() {
        let temp = TempDir::new().unwrap();
        let engine_dir = temp.path().join("engine");
        fs::create_dir(&engine_dir).unwrap();
        fs::write(engine_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let has_engine = temp.path().join("engine/Cargo.toml").exists();
        let has_ui = temp.path().join("ui/package.json").exists();

        assert!(has_engine);
        assert!(!has_ui);
    }

    #[test]
    fn test_detects_ui_only() {
        let temp = TempDir::new().unwrap();
        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        let has_engine = temp.path().join("engine/Cargo.toml").exists();
        let has_ui = temp.path().join("ui/package.json").exists();

        assert!(!has_engine);
        assert!(has_ui);
    }

    #[test]
    fn test_detects_both() {
        let temp = TempDir::new().unwrap();

        let engine_dir = temp.path().join("engine");
        fs::create_dir(&engine_dir).unwrap();
        fs::write(engine_dir.join("Cargo.toml"), "[package]").unwrap();

        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        let has_engine = temp.path().join("engine/Cargo.toml").exists();
        let has_ui = temp.path().join("ui/package.json").exists();

        assert!(has_engine);
        assert!(has_ui);
    }
}
