# Implementation Plan: CLI Version and Update Command

**Feature:** CLI Enhancements (Milestone 14)  
**Target Version:** `0.8.1` (patch — CLI improvements, no breaking changes)  
**Based on:** [Low-Level Design](low-level-design-cli-version-and-update.md)  
**Created:** 2026-02-08

---

## Overview

This plan implements two CLI enhancements:
1. **Version flags** (`-v` / `--version`) using clap's built-in support
2. **Update command** (`wavecraft update`) for dependency management

The implementation is split into 4 phases, with each phase building on the previous. Total estimated effort: 11-16 hours (1.5-2 days).

---

## Requirements Summary

- Add `-v` and `--version` flags with output format: `wavecraft 0.8.1`
- Add `wavecraft update` command that updates both Rust and npm dependencies
- Detect workspace structure by checking for `engine/Cargo.toml` and `ui/package.json`
- Provide clear error messages for edge cases
- Maintain zero breaking changes to existing CLI behavior

---

## Architecture Overview

### Version Flag
```
clap #[command(version)] → env!("CARGO_PKG_VERSION") → "wavecraft 0.8.1"
```

### Update Command
```
Detection → Rust update (cargo update) → npm update (npm update) → Report results
```

---

## Implementation Steps

### Phase 1: Version Flag Implementation

