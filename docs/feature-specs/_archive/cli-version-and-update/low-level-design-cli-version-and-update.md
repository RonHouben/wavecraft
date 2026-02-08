# Low-Level Design: CLI Version and Update Command

**Feature:** CLI Enhancements (Milestone 14)  
**Target Version:** `0.8.1` (patch — CLI improvements, no breaking changes)  
**Author:** Architect Agent  
**Created:** 2026-02-08

---

## Overview

This feature adds two quality-of-life improvements to the Wavecraft CLI:

1. **Version flags** (`-v` / `--version`) — Quick way to check installed CLI version
2. **Update command** (`wavecraft update`) — Update all project dependencies (Rust + npm)

Both features enhance developer experience without introducing breaking changes. The version flag is a standard CLI convention, while the update command addresses a common workflow pain point.

---

## Goals

| Goal | Priority |
|------|----------|
| Add `-v` and `--version` flags with standard output format | High |
| Add `wavecraft update` command for dependency updates | High |
| Detect and update both Rust and npm dependencies | High |
| Clear error messages for edge cases | High |
| Zero breaking changes to existing CLI | High |

---

## Non-Goals

- **Not** updating the Wavecraft CLI itself (use `cargo install wavecraft` for that)
- **Not** selective dependency updates (always update all)
- **Not** lock file manipulation (use underlying tools' behavior)
- **Not** version pinning or resolution (delegate to Cargo/npm)

---

## Architecture

### Feature 1: Version Flags

```
┌─────────────────────────────────────────────────────────────────┐
│                      VERSION FLAG FLOW                          │
└─────────────────────────────────────────────────────────────────┘

  User runs:
  $ wavecraft --version
  $ wavecraft -v
        │
        ▼
  ┌─────────────────┐
  │ clap ArgParser  │  Check for version flag before subcommands
  └────────┬────────┘
           │
           ▼
  ┌─────────────────┐
  │ env!("CARGO_    │  Compile-time version injection
  │ PKG_VERSION")   │  (from cli/Cargo.toml)
  └────────┬────────┘
           │
           ▼
  Output: wavecraft 0.8.1
  Exit code: 0
```

**Key Decision:** Use clap's built-in `--version` handling with `#[command(version)]` attribute. This gives us both `-v` and `--version` for free with standard behavior.

### Feature 2: Update Command

```
┌─────────────────────────────────────────────────────────────────┐
│                   UPDATE COMMAND FLOW                           │
└─────────────────────────────────────────────────────────────────┘

  User runs:
  $ wavecraft update
        │
        ▼
  ┌─────────────────────────────────────────────┐
  │ Detect project structure                     │
  │ ├─ Look for engine/Cargo.toml               │
  │ └─ Look for ui/package.json                 │
  └────────┬────────────────────────────────────┘
           │
           ├──────────────┬──────────────────┐
           ▼              ▼                  ▼
  ┌────────────────┐  ┌─────────────┐  ┌─────────────┐
  │ engine/ found? │  │ ui/ found?  │  │ Neither?    │
  └────────┬───────┘  └──────┬──────┘  └──────┬──────┘
           │                 │                 │
           ▼                 ▼                 ▼
  Run cargo update    Run npm update     Error: Not a
  in engine/          in ui/             plugin project
           │                 │                 │
           └─────────────────┴─────────────────┘
                             │
                             ▼
                    ┌────────────────────┐
                    │ Report results     │
                    │ Exit code: 0 or 1  │
                    └────────────────────┘
```

**Key Decision:** Detect workspace structure by looking for `engine/Cargo.toml` and `ui/package.json`. Run updates independently and report combined results.

---

## Detailed Design

### 1. Version Flag Implementation

**CLI Structure (clap):**

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wavecraft")]
#[command(version)]  // ← This adds -v/--version automatically
#[command(about = "Wavecraft audio plugin development toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Create { /* ... */ },
    Start { /* ... */ },
    Update,  // ← New command
}
```

**Version Source:**

The version comes from `cli/Cargo.toml`:

```toml
[package]
name = "wavecraft"
version = "0.8.1"  # ← Single source of truth
```

At compile time, Cargo injects this as `env!("CARGO_PKG_VERSION")`. clap automatically uses it when `#[command(version)]` is present.

