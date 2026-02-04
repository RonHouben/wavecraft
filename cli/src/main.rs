mod commands;
mod template;
mod validation;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use commands::NewCommand;

#[derive(Parser)]
#[command(
    name = "wavecraft",
    version,
    about = "Wavecraft SDK - Audio plugin development toolkit",
    long_about = "Create, build, and manage audio plugins (VST3/CLAP) with Rust + React UI."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new plugin project from the template
    New {
        /// Plugin name (lowercase, alphanumeric + underscore/hyphen)
        name: String,
        
        /// Vendor name (company or developer name)
        #[arg(short, long)]
        vendor: Option<String>,
        
        /// Contact email (optional)
        #[arg(short, long)]
        email: Option<String>,
        
        /// Website URL (optional)
        #[arg(short, long)]
        url: Option<String>,
        
        /// Output directory (defaults to plugin name)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Skip git initialization
        #[arg(long)]
        no_git: bool,
        
        /// Wavecraft SDK version to use (git tag)
        #[arg(long, default_value = "v0.7.0")]
        sdk_version: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New {
            name,
            vendor,
            email,
            url,
            output,
            no_git,
            sdk_version,
        } => {
            let cmd = NewCommand {
                name,
                vendor,
                email,
                url,
                output,
                no_git,
                sdk_version,
            };
            cmd.execute()?;
        }
    }
    
    Ok(())
}
