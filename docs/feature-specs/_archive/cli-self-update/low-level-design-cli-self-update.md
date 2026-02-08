# Low-Level Design: CLI Self-Update

**Feature:** CLI Self-Update (`wavecraft update`)  
**Target Version:** `0.9.1`  
**Branch:** `feature/cli-self-update`  
**Created:** 2026-02-08

---

## Related Documents

- [User Stories](./user-stories.md) — Requirements and acceptance criteria
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Conventions and patterns

---

## 1. Overview

Restructure the `wavecraft update` command from a project-only dependency updater to a two-phase command:

1. **Phase 1 — CLI self-update:** Run `cargo install wavecraft` to update the CLI binary.
2. **Phase 2 — Project dependency update:** If inside a Wavecraft plugin project, update Rust crates and npm packages (existing behavior).

Phase 1 runs unconditionally (from any directory). Phase 2 is context-dependent (requires project markers). Either phase can fail independently without blocking the other.

---

## 2. Technical Approach

### 2.1 Self-Update via `cargo install`

The CLI self-update uses `cargo install wavecraft`, which:

- Downloads and compiles the latest version from crates.io
- Replaces the binary at `~/.cargo/bin/wavecraft`
- Is **idempotent** — if already at latest, outputs "already installed" and exits 0
- Requires no additional dependencies (cargo is already available since the user installed the CLI via `cargo install`)

**Why not a custom HTTP download?** Using `cargo install` is the simplest approach: no checksum verification, no platform detection, no archive extraction, no binary signing. Cargo handles all of this. The trade-off is compile time (~30-60s), which is acceptable for a CLI tool update.

### 2.2 Version Detection

**Current version** is known at compile time:

```rust
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
```

**Newly installed version** is detected by running the freshly installed binary:

```rust
// After `cargo install wavecraft` completes successfully:
let output = Command::new("wavecraft")
    .arg("--version")
    .output()?;
// Parse: "wavecraft X.Y.Z" → "X.Y.Z"
```

This approach is reliable because:

- `cargo install` updates the binary on disk before returning
- Running `wavecraft --version` invokes the **new** binary (not the currently running process)
- The `--version` flag is provided by clap and always outputs `wavecraft {version}`

### 2.3 Two-Phase Execution Flow

```
wavecraft update
│
├── Phase 1: CLI Self-Update (always runs)
│   ├── Print "🔄 Checking for CLI updates..."
│   ├── Run `cargo install wavecraft`
│   ├── Capture output (stdout + stderr)
│   ├── Determine outcome:
│   │   ├── Updated → run `wavecraft --version`, compare with CURRENT_VERSION
│   │   │   └── Print "✅ CLI updated to X.Y.Z (was A.B.C)"
│   │   ├── Already up-to-date → detected from cargo output
│   │   │   └── Print "✅ CLI is up to date (X.Y.Z)"
│   │   └── Failed → capture error
│   │       └── Print "⚠️  CLI self-update failed: {reason}"
│   │       └── Print "   Run 'cargo install wavecraft' manually to update the CLI"
│   └── Continue regardless of outcome
│
├── Phase 2: Project Dependency Update (conditional)
│   ├── Check for project markers (engine/Cargo.toml, ui/package.json)
│   ├── If NOT in project:
│   │   └── Print info message about running from project root
│   ├── If in project:
│   │   ├── Update Rust deps (existing `update_rust_deps()`)
│   │   └── Update npm deps (existing `update_npm_deps()`)
│   └── Collect errors (existing pattern)
│
└── Summary
    ├── If CLI was updated AND project deps ran:
    │   └── Print "💡 Re-run `wavecraft update` to use the new CLI for dependency updates"
    └── Exit code: 0 if all attempted operations succeeded
```

---

## 3. Code Changes

### 3.1 `cli/src/commands/update.rs` — Restructured

The current `run()` function becomes the project dependency update logic. A new top-level `run()` orchestrates both phases.

**New structure:**

