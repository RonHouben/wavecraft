# Low-Level Design: CLI Update UX Quick Wins

**Feature:** Pre-M19 Initiative — CLI Update UX Quick Wins  
**Status:** Implemented (Items #1 and #2; Item #3 deferred)  
**Author:** Architect Agent  
**Timebox:** Max 2 working days

---

## Related Documents

- [Roadmap](../../roadmap.md) — Pre-M19 Initiative section
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards — Rust](../../architecture/coding-standards-rust.md) — Rust conventions
- [Coding Standards — Testing](../../architecture/coding-standards-testing.md) — Testing conventions

---

## Overview

This initiative contains three scoped improvements to the `wavecraft update` CLI command UX, ordered by risk (lowest first):

| Item   | Description                    | Priority           | Risk   |
| ------ | ------------------------------ | ------------------ | ------ |
| **#2** | Split progress messaging       | First              | Low    |
| **#1** | CLI update rerun elimination   | Second             | Medium |
| **#3** | Dev build profile optimization | Last (conditional) | Low    |

All changes are confined to `cli/src/commands/update.rs`, `cli/src/main.rs`, and their test files.

---

## Current Architecture

### File: `cli/src/commands/update.rs`

The `run()` function executes two sequential phases:

```
run()
  ├── Phase 1: update_cli()        — always runs
  │   ├── start_cli_update_progress()  — background thread, shows message after 3s
  │   ├── Command::new("cargo").args(["install", "wavecraft"]).output()
  │   ├── is_already_up_to_date(stderr)
  │   └── get_installed_version()  — runs `wavecraft --version` on disk binary
  │
  ├── Phase 2: update_project_deps()  — context-dependent
  │   ├── detect_project(".")       — checks engine/Cargo.toml, ui/package.json
  │   ├── update_rust_deps()        — cargo update in engine/
  │   └── update_npm_deps()         — npm update in ui/
  │
  └── print_summary()
      └── determine_summary()      — pure function, decides messages + exit code
```

### Key Enums

- `SelfUpdateResult`: `Updated | AlreadyUpToDate | Failed`
- `ProjectUpdateResult`: `NotInProject | Updated { errors: Vec<String> }`
- `SummaryOutcome`: `AllComplete | ProjectOnlyComplete | ProjectErrors | NoAction`

### Current Pain Points

1. **Single progress message:** `"📦 Downloading and installing... this may take a minute on slow networks."` shown after 3s delay. Users can't distinguish download vs compilation phases.

2. **Rerun requirement:** When CLI is updated, Phase 2 still runs with the OLD process image. User sees: `"💡 Note: Project dependencies were updated using the previous CLI version. Re-run 'wavecraft update' to use the new CLI for dependency updates."` This is an unnecessary manual step.

---

## Item #2: Split Progress Messaging

### Problem

The `start_cli_update_progress()` function spawns a thread that sleeps 3 seconds, then shows a single message. `cargo install wavecraft` is invoked with `.output()` which blocks until completion — there is no streaming, so we cannot observe the download→compile transition in real time.

### Design

**Approach:** Replace `.output()` with `.spawn()` + streamed stderr reading. Parse cargo's stderr lines to detect phase transitions and display phase-appropriate progress messages.

#### Cargo stderr output patterns

`cargo install` writes progress to stderr. Key line patterns:

| Phase      | Pattern                | Example                                           |
| ---------- | ---------------------- | ------------------------------------------------- |
| Download   | `Downloading`          | `Downloading crates ...`                          |
| Download   | `Downloaded`           | `Downloaded wavecraft v0.9.2`                     |
| Compile    | `Compiling`            | `Compiling wavecraft v0.9.2`                      |
| Install    | `Installing`           | `Installing /Users/.../.cargo/bin/wavecraft`      |
| Up-to-date | `is already installed` | `package 'wavecraft v0.9.2' is already installed` |

#### New progress state machine

```
Checking → Downloading → Compiling → Done
              │               │
              └───────────────┘ (transitions on first line matching next phase)
```

#### Implementation

