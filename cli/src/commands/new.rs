use anyhow::{Context, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
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
    /// Local SDK path for development (generates path deps instead of git deps)
    pub local_dev: Option<PathBuf>,
}

impl NewCommand {
    pub fn execute(&self) -> Result<()> {
        // Validate plugin name
        validate_crate_name(&self.name)?;
        
        // Determine output directory
        let output_dir = self.output.clone().unwrap_or_else(|| PathBuf::from(&self.name));
        
        if output_dir.exists() {
            anyhow::bail!(
                "Directory '{}' already exists. Please choose a different name or location.",
                output_dir.display()
            );
        }
        
        // Interactive prompts for missing fields
        let vendor = self.get_vendor()?;
        let email = self.get_email()?;
        let url = self.get_url()?;
        
        // Create template variables
        let vars = TemplateVariables::new(
            self.name.clone(),
            vendor,
            email,
            url,
            self.sdk_version.clone(),
            self.local_dev.clone(),
        );
        
        // Extract template with progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
        );
        pb.set_message("Creating project...");
        
        extract_template(&output_dir, &vars)
            .context("Failed to extract template")?;
        
        pb.finish_with_message("Project created");
        
        // Initialize git repository (unless --no-git)
        if !self.no_git {
            self.init_git(&output_dir)?;
        }
        
        // Success message
        println!();
        println!("{}", style("✓ Plugin project created successfully!").green().bold());
        println!();
        println!("Next steps:");
        println!("  cd {}", self.name);
        println!("  cargo xtask dev    # Start development servers");
        println!();
        println!("Documentation: https://github.com/RonHouben/wavecraft/tree/main/docs");
        
        Ok(())
    }
    
    fn get_vendor(&self) -> Result<String> {
        if let Some(vendor) = &self.vendor {
            return Ok(vendor.clone());
        }
        
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Vendor name (company or developer name)")
            .interact_text()
            .context("Failed to read vendor input")
    }
    
    fn get_email(&self) -> Result<Option<String>> {
        if self.email.is_some() {
            return Ok(self.email.clone());
        }
        
        let email: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Email (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to read email input")?;
        
        Ok(if email.is_empty() { None } else { Some(email) })
    }
    
    fn get_url(&self) -> Result<Option<String>> {
        if self.url.is_some() {
            return Ok(self.url.clone());
        }
        
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("URL (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to read URL input")?;
        
        Ok(if url.is_empty() { None } else { Some(url) })
    }
    
    fn init_git(&self, dir: &PathBuf) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
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