```rust
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Current CLI version, known at compile time.
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result of the CLI self-update attempt.
enum SelfUpdateResult {
    /// Updated from old_version to new_version.
    Updated {
        old_version: String,
        new_version: String,
    },
    /// Already at the latest version.
    AlreadyUpToDate,
    /// Self-update failed (non-fatal).
    Failed(String),
}

/// Update the CLI and (if in a project) project dependencies.
pub fn run() -> Result<()> {
    // Phase 1: CLI self-update
    let self_update_result = update_cli();

    // Phase 2: Project dependency update (context-dependent)
    let project_result = update_project_deps();

    // Summary
    print_summary(&self_update_result, &project_result)
}
```

**Key functions to add:**

#### `update_cli() -> SelfUpdateResult`

Runs `cargo install wavecraft`, captures output, determines outcome.

```rust
fn update_cli() -> SelfUpdateResult {
    println!("🔄 Checking for CLI updates...");

    let output = match Command::new("cargo")
        .args(["install", "wavecraft"])
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            return SelfUpdateResult::Failed(format!(
                "Failed to run 'cargo install'. Is cargo installed? ({})",
                e
            ));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return SelfUpdateResult::Failed(format!("cargo install failed: {}", stderr.trim()));
    }

    // Detect whether a new version was installed vs already up-to-date
    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_already_up_to_date(&stderr) {
        println!("✅ CLI is up to date ({})", CURRENT_VERSION);
        return SelfUpdateResult::AlreadyUpToDate;
    }

    // A new version was installed — query it
    match get_installed_version() {
        Ok(new_version) => {
            println!(
                "✅ CLI updated to {} (was {})",
                new_version, CURRENT_VERSION
            );
            SelfUpdateResult::Updated {
                old_version: CURRENT_VERSION.to_string(),
                new_version,
            }
        }
        Err(_) => {
            // Binary was updated but we couldn't determine the version
            println!("✅ CLI updated (was {})", CURRENT_VERSION);
            SelfUpdateResult::Updated {
                old_version: CURRENT_VERSION.to_string(),
                new_version: "unknown".to_string(),
            }
        }
    }
}
```

#### `is_already_up_to_date(stderr: &str) -> bool`

Parses `cargo install` stderr to detect the "already installed" case.

```rust
/// Detect if `cargo install` output indicates the package is already at the latest version.
///
/// `cargo install` writes to stderr. When the package is already installed at
/// the latest version, it outputs a line matching:
///   "package `wavecraft vX.Y.Z` is already installed"
fn is_already_up_to_date(stderr: &str) -> bool {
    stderr
        .lines()
        .any(|line| line.contains("is already installed"))
}
```

**Cargo output patterns** (observed from `cargo install`):

| Scenario | stderr contains |
|----------|-----------------|
| Already installed | `package \`wavecraft vX.Y.Z\` is already installed, use --force to override` |
| Newly installed | `Installing wavecraft vX.Y.Z` followed by `Installed package \`wavecraft vX.Y.Z\`` |
| Compile error | `error[E...]` or non-zero exit code |
| Not found | `error: could not find \`wavecraft\`` |

The `is already installed` substring is the stable marker for the up-to-date case. This has been consistent across cargo versions and is the simplest reliable detection.

#### `get_installed_version() -> Result<String>`

Runs the newly installed binary to get its version string.

```rust
/// Query the version of the wavecraft binary currently on disk.
fn get_installed_version() -> Result<String> {
    let output = Command::new("wavecraft")
        .arg("--version")
        .output()
        .context("Failed to run 'wavecraft --version'")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // clap outputs: "wavecraft X.Y.Z\n"
    let version = stdout
        .trim()
        .strip_prefix("wavecraft ")
        .unwrap_or(stdout.trim())
        .to_string();

    Ok(version)
}
```

#### `update_project_deps() -> ProjectUpdateResult`

Refactored from the current `run()`. Returns a structured result instead of bailing.

```rust
/// Result of project dependency updates.
enum ProjectUpdateResult {
    /// Not in a project directory — deps skipped.
    NotInProject,
    /// Project deps updated (may include partial failures).
    Updated { errors: Vec<String> },
}

fn update_project_deps() -> ProjectUpdateResult {
    let has_engine = Path::new("engine/Cargo.toml").exists();
    let has_ui = Path::new("ui/package.json").exists();

    if !has_engine && !has_ui {
        println!();
        println!("ℹ️  Not in a Wavecraft plugin project — skipping dependency updates.");
        println!(
            "   Run this command from a project root to also update Rust and npm dependencies."
        );
        return ProjectUpdateResult::NotInProject;
    }

    // Existing logic: update Rust and npm deps, collect errors
    let mut errors = Vec::new();

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

    ProjectUpdateResult::Updated { errors }
}
```

