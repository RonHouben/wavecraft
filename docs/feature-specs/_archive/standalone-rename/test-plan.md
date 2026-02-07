# Test Plan: Rename `standalone` → `wavecraft-dev-server`

## Overview
- **Feature**: Rename `standalone` crate to `wavecraft-dev-server`
- **Spec Location**: `docs/feature-specs/standalone-rename/`
- **Date**: 2026-02-07
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 4 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask check` passes (all lint + tests)
- [x] Dev server help output verified
- [x] Manual dev server smoke test (start/stop)

## Test Cases

### TC-001: Full workspace checks

**Description**: Verifies linting and automated tests after rename.

**Preconditions**:
- Repo is clean or changes are expected for this feature.

**Steps**:
1. Run `cargo xtask check` from `engine/`.

**Expected Result**: Command completes successfully with all checks passing.

**Status**: ✅ PASS

**Actual Result**: `cargo xtask check` completed successfully with all linting and tests passing.

**Notes**: Command run from `engine/`.

---

### TC-002: Dev server help output

**Description**: Confirms the renamed binary reports help output under the new crate name.

**Preconditions**:
- Rust toolchain installed.

**Steps**:
1. Run `cargo run -p wavecraft-dev-server -- --help` from `engine/`.

**Expected Result**: Help output prints and shows the expected command name and options.

**Status**: ✅ PASS

**Actual Result**: Help output printed with expected description and options.

**Notes**: Running from repo root fails due to missing root `Cargo.toml`.

---

### TC-003: Manual dev server smoke test

**Description**: Ensures dev server can start and shut down cleanly under the new crate name.

**Preconditions**:
- Ports 9000 (server) and 5173 (UI) are free.

**Steps**:
1. Run `cargo run --manifest-path engine/xtask/Cargo.toml -- dev` from repo root.
2. Wait for server startup logs.
3. Stop the process after confirming startup.

**Expected Result**: Dev server starts without errors and exits cleanly when stopped.

**Status**: ✅ PASS

**Actual Result**: Dev server and Vite UI started successfully; process stopped cleanly.

**Notes**: Server logs showed WebSocket and UI endpoints at expected ports.

---

### TC-004: CLI start command smoke test (local SDK)

**Description**: Validates that the CLI can start a local SDK project and invoke the renamed dev server.

**Preconditions**:
- CLI builds locally.

**Steps**:
1. Create a local SDK project with `wavecraft create`.
2. Run `wavecraft start` with `--install` for the generated project.

**Expected Result**: CLI starts dev server successfully without crate-name errors.

**Status**: ✅ PASS

**Actual Result**: Dependencies installed, plugin built, WebSocket and UI servers started on requested ports.

**Notes**: `--no-install` fails if `ui/node_modules` is missing, which is expected behavior.

---

## Issues Found

_None._

## Testing Notes

- Dev server verified via `cargo run --manifest-path engine/xtask/Cargo.toml -- dev`.
- CLI start verified with `--install` to ensure dependency installation.

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] Issues documented for coder agent
- [x] Ready for release: YES
