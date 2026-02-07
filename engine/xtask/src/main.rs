//! Wavecraft xtask build system.
//!
//! This provides a unified Rust-based build tool for the Wavecraft audio plugin project.
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

use xtask::{BuildMode, output};

/// Wavecraft build system - Build, test, and install audio plugins.
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
        /// Package name to bundle (default: wavecraft)
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

        /// Run UI tests only
        #[arg(long)]
        ui: bool,

        /// Run engine tests only
        #[arg(long)]
        engine: bool,
    },

    /// Build and run the desktop POC application
    #[command(about = "Build and run the desktop POC")]
    Desktop {
        /// Also rebuild the React UI
        #[arg(long)]
        build_ui: bool,
    },

    /// Build the React UI
    #[command(about = "Build the React UI")]
    BuildUi,

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

    /// Sign plugin bundles for macOS distribution
    #[command(about = "Sign plugin bundles for macOS distribution")]
    Sign {
        /// Signing identity (overrides APPLE_SIGNING_IDENTITY)
        #[arg(long)]
        identity: Option<String>,

        /// Path to entitlements.plist
        #[arg(long)]
        entitlements: Option<String>,

        /// Use ad-hoc signing (for local development)
        #[arg(long)]
        adhoc: bool,

        /// Verify signatures only (no signing)
        #[arg(long)]
        verify: bool,
    },

    /// Notarize plugin bundles with Apple
    #[command(about = "Notarize plugin bundles with Apple")]
    Notarize {
        /// Submit bundles for notarization
        #[arg(long)]
        submit: bool,

        /// Check notarization status
        #[arg(long)]
        status: bool,

        /// Staple ticket to bundles
        #[arg(long)]
        staple: bool,

        /// Full workflow (submit, wait, staple)
        #[arg(long)]
        full: bool,
    },

    /// Build, sign, and notarize for release
    #[command(about = "Build, sign, and notarize for release")]
    Release {
        /// Skip notarization (sign only)
        #[arg(long)]
        skip_notarize: bool,
    },

    /// Run linters for UI and/or engine code
    #[command(about = "Run linters for UI and/or engine code")]
    Lint {
        /// Run UI linting only (ESLint + Prettier)
        #[arg(long)]
        ui: bool,

        /// Run engine linting only (Clippy + fmt)
        #[arg(long)]
        engine: bool,

        /// Auto-fix issues where possible
        #[arg(long)]
        fix: bool,
    },

    /// Run dev servers (WebSocket + UI) for development
    #[command(about = "Run dev servers (WebSocket + UI) for development")]
    Dev {
        /// WebSocket server port
        #[arg(long, default_value = "9000")]
        port: u16,
    },

    /// Pre-push validation (fast local CI simulation)
    #[command(
        about = "Pre-push validation - runs linting and automated tests",
        name = "ci-check"
    )]
    Check {
        /// Auto-fix linting issues where possible
        #[arg(long)]
        fix: bool,

        /// Skip linting
        #[arg(long)]
        skip_lint: bool,

        /// Skip automated tests
        #[arg(long)]
        skip_tests: bool,
    },

    /// Validate CLI template generation
    #[command(
        about = "Validate CLI template generation (replicates CI workflow)",
        name = "ci-validate-template"
    )]
    ValidateTemplate {
        /// Keep the generated test project (don't clean up)
        #[arg(long)]
        keep: bool,
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
        Some(Commands::Bundle {
            package,
            features,
            install,
        }) => {
            let features_vec: Vec<&str> = features
                .as_deref()
                .map(|f| f.split(',').collect())
                .unwrap_or_default();
            commands::bundle::run_with_features(
                mode,
                package.as_deref(),
                &features_vec,
                cli.verbose,
            )?;

            if install {
                commands::install::run(cli.dry_run, cli.verbose)?;
            }
            Ok(())
        }
        Some(Commands::Test {
            package,
            all,
            ui,
            engine,
        }) => {
            let packages = if package.is_empty() {
                None
            } else {
                Some(package)
            };
            commands::test::run(packages, all, ui, engine, cli.verbose)
        }
        Some(Commands::Desktop { build_ui }) => {
            commands::desktop::run(!cli.debug, build_ui, cli.verbose)
        }
        Some(Commands::BuildUi) => commands::build_ui::run(cli.verbose),
        Some(Commands::Au) => commands::au::run(cli.dry_run, cli.verbose),
        Some(Commands::Install) => commands::install::run(cli.dry_run, cli.verbose),
        Some(Commands::Clean { installed, force }) => {
            commands::clean::run(installed, force, cli.dry_run, cli.verbose)
        }
        Some(Commands::All {
            skip_tests,
            skip_au,
        }) => commands::run_all(mode, skip_tests, skip_au, cli.dry_run, cli.verbose),
        Some(Commands::Sign {
            identity,
            entitlements,
            adhoc,
            verify,
        }) => {
            if verify {
                commands::sign::run_verify(cli.verbose)
            } else if adhoc {
                commands::sign::run_adhoc()
            } else {
                let config = if let Some(id) = identity {
                    commands::sign::SigningConfig {
                        identity: id,
                        entitlements,
                        verbose: cli.verbose,
                    }
                } else {
                    let mut config = commands::sign::SigningConfig::from_env()?;
                    config.verbose = cli.verbose;
                    if entitlements.is_some() {
                        config.entitlements = entitlements;
                    }
                    config
                };
                commands::sign::run(config)
            }
        }
        Some(Commands::Notarize {
            submit,
            status,
            staple,
            full,
        }) => {
            let action = if submit {
                commands::notarize::NotarizeAction::Submit
            } else if status {
                commands::notarize::NotarizeAction::Status
            } else if staple {
                commands::notarize::NotarizeAction::Staple
            } else if full {
                commands::notarize::NotarizeAction::Full
            } else {
                anyhow::bail!("Must specify one of: --submit, --status, --staple, or --full");
            };

            let mut config = commands::notarize::NotarizeConfig::from_env()?;
            config.verbose = cli.verbose;
            commands::notarize::run(action, config)
        }
        Some(Commands::Release { skip_notarize }) => {
            commands::release::run(skip_notarize, cli.verbose)
        }
        Some(Commands::Lint { ui, engine, fix }) => {
            let targets = if !ui && !engine {
                // Neither specified = run both
                commands::lint::LintTargets {
                    ui: true,
                    engine: true,
                    fix,
                }
            } else {
                commands::lint::LintTargets { ui, engine, fix }
            };
            commands::lint::run(targets, cli.verbose)
        }
        Some(Commands::Dev { port }) => commands::dev::run(port, cli.verbose),
        Some(Commands::Check {
            fix,
            skip_lint,
            skip_tests,
        }) => {
            let config = commands::check::CheckConfig {
                fix,
                skip_lint,
                skip_tests,
                verbose: cli.verbose,
            };
            commands::check::run(config)
        }
        Some(Commands::ValidateTemplate { keep }) => {
            let config = commands::validate_template::ValidateTemplateConfig {
                verbose: cli.verbose,
                keep,
            };
            commands::validate_template::run(config)
        }
        None => {
            // Default behavior: run nih_plug_xtask for backward compatibility
            // This handles `cargo xtask bundle wavecraft --release` style invocations
            nih_plug_xtask::main()
        }
    };

    if let Err(e) = result {
        output::print_error(&format!("Error: {:#}", e));
        std::process::exit(1);
    }

    Ok(())
}
