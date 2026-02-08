# Test Plan: CLI Self-Update

## Overview
- **Feature**: CLI Self-Update (`wavecraft update` two-phase command)
- **Spec Location**: `docs/feature-specs/cli-self-update/`
- **Date**: 2026-02-08
- **Tester**: Tester Agent
- **Branch**: `feature/cli-self-update`
- **Target Version**: `0.9.1`

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 12 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask ci-check` passes (all lint + tests) — **12.6s, all green**
- [x] CLI unit tests pass (4 `is_already_up_to_date` tests + 3 project detection tests)
- [x] CLI integration tests pass (2 non-ignored: help text, update help)
- [x] No new compiler warnings introduced (only pre-existing deprecation warning for `cargo_bin`)

## Automated Test Results

### CI Check (`cargo xtask ci-check`)

- ✅ Rust formatting: PASSED
- ✅ Clippy: PASSED
- ✅ ESLint: PASSED
- ✅ Prettier: PASSED
- ✅ Engine tests: 155 passed
- ✅ UI tests: 28 passed

### CLI Unit Tests (`cargo test --manifest-path cli/Cargo.toml`)

- ✅ `test_is_already_up_to_date_true` — detects cargo "already installed" output
- ✅ `test_is_already_up_to_date_false_new_install` — correctly returns false for new installs
- ✅ `test_is_already_up_to_date_empty` — correctly returns false for empty string
- ✅ `test_is_already_up_to_date_with_prefix` — handles cargo prefix whitespace
- ✅ `test_detects_engine_only` — project detection with engine only
- ✅ `test_detects_ui_only` — project detection with ui only
- ✅ `test_detects_both` — project detection with both engine and ui

### CLI Integration Tests

- ✅ `test_help_shows_update_command` — "update" appears in help
- ✅ `test_update_help_shows_any_directory_info` — help mentions "any directory" and "CLI"
- ⏭️ `test_update_outside_plugin_project` — IGNORED (requires network/crates.io)
- ⏭️ `test_update_detects_engine_directory` — IGNORED (requires network/crates.io)
- ⏭️ `test_update_detects_ui_directory` — IGNORED (requires network/crates.io)
- ⏭️ `test_update_command_output_format` — IGNORED (requires network/crates.io)

### Pre-Existing Failure (Not Related)

- ❌ `template::tests::test_apply_local_dev_overrides` — FAILS on `main` too (missing `wavecraft-dev-server` path dependency). **Pre-existing bug, not introduced by this feature.**

---

## Test Cases

### TC-001: Version Flag Shows 0.9.1

**Description**: Verify `--version` displays the bumped version.

**Steps**:
1. Run `cargo run --manifest-path cli/Cargo.toml -- --version`

**Expected Result**: Output `wavecraft 0.9.1`

**Status**: ✅ PASS

**Actual Result**: `wavecraft 0.9.1`

---

### TC-002: Main Help Text Shows Updated Description

**Description**: Verify `--help` shows the update command with "CLI and project dependencies" description.

**Steps**:
1. Run `cargo run --manifest-path cli/Cargo.toml -- --help`

**Expected Result**: Commands list includes `update  Update the CLI and project dependencies (Rust crates + npm packages)`

**Status**: ✅ PASS

**Actual Result**: Matches expected. Full output:
```
Commands:
  create  Create a new plugin project from the Wavecraft template
  start   Start development servers (WebSocket + UI)
  update  Update the CLI and project dependencies (Rust crates + npm packages)
  help    Print this message or the help of the given subcommand(s)
```

---

### TC-003: Update Help Text Describes Two-Phase Behavior

**Description**: Verify `update --help` explains CLI update + project deps and mentions "any directory".

**Steps**:
1. Run `cargo run --manifest-path cli/Cargo.toml -- update --help`

**Expected Result**: Long help mentions CLI update, project dependencies, and "any directory"

**Status**: ✅ PASS

**Actual Result**:
```
Update the Wavecraft CLI to the latest version, then update Rust crates and npm
packages if run from a plugin project directory.

Can be run from any directory. When outside a project, only the CLI is updated.

Usage: wavecraft update

Options:
  -h, --help  Print help (see a summary with '-h')
```

---

### TC-004: Update from Outside a Project (/tmp)

**Description**: Verify running `wavecraft update` from a non-project directory succeeds with CLI-only update and info message about no project.

**Steps**:
1. `cd /tmp`
2. Run the wavecraft binary with `update`

**Expected Result**: Shows "Checking for CLI updates...", shows CLI status, shows "Not in a Wavecraft plugin project" info, exits 0.

**Status**: ✅ PASS

**Actual Result**:
```
🔄 Checking for CLI updates...
✅ CLI is up to date (0.9.1)

ℹ️  Not in a Wavecraft plugin project — skipping dependency updates.
   Run this command from a project root to also update Rust and npm dependencies.