**Goal:** Add `-v` and `--version` flags with standard output.  
**Risk:** Low (uses clap's built-in functionality)  
**Duration:** 1-2 hours

#### Step 1.1: Add version attribute to CLI struct

**File:** `cli/src/main.rs`

**Action:** Add `#[command(version)]` attribute to the `Cli` struct.

**Code change:**
```rust
#[derive(Parser)]
#[command(name = "wavecraft")]
#[command(version)]  // ← Add this line
#[command(about = "Wavecraft audio plugin development toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}
```

**Why:** This tells clap to automatically generate `-v` and `--version` flags that display `env!("CARGO_PKG_VERSION")` from `Cargo.toml`.

**Dependencies:** None (clap already imported)

**Risk:** Low — standard clap feature

**Verification:**
```bash
cargo build --manifest-path cli/Cargo.toml
./cli/target/debug/wavecraft --version
# Expected: wavecraft 0.8.1
```

---

#### Step 1.2: Add integration test for version flag

**File:** `cli/tests/version_flag.rs` (create new file)

**Action:** Create integration test to verify version flag output.

**Code:**
```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_long_flag_works() {
    Command::cargo_bin("wavecraft")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("wavecraft "));
}

#[test]
fn version_short_flag_works() {
    Command::cargo_bin("wavecraft")
        .unwrap()
        .arg("-v")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("wavecraft "));
}

#[test]
fn version_format_is_correct() {
    let output = Command::cargo_bin("wavecraft")
        .unwrap()
        .arg("--version")
        .output()
        .unwrap();
    
    let version_str = String::from_utf8(output.stdout).unwrap();
    assert!(
        version_str.trim().matches('.').count() == 2,
        "Version should be in format X.Y.Z, got: {}",
        version_str.trim()
    );
}
```

**Why:** Ensures version flag works correctly and output format matches expectations.

**Dependencies:** Requires step 1.1

**Risk:** Low

**Verification:**
```bash
cargo test --manifest-path cli/Cargo.toml version_flag
```

---

#### Step 1.3: Manual testing for version flag

**Action:** Manually test version flags in various scenarios.

**Test cases:**
1. Run `wavecraft --version` from any directory → displays version
2. Run `wavecraft -v` from any directory → displays version
3. Verify version matches `cli/Cargo.toml` version field
4. Verify no extra output (no emoji, timestamps, etc.)

**Why:** Confirm user-facing behavior is correct.

**Dependencies:** Requires step 1.1

**Risk:** Low

---

### Phase 2: Update Command Core Implementation

**Goal:** Implement the `wavecraft update` command with workspace detection.  
**Risk:** Medium (includes file I/O, subprocess execution)  
**Duration:** 6-8 hours

#### Step 2.1: Create update command module

**File:** `cli/src/commands/update.rs` (create new file)

**Action:** Create the command module with basic structure.

**Code:**
```rust
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Update all project dependencies (Rust crates + npm packages).
pub fn run() -> Result<()> {
    println!("🔍 Detecting project structure...");
    
    // Implementation will be added in subsequent steps
    Ok(())
}
```

**Why:** Establishes the module structure for the update command.

**Dependencies:** None

**Risk:** Low

**Verification:** File compiles without errors

---

#### Step 2.2: Implement workspace detection

**File:** `cli/src/commands/update.rs`

**Action:** Add logic to detect `engine/` and `ui/` directories.

**Code:**
```rust
pub fn run() -> Result<()> {
    // Detect workspace structure
    let has_engine = Path::new("engine/Cargo.toml").exists();
    let has_ui = Path::new("ui/package.json").exists();

    if !has_engine && !has_ui {
        bail!(
            "Not a Wavecraft plugin project.\n\
             Expected to find 'engine/Cargo.toml' or 'ui/package.json'.\n\
             Run this command from the root of a Wavecraft plugin project."
        );
    }

    println!("✅ Detected:");
    if has_engine {
        println!("   • Rust workspace (engine/)");
    }
    if has_ui {
        println!("   • npm workspace (ui/)");
    }

    Ok(())
}
```

**Why:** Validates we're in a plugin project before attempting updates.

**Dependencies:** Requires step 2.1

**Risk:** Low — simple file existence checks

**Verification:** Run from plugin project root and non-plugin directory

---

#### Step 2.3: Implement Rust dependency update

**File:** `cli/src/commands/update.rs`

**Action:** Add `update_rust_deps()` function.

**Code:**
```rust
fn update_rust_deps() -> Result<()> {
    let status = Command::new("cargo")
        .arg("update")
        .current_dir("engine")
        .status()
        .context("Failed to run 'cargo update'. Is cargo installed?")?;

    if !status.success() {
        bail!("cargo update exited with status {}", status);
    }

    Ok(())
}
```

**Why:** Delegates to cargo for dependency updates.

**Dependencies:** Requires step 2.2

**Risk:** Medium — depends on cargo being in PATH

**Verification:** Test in plugin project with outdated dependencies

---

#### Step 2.4: Implement npm dependency update

**File:** `cli/src/commands/update.rs`

**Action:** Add `update_npm_deps()` function.

**Code:**
```rust
fn update_npm_deps() -> Result<()> {
    let status = Command::new("npm")
        .arg("update")
        .current_dir("ui")
        .status()
        .context("Failed to run 'npm update'. Is npm installed?")?;

    if !status.success() {
        bail!("npm update exited with status {}", status);
    }

    Ok(())
}
```

**Why:** Delegates to npm for dependency updates.

**Dependencies:** Requires step 2.3

**Risk:** Medium — depends on npm being in PATH

**Verification:** Test in plugin project with outdated npm packages

---

#### Step 2.5: Integrate update functions with main logic

**File:** `cli/src/commands/update.rs`

**Action:** Wire up detection and update functions with error handling.

**Code:**
```rust
pub fn run() -> Result<()> {
    // Detect workspace structure
    let has_engine = Path::new("engine/Cargo.toml").exists();
    let has_ui = Path::new("ui/package.json").exists();

    if !has_engine && !has_ui {
        bail!(
            "Not a Wavecraft plugin project.\n\
             Expected to find 'engine/Cargo.toml' or 'ui/package.json'.\n\
             Run this command from the root of a Wavecraft plugin project."
        );
    }

    let mut errors = Vec::new();

    // Update Rust dependencies
    if has_engine {
        println!("📦 Updating Rust dependencies...");
        match update_rust_deps() {
            Ok(()) => println!("✅ Rust dependencies updated"),
            Err(e) => {
                eprintln!("❌ Rust update failed: {}", e);
                errors.push(format!("Rust: {}", e));
            }
        }
    }

    // Update npm dependencies
    if has_ui {
        println!("📦 Updating npm dependencies...");
        match update_npm_deps() {
            Ok(()) => println!("✅ npm dependencies updated"),
            Err(e) => {
                eprintln!("❌ npm update failed: {}", e);
                errors.push(format!("npm: {}", e));
            }
        }
    }

    if errors.is_empty() {
        println!("\n✨ All dependencies updated successfully");
        Ok(())
    } else {
        bail!(
            "Failed to update some dependencies:\n  {}",
            errors.join("\n  ")
        );
    }
}
```

**Why:** Combines detection and updates with proper error handling.

**Dependencies:** Requires steps 2.2-2.4

**Risk:** Medium — orchestrates multiple steps

**Verification:** Test with various scenarios (both succeed, one fails, both fail)

---

#### Step 2.6: Register update command in CLI

**File:** `cli/src/commands/mod.rs`

**Action:** Add `pub mod update;` to expose the module.

**Code change:**
```rust
pub mod create;
pub mod new;
pub mod start;
pub mod update;  // ← Add this line
```

**Why:** Makes the update module available to main.rs.

**Dependencies:** Requires step 2.5

**Risk:** Low

---

#### Step 2.7: Add Update variant to Commands enum

**File:** `cli/src/main.rs`

**Action:** Add `Update` variant to the `Commands` enum.

**Code change:**
```rust
#[derive(Subcommand)]
enum Commands {
    Create {
        // ... existing fields
    },
    Start {
        // ... existing fields
    },
    Update,  // ← Add this line
}
```

**Why:** Adds the subcommand to clap's parser.

**Dependencies:** Requires step 2.6

**Risk:** Low

**Verification:** Run `wavecraft help` and verify "update" appears

---

#### Step 2.8: Wire update command to match statement

**File:** `cli/src/main.rs`

**Action:** Add match arm for `Commands::Update`.

**Code change:**
```rust
match cli.command {
    Some(Commands::Create { name, output }) => commands::create::run(name, output),
    Some(Commands::Start { install, port }) => commands::start::run(install, port),
    Some(Commands::Update) => commands::update::run(),  // ← Add this line
    None => {
        println!("Run 'wavecraft --help' for usage information");
        std::process::exit(0);
    }
}
.unwrap_or_else(|e| {
    eprintln!("Error: {:?}", e);
    std::process::exit(1);
});
```

**Why:** Routes the update command to its handler.

**Dependencies:** Requires step 2.7

**Risk:** Low

**Verification:** Run `wavecraft update` and verify it executes (even if it fails due to not being in a plugin project)

---

### Phase 3: Testing & Documentation

**Goal:** Add comprehensive tests and update documentation.  
**Risk:** Low  
**Duration:** 2-3 hours

#### Step 3.1: Add unit tests for workspace detection

**File:** `cli/src/commands/update.rs`

**Action:** Add unit tests in a `#[cfg(test)]` module.

**Code:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detects_engine_only() {
        let temp = TempDir::new().unwrap();
        let engine_dir = temp.path().join("engine");
        fs::create_dir(&engine_dir).unwrap();
        fs::write(engine_dir.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let has_engine = temp.path().join("engine/Cargo.toml").exists();
        let has_ui = temp.path().join("ui/package.json").exists();

        assert!(has_engine);
        assert!(!has_ui);
    }

    #[test]
    fn test_detects_ui_only() {
        let temp = TempDir::new().unwrap();
        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        let has_engine = temp.path().join("engine/Cargo.toml").exists();
        let has_ui = temp.path().join("ui/package.json").exists();

        assert!(!has_engine);
        assert!(has_ui);
    }

    #[test]
    fn test_detects_both() {
        let temp = TempDir::new().unwrap();
        
        let engine_dir = temp.path().join("engine");
        fs::create_dir(&engine_dir).unwrap();
        fs::write(engine_dir.join("Cargo.toml"), "[package]").unwrap();
        
        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        let has_engine = temp.path().join("engine/Cargo.toml").exists();
        let has_ui = temp.path().join("ui/package.json").exists();

        assert!(has_engine);
        assert!(has_ui);
    }
}
```

**Why:** Tests the core detection logic in isolation.

**Dependencies:** Requires step 2.5

**Risk:** Low

**Verification:**
```bash
cargo test --manifest-path cli/Cargo.toml update::tests
```

---

#### Step 3.2: Add integration tests for update command

**File:** `cli/tests/update_command.rs` (create new file)

**Action:** Create integration tests for the update command.

**Code:**
```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use std::fs;

#[test]
fn update_requires_plugin_project() {
    let temp = TempDir::new().unwrap();
    
    Command::cargo_bin("wavecraft")
        .unwrap()
        .arg("update")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not a Wavecraft plugin project"));
}

#[test]
fn update_command_appears_in_help() {
    Command::cargo_bin("wavecraft")
        .unwrap()
        .arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("update"));
}