**Replace `start_cli_update_progress()` and the `.output()` call** with a streaming approach:

```rust
use std::io::{BufRead, BufReader};
use std::process::Stdio;

/// Progress phases during `cargo install`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum InstallPhase {
    Checking,
    Downloading,
    Compiling,
}

fn update_cli() -> SelfUpdateResult {
    println!("🔄 Checking for CLI updates...");

    let mut child = match Command::new("cargo")
        .args(["install", "wavecraft"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!(
                "⚠️  CLI self-update failed: Failed to run 'cargo install'. \
                 Is cargo installed? ({})", e
            );
            eprintln!("   Run 'cargo install wavecraft' manually to update the CLI.");
            return SelfUpdateResult::Failed;
        }
    };

    // Stream stderr, detect phases, collect full output
    let stderr_pipe = child.stderr.take().expect("stderr piped");
    let stderr_content = stream_install_progress(stderr_pipe);

    let status = match child.wait() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("⚠️  CLI self-update failed: {}", e);
            return SelfUpdateResult::Failed;
        }
    };

    if !status.success() {
        eprintln!(
            "⚠️  CLI self-update failed: cargo install failed: {}",
            stderr_content.trim()
        );
        eprintln!("   Run 'cargo install wavecraft' manually to update the CLI.");
        return SelfUpdateResult::Failed;
    }

    // Detect whether a new version was installed vs already up-to-date
    if is_already_up_to_date(&stderr_content) {
        println!("✅ CLI is up to date ({})", CURRENT_VERSION);
        return SelfUpdateResult::AlreadyUpToDate;
    }

    // ... (version detection unchanged)
}

/// Stream stderr from `cargo install`, showing phase-appropriate progress messages.
///
/// Returns the full stderr content for later analysis.
fn stream_install_progress(stderr: impl std::io::Read) -> String {
    let reader = BufReader::new(stderr);
    let mut all_output = String::new();
    let mut current_phase = InstallPhase::Checking;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        all_output.push_str(&line);
        all_output.push('\n');

        let new_phase = detect_phase(&line);
        if let Some(phase) = new_phase {
            if phase != current_phase {
                match phase {
                    InstallPhase::Downloading => {
                        println!("📥 Downloading...");
                    }
                    InstallPhase::Compiling => {
                        println!("🔨 Compiling... this may take a minute.");
                    }
                    InstallPhase::Checking => {} // won't transition back
                }
                current_phase = phase;
            }
        }
    }

    all_output
}

/// Detect the install phase from a cargo stderr line.
fn detect_phase(line: &str) -> Option<InstallPhase> {
    let trimmed = line.trim();
    if trimmed.starts_with("Downloading") || trimmed.starts_with("Downloaded") {
        Some(InstallPhase::Downloading)
    } else if trimmed.starts_with("Compiling") {
        Some(InstallPhase::Compiling)
    } else {
        None
    }
}
```

#### Removed code

- `start_cli_update_progress()` — entire function deleted (the background timer thread is replaced by streaming)
- The `Arc<AtomicBool>` signaling mechanism — no longer needed

#### New user-facing messages

| Old                                                                           | New                                                                    |
| ----------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `"📦 Downloading and installing... this may take a minute on slow networks."` | `"📥 Downloading..."` then `"🔨 Compiling... this may take a minute."` |

If `cargo install` determines the package is already up-to-date, neither message is shown (cargo emits the "is already installed" line without download/compile phases).

#### Tests

Add unit tests for `detect_phase()` and `stream_install_progress()`:

```rust
#[test]
fn test_detect_phase_downloading() {
    assert_eq!(detect_phase("  Downloading crates ..."), Some(InstallPhase::Downloading));
}

#[test]
fn test_detect_phase_compiling() {
    assert_eq!(detect_phase("   Compiling wavecraft v0.9.2"), Some(InstallPhase::Compiling));
}

#[test]
fn test_detect_phase_unrelated() {
    assert_eq!(detect_phase("   Updating crates.io index"), None);
}

#[test]
fn test_stream_collects_all_output() {
    let input = "  Downloading crates ...\n   Compiling wavecraft v0.9.2\n";
    let output = stream_install_progress(std::io::Cursor::new(input));
    assert!(output.contains("Downloading"));
    assert!(output.contains("Compiling"));
}
```

