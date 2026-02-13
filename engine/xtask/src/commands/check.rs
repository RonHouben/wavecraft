//! Check command - Pre-push validation script that simulates CI pipeline locally.
//!
//! This command runs all the checks that would run in the CI pipeline:
//! 0. Documentation link checking (`scripts/check-links.sh`)
//! 1. UI dist build (always rebuild to mirror CI)
//! 2. Linting + type-checking (with optional auto-fix for lint/format)
//! 3. Automated tests (engine + UI)
//! 4. Template validation (`validate-template`) [--full only]
//! 5. CD dry-run (`cd_dry_run`) [--full only]
//!
//! This is much faster than running the full CI pipeline via Docker/act
//! because it runs natively on the local machine.
//!
//! **Note:** Visual testing is done separately via the Playwright MCP skill
//! by the Tester agent. Use `cargo xtask dev` to start the dev servers,
//! then invoke the "playwright-mcp-ui-testing" skill for visual validation.

use anyhow::Result;
use std::process::Command;
use std::time::{Duration, Instant};

use super::{build_ui, cd_dry_run, lint, test, validate_template};
use xtask::output::*;
use xtask::paths;

/// Check configuration options.
#[derive(Debug, Clone, Default)]
pub struct CheckConfig {
    /// Auto-fix linting issues where possible
    pub fix: bool,
    /// Skip documentation checks
    pub skip_docs: bool,
    /// Skip linting
    pub skip_lint: bool,
    /// Skip automated tests
    pub skip_tests: bool,
    /// Enable extended checks (template validation + CD dry-run)
    pub full: bool,
    /// Skip template validation phase (only applicable with --full)
    pub skip_template: bool,
    /// Skip CD dry-run phase (only applicable with --full)
    pub skip_cd: bool,
    /// Show verbose output
    pub verbose: bool,
}

/// Result tracking for each check phase.
struct CheckResults {
    docs: Option<Result<Duration>>,
    lint: Option<Result<Duration>>,
    test: Option<Result<Duration>>,
    template: Option<Result<Duration>>,
    cd_dry_run: Option<Result<Duration>>,
}

impl CheckResults {
    fn new() -> Self {
        Self {
            docs: None,
            lint: None,
            test: None,
            template: None,
            cd_dry_run: None,
        }
    }

    fn all_passed(&self) -> bool {
        let docs_ok = self.docs.as_ref().is_none_or(|r| r.is_ok());
        let lint_ok = self.lint.as_ref().is_none_or(|r| r.is_ok());
        let test_ok = self.test.as_ref().is_none_or(|r| r.is_ok());
        let template_ok = self.template.as_ref().is_none_or(|r| r.is_ok());
        let cd_ok = self.cd_dry_run.as_ref().is_none_or(|r| r.is_ok());
        docs_ok && lint_ok && test_ok && template_ok && cd_ok
    }
}

/// Run the check command - a fast local CI simulation.
///
/// This is the recommended way to validate changes before pushing.
/// It runs the same checks as CI but natively (no Docker), making it
/// approximately 20-30x faster.
pub fn run(config: CheckConfig) -> Result<()> {
    print_header("Wavecraft Pre-Push Check");
    println!();
    println!("Running local CI checks (faster than Docker-based CI)");
    println!();

    let mut results = CheckResults::new();
    let total_start = Instant::now();

    // Phase 0: Documentation
    if !config.skip_docs {
        results.docs = Some(run_docs_phase(&config));
    } else {
        print_skip("Skipping documentation checks (--skip-docs)");
        println!();
    }

    // Phase 1: UI Dist Build (always rebuild to mirror CI behavior)
    run_ui_build_phase(&config)?;

    // Phase 2: Linting + type-checking
    if !config.skip_lint {
        results.lint = Some(run_lint_phase(&config));
    } else {
        print_skip("Skipping linting (--skip-lint)");
        println!();
    }

    // Phase 3: Automated Tests
    if !config.skip_tests {
        results.test = Some(run_test_phase(&config));
    } else {
        print_skip("Skipping tests (--skip-tests)");
        println!();
    }

    // Phase 4: Template Validation (--full only)
    if config.full && !config.skip_template {
        results.template = Some(run_template_phase(&config));
    } else if config.full && config.skip_template {
        print_skip("Skipping template validation (--skip-template)");
        println!();
    }

    // Phase 5: CD Dry-Run (--full only)
    if config.full && !config.skip_cd {
        results.cd_dry_run = Some(run_cd_dry_run_phase(&config));
    } else if config.full && config.skip_cd {
        print_skip("Skipping CD dry-run (--skip-cd)");
        println!();
    }

    // Print summary
    let total_duration = total_start.elapsed();
    print_summary(&results, total_duration, config.full);

    if results.all_passed() {
        Ok(())
    } else {
        anyhow::bail!("Some checks failed. See summary above.");
    }
}

fn build_ui_dist(verbose: bool) -> Result<()> {
    print_status("Building UI dist for embedded asset tests (always rebuild)");
    build_ui::run(verbose, true)
}

/// Run the documentation link check phase.
fn run_docs_phase(config: &CheckConfig) -> Result<Duration> {
    print_phase("Phase 0: Documentation");
    let start = Instant::now();

    let project_root = paths::project_root()?;
    let check_links_script = project_root.join("scripts/check-links.sh");

    if !check_links_script.exists() {
        anyhow::bail!(
            "Documentation check script not found at {}",
            check_links_script.display()
        );
    }

    if config.verbose {
        println!("  Running: bash {}", check_links_script.display());
    }

    let status = Command::new("bash")
        .arg(&check_links_script)
        .current_dir(project_root)
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run check-links.sh: {}", e))?;

    if !status.success() {
        anyhow::bail!("Documentation link check failed");
    }

    Ok(start.elapsed())
}

