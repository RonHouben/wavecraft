# Test Plan: Internal Testing (Milestone 12)

## Overview
- **Feature**: Internal Testing — Comprehensive SDK validation before external beta
- **Spec Location**: `docs/feature-specs/internal-testing/`
- **Date**: February 3, 2026
- **Tester**: Tester Agent
- **Branch**: `feature/internal-testing`
- **Base Version**: 0.6.2

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 5 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 17 |

## Prerequisites

- [✅] macOS development environment
- [✅] Rust 1.75+ installed
- [✅] Node.js 18+ installed
- [⬜] Ableton Live 12 available
- [✅] Main repo at latest commit

---

## Phase 1: Automated Verification

### TC-001: cargo xtask check (All Tests)

**Description**: Verify all automated tests and linting pass

**Preconditions**:
- Working directory: `/Users/ronhouben/code/private/wavecraft`
- Clean git state (no uncommitted changes that affect tests)

**Steps**:
1. Run `cargo xtask check`
2. Observe output for any failures
3. Count passing tests

**Expected Result**: 
- All engine tests pass (110+)
- All UI tests pass (43+)
- All linting checks pass
- Zero errors or warnings
- Total time: ~52 seconds

**Status**: ✅ PASS

**Actual Result**: 
- **Engine tests**: 113 passed, 0 failed (2 ignored - environment-dependent)
- **UI tests**: 43 passed, 0 failed
- **Linting**: All checks passed (Rust fmt + clippy, ESLint, Prettier)
- **Total time**: 27.5 seconds (faster than expected!)

**Notes**: 
- No issues found
- All docstring examples pass or are appropriately ignored
- Test suite is comprehensive and runs quickly 

---

### TC-002: Engine Tests Only

**Description**: Verify Rust test suite passes independently

**Preconditions**:
- Working directory: `/Users/ronhouben/code/private/wavecraft/engine`

**Steps**:
1. Run `cargo test --workspace`
2. Count passing tests
3. Check for ignored tests

**Expected Result**: 
- All tests pass
- 110+ passing tests
- Some environment-dependent tests may be ignored (acceptable)

**Status**: ✅ PASS

**Actual Result**: 113 passed, 0 failed, 2 ignored

**Notes**: Covered by TC-001, no independent issues 

---

### TC-003: UI Tests Only

**Description**: Verify React/TypeScript test suite passes independently

**Preconditions**:
- Working directory: `/Users/ronhouben/code/private/wavecraft/ui`
- Node modules installed

**Steps**:
1. Run `npm test`
2. Count passing tests
3. Check coverage if available

**Expected Result**: 
- All tests pass
- 43+ passing tests
- No test timeouts or failures

**Status**: ✅ PASS

**Actual Result**: 43 passed, 0 failed

**Notes**: Covered by TC-001, no independent issues 

---

### TC-004: Rust Linting

**Description**: Verify Rust code follows formatting and lint standards

**Preconditions**:
- Working directory: `/Users/ronhouben/code/private/wavecraft/engine`

**Steps**:
1. Run `cargo fmt --check`
2. Run `cargo clippy --workspace --all-targets -- -D warnings`
3. Check for any warnings or errors

**Expected Result**: 
- No formatting issues
- No clippy warnings
- Exit code 0

**Status**: ✅ PASS

**Actual Result**: Formatting OK, Clippy OK, exit code 0

**Notes**: Covered by TC-001, no independent issues 

---

### TC-005: UI Linting

**Description**: Verify TypeScript/React code follows formatting and lint standards

**Preconditions**:
- Working directory: `/Users/ronhouben/code/private/wavecraft/ui`

**Steps**:
1. Run `npm run lint`
2. Run `npm run format:check`
3. Run `npm run typecheck`

**Expected Result**: 
- No ESLint errors
- No Prettier formatting issues
- No TypeScript type errors

**Status**: ✅ PASS

**Actual Result**: ESLint OK, Prettier OK, TypeScript OK

**Notes**: Covered by TC-001, no independent issues 

---

## Phase 2: Manual Workflow Testing

### TC-006: Fresh Clone Experience

**Description**: Simulate new developer cloning the template and building a plugin

**Preconditions**:
- Fresh terminal session
- No cached dependencies
- Template exists at `wavecraft-plugin-template/`

**Steps**:
1. Create test directory: `mkdir -p /tmp/wavecraft-internal-test && cd /tmp/wavecraft-internal-test`
2. Clone template: `cp -r /Users/ronhouben/code/private/wavecraft/wavecraft-plugin-template test-plugin`
3. Time the following:
   - `cd test-plugin/ui && npm install`
   - `npm run build`
   - `cd ../engine && cargo xtask bundle --release`