---

## Item #1: CLI Update Rerun Elimination

### Problem

When `update_cli()` returns `SelfUpdateResult::Updated`, the project dependency phase still runs under the OLD binary. Users must manually re-run `wavecraft update`. This is a poor UX — the tool knows it has a new binary on disk and should use it automatically.

### Design

**Approach:** After detecting a successful CLI self-update, replace the current process with the new binary using `exec()`. The new binary receives a hidden `--skip-self` flag and runs only Phase 2.

#### New clap flag

In `cli/src/main.rs`, change the `Update` variant to carry a hidden flag:

```rust
/// Update the CLI and project dependencies (Rust crates + npm packages)
#[command(
    long_about = "Update the Wavecraft CLI to the latest version, then update Rust crates \
    and npm packages if run from a plugin project directory.\n\n\
    Can be run from any directory. When outside a project, only the CLI is updated."
)]
Update {
    /// Skip CLI self-update (used internally after re-exec).
    #[arg(long, hide = true, default_value_t = false)]
    skip_self: bool,
},
```

Update the match arm:

```rust
Commands::Update { skip_self } => {
    commands::update::run(skip_self)?;
}
```

#### Modified `run()` signature

```rust
pub fn run(skip_self: bool) -> Result<()> {
    // Phase 1: CLI self-update (skipped if re-exec'd)
    let self_update_result = if skip_self {
        println!("✅ CLI updated to {}", CURRENT_VERSION);
        SelfUpdateResult::AlreadyUpToDate  // treat as resolved
    } else {
        update_cli()
    };

    // If CLI was updated and not in --skip-self mode, re-exec new binary
    if matches!(self_update_result, SelfUpdateResult::Updated) {
        return reexec_with_new_binary();
    }

    // Phase 2: Project dependency update (context-dependent)
    let project_result = update_project_deps();

    // Summary and exit code
    print_summary(&self_update_result, &project_result)
}
```

#### Re-exec implementation

```rust
/// Re-execute the newly installed CLI binary to continue with project deps.
///
/// Uses `exec()` on Unix to replace the process image. The new binary
/// runs `wavecraft update --skip-self`, which skips Phase 1 and runs
/// Phase 2 using the updated code.
fn reexec_with_new_binary() -> Result<()> {
    println!();
    println!("🔄 Continuing with updated CLI...");

    // Resolve the new binary path
    let binary = which_wavecraft()?;

    // Build the command for re-exec
    let mut cmd = Command::new(&binary);
    cmd.args(["update", "--skip-self"]);

    // On Unix, exec() replaces the process image (no child process)
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        // exec() only returns on error
        bail!("Failed to re-exec CLI: {}", err);
    }

    // On non-Unix, spawn and forward exit code
    #[cfg(not(unix))]
    {
        let status = cmd.status().context("Failed to re-exec CLI")?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

/// Find the wavecraft binary path.
///
/// Uses `which` crate or falls back to PATH lookup.
fn which_wavecraft() -> Result<std::path::PathBuf> {
    which::which("wavecraft").context(
        "Could not find 'wavecraft' binary after update. \
         Re-run 'wavecraft update' manually."
    )
}
```

#### Dependency: `which` crate

Add to `cli/Cargo.toml`:

```toml
[dependencies]
which = "7"
```

The `which` crate is a well-maintained, minimal dependency for portable binary path resolution. It avoids hand-rolling PATH search logic.

#### Rerun hint removal

With reexec in place, the `print_rerun_hint()` function and the `show_rerun_hint` field in `SummaryOutcome` become dead code:

- **Delete:** `print_rerun_hint()`
- **Simplify** `SummaryOutcome::AllComplete` — remove `show_rerun_hint: bool` field
- **Simplify** `SummaryOutcome::ProjectErrors` — remove `show_rerun_hint: bool` field
- **Simplify** `determine_summary()` — remove all `show_rerun_hint` logic