#### `print_summary(...)` — Final output and exit logic

```rust
fn print_summary(
    self_update: &SelfUpdateResult,
    project: &ProjectUpdateResult,
) -> Result<()> {
    let cli_updated = matches!(self_update, SelfUpdateResult::Updated { .. });
    let cli_failed = matches!(self_update, SelfUpdateResult::Failed(_));

    let project_errors = match project {
        ProjectUpdateResult::Updated { errors } => errors.clone(),
        ProjectUpdateResult::NotInProject => vec![],
    };

    // Show re-run hint if CLI was updated and project deps were also run
    if cli_updated && matches!(project, ProjectUpdateResult::Updated { .. }) {
        println!();
        println!("💡 Note: Project dependencies were updated using the previous CLI version.");
        println!("   Re-run `wavecraft update` to use the new CLI for dependency updates.");
    }

    // Determine final summary and exit code
    if project_errors.is_empty() && !cli_failed {
        println!();
        println!("✨ All updates complete");
        Ok(())
    } else if project_errors.is_empty() && cli_failed {
        // CLI failed but no project errors (or not in project)
        println!();
        println!("✨ Project dependencies updated (CLI self-update skipped)");
        Ok(())
    } else {
        anyhow::bail!(
            "Failed to update some dependencies:\n  {}",
            project_errors.join("\n  ")
        );
    }
}
```

#### Existing functions — Unchanged

`update_rust_deps()` and `update_npm_deps()` remain as-is. They are private helper functions with correct error handling already.

### 3.2 `cli/src/main.rs` — Clap Descriptions

Minimal changes to the `Commands` enum:

```rust
/// Update the CLI and project dependencies (Rust crates + npm packages)
#[command(
    long_about = "Update the Wavecraft CLI to the latest version, then update Rust crates \
    and npm packages if run from a plugin project directory.\n\n\
    Can be run from any directory. When outside a project, only the CLI is updated."
)]
Update,
```

No structural changes to `main()` — it still calls `commands::update::run()`.

### 3.3 `cli/Cargo.toml` — Version Bump

```toml
[package]
name = "wavecraft"
version = "0.9.1"
```

No new dependencies required. The implementation uses only `std::process::Command` (already available) and `anyhow` (already a dependency).

---

## 4. Error Handling

### 4.1 Error Matrix

| Failure | Impact | Behavior |
|---------|--------|----------|
| `cargo` not in PATH | CLI self-update fails | Warn + continue to project deps |
| `cargo install` non-zero exit | Self-update fails | Warn + continue to project deps |
| Network unavailable | `cargo install` fails | Warn + continue to project deps |
| `wavecraft --version` parse error | Can't determine new version | Show "updated" without version number |
| Not in a project | No deps to update | Info message, exit 0 |
| `cargo update` fails (engine) | Rust dep update fails | Error collected, npm still attempted |
| `npm update` fails (ui) | npm dep update fails | Error collected |
| Both dep updates fail | All project deps failed | Exit 1 with collected errors |

### 4.2 Design Principle

Phase 1 (self-update) **never** causes `run()` to return `Err`. It always captures failures as `SelfUpdateResult::Failed` and continues. This ensures the existing project dependency update behavior is preserved even if the new self-update feature has issues.

Phase 2 (project deps) follows the existing error collection pattern where individual failures don't block other operations. Only if no operations succeeded does the command return a non-zero exit.

### 4.3 Exit Code Logic

| CLI Update | In Project? | Dep Update | Exit Code |
|------------|-------------|------------|-----------|
| Success | No | N/A | 0 |
| Success | Yes | Success | 0 |
| Success | Yes | Partial fail | 1 |
| Failed | No | N/A | 0 (warn only) |
| Failed | Yes | Success | 0 (warn + deps ok) |
| Failed | Yes | All failed | 1 |

