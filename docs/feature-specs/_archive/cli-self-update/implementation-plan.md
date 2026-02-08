# Implementation Plan: CLI Self-Update

**Feature:** CLI Self-Update (`wavecraft update`)  
**Target Version:** `0.9.1`  
**Branch:** `feature/cli-self-update`  
**Created:** 2026-02-08

---

## Related Documents

- [User Stories](./user-stories.md) — Requirements and acceptance criteria
- [Low-Level Design](./low-level-design-cli-self-update.md) — Technical design
- [Coding Standards](../../architecture/coding-standards.md) — Conventions

---

## Overview

Restructure `wavecraft update` from a project-only dependency updater into a two-phase command:
1. **Phase 1** — Self-update the CLI binary via `cargo install wavecraft` (runs from any directory)
2. **Phase 2** — Update project dependencies if inside a plugin project (existing behavior, preserved)

The implementation touches 3 files with no new dependencies. Existing tests are updated to match new behavior.

---

## Requirements

- `wavecraft update` self-updates the CLI before updating project deps
- Works from any directory (no longer errors outside a project)
- CLI self-update failure does not block project dependency updates
- Clear output messaging with old → new version display
- Help text updated to reflect new behavior
- Version bump: `0.9.0` → `0.9.1` (CLI `Cargo.toml` only)

---

## Implementation Steps

### Phase 1: Restructure `update.rs` — Core Logic

#### Step 1: Add enums and constants

**File:** `cli/src/commands/update.rs`

Add the following at the top of the file (after existing `use` statements):

1. Add `CURRENT_VERSION` constant:
   ```rust
   const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
   ```

2. Add `SelfUpdateResult` enum:
   ```rust
   enum SelfUpdateResult {
       Updated { old_version: String, new_version: String },
       AlreadyUpToDate,
       Failed(String),
   }
   ```

3. Add `ProjectUpdateResult` enum:
   ```rust
   enum ProjectUpdateResult {
       NotInProject,
       Updated { errors: Vec<String> },
   }
   ```

**Why:** These types structure the two-phase result handling, enabling the summary printer to compose messaging based on outcomes.

**Dependencies:** None  
**Risk:** Low

---

#### Step 2: Add `is_already_up_to_date()` helper

**File:** `cli/src/commands/update.rs`

Add a private function that parses `cargo install` stderr to detect the "already installed" case:

```rust
fn is_already_up_to_date(stderr: &str) -> bool {
    stderr.lines().any(|line| line.contains("is already installed"))
}
```

This checks for the stable marker string that cargo has used consistently from 1.60 through 1.84+. See LLD §5.1 for cargo output patterns.

**Dependencies:** None  
**Risk:** Low — if cargo changes this string, fallthrough behavior is still correct (see LLD §5.3)

---

#### Step 3: Add `get_installed_version()` helper

**File:** `cli/src/commands/update.rs`

Add a private function that queries the newly installed binary's version:

```rust
fn get_installed_version() -> Result<String> {
    let output = Command::new("wavecraft")
        .arg("--version")
        .output()
        .context("Failed to run 'wavecraft --version'")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout
        .trim()
        .strip_prefix("wavecraft ")
        .unwrap_or(stdout.trim())
        .to_string();

    Ok(version)
}
```

This invokes the **disk binary** (not the running process), so after `cargo install` completes it reflects the newly installed version. The clap `--version` flag outputs `wavecraft X.Y.Z\n`.

**Dependencies:** None  
**Risk:** Low

---

#### Step 4: Add `update_cli()` function

**File:** `cli/src/commands/update.rs`