#[test]
fn update_accepts_no_arguments() {
    // Should fail because we're not in a project, but shouldn't complain about args
    Command::cargo_bin("wavecraft")
        .unwrap()
        .arg("update")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Update all project dependencies"));
}
```

**Why:** Tests end-to-end behavior including error messages.

**Dependencies:** Requires step 2.8

**Risk:** Low

**Verification:**
```bash
cargo test --manifest-path cli/Cargo.toml update_command
```

---

#### Step 3.3: Update SDK Getting Started guide

**File:** `docs/guides/sdk-getting-started.md`

**Action:** Add a new section on updating dependencies.

**Insert after the "Development Workflow" section:**
```markdown
### Updating Dependencies

To update all dependencies in your plugin project:

\`\`\`bash
cd my-awesome-plugin
wavecraft update
\`\`\`

This updates:
- **Rust crates** in `engine/Cargo.lock`
- **npm packages** in `ui/package-lock.json`

The command automatically detects which components are present and updates accordingly.

**Best practice:** Commit your changes before running updates. This makes it easy to review lock file changes via `git diff`.

**Note:** If you need to update the Wavecraft CLI itself:
\`\`\`bash
cargo install wavecraft
\`\`\`
```

**Why:** Documents the new feature for users.

**Dependencies:** Requires step 2.8