4. Verify artifacts exist in `engine/target/bundled/`

**Expected Result**: 
- No errors during any step
- `test-plugin/ui/dist/` contains built UI
- `test-plugin/engine/target/bundled/` contains VST3 and CLAP bundles
- Total time < 30 minutes (excluding download time)

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-007: Plugin Loads in Ableton Live

**Description**: Verify the bundled plugin loads correctly in Ableton Live

**Preconditions**:
- TC-006 passed (plugin bundled successfully)
- Ableton Live 12 installed
- Plugin installed to system location

**Steps**:
1. Run `cargo xtask install` from `test-plugin/engine/`
2. Open Ableton Live
3. Rescan plugins (if needed)
4. Add plugin to an audio track
5. Observe plugin UI opens

**Expected Result**: 
- Plugin appears in Ableton's plugin browser
- Plugin loads without errors
- Plugin UI window opens and renders correctly
- No crashes or warnings in macOS Console

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-008: Audio Passthrough

**Description**: Verify audio passes through the plugin cleanly

**Preconditions**:
- TC-007 passed (plugin loaded in Ableton)

**Steps**:
1. Load a test audio file on the track
2. Play the track with plugin active
3. Listen for glitches or dropouts
4. Check CPU usage in Ableton

**Expected Result**: 
- Clean audio output
- No glitches or pops
- CPU usage reasonable (<5% for simple gain plugin)
- No warnings in DAW

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-009: Parameter Sync (UI → DAW)

**Description**: Verify parameter changes in plugin UI update DAW automation

**Preconditions**:
- TC-007 passed (plugin loaded)
- Automation visible in DAW

**Steps**:
1. Open plugin UI
2. Enable automation recording for a parameter
3. Move a slider in the plugin UI
4. Observe DAW automation lane

**Expected Result**: 
- DAW automation lane shows parameter movement
- Automation values match UI slider position
- No lag or glitches

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-010: Parameter Sync (DAW → UI)

**Description**: Verify DAW automation updates plugin UI

**Preconditions**:
- TC-007 passed (plugin loaded)
- Automation written to timeline

**Steps**:
1. Draw automation curve in DAW for a parameter
2. Open plugin UI
3. Play timeline
4. Observe plugin slider movement

**Expected Result**: 
- Plugin UI slider follows automation curve
- Movement is smooth and synchronized
- No visual glitches

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-011: State Save/Restore

**Description**: Verify plugin state persists across project save/load

**Preconditions**:
- TC-007 passed (plugin loaded)

**Steps**:
1. Set plugin parameters to non-default values
2. Save Ableton project
3. Close Ableton
4. Reopen Ableton and load project
5. Open plugin UI
6. Check parameter values

**Expected Result**: 
- All parameter values restored correctly
- UI reflects saved state
- No warnings or errors

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-012: Multi-Instance Test

**Description**: Verify multiple plugin instances work independently

**Preconditions**:
- TC-007 passed (plugin loaded)

**Steps**:
1. Add 3 instances of the plugin to different tracks
2. Set different parameter values for each
3. Play audio through all instances
4. Open all 3 UIs simultaneously

**Expected Result**: 
- All instances work independently
- Parameter values don't cross-contaminate
- No audio glitches with multiple instances
- No crashes or hangs

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-013: Low Buffer Size (64 samples)

**Description**: Test plugin stability at low buffer sizes

**Preconditions**:
- TC-007 passed (plugin loaded)

**Steps**:
1. Change Ableton buffer size to 64 samples
2. Play audio through plugin
3. Move parameters while playing
4. Listen for glitches

**Expected Result**: 
- No dropouts or glitches
- CPU usage may increase (acceptable)
- Plugin remains responsive
- No crashes

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-014: High Buffer Size (4096 samples)

**Description**: Test plugin behavior at high buffer sizes

**Preconditions**:
- TC-007 passed (plugin loaded)

**Steps**:
1. Change Ableton buffer size to 4096 samples
2. Play audio through plugin
3. Move parameters while playing
4. Check parameter responsiveness

**Expected Result**: 
- Audio remains clean
- Parameters respond (with expected latency)
- No issues or warnings
- UI remains responsive

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-015: Rapid Parameter Automation

**Description**: Test plugin with dense automation curves

**Preconditions**:
- TC-007 passed (plugin loaded)

**Steps**:
1. Draw very dense automation curve (many breakpoints)
2. Play timeline
3. Observe CPU usage
4. Check UI responsiveness
5. Listen for audio artifacts

