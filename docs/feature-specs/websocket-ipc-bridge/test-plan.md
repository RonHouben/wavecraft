# Test Plan: WebSocket IPC Bridge

## Overview
- **Feature**: WebSocket IPC Bridge (Milestone 6)
- **Spec Location**: `docs/feature-specs/websocket-ipc-bridge/`
- **Date**: 2026-02-01
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 0 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 14 |

## Prerequisites

- [⬜] Docker is running: `docker info`
- [⬜] CI image exists: `docker images | grep vstkit-ci`
- [⬜] Local CI passes (see Phase 2)

## Test Cases

### TC-001: Local CI Pipeline Execution

**Description**: Verify all CI jobs pass locally using act

**Preconditions**:
- Docker Desktop running
- CI image built: `vstkit-ci:latest`

**Steps**:
1. Run full CI pipeline: `act -W .github/workflows/ci.yml --container-architecture linux/amd64 -P ubuntu-latest=vstkit-ci:latest --pull=false --artifact-server-path /tmp/act-artifacts`
2. Verify all Linux-compatible jobs pass:
   - check-ui (Prettier, ESLint, TypeScript)
   - test-ui (Vitest unit tests)
   - prepare-engine (UI build + Rust compilation)
   - check-engine (cargo fmt + clippy)
   - test-engine (cargo test)

**Expected Result**: All CI jobs complete successfully with exit code 0

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-002: Dev Server Startup (Default Port)

**Description**: Verify standalone dev server starts on default port 9000

**Preconditions**:
- Standalone binary built: `cargo build -p standalone`

**Steps**:
1. Start dev server: `cargo run -p standalone -- --dev-server`
2. Verify output shows: "Starting VstKit dev server on port 9000..."
3. Verify output shows: "[WebSocket] Server listening on ws://127.0.0.1:9000"
4. Press Ctrl+C to stop

**Expected Result**: Server starts successfully, listens on port 9000, shuts down cleanly

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-003: Dev Server Startup (Custom Port)

**Description**: Verify --port argument works

**Preconditions**:
- Standalone binary built

**Steps**:
1. Start dev server: `cargo run -p standalone -- --dev-server --port 9999`
2. Verify output shows: "Starting VstKit dev server on port 9999..."
3. Verify output shows: "[WebSocket] Server listening on ws://127.0.0.1:9999"
4. Press Ctrl+C to stop

**Expected Result**: Server starts on custom port 9999

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-004: WebSocket Connection Establishment

**Description**: Verify browser UI can connect to dev server via WebSocket

**Preconditions**:
- Dev server running on port 9000
- UI dev server running: `cd ui && npm run dev`

**Steps**:
1. Open browser to `http://localhost:5173`
2. Open browser DevTools console
3. Verify ConnectionStatus component shows "Connected (WebSocket)" with green indicator
4. Check console for WebSocket connection messages
5. Verify no errors in console

**Expected Result**: UI connects successfully, shows green connection status

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-005: Version Display

**Description**: Verify version 0.3.0 is displayed in the UI

**Preconditions**:
- Browser UI connected to dev server

**Steps**:
1. Locate VersionBadge component in footer
2. Verify it displays "v0.3.0"

**Expected Result**: Version badge shows v0.3.0

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-006: Parameter Get Operation (WebSocket)

**Description**: Verify parameter retrieval works over WebSocket

**Preconditions**:
- Browser UI connected to dev server

**Steps**:
1. Observe the "Gain" parameter slider
2. Verify slider displays a value (not null/undefined)
3. Verify slider is interactive (not frozen)
4. Check browser console for any errors

**Expected Result**: Gain parameter loads successfully, displays value

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-007: Parameter Set Operation (WebSocket)

**Description**: Verify parameter changes work over WebSocket

**Preconditions**:
- Browser UI connected to dev server

**Steps**:
1. Note current gain slider value
2. Drag slider to a different position
3. Verify slider updates immediately
4. Check browser console for any errors
5. Check dev server terminal for any errors

**Expected Result**: Parameter update sends via WebSocket, slider reflects new value

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-008: Meter Frame Display (WebSocket)

**Description**: Verify meters display real-time audio data via WebSocket

