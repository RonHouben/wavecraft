# Test Plan: M5 Dead Code Cleanup

## Overview
- **Feature**: Dead Code Cleanup - Remove stale `#[allow(dead_code)]` suppressions
- **Spec Location**: `docs/feature-specs/m5-dead-code-cleanup/`
- **Date**: 1 February 2026
- **Tester**: Tester Agent
- **Version**: 0.2.2

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 0 |
| ❌ FAIL | 1 |
| ⏸️ BLOCKED | 7 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] Changes committed to `feature/m5-dead-code-cleanup` branch
- [ ] Docker is running: `docker info`
- [ ] CI image exists: `docker images | grep vstkit-ci`
- [ ] Local CI passes (see Phase 2)

## Test Objectives

This feature reduces technical debt by removing 11 stale `#[allow(dead_code)]` suppressions that became obsolete when the React UI feature flag was removed. The cleanup should:
1. Result in clean compilation (0 clippy warnings)
2. Not break any existing functionality
3. Pass all automated tests
4. Maintain plugin stability in DAW environment

**Success Criteria**: 
- 14 → 3 suppressions (79% reduction achieved in implementation)
- All tests pass
- Plugin loads and functions correctly in Ableton Live

## Test Cases

### TC-001: Local CI Pipeline Execution

**Description**: Verify all CI checks pass locally using Docker

**Preconditions**:
- Docker Desktop is running
- CI image `vstkit-ci:latest` exists

**Steps**:
1. Run `docker info` to verify Docker is available
2. Execute full CI pipeline: `act -W .github/workflows/ci.yml --container-architecture linux/amd64 -P ubuntu-latest=vstkit-ci:latest --pull=false --artifact-server-path /tmp/act-artifacts`
3. Monitor output for job status

**Expected Result**: All jobs pass (check-ui, test-ui, prepare-engine, check-engine, test-engine)

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-002: Code Formatting Check

**Description**: Verify Rust code formatting is consistent

**Preconditions**:
- Changes committed on feature branch

**Steps**:
1. Run `cd engine && cargo fmt --check`

**Expected Result**: Exit code 0, no formatting issues

**Status**: ⏸️ BLOCKED

**Actual Result**: Not run - blocked by TC-001 failure

**Notes**: Blocked until platform-specific dead code issue is resolved

---

### TC-003: Clippy Linting (Zero Warnings)

**Description**: Verify no clippy warnings after suppression removal

**Preconditions**:
- All code changes applied

**Steps**:
1. Run `cd engine && cargo clippy --workspace --all-targets -- -D warnings`

**Expected Result**: 
- Exit code 0
- Output shows "0 warnings"
- No dead code warnings for any removed suppressions

**Status**: ❌ FAIL

**Actual Result**: CI pipeline failed at Clippy step with 8 dead code errors:

```
error: static `UI_ASSETS` is never used
  --> crates/plugin/src/editor/assets.rs:13:8

error: function `get_asset` is never used
  --> crates/plugin/src/editor/assets.rs:23:8

error: function `mime_type_from_path` is never used
  --> crates/plugin/src/editor/assets.rs:40:4

error: struct `PluginEditorBridge` is never constructed
  --> crates/plugin/src/editor/bridge.rs:19:12

error: associated function `new` is never used
  --> crates/plugin/src/editor/bridge.rs:30:12

error: multiple fields are never read
  --> crates/plugin/src/editor/webview.rs:42:9
   |
41 | pub struct WebViewConfig {
42 |     pub params: Arc<VstKitParams>,
43 |     pub context: Arc<dyn GuiContext>,
44 |     pub parent: ParentWindowHandle,
45 |     pub width: u32,
46 |     pub height: u32,
48 |     pub meter_consumer: Arc<Mutex<MeterConsumer>>,
50 |     pub editor_size: Arc<Mutex<(u32, u32)>>,

error: function `create_ipc_handler` is never used
  --> crates/plugin/src/editor/webview.rs:78:8

error: constant `IPC_PRIMITIVES_JS` is never used
  --> crates/plugin/src/editor/webview.rs:92:11
```

**Root Cause**: The code in `assets.rs`, `bridge.rs`, and `webview.rs` is only used in platform-specific modules (`macos.rs`, `windows.rs`). On Linux (CI environment), these platform-specific modules don't compile, so the shared code appears dead to Clippy.

**Notes**: This is a critical architectural issue - the low-level design failed to account for platform-specific compilation. The cleanup removed legitimate suppressions that hide cross-platform false positives. All remaining tests blocked until fix.

---

### TC-004: Engine Unit Tests

**Description**: Verify all Rust unit tests pass

**Preconditions**:
- Engine code compiled successfully

**Steps**:
1. Run `cd engine && cargo test --workspace`

**Expected Result**: 
- All tests pass (expected ~49 tests)
- 0 failures
- May have 2 ignored tests (ui/dist related)

**Status**: ⏸️ BLOCKED

**Actual Result**: Not run - blocked by TC-003 Clippy failure

**Notes**: Cannot run tests until code compiles without errors 

---

### TC-005: UI Unit Tests

**Description**: Verify React component tests pass

**Preconditions**:
- UI dependencies installed

**Steps**:
1. Run `cd ui && npm test`

**Expected Result**: 
- All tests pass (expected ~35 tests)
- 0 failures

**Status**: ✅ PASS (from CI)

**Actual Result**: 35 tests passed in 196ms

**Notes**: UI tests passed in CI pipeline before Clippy failure 

---

