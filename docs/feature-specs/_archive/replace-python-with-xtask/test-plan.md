# Test Plan: Replace Python Validation with xtask

## Overview
- **Feature**: Replace inline Python heredoc scripts in CD pipeline with `cargo xtask validate-cli-deps`
- **Branch**: `feature/replace-python-with-xtask`
- **Spec Location**: `docs/feature-specs/replace-python-with-xtask/`
- **Date**: 2026-02-09
- **Re-test Date**: 2026-02-09
- **Tester**: Tester Agent

> **Re-test note**: This is a re-test pass after QA findings were fixed by the Coder.
> QA report: `docs/feature-specs/replace-python-with-xtask/QA-report.md`
> Fixes verified: Finding 1 (High), Finding 2 (Medium), Finding 3 (Medium), Finding 5 (Low).

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 11 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask ci-check` passes (all lint + tests)

## Test Cases

### TC-001: `cargo xtask validate-cli-deps` exits 0

**Description**: Run the new xtask command and verify it succeeds on the current repo state.

**Steps**:
1. `cd engine && cargo xtask validate-cli-deps`
2. Verify exit code is 0
3. Verify output lists all 3 wavecraft-* deps as validated

**Expected Result**: Command exits 0 and prints "All 3 dependencies validated: wavecraft-bridge, wavecraft-dev-server, wavecraft-protocol".

**Status**: ✅ PASS

**Actual Result**: Exit code 0. Output:
```
Discovered 3 wavecraft-* dependencies in cli/Cargo.toml
All 3 dependencies validated: wavecraft-bridge, wavecraft-dev-server, wavecraft-protocol
```

---

### TC-002: `cargo xtask validate-cli-deps --verbose` shows details

**Description**: Run with `--verbose` flag and verify per-dependency detail output.

**Steps**:
1. `cd engine && cargo xtask validate-cli-deps --verbose`
2. Verify each dependency shows version and publishability status

**Expected Result**: Output includes lines like `✓ wavecraft-bridge — version: 0.11.0, publishable: yes` for each dep.

**Status**: ✅ PASS

**Actual Result**: Exit code 0. Verbose output:
```
  ✓ wavecraft-bridge — version: 0.11.0, publishable: yes
  ✓ wavecraft-dev-server — version: 0.11.0, publishable: yes
  ✓ wavecraft-protocol — version: 0.11.0, publishable: yes
All 3 dependencies validated: wavecraft-bridge, wavecraft-dev-server, wavecraft-protocol
```

---

### TC-003: Unit tests pass

**Description**: Run xtask unit tests and verify new validate_cli_deps tests all pass.

**Steps**:
1. `cd engine && cargo test -p xtask -- validate_cli_deps`
2. Verify all `validate_cli_deps::tests::*` tests pass

**Expected Result**: 9 validate_cli_deps tests pass (6 original + 3 QA-fix tests).

**Status**: ✅ PASS

**Actual Result**: All 9 tests passed (re-test 2026-02-09):
```
test commands::validate_cli_deps::tests::test_validate_dependency_missing_crate_dir ... ok
test commands::validate_cli_deps::tests::test_detect_unpublishable_crate ... ok
test commands::validate_cli_deps::tests::test_detect_missing_version ... ok
test commands::validate_cli_deps::tests::test_no_wavecraft_deps_found ... ok
test commands::validate_cli_deps::tests::test_inline_table_format ... ok
test commands::validate_cli_deps::tests::test_all_passing_synthetic ... ok
test commands::validate_cli_deps::tests::test_parse_real_cli_cargo_toml ... ok
test commands::validate_cli_deps::tests::test_validate_dependency_malformed_toml ... ok
test commands::validate_cli_deps::tests::test_publish_key_absent_is_publishable ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out
```

---

### TC-004: `cargo xtask ci-check` passes

**Description**: Run full CI check to verify no regressions.

**Steps**:
1. `cd engine && cargo xtask ci-check`
2. Verify all lint and test phases pass

**Expected Result**: All checks pass with exit code 0.

**Status**: ✅ PASS

**Actual Result**: All lint and test phases passed with exit code 0 in 26.6s (re-test 2026-02-09). Engine: 42 xtask tests + all crate tests. UI: 28 Vitest tests.

---

### TC-005: No Python heredoc scripts in workflow

**Description**: Verify `continuous-deploy.yml` no longer contains any Python heredoc scripts.

**Steps**:
1. Open `.github/workflows/continuous-deploy.yml`
2. Search for `python -` and `<<'PY'`
3. Verify the old "Validate CLI dependency versions" step is replaced
4. Verify the old "Validate CLI dependency publishability" step is replaced
5. Verify a single "Validate CLI dependencies" step exists with `cargo xtask validate-cli-deps`

**Expected Result**: Zero Python heredoc scripts found. Single xtask-based validation step present.

**Status**: ✅ PASS

**Actual Result**: Grep for `python -|<<'PY'|python3|heredoc` returned zero matches in `continuous-deploy.yml`. A single "Validate CLI dependencies" step exists at line 109 with `run: cargo xtask validate-cli-deps`.

---

### TC-006: Documentation is accurate

**Description**: Verify updated documentation correctly describes the xtask-based validation.

**Steps**:
1. Check `docs/guides/ci-pipeline.md` — "CLI Dependency Validation" section
2. Verify it describes `cargo xtask validate-cli-deps`
3. Verify it mentions local runnability
4. Verify no references to Python scripts
5. Check `cli/Cargo.toml` — verify NOTE comment references xtask

**Expected Result**: Documentation accurately describes xtask validation, no Python references remain.

**Status**: ✅ PASS

**Actual Result**:
- `docs/guides/ci-pipeline.md` lines 358–369: "CLI Dependency Validation" section accurately describes the xtask command, both checks (version field + publishability), dynamic discovery, and local runnability with both basic and verbose examples.
- No Python references found in the section.
- `cli/Cargo.toml` line 45: `# NOTE: cargo xtask validate-cli-deps validates these wavecraft-* dependencies` — correct.