✨ All updates complete
```
Exit code: 0

---

### TC-005: Update from SDK Repo Root (Has engine/ and ui/)

**Description**: Verify running from a directory with `engine/Cargo.toml` and `ui/package.json` triggers both CLI update and project dependency updates.

**Steps**:
1. `cd /Users/ronhouben/code/private/wavecraft`
2. Run wavecraft binary with `update`

**Expected Result**: CLI check + Rust deps update + npm deps update + "All updates complete"

**Status**: ✅ PASS

**Actual Result**: CLI reported up to date, Rust dependencies updated (14 packages locking), npm dependencies updated. All showed correct emoji indicators. Final: "✨ All updates complete"

---

### TC-006: Update from Inside Generated Plugin Project

**Description**: Verify the full flow works inside a `wavecraft create`-generated project.

**Steps**:
1. Generate test project: `wavecraft create TestSelfUpdate --output target/tmp/test-self-update`
2. `cd target/tmp/test-self-update`
3. Run wavecraft binary with `update`

**Expected Result**: CLI check + Rust deps + npm deps all run successfully

**Status**: ✅ PASS

**Actual Result**: All three phases completed. Rust fetched 448 packages, npm installed 395 packages. Final: "✨ All updates complete"

---

### TC-007: Exit Code is 0 for Successful Operations

**Description**: Verify exit code is 0 for both "outside project" and "inside project" successful runs.

**Steps**:
1. Run update from `/tmp`, check `$?`
2. Run update from generated project, check `$?`

**Expected Result**: Both return exit code 0

**Status**: ✅ PASS

**Actual Result**: Both `EXIT CODE: 0`

---

### TC-008: Error Handling — Code Review

**Description**: Verify the code structure ensures CLI self-update failure cannot block project dependency updates.

**Steps**:
1. Review `update_cli()` — must never return `Err` or panic
2. Review `run()` — must call `update_project_deps()` unconditionally
3. Review `print_summary()` — must handle `Failed` + project success case

**Expected Result**: 
- `update_cli()` returns `SelfUpdateResult` enum (all failures captured as `Failed` variant)
- `run()` calls both phases sequentially with no early return on Phase 1 failure
- `print_summary()` shows "Project dependencies updated (CLI self-update skipped)" when CLI fails but deps succeed

**Status**: ✅ PASS

**Actual Result**: Code review confirms all three conditions are met. Key observations:
- `update_cli()` uses `match` on `Command::new("cargo")` — `Err` returns `SelfUpdateResult::Failed`, does not propagate
- Non-zero exit from `cargo install` also returns `SelfUpdateResult::Failed`
- `run()` at lines 30-36: calls `update_cli()` then `update_project_deps()` unconditionally
- `print_summary()` at line 199-201: handles `cli_failed && in_project` with appropriate message

---

### TC-009: Integration Tests Pass

**Description**: Verify all non-ignored integration tests in `cli/tests/update_command.rs` pass.

**Steps**:
1. Run `cargo test --manifest-path cli/Cargo.toml --test update_command`

**Expected Result**: 2 non-ignored tests pass, 4 network-dependent tests ignored

**Status**: ✅ PASS

**Actual Result**: `test result: ok. 2 passed; 0 failed; 4 ignored; 0 measured; 0 filtered out`

---

### TC-010: Version Flag Integration Tests Pass

**Description**: Verify all version-related integration tests still pass.

**Steps**:
1. Run `cargo test --manifest-path cli/Cargo.toml --test version_flag`

**Expected Result**: All 4 version tests pass

**Status**: ✅ PASS

**Actual Result**: `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

---

### TC-011: Changes Scoped Correctly

**Description**: Verify only the expected files were changed in this feature branch.

**Steps**:
1. Run `git diff --stat main..feature/cli-self-update`

**Expected Result**: Changes limited to `cli/Cargo.toml`, `cli/Cargo.lock`, `cli/src/commands/update.rs`, `cli/src/main.rs`, `cli/tests/update_command.rs`, and documentation.

**Status**: ✅ PASS

**Actual Result**:
```
cli/Cargo.lock                                     |   2 +-
cli/Cargo.toml                                     |   2 +-
cli/src/commands/update.rs                         | 210 ++++++++++++++++++--
cli/src/main.rs                                    |   7 +-
cli/tests/update_command.rs                        |  38 +++-
docs/.../cli-self-update/implementation-progress.md|  42 +++++
```
6 files changed, 280 insertions, 21 deletions. No unexpected files touched.

---

### TC-012: Coding Standards Compliance

**Description**: Verify the implementation follows project coding standards.

**Steps**:
1. Check doc comments on public and private functions
2. Verify `globalThis` not used (N/A for Rust)
3. Verify error handling uses `Result<T>` / `anyhow`
4. Check naming conventions (snake_case functions, PascalCase enums)
5. Verify no `unwrap()` without justification in production code
6. Verify structured logging standards (N/A — CLI uses println for user-facing output)

**Expected Result**: All coding standards met

**Status**: ✅ PASS

