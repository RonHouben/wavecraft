use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

/// Current CLI version, known at compile time.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result of the CLI self-update attempt.
///
/// Note: Uses unit variants rather than data-carrying variants (as described in LLD).
/// Version information is printed directly in `update_cli()` rather than stored in the
/// enum, which simplifies control flow without losing functionality.
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

/// Outcome of the summary decision logic, extracted for testability.
#[cfg_attr(test, derive(Debug, PartialEq))]
enum SummaryOutcome {
    /// Both phases completed successfully.
    AllComplete { show_rerun_hint: bool },
    /// CLI failed but project deps succeeded.
    ProjectOnlyComplete,
    /// Project dependency updates failed.
    ProjectErrors {
        errors: Vec<String>,
        show_rerun_hint: bool,
    },
    /// CLI failed and not in a project — messages already shown inline.
    NoAction,
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
///
/// Note: No timeout is applied to the `cargo install` subprocess. Compilation
/// typically takes 30-60 seconds. A hang (network stall, compilation freeze) will
/// block `wavecraft update` indefinitely. This is acceptable for a CLI tool —
/// users can Ctrl-C to abort. A timeout may be added in a future version if needed.
fn update_cli() -> SelfUpdateResult {
    println!("🔄 Checking for CLI updates...");

    let update_done = Arc::new(AtomicBool::new(false));
    let progress_handle = start_cli_update_progress(update_done.clone());

    let output_result = Command::new("cargo")
        .args(["install", "wavecraft"])
        .output();

    update_done.store(true, Ordering::Relaxed);
    let _ = progress_handle.join();

    let output = match output_result {
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

fn start_cli_update_progress(done: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let delay = Duration::from_secs(3);
        let mut slept = Duration::from_millis(0);

        while slept < delay {
            if done.load(Ordering::Relaxed) {
                return;
            }
            let step = Duration::from_millis(200);
            thread::sleep(step);
            slept += step;
        }

        if !done.load(Ordering::Relaxed) {
            println!("⏳ Still checking... this can take a minute on slow networks.");
        }
    })
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

/// Detect whether the given directory is a Wavecraft plugin project.
///
/// Returns `(has_engine, has_ui)` based on the presence of marker files
/// (`engine/Cargo.toml` and `ui/package.json`).
fn detect_project(root: &Path) -> (bool, bool) {
    let has_engine = root.join("engine/Cargo.toml").exists();
    let has_ui = root.join("ui/package.json").exists();
    (has_engine, has_ui)
}

/// Update project dependencies (Rust crates + npm packages) if in a project directory.
fn update_project_deps() -> ProjectUpdateResult {
    let (has_engine, has_ui) = detect_project(Path::new("."));

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

/// Determine the summary outcome from both update phases.
///
/// This is a pure function extracted from `print_summary()` for testability.
/// It decides what messages should be shown and whether the process should fail.
fn determine_summary(
    self_update: &SelfUpdateResult,
    project: &ProjectUpdateResult,
) -> SummaryOutcome {
    let cli_updated = matches!(self_update, SelfUpdateResult::Updated);
    let cli_failed = matches!(self_update, SelfUpdateResult::Failed);
    let in_project = matches!(project, ProjectUpdateResult::Updated { .. });

    let project_errors: &[String] = match project {
        ProjectUpdateResult::Updated { errors } => errors,
        ProjectUpdateResult::NotInProject => &[],
    };

    let show_rerun_hint = cli_updated && in_project;

    if !project_errors.is_empty() {
        return SummaryOutcome::ProjectErrors {
            errors: project_errors.to_vec(),
            show_rerun_hint,
        };
    }

    if cli_failed && in_project {
        return SummaryOutcome::ProjectOnlyComplete;
    }

    if cli_failed && !in_project {
        return SummaryOutcome::NoAction;
    }

    SummaryOutcome::AllComplete { show_rerun_hint }
}

/// Print a summary of both update phases and determine the exit code.
fn print_summary(self_update: &SelfUpdateResult, project: &ProjectUpdateResult) -> Result<()> {
    let outcome = determine_summary(self_update, project);

    match outcome {
        SummaryOutcome::AllComplete { show_rerun_hint } => {
            if show_rerun_hint {
                print_rerun_hint();
            }
            println!();
            println!("✨ All updates complete");
        }
        SummaryOutcome::ProjectOnlyComplete => {
            println!();
            println!("✨ Project dependencies updated (CLI self-update skipped)");
        }
        SummaryOutcome::ProjectErrors {
            errors,
            show_rerun_hint,
        } => {
            if show_rerun_hint {
                print_rerun_hint();
            }
            bail!(
                "Failed to update some dependencies:\n  {}",
                errors.join("\n  ")
            );
        }
        SummaryOutcome::NoAction => {}
    }

    Ok(())
}

/// Print the re-run hint for when CLI was updated but project deps ran with old binary.
fn print_rerun_hint() {
    println!();
    println!("💡 Note: Project dependencies were updated using the previous CLI version.");
    println!("   Re-run `wavecraft update` to use the new CLI for dependency updates.");
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

    // --- detect_project tests (QA-L-004) ---

    #[test]
    fn test_detect_project_engine_only() {
        let temp = TempDir::new().unwrap();
        let engine_dir = temp.path().join("engine");
        fs::create_dir(&engine_dir).unwrap();
        fs::write(engine_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let (has_engine, has_ui) = detect_project(temp.path());
        assert!(has_engine);
        assert!(!has_ui);
    }

    #[test]
    fn test_detect_project_ui_only() {
        let temp = TempDir::new().unwrap();
        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        let (has_engine, has_ui) = detect_project(temp.path());
        assert!(!has_engine);
        assert!(has_ui);
    }

    #[test]
    fn test_detect_project_both() {
        let temp = TempDir::new().unwrap();

        let engine_dir = temp.path().join("engine");
        fs::create_dir(&engine_dir).unwrap();
        fs::write(engine_dir.join("Cargo.toml"), "[package]").unwrap();

        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        let (has_engine, has_ui) = detect_project(temp.path());
        assert!(has_engine);
        assert!(has_ui);
    }

    #[test]
    fn test_detect_project_no_markers() {
        let temp = TempDir::new().unwrap();

        let (has_engine, has_ui) = detect_project(temp.path());
        assert!(!has_engine);
        assert!(!has_ui);
    }

    // --- determine_summary tests (QA-L-003) ---

    #[test]
    fn test_summary_all_complete_no_project() {
        let outcome = determine_summary(
            &SelfUpdateResult::AlreadyUpToDate,
            &ProjectUpdateResult::NotInProject,
        );
        assert_eq!(
            outcome,
            SummaryOutcome::AllComplete {
                show_rerun_hint: false
            }
        );
    }

    #[test]
    fn test_summary_all_complete_with_project() {
        let outcome = determine_summary(
            &SelfUpdateResult::AlreadyUpToDate,
            &ProjectUpdateResult::Updated { errors: vec![] },
        );
        assert_eq!(
            outcome,
            SummaryOutcome::AllComplete {
                show_rerun_hint: false
            }
        );
    }

    #[test]
    fn test_summary_updated_with_project_shows_rerun_hint() {
        let outcome = determine_summary(
            &SelfUpdateResult::Updated,
            &ProjectUpdateResult::Updated { errors: vec![] },
        );
        assert_eq!(
            outcome,
            SummaryOutcome::AllComplete {
                show_rerun_hint: true
            }
        );
    }

    #[test]
    fn test_summary_cli_failed_in_project() {
        let outcome = determine_summary(
            &SelfUpdateResult::Failed,
            &ProjectUpdateResult::Updated { errors: vec![] },
        );
        assert_eq!(outcome, SummaryOutcome::ProjectOnlyComplete);
    }

    #[test]
    fn test_summary_cli_failed_not_in_project() {
        let outcome = determine_summary(
            &SelfUpdateResult::Failed,
            &ProjectUpdateResult::NotInProject,
        );
        assert_eq!(outcome, SummaryOutcome::NoAction);
    }

    #[test]
    fn test_summary_project_errors() {
        let outcome = determine_summary(
            &SelfUpdateResult::AlreadyUpToDate,
            &ProjectUpdateResult::Updated {
                errors: vec!["Rust: compile failed".to_string()],
            },
        );
        assert_eq!(
            outcome,
            SummaryOutcome::ProjectErrors {
                errors: vec!["Rust: compile failed".to_string()],
                show_rerun_hint: false,
            }
        );
    }

    #[test]
    fn test_summary_updated_with_project_errors_shows_rerun_hint() {
        let outcome = determine_summary(
            &SelfUpdateResult::Updated,
            &ProjectUpdateResult::Updated {
                errors: vec!["npm: fetch failed".to_string()],
            },
        );
        assert_eq!(
            outcome,
            SummaryOutcome::ProjectErrors {
                errors: vec!["npm: fetch failed".to_string()],
                show_rerun_hint: true,
            }
        );
    }
}