Add the Phase 1 function that performs the self-update:

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
                "Failed to run 'cargo install'. Is cargo installed? ({})", e
            ));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return SelfUpdateResult::Failed(format!("cargo install failed: {}", stderr.trim()));
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if is_already_up_to_date(&stderr) {
        println!("✅ CLI is up to date ({})", CURRENT_VERSION);
        return SelfUpdateResult::AlreadyUpToDate;
    }

    match get_installed_version() {
        Ok(new_version) => {
            println!("✅ CLI updated to {} (was {})", new_version, CURRENT_VERSION);
            SelfUpdateResult::Updated {
                old_version: CURRENT_VERSION.to_string(),
                new_version,
            }
        }
        Err(_) => {
            println!("✅ CLI updated (was {})", CURRENT_VERSION);
            SelfUpdateResult::Updated {
                old_version: CURRENT_VERSION.to_string(),
                new_version: "unknown".to_string(),
            }
        }
    }
}
```

**Key design decisions:**
- Uses `Command::output()` (captures all output) to parse the "already installed" marker from stderr
- Never returns `Err` — all failures are captured as `SelfUpdateResult::Failed`
- Falls through to `get_installed_version()` when parsing is ambiguous

**Dependencies:** Steps 2, 3  
**Risk:** Medium — relies on `cargo install` output parsing. Mitigated by LLD §5.3 fallthrough behavior.

---

#### Step 5: Refactor existing `run()` into `update_project_deps()`

**File:** `cli/src/commands/update.rs`

Rename and restructure the current `run()` function:

1. **Extract** the core project detection and update logic from the current `run()` into a new `fn update_project_deps() -> ProjectUpdateResult`
2. **Remove the `bail!`** for "Not a Wavecraft plugin project" — replace with returning `ProjectUpdateResult::NotInProject` with an informational message
3. **Keep** `update_rust_deps()` and `update_npm_deps()` unchanged
4. The function should print `ℹ️  Not in a Wavecraft plugin project — skipping dependency updates.` when not in a project, instead of erroring

Refer to LLD §3.1 for the exact implementation of `update_project_deps()`.

**Dependencies:** None (but logically follows step 1 for the enum)  
**Risk:** Low — preserves existing behavior while changing error-to-info conversion

---

#### Step 6: Add `print_summary()` function

**File:** `cli/src/commands/update.rs`

Add the summary function that composes final output based on both phase results:

```rust
fn print_summary(
    self_update: &SelfUpdateResult,
    project: &ProjectUpdateResult,
) -> Result<()>
```

This function handles:
- The "re-run" hint when CLI was updated AND deps were run (User Story 4)
- `SelfUpdateResult::Failed` messaging with manual command suggestion (User Story 3)
- Final "✨ All updates complete" or error bail with collected errors
- Exit code logic per LLD §4.3

Refer to LLD §3.1 `print_summary()` for the full implementation.

**Dependencies:** Steps 1, 4, 5  
**Risk:** Low

---

#### Step 7: Write new `run()` orchestrator

**File:** `cli/src/commands/update.rs`

Replace the old `run()` with the two-phase orchestrator:

```rust
pub fn run() -> Result<()> {
    let self_update_result = update_cli();
    let project_result = update_project_deps();
    print_summary(&self_update_result, &project_result)
}
```

The public API (`pub fn run()`) remains identical — `main.rs` still calls `commands::update::run()`.

**Dependencies:** Steps 4, 5, 6  
**Risk:** Low

---

### Phase 2: Update Help Text

#### Step 8: Update `Commands::Update` help text in `main.rs`

**File:** `cli/src/main.rs`

Change the `Update` variant from:

```rust
/// Update all project dependencies (Rust crates + npm packages)
Update,
```

To:

```rust
/// Update the CLI and project dependencies (Rust crates + npm packages)
#[command(
    long_about = "Update the Wavecraft CLI to the latest version, then update Rust crates \
    and npm packages if run from a plugin project directory.\n\n\
    Can be run from any directory. When outside a project, only the CLI is updated."
)]
Update,
```

This satisfies User Story 5 — the short help shows "Update the CLI and project dependencies" and the long help explains the two-phase behavior and any-directory support.

**Dependencies:** None  
**Risk:** Low

---

### Phase 3: Version Bump

#### Step 9: Bump CLI version to `0.9.1`

**File:** `cli/Cargo.toml`

Change:
```toml
version = "0.9.0"
```
To:
```toml
version = "0.9.1"
```

**Important:** Only bump the CLI crate version. Do **not** change `engine/Cargo.toml` workspace version (stays at `0.9.0`). The CLI has its own independent version — see coding standards §SDK Distribution Versioning.

**Dependencies:** None (can be done at any step, but placing it here ensures tests validate the new version)  
**Risk:** Low

---

### Phase 4: Unit Tests

#### Step 10: Add unit tests for `is_already_up_to_date()`

**File:** `cli/src/commands/update.rs`

Add the following tests to the existing `#[cfg(test)] mod tests` block:

