use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::process::Command;

use crate::template::{extract_template, variables::TemplateVariables};
use crate::validation::validate_crate_name;

/// Options for the `new` command.
#[derive(Debug)]
pub struct NewCommand {
    pub name: String,
    pub vendor: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub output: Option<PathBuf>,
    pub no_git: bool,
    pub sdk_version: String,
    pub local_sdk: bool,
}

impl NewCommand {
    pub fn execute(&self) -> Result<()> {
        // Validate plugin name
        validate_crate_name(&self.name)?;

        // Determine output directory
        let output_dir = self
            .output
            .clone()
            .unwrap_or_else(|| PathBuf::from(&self.name));

        if output_dir.exists() {
            anyhow::bail!(
                "Directory '{}' already exists. Please choose a different name or location.",
                output_dir.display()
            );
        }

        // Use defaults for missing fields
        let vendor = self
            .vendor
            .clone()
            .unwrap_or_else(|| "Your Company".to_string());
        let email = self.email.clone();
        let url = self.url.clone();

        // Resolve SDK path if --local-sdk is set
        let sdk_path = if self.local_sdk {
            Some(find_local_sdk_path()?)
        } else {
            None
        };

        // Create template variables
        let vars = TemplateVariables::new(
            self.name.clone(),
            vendor,
            email,
            url,
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

        let status = Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .context("Failed to run git init")?;

        if !status.status.success() {
            pb.finish_with_message("Git initialization failed (continuing...)");
            eprintln!(
                "{}",
                style("Warning: git init failed. You can initialize the repository manually.")
                    .yellow()
            );
        } else {
            pb.finish_with_message("Git repository initialized");
        }

        Ok(())
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
