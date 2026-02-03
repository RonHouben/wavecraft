//! Check command - Pre-push validation script that simulates CI pipeline locally.
//!
//! This command runs all the checks that would run in the CI pipeline:
//! 1. Linting (with optional auto-fix)
//! 2. Automated tests (engine + UI)
//!
//! This is much faster than running the full CI pipeline via Docker/act
//! because it runs natively on the local machine.
//!
//! **Note:** Visual testing is done separately via the Playwright MCP skill
//! by the Tester agent. Use `cargo xtask dev` to start the dev servers,
//! then invoke the "playwright-mcp-ui-testing" skill for visual validation.

use anyhow::Result;
use std::time::{Duration, Instant};

use super::{lint, test};
use xtask::output::*;

/// Check configuration options.
#[derive(Debug, Clone, Default)]
pub struct CheckConfig {
    /// Auto-fix linting issues where possible
    pub fix: bool,
    /// Skip linting
    pub skip_lint: bool,
    /// Skip automated tests
    pub skip_tests: bool,
    /// Show verbose output
    pub verbose: bool,
}

/// Result tracking for each check phase.
struct CheckResults {
    lint: Option<Result<Duration>>,
    test: Option<Result<Duration>>,
}

impl CheckResults {
    fn new() -> Self {
        Self {
            lint: None,
            test: None,
        }
    }

    fn all_passed(&self) -> bool {
        let lint_ok = self.lint.as_ref().is_none_or(|r| r.is_ok());
        let test_ok = self.test.as_ref().is_none_or(|r| r.is_ok());
        lint_ok && test_ok
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

    // Phase 1: Linting
    if !config.skip_lint {
        results.lint = Some(run_lint_phase(&config));
    } else {
        print_skip("Skipping linting (--skip-lint)");
        println!();
    }

    // Phase 2: Automated Tests
    if !config.skip_tests {
        results.test = Some(run_test_phase(&config));
    } else {
        print_skip("Skipping tests (--skip-tests)");
        println!();
    }

    // Print summary
    let total_duration = total_start.elapsed();
    print_summary(&results, total_duration);

    if results.all_passed() {
        Ok(())
    } else {
        anyhow::bail!("Some checks failed. See summary above.");
    }
}

/// Run the linting phase.
fn run_lint_phase(config: &CheckConfig) -> Result<Duration> {
    print_phase("Phase 1: Linting");
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
    print_phase("Phase 2: Automated Tests");
    let start = Instant::now();

    // Run both engine and UI tests
    test::run(None, true, false, false, config.verbose)?;
    Ok(start.elapsed())
}

/// Print a phase header.
fn print_phase(name: &str) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status(name);
}

/// Print the final summary.
fn print_summary(results: &CheckResults, total_duration: Duration) {
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status("Summary");
    println!();

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

    println!();
    println!("Total time: {:.1}s", total_duration.as_secs_f64());
    println!();
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
        assert!(!config.skip_lint);
        assert!(!config.skip_tests);
    }
}