**Risk:** Low

---

#### Step 3.4: Update High-Level Design documentation

**File:** `docs/architecture/high-level-design.md`

**Action:** Add CLI update command to the "Wavecraft SDK Architecture" section.

**Find the CLI tool description and add:**
```markdown
### CLI Commands

| Command | Purpose |
|---------|---------|
| `wavecraft create <name>` | Create a new plugin project from template |
| `wavecraft start` | Start development servers (WebSocket + Vite) |
| `wavecraft update` | Update all project dependencies (Rust + npm) |
| `wavecraft --version` | Display CLI version |
```

**Why:** Keeps architecture documentation current.

**Dependencies:** Requires step 2.8

**Risk:** Low

---

#### Step 3.5: Update Coding Standards (CLI conventions)

**File:** `docs/architecture/coding-standards.md`

**Action:** Add a section on CLI version flags if not present.

**Add to Rust section:**
```markdown
### CLI Tools

**Version Flag:**

All CLI tools should support `-v` and `--version` flags using clap's built-in `#[command(version)]` attribute. The version is automatically extracted from `Cargo.toml` at compile time.

**Output format:** `<binary-name> <version>`

**Example:**
\`\`\`bash
$ wavecraft --version
wavecraft 0.8.1
\`\`\`

**Update Commands:**

Commands that update dependencies should:
- Delegate to the underlying package managers (`cargo update`, `npm update`)
- Not implement custom version resolution logic
- Provide clear error messages when tools are missing
- Report results for each component independently
```