Updated `SummaryOutcome`:

```rust
enum SummaryOutcome {
    /// Both phases completed successfully.
    AllComplete,
    /// CLI failed but project deps succeeded.
    ProjectOnlyComplete,
    /// Project dependency updates failed.
    ProjectErrors { errors: Vec<String> },
    /// CLI failed and not in a project.
    NoAction,
}
```

#### Updated UX flow

**Before (current):**

```
🔄 Checking for CLI updates...
📦 Downloading and installing... this may take a minute on slow networks.
✅ CLI updated to 0.9.3 (was 0.9.2)
📦 Updating Rust dependencies...
✅ Rust dependencies updated
📦 Updating npm dependencies...
✅ npm dependencies updated

💡 Note: Project dependencies were updated using the previous CLI version.
   Re-run `wavecraft update` to use the new CLI for dependency updates.

✨ All updates complete
```

**After (new):**

```
🔄 Checking for CLI updates...
📥 Downloading...
🔨 Compiling... this may take a minute.
✅ CLI updated to 0.9.3 (was 0.9.2)

🔄 Continuing with updated CLI...
✅ CLI updated to 0.9.3
📦 Updating Rust dependencies...
✅ Rust dependencies updated
📦 Updating npm dependencies...
✅ npm dependencies updated

✨ All updates complete
```

#### Failure mode: exec fails

If `reexec_with_new_binary()` fails (binary not found, permissions, etc.):

- Return `Err` which causes the process to exit with error
- User sees: `"Failed to re-exec CLI: ..."` plus `"Re-run 'wavecraft update' manually."`
- This is acceptable — exec failure is rare and the user has a clear recovery path

#### Failure mode: --skip-self with old binary

If a user manually passes `--skip-self`, it simply skips Phase 1 and runs Phase 2. This is harmless — no different from running `cargo update` + `npm update` manually. The flag is hidden from help text.

#### Tests

Update existing `determine_summary` unit tests to reflect removed `show_rerun_hint`:

```rust
#[test]
fn test_summary_all_complete_no_project() {
    let outcome = determine_summary(
        &SelfUpdateResult::AlreadyUpToDate,
        &ProjectUpdateResult::NotInProject,
    );
    assert_eq!(outcome, SummaryOutcome::AllComplete);
}

#[test]
fn test_summary_all_complete_with_project() {
    let outcome = determine_summary(
        &SelfUpdateResult::AlreadyUpToDate,
        &ProjectUpdateResult::Updated { errors: vec![] },
    );
    assert_eq!(outcome, SummaryOutcome::AllComplete);
}
```

Add new test for `--skip-self` flag:

```rust
// In cli/tests/update_command.rs
#[test]
fn test_update_skip_self_flag_hidden_from_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.args(["update", "--help"]);

    let output = cmd.output().expect("Failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("skip-self"),
        "--skip-self should be hidden from help output"
    );
}
```

---

## Item #3: Dev Build Profile Optimization (Conditional Spike)

### Problem

Rust compilation during development can be slow. Optimizing the dev build profile may improve iteration speed for plugin developers.

### Scope

This is a **timeboxed experiment** (max 4 hours). It should only proceed if Items #1 and #2 are completed within the first day of the 2-day timebox.

### Investigation Areas

