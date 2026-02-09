# Test Plan: Build-Time Parameter Discovery

## Overview

- **Feature**: Build-Time Parameter Discovery
- **Branch**: `feat/build-time-param-discovery`
- **Date**: 2026-02-09
- **Tester**: Tester Agent

## Purpose

This feature prevents `wavecraft start` from hanging at "Loading plugin parameters..." on macOS by feature-gating nih-plug's VST3/CLAP exports. When `_param-discovery` is active, the dylib contains only parameter FFI functions — no plugin factory registrations — preventing blocking audio subsystem initialization.

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 5 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] Working on branch `feat/build-time-param-discovery`
- [x] Cargo and Rust toolchain available
- [x] macOS environment (for symbol verification)
- [x] `nm` command available for symbol inspection
- [x] All dependencies installed

## Test Cases

### TC-001: Automated CI Checks (Linting + Tests)

**Description**: Verify that all automated linting and test suites pass without errors.

**Steps**:
1. Run `cargo xtask ci-check`
2. Wait for completion
3. Inspect output for any failures

**Expected Result**: All linting and tests pass

**Status**: ✅ PASS

**Actual Result**: 
```
✓ Linting: PASSED (6.2s)
✓ Automated Tests: PASSED (15.1s)
  Engine tests: 87 passed
  UI tests: 28 passed
Total time: 21.3s
```

---

### TC-002: Template Generation

**Description**: Verify that `wavecraft create` generates a valid plugin project with the `_param-discovery` feature defined.

**Status**: ✅ PASS

**Actual Result**: Generated Cargo.toml includes the feature flag correctly.

---

### TC-003: Generated Code Quality (Clippy)

**Description**: Verify that the generated plugin template code passes clippy without warnings.

**Status**: ✅ PASS

**Actual Result**: Compilation completed successfully with zero clippy diagnostics

---

### TC-004: Feature Gate — nih-plug Symbols Excluded with `_param-discovery`

**Description**: Verify that building with `_param-discovery` feature EXCLUDES nih-plug plugin factory symbols while PRESERVING param FFI functions.

**Status**: ✅ PASS

**Actual Result**: 
```
# Symbol check output:
- clap_entry: NOT found ✓
- GetPluginFactory: NOT found ✓
- wavecraft_get_params_json: FOUND ✓
- wavecraft_free_string: FOUND ✓
```

---

### TC-005: Normal Build — nih-plug Symbols Included Without Feature

**Description**: Verify that building WITHOUT the `_param-discovery` feature INCLUDES nih-plug plugin factory symbols.

**Status**: ✅ PASS

**Actual Result**: All expected symbols present (nih-plug + param FFI functions)

---

## Issues Found

**None** — All test cases passed successfully.

## Sign-off

- [x] `cargo xtask ci-check` passes (all lint + tests)
- [x] Template validation passes (clippy, feature present)
- [x] Feature gate verified (symbols excluded/included correctly)
- [x] Backward compatibility confirmed
- [x] No critical or high-priority issues found
- [x] Ready for release: **YES**

## Recommendations

1. ✅ **Approve for merge** — All testing passed, feature complete
2. Update `docs/roadmap.md` to mark feature as complete
3. Archive feature spec to `docs/feature-specs/_archive/build-time-param-discovery/`

---

**Test completed by**: Tester Agent  
**Tested on**: macOS (Apple Silicon)