### TC-006: TypeScript Type Checking

**Description**: Verify no TypeScript errors

**Preconditions**:
- UI code exists

**Steps**:
1. Run `cd ui && npm run typecheck`

**Expected Result**: Exit code 0, no type errors

**Status**: ⏸️ BLOCKED

**Actual Result**: Not run locally - blocked by higher priority Clippy fix

**Notes**: Should verify after fix applied 

---

### TC-007: Plugin Build & Sign (macOS)

**Description**: Verify plugin bundles build and sign correctly

**Preconditions**:
- macOS environment
- Engine compiles cleanly

**Steps**:
1. Run `cd engine && cargo xtask bundle --release`
2. Run `cargo xtask sign --adhoc`
3. Run `cargo xtask sign --verify`
4. Run `cargo xtask install`

**Expected Result**: 
- VST3, CLAP, and AU bundles created in `engine/target/bundled/`
- Ad-hoc signing succeeds
- Verification passes
- Plugins installed to system directories

**Status**: ⏸️ BLOCKED

**Actual Result**: Not run - blocked by Clippy failures

**Notes**: This is macOS-specific and cannot run in Docker. Must fix CI issues first.

---

### TC-008: Plugin Functional Test (Ableton Live)

**Description**: Verify plugin loads and functions correctly in DAW

**Preconditions**:
- Plugins installed via `cargo xtask install`
- Ableton Live installed

**Steps**:
1. Open Ableton Live
2. Rescan plugins if necessary
3. Load VstKit plugin on an audio track
4. Verify UI renders without errors
5. Adjust Volume, Pan, and Gain parameters
6. Play audio and verify meters display correctly
7. Enable automation and verify parameter changes respond
8. Close and reopen UI window

**Expected Result**: 
- Plugin appears in Ableton's plugin list
- UI loads without blank screen or errors
- All parameters respond to changes
- Meters display audio levels correctly
- No crashes or freezes
- UI reopens successfully

**Status**: ⏸️ BLOCKED

**Actual Result**: Not run - blocked by compilation failures

**Notes**: This is the critical manual test to verify no functional regressions from code cleanup. Cannot proceed until Clippy passes.

---

## Issues Found

### Issue #1: Platform-Specific Dead Code False Positives (CRITICAL)

- **Severity**: Critical
- **Test Case**: TC-001, TC-003
- **Description**: The implementation removed `#[allow(dead_code)]` suppressions from code that is only used in platform-specific modules (`macos.rs`, `windows.rs`). On Linux (CI environment), this code appears dead because the platform-specific modules don't compile.
- **Expected**: All 8 removed suppressions in `assets.rs`, `bridge.rs`, and `webview.rs` should be truly dead code
- **Actual**: CI Clippy fails with 8 dead code errors because the code is only used on macOS/Windows
- **Files Affected**:
  - `plugin/src/editor/assets.rs` - UI_ASSETS, get_asset, mime_type_from_path (used by macos.rs/windows.rs URL handlers)
  - `plugin/src/editor/bridge.rs` - PluginEditorBridge, new() (instantiated in macos.rs/windows.rs)
  - `plugin/src/editor/webview.rs` - WebViewConfig fields, create_ipc_handler, IPC_PRIMITIVES_JS (used in platform impls)
- **Steps to Reproduce**:
  1. Checkout `feature/m5-dead-code-cleanup` branch
  2. Run `act -W .github/workflows/ci.yml` (Linux CI)
  3. Observe Clippy failure in Check Engine job
- **Evidence**: See TC-001 actual result (full CI output)
- **Root Cause**: The low-level design incorrectly classified these suppressions as "stale from feature flag era" without considering platform-specific conditional compilation
- **Suggested Fix**: 
  - **Option A**: Restore the 8 suppressions with updated comments explaining they're platform-specific (quick fix)
  - **Option B**: Use `#[cfg(any(target_os = "macos", target_os = "windows"))]` attributes on the items (architecturally cleaner)
  - **Option C**: Move shared code into a platform-specific module hierarchy (largest refactor)
  - **Recommendation**: Option B - adds `#[cfg(...)]` to make platform usage explicit, satisfies Clippy, documents intent

---

_No additional issues found yet - testing blocked_

---

## Testing Notes

### Implementation Summary
The cleanup removed these suppressions:
- **webview.rs**: WebViewConfig, create_ipc_handler, IPC_PRIMITIVES_JS (3)
- **assets.rs**: UI_ASSETS, get_asset, mime_type_from_path (3)
- **bridge.rs**: struct and constructor (2)

Deleted dead code:
- **mod.rs**: EditorMessage enum, message_tx field

Refactored debug utility:
- **desktop/assets.rs**: Moved list_assets to test module
- **desktop/main.rs**: Removed --list-assets CLI flag

Updated comments (suppressions kept):
- **webview.rs**: resize, close trait methods (Rust analyzer false positives)
- **windows.rs**: hwnd field (Windows-specific, used in future methods)

### Risk Assessment
- **Low Risk**: All removed suppressions were analyzed and confirmed stale
- **Low Risk**: Deleted code was genuinely unused (EditorMessage)
- **Low Risk**: list_assets refactor only affects debug tooling
- **Medium Risk**: Plugin must be tested in actual DAW to verify no runtime regressions

---

## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Plugin verified in Ableton Live
- [ ] Issues documented for coder agent
- [ ] Ready for release: **NO** - Critical issue found, requires fix

**Blocking Issues**: Issue #1 (platform-specific dead code) must be resolved before proceeding.