**Output Format:**

```
$ wavecraft --version
wavecraft 0.8.1

$ wavecraft -v
wavecraft 0.8.1
```

No additional metadata (like git hash or build date) at this stage. Keep it simple and standard.

---

### 2. Update Command Implementation

**File:** `cli/src/commands/update.rs`

```rust
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Update all project dependencies (Rust crates + npm packages).
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

fn update_rust_deps() -> Result<()> {
    let status = Command::new("cargo")
        .arg("update")
        .current_dir("engine")
        .status()
        .context("Failed to run 'cargo update'")?;

    if !status.success() {
        bail!("cargo update exited with status {}", status);
    }

    Ok(())
}

fn update_npm_deps() -> Result<()> {
    let status = Command::new("npm")
        .arg("update")
        .current_dir("ui")
        .status()
        .context("Failed to run 'npm update'")?;

    if !status.success() {
        bail!("npm update exited with status {}", status);
    }

    Ok(())
}
```

**Command Registration:**

```rust
// cli/src/main.rs
mod commands;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Create { name, output }) => commands::create::run(name, output),
        Some(Commands::Start { install, port }) => commands::start::run(install, port),
        Some(Commands::Update) => commands::update::run(),  // ← New
        None => {
            println!("Run 'wavecraft --help' for usage information");
            std::process::exit(0);
        }
    }
    .unwrap_or_else(|e| {
        eprintln!("Error: {:?}", e);
        std::process::exit(1);
    });
}
```

---

## Workspace Structure Detection

### Strategy: File-Based Detection

The update command detects the workspace structure by checking for marker files:

| Marker File | Component | Update Command |
|-------------|-----------|----------------|
| `engine/Cargo.toml` | Rust workspace | `cd engine && cargo update` |
| `ui/package.json` | npm workspace | `cd ui && npm update` |

**Why this approach:**

1. **Simple** — No parsing, just file existence checks
2. **Reliable** — Files are required for projects to work
3. **Fast** — No need to walk directory trees
4. **Portable** — Works consistently across platforms

**Edge Cases:**

| Case | Behavior |
|------|----------|
| Neither `engine/` nor `ui/` exists | Error: "Not a Wavecraft plugin project" |
| Only `engine/` exists | Update Rust deps only |
| Only `ui/` exists | Update npm deps only |
| Both exist | Update both, report combined results |
| User runs from subdirectory | Fails with clear error (expects root) |

---

## Error Handling

### Categories

| Error Type | Example | Exit Code | User Action |
|------------|---------|-----------|-------------|
| **Not a plugin project** | Missing engine/ and ui/ | 1 | Check current directory |
| **Tool not found** | `cargo` or `npm` not in PATH | 1 | Install missing tool |
| **Network failure** | Can't reach crates.io/npm | 1 | Check internet connection |
| **Dependency conflict** | Incompatible versions | 1 | Resolve manually |
| **Permission denied** | Can't write lock files | 1 | Check file permissions |

### Error Message Format

**Bad (vague):**
```
Error: Update failed
```

**Good (actionable):**
```
❌ Rust update failed: Failed to run 'cargo update'

Not a Wavecraft plugin project.
Expected to find 'engine/Cargo.toml' or 'ui/package.json'.
Run this command from the root of a Wavecraft plugin project.
```

**Best Practice:** Use context with `anyhow::Context` trait to wrap underlying errors with user-friendly explanations.

---

## Output Format

### Version Flag Output

```bash
$ wavecraft --version
wavecraft 0.8.1
```

**Design Decisions:**

- **Lowercase binary name** (standard convention)
- **Version only** (no git hash, build date, etc.)
- **No emoji** (keeps it professional and copy-pasteable)
- **Exit code 0** (success)

### Update Command Output

**Success (both components):**

```bash
$ wavecraft update
📦 Updating Rust dependencies...
✅ Rust dependencies updated
📦 Updating npm dependencies...
✅ npm dependencies updated

✨ All dependencies updated successfully
```

**Partial failure:**

