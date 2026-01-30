# Implementation Plan: Unified Rust xtask Build System

## Overview

Migrate the build orchestration from `scripts/build.sh` to a fully Rust-based `xtask` implementation. This will provide cross-platform build support, type-safe argument parsing, better error handling, and alignment with Rust ecosystem conventions.

## Requirements

- Replace all `build.sh` functionality with Rust code in `xtask`
- Support all existing commands: `--clean`, `--release`, `--debug`, `--install`, `--test`, `--au`, `--all`
- Maintain compatibility with `nih_plug_xtask` for VST3/CLAP bundling
- Cross-platform support (macOS primary, Windows secondary, Linux tertiary)
- Colored terminal output matching current UX
- Proper error handling with actionable error messages

## Architecture Changes

- [engine/xtask/Cargo.toml](../../engine/xtask/Cargo.toml): Add dependencies for CLI parsing, filesystem ops, colored output
- [engine/xtask/src/main.rs](../../engine/xtask/src/main.rs): Complete rewrite with custom CLI and subcommands
- [engine/xtask/src/commands/](../../engine/xtask/src/commands/): New module structure for command implementations
- [scripts/build.sh](../../scripts/build.sh): Deprecate (keep as reference, add deprecation notice)

## Implementation Steps

### Phase 1: Project Setup & Dependencies

1. **Update xtask Cargo.toml** (File: `engine/xtask/Cargo.toml`)
   - Action: Add required dependencies for CLI parsing and cross-platform operations
   - Dependencies to add:
     - `clap` (v4) with `derive` feature for argument parsing
     - `anyhow` for error handling
     - `colored` for terminal colors
     - `which` for finding executables (cmake, cargo)
     - `fs_extra` for recursive directory operations
     - `dirs` for platform-specific paths (Library/Audio, AppData, etc.)
   - Why: These are standard Rust crates for CLI tooling, well-maintained and widely used
   - Dependencies: None
   - Risk: Low

2. **Create module structure** (File: `engine/xtask/src/`)
   - Action: Create directory structure:
     ```
     src/
       main.rs          # Entry point, CLI definition
       lib.rs           # Shared types and utilities
       commands/
         mod.rs         # Command module exports
         bundle.rs      # VST3/CLAP bundling (wraps nih_plug_xtask)
         test.rs        # Run cargo test
         au.rs          # AU wrapper build (macOS)
         install.rs     # Plugin installation
         clean.rs       # Clean build artifacts
     ```
   - Why: Modular structure makes each command testable and maintainable
   - Dependencies: Step 1
   - Risk: Low

### Phase 2: Core Infrastructure

3. **Implement CLI argument parsing** (File: `engine/xtask/src/main.rs`)
   - Action: Define CLI structure using clap derive macros
   - CLI Design:
     ```
     cargo xtask <COMMAND>
     
     Commands:
       bundle    Build and bundle VST3/CLAP plugins
       test      Run unit tests for specified crates
       au        Build AU wrapper (macOS only)
       install   Install plugins to system directories
       clean     Clean build artifacts
       all       Run full build pipeline (test → bundle → au → install)
     
     Global Options:
       --release / --debug    Build mode (default: release)
       --verbose              Show detailed output
       --dry-run              Show what would be done without executing
     ```
   - Why: Subcommand structure is more composable than flags
   - Dependencies: Step 1
   - Risk: Low

4. **Implement shared utilities** (File: `engine/xtask/src/lib.rs`)
   - Action: Create shared types and helper functions
   - Contents:
     - `BuildMode` enum (Release, Debug, ReleaseDebug)
     - `Platform` detection (macOS, Windows, Linux)
     - `paths` module with functions for:
       - `project_root()` → workspace root
       - `engine_dir()` → engine directory
       - `bundled_dir()` → target/bundled
       - `vst3_install_dir()` → platform-specific VST3 location
       - `clap_install_dir()` → platform-specific CLAP location
       - `au_install_dir()` → ~/Library/Audio/Plug-Ins/Components (macOS)
     - `print_header()`, `print_success()`, `print_error()` for colored output
     - `run_command()` helper that handles stdout/stderr and errors
   - Why: Centralizes platform logic and avoids duplication
   - Dependencies: Step 1
   - Risk: Low