**Expected Result**: 
- UI keeps up with automation (may lag slightly, acceptable)
- No crashes
- Audio remains glitch-free
- CPU usage increases (acceptable)

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-016: Developer Workflow (cargo xtask dev)

**Description**: Verify the WebSocket-based development workflow

**Preconditions**:
- Working directory: `/tmp/wavecraft-internal-test/test-plugin`

**Steps**:
1. Run `cargo xtask dev` from project root
2. Wait for both servers to start
3. Open browser to `http://localhost:5173`
4. Check connection status in UI
5. Check meter data updates
6. Move a parameter slider
7. Edit `ui/src/App.tsx` and save
8. Observe browser hot reload
9. Press Ctrl+C to stop servers

**Expected Result**: 
- Both servers start successfully
- UI loads in browser
- Connection status shows "Connected"
- Meters display real data (not mock)
- Parameters sync with engine
- Hot reload works for UI changes
- Clean shutdown on Ctrl+C

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-017: UI Open/Close Stress Test

**Description**: Test for memory leaks when repeatedly opening/closing UI

**Preconditions**:
- TC-007 passed (plugin loaded in Ableton)

**Steps**:
1. Note initial memory usage in Activity Monitor
2. Open plugin UI
3. Close plugin UI
4. Repeat steps 2-3 ten times
5. Check memory usage again

**Expected Result**: 
- Memory usage doesn't grow significantly
- No crashes or hangs
- UI opens reliably each time
- Performance remains consistent

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

## Phase 3: Documentation Review

### TC-018: SDK Getting Started Guide

**Description**: Verify the SDK guide is accurate end-to-end

**Preconditions**:
- Fresh perspective (read as newcomer)

**Steps**:
1. Open `docs/guides/sdk-getting-started.md`
2. Follow each instruction literally
3. Execute all commands exactly as written
4. Note any unclear sections
5. Check for missing prerequisites

**Expected Result**: 
- All commands execute successfully
- No undocumented steps required
- Code examples work as shown
- Prerequisites clearly stated
- No broken links

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-019: Documentation Link Validation

**Description**: Verify all documentation links are valid

**Preconditions**:
- Working directory: `/Users/ronhouben/code/private/wavecraft`

**Steps**:
1. Find all markdown files in `docs/`
2. Extract relative links
3. Verify each link resolves to an existing file
4. Check for broken cross-references

**Expected Result**: 
- All relative links resolve correctly
- No 404s or broken references
- Cross-references between docs are correct

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-020: Template README Accuracy

**Description**: Verify template README instructions work

**Preconditions**:
- Fresh clone in `/tmp/` (from TC-006)

**Steps**:
1. Open `test-plugin/README.md`
2. Follow "Quick Start" instructions
3. Follow "Development Workflow" examples
4. Verify all commands produce expected results

**Expected Result**: 
- Quick Start produces working plugin
- All code examples compile
- All commands execute without errors
- Instructions match reality

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-021: Template Independence

**Description**: Verify template has no hidden monorepo dependencies

**Preconditions**:
- Template cloned to `/tmp/` (from TC-006)

**Steps**:
1. Inspect `test-plugin/engine/Cargo.toml` for path dependencies
2. Check for relative paths outside template directory
3. Run `cargo clean` and rebuild from scratch
4. Verify build succeeds without main repo present

**Expected Result**: 
- No `path = "../../../engine/crates/..."` dependencies
- Template builds independently
- All dependencies come from git or crates.io
- No assumptions about monorepo structure

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-022: Code Signing Workflow

**Description**: Verify ad-hoc signing workflow for local testing

**Preconditions**:
- Working directory: `/Users/ronhouben/code/private/wavecraft/engine`
- Plugin bundled

**Steps**:
1. Run `cargo xtask sign --adhoc`
2. Run `cargo xtask sign --verify`
3. Check for any errors

**Expected Result**: 
- Signing completes without errors
- Verification passes
- No warnings about invalid signatures
- Documentation in `docs/guides/macos-signing.md` matches actual behavior

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

## Issues Found

_Issues will be documented here as they are discovered during testing_

---

## Testing Notes

_General observations and notes will be added here during testing_

---

## Sign-off

- [ ] All Phase 1 tests pass
- [ ] All Phase 2 tests pass
- [ ] All Phase 3 tests pass
- [ ] Critical issues resolved
- [ ] High issues resolved
- [ ] Version bumped to 0.6.3
- [ ] Ready for external beta testing (M13): YES / NO
