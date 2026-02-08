# Test Plan: CLI `start` UI Asset Fallback

## Overview
- **Feature**: CLI `wavecraft start` should build without `ui/dist` present by using embedded fallback UI assets.
- **Spec Location**: `docs/feature-specs/cli-start-ui-assets/`
- **Date**: 2026-02-08 (re-run)
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 5 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask ci-check` passes (all lint + tests)
- [x] CLI binary can be run from source (`cargo run --manifest-path cli/Cargo.toml -- --help`)

## Standard Testing Workflow

When testing CLI-generated plugins, use the `--output` flag to isolate test artifacts:

```bash
# Generate test plugin into SDK's build directory (gitignored)
cargo run --manifest-path cli/Cargo.toml -- create TestPlugin \
  --output target/tmp/test-plugin \
  --local-sdk

# Test the generated plugin
cd target/tmp/test-plugin
cargo run --manifest-path /path/to/wavecraft/cli/Cargo.toml -- start --install
```

**Why this workflow:**
- Test artifacts live in `target/tmp/` (gitignored)
- Easy cleanup: `rm -rf target/tmp/test-plugin`
- `--local-sdk` uses path dependencies, so local changes are tested without publishing

## Test Cases

### TC-001: Automated checks (ci-check)

**Description**: Ensure repo-wide linting and tests pass after the change.

**Preconditions**:
- None

**Steps**:
1. Run `cargo xtask ci-check` from repository root.

**Expected Result**: Command completes successfully with no failures.

**Status**: ✅ PASS

**Actual Result**: `cargo xtask ci-check` completed successfully from `engine/`; linting and all tests passed.

**Notes**: Running from repo root failed with `error: no such command: xtask` (exit 101). Re-ran from `engine/` on 2026-02-08; all checks green.
Re-ran on 2026-02-08 from `engine/`; all checks passed.

---

### TC-002: CLI `create` + `start` in a fresh project

**Description**: Verify `wavecraft start` no longer fails with `include_dir` panic when `ui/dist` is missing in the git checkout.

**Preconditions**:
- CLI can be executed via `cargo run --manifest-path cli/Cargo.toml -- ...`

**Steps**:
1. Create a temp directory (e.g., `target/tmp/cli-start-test`).
2. Run `cargo run --manifest-path cli/Cargo.toml -- create MyPlugin` in that temp directory.
3. Change into the newly created project folder.
4. Run `cargo run --manifest-path /Users/ronhouben/code/private/wavecraft/cli/Cargo.toml -- start`.

**Expected Result**: `start` proceeds without compile errors related to `ui/dist` or `include_dir` and begins the dev workflow.

**Status**: ✅ PASS

**Actual Result**: Using a local-sdk project, `wavecraft start --install` proceeded into plugin build without the previous `include_dir` panic.

**Notes**: Local test uses `--local-sdk` from repo root, so it validates the fixed path. Default template retest is not possible until tag `wavecraft-cli-v0.8.4` is published, but the failure signature was not observed.

---

### TC-003: Embedded fallback asset availability

**Description**: Validate `wavecraft-nih_plug` embedded assets include `index.html`.

**Preconditions**:
- None

**Steps**:
1. Run `cargo test -p wavecraft-nih_plug --lib` from `engine/`.

**Expected Result**: Tests pass, including `test_index_html_exists`.

**Status**: ✅ PASS

**Actual Result**: `cargo test -p wavecraft-nih_plug --lib` passed, including `test_index_html_exists`.

**Notes**: Also covered as part of `cargo xtask ci-check`.
Re-ran on 2026-02-08; tests passed.

---

### TC-004: UI port in use should fail fast

**Description**: When the UI port is already in use, `wavecraft start` should fail fast with a clear error before attempting to start servers.

**Preconditions**:
- A process is listening on port 5173

**Steps**:
1. Start a dummy HTTP server on port 5173.
2. Run `cargo run --manifest-path /Users/ronhouben/code/private/wavecraft/cli/Cargo.toml -- start --install --ui-port 5173` from `target/tmp/test-plugin`.

**Expected Result**: Command exits early with a clear error indicating the UI port is in use; servers are not started.

**Status**: ✅ PASS

**Actual Result**: Command failed immediately with `UI dev server port 5173 is already in use` before building the plugin or starting any servers.

**Notes**: Observed on 2026-02-08. Preflight check now fails fast.
Re-validated on 2026-02-08 with identical fail-fast behavior.

---

### TC-005: UI dev server starts on requested port

**Description**: With a free UI port, `wavecraft start` should start Vite on the requested port without switching to another port.

**Preconditions**:
- Ports 9000 and 5173 are free

**Steps**:
1. Ensure no processes are listening on ports 9000 or 5173.
2. Run `cargo run --manifest-path /Users/ronhouben/code/private/wavecraft/cli/Cargo.toml -- start --install --ui-port 5173` from `target/tmp/test-plugin`.
3. Confirm Vite reports `Local: http://localhost:5173/`.