**Why:** Documents conventions for future CLI commands.

**Dependencies:** None

**Risk:** Low

---

### Phase 4: Manual Testing & QA

**Goal:** Comprehensive manual testing and QA verification.  
**Risk:** Low  
**Duration:** 2-3 hours

#### Step 4.1: Manual testing - Version flag

**Action:** Execute manual test cases for version flag.

**Test cases:**
1. `wavecraft --version` from project root → displays version
2. `wavecraft -v` from project root → displays version  
3. `wavecraft --version` from random directory → displays version
4. Verify version matches `cli/Cargo.toml`
5. Verify output is clean (no emoji, no extra text)

**Expected results:** All tests pass, output format is `wavecraft 0.8.1`

**Dependencies:** Requires Phase 1 complete

**Risk:** Low

---

#### Step 4.2: Manual testing - Update command (success cases)

**Action:** Test update command in real plugin projects.

**Setup:** Use `target/tmp/test-plugin` from CLI testing or create fresh project.

**Test cases:**
1. Run `wavecraft update` from project root with both engine/ and ui/
2. Verify `engine/Cargo.lock` is modified (check `git diff`)
3. Verify `ui/package-lock.json` is modified (check `git diff`)
4. Both components report "✅ updated" in output
5. Exit code is 0

**Expected results:** Both lock files updated, success message displayed

**Dependencies:** Requires Phase 2 complete

**Risk:** Medium — requires working project setup

---

#### Step 4.3: Manual testing - Update command (error cases)

**Action:** Test error handling scenarios.

**Test cases:**
1. Run `wavecraft update` from non-plugin directory → error message
2. Run `wavecraft update` from directory with only `engine/` → updates Rust only
3. Run `wavecraft update` from directory with only `ui/` → updates npm only
4. Run from `engine/` subdirectory → error (expects root)
5. Run with no internet connection → tool error propagates

**Expected results:** Clear error messages, appropriate exit codes

**Dependencies:** Requires Phase 2 complete

**Risk:** Low

---

#### Step 4.4: Manual testing - Help text verification

**Action:** Verify CLI help output includes new command.

**Test cases:**
1. `wavecraft --help` → lists "update" command
2. `wavecraft update --help` → shows update command help
3. Help text is clear and matches documentation

**Expected results:** Help text is accurate and helpful

**Dependencies:** Requires Phase 2 complete

**Risk:** Low

---

#### Step 4.5: Run automated test suite

**Action:** Run all CLI tests to ensure no regressions.

**Commands:**
```bash
# Run all CLI tests
cargo test --manifest-path cli/Cargo.toml

# Run with verbose output
cargo test --manifest-path cli/Cargo.toml -- --nocapture

# Run specific test suites
cargo test --manifest-path cli/Cargo.toml version_flag
cargo test --manifest-path cli/Cargo.toml update_command
cargo test --manifest-path cli/Cargo.toml update::tests
```

**Expected results:** All tests pass, no failures

**Dependencies:** Requires Phases 1-3 complete

**Risk:** Low

---

#### Step 4.6: Run cargo xtask ci-check

**Action:** Run full CI checks locally.

**Command:**
```bash
cargo xtask ci-check
```

**Expected results:**
- Linting passes (clippy, fmt, ESLint, Prettier)
- All tests pass (engine + UI + CLI)
- No warnings or errors

**Dependencies:** Requires Phases 1-3 complete

**Risk:** Low

---

#### Step 4.7: Test in external plugin project

**Action:** Generate a fresh plugin with `wavecraft create` and test update command.

**Steps:**
```bash
# Generate test plugin
cargo run --manifest-path cli/Cargo.toml -- create TestUpdatePlugin \
  --output target/tmp/test-update-plugin

# Enter project
cd target/tmp/test-update-plugin

# Run update command
cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- update

# Verify lock files changed
git diff
```

