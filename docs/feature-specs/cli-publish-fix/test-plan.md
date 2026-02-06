# Test Plan: CLI Publish Fix

## Overview
- **Feature**: Fix CLI crates.io publish failure
- **Spec Location**: `docs/feature-specs/cli-publish-fix/`
- **Date**: 2026-02-06
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 0 |
| ❌ FAIL | 1 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Problem Description

The CLI fails to publish to crates.io during GitHub Actions with the error:

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

1. The CLI uses `include_dir!("$CARGO_MANIFEST_DIR/../plugin-template")` to embed the template
2. The `plugin-template/` directory is **outside** the `cli/` crate directory
3. `cargo publish` only packages files **within** the crate directory
4. During verification, cargo extracts the tarball and builds from it
5. The `../plugin-template` path doesn't exist relative to the extracted tarball

## Test Cases

### TC-001: Verify CI publish failure

**Description**: Confirm the publish-cli job fails in CI

**Steps**:
1. Check GitHub Actions for the continuous-deploy workflow
2. Observe publish-cli job failure

**Expected Result**: Job fails with the "not a directory" error

**Status**: ❌ FAIL (as expected - this is the bug being reported)

**Actual Result**: 
```
error: proc macro panicked
= help: message: "/home/runner/work/wavecraft/wavecraft/cli/target/package/wavecraft-0.7.1/../plugin-template" is not a directory
```

---

## Issues Found

### Issue #1: CLI cannot publish to crates.io due to external template path

- **Severity**: Critical
- **Test Case**: TC-001
- **Description**: The `include_dir!` macro references `../plugin-template` which is outside the CLI crate. `cargo publish` only packages files within the crate directory, so the template is not included in the tarball.
- **Expected**: CLI publishes successfully to crates.io
- **Actual**: Build fails during `cargo publish` verification
- **Steps to Reproduce**:
  1. Run `cd cli && cargo publish --dry-run`
  2. Observe the error about missing directory
- **Evidence**: GitHub Actions error from publish-cli job
- **Suggested Fix**: 
  1. Update CI workflow to copy `plugin-template/` into `cli/plugin-template/` before publishing
  2. Update `include_dir!` path from `"$CARGO_MANIFEST_DIR/../plugin-template"` to `"$CARGO_MANIFEST_DIR/plugin-template"`
  3. Add `cli/plugin-template/` to `.gitignore` (it's a build artifact, not source of truth)
  4. For local development, either:
     - Use a build script to copy before build, OR
     - Update contributing docs to require the copy step

## Files Involved

| File | Purpose |
|------|---------|
| `cli/src/template/mod.rs:11` | Contains the `include_dir!` macro call |
| `cli/Cargo.toml` | CLI package configuration |
| `.github/workflows/continuous-deploy.yml` | CI publish workflow |
| `plugin-template/` | The template directory to embed |

## Testing Notes

The fix needs to ensure:
1. ✅ Local development still works (`cargo build` in `cli/`)
2. ✅ `cargo publish --dry-run` succeeds
3. ✅ CI publish-cli job succeeds
4. ✅ Published crate includes the template correctly

## Sign-off

- [x] Critical issue documented
- [x] Issues documented for coder agent
- [ ] Ready for release: **NO** (blocked by Issue #1)

---

## Handoff to Coder

**Recommended Action**: Hand off to coder agent to implement the fix.

The coder should:
1. Update CI workflow to copy `plugin-template/` into `cli/plugin-template/` before publishing
2. Update `include_dir!` path in `cli/src/template/mod.rs`
3. Add `cli/plugin-template/` to `.gitignore`
4. Optionally add a local dev convenience (build.rs or script)
5. Test with `cargo publish --dry-run`
