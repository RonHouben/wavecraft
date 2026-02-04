//! Template validation command - Validates that the CLI generates working projects.
//!
//! This command replicates the GitHub Actions template-validation.yml workflow
//! locally for faster iteration. It:
//! 1. Builds the CLI
//! 2. Generates a test plugin project
//! 3. Validates the generated code compiles and passes linting
//!
//! This is the local equivalent of the CI template-validation workflow.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{env, fs};

use xtask::output::*;

/// Template validation configuration.
#[derive(Debug, Clone, Default)]
pub struct ValidateTemplateConfig {
    /// Show verbose output
    pub verbose: bool,
    /// Keep the generated test project (don't clean up)
    pub keep: bool,
}

/// Run the template validation command.
pub fn run(config: ValidateTemplateConfig) -> Result<()> {
    let start_time = Instant::now();

    print_header("Wavecraft Template Validation");
    println!();
    println!("Replicating CI template-validation.yml workflow locally");
    println!();

    // Get paths
    let workspace_root = xtask::paths::project_root()?;
    let cli_dir = workspace_root.join("cli");
    let parent_dir = env::temp_dir();
    let test_project_dir = parent_dir.join("test-plugin");

    // Clean up any existing test project
    if test_project_dir.exists() {
        if config.verbose {
            println!("Cleaning up existing test project...");
        }
        fs::remove_dir_all(&test_project_dir).context("Failed to remove existing test project")?;
    }

    // Ensure cleanup happens even on error (unless --keep is specified)
    let cleanup_guard = CleanupGuard::new(test_project_dir.clone(), config.keep, config.verbose);

    // Step 1: Build CLI
    print_phase("Step 1: Build Wavecraft CLI");
    build_cli(&cli_dir, config.verbose)?;
    print_success("CLI built successfully");
    println!();

    // Step 2: Generate test project
    print_phase("Step 2: Generate Test Plugin");
    let cli_binary = cli_dir.join("target/release/wavecraft");
    let parent_dir = env::temp_dir();
    generate_test_plugin(&cli_binary, &parent_dir, config.verbose)?;
    // The CLI creates the project in a subdirectory
    let test_project_dir = parent_dir.join("test-plugin");
    verify_generated_files(&test_project_dir)?;
    print_success("Test plugin generated successfully");
    println!();

    // Step 3: Add path overrides for local workspace
    print_phase("Step 3: Configure Local Dependencies");
    add_workspace_overrides(&test_project_dir, &workspace_root)?;
    print_success("Local workspace dependencies configured");
    println!();

    // Step 4: Validate Engine
    print_phase("Step 4: Validate Engine Code");
    validate_engine(&test_project_dir, config.verbose)?;
    print_success("Engine validation passed");
    println!();

    // Step 5: Validate UI
    print_phase("Step 5: Validate UI Code");
    validate_ui(&test_project_dir, config.verbose)?;
    print_success("UI validation passed");
    println!();

    // Step 6: Validate xtask
    print_phase("Step 6: Validate xtask Commands");
    validate_xtask(&test_project_dir, config.verbose)?;
    print_success("xtask validation passed");
    println!();

    // Print summary
    let duration = start_time.elapsed();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status("Summary");
    println!();
    print_success_item(&format!(
        "✅ All validation checks passed ({:.1}s)",
        duration.as_secs_f64()
    ));
    println!();
    print_success("Template validation successful!");

    // CleanupGuard will handle cleanup on drop (unless --keep is specified)
    drop(cleanup_guard);

    Ok(())
}

/// Build the Wavecraft CLI in release mode.
fn build_cli(cli_dir: &Path, verbose: bool) -> Result<()> {
    if verbose {
        println!("Building CLI in release mode...");
    }

    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(cli_dir)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        bail!("Failed to build CLI");
    }

    Ok(())
}

/// Generate a test plugin project using the CLI.
fn generate_test_plugin(cli_binary: &Path, parent_dir: &Path, verbose: bool) -> Result<()> {
    if verbose {
        println!("Generating test plugin in: {}", parent_dir.display());
    }

    // Use a fake version tag - we'll override with local paths anyway
    // This avoids issues when the actual version tag doesn't exist yet
    let status = Command::new(cli_binary)
        .args([
            "new",
            "test-plugin",
            "--vendor",
            "Test Vendor",
            "--email",
            "test@example.com",
            "--url",
            "https://example.com",
            "--no-git",
            "--sdk-version",
            "v0.0.0-local-test", // Placeholder version - will be patched
        ])
        .current_dir(parent_dir)
        .status()
        .context("Failed to run wavecraft new")?;

    if !status.success() {
        bail!("Failed to generate test plugin");
    }

    Ok(())
}

/// Verify that all expected files were generated.
fn verify_generated_files(project_dir: &Path) -> Result<()> {
    let expected_files = [
        "engine/Cargo.toml",
        "engine/src/lib.rs",
        "ui/package.json",
        "ui/src/App.tsx",
        "README.md",
    ];

    for file in &expected_files {
        let path = project_dir.join(file);
        if !path.exists() {
            bail!("Missing expected file: {}", file);
        }
    }

    Ok(())
}