1. `test_is_already_up_to_date_true` — Pass cargo's "already installed" stderr output, assert returns `true`
2. `test_is_already_up_to_date_false_new_install` — Pass cargo's "Installing..." output, assert returns `false`
3. `test_is_already_up_to_date_empty` — Pass empty string, assert returns `false`
4. `test_is_already_up_to_date_with_prefix` — Pass output with the `Ignored` prefix cargo uses, assert returns `true`

Use the exact cargo output patterns from LLD §5.1 as test data.

**Dependencies:** Step 2  
**Risk:** Low

---

#### Step 11: Add unit tests for project detection with new enum

**File:** `cli/src/commands/update.rs`

The existing three unit tests (`test_detects_engine_only`, `test_detects_ui_only`, `test_detects_both`) should still pass since they test raw file existence checks. Keep them as-is.

Optionally add a test that calls a testable version of the project detection logic to verify `ProjectUpdateResult::NotInProject` is returned when no markers exist. Note: the current `update_project_deps()` uses `Path::new(...)` which checks the CWD, making it hard to unit-test in isolation. This is acceptable — the integration tests (step 12) cover this behavior end-to-end.

**Dependencies:** Step 5  
**Risk:** Low

---

### Phase 5: Integration Tests

#### Step 12: Update existing integration tests

**File:** `cli/tests/update_command.rs`

The following existing tests need updates:

1. **`test_help_shows_update_command`** — Update the assertion from:
   ```rust
   assert!(stdout.contains("Update all project dependencies"));
   ```
   To:
   ```rust
   assert!(stdout.contains("Update the CLI and project dependencies"));
   ```
   (Matches the new doc comment on `Commands::Update`)

2. **`test_update_outside_plugin_project`** — This test currently asserts `!output.status.success()` and checks for "Not a Wavecraft plugin project" error. With the new behavior, running outside a project is **no longer an error**. Update to:
   - Assert `output.status.success()` (exit code 0)
   - Assert output contains "Not in a Wavecraft plugin project" (info message, not error)
   - Assert output contains "skipping dependency updates"
   - Note: This test will trigger the actual `cargo install wavecraft` self-update (network call). If this is undesirable in CI, consider adding `#[ignore]` and testing only the help text. **However**, the LLD intentionally chose not to mock this for integration tests — see LLD §7.3.

3. **`test_update_detects_engine_directory`** — This test creates a temp dir with engine/ and runs `wavecraft update`. With the new code, Phase 1 (self-update) runs first, so the output will include CLI update messages before the Rust dep messages. Update assertions to be more lenient or check that both phases appear in combined output.

4. **`test_update_detects_ui_directory`** — Same consideration as above.

5. **`test_update_command_output_format`** — Should still pass as-is (checks for emoji indicators).

**Dependencies:** Steps 7, 8  
**Risk:** Medium — integration tests will trigger `cargo install wavecraft` which requires network. The tests that create temp dirs and run update will now hit self-update first. Consider whether to `#[ignore]` the network-dependent tests or accept the network dependency.

---

#### Step 13: Add integration test for update help long text

**File:** `cli/tests/update_command.rs`

Add a new test:

```rust
#[test]
fn test_update_help_shows_any_directory_info() {
    let mut cmd = Command::new(cargo_bin!("wavecraft"));
    cmd.args(["update", "--help"]);

    let output = cmd.output().expect("Failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("any directory"));
    assert!(stdout.contains("CLI"));
}
```

This validates User Story 5 — the `--help` output mentions "any directory" and "CLI".

**Dependencies:** Step 8  
**Risk:** Low

---

### Phase 6: Validation

#### Step 14: Run `cargo xtask ci-check`

Run the full CI check locally to validate all changes:

```bash
cargo xtask ci-check
```