### Phase 3: Command Implementations

5. **Implement `bundle` command** (File: `engine/xtask/src/commands/bundle.rs`)
   - Action: Wrap `nih_plug_xtask::main_with_args()` to invoke bundling
   - Details:
     - Accept `--release` / `--debug` flag
     - Accept optional package name (default: "vstkit")
     - Call into nih_plug_xtask programmatically or shell out to `cargo xtask bundle`
     - Note: May need to investigate nih_plug_xtask API for programmatic invocation
   - Why: Core functionality, must integrate with existing nih_plug_xtask
   - Dependencies: Steps 3, 4
   - Risk: Medium — need to verify nih_plug_xtask can be called as library

6. **Implement `test` command** (File: `engine/xtask/src/commands/test.rs`)
   - Action: Run `cargo test` for specified crates
   - Details:
     - Default crates: `dsp`, `protocol`
     - Accept `--package` / `-p` to specify additional crates
     - Accept `--all` to test entire workspace
     - Pass through `--release` if specified
     - Stream output in real-time
   - Why: Testing is part of CI workflow
   - Dependencies: Steps 3, 4
   - Risk: Low

7. **Implement `clean` command** (File: `engine/xtask/src/commands/clean.rs`)
   - Action: Clean build artifacts and optionally installed plugins
   - Details:
     - Run `cargo clean` in engine directory
     - Remove `target/bundled/` directory
     - Remove AU wrapper build directory
     - Optional `--installed` flag to also remove installed plugins
     - Confirm before deleting installed plugins (unless `--force`)
   - Why: Clean builds are needed for reproducibility
   - Dependencies: Steps 3, 4
   - Risk: Low

8. **Implement `au` command** (File: `engine/xtask/src/commands/au.rs`)
   - Action: Build AU wrapper using CMake (macOS only)
   - Details:
     - Check platform is macOS, error gracefully on other platforms
     - Check CMake is installed using `which` crate
     - Run `cmake -B build` and `cmake --build build`
     - Working directory: `packaging/macos/au-wrapper`
     - Handle CMake errors with actionable messages
   - Why: AU support is macOS-specific but important for Logic/GarageBand
   - Dependencies: Steps 3, 4, 5 (CLAP must be built first)
   - Risk: Medium — CMake error handling can be tricky