**Expected results:** Command works in generated projects

**Dependencies:** Requires Phase 2 complete

**Risk:** Medium — end-to-end integration

---

#### Step 4.8: QA review

**Action:** Hand off to QA agent for code quality review.

**Focus areas:**
- Error handling completeness
- Code clarity and maintainability
- Test coverage
- Documentation accuracy
- Edge case handling

**Dependencies:** Requires all previous steps complete

**Risk:** Low

---

## Testing Strategy

### Unit Tests

| Test Suite | Location | Focus |
|------------|----------|-------|
| Version flag format | `cli/tests/version_flag.rs` | Output format, both flags work |
| Workspace detection | `cli/src/commands/update.rs` | File existence checking |
| Update command | `cli/tests/update_command.rs` | Error cases, help text |

### Integration Tests

| Test | Purpose |
|------|---------|
| `version_long_flag_works` | Verify `--version` displays version |
| `version_short_flag_works` | Verify `-v` displays version |
| `version_format_is_correct` | Verify X.Y.Z format |
| `update_requires_plugin_project` | Error when not in project |
| `update_command_appears_in_help` | Help text includes update |

### Manual Tests

| Category | Test Count |
|----------|------------|
| Version flag | 5 tests |
| Update command (success) | 5 tests |
| Update command (errors) | 5 tests |
| Help text | 3 tests |
| External project | 1 test |
| **Total** | **19 manual tests** |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| cargo/npm not in PATH | Medium | High | Clear error message with context |
| User runs from wrong directory | Medium | Low | Detection logic with helpful error |
| Network failures during update | Low | Medium | Let underlying tool error propagate |
| Lock file conflicts (git) | Low | Low | Document best practices |
| Partial update failure | Medium | Low | Continue with other component, report all errors |

---

## Success Criteria

- [ ] `wavecraft -v` and `wavecraft --version` display correct version
- [ ] `wavecraft update` successfully updates Rust dependencies
- [ ] `wavecraft update` successfully updates npm dependencies
- [ ] Clear error messages for all edge cases
- [ ] Help text includes new command
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All manual tests pass
- [ ] Documentation updated
- [ ] No breaking changes to existing CLI behavior
- [ ] `cargo xtask ci-check` passes clean

---

## Dependencies Between Phases

```
Phase 1 (Version Flag)
    ↓
    ├─ Can proceed independently
    │
Phase 2 (Update Command)
    ↓
    ├─ Depends on: None
    │
Phase 3 (Testing & Docs)
    ↓
    ├─ Depends on: Phase 1 + Phase 2
    │
Phase 4 (QA)
    ↓
    └─ Depends on: Phase 1 + Phase 2 + Phase 3
```

**Note:** Phase 1 and Phase 2 can be worked on in parallel since they have no dependencies on each other.

---

## Rollback Plan

If critical issues are discovered:

1. **Version flag issues:** Remove `#[command(version)]` attribute, revert to main
2. **Update command issues:** Remove `Commands::Update` variant and match arm, remove `commands/update.rs`
3. **Documentation issues:** Revert doc changes only

**Low risk:** All changes are additive (no modifications to existing functionality).

---

## Post-Implementation Tasks

After merging to main:

1. **Version bump:** Update `cli/Cargo.toml` to `0.8.1`
2. **Changelog:** Add entry to roadmap changelog
3. **Archive:** Move feature-specs to `_archive/` (handled by PO)
4. **Publish:** CLI will auto-publish via CD pipeline
5. **Announce:** Update users about new commands (optional)

---

## Open Questions

None. Design and plan are complete.

---

## Related Documents

- [User Stories](user-stories.md) — Original requirements (to be created)
- [Low-Level Design](low-level-design-cli-version-and-update.md) — Architectural decisions
- [Roadmap](../../roadmap.md) — Milestone 14
- [Coding Standards](../../architecture/coding-standards.md) — CLI conventions
