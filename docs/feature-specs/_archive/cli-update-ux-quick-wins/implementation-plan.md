# Implementation Plan: CLI Update UX Quick Wins

## Overview

Three scoped improvements to the `wavecraft update` CLI command, ordered by dependency and risk. Items #2 and #1 are core scope; Item #3 is conditional on remaining timebox. Total timebox: 2 working days.

## Requirements

- **Item #2**: Replace single "Downloading and installing" progress message with separate download/compile phase indicators by streaming cargo stderr
- **Item #1**: Eliminate the manual "re-run `wavecraft update`" step by re-executing the new binary after self-update
- **Item #3** (conditional): Optimize dev build profile in sdk-template for faster incremental compilation

## Architecture Changes

All changes are confined to the `cli/` crate:

- `cli/src/commands/update.rs` — streaming progress, reexec logic, simplified summary
- `cli/src/main.rs` — hidden `--skip-self` flag on Update variant
- `cli/Cargo.toml` — new `which` dependency
- `cli/tests/update_command.rs` — new integration tests
- `sdk-template/engine/Cargo.toml.template` — dev profile (conditional, Item #3)

## Implementation Steps

### Phase 1: Split Progress Messaging (Item #2)

#### Step 1.1 — Add `InstallPhase` enum and `detect_phase()` function

**File:** `cli/src/commands/update.rs`  
**Location:** After `SummaryOutcome` enum (after line 52)  
**Action:** Add new types for phase detection:

```rust
/// Progress phases during `cargo install`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum InstallPhase {
    Checking,
    Downloading,
    Compiling,
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

**Why:** Pure function, easily testable, no side effects. Foundation for streaming.  
**Dependencies:** None  
**Risk:** Low

---

#### Step 1.2 — Add `stream_install_progress()` function

**File:** `cli/src/commands/update.rs`  
**Location:** After `detect_phase()` (from Step 1.1)  
**Action:** Add the streaming function:

```rust
use std::io::{BufRead, BufReader};

/// Stream stderr from `cargo install`, showing phase-appropriate progress messages.
///
/// Returns the full stderr content for later analysis (version detection, etc.).
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
```

**Why:** Processes cargo stderr line-by-line and transitions between Downloading → Compiling.  
**Dependencies:** Step 1.1  
**Risk:** Low

---

#### Step 1.3 — Rewrite `update_cli()` to use streaming

**File:** `cli/src/commands/update.rs`  
**Location:** `update_cli()` function (lines 75–133)  
**Action:** Replace the `.output()` call with `.spawn()` + `stream_install_progress()`. The full replacement:

1. Remove the `Arc<AtomicBool>` setup and `start_cli_update_progress()` call (lines 78–82)
2. Replace `Command::new("cargo").args(["install", "wavecraft"]).output()` with:

   ```rust
   use std::process::Stdio;

   let mut child = match Command::new("cargo")
       .args(["install", "wavecraft"])
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .spawn()
   {
       Ok(child) => child,
       Err(e) => { /* same error handling as current */ }
   };

   let stderr_pipe = child.stderr.take().expect("stderr piped");
   let stderr_content = stream_install_progress(stderr_pipe);

   let status = match child.wait() {
       Ok(s) => s,
       Err(e) => { /* failure path */ }
   };
   ```

3. Replace `String::from_utf8_lossy(&output.stderr)` with `stderr_content` (already a String)
4. Replace `output.status` references with `status`
5. Remove `update_done.store(true, ...)` and `progress_handle.join()`

**Why:** Enables real-time phase detection from cargo's output stream.  
**Dependencies:** Step 1.2  
**Risk:** Low — same error handling, same version detection, just different I/O approach

---

#### Step 1.4 — Delete `start_cli_update_progress()`

**File:** `cli/src/commands/update.rs`  
**Location:** Lines 135–155  
**Action:** Delete the entire function. Also clean up now-unused imports:

- Remove `use std::io::{self, Write}` (replaced by `BufRead, BufReader` from Step 1.2)
- Remove `use std::sync::{ atomic::{AtomicBool, Ordering}, Arc }`
- Remove `use std::thread`
- Remove `use std::time::Duration`

**Why:** Replaced by streaming approach. The 3-second timer thread is no longer needed.  
**Dependencies:** Step 1.3  
**Risk:** Low

---

#### Step 1.5 — Add unit tests for phase detection and streaming

**File:** `cli/src/commands/update.rs`  
**Location:** Inside `#[cfg(test)] mod tests` (after line 353)  
**Action:** Add tests:

```rust
// --- InstallPhase detection tests ---

#[test]
fn test_detect_phase_downloading() {
    assert_eq!(
        detect_phase("  Downloading crates ..."),
        Some(InstallPhase::Downloading)
    );
}

#[test]
fn test_detect_phase_downloaded() {
    assert_eq!(
        detect_phase("  Downloaded wavecraft v0.9.2"),
        Some(InstallPhase::Downloading)
    );
}

#[test]
fn test_detect_phase_compiling() {
    assert_eq!(
        detect_phase("   Compiling wavecraft v0.9.2"),
        Some(InstallPhase::Compiling)
    );
}

#[test]
fn test_detect_phase_updating_index() {
    assert_eq!(detect_phase("   Updating crates.io index"), None);
}

#[test]
fn test_detect_phase_installing() {
    assert_eq!(detect_phase("  Installing /path/to/bin"), None);
}

#[test]
fn test_detect_phase_empty_line() {
    assert_eq!(detect_phase(""), None);
}

// --- stream_install_progress tests ---

#[test]
fn test_stream_collects_all_output() {
    let input = "  Downloading crates ...\n   Compiling wavecraft v0.9.2\n";
    let output = stream_install_progress(std::io::Cursor::new(input));
    assert!(output.contains("Downloading"));
    assert!(output.contains("Compiling"));
}

#[test]
fn test_stream_handles_already_installed() {
    let input = "  Ignored package `wavecraft v0.9.1` is already installed, use --force to override\n";
    let output = stream_install_progress(std::io::Cursor::new(input));
    assert!(output.contains("is already installed"));
}

#[test]
fn test_stream_empty_input() {
    let output = stream_install_progress(std::io::Cursor::new(""));
    assert!(output.is_empty());
}
```

**Why:** Validates phase detection patterns and the end-to-end streaming function.  
**Dependencies:** Steps 1.1, 1.2  
**Risk:** Low

---

#### Step 1.6 — Verify: `cargo test -p wavecraft`

**Action:** Run CLI crate tests to confirm all existing + new tests pass.  
**Dependencies:** Steps 1.1–1.5  
**Risk:** Low

---

### Phase 2: Rerun Elimination (Item #1)

#### Step 2.1 — Add `which` dependency to `cli/Cargo.toml`

**File:** `cli/Cargo.toml`  
**Location:** `[dependencies]` section  
**Action:** Add:

```toml
which = "7"
```

**Why:** Portable binary path resolution for finding the updated wavecraft binary.  
**Dependencies:** None  
**Risk:** Low

---

#### Step 2.2 — Add hidden `--skip-self` flag to `Commands::Update`

**File:** `cli/src/main.rs`  
**Location:** Lines 94–100 (the `Update` variant in `Commands` enum)  
**Action:** Change from unit variant to struct variant:

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

Update the match arm (line 153):

```rust
Commands::Update { skip_self } => {
    commands::update::run(skip_self)?;
}
```

**Why:** Hidden flag allows the re-exec'd binary to skip Phase 1.  
**Dependencies:** None  
**Risk:** Low — flag is hidden, existing behavior unchanged when not passed

---

#### Step 2.3 — Add `reexec_with_new_binary()` and `which_wavecraft()` functions

**File:** `cli/src/commands/update.rs`  
**Location:** After `update_cli()` function  
**Action:** Add:

```rust
/// Re-execute the newly installed CLI binary to continue with project deps.
///
/// Uses `exec()` on Unix to replace the process image. The new binary
/// runs `wavecraft update --skip-self`, which skips Phase 1 and runs
/// Phase 2 using the updated code.
fn reexec_with_new_binary() -> Result<()> {
    println!();
    println!("🔄 Continuing with updated CLI...");

    let binary = which_wavecraft()?;

    let mut cmd = Command::new(&binary);
    cmd.args(["update", "--skip-self"]);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        // exec() only returns on error
        bail!("Failed to re-exec CLI: {}", err);
    }

    #[cfg(not(unix))]
    {
        let status = cmd.status().context("Failed to re-exec CLI")?;
        std::process::exit(status.code().unwrap_or(1));
    }
}

/// Find the wavecraft binary path via PATH lookup.
fn which_wavecraft() -> Result<std::path::PathBuf> {
    which::which("wavecraft").context(
        "Could not find 'wavecraft' binary after update. \
         Re-run 'wavecraft update' manually."
    )
}
```

**Why:** Core reexec mechanism. Unix uses `exec()` for seamless process replacement.  
**Dependencies:** Step 2.1  
**Risk:** Medium — exec() replaces process image. Failure path is well-defined.

---

#### Step 2.4 — Modify `run()` to accept `skip_self` and use reexec

**File:** `cli/src/commands/update.rs`  
**Location:** `pub fn run()` (line 54)  
**Action:** Change signature and add reexec logic:

```rust
pub fn run(skip_self: bool) -> Result<()> {
    // Phase 1: CLI self-update (skipped on re-exec)
    let self_update_result = if skip_self {
        println!("✅ CLI updated to {}", CURRENT_VERSION);
        SelfUpdateResult::AlreadyUpToDate
    } else {
        update_cli()
    };

    // If CLI was updated, re-exec the new binary for Phase 2
    if matches!(self_update_result, SelfUpdateResult::Updated) {
        return reexec_with_new_binary();
    }

    // Phase 2: Project dependency update (context-dependent)
    let project_result = update_project_deps();

    // Summary and exit code
    print_summary(&self_update_result, &project_result)
}
```

**Why:** After self-update succeeds, the new binary takes over. No more stale-binary deps.  
**Dependencies:** Steps 2.2, 2.3  
**Risk:** Medium — changes control flow. All existing paths preserved when skip_self=false and CLI not updated.

---

#### Step 2.5 — Simplify `SummaryOutcome` and remove rerun hint

**File:** `cli/src/commands/update.rs`  
**Location:** `SummaryOutcome` enum (line 39), `determine_summary()` (line 251), `print_summary()` (line 285), `print_rerun_hint()` (line 319)  
**Action:**

1. **Simplify `SummaryOutcome`** — remove `show_rerun_hint` fields:

   ```rust
   enum SummaryOutcome {
       AllComplete,
       ProjectOnlyComplete,
       ProjectErrors { errors: Vec<String> },
       NoAction,
   }
   ```

2. **Simplify `determine_summary()`** — remove `show_rerun_hint` logic:

   ```rust
   fn determine_summary(
       self_update: &SelfUpdateResult,
       project: &ProjectUpdateResult,
   ) -> SummaryOutcome {
       let cli_failed = matches!(self_update, SelfUpdateResult::Failed);
       let in_project = matches!(project, ProjectUpdateResult::Updated { .. });

       let project_errors: &[String] = match project {
           ProjectUpdateResult::Updated { errors } => errors,
           ProjectUpdateResult::NotInProject => &[],
       };

       if !project_errors.is_empty() {
           return SummaryOutcome::ProjectErrors {
               errors: project_errors.to_vec(),
           };
       }

       if cli_failed && in_project {
           return SummaryOutcome::ProjectOnlyComplete;
       }

       if cli_failed && !in_project {
           return SummaryOutcome::NoAction;
       }

       SummaryOutcome::AllComplete
   }
   ```

3. **Simplify `print_summary()`** — remove rerun hint calls:

   ```rust
   fn print_summary(self_update: &SelfUpdateResult, project: &ProjectUpdateResult) -> Result<()> {
       let outcome = determine_summary(self_update, project);

       match outcome {
           SummaryOutcome::AllComplete => {
               println!();
               println!("✨ All updates complete");
           }
           SummaryOutcome::ProjectOnlyComplete => {
               println!();
               println!("✨ Project dependencies updated (CLI self-update skipped)");
           }
           SummaryOutcome::ProjectErrors { errors } => {
               bail!(
                   "Failed to update some dependencies:\n  {}",
                   errors.join("\n  ")
               );
           }
           SummaryOutcome::NoAction => {}
       }

       Ok(())
   }
   ```

4. **Delete `print_rerun_hint()`** (lines 319–322) — no longer needed.

5. **Note:** The `cli_updated` variable in `determine_summary()` is also removed. The `SelfUpdateResult::Updated` state is now handled by the reexec path in `run()` and never reaches `determine_summary()`.

**Why:** The reexec approach means project deps always run under the correct binary. The rerun hint is obsolete.  
**Dependencies:** Step 2.4  
**Risk:** Low — simplification only, removes dead code

---

#### Step 2.6 — Update `determine_summary` unit tests

**File:** `cli/src/commands/update.rs`  
**Location:** Tests starting at line 466  
**Action:** Update all `determine_summary` tests to match simplified `SummaryOutcome`:

1. `test_summary_all_complete_no_project` — change expected to `SummaryOutcome::AllComplete`
2. `test_summary_all_complete_with_project` — change expected to `SummaryOutcome::AllComplete`
3. `test_summary_updated_with_project_shows_rerun_hint` — **rename** to `test_summary_updated_with_project` and change expected to `SummaryOutcome::AllComplete` (no rerun hint needed; in practice this path won't happen due to reexec, but the pure function should handle it gracefully)
4. `test_summary_cli_failed_in_project` — unchanged
5. `test_summary_cli_failed_not_in_project` — unchanged
6. `test_summary_project_errors` — change expected to `SummaryOutcome::ProjectErrors { errors: ... }` (no `show_rerun_hint`)
7. `test_summary_updated_with_project_errors_shows_rerun_hint` — **rename** to `test_summary_updated_with_project_errors` and change expected similarly

**Why:** Tests must match the simplified enum.  
**Dependencies:** Step 2.5  
**Risk:** Low

---

#### Step 2.7 — Add integration test for hidden `--skip-self` flag

**File:** `cli/tests/update_command.rs`  
**Location:** After existing tests (after line 148)  
**Action:** Add:

```rust
#[test]
fn test_update_skip_self_flag_hidden_from_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("wavecraft"));
    cmd.args(["update", "--help"]);

    let output = cmd.output().expect("Failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("skip-self"),
        "--skip-self should be hidden from help output"
    );
}
```

**Why:** Validates the flag is hidden from user-facing help text.  
**Dependencies:** Step 2.2  
**Risk:** Low

---

#### Step 2.8 — Update integration test for output format

**File:** `cli/tests/update_command.rs`  
**Location:** `test_update_command_output_format` (line 125)  
**Action:** Update emoji check to include new emojis from split progress:

```rust
assert!(
    stdout.contains("📦") || stdout.contains("✅") || stdout.contains("❌")
        || stdout.contains("📥") || stdout.contains("🔨"),
    "Output should contain emoji indicators"
);
```

**Why:** Test should pass with both old and new emoji patterns.  
**Dependencies:** Phase 1  
**Risk:** Low

---

#### Step 2.9 — Verify: `cargo test -p wavecraft` and `cargo test --test update_command`

**Action:** Run both unit and integration tests.  
**Dependencies:** Steps 2.1–2.8  
**Risk:** Low

---

### Phase 3: Dev Build Profile Optimization (Item #3 — Conditional)

> **Gate:** Only proceed if Phases 1 and 2 are complete within day 1 of the 2-day timebox.

#### Step 3.1 — Baseline measurement

**Action:** Generate a test plugin and measure compile times:

```bash
cargo run --manifest-path cli/Cargo.toml -- create TestPlugin \
  --output target/tmp/test-build-profile
cd target/tmp/test-build-profile/engine

# Clean build timing
cargo clean && time cargo build 2>&1 | tail -1

# Incremental build timing (touch a source file)
touch src/lib.rs && time cargo build 2>&1 | tail -1
```

Record: clean build time, incremental build time.  
**Dependencies:** None  
**Risk:** Low

---

#### Step 3.2 — Add dev profile settings

**File:** `sdk-template/engine/Cargo.toml.template`  
**Location:** End of file (after `[features]` section)  
**Action:** Add:

```toml
[profile.dev]
opt-level = 0          # fastest recompilation for plugin code
incremental = true

[profile.dev.package."*"]
opt-level = 2          # optimize deps (compile once, run fast)
```

**Why:** Deps compile with optimizations once; plugin code compiles fast for iteration.  
**Dependencies:** None  
**Risk:** Low — template-only, easy revert

---

#### Step 3.3 — Measure improvement

**Action:** Repeat Step 3.1 measurements with the new profile. Compare clean and incremental build times.

**Decision gate:** If improvement is ≤10% on either metric, revert Step 3.2.  
**Dependencies:** Step 3.2  
**Risk:** Low

---

#### Step 3.4 — Validate generated project

**Action:**

```bash
cargo run --manifest-path cli/Cargo.toml -- create TestPlugin2 \
  --output target/tmp/test-build-profile2
cd target/tmp/test-build-profile2/engine
cargo clippy --all-targets -- -D warnings
```

**Why:** Ensure profile changes don't introduce clippy warnings or break the template.  
**Dependencies:** Step 3.2  
**Risk:** Low

---

## Testing Strategy

### Unit Tests (in `cli/src/commands/update.rs`)

| Test                                       | What it validates                                | Phase |
| ------------------------------------------ | ------------------------------------------------ | ----- |
| `test_detect_phase_downloading`            | Cargo "Downloading" line → `Downloading` phase   | 1     |
| `test_detect_phase_downloaded`             | Cargo "Downloaded" line → `Downloading` phase    | 1     |
| `test_detect_phase_compiling`              | Cargo "Compiling" line → `Compiling` phase       | 1     |
| `test_detect_phase_updating_index`         | Unrecognized line → `None`                       | 1     |
| `test_detect_phase_installing`             | "Installing" line → `None` (not a tracked phase) | 1     |
| `test_detect_phase_empty_line`             | Empty line → `None`                              | 1     |
| `test_stream_collects_all_output`          | Full stderr is collected from stream             | 1     |
| `test_stream_handles_already_installed`    | Up-to-date message passes through                | 1     |
| `test_stream_empty_input`                  | Empty stream → empty string                      | 1     |
| `test_summary_all_complete_no_project`     | Updated for simplified enum                      | 2     |
| `test_summary_all_complete_with_project`   | Updated for simplified enum                      | 2     |
| `test_summary_updated_with_project`        | Renamed, no rerun hint                           | 2     |
| `test_summary_project_errors`              | Updated, no show_rerun_hint field                | 2     |
| `test_summary_updated_with_project_errors` | Renamed, no show_rerun_hint field                | 2     |

### Integration Tests (in `cli/tests/update_command.rs`)

| Test                                          | What it validates                       | Phase |
| --------------------------------------------- | --------------------------------------- | ----- |
| `test_update_skip_self_flag_hidden_from_help` | `--skip-self` not visible in help       | 2     |
| `test_update_command_output_format` (updated) | Updated emoji list                      | 2     |
| Existing tests                                | Unchanged behavior for non-update paths | —     |

### Manual Verification

| Scenario                                                | Expected result                                                                                                                             |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| `wavecraft update` (CLI up-to-date, outside project)    | "✅ CLI is up to date" + "Not in a Wavecraft plugin project"                                                                                |
| `wavecraft update` (CLI up-to-date, inside project)     | "✅ CLI is up to date" + project deps update + "✨ All updates complete"                                                                    |
| `wavecraft update` (CLI outdated, inside project)       | "📥 Downloading..." → "🔨 Compiling..." → "✅ CLI updated" → "🔄 Continuing with updated CLI..." → project deps + "✨ All updates complete" |
| `wavecraft update` (network failure during self-update) | "⚠️ CLI self-update failed" + project deps still run                                                                                        |
| `wavecraft update --skip-self` (manual)                 | Skips self-update, runs project deps only                                                                                                   |

## Risks & Mitigations

| Risk                               | Likelihood | Impact                                        | Mitigation                                                     |
| ---------------------------------- | ---------- | --------------------------------------------- | -------------------------------------------------------------- |
| Cargo stderr format changes        | Low        | Phase messages not shown (silent degradation) | `is_already_up_to_date()` unchanged; phases are best-effort UX |
| `exec()` fails                     | Very low   | Error with recovery instructions              | `bail!()` with clear message; non-destructive                  |
| `which` resolves wrong binary      | Very low   | Wrong binary runs `update --skip-self`        | Prints `CURRENT_VERSION` on skip-self path; user can verify    |
| Dev profile regresses some configs | Low        | Slower builds for some users                  | Timeboxed; measured; template-only change; easy revert         |

## Success Criteria

- [ ] `cargo test -p wavecraft` passes (all unit tests)
- [ ] `cargo test --test update_command` passes (all integration tests)
- [ ] `cargo clippy -p wavecraft -- -D warnings` clean
- [ ] Split progress messages show during `cargo install` (manual verification)
- [ ] No "Re-run `wavecraft update`" message appears after CLI update
- [ ] `--skip-self` flag hidden from `wavecraft update --help`
- [ ] `cargo xtask ci-check` passes