```bash
$ wavecraft update
📦 Updating Rust dependencies...
❌ Rust update failed: Failed to run 'cargo update'
📦 Updating npm dependencies...
✅ npm dependencies updated

Error: Failed to update some dependencies:
  Rust: Failed to run 'cargo update'
```

**Complete failure:**

```bash
$ wavecraft update
Not a Wavecraft plugin project.
Expected to find 'engine/Cargo.toml' or 'ui/package.json'.
Run this command from the root of a Wavecraft plugin project.
```

**Design Decisions:**

- **Emoji for visual scanning** (easy to spot success/failure)
- **Progressive output** (user sees progress in real-time)
- **Combined exit code** (1 if any component fails)
- **Specific error context** (which component failed and why)

---

## Dependency Update Semantics

### What `cargo update` Does

From Cargo's perspective:

- Reads `Cargo.toml` dependencies (e.g., `wavecraft = "0.8.0"`)
- Updates `Cargo.lock` to the latest compatible versions within semver constraints
- Does **not** modify `Cargo.toml`
- Respects version specifiers (`^0.8.0` means `>=0.8.0, <0.9.0`)

**Example:**

```toml
# Cargo.toml (unchanged)
[dependencies]
wavecraft = "0.8.0"
```

```toml
# Cargo.lock (updated)
[[package]]
name = "wavecraft"
version = "0.8.1"  # ← Updated from 0.8.0
```

### What `npm update` Does

From npm's perspective:

- Reads `package.json` dependencies (e.g., `"@wavecraft/core": "^0.7.0"`)
- Updates `package-lock.json` to the latest compatible versions
- **May update** `package.json` if versions are specified with `^` or `~`
- Respects semver ranges

**Example:**

```json
// package.json (may be updated)
{
  "dependencies": {
    "@wavecraft/core": "^0.7.1"  // ← May change from ^0.7.0
  }
}
```

### Our Design Decision

**We delegate entirely to the underlying tools:**

- No custom version resolution logic
- No lock file manipulation
- No semver parsing
- Trust Cargo and npm to do their jobs correctly

**Rationale:**

1. **Simplicity** — Less code, fewer bugs
2. **Consistency** — Users get the same behavior as running commands manually
3. **Maintainability** — No need to track changes in Cargo/npm's update logic
4. **Correctness** — Cargo and npm have extensive testing for dependency resolution

---

## Testing Strategy

### Unit Tests

**File:** `cli/src/commands/update.rs`

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
        fs::write(engine_dir.join("Cargo.toml"), "[package]").unwrap();

        // Change to temp dir, run detection
        // Assert: only Rust update attempted
    }

    #[test]
    fn test_detects_ui_only() {
        let temp = TempDir::new().unwrap();
        let ui_dir = temp.path().join("ui");
        fs::create_dir(&ui_dir).unwrap();
        fs::write(ui_dir.join("package.json"), "{}").unwrap();

        // Change to temp dir, run detection
        // Assert: only npm update attempted
    }

    #[test]
    fn test_fails_when_neither_present() {
        let temp = TempDir::new().unwrap();
        
        // Change to empty temp dir
        // Assert: returns error with helpful message
    }

    #[test]
    fn test_version_flag_format() {
        // Mock clap output for --version
        // Assert: matches "wavecraft X.Y.Z\n"
    }
}
```

### Integration Tests

**File:** `cli/tests/update_command.rs`

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn version_flag_works() {
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
```

### Manual Testing Checklist

| Test Case | Expected Behavior |
|-----------|-------------------|
| `wavecraft --version` from any dir | Displays version, exit 0 |
| `wavecraft -v` from any dir | Displays version, exit 0 |
| `wavecraft update` from plugin root | Updates both Rust and npm deps |
| `wavecraft update` from non-plugin dir | Error: "Not a Wavecraft plugin project" |
| `wavecraft update` with no internet | Error from underlying tool (cargo/npm) |
| `wavecraft update` with outdated deps | Lock files updated, dependencies upgraded |
| User in `engine/` subdir runs update | Error (expects root) |
| User in `ui/` subdir runs update | Error (expects root) |

---

## Edge Cases & Considerations

### 1. Running from Subdirectories

**Problem:** User runs `wavecraft update` from `engine/` or `ui/`