The rationale: if the user is outside a project, the only action is CLI self-update. A failure there is a warning, not a hard error — the user can always run `cargo install wavecraft` manually. Inside a project, the exit code is driven by project dependency results.

---

## 5. Output Parsing: `cargo install` Behavior

### 5.1 Cargo Output Patterns

`cargo install` writes progress/status to **stderr** and binary path info to **stdout**.

**Already installed (exit 0):**
```
$ cargo install wavecraft 2>&1
    Updating crates.io index
     Ignored package `wavecraft v0.9.1` is already installed, use --force to override
```
Key marker: line containing `is already installed`.

**Newly installed (exit 0):**
```
$ cargo install wavecraft 2>&1
    Updating crates.io index
  Installing wavecraft v0.9.2
   Compiling ...
    Finished ...
  Installing /Users/user/.cargo/bin/wavecraft
   Installed package `wavecraft v0.9.2` ...
```
Key marker: absence of `is already installed` + exit code 0.

**Failure (exit non-zero):**
Various error messages. We don't need to parse these — non-zero exit code is sufficient.

### 5.2 Why stderr, not stdout?

Cargo writes its human-readable progress messages to stderr. This is intentional — stdout is reserved for machine-readable output (like `cargo metadata --format-version=1`). Our `Command::output()` captures both.

### 5.3 Stability of the `is already installed` marker

This string has been stable across cargo 1.x releases (verified from cargo 1.60 through 1.84). It's unlikely to change without a major cargo version bump. If it does change, the worst case is that we fail to detect the "already up-to-date" case and fall through to `get_installed_version()`, which still produces correct behavior (comparing old vs new version).

---

## 6. Help Text Changes

### 6.1 Clap `Commands` enum update

**Before:**
```rust
/// Update all project dependencies (Rust crates + npm packages)
Update,
```

**After:**
```rust
/// Update the CLI and project dependencies (Rust crates + npm packages)
#[command(
    long_about = "Update the Wavecraft CLI to the latest version, then update Rust crates \
    and npm packages if run from a plugin project directory.\n\n\
    Can be run from any directory. When outside a project, only the CLI is updated."
)]
Update,
```

### 6.2 Expected `--help` output

**`wavecraft --help`** (short help):
```
Commands:
  create  Create a new plugin project from the Wavecraft template
  start   Start development servers (WebSocket + UI)
  update  Update the CLI and project dependencies (Rust crates + npm packages)
  help    Print this message or the help of the given subcommand(s)
```

**`wavecraft update --help`** (long help):
```
Update the Wavecraft CLI to the latest version, then update Rust crates
and npm packages if run from a plugin project directory.

Can be run from any directory. When outside a project, only the CLI is updated.

Usage: wavecraft update

Options:
  -h, --help  Print help
```

---

## 7. Testing Strategy

### 7.1 Unit Tests (in `update.rs`)

These test the parsing and detection logic without running external commands.