/// Run the UI dist build phase.
fn run_ui_build_phase(config: &CheckConfig) -> Result<()> {
    println!();
    print_phase("Phase 1: UI Dist Build");
    build_ui_dist(config.verbose)
}

/// Run the linting + type-checking phase.
fn run_lint_phase(config: &CheckConfig) -> Result<Duration> {
    print_phase("Phase 2: Linting + Type-Checking");
    let start = Instant::now();

    let targets = lint::LintTargets {
        ui: true,
        engine: true,
        fix: config.fix,
    };

    lint::run(targets, config.verbose)?;
    Ok(start.elapsed())
}

/// Run the automated test phase.
fn run_test_phase(config: &CheckConfig) -> Result<Duration> {
    println!();
    print_phase("Phase 3: Automated Tests");
    let start = Instant::now();

    // Run both engine and UI tests
    test::run(None, true, false, false, config.verbose)?;
    Ok(start.elapsed())
}

/// Run the template validation phase.
fn run_template_phase(config: &CheckConfig) -> Result<Duration> {
    println!();
    print_phase("Phase 4: Template Validation");
    let start = Instant::now();

    validate_template::run(validate_template::ValidateTemplateConfig {
        verbose: config.verbose,
        keep: false,
    })?;

    Ok(start.elapsed())
}

/// Run the CD dry-run phase.
fn run_cd_dry_run_phase(config: &CheckConfig) -> Result<Duration> {
    println!();
    print_phase("Phase 5: CD Dry-Run");
    let start = Instant::now();

    cd_dry_run::run(cd_dry_run::CdDryRunConfig {
        verbose: config.verbose,
        base_ref: "main".to_string(),
    })?;

    Ok(start.elapsed())
}

/// Print a phase header.
fn print_phase(name: &str) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status(name);
}

/// Print the final summary.
fn print_summary(results: &CheckResults, total_duration: Duration, full: bool) {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status("Summary");
    println!();

    // Documentation result
    if let Some(ref result) = results.docs {
        match result {
            Ok(duration) => {
                print_success_item(&format!(
                    "Documentation: PASSED ({:.1}s)",
                    duration.as_secs_f64()
                ));
            }
            Err(e) => {
                print_error(&format!("  ✗ Documentation: FAILED - {}", e));
            }
        }
    } else {
        println!("  ⊘ Documentation: SKIPPED");
    }

    // Lint result
    if let Some(ref result) = results.lint {
        match result {
            Ok(duration) => {
                print_success_item(&format!("Linting: PASSED ({:.1}s)", duration.as_secs_f64()));
            }
            Err(e) => {
                print_error(&format!("  ✗ Linting: FAILED - {}", e));
            }
        }
    } else {
        println!("  ⊘ Linting: SKIPPED");
    }

    // Test result
    if let Some(ref result) = results.test {
        match result {
            Ok(duration) => {
                print_success_item(&format!(
                    "Automated Tests: PASSED ({:.1}s)",
                    duration.as_secs_f64()
                ));
            }
            Err(e) => {
                print_error(&format!("  ✗ Automated Tests: FAILED - {}", e));
            }
        }
    } else {
        println!("  ⊘ Automated Tests: SKIPPED");
    }

    // Template validation result
    if let Some(ref result) = results.template {
        match result {
            Ok(duration) => {
                print_success_item(&format!(
                    "Template Validation: PASSED ({:.1}s)",
                    duration.as_secs_f64()
                ));
            }
            Err(e) => {
                print_error(&format!("  ✗ Template Validation: FAILED - {}", e));
            }
        }
    } else if full {
        println!("  ⊘ Template Validation: SKIPPED");
    }

    // CD dry-run result
    if let Some(ref result) = results.cd_dry_run {
        match result {
            Ok(duration) => {
                print_success_item(&format!(
                    "CD Dry-Run: PASSED ({:.1}s)",
                    duration.as_secs_f64()
                ));
            }
            Err(e) => {
                print_error(&format!("  ✗ CD Dry-Run: FAILED - {}", e));
            }
        }
    } else if full {
        println!("  ⊘ CD Dry-Run: SKIPPED");
    }

    println!();
    println!("Total time: {:.1}s", total_duration.as_secs_f64());
    println!();
    if !full {
        println!("💡 Run with --full to include template validation and CD dry-run.");
    }
    println!("💡 For visual testing, run 'cargo xtask dev' and use the");
    println!("   'playwright-mcp-ui-testing' skill for browser-based validation.");

    if results.all_passed() {
        println!();
        print_success("All checks passed! Ready to push.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_results_all_passed() {
        let mut results = CheckResults::new();
        assert!(results.all_passed(), "Empty results should pass");

        results.lint = Some(Ok(Duration::from_secs(1)));
        assert!(results.all_passed(), "Passing lint should pass");

        results.test = Some(Ok(Duration::from_secs(2)));
        assert!(results.all_passed(), "Passing tests should pass");
    }

    #[test]
    fn test_check_results_failure() {
        let mut results = CheckResults::new();
        results.lint = Some(Err(anyhow::anyhow!("lint failed")));
        assert!(!results.all_passed(), "Failed lint should fail");
    }

    #[test]
    fn test_default_config() {
        let config = CheckConfig::default();
        assert!(!config.fix);
        assert!(!config.skip_docs);
        assert!(!config.skip_lint);
        assert!(!config.skip_tests);
        assert!(!config.full);
        assert!(!config.skip_template);
        assert!(!config.skip_cd);
    }
}
