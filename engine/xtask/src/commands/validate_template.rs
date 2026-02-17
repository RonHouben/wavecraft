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
use std::process::{Command, Output};
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
    generate_test_plugin(
        &cli_binary,
        &workspace_root,
        &test_project_dir,
        config.verbose,
    )?;
    verify_generated_files(&test_project_dir)?;

    normalize_generated_ui_dependencies(&test_project_dir, &workspace_root)?;

    print_success("Test plugin generated successfully");
    println!();

    // Step 3: Validate Engine
    print_phase("Step 3: Validate Engine Code");
    validate_engine(&test_project_dir, config.verbose)?;
    print_success("Engine validation passed");
    println!();

    // Step 4: Validate UI
    print_phase("Step 4: Validate UI Code");
    validate_ui(&test_project_dir, config.verbose)?;
    print_success("UI validation passed");
    println!();

    // Step 5: Validate xtask
    print_phase("Step 5: Validate xtask Commands");
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
fn generate_test_plugin(
    cli_binary: &Path,
    workspace_root: &Path,
    output_dir: &Path,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Generating test plugin in: {}", output_dir.display());
    }

    let output_dir_arg = output_dir.display().to_string();

    let status = Command::new(cli_binary)
        .args(create_test_plugin_args(&output_dir_arg))
        .current_dir(workspace_root)
        .status()
        .context("Failed to run wavecraft create")?;

    if !status.success() {
        bail!("Failed to generate test plugin");
    }

    Ok(())
}

fn create_test_plugin_args(output_dir: &str) -> Vec<&str> {
    vec![
        "create",
        "test-plugin",
        "--vendor",
        "Test Vendor",
        "--email",
        "test@example.com",
        "--url",
        "https://example.com",
        "--no-git",
        "--output",
        output_dir,
    ]
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

/// Normalize generated UI dependencies to local workspace package paths.
///
/// In SDK development mode, `wavecraft create` may emit local file dependencies
/// for Rust crates only while keeping npm package versions. For local template
/// validation we validate against current workspace UI packages to avoid stale
/// published npm package drift.
fn normalize_generated_ui_dependencies(project_dir: &Path, workspace_root: &Path) -> Result<()> {
    let generated_ui_package_json = project_dir.join("ui/package.json");
    let core_package_dir = workspace_root.join("ui/packages/core");
    let components_package_dir = workspace_root.join("ui/packages/components");

    let generated_content = fs::read_to_string(&generated_ui_package_json)
        .context("Failed to read generated ui/package.json")?;
    let mut generated: serde_json::Value = serde_json::from_str(&generated_content)
        .context("Failed to parse generated ui/package.json")?;

    let Some(deps) = generated
        .get_mut("dependencies")
        .and_then(serde_json::Value::as_object_mut)
    else {
        bail!("Generated ui/package.json missing dependencies object");
    };

    deps.insert(
        "@wavecraft/core".to_string(),
        serde_json::Value::String(format!("file:{}", core_package_dir.display())),
    );
    deps.insert(
        "@wavecraft/components".to_string(),
        serde_json::Value::String(format!("file:{}", components_package_dir.display())),
    );

    let serialized = serde_json::to_string_pretty(&generated)
        .context("Failed to serialize generated package")?;
    fs::write(&generated_ui_package_json, format!("{}\n", serialized))
        .context("Failed to write normalized generated ui/package.json")?;

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
    run_command("npm", &["install"], &ui_dir)?;

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
        println!("Validating xtask bundle command contract...");
    }

    let help_output = run_command_capture("cargo", &["xtask", "bundle", "--help"], &engine_dir)?;
    assert_output_contains(
        &help_output,
        "--install",
        "Expected generated xtask bundle --help to include --install flag",
    )?;

    run_command("cargo", &["xtask", "bundle", "--check"], &engine_dir)?;
    let check_install_output = run_command_capture(
        "cargo",
        &["xtask", "bundle", "--check", "--install"],
        &engine_dir,
    )?;
    assert_output_contains(
        &check_install_output,
        "Install",
        "Expected dry-run bundle --check --install output to mention install validation",
    )?;

    if verbose {
        println!("Running generated xtask unit tests...");
    }
    run_command(
        "cargo",
        &["test", "--manifest-path", "engine/xtask/Cargo.toml"],
        project_dir,
    )?;

    Ok(())
}

/// Run a command and check for success.
fn run_command(cmd: &str, args: &[&str], cwd: &Path) -> Result<()> {
    let mut command = if cmd == "npm" {
        xtask::npm_command()
    } else {
        Command::new(cmd)
    };

    let status = command
        .args(args)
        .current_dir(cwd)
        .status()
        .with_context(|| format!("Failed to execute: {} {}", cmd, args.join(" ")))?;

    if !status.success() {
        bail!("Command failed: {} {}", cmd, args.join(" "));
    }

    Ok(())
}

/// Run a command, capture stdout/stderr, and require success.
fn run_command_capture(cmd: &str, args: &[&str], cwd: &Path) -> Result<Output> {
    let mut command = if cmd == "npm" {
        xtask::npm_command()
    } else {
        Command::new(cmd)
    };

    let output = command
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("Failed to execute: {} {}", cmd, args.join(" ")))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Command failed: {} {}\nstdout:\n{}\nstderr:\n{}",
            cmd,
            args.join(" "),
            stdout,
            stderr
        );
    }

    Ok(output)
}

fn assert_output_contains(output: &Output, needle: &str, message: &str) -> Result<()> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}\n{}", stdout, stderr);

    if !combined.contains(needle) {
        bail!(
            "{}\nMissing substring: '{}'\nCommand output:\n{}",
            message,
            needle,
            combined
        );
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

    #[test]
    fn test_create_test_plugin_args_include_output_mode() {
        let args = create_test_plugin_args("/tmp/test-plugin");

        assert!(args.contains(&"--output"));

        let output_idx = args
            .iter()
            .position(|arg| *arg == "--output")
            .expect("--output flag missing");
        assert_eq!(args.get(output_idx + 1), Some(&"/tmp/test-plugin"));
    }
}
