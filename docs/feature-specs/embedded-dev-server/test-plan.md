# Test Plan: Embedded Dev Server

## Overview
- **Feature**: Embedded WebSocket dev server in `wavecraft start`
- **Spec Location**: `docs/feature-specs/embedded-dev-server/`
- **Date**: 2026-02-06
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 3 |
| ❌ FAIL | 1 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask check` passes (engine lint + tests)
- [x] CLI tests pass (`cargo test` in `cli/`)

## Test Cases

### TC-001: Engine validation via xtask

**Description**: Verify engine linting and tests pass after SDK changes.

**Preconditions**:
- Rust toolchain installed
- Workspace dependencies available

**Steps**:
1. Run `cargo xtask check` from `engine/` directory

**Expected Result**: Command completes successfully with no errors.

**Status**: ✅ PASS

**Actual Result**: `cargo xtask check` completed successfully (lint + tests passed).

**Notes**:

---

### TC-002: CLI unit tests

**Description**: Verify CLI unit tests pass with new dev_server module.

**Preconditions**:
- Rust toolchain installed

**Steps**:
1. Run `cargo test` from `cli/` directory

**Expected Result**: All tests pass.

**Status**: ✅ PASS

**Actual Result**: `cargo test` completed successfully (28 tests passed).

**Notes**:

---

### TC-003: `wavecraft start` with auto-install

**Description**: Verify `wavecraft start` builds plugin, loads parameters via FFI, starts embedded WebSocket server, and starts Vite UI server.

**Preconditions**:
- CLI built/runnable
- Test project created via `wavecraft create`

**Steps**:
1. Create test project in a temp directory
2. Run `wavecraft start --install --port 9010 --ui-port 5174 --verbose`
3. Observe logs for:
   - Plugin build success
   - Parameter loading count
   - WebSocket server running message
   - UI dev server running message

**Expected Result**: Both servers start successfully; parameters are loaded and reported in logs.

**Status**: ❌ FAIL

**Actual Result**: Start failed while loading parameters. The CLI located `libwavecraft_nih_plug.dylib` and the FFI symbol `wavecraft_get_params_json` was missing.

**Notes**:
- Found dylib: `/Users/ronhouben/code/private/wavecraft/engine/target/debug/libwavecraft_nih_plug.dylib`
- Error: `Symbol not found: wavecraft_get_params_json`

---

### TC-004: `wavecraft start` with `--no-install` and missing deps

**Description**: Verify `wavecraft start --no-install` fails gracefully when `node_modules` is missing.

**Preconditions**:
- Test project created with no `node_modules` in `ui/`

**Steps**:
1. Run `wavecraft start --no-install` in test project

**Expected Result**: Command exits with a clear error instructing user to install dependencies.

**Status**: ✅ PASS

**Actual Result**: Command exited with message: "Dependencies not installed. Run `npm install` in the ui/ directory, or use `wavecraft start --install` to install automatically."

**Notes**:

---

## Issues Found

### Issue #1: Wrong dylib selected for FFI parameter export

- **Severity**: High
- **Test Case**: TC-003
- **Description**: `wavecraft start --install` loads the first matching `lib*.dylib` in `engine/target/debug` and selects `libwavecraft_nih_plug.dylib` instead of the plugin dylib. This causes FFI symbol lookup to fail.
- **Expected**: CLI should load the plugin dylib that exports `wavecraft_get_params_json`.
- **Actual**: CLI loaded `libwavecraft_nih_plug.dylib`; `dlsym` failed for `wavecraft_get_params_json`.
- **Steps to Reproduce**:
   1. Create project via `wavecraft create`
   2. Run `wavecraft start --install --verbose`
   3. Observe error during parameter loading
- **Evidence**:
   - Found dylib: `/Users/ronhouben/code/private/wavecraft/engine/target/debug/libwavecraft_nih_plug.dylib`
   - Error: `Symbol not found: wavecraft_get_params_json`
- **Suggested Fix**: Update `find_plugin_dylib()` to target the plugin crate output (e.g., by reading `engine/Cargo.toml` crate name or filtering out `libwavecraft_nih_plug`).

## Testing Notes

- Manual UI verification (browser interactions) was not performed due to early failure in parameter loading.
- Dev servers did not start successfully because FFI symbol lookup failed.

## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [x] Issues documented for coder agent
- [ ] Ready for release: NO