This runs:
- Linting: `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, ESLint, Prettier
- Tests: `cargo test --workspace`, Vitest

All checks must pass. Use `cargo xtask ci-check --fix` to auto-fix any formatting issues.

**Dependencies:** All previous steps  
**Risk:** Low

---

#### Step 15: Manual verification

Run the following manual checks (for the Tester agent):

1. **Version flag:**
   ```bash
   cargo run --manifest-path cli/Cargo.toml -- --version
   # Expected: "wavecraft 0.9.1"
   ```

2. **Help text:**
   ```bash
   cargo run --manifest-path cli/Cargo.toml -- update --help
   # Expected: mentions CLI update, project deps, any directory
   ```

3. **Outside project:**
   ```bash
   cd /tmp && cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- update
   # Expected: "🔄 Checking for CLI updates..."
   # Expected: "ℹ️  Not in a Wavecraft plugin project — skipping dependency updates."
   ```

4. **Inside project (use test plugin):**
   ```bash
   cargo run --manifest-path cli/Cargo.toml -- create TestUpdate --output target/tmp/test-update
   cd target/tmp/test-update
   cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- update
   # Expected: CLI check + Rust deps + npm deps
   ```

5. **Help listing:**
   ```bash
   cargo run --manifest-path cli/Cargo.toml -- --help
   # Expected: "update  Update the CLI and project dependencies..."
   ```

**Dependencies:** All previous steps  
**Risk:** Low

---

## Testing Strategy

### Unit Tests (`cli/src/commands/update.rs`)

| Test | Validates |
|------|-----------|
| `test_is_already_up_to_date_true` | Cargo "already installed" detection |
| `test_is_already_up_to_date_false_new_install` | New install detection |
| `test_is_already_up_to_date_empty` | Edge case: empty input |
| `test_is_already_up_to_date_with_prefix` | Cargo `Ignored` prefix handling |
| `test_detects_engine_only` (existing) | Engine-only project detection |
| `test_detects_ui_only` (existing) | UI-only project detection |
| `test_detects_both` (existing) | Full project detection |

### Integration Tests (`cli/tests/update_command.rs`)

| Test | Validates |
|------|-----------|
| `test_help_shows_update_command` (updated) | Short help text reflects new wording |
| `test_update_outside_plugin_project` (updated) | No longer errors; exit 0 with info message |
| `test_update_help_shows_any_directory_info` (new) | Long help mentions any-directory support |
| `test_update_detects_engine_directory` (updated) | Engine dep update still attempted |
| `test_update_detects_ui_directory` (updated) | npm dep update still attempted |
| `test_update_command_output_format` (existing) | Emoji indicators present |

### Manual Testing

See Step 15 above for the full manual test checklist.

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Integration tests trigger network calls via `cargo install` | High | CI slowdown (~30-60s) | Accept for now; tests already use `cargo_bin!` which requires build. Consider `#[ignore]` for network tests if they cause CI flakiness |
| `cargo install` output format changes | Low | Cosmetic — false "updated" message | Fallthrough to `get_installed_version()` is correct regardless |
| Existing integration test `test_update_outside_plugin_project` breaks | Certain | Blocks CI | Explicitly updated in Step 12 |
| `test_update_detects_engine_directory` now includes self-update output | Certain | Assertion may break | Updated in Step 12 to account for Phase 1 output |

---

## Success Criteria

- [ ] `wavecraft update` from outside a project: exits 0, shows CLI update status, info about not being in a project
- [ ] `wavecraft update` from inside a project: exits 0, shows CLI update status + dependency updates
- [ ] CLI self-update failure: warns but continues to project deps
- [ ] `wavecraft --version` shows `0.9.1`
- [ ] `wavecraft update --help` mentions CLI update and any-directory support
- [ ] All existing unit tests pass
- [ ] All updated integration tests pass
- [ ] `cargo xtask ci-check` passes clean

---

## File Change Summary

| File | Change | Lines Changed (est.) |
|------|--------|---------------------|
| `cli/src/commands/update.rs` | Restructure into two-phase; add enums, 4 new functions, 4 new unit tests | ~150 net new |
| `cli/src/main.rs` | Update `Commands::Update` doc comment + `long_about` | ~5 |
| `cli/Cargo.toml` | Version `0.9.0` → `0.9.1` | 1 |
| `cli/tests/update_command.rs` | Update 4 existing tests, add 1 new test | ~30 |

**No new files. No new dependencies.**
