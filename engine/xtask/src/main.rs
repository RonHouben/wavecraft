//! VstKit xtask build system.
//!
//! This provides a unified Rust-based build tool for the VstKit audio plugin project.
//!
//! # Usage
//!
//! ```bash
//! cargo xtask <COMMAND>
//! ```
//!
//! # Commands
//!
//! - `bundle` - Build and bundle VST3/CLAP plugins
//! - `test` - Run unit tests
//! - `desktop` - Build and run the desktop POC
//! - `au` - Build AU wrapper (macOS only)
//! - `install` - Install plugins to system directories
//! - `clean` - Clean build artifacts
//! - `all` - Run full build pipeline

use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

use xtask::{output, BuildMode};

/// VstKit build system - Build, test, and install audio plugins.
#[derive(Parser)]
#[command(name = "xtask")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Build in release mode (default)
    #[arg(long, global = true, conflicts_with = "debug")]
    release: bool,

    /// Build in debug mode
    #[arg(long, global = true)]
    debug: bool,

    /// Show detailed output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Show what would be done without executing
    #[arg(long, global = true)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Build and bundle VST3/CLAP plugins
    #[command(about = "Build and bundle VST3/CLAP plugins")]
    Bundle {
        /// Package name to bundle (default: vstkit)
        #[arg(short, long)]
        package: Option<String>,
        
        /// Features to enable (comma-separated)
        #[arg(short, long)]
        features: Option<String>,
        
        /// Install plugins after building
        #[arg(short, long)]
        install: bool,
    },

    /// Run unit tests for specified crates
    #[command(about = "Run unit tests")]
    Test {
        /// Packages to test (default: dsp, protocol)
        #[arg(short, long)]
        package: Vec<String>,

        /// Test all workspace crates
        #[arg(long)]
        all: bool,
    },

    /// Build and run the desktop POC application
    #[command(about = "Build and run the desktop POC")]
    Desktop {
        /// Also rebuild the React UI
        #[arg(long)]
        build_ui: bool,
    },

    /// Build AU wrapper (macOS only)
    #[command(about = "Build AU wrapper (macOS only)")]
    Au,

    /// Install plugins to system directories
    #[command(about = "Install plugins to system directories")]
    Install,

    /// Clean build artifacts
    #[command(about = "Clean build artifacts")]
    Clean {
        /// Also remove installed plugins
        #[arg(long)]
        installed: bool,

        /// Skip confirmation when removing installed plugins
        #[arg(long)]
        force: bool,
    },

    /// Run full build pipeline (test → bundle → au → install)
    #[command(about = "Run full build pipeline")]
    All {
        /// Skip running tests
        #[arg(long)]
        skip_tests: bool,

        /// Skip AU build (macOS)
        #[arg(long)]
        skip_au: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine build mode
    let mode = if cli.debug {
        BuildMode::Debug
    } else {
        BuildMode::Release
    };

    // If no command specified, default to showing help or running bundle
    let result = match cli.command {
        Some(Commands::Bundle { package, features, install }) => {
            let features_vec: Vec<&str> = features
                .as_deref()
                .map(|f| f.split(',').collect())
                .unwrap_or_default();
            commands::bundle::run_with_features(mode, package.as_deref(), &features_vec, cli.verbose)?;
            
            if install {
                commands::install::run(cli.dry_run, cli.verbose)?;
            }
            Ok(())
        }
        Some(Commands::Test { package, all }) => {
            let packages = if package.is_empty() {
                None
            } else {
                Some(package)
            };
            commands::test::run(packages, all, cli.verbose)
        }
        Some(Commands::Desktop { build_ui }) => {
            commands::desktop::run(!cli.debug, build_ui, cli.verbose)
        }
        Some(Commands::Au) => commands::au::run(cli.dry_run, cli.verbose),
        Some(Commands::Install) => commands::install::run(cli.dry_run, cli.verbose),
        Some(Commands::Clean { installed, force }) => {
            commands::clean::run(installed, force, cli.dry_run, cli.verbose)
        }
        Some(Commands::All {
            skip_tests,
            skip_au,
        }) => commands::run_all(mode, skip_tests, skip_au, cli.dry_run, cli.verbose),
        None => {
            // Default behavior: run nih_plug_xtask for backward compatibility
            // This handles `cargo xtask bundle vstkit --release` style invocations
            nih_plug_xtask::main()
        }
    };

    if let Err(e) = result {
        output::print_error(&format!("Error: {:#}", e));
        std::process::exit(1);
    }

    Ok(())
}