---

### TC-007: `cargo xtask --help` lists the command

**Description**: Verify the new command appears in xtask help output.

**Steps**:
1. `cd engine && cargo xtask --help`
2. Verify `validate-cli-deps` appears in the command list

**Expected Result**: Help output includes `validate-cli-deps  Validate CLI wavecraft-* dependency versions and publishability`.

**Status**: ✅ PASS

**Actual Result**: Help output includes:
```
validate-cli-deps     Validate CLI wavecraft-* dependency versions and publishability
```

## Issues Found

None.

## QA Re-test Verification

The following test cases verify the specific QA findings were fixed correctly.

### TC-008: QA Finding 1 — No `std::process::exit` in validate_cli_deps.rs (High)

**Description**: Verify `std::process::exit(1)` was replaced with `anyhow::bail!()`.

**Steps**:
1. Grep for `std::process::exit` in `engine/xtask/src/commands/validate_cli_deps.rs`
2. Verify zero occurrences
3. Grep for `anyhow::bail!` to confirm replacements

**Expected Result**: Zero `std::process::exit` calls. `anyhow::bail!` used on both error paths.

**Status**: ✅ PASS

**Actual Result**: `grep std::process::exit` returned 0 matches. Two `anyhow::bail!` calls present: line 66 ("No wavecraft-* dependencies found") and line 112 ("validation error(s) found"). Consistent with all other xtask commands.

---

### TC-009: QA Finding 3 — `print_error_item()` in shared output module (Medium)

**Description**: Verify `print_error_item()` was moved from local function to `xtask::output` module.

**Steps**:
1. Grep for `print_error_item` definition in `engine/xtask/src/lib.rs`
2. Verify no local definition exists in `validate_cli_deps.rs`
3. Verify `validate_cli_deps.rs` imports it via `use xtask::output::*`

**Expected Result**: `print_error_item` defined once in `lib.rs` output module, called from `validate_cli_deps.rs` via import.

**Status**: ✅ PASS

**Actual Result**: `print_error_item` defined at `lib.rs:240` inside `pub mod output`. `validate_cli_deps.rs` imports via `use xtask::output::*` (line 22) and calls at line 88. No local definition exists. Symmetrical with `print_success_item` at `lib.rs:230`.

---

### TC-010: QA Finding 2 — Unit tests for `validate_dependency()` error paths (Medium)

**Description**: Verify new tests cover `validate_dependency()` edge cases.

**Steps**:
1. Check that `test_validate_dependency_missing_crate_dir` exists and passes
2. Check that `test_validate_dependency_malformed_toml` exists and passes
3. Verify they test the actual `validate_dependency()` function (not just TOML parsing)

**Expected Result**: Both tests exist, call `validate_dependency()` directly, and pass.

**Status**: ✅ PASS

**Actual Result**: Both tests exist in `validate_cli_deps.rs` tests module. `test_validate_dependency_missing_crate_dir` (line ~369) uses a fake temp dir to verify "crate Cargo.toml not found" error. `test_validate_dependency_malformed_toml` (line ~383) writes invalid TOML to a tempdir and verifies "failed to parse" error. Both call `validate_dependency()` directly and passed in TC-003.

---

### TC-011: QA Finding 5 — Test for absent `publish` key = publishable (Low)

**Description**: Verify a test exists confirming that a crate TOML without `publish` key is treated as publishable.

**Steps**:
1. Check that `test_publish_key_absent_is_publishable` exists and passes
2. Verify it creates a crate TOML without `publish` field
3. Verify it calls `validate_dependency()` and asserts zero errors

**Expected Result**: Test exists, calls `validate_dependency()`, and asserts no errors for absent `publish` key.

**Status**: ✅ PASS

**Actual Result**: Test exists at ~line 398. Creates a tempdir with a Cargo.toml containing only `[package] name` and `version` (no `publish` key). Calls `validate_dependency()` and asserts `errors.is_empty()`. Passed in TC-003.

## Testing Notes

### Original test pass (2026-02-09)
- The old `fix-cd-stale-python-validation` feature spec does not exist in the repo (confirmed by repo-wide grep). The implementation progress notes this — no action needed.
- All 6 unit tests comprehensively cover: real repo TOML parsing, missing version detection, unpublishable crate detection, empty deps edge case, section-style TOML, and inline-table TOML.
- The command dynamically discovers deps, so adding/removing wavecraft-* deps from `cli/Cargo.toml` requires no workflow changes.

### Re-test pass (2026-02-09, post-QA fixes)
- All 4 actionable QA findings (1 High, 2 Medium, 1 Low) have been verified as fixed.
- Test count increased from 6 → 9 with the 3 new tests covering `validate_dependency()` edge cases and absent `publish` key.
- The `xtask::output` module is now consistent — both `print_success_item` and `print_error_item` live in the shared module.
- Error handling now uses `anyhow::bail!()` consistently, matching all other xtask command modules.
- QA Finding 4 (Low, missing spec docs) was noted as optional by QA and is not a code issue — no re-test needed.

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] QA findings verified as fixed (4/4 actionable findings)
- [x] Issues documented for coder agent: N/A — all issues resolved
- [x] Ready for release: **YES**