9. **Implement `install` command** (File: `engine/xtask/src/commands/install.rs`)
   - Action: Copy built plugins to system directories
   - Details:
     - Create destination directories if they don't exist
     - Remove existing plugins before copying (atomic replacement)
     - Install VST3 to platform-specific location
     - Install CLAP to platform-specific location  
     - Install AU to Components folder (macOS only)
     - Refresh AU cache on macOS (`killall -9 AudioComponentRegistrar`)
     - Print checkmarks for each successful install
   - Platform paths:
     - macOS VST3: `~/Library/Audio/Plug-Ins/VST3/`
     - macOS CLAP: `~/Library/Audio/Plug-Ins/CLAP/`
     - macOS AU: `~/Library/Audio/Plug-Ins/Components/`
     - Windows VST3: `C:\Program Files\Common Files\VST3\`
     - Windows CLAP: `C:\Program Files\Common Files\CLAP\`
     - Linux VST3: `~/.vst3/`
     - Linux CLAP: `~/.clap/`
   - Why: Installation is developer workflow critical
   - Dependencies: Steps 3, 4
   - Risk: Low

10. **Implement `all` command** (File: `engine/xtask/src/commands/mod.rs` or inline)
    - Action: Orchestrate full build pipeline
    - Details:
      - Run test → bundle → au (if macOS) → install
      - Stop on first error
      - Accept `--skip-tests` to bypass testing
      - Accept `--skip-au` to bypass AU build
      - Print summary at end
    - Why: Single command for full CI/release workflow
    - Dependencies: Steps 5-9
    - Risk: Low

### Phase 4: Integration & Migration

11. **Update main.rs entry point** (File: `engine/xtask/src/main.rs`)
    - Action: Wire up CLI to command handlers
    - Details:
      - Parse CLI args with clap
      - Dispatch to appropriate command handler
      - Handle top-level errors with colored output
      - Return appropriate exit codes
    - Why: Final integration step
    - Dependencies: Steps 3-10
    - Risk: Low

12. **Add deprecation notice to build.sh** (File: `scripts/build.sh`)
    - Action: Add warning banner, keep script functional for transition period
    - Details:
      - Print deprecation warning pointing to `cargo xtask`
      - Add comment explaining migration
      - Keep functionality intact for anyone who hasn't migrated
    - Why: Smooth transition for any existing users/CI
    - Dependencies: Step 11
    - Risk: Low

13. **Update documentation** (Files: `README.md`, `docs/`)
    - Action: Update build instructions to use new xtask commands
    - Details:
      - Document all available commands and options
      - Add examples for common workflows
      - Note platform-specific behavior
    - Why: Developer onboarding requires accurate docs
    - Dependencies: Step 11
    - Risk: Low

### Phase 5: Polish & Testing

14. **Add integration tests for xtask** (File: `engine/xtask/tests/`)
    - Action: Add tests that verify command behavior
    - Details:
      - Test CLI argument parsing
      - Test path resolution on current platform
      - Test dry-run mode produces expected output
      - Note: Full build tests may be too slow for unit tests
    - Why: Prevents regressions in build tooling
    - Dependencies: Step 11
    - Risk: Low

15. **Add `--help` documentation** (File: `engine/xtask/src/main.rs`)
    - Action: Ensure all commands have descriptive help text
    - Details:
      - Add `about` and `long_about` to all clap commands
      - Add examples in help text where useful
      - Test `cargo xtask --help` produces good output
    - Why: Self-documenting CLI improves DX
    - Dependencies: Step 11
    - Risk: Low

## Testing Strategy

- **Unit tests**: CLI parsing, path resolution, platform detection
- **Integration tests**: Dry-run mode verification, command dispatch
- **Manual testing**: 
  - Full build on macOS with AU
  - Full build on Linux (no AU)
  - Installation verification in DAW
- **CI verification**: Ensure GitHub Actions (if present) works with new commands

## Risks & Mitigations

- **Risk**: `nih_plug_xtask` may not expose a clean library API
  - Mitigation: Shell out to `cargo xtask bundle` as subprocess if needed (this is what the shell script does anyway)
  
- **Risk**: Platform-specific path handling edge cases
  - Mitigation: Use `dirs` crate for standard locations, test on all platforms
  
- **Risk**: CMake error messages may be opaque
  - Mitigation: Parse common errors and provide actionable suggestions (e.g., "CMake not found. Install with: brew install cmake")

- **Risk**: Breaking existing CI workflows
  - Mitigation: Keep build.sh functional during transition, update CI after xtask is proven

## Success Criteria

- [ ] `cargo xtask bundle --release` produces VST3 and CLAP bundles
- [ ] `cargo xtask test` runs dsp and protocol tests
- [ ] `cargo xtask au` builds AU wrapper on macOS (skips gracefully elsewhere)
- [ ] `cargo xtask install` installs plugins to correct system directories
- [ ] `cargo xtask clean` removes all build artifacts
- [ ] `cargo xtask all` runs complete pipeline
- [ ] All commands work on macOS (primary platform)
- [ ] Bundle and install commands work on Windows
- [ ] Bundle and install commands work on Linux
- [ ] Colored output matches or improves on build.sh UX
- [ ] --help provides useful documentation for all commands
- [ ] build.sh shows deprecation warning

## Appendix: Dependency Versions

```toml
[dependencies]
nih_plug_xtask = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "28b149ec4d62757d0b448809148a0c3ca6e09a95" }
clap = { version = "4", features = ["derive", "color"] }
anyhow = "1"
colored = "2"
which = "6"
fs_extra = "1"
dirs = "5"
```

## Appendix: Example CLI Usage

```bash
# Basic build
cargo xtask bundle

# Debug build
cargo xtask bundle --debug

# Run tests
cargo xtask test
cargo xtask test --all

# Build AU (macOS)
cargo xtask au

# Install to system
cargo xtask install

# Full pipeline
cargo xtask all

# Clean everything including installed plugins
cargo xtask clean --installed --force

# See what would happen
cargo xtask all --dry-run
```
