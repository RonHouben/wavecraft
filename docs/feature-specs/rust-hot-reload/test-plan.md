# Test Plan: Rust Hot-Reload for Dev Mode

## Related Documents

- [Low-Level Design](./low-level-design-rust-hot-reload.md) — Architecture and design decisions
- [Implementation Plan](./implementation-plan.md) — Step-by-step implementation plan
- [Development Workflows](../../architecture/development-workflows.md) — Dev workflow documentation
- [Roadmap](../../roadmap.md) — Milestone 18.9

## Overview

- **Feature**: Rust Hot-Reload for Dev Mode (Milestone 18.9)
- **Spec Location**: `docs/feature-specs/rust-hot-reload/`
- **Date**: February 10, 2026

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 1 |
| ⏸️ BLOCKED | 5 |

## Prerequisites

- [x] `cargo xtask ci-check` passes — ✅ PASSED (58 UI + 42 engine + 61 CLI tests)
- [x] Test plugin generated with `wavecraft create`
- [x] Dev servers running with `wavecraft start`
- [x] File watcher active ("👀 Watching engine/src/ for changes")

---

## Test Cases

### TC-001: Happy Path — Parameter Addition

**Description**: Verify that adding a new processor to the signal chain causes its parameters to appear in the browser UI without restart.

**Steps**:
1. Note current parameters in browser at `http://localhost:5173` (InputGain, OutputGain = 2 total)
2. Open `engine/src/lib.rs` in editor
3. Change `signal: SignalChain![InputGain, OutputGain]` to `signal: SignalChain![InputGain, Oscillator, OutputGain]`
4. Save the file
5. Watch terminal for rebuild messages
6. Check browser for new oscillator parameters

**Expected Result**:
- Terminal shows rebuild + success messages
- Browser UI shows new oscillator parameters (~3–4 additional)
- No page refresh required, no WebSocket disconnection
- Total time from save to UI update: <10 seconds

**Status**: ⏸️ BLOCKED — Requires manual file modification

---

### TC-002: Value Preservation Across Reload

**Description**: Verify that existing parameter values are preserved when new parameters are added.

**Steps**:
1. In browser, adjust InputGain slider to 0.5
2. In `engine/src/lib.rs`, add another processor
3. Save the file
4. Wait for rebuild
5. Check InputGain value in browser

**Expected Result**:
- InputGain retains value 0.5
- New parameters appear with defaults
- No value reset

**Status**: ⏸️ BLOCKED — Depends on TC-001

---

### TC-003: Build Failure Handling

**Description**: Verify that a syntax error displays clear errors and preserves running state.

**Steps**:
1. Note current parameters in browser
2. Introduce syntax error in `engine/src/lib.rs` (e.g., remove semicolon)
3. Save the file
4. Watch terminal output

**Expected Result**:
- Terminal shows formatted compiler error with file/line info
- Browser UI remains unchanged (old parameters preserved)
- WebSocket stays connected, servers continue running

**Status**: ⏸️ BLOCKED — Requires manual file modification

---

### TC-004: Failure Recovery

**Description**: Verify that fixing a syntax error after a failed build successfully triggers hot-reload.

**Steps**:
1. Fix the syntax error from TC-003
2. Save the file
3. Watch terminal and browser

**Expected Result**:
- Terminal shows successful rebuild
- Browser updates with current parameters
- System recovers without restart

**Status**: ⏸️ BLOCKED — Depends on TC-003

---

### TC-005: Rapid Save Debouncing

**Description**: Verify that rapid saves trigger only 1–2 builds due to debouncing.

**Steps**:
1. Make small change in `engine/src/lib.rs`, save
2. Immediately (<500ms) make another change, save
3. Immediately (<500ms) make third change, save
4. Count rebuild messages in terminal

**Expected Result**:
- 1–2 rebuilds total (not 3)
- Final build reflects last saved state
- No excessive CPU usage

**Status**: ⏸️ BLOCKED — Requires rapid file modifications

---

### TC-006: Audio Reload (audio-dev feature)

**Description**: Verify DSP code changes cause brief audio dropout followed by resumed processing.

**Steps**:
1. Enable `audio-dev` feature and restart `wavecraft start`
2. Verify audio is playing
3. Modify DSP code in `engine/src/lib.rs`
4. Save and listen

**Expected Result**:
- Brief audio dropout (~200–500ms)
- Audio resumes with new processing
- No manual intervention required

**Status**: ⏸️ BLOCKED — Requires audio-dev feature + audio hardware

---

## Automated Test Results

All automated tests pass via `cargo xtask ci-check`:

| Suite | Tests | Status |
|-------|-------|--------|
| UI (Vitest) | 58 | ✅ PASS |
| Engine (Rust) | 42 | ✅ PASS |
| CLI (Rust) | 61 | ✅ PASS |

Key automated tests covering hot-reload:
- `BuildGuard` concurrency (unit tests in CLI)
- `FileWatcher` event filtering (unit tests in CLI)
- `useAllParameters` notification handler (Vitest)

---

## Manual Test Execution Instructions

For the user to manually execute the blocked test cases:

### Test 1 (TC-001): Happy Path
1. Keep `wavecraft start` running
2. Open `engine/src/lib.rs` in your editor
3. Change signal chain to include `Oscillator`: `signal: SignalChain![InputGain, Oscillator, OutputGain],`
4. Save the file
5. Watch terminal for "🔄 Rebuilding..." → "✓ Build succeeded"
6. Check browser at `http://localhost:5173` for new parameters

### Test 2 (TC-002): Value Preservation
1. In browser, move InputGain slider to a non-default value
2. Add another processor in Rust code, save
3. Check that InputGain slider kept its value

### Test 3 (TC-003): Build Failure
1. Remove a semicolon from signal chain line, save
2. Check terminal shows clear error, browser unchanged

### Test 4 (TC-004): Recovery
1. Fix the error from Test 3, save
2. Check terminal shows success, browser updates

### Test 5 (TC-005): Debouncing
1. Save 3 times rapidly (<1 second total)
2. Count rebuild messages — should be 1–2, not 3

---

## Sign-off

- [x] Automated tests pass — ✅
- [x] Dev server infrastructure verified — ✅
- [ ] Hot-reload behavior verified — ⏸️ BLOCKED (TC-001–TC-005)
- [ ] Audio reload tested — ⏸️ BLOCKED (TC-006)

**Status**: 🟡 PARTIAL PASS — Automated tests pass, manual tests require user execution

**Recommendation**: User should execute manual tests following the instructions above. If all pass, hand off to QA for static analysis.