1. **`profile.dev` settings** in `sdk-template/engine/Cargo.toml.template`:
   - `opt-level = 1` for dependencies (faster debug builds with some optimization)
   - `debug = "line-tables-only"` (smaller debug info)
   - `incremental = true` (usually default, verify it's enabled)

2. **`profile.dev.package."*"` overrides**:
   - `opt-level = 2` for third-party crates (compile once, run faster)
   - Keep `opt-level = 0` for the plugin crate itself (fastest recompilation)

3. **Linker optimization**:
   - `lld` on Linux, `zld` or `lld` on macOS (if available)
   - Check if `mold` is worth recommending

### Approach

```toml
# sdk-template/engine/Cargo.toml.template
[profile.dev]
opt-level = 0          # fastest recompilation for plugin code
incremental = true

[profile.dev.package."*"]
opt-level = 2          # optimize deps (compile once, run fast)
```

### Decision Gate

After implementing, measure compile times for a fresh build and incremental rebuild of a generated test plugin. If improvement is ≤10%, revert and defer to backlog.

### Output

If successful: changes to `sdk-template/engine/Cargo.toml.template` only.
If deferred: document findings in a comment on the roadmap task.

---

## Implementation Notes

The following deviations from this design were made during implementation:

1. **`stdout(Stdio::null())` instead of `Stdio::piped()`** — `cargo install` produces no useful stdout, so the implementation nulls it rather than piping. This avoids an unnecessary pipe allocation.

2. **`SelfUpdateResult` uses unit variants** — The LLD implied data-carrying variants. The implementation uses unit variants and prints version information inline in `update_cli()`, which simplifies the control flow without losing functionality.

3. **Item #3 (Dev Build Profile)** — Not implemented. Items #1 and #2 consumed the allocated timebox. Item #3 remains in the backlog as a future optimization.

---

## Implementation Order

```
Item #2 (Split progress)  ──►  Item #1 (Rerun elimination)  ──►  Item #3 (Build profile)
         Low risk                    Medium risk                   Conditional
```

**Rationale:** Item #2 modifies `update_cli()` internals. Item #1 modifies `run()` control flow and adds `--skip-self`. Doing #2 first means the streaming infrastructure is in place before #1 modifies the outer flow. Item #3 is independent and conditional.

---

## Files Changed

| File                                      | Item   | Change                                                                                                                                            |
| ----------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cli/src/commands/update.rs`              | #2     | Replace `start_cli_update_progress()` with streaming; add `InstallPhase`, `detect_phase()`, `stream_install_progress()`                           |
| `cli/src/commands/update.rs`              | #1     | Add `reexec_with_new_binary()`, `which_wavecraft()`; modify `run()` to accept `skip_self`; delete `print_rerun_hint()`; simplify `SummaryOutcome` |
| `cli/src/main.rs`                         | #1     | Add `skip_self` field to `Commands::Update`; pass to `run()`                                                                                      |
| `cli/Cargo.toml`                          | #1     | Add `which = "7"` dependency                                                                                                                      |
| `cli/tests/update_command.rs`             | #1, #2 | Add `--skip-self` hidden flag test; update any assertions about old messages                                                                      |
| `sdk-template/engine/Cargo.toml.template` | #3     | Add dev profile settings (conditional)                                                                                                            |

---

## Risks and Mitigations

| Risk                                           | Impact                                   | Mitigation                                                                                                   |
| ---------------------------------------------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Cargo stderr format changes between versions   | Progress phase detection breaks silently | Detect nothing → no phase messages shown (graceful degradation). `is_already_up_to_date()` is unchanged.     |
| `exec()` fails on obscure system configs       | User stuck after CLI update              | Return error with clear message; user can re-run manually. Failure is non-destructive.                       |
| `which` crate resolves wrong binary            | Exec runs unexpected binary              | Verify version after exec via `CURRENT_VERSION` check in `--skip-self` path. Extremely unlikely in practice. |
| Item #3 regresses build times for some configs | Slower dev experience                    | Timeboxed; measure before committing; easy revert (template-only change).                                    |

---

## Testing Strategy

### Unit Tests (in `update.rs`)

- `detect_phase()` — all phase patterns + edge cases
- `stream_install_progress()` — mock stderr with cursor, verify collected output
- `determine_summary()` — updated tests without `show_rerun_hint`

### Integration Tests (in `cli/tests/update_command.rs`)

- `--skip-self` hidden from help text
- Existing tests remain valid (behavior unchanged for `AlreadyUpToDate` path)

### Manual Verification

- Run `wavecraft update` outside a project → verify split progress messages
- Run `wavecraft update` inside a project with outdated CLI → verify reexec flow
- Run `wavecraft update` when already up-to-date → verify no regression
- Run `wavecraft update` with network failure → verify graceful degradation