**Actual Result**:
- All functions have `///` doc comments explaining purpose
- `update_cli()`, `is_already_up_to_date()`, `get_installed_version()`, `update_project_deps()`, `print_summary()` all properly documented
- Error handling uses `anyhow::Result`, `anyhow::bail!`, and `anyhow::Context`
- Naming follows conventions: `SelfUpdateResult`, `ProjectUpdateResult` (PascalCase), `update_cli`, `is_already_up_to_date` (snake_case)
- One `unwrap_or` in `get_installed_version()` for version parsing fallback (acceptable — non-critical path)
- No `unwrap()` in production code paths

---

## Acceptance Criteria Verification

### User Story 1: Self-Update the CLI First
- [x] `wavecraft update` runs `cargo install wavecraft` as the first step (confirmed in code + TC-004, TC-005, TC-006)
- [x] Shows "🔄 Checking for CLI updates..." (confirmed in TC-004)
- [x] If already up-to-date, shows "✅ CLI is up to date (X.Y.Z)" (confirmed in TC-004, TC-005)
- [x] If CLI update fails, shows error but continues (confirmed via code review TC-008)
- [x] CLI self-update happens before any project dependency updates (confirmed in code — Phase 1 before Phase 2)
- [ ] If a newer version was installed, shows "✅ CLI updated to X.Y.Z (was A.B.C)" — **not testable** (requires publishing a newer version to crates.io)

### User Story 2: Work from Any Directory
- [x] Works from any directory (confirmed in TC-004 from /tmp)
- [x] If outside a project, only updates CLI (confirmed in TC-004)
- [x] If inside a project, updates CLI and project deps (confirmed in TC-005, TC-006)
- [x] Shows clear messaging based on context (confirmed in TC-004, TC-005)
- [x] Exit code is 0 if all applicable updates succeed (confirmed in TC-007)

### User Story 3: Graceful Handling When cargo Is Unavailable
- [x] If `cargo install` fails, shows clear error with reason (confirmed via code review TC-008)
- [x] Failure to self-update does not prevent project dependency updates (confirmed via code review TC-008)
- [x] Suggests manual alternative (code prints "Run 'cargo install wavecraft' manually to update the CLI")
- [ ] Exit code reflects partial success — **not fully testable** without simulating cargo failure

### User Story 4: Version Change Notification
- [x] Re-run suggestion shown when CLI was updated and in project (confirmed via code review of `print_summary()`)
- [x] No re-run suggestion when CLI is already up-to-date (confirmed in TC-005)
- [x] Version comparison uses `env!("CARGO_PKG_VERSION")` (confirmed at line 6 of update.rs)
- [ ] Old → new version display — **not testable** (requires publishing a newer version)

### User Story 5: Help Text Reflects New Behavior
- [x] `wavecraft --help` shows updated description (confirmed in TC-002)
- [x] `wavecraft update --help` explains two-phase update (confirmed in TC-003)
- [x] Help text mentions running from any directory (confirmed in TC-003)

---

## Issues Found

No issues were found specific to this feature.

### Pre-Existing Issue (Not Introduced by This Feature)

#### Pre-Existing: `test_apply_local_dev_overrides` fails on main

- **Severity**: Low (pre-existing, not related to this feature)
- **Test Case**: N/A (not a test case in this plan)
- **Description**: `template::tests::test_apply_local_dev_overrides` expects a `wavecraft-dev-server` path dependency in generated Cargo.toml, but it's not present. Fails identically on the `main` branch.
- **Evidence**: Verified by checking out `main` and running the same test — same failure.
- **Action**: No action required for this feature. Should be tracked as a separate bug.

### Minor Observations

1. **Deprecated `cargo_bin` import**: Both `update_command.rs` and `version_flag.rs` use `assert_cmd::cargo::cargo_bin` which triggers a deprecation warning. Pre-existing, not introduced by this feature.

2. **`SelfUpdateResult` simplification**: The implementation uses unit variants (`Updated`, `Failed`) instead of the LLD's data-carrying variants (`Updated { old_version, new_version }`, `Failed(String)`). This is a valid simplification — version info is printed directly in `update_cli()` rather than stored in the enum. Behavior matches requirements.

3. **No summary line when CLI fails outside project**: When `cli_failed && !in_project`, no "✨" summary line is printed — only the individual warning and info messages. This is acceptable per the LLD design (comment at line 206) and the output is still clear.

---

## Testing Notes

- The "CLI updated to X.Y.Z (was A.B.C)" message path could not be exercised because crates.io already has 0.9.1 installed. This path requires a newer version to be published first. However, the code logic is straightforward and covered by the unit tests for `is_already_up_to_date`.
- The `cargo install wavecraft` failure path could not be exercised directly without mocking system commands. The code review (TC-008) confirms the error handling is correctly structured.
- All 4 network-dependent integration tests are correctly marked `#[ignore]` with clear rationale comments.

---

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] No issues found specific to this feature
- [x] Pre-existing issues documented but not blocking
- [x] **Ready for release: YES**
