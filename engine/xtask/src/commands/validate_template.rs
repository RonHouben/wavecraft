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

    let status = Command::new(cli_binary)
        .args([
            "create",
            "test-plugin",
            "--vendor",
            "Test Vendor",
            "--email",
            "test@example.com",
            "--url",
            "https://example.com",
            "--no-git",
        ])
        .current_dir(parent_dir)
        .status()
        .context("Failed to run wavecraft create")?;

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
    let ui_package_json = project_dir.join("ui/package.json");
    let engine_crates = workspace_root.join("engine/crates");
    let ui_packages = workspace_root.join("ui/packages");

    // Read the current Cargo.toml
    let content =
        fs::read_to_string(&engine_cargo_toml).context("Failed to read engine/Cargo.toml")?;

    let mut updated_lines = Vec::new();
    for line in content.lines() {
        if line
            .trim_start()
            .starts_with("wavecraft = { package = \"wavecraft-nih_plug\"")
        {
            updated_lines.push(format!(
                "wavecraft = {{ package = \"wavecraft-nih_plug\", path = \"{}/wavecraft-nih_plug\" }}",
                engine_crates.display()
            ));
            continue;
        }

        updated_lines.push(line.to_string());
    }

    fs::write(&engine_cargo_toml, updated_lines.join("\n"))
        .context("Failed to write engine/Cargo.toml")?;

    let ui_package_content =
        fs::read_to_string(&ui_package_json).context("Failed to read ui/package.json")?;
    let mut ui_package: serde_json::Value =
        serde_json::from_str(&ui_package_content).context("Failed to parse ui/package.json")?;

    let Some(dependencies) = ui_package
        .get_mut("dependencies")
        .and_then(serde_json::Value::as_object_mut)
    else {
        bail!("Missing dependencies object in ui/package.json");
    };

    dependencies.insert(
        "@wavecraft/core".to_string(),
        serde_json::Value::String(format!("file:{}", ui_packages.join("core").display())),
    );
    dependencies.insert(
        "@wavecraft/components".to_string(),
        serde_json::Value::String(format!("file:{}", ui_packages.join("components").display())),
    );

    let serialized_ui_package = serde_json::to_string_pretty(&ui_package)
        .context("Failed to serialize updated ui/package.json")?;
    fs::write(&ui_package_json, format!("{}\n", serialized_ui_package))
        .context("Failed to write ui/package.json")?;

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
        println!("Testing xtask bundle command (dry run)...");
    }

    run_command("cargo", &["xtask", "bundle", "--check"], &engine_dir)?;

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
