use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::process::{Command, Output};

use crate::sdk_detect;
use crate::template::{extract_template, variables::TemplateVariables};
use crate::validation::validate_crate_name;

/// Options for the `create` command.
#[derive(Debug)]
pub struct CreateCommand {
    pub name: String,
    pub vendor: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub output: Option<PathBuf>,
    pub no_git: bool,
    pub sdk_version: String,
    pub local_sdk: bool,
}

impl CreateCommand {
    pub fn execute(&self) -> Result<()> {
        // Validate plugin name
        validate_crate_name(&self.name)?;

        // Determine output directory
        let output_dir = self.resolve_output_dir();

        if output_dir.exists() {
            anyhow::bail!(
                "Directory '{}' already exists. Please choose a different name or location.",
                output_dir.display()
            );
        }

        // Use defaults for missing fields
        let (author_name, author_email, homepage) = self.default_author_metadata();

        // Resolve SDK path:
        // 1. Explicit --local-sdk flag (manual override)
        // 2. Auto-detect if running from SDK repo (via cargo run)
        // 3. None → use git tag dependencies
        let sdk_path = if self.local_sdk {
            Some(find_local_sdk_path()?)
        } else if let Some(detected_path) = sdk_detect::detect_sdk_repo() {
            println!();
            println!(
                "{} {}",
                style("ℹ").cyan().bold(),
                style("Detected SDK development mode (running from source checkout)").cyan()
            );
            println!(
                "  {} Using local path dependencies instead of git tags",
                style("→").dim()
            );
            println!(
                "  {} To force git tag mode, install via: cargo install wavecraft",
                style("→").dim()
            );
            println!();
            Some(detected_path)
        } else {
            None
        };

        // Create template variables
        let vars = TemplateVariables::new(
            self.name.clone(),
            author_name,
            author_email,
            homepage,
            self.sdk_version.clone(),
            sdk_path,
        );

        // Extract template with progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Creating project...");

        extract_template(&output_dir, &vars).context("Failed to extract template")?;

        pb.finish_with_message("Project created");

        // Initialize git repository (unless --no-git)
        if !self.no_git {
            self.init_git(&output_dir)?;
        }

        // Success message
        println!();
        println!(
            "{}",
            style("✓ Plugin project created successfully!")
                .green()
                .bold()
        );
        println!();
        println!("Next steps:");
        println!("  cd {}", self.name);
        println!("  wavecraft bundle --install    # Build + install VST3 for DAW testing");
        println!("  wavecraft start    # Start development servers");
        println!();
        println!("Documentation: https://github.com/RonHouben/wavecraft/tree/main/docs");

        Ok(())
    }

    fn init_git(&self, dir: &PathBuf) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Initializing git repository...");

        let status = self.run_git_init(dir)?;

        if !Self::git_init_succeeded(&status) {
            pb.finish_with_message("Git initialization failed (continuing...)");
            eprintln!(
                "{}",
                style("Warning: git init failed. You can initialize the repository manually.")
                    .yellow()
            );
        } else {
            pb.set_message("Creating initial git commit...");

            let add_status = Command::new("git")
                .args(["add", "."])
                .current_dir(dir)
                .output()
                .context("Failed to run git add")?;

            if !add_status.status.success() {
                pb.finish_with_message("Git staging failed (continuing...)");
                eprintln!(
                    "{}",
                    style("Warning: git add failed. You can stage files manually.").yellow()
                );
                return Ok(());
            }

            let commit_status = Command::new("git")
                .args(["commit", "-m", "Initial commit"])
                .current_dir(dir)
                .output()
                .context("Failed to run git commit")?;

            if !commit_status.status.success() {
                pb.finish_with_message("Initial commit failed (continuing...)");
                eprintln!(
                    "{}",
                    style(
                        "Warning: git commit failed. Configure git (name/email) or commit manually."
                    )
                    .yellow()
                );
            } else {
                pb.finish_with_message("Git repository initialized with initial commit");
            }
        }

        Ok(())
    }

    fn resolve_output_dir(&self) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(&self.name))
    }

    fn default_author_metadata(&self) -> (String, String, String) {
        (
            self.vendor
                .clone()
                .unwrap_or_else(|| "Your Name".to_string()),
            self.email
                .clone()
                .unwrap_or_else(|| "your.email@example.com".to_string()),
            self.url
                .clone()
                .unwrap_or_else(|| "https://yourproject.com".to_string()),
        )
    }

    fn run_git_init(&self, dir: &PathBuf) -> Result<Output> {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .context("Failed to run git init")
    }

    fn git_init_succeeded(output: &Output) -> bool {
        output.status.success()
    }
}

fn find_local_sdk_path() -> Result<PathBuf> {
    let sdk_path = PathBuf::from("engine/crates");

    if !sdk_path.exists() {
        anyhow::bail!(
            "Error: --local-sdk must be run from the wavecraft repository root.\n\
             Could not find: engine/crates"
        );
    }

    sdk_path
        .canonicalize()
        .context("Failed to resolve SDK path")
}
