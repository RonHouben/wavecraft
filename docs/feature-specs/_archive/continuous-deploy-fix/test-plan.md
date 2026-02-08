# Test Plan: Continuous Deploy Fix

## Overview
- **Feature**: Continuous Deploy engine publish dependency fix
- **Spec Location**: `docs/feature-specs/continuous-deploy-fix/`
- **Date**: 2026-02-08
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 2 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask ci-check` passes (all lint + tests)
- [x] GitHub Actions workflow updated in `/.github/workflows/continuous-deploy.yml`

## Test Cases

### TC-001: Workflow dependency installation present

**Description**: Verify the `publish-engine` job installs GTK/GLib system dependencies required by `wavecraft-dev-server`.

**Preconditions**:
- Repository contains updated workflow file.

**Steps**:
1. Open `/.github/workflows/continuous-deploy.yml`.
2. Locate the `publish-engine` job.
3. Confirm a step installs `pkg-config`, `libglib2.0-dev`, `libgtk-3-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev`, and `libwebkit2gtk-4.1-dev`.

**Expected Result**: The dependency install step exists and matches required packages.

**Status**: ✅ PASS

**Actual Result**: `publish-engine` includes the Linux dependency install step with GTK/GLib packages.

**Notes**: Verified in `/.github/workflows/continuous-deploy.yml`.

---

### TC-002: Local validation (ci-check)

**Description**: Run the standard CI check to ensure no regressions from workflow changes.

**Preconditions**:
- Rust toolchain installed.

**Steps**:
1. Run `cargo xtask ci-check` from repo root.

**Expected Result**: Command completes successfully.

**Status**: ✅ PASS

**Actual Result**: `cargo run -p xtask -- ci-check` succeeded from `engine/`. All linting and tests passed.

**Notes**: Running `cargo xtask ci-check` at repo root failed because there is no root `Cargo.toml`. Used the engine workspace as intended.

---

## Issues Found

_None._

## Testing Notes

- This test plan validates the workflow change locally and confirms the dependency installation step exists.

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] Issues documented for coder agent
- [x] Ready for release: YES