**Preconditions**:
- Browser UI connected to dev server

**Steps**:
1. Locate the Meters section with left/right channel bars
2. Verify meter bars are visible (not empty)
3. Observe meters for ~5 seconds
4. Verify meters show some activity (not stuck at zero)
5. Check refresh rate appears smooth (no obvious jitter)

**Expected Result**: Meters display and update smoothly via WebSocket polling

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-009: Latency Monitor (WebSocket)

**Description**: Verify latency monitoring works over WebSocket

**Preconditions**:
- Browser UI connected to dev server

**Steps**:
1. Locate the "Diagnostics" section with LatencyMonitor
2. Verify latency value is displayed (not "-- ms")
3. Verify "Average" latency shows a number
4. Observe for ~5 seconds, verify values update
5. Check that latency is reasonable (< 100ms for localhost)

**Expected Result**: Latency monitor shows round-trip times via WebSocket

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-010: Connection Recovery (Server Restart)

**Description**: Verify WebSocketTransport reconnects after server restart

**Preconditions**:
- Browser UI connected to dev server

**Steps**:
1. Verify ConnectionStatus shows "Connected"
2. Stop dev server (Ctrl+C in server terminal)
3. Verify ConnectionStatus changes to "Connecting..." (yellow)
4. Wait 5 seconds
5. Restart dev server: `cargo run -p standalone -- --dev-server`
6. Verify ConnectionStatus changes back to "Connected" (green)
7. Test parameter slider works after reconnection

**Expected Result**: Transport reconnects automatically, UI remains functional

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-011: Max Reconnection Attempts

**Description**: Verify transport stops reconnecting after max attempts

**Preconditions**:
- Browser UI connected to dev server

**Steps**:
1. Stop dev server (Ctrl+C)
2. Observe ConnectionStatus indicator
3. Check browser console for reconnection messages
4. Count reconnection attempts (should be 5 by default)
5. Wait ~30 seconds after last attempt
6. Verify no more reconnection attempts

**Expected Result**: Transport attempts 5 reconnections then stops

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-012: Help Text (CLI)

**Description**: Verify --help shows usage information

**Preconditions**:
- Standalone binary built

**Steps**:
1. Run: `cargo run -p standalone -- --help`
2. Verify output shows:
   - Description: "VstKit standalone app for UI development and testing"
   - Usage examples
   - --dev-server flag description
   - --port option description

**Expected Result**: Help text displays correctly

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-013: Native Mode Still Works (GUI)

**Description**: Verify standalone app still works in native GUI mode

**Preconditions**:
- Standalone binary built
- Running on macOS (GUI mode not available on Linux)

**Steps**:
1. Run without flags: `cargo run -p standalone`
2. Verify native window opens with WKWebView
3. Verify ConnectionStatus component is hidden (native transport)
4. Test parameter slider works
5. Test meters display
6. Close window

**Expected Result**: Native GUI mode works as before, uses NativeTransport

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: macOS-only test, skip on Linux CI

---

### TC-014: Build and Bundle (macOS)

**Description**: Verify plugin bundles still build correctly

**Preconditions**:
- Running on macOS

**Steps**:
1. Build bundles: `cargo xtask bundle --release`
2. Verify output shows VST3 and CLAP bundles created
3. Sign bundles: `cargo xtask sign --adhoc`
4. Verify signing: `cargo xtask sign --verify`
5. Check no errors in output

**Expected Result**: Bundles build and sign successfully

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: macOS-only test, cannot run in Docker

---

## Issues Found

### Issue #1: (Example Template)

- **Severity**: Critical / High / Medium / Low
- **Test Case**: TC-XXX
- **Description**: Detailed description of the issue
- **Expected**: Expected behavior
- **Actual**: Actual behavior observed
- **Steps to Reproduce**:
  1. Step 1
  2. Step 2
- **Evidence**: Command output, screenshots, logs
- **Suggested Fix**: If applicable

---

## Testing Notes

<!-- Additional observations, concerns, or recommendations -->

---

## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Issues documented for coder agent
- [ ] Ready for release: YES / NO

**Tester Signature**: ____________________  
**Date**: ____________________