| Test | What It Verifies |
|------|-----------------|
| `test_is_already_up_to_date_true` | `is_already_up_to_date()` returns `true` for cargo's "already installed" output |
| `test_is_already_up_to_date_false` | Returns `false` for "Installing..." output (new version) |
| `test_is_already_up_to_date_empty` | Returns `false` for empty string |
| `test_parse_version_output` | `get_installed_version()` extraction works — tested via the parsing logic directly |
| `test_project_detection_no_markers` | `update_project_deps()` returns `NotInProject` when no engine/ui dirs exist |
| `test_project_detection_with_markers` | Detects engine-only, ui-only, and both |

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_already_up_to_date_true() {
        let stderr = "    Updating crates.io index\n     \
            Ignored package `wavecraft v0.9.1` is already installed, \
            use --force to override\n";
        assert!(is_already_up_to_date(stderr));
    }

    #[test]
    fn test_is_already_up_to_date_false_new_install() {
        let stderr = "    Updating crates.io index\n  \
            Installing wavecraft v0.9.2\n   \
            Compiling wavecraft v0.9.2\n";
        assert!(!is_already_up_to_date(stderr));
    }

    #[test]
    fn test_is_already_up_to_date_empty() {
        assert!(!is_already_up_to_date(""));
    }
}
```

### 7.2 Existing Tests — Preserved

The three existing tests (`test_detects_engine_only`, `test_detects_ui_only`, `test_detects_both`) remain unchanged. They test project marker detection which is still used by `update_project_deps()`.

### 7.3 Integration Tests (in `cli/tests/`)

| Test | What It Verifies |
|------|-----------------|
| `test_update_help_text` | `wavecraft update --help` contains expected description text |
| `test_update_outside_project` | Running `wavecraft update` from a non-project directory doesn't error (exit 0, skips deps) |

These use `assert_cmd` (already a dev-dependency) to invoke the CLI binary.

**Note:** Full self-update integration testing (actually running `cargo install`) is intentionally **not** automated because:
- It requires network access (crates.io)
- It modifies the user's `~/.cargo/bin/wavecraft` binary
- It takes 30-60s to compile
- The logic is simple enough that unit tests on the parsing cover the risky parts

### 7.4 Manual Testing Checklist

For the Tester agent to verify after implementation:

1. **Outside project, CLI up to date:**
   ```bash
   cd ~ && wavecraft update
   # Expected: "✅ CLI is up to date (0.9.1)"
   # Expected: "ℹ️  Not in a Wavecraft plugin project"
   ```

2. **Inside project, CLI up to date:**
   ```bash
   cd ~/my-plugin && wavecraft update
   # Expected: CLI check + Rust deps + npm deps
   ```

3. **Version display:**
   ```bash
   wavecraft --version
   # Expected: "wavecraft 0.9.1"
   ```

4. **Help text:**
   ```bash
   wavecraft update --help
   # Expected: mentions CLI update + project deps + any directory
   ```

---

## 8. File Change Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `cli/Cargo.toml` | Modify | Version bump `0.9.0` → `0.9.1` |
| `cli/src/commands/update.rs` | Modify | Restructure `run()` into two-phase; add enums, `update_cli()`, `is_already_up_to_date()`, `get_installed_version()`, `update_project_deps()`, `print_summary()` |
| `cli/src/main.rs` | Modify | Update `Commands::Update` help text and `long_about` |

No new files. No new dependencies. No changes to other commands.

---

## 9. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `cargo install` output format changes | Low | Medium — version detection degrades | Fall through to `get_installed_version()` which queries the binary directly |
| Self-update replaces binary mid-execution | N/A | None — OS loads the binary into memory at process start | The running process is unaffected; only the file on disk changes |
| User has old cargo that behaves differently | Low | Low — worst case: false "updated" message | Version comparison via `wavecraft --version` is authoritative |
| Network timeout during `cargo install` | Medium | Low — warns and continues to project deps | Already handled by `SelfUpdateResult::Failed` branch |
| `cargo install wavecraft` compiles slowly | Expected | UX — 30-60s wait | Cargo's own progress output streams to stderr, giving the user feedback |

### 9.1 Cargo Output Forwarding

**Decision:** Use `Command::output()` (captures all output) rather than `Command::status()` (inherits stdio).

**Trade-off:** The user won't see cargo's real-time compilation progress during self-update. However, `cargo install` is fast when already up-to-date (2-3s for index check) and the compilation case (30-60s) only happens when there's an actual update. Capturing output is required to detect the "already installed" marker.

**Alternative considered:** Use `Command::status()` for live output, then re-run `wavecraft --version` to detect changes. Rejected because it means we can't distinguish "updated" from "already up to date" — both exit 0 with live output, and comparing versions before/after is less reliable than parsing the output.

---

## 10. Future Considerations

These are **out of scope** for v0.9.1 but noted for future design:

- **`--cli-only` flag:** Skip project deps, only update the CLI. Low value since Phase 2 already skips when not in a project.
- **`--skip-cli` flag:** Skip self-update, only update project deps. Potentially useful for CI where the CLI version is pinned.
- **Version pinning:** `wavecraft update --cli-version 0.9.0` to install a specific version. Would use `cargo install wavecraft@0.9.0`.
- **Progress bars:** Replace cargo's captured output with an `indicatif` spinner during self-update. The `indicatif` crate is already a dependency.
