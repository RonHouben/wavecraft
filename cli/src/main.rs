mod commands;
mod project;
mod sdk_detect;
mod template;
mod validation;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use commands::{CreateCommand, StartCommand};

/// SDK version derived from CLI package version at compile time.
/// Used for git tag dependencies in generated projects.
/// Format: "wavecraft-cli-v{version}" to match repository tag convention.
const SDK_VERSION: &str = concat!("wavecraft-cli-v", env!("CARGO_PKG_VERSION"));

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
    /// Create a new plugin project from the Wavecraft template
    #[command(
        long_about = "Scaffold a new audio plugin project with Rust engine and React UI.\n\n\
        The generated project includes a complete build system (xtask), \
        development servers, and example DSP code ready to customize."
    )]
    Create {
        /// Plugin name (lowercase, alphanumeric + underscore/hyphen)
        name: String,

        /// Vendor name for plugin metadata (company or developer name)
        #[arg(short, long)]
        vendor: Option<String>,

        /// Contact email for plugin metadata
        #[arg(short, long)]
        email: Option<String>,

        /// Website URL for plugin metadata
        #[arg(short, long)]
        url: Option<String>,

        /// Output directory (defaults to plugin name)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Skip git repository initialization
        #[arg(long)]
        no_git: bool,

        /// Use local SDK from repository (for SDK development only)
        #[arg(long, hide = true)]
        local_sdk: bool,
    },

    /// Start development servers (WebSocket + UI)
    #[command(
        long_about = "Launch the Rust WebSocket server and Vite UI dev server \
        for browser-based plugin UI development with hot module reloading."
    )]
    Start {
        /// WebSocket server port for engine communication
        #[arg(short, long, default_value = "9000")]
        port: u16,

        /// Vite UI dev server port
        #[arg(long, default_value = "5173")]
        ui_port: u16,

        /// Auto-install npm dependencies without prompting
        #[arg(long)]
        install: bool,

        /// Fail if node_modules is missing (CI mode, no prompts)
        #[arg(long)]
        no_install: bool,

        /// Show verbose output from servers
        #[arg(short, long)]
        verbose: bool,
    },

    /// Update the CLI and project dependencies (Rust crates + npm packages)
    #[command(
        long_about = "Update the Wavecraft CLI to the latest version, then update Rust crates \
        and npm packages if run from a plugin project directory.\n\n\
        Can be run from any directory. When outside a project, only the CLI is updated."
    )]
    Update,

    /// Extract parameters from a plugin dylib (hidden — internal use only)
    #[command(hide = true)]
    ExtractParams {
        /// Path to the plugin dylib
        dylib_path: std::path::PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            name,
            vendor,
            email,
            url,
            output,
            no_git,
            local_sdk,
        } => {
            let cmd = CreateCommand {
                name,
                vendor,
                email,
                url,
                output,
                no_git,
                sdk_version: SDK_VERSION.to_string(),
                local_sdk,
            };
            cmd.execute()?;
        }

        Commands::Start {
            port,
            ui_port,
            install,
            no_install,
            verbose,
        } => {
            let cmd = StartCommand {
                port,
                ui_port,
                install,
                no_install,
                verbose,
            };
            cmd.execute()?;
        }

        Commands::Update => {
            commands::update::run()?;
        }

        Commands::ExtractParams { dylib_path } => {
            commands::extract_params::execute(dylib_path)?;
        }
    }

    Ok(())
}