**Behavior:** Fails with clear error because `engine/Cargo.toml` doesn't exist from that path.

**Solution:** Document in error message that command must be run from project root.

### 2. Monorepo Workspaces

**Problem:** Plugin is part of a larger monorepo with nested workspaces.

**Behavior:** Works as long as `engine/` and `ui/` exist at the current directory level.

**Tradeoff:** We don't walk up the directory tree looking for workspace roots. This keeps detection simple and predictable.

### 3. Cargo vs npm Failure Handling

**Problem:** One tool succeeds, the other fails.

**Behavior:** Both always attempted. Combined error reported at the end.

**Rationale:** Partial updates are better than no updates. User gets as much progress as possible.

### 4. Network Failures

**Problem:** Can't reach crates.io or npm registry.

**Behavior:** Underlying tool error propagates with context.

**Example Output:**

```
❌ Rust update failed: Failed to run 'cargo update'
caused by: Unable to update registry `crates-io`
```

User sees full error chain for debugging.

### 5. Lock File Conflicts (Git)

**Problem:** Lock files are modified, user has uncommitted changes.

**Behavior:** Not our problem. Git will handle merge conflicts normally.

**Documentation:** Add note in SDK Getting Started guide about committing before running updates.

---

## Performance Considerations

### Update Command Timing

| Component | Typical Duration | Notes |
|-----------|------------------|-------|
| Detection | <10ms | Just file existence checks |
| Cargo update | 5-30s | Network-dependent, crate count |
| npm update | 10-60s | Network-dependent, package count |
| **Total** | **15-90s** | Dominated by network I/O |

**Not a bottleneck.** This is an infrequent operation (maybe once per week per developer). No need to optimize.

### Parallel Updates?

**Question:** Should we run `cargo update` and `npm update` in parallel?

**Answer:** No, for these reasons:

1. **Marginal benefit** — Network I/O dominates; CPU parallelism doesn't help
2. **Output complexity** — Interleaved stdout/stderr would be confusing
3. **Error handling** — Harder to report which failed when running concurrently
4. **Implementation cost** — More code for minimal gain

**Decision:** Run sequentially. Simple, clear output, easy to debug.

---

## Documentation Updates

### 1. CLI Help Text

```bash
$ wavecraft --help
Wavecraft audio plugin development toolkit

Usage: wavecraft [OPTIONS] [COMMAND]

Commands:
  create  Create a new Wavecraft plugin project
  start   Start development servers for browser-based UI development
  update  Update all project dependencies (Rust crates + npm packages)
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### 2. SDK Getting Started Guide

Add a new section: **Updating Dependencies**

```markdown
## Updating Dependencies

To update all dependencies in your plugin project:

```bash
cd my-awesome-plugin
wavecraft update
```

This updates:
- Rust crates in `engine/Cargo.lock`
- npm packages in `ui/package-lock.json`

**Note:** Commit your changes before running updates to make it easy to review 
lock file changes via `git diff`.
```

### 3. Coding Standards

Add to the "CLI Tool" section (if it exists):

```markdown
### Version Flag

All CLI tools should support `-v` and `--version` flags using clap's built-in 
`#[command(version)]` attribute. The version is automatically extracted from 
`Cargo.toml` at compile time.

Output format: `<binary-name> <version>`

Example: `wavecraft 0.8.1`
```

---

## Security Considerations

### 1. Command Injection

**Risk:** Low. No user input is passed to shell commands.

**Mitigation:** We use `Command::new("cargo")` and `Command::new("npm")` with explicit arguments, not shell interpolation.

**Example of what we DON'T do:**

```rust
// ❌ UNSAFE: Shell injection possible
Command::new("sh")
    .arg("-c")
    .arg(format!("cd {} && cargo update", user_input))  // BAD!
```

**What we DO:**

```rust
// ✅ SAFE: No shell, explicit args
Command::new("cargo")
    .arg("update")
    .current_dir("engine")  // Hardcoded, not user input
