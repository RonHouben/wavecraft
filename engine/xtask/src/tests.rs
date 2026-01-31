//! Integration tests for the xtask CLI.
//!
//! These tests verify CLI argument parsing and command structure
//! without actually executing the underlying operations.

#[cfg(test)]
mod cli_tests {
    use clap::Parser;

    /// CLI structure mirror for testing (must match main.rs)
    #[derive(Parser, Debug)]
    #[command(name = "xtask")]
    struct TestCli {
        #[command(subcommand)]
        command: Option<TestCommands>,

        #[arg(long, global = true, conflicts_with = "debug")]
        release: bool,

        #[arg(long, global = true)]
        debug: bool,

        #[arg(short, long, global = true)]
        verbose: bool,

        #[arg(long, global = true)]
        dry_run: bool,
    }

    #[derive(clap::Subcommand, Debug)]
    enum TestCommands {
        Bundle {
            #[arg(short, long)]
            package: Option<String>,
        },
        Test {
            #[arg(short, long)]
            package: Vec<String>,
            #[arg(long)]
            all: bool,
        },
        Au,
        Install,
        Clean {
            #[arg(long)]
            installed: bool,
            #[arg(long)]
            force: bool,
        },
        All {
            #[arg(long)]
            skip_tests: bool,
            #[arg(long)]
            skip_au: bool,
        },
    }

    // Helper to parse CLI args for testing
    fn parse_args(args: &[&str]) -> Result<TestCli, clap::Error> {
        let mut full_args = vec!["xtask"];
        full_args.extend(args);
        TestCli::try_parse_from(full_args)
    }

    // =========================================================================
    // Basic Parsing Tests
    // =========================================================================

    #[test]
    fn test_no_args_parses() {
        let cli = parse_args(&[]).expect("Failed to parse empty args - should never fail");
        assert!(cli.command.is_none());
        assert!(!cli.verbose);
        assert!(!cli.dry_run);
        assert!(!cli.debug);
        assert!(!cli.release);
    }

    #[test]
    fn test_global_verbose_flag() {
        let cli = parse_args(&["--verbose"]).expect("Failed to parse --verbose flag");
        assert!(cli.verbose);
    }

    #[test]
    fn test_global_verbose_short_flag() {
        let cli = parse_args(&["-v"]).expect("Failed to parse -v flag");
        assert!(cli.verbose);
    }

    #[test]
    fn test_global_dry_run_flag() {
        let cli = parse_args(&["--dry-run"]).expect("Failed to parse --dry-run flag");
        assert!(cli.dry_run);
    }

    #[test]
    fn test_global_debug_flag() {
        let cli = parse_args(&["--debug"]).expect("Failed to parse --debug flag");
        assert!(cli.debug);
        assert!(!cli.release);
    }

    #[test]
    fn test_global_release_flag() {
        let cli = parse_args(&["--release"]).expect("Failed to parse --release flag");
        assert!(cli.release);
        assert!(!cli.debug);
    }

    #[test]
    fn test_debug_and_release_conflict() {
        let result = parse_args(&["--debug", "--release"]);
        assert!(result.is_err());
    }

    // =========================================================================
    // Bundle Command Tests
    // =========================================================================

    #[test]
    fn test_bundle_command_default() {
        let cli = parse_args(&["bundle"]).expect("Failed to parse bundle command");
        match cli.command {
            Some(TestCommands::Bundle { package }) => {
                assert!(package.is_none());
            }
            _ => panic!("Expected Bundle command"),
        }
    }

    #[test]
    fn test_bundle_command_with_package() {
        let cli = parse_args(&["bundle", "--package", "my-plugin"])
            .expect("Failed to parse bundle --package");
        match cli.command {
            Some(TestCommands::Bundle { package }) => {
                assert_eq!(package.as_deref(), Some("my-plugin"));
            }
            _ => panic!("Expected Bundle command"),
        }
    }

    #[test]
    fn test_bundle_command_with_short_package() {
        let cli = parse_args(&["bundle", "-p", "my-plugin"]).expect("Failed to parse bundle -p");
        match cli.command {
            Some(TestCommands::Bundle { package }) => {
                assert_eq!(package.as_deref(), Some("my-plugin"));
            }
            _ => panic!("Expected Bundle command"),
        }
    }

    #[test]
    fn test_bundle_with_debug_mode() {
        let cli = parse_args(&["bundle", "--debug"]).expect("Failed to parse bundle --debug");
        assert!(cli.debug);
        match cli.command {
            Some(TestCommands::Bundle { .. }) => {}
            _ => panic!("Expected Bundle command"),
        }
    }

    // =========================================================================
    // Test Command Tests
    // =========================================================================