**Expected Result**: WebSocket server starts on 9000; Vite starts on 5173 without auto-switching ports.

**Status**: ✅ PASS

**Actual Result**: Vite started successfully with `Local: http://localhost:5173/` and no port switching.

**Notes**: Observed on 2026-02-08 after clearing ports 9000 and 5173.
Re-validated on 2026-02-08; initial attempt failed because 5173 was still in use, then passed after freeing the port.

## Issues Found

### Issue #1: `wavecraft start` fails due to missing `ui/dist` in git checkout

- **Severity**: High
- **Test Case**: TC-002
- **Description**: The build panics at compile time because `include_dir!` points to `ui/dist` in the git checkout, which does not exist.
- **Expected**: CLI `start` should build without requiring `ui/dist` in the git dependency checkout.
- **Actual**: Build fails with `proc macro panicked` and `ui/dist is not a directory`.
- **Steps to Reproduce**:
	1. Create a plugin with `cargo run --manifest-path cli/Cargo.toml -- create MyPlugin`.
	2. `cd MyPlugin`
	3. `printf "y\n" | cargo run --manifest-path /Users/ronhouben/code/private/wavecraft/cli/Cargo.toml -- start`
- **Evidence**: Error points to `/Users/ronhouben/.cargo/git/checkouts/wavecraft-*/engine/crates/wavecraft-nih_plug/src/editor/assets.rs:14` using `include_dir!("$CARGO_MANIFEST_DIR/../../../ui/dist")`.
- **Suggested Fix**: Embed fallback assets within the crate (e.g., `assets/ui-dist`) and update `include_dir!` to that path.
- **Status**: Fixed in code; local validation passed. Awaiting release tag `wavecraft-cli-v0.8.4` for default template verification.

### Issue #2: UI port check does not fail fast

- **Severity**: Medium
- **Test Case**: TC-004
- **Description**: When the UI port is already in use, `wavecraft start` continues startup, prints "Both servers running", and only then fails when Vite reports the port error.
- **Expected**: Command should exit early with a clear error before starting any servers.
- **Actual**: WebSocket server starts; Vite fails with `Error: Port 5173 is already in use` after startup.
- **Steps to Reproduce**:
	1. Start a dummy server on port 5173.
	2. Run `cargo run --manifest-path /Users/ronhouben/code/private/wavecraft/cli/Cargo.toml -- start --install --ui-port 5173` from a generated plugin.
- **Evidence**: Vite error stack traces `Port 5173 is already in use` after the CLI reports both servers running.
- **Suggested Fix**: Ensure the UI port availability check correctly fails before server startup; consider binding to `0.0.0.0` or checking with OS-specific APIs.
- **Status**: Fixed; TC-004 now passes with fail-fast behavior.

## Testing Notes

- This plan focuses on preventing the `include_dir` panic caused by missing `ui/dist` in git checkouts.
- `cargo xtask ci-check` should be invoked from `engine/` in this repository; running from repo root fails with `no such command: xtask`.

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] Issues documented for coder agent
- [ ] Ready for release: NO (pending tag publication for default-template retest)
