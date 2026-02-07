# Test Plan: crates.io Publishing Fix

## Overview
- **Feature**: crates.io publishing fix for CLI + dev-server
- **Spec Location**: `docs/feature-specs/crates-io-publishing-fix/`
- **Date**: 2026-02-07
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 2 |
| ❌ FAIL | 2 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [ ] `cargo xtask ci-check` passes (all lint + tests)
- [x] crates.io Trusted Publishing configured for workflow

## Test Cases

### TC-001: Local CI dry-run

**Description**: Run the local CI check via xtask.

**Preconditions**:
- Rust toolchain installed

**Steps**:
1. Run `cargo run --manifest-path engine/xtask/Cargo.toml -- ci-check`.

**Expected Result**: Command exits successfully with code 0.

**Status**: ❌ FAIL

**Actual Result**: Lint phase fails due to rustfmt diff in `engine/crates/wavecraft-dev-server/src/assets.rs` (formatting in `ui_dist_dir` assignment).

**Notes**: Tests completed; lint failure blocks overall ci-check.

---

### TC-002: CLI publish dry-run

**Description**: Validate CLI publishability (versions present, no missing deps).

**Preconditions**:
- CLI manifest updated with versioned path deps

**Steps**:
1. Run `cargo publish --manifest-path cli/Cargo.toml --dry-run --allow-dirty`.

**Expected Result**: Dry-run completes successfully.

**Status**: ✅ PASS

**Actual Result**: Dry-run succeeded; cargo packaged `wavecraft v0.8.0` and exited successfully.

**Notes**: Ran with `--allow-dirty`.

---

### TC-003: Engine publish dry-run (workspace)

**Description**: Verify engine crates are publishable via cargo-workspaces.

**Preconditions**:
- `cargo-workspaces` available via xtask in CI

**Steps**:
1. Run `cargo run --manifest-path engine/xtask/Cargo.toml -- ci-check --skip-tests` (lint-only).
2. Run `cargo ws publish --from-git --dry-run --yes --allow-branch main` from `engine/`.

**Expected Result**: Dry-run completes successfully.

**Status**: ✅ PASS

**Actual Result**: `cargo ws publish --from-git --dry-run --yes --allow-branch main` completed successfully.

**Notes**: Warning: `http.cainfo` not set; no failure.

---

### TC-004: CI workflow validation (OIDC)

**Description**: Trigger Continuous Deploy on the fix branch and verify publish jobs run with OIDC.

**Preconditions**:
- Trusted Publisher configured for the workflow filename

**Steps**:
1. Trigger `continuous-deploy.yml` on `fix/publish-packages-workflow`.
2. Verify `publish-cli` and `publish-engine` jobs succeed and use OIDC.

**Expected Result**: Jobs succeed without `CARGO_REGISTRY_TOKEN`.

**Status**: ❌ FAIL

**Actual Result**: Workflow run 21778718367 failed in `publish-cli` at “Verify publishability (dry-run)”.

**Notes**: Failure due to missing system libraries on Linux runner (`gobject-2.0`, `glib-2.0`, `gio-2.0`, `gdk-3.0`).

---

## Issues Found

### Issue #1: `cargo xtask ci-check` fails rustfmt

- **Severity**: High
- **Test Case**: TC-001
- **Description**: Local ci-check fails during lint phase due to rustfmt diff in `engine/crates/wavecraft-dev-server/src/assets.rs`.
- **Expected**: Lint phase completes successfully.
- **Actual**: Lint fails with rustfmt diff for `ui_dist_dir` line formatting.
- **Steps to Reproduce**:
	1. Run `cargo run --manifest-path engine/xtask/Cargo.toml -- ci-check`.
- **Evidence**: Lint output indicates formatting diff in `assets.rs`.
- **Suggested Fix**: Run rustfmt and commit formatting changes.

### Issue #2: CI publish-cli dry-run fails on Linux runner

- **Severity**: High
- **Test Case**: TC-004
- **Description**: `publish-cli` job fails during `cargo publish --dry-run` because GTK/GLib system libraries are missing on the Linux runner.
- **Expected**: CLI dry-run completes successfully in CI.
- **Actual**: Build fails in `gobject-sys`, `glib-sys`, `gio-sys`, and `gdk-sys` with missing `*.pc` files and `PKG_CONFIG_PATH` unset.
- **Steps to Reproduce**:
	1. Trigger `continuous-deploy.yml` on `fix/publish-packages-workflow`.
	2. Observe `publish-cli` failure in run 21778718367.
- **Evidence**: Job log shows missing `gobject-2.0.pc`, `glib-2.0.pc`, `gio-2.0.pc`, `gdk-3.0.pc` and `pkg-config exited with status code 1`.
- **Suggested Fix**: Install required GTK/GLib dev packages on the runner or gate dry-run build to avoid GTK-dependent paths.

## Testing Notes


## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Issues documented for coder agent
- [ ] Ready for release: YES / NO