    #[test]
    fn test_test_command_default() {
        let cli = parse_args(&["test"]).expect("Failed to parse test command");
        match cli.command {
            Some(TestCommands::Test { package, all }) => {
                assert!(package.is_empty());
                assert!(!all);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_test_command_with_all() {
        let cli = parse_args(&["test", "--all"]).expect("Failed to parse test --all");
        match cli.command {
            Some(TestCommands::Test { all, .. }) => {
                assert!(all);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_test_command_with_packages() {
        let cli = parse_args(&["test", "-p", "dsp", "-p", "protocol"])
            .expect("Failed to parse test -p flags");
        match cli.command {
            Some(TestCommands::Test { package, .. }) => {
                assert_eq!(package, vec!["dsp", "protocol"]);
            }
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_test_command_verbose() {
        let cli = parse_args(&["test", "--verbose"]).expect("Failed to parse test --verbose");
        assert!(cli.verbose);
        assert!(matches!(cli.command, Some(TestCommands::Test { .. })));
    }

    // =========================================================================
    // AU Command Tests
    // =========================================================================

    #[test]
    fn test_au_command() {
        let cli = parse_args(&["au"]).expect("Failed to parse au command");
        assert!(matches!(cli.command, Some(TestCommands::Au)));
    }

    #[test]
    fn test_au_command_with_dry_run() {
        let cli = parse_args(&["au", "--dry-run"]).expect("Failed to parse au --dry-run");
        assert!(cli.dry_run);
        assert!(matches!(cli.command, Some(TestCommands::Au)));
    }

    // =========================================================================
    // Install Command Tests
    // =========================================================================

    #[test]
    fn test_install_command() {
        let cli = parse_args(&["install"]).expect("Failed to parse install command");
        assert!(matches!(cli.command, Some(TestCommands::Install)));
    }

    #[test]
    fn test_install_command_with_dry_run() {
        let cli = parse_args(&["install", "--dry-run"]).expect("Failed to parse install --dry-run");
        assert!(cli.dry_run);
        assert!(matches!(cli.command, Some(TestCommands::Install)));
    }

    // =========================================================================
    // Clean Command Tests
    // =========================================================================

    #[test]
    fn test_clean_command_default() {
        let cli = parse_args(&["clean"]).expect("Failed to parse clean command");
        match cli.command {
            Some(TestCommands::Clean { installed, force }) => {
                assert!(!installed);
                assert!(!force);
            }
            _ => panic!("Expected Clean command"),
        }
    }

    #[test]
    fn test_clean_command_with_installed() {
        let cli = parse_args(&["clean", "--installed"]).expect("Failed to parse clean --installed");
        match cli.command {
            Some(TestCommands::Clean { installed, force }) => {
                assert!(installed);
                assert!(!force);
            }
            _ => panic!("Expected Clean command"),
        }
    }

    #[test]
    fn test_clean_command_with_installed_and_force() {
        let cli = parse_args(&["clean", "--installed", "--force"])
            .expect("Failed to parse clean --installed --force");
        match cli.command {
            Some(TestCommands::Clean { installed, force }) => {
                assert!(installed);
                assert!(force);
            }
            _ => panic!("Expected Clean command"),
        }
    }

    #[test]
    fn test_clean_command_with_dry_run() {
        let cli = parse_args(&["clean", "--dry-run"]).expect("Failed to parse clean --dry-run");
        assert!(cli.dry_run);
        assert!(matches!(cli.command, Some(TestCommands::Clean { .. })));
    }

    // =========================================================================
    // All Command Tests
    // =========================================================================

    #[test]
    fn test_all_command_default() {
        let cli = parse_args(&["all"]).expect("Failed to parse all command");
        match cli.command {
            Some(TestCommands::All {
                skip_tests,
                skip_au,
            }) => {
                assert!(!skip_tests);
                assert!(!skip_au);
            }
            _ => panic!("Expected All command"),
        }
    }

    #[test]
    fn test_all_command_skip_tests() {
        let cli = parse_args(&["all", "--skip-tests"]).expect("Failed to parse all --skip-tests");
        match cli.command {
            Some(TestCommands::All { skip_tests, .. }) => {
                assert!(skip_tests);
            }
            _ => panic!("Expected All command"),
        }
    }

    #[test]
    fn test_all_command_skip_au() {
        let cli = parse_args(&["all", "--skip-au"]).expect("Failed to parse all --skip-au");
        match cli.command {
            Some(TestCommands::All { skip_au, .. }) => {
                assert!(skip_au);
            }
            _ => panic!("Expected All command"),
        }
    }

    #[test]
    fn test_all_command_with_dry_run() {
        let cli = parse_args(&["all", "--dry-run"]).expect("Failed to parse all --dry-run");
        assert!(cli.dry_run);
        assert!(matches!(cli.command, Some(TestCommands::All { .. })));
    }

    #[test]
    fn test_all_command_full_options() {
        let cli = parse_args(&["all", "--skip-tests", "--skip-au", "--dry-run", "--verbose"])
            .expect("Failed to parse all with full options");
        assert!(cli.dry_run);
        assert!(cli.verbose);
        match cli.command {
            Some(TestCommands::All {
                skip_tests,
                skip_au,
            }) => {
                assert!(skip_tests);
                assert!(skip_au);
            }
            _ => panic!("Expected All command"),
        }
    }

    // =========================================================================
    // Invalid Input Tests
    // =========================================================================

    #[test]
    fn test_unknown_command_fails() {
        let result = parse_args(&["unknown"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_flag_fails() {
        let result = parse_args(&["bundle", "--unknown-flag"]);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod lib_tests {
    use crate::{BuildMode, PLUGIN_NAME, Platform};

    // =========================================================================
    // BuildMode Tests
    // =========================================================================

    #[test]
    fn test_build_mode_default_is_release() {
        let mode = BuildMode::default();
        assert_eq!(mode, BuildMode::Release);
    }

    #[test]
    fn test_build_mode_cargo_flags() {
        assert_eq!(BuildMode::Debug.cargo_flag(), None);
        assert_eq!(BuildMode::Release.cargo_flag(), Some("--release"));
        assert_eq!(
            BuildMode::ReleaseDebug.cargo_flag(),
            Some("--profile=release-debug")
        );
    }

    #[test]
    fn test_build_mode_target_dirs() {
        assert_eq!(BuildMode::Debug.target_dir(), "debug");
        assert_eq!(BuildMode::Release.target_dir(), "release");
        assert_eq!(BuildMode::ReleaseDebug.target_dir(), "release-debug");
    }

    // =========================================================================
    // Platform Tests
    // =========================================================================

    #[test]
    fn test_platform_current_returns_valid() {
        let platform = Platform::current();
        // Should match one of the known platforms
        assert!(matches!(
            platform,
            Platform::MacOS | Platform::Windows | Platform::Linux
        ));
    }

    #[test]
    fn test_platform_is_macos() {
        assert!(Platform::MacOS.is_macos());
        assert!(!Platform::Windows.is_macos());
        assert!(!Platform::Linux.is_macos());
    }

    // =========================================================================
    // Constants Tests
    // =========================================================================

    #[test]
    fn test_plugin_name_is_defined() {
        assert!(!PLUGIN_NAME.is_empty());
        assert_eq!(PLUGIN_NAME, "vstkit");
    }
}

#[cfg(test)]
mod path_tests {
    use crate::paths;

    #[test]
    fn test_project_root_exists() {
        // This test may fail if run from a different context
        // but should work within cargo test
        if let Ok(root) = paths::project_root() {
            assert!(root.exists() || root.to_string_lossy().contains("vstkit"));
        }
    }

    #[test]
    fn test_engine_dir_is_child_of_root() {
        if let (Ok(root), Ok(engine)) = (paths::project_root(), paths::engine_dir()) {
            assert!(engine.starts_with(&root));
            assert!(engine.ends_with("engine"));
        }
    }

    #[test]
    fn test_bundled_dir_is_in_target() {
        if let Ok(bundled) = paths::bundled_dir() {
            assert!(bundled.to_string_lossy().contains("target"));
            assert!(bundled.ends_with("bundled"));
        }
    }

    #[test]
    fn test_vst3_install_dir_is_platform_specific() {
        if let Ok(vst3_dir) = paths::vst3_install_dir() {
            let path_str = vst3_dir.to_string_lossy();

            #[cfg(target_os = "macos")]
            assert!(path_str.contains("Library/Audio/Plug-Ins/VST3"));

            #[cfg(target_os = "windows")]
            assert!(path_str.contains("VST3"));

            #[cfg(target_os = "linux")]
            assert!(path_str.contains(".vst3"));
        }
    }

    #[test]
    fn test_clap_install_dir_is_platform_specific() {
        if let Ok(clap_dir) = paths::clap_install_dir() {
            let path_str = clap_dir.to_string_lossy();

            #[cfg(target_os = "macos")]
            assert!(path_str.contains("Library/Audio/Plug-Ins/CLAP"));

            #[cfg(target_os = "windows")]
            assert!(path_str.contains("CLAP"));

            #[cfg(target_os = "linux")]
            assert!(path_str.contains(".clap"));
        }
    }

    #[test]
    fn test_au_wrapper_dir_macos_only() {
        if let Ok(au_dir) = paths::au_wrapper_dir() {
            assert!(au_dir.to_string_lossy().contains("au-wrapper"));
        }
    }
}