/// Add Cargo path overrides to use local workspace dependencies.
fn add_workspace_overrides(project_dir: &Path, workspace_root: &Path) -> Result<()> {
    let engine_cargo_toml = project_dir.join("engine/Cargo.toml");
    let engine_crates = workspace_root.join("engine/crates");

    // Read the current Cargo.toml
    let mut content =
        fs::read_to_string(&engine_cargo_toml).context("Failed to read engine/Cargo.toml")?;

    // Replace git dependencies with path dependencies
    // This is simpler and more reliable than using [patch] sections
    content = content.replace(
        r#"wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.0.0-local-test" }"#,
        &format!(r#"wavecraft-core = {{ path = "{}/wavecraft-core" }}"#, engine_crates.display())
    );
    content = content.replace(
        r#"wavecraft-protocol = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.0.0-local-test" }"#,
        &format!(r#"wavecraft-protocol = {{ path = "{}/wavecraft-protocol" }}"#, engine_crates.display())
    );
    content = content.replace(
        r#"wavecraft-dsp = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.0.0-local-test" }"#,
        &format!(r#"wavecraft-dsp = {{ path = "{}/wavecraft-dsp" }}"#, engine_crates.display())
    );
    content = content.replace(
        r#"wavecraft-bridge = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.0.0-local-test" }"#,
        &format!(r#"wavecraft-bridge = {{ path = "{}/wavecraft-bridge" }}"#, engine_crates.display())
    );
    content = content.replace(
        r#"wavecraft-metering = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.0.0-local-test" }"#,
        &format!(r#"wavecraft-metering = {{ path = "{}/wavecraft-metering" }}"#, engine_crates.display())
    );

    fs::write(&engine_cargo_toml, content).context("Failed to write engine/Cargo.toml")?;

    Ok(())
}

/// Validate engine code compilation and linting.
fn validate_engine(project_dir: &Path, verbose: bool) -> Result<()> {
    let engine_dir = project_dir.join("engine");

    // Check compilation
    if verbose {
        println!("Running cargo check...");
    }
    run_command(
        "cargo",
        &["check", "--manifest-path", "engine/Cargo.toml"],
        project_dir,
    )?;

    // Clippy
    if verbose {
        println!("Running clippy...");
    }
    run_command(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
        &engine_dir,
    )?;

    // Formatting
    if verbose {
        println!("Checking formatting...");
    }
    run_command("cargo", &["fmt", "--check"], &engine_dir)?;

    Ok(())
}

/// Validate UI code compilation and linting.
fn validate_ui(project_dir: &Path, verbose: bool) -> Result<()> {
    let ui_dir = project_dir.join("ui");

    // Install dependencies
    if verbose {
        println!("Installing UI dependencies...");
    }
    run_command("npm", &["ci"], &ui_dir)?;

    // Lint
    if verbose {
        println!("Running ESLint...");
    }
    run_command("npm", &["run", "lint"], &ui_dir)?;

    // Format check
    if verbose {
        println!("Checking Prettier formatting...");
    }
    run_command("npm", &["run", "format:check"], &ui_dir)?;

    // Type-check
    if verbose {
        println!("Running TypeScript type-check...");
    }
    run_command("npm", &["run", "typecheck"], &ui_dir)?;

    // Build
    if verbose {
        println!("Building UI...");
    }
    run_command("npm", &["run", "build"], &ui_dir)?;

    Ok(())
}

/// Validate xtask commands work.
fn validate_xtask(project_dir: &Path, verbose: bool) -> Result<()> {
    let engine_dir = project_dir.join("engine");

    if verbose {
        println!("Testing xtask bundle command (dry run)...");
    }

    run_command("cargo", &["xtask", "bundle", "--check"], &engine_dir)?;

    Ok(())
}

/// Run a command and check for success.
fn run_command(cmd: &str, args: &[&str], cwd: &Path) -> Result<()> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .status()
        .with_context(|| format!("Failed to execute: {} {}", cmd, args.join(" ")))?;

    if !status.success() {
        bail!("Command failed: {} {}", cmd, args.join(" "));
    }

    Ok(())
}

/// Print a phase header.
fn print_phase(name: &str) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    print_status(name);
}

/// Cleanup guard to ensure test project is removed even on error.
struct CleanupGuard {
    path: PathBuf,
    keep: bool,
    verbose: bool,
}

impl CleanupGuard {
    fn new(path: PathBuf, keep: bool, verbose: bool) -> Self {
        Self {
            path,
            keep,
            verbose,
        }
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if !self.keep && self.path.exists() {
            if self.verbose {
                println!();
                println!("Cleaning up test project...");
            }
            if let Err(e) = fs::remove_dir_all(&self.path) {
                eprintln!("Warning: Failed to clean up test project: {}", e);
            } else if self.verbose {
                println!("Test project removed: {}", self.path.display());
            }
        } else if self.keep && self.path.exists() {
            println!();
            println!("Test project kept at: {}", self.path.display());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ValidateTemplateConfig::default();
        assert!(!config.verbose);
        assert!(!config.keep);
    }
}
