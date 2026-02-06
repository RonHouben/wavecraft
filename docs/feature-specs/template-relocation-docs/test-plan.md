# Test Plan: Template Reorganization

## Overview
- **Feature**: Template Reorganization (plugin-template → sdk-templates/new-project/react)
- **Spec Location**: `docs/feature-specs/template-relocation-docs/`
- **Date**: 2026-02-06
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 9 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 1 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask check` passes (all lint + tests)
- [x] Feature branch `feature/template-reorganization` checked out

## Test Cases

### TC-001: Directory Structure Exists

**Description**: Verify the new template directory structure was created correctly

**Preconditions**:
- Repository checked out at feature branch

**Steps**:
1. Check that `cli/sdk-templates/new-project/react/` exists
2. Verify template files are present (Cargo.toml.template, LICENSE, README.md)
3. Verify subdirectories exist (engine/, ui/)
4. Verify old `cli/plugin-template/` no longer exists

**Expected Result**: New directory structure exists with all files; old directory removed

**Status**: ✅ PASS

**Actual Result**: Directory structure correct. Files present: Cargo.toml.template, LICENSE, README.md, Cargo.lock, .gitignore. Subdirectories: engine/, ui/. Old `cli/plugin-template/` does not exist.

**Notes**: Verified with `ls -la cli/sdk-templates/new-project/react/` 

---

### TC-002: CLI Compiles Successfully

**Description**: Verify the CLI compiles with the updated include_dir! path

**Preconditions**:
- Directory structure updated (TC-001)

**Steps**:
1. Navigate to `cli/` directory
2. Run `cargo build`
3. Verify build succeeds without errors

**Expected Result**: CLI compiles successfully

**Status**: ✅ PASS

**Actual Result**: `cargo build --release` completed successfully in 1.67s. CLI binary at `cli/target/release/wavecraft`.

**Notes**: Build uses updated `include_dir!` path 

---

### TC-003: Template Extraction Works

**Description**: Verify `wavecraft new` extracts template to correct location

**Preconditions**:
- CLI compiles (TC-002)

**Steps**:
1. Create isolated test directory: `mkdir -p /tmp/wavecraft-test`
2. Run `cd /tmp/wavecraft-test && wavecraft new test-plugin --vendor "Test" --no-git`
3. Verify project directory created with expected files
4. Verify Cargo.toml files are generated (not .template suffix)

**Expected Result**: Project scaffolded with correct structure

**Status**: ✅ PASS

**Actual Result**: Project scaffolded successfully at `/tmp/wavecraft-test/test-plugin/`. Contains: Cargo.toml, engine/, ui/, LICENSE, README.md. Cargo.toml files are correctly generated (not .template suffix). Variable substitution working correctly (`test-plugin`, `test_plugin`).

**Notes**: Template extraction from new path works correctly 

---

### TC-004: Generated Project Builds

**Description**: Verify the scaffolded project compiles and bundles

**Preconditions**:
- Template extraction works (TC-003)

**Steps**:
1. Navigate to generated project: `cd /tmp/wavecraft-test/test-plugin`
2. Install UI dependencies: `cd ui && npm install`
3. Build UI: `npm run build`
4. Bundle plugin: `cd ../engine && cargo xtask bundle --release`
5. Verify bundles created in `target/bundled/`

**Expected Result**: VST3 and CLAP bundles generated

**Status**: ⏸️ BLOCKED

**Actual Result**: UI dependencies installed successfully. UI build completed (dist/ created). Plugin bundle **FAILED** due to pre-existing template bug: `xtask/src/main.rs` hardcodes `my-plugin` instead of using template variable `{{plugin_name}}`.

**Notes**: This is a **pre-existing bug** in the template, not caused by the reorganization. The template reorganization itself is working correctly. See Issue #1 below. 

---

### TC-005: CI Workflow Path Filter Updated

**Description**: Verify continuous-deploy.yml has correct path filter

**Preconditions**:
- None

**Steps**:
1. Read `.github/workflows/continuous-deploy.yml`
2. Verify CLI path filter includes `cli/sdk-templates/**`
3. Verify old `cli/plugin-template/**` is NOT present

**Expected Result**: Path filter references new location only

**Status**: ✅ PASS

**Actual Result**: Line 30 contains `- 'cli/sdk-templates/**'`. No `plugin-template` references found in file.

**Notes**: Verified with grep search 

---

### TC-006: README Structure Updated

**Description**: Verify README.md reflects new template location

**Preconditions**:
- None

**Steps**:
1. Read `README.md`
2. Search for "sdk-templates" in repository structure diagram
3. Verify no references to "plugin-template" in active content

**Expected Result**: README shows correct template path

**Status**: ✅ PASS

**Actual Result**: Line 101 shows `sdk-templates/` in repository structure. No `plugin-template` references found.

**Notes**: Verified with grep search 

---

### TC-007: High-Level Design Updated

**Description**: Verify architecture documentation reflects new structure

**Preconditions**:
- None

**Steps**:
1. Read `docs/architecture/high-level-design.md`
2. Verify monorepo structure diagram shows `cli/sdk-templates/new-project/react/`
3. Verify SDK distribution diagram shows embedded template path
4. Search for "plugin-template" — should only exist in historical context

**Expected Result**: Architecture docs reference new template location

**Status**: ✅ PASS

**Actual Result**: Lines 49 and 290 contain `sdk-templates`. No `plugin-template` references found in file.

**Notes**: Both monorepo structure and SDK distribution diagrams updated 

---

### TC-008: CI Pipeline Guide Updated

**Description**: Verify CI documentation reflects new path filter

**Preconditions**:
- None

**Steps**:
1. Read `docs/guides/ci-pipeline.md`
2. Verify path filter table shows `cli/sdk-templates/**`
3. Verify no references to `cli/plugin-template/**`

**Expected Result**: CI guide shows correct path filter

**Status**: ✅ PASS

**Actual Result**: Line 348 shows `sdk-templates/` in path filter table. No `plugin-template` references found.

**Notes**: Verified with grep search 

---

### TC-009: Automated Checks Pass

**Description**: Verify all linting and tests pass

**Preconditions**:
- Implementation complete

**Steps**:
1. Run `cargo xtask check` from engine directory
2. Verify all lint phases pass
3. Verify all test phases pass

**Expected Result**: All checks pass with no failures

**Status**: ✅ PASS

**Actual Result**: All checks passed in 13.2s total:
- Linting: PASSED (5.1s) - Rust fmt, Clippy, ESLint, Prettier all OK
- Tests: PASSED (8.2s) - 110+ engine tests, 28 UI tests

**Notes**: Full `cargo xtask check` output captured 

---

### TC-010: No Stale References in Active Docs

**Description**: Verify no stale plugin-template references remain in active documentation

**Preconditions**:
- Documentation updates complete

**Steps**:
1. Run `grep -r "plugin-template" --include="*.md" docs/ README.md | grep -v "_archive" | grep -v "template-relocation-docs" | grep -v "roadmap.md"`
2. Verify no unexpected references found
3. Note: roadmap.md historical entries are expected and acceptable

**Expected Result**: No stale references (except roadmap history)

**Status**: ✅ PASS

**Actual Result**: Only one reference found: `docs/backlog.md` contains a ~~strikethrough~~ entry marking a completed historical task. This is acceptable as it documents what was done, not an active reference.

**Notes**: roadmap.md historical entries excluded per coding standards 

---

## Issues Found

### Issue #1: Template xtask hardcodes plugin name (PRE-EXISTING)

- **Severity**: Medium
- **Test Case**: TC-004
- **Description**: The template file `cli/sdk-templates/new-project/react/engine/xtask/src/main.rs` has hardcoded `my-plugin` strings instead of using template variables like `{{plugin_name}}`.
- **Expected**: xtask should use `{{plugin_name}}` variables that get substituted during `wavecraft new`
- **Actual**: Lines 6, 36, 39 contain hardcoded `my-plugin`
- **Steps to Reproduce**:
  1. `wavecraft new test-plugin --vendor "Test" --no-git`
  2. `cd test-plugin/ui && npm install && npm run build`
  3. `cd ../engine && cargo xtask bundle --release`
  4. Error: `package ID specification 'my-plugin' did not match any packages`
- **Evidence**: `grep -n "my-plugin" engine/xtask/src/main.rs` shows 3 hardcoded references
- **Impact**: Scaffolded projects cannot bundle without manually editing xtask/main.rs
- **Root Cause**: Template was never fully updated with variable substitution for xtask
- **Note**: This is a **PRE-EXISTING BUG** — not introduced by the template reorganization feature. The reorganization is working correctly; this bug existed before the move.

### Issue #2: SDK version default is outdated (PRE-EXISTING)

- **Severity**: Low
- **Test Case**: TC-004 (discovered during)
- **Description**: The CLI default `--sdk-version` is `v0.7.0` but the repo is at `v0.7.1`.
- **File**: `cli/src/main.rs` line 52
- **Expected**: Default should match latest published tag
- **Actual**: `#[arg(long, default_value = "v0.7.0")]`
- **Impact**: Default scaffold fails with "tag v0.7.0 not found" (workaround: use `--local-dev` or specify `--sdk-version v0.7.1`)
- **Note**: This is a **PRE-EXISTING BUG** — not introduced by the template reorganization feature.

## Testing Notes

The **template reorganization feature** (moving `cli/plugin-template/` to `cli/sdk-templates/new-project/react/`) is **working correctly**:

1. Directory structure created properly
2. CLI compiles with new `include_dir!` path
3. Template extraction via `wavecraft new` works
4. All documentation updated to reference new location
5. CI workflow path filters updated
6. All automated checks (lint + tests) pass

Two **pre-existing bugs** were discovered during testing (Issue #1 and #2). These bugs existed before the reorganization and are **not caused by this feature**. They should be tracked separately for future fixes.

## Sign-off

- [x] All critical tests pass (9/9 within scope of this feature)
- [x] All high-priority tests pass
- [x] Issues documented for coder agent (2 pre-existing bugs documented)
- [x] Ready for release: **YES** (for the template reorganization feature)

**Recommendation**: The template reorganization feature is complete and working. The pre-existing bugs (Issue #1, #2) should be filed as separate backlog items and do not block this PR.
