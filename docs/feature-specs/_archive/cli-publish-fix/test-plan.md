# Test Plan: CLI Publish Fix

## Overview
- **Feature**: Fix CLI crates.io publish failure
- **Spec Location**: `docs/feature-specs/cli-publish-fix/`
- **Date**: 2026-02-06
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 2 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Problem Description

The CLI failed to publish to crates.io during GitHub Actions with the error:

```
error: proc macro panicked
  --> src/template/mod.rs:11:28
   |
11 | static TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../plugin-template");
   |                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = help: message: "/home/runner/work/wavecraft/wavecraft/cli/target/package/wavecraft-0.7.1/../plugin-template" is not a directory
```

### Root Cause

1. The `plugin-template/` was located **outside** the `cli/` crate directory
2. `cargo publish` only packages files **within** the crate directory
3. Additionally, cargo excludes directories containing `Cargo.toml` files (treats them as separate crates)

### Solution Implemented

1. **Moved template into CLI crate**: `plugin-template/` → `cli/plugin-template/`
2. **Renamed Cargo.toml files**: `Cargo.toml` → `Cargo.toml.template` to prevent cargo from excluding them
3. **Updated extraction code**: Template extraction now renames `.template` files back to their original names

## Test Cases

### TC-001: Verify cargo publish --dry-run succeeds

**Description**: Confirm the CLI can be packaged and verified

**Steps**:
1. Run `cd cli && cargo publish --dry-run`
2. Verify compilation succeeds
3. Verify "aborting upload due to dry run" message appears

**Expected Result**: Package verification passes

**Status**: ✅ PASS

**Actual Result**: 
```
Packaged 33 files, 72.2KiB (22.4KiB compressed)
Verifying wavecraft v0.7.1
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.40s
Uploading wavecraft v0.7.1
warning: aborting upload due to dry run
```

---

### TC-002: Verify template extraction renames .template files

**Description**: Confirm generated projects have proper Cargo.toml files (not .template)

**Steps**:
1. Run `wavecraft new test-plugin --vendor "Test" --no-git`
2. Check that `test-plugin/Cargo.toml` exists (not `.template`)
3. Check that `test-plugin/engine/Cargo.toml` exists
4. Verify the content is valid TOML

**Expected Result**: All Cargo.toml files exist with correct names and valid content

**Status**: ✅ PASS

**Actual Result**: 
- `test-plugin/Cargo.toml` ✓
- `test-plugin/engine/Cargo.toml` ✓
- `test-plugin/engine/xtask/Cargo.toml` ✓
- All files contain valid TOML with correct plugin name substitution

---

## Files Changed

| File | Change |
|------|--------|
| `cli/plugin-template/` | Moved from repo root `plugin-template/` |
| `cli/plugin-template/Cargo.toml` | Renamed to `Cargo.toml.template` |
| `cli/plugin-template/engine/Cargo.toml` | Renamed to `Cargo.toml.template` |
| `cli/plugin-template/engine/xtask/Cargo.toml` | Renamed to `Cargo.toml.template` |
| `cli/src/template/mod.rs` | Updated to strip `.template` suffix during extraction |
| `cli/Cargo.toml` | Removed unneeded `include` directive |
| `.github/workflows/continuous-deploy.yml` | Updated paths-filter, removed copy step |
| `.github/workflows/cli-release.yml` | Removed copy step |
| `.gitignore` | Removed `/cli/plugin-template` and `/plugin-template/target` |

## Sign-off

- [x] All test cases pass
- [x] No blockers remain
- [x] Ready for release: **YES**