```

### 2. Path Traversal

**Risk:** Low. We only check for `engine/Cargo.toml` and `ui/package.json` in the current directory.

**Mitigation:** No path construction from user input. All paths are hardcoded literals.

### 3. Tool PATH Hijacking

**Risk:** Low on developer machines (trusted environment).

**Consideration:** If `cargo` or `npm` in PATH is malicious, we'd run it.

**Mitigation:** Not feasible to sandbox. Developer machines assume trusted PATH. This is the same risk as running `cargo` or `npm` manually.

---

## Alternatives Considered

### Alternative 1: Combined `--update` Flag on `create`/`start`

**Idea:** `wavecraft create my-plugin --update-deps` to create and immediately update dependencies.

**Rejected because:**

- Orthogonal concerns (creation vs maintenance)
- Adds cognitive load to project creation
- Update is a separate, repeatable operation

### Alternative 2: Interactive Prompts for Selective Updates

**Idea:** Prompt user: "Update Rust deps? [y/n]", "Update npm deps? [y/n]"

**Rejected because:**

- Violates CLI UX improvement goal (zero prompts)
- Adds friction to simple operation
- Power users can run `cargo update` or `npm update` directly for selective updates

### Alternative 3: Lock File Diff Display

**Idea:** Show a summary of what changed (e.g., "Updated 12 crates, 8 npm packages")

**Rejected because:**

- Requires parsing lock files (extra complexity)
- User can see this via `git diff` if they commit first
- Marginal value for implementation cost

**Future consideration:** Could add this in a later iteration if users request it.

### Alternative 4: `--dry-run` Flag

**Idea:** `wavecraft update --dry-run` to preview changes without applying them.

**Rejected for v0.8.1:**

- `cargo update --dry-run` and `npm update --dry-run` already exist (low-level tools)
- Would need to aggregate and parse their output
- Can't reliably predict what would change without running the actual update

**Future consideration:** Could add if there's user demand.

---

## Implementation Checklist

### Phase 1: Version Flag (Low Risk)

- [ ] Add `#[command(version)]` to CLI struct
- [ ] Add unit test for version output format
- [ ] Add integration test for `-v` and `--version`
- [ ] Manual test: verify output matches expected format
- [ ] Update README with version flag usage

**Estimated effort:** 1-2 hours

### Phase 2: Update Command (Medium Risk)

- [ ] Create `cli/src/commands/update.rs`
- [ ] Implement workspace detection logic
- [ ] Implement `update_rust_deps()` function
- [ ] Implement `update_npm_deps()` function
- [ ] Add error handling with context
- [ ] Register command in `main.rs`
- [ ] Add unit tests for detection logic
- [ ] Add integration tests for command behavior
- [ ] Manual testing in real plugin project
- [ ] Update SDK Getting Started guide

**Estimated effort:** 6-8 hours

### Phase 3: Documentation (Low Risk)

- [ ] Update CLI help text (already done via clap)
- [ ] Add "Updating Dependencies" section to Getting Started
- [ ] Add version flag conventions to Coding Standards
- [ ] Update High-Level Design (if needed)

**Estimated effort:** 2-3 hours

### Phase 4: QA & Merge (Low Risk)

- [ ] Run `cargo xtask ci-check` (all tests pass)
- [ ] Manual testing (8 test cases)
- [ ] QA review
- [ ] Architect review
- [ ] Merge to main

**Estimated effort:** 2-3 hours

**Total estimated effort:** 11-16 hours (1.5-2 days)

---

## Success Criteria

| Criterion | Verification Method |
|-----------|---------------------|
| `wavecraft -v` displays correct version | Manual test + integration test |
| `wavecraft --version` displays correct version | Manual test + integration test |
| `wavecraft update` updates Rust deps | Manual test in real plugin |
| `wavecraft update` updates npm deps | Manual test in real plugin |
| Clear error message when not in plugin dir | Integration test |
| Help text includes new command | Manual inspection |
| No breaking changes to existing commands | Regression tests pass |
| Documentation updated | Manual review |

---

## Open Questions

None at this time. Design is complete and ready for implementation.

---

## Related Documents

- [Roadmap](../../roadmap.md) — Milestone 14: CLI Enhancements
- [Backlog](../../backlog.md) — Original feature requests
- [Coding Standards](../../architecture/coding-standards.md) — CLI conventions
- [SDK Getting Started](../../guides/sdk-getting-started.md) — User-facing documentation
