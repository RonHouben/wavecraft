# Test Plan: crates.io Publishing with cargo-workspaces

## Overview
- **Feature**: crates.io Publishing with cargo-workspaces
- **Spec Location**: `docs/feature-specs/crates-io-publishing/`
- **Date**: 2026-02-06
- **Tester**: Tester Agent
- **Branch**: `feat/test-changes-for-publishing`
- **PR**: #26

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 14 |
| ❌ FAIL | 1 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask check` passes (all lint + tests)
- [ ] macOS-only checks pass (if applicable): bundle, sign, install

## Test Cases

### TC-001: Workspace Compiles Successfully

**Description**: Verify the workspace compiles after removing `version` from `[workspace.package]` and switching to independent crate versions.

**Steps**:
1. Run `cd engine && cargo check --workspace`

**Expected Result**: All crates compile without errors.

**Status**: ✅ PASS

**Actual Result**: All 8 crates compile without errors. Verified via `cargo xtask check` which includes `cargo check --workspace` internally.

**Notes**: Ran as part of TC-010 (`cargo xtask check`). All crates compiled successfully with independent versions.

---

### TC-002: Clippy Passes Without Warnings

**Description**: Verify clippy passes with `-D warnings` across the workspace.

**Steps**:
1. Run `cd engine && cargo clippy --workspace -- -D warnings`

**Expected Result**: Zero warnings or errors.

**Status**: ✅ PASS

**Actual Result**: Zero warnings. Clippy passed with `-D warnings` flag across all workspace crates. Output shows "Clippy OK".

**Notes**: Ran as part of TC-010 (`cargo xtask check`). All crates checked: wavecraft-protocol, wavecraft-macros, wavecraft-metering, wavecraft-bridge, wavecraft-dsp, wavecraft-core, standalone, xtask.

---

### TC-003: Workspace Root Cargo.toml Metadata

**Description**: Verify `engine/Cargo.toml` no longer has `version` in `[workspace.package]` and has `repository` field.

**Steps**:
1. Read `engine/Cargo.toml`
2. Verify `version` is NOT in `[workspace.package]`
3. Verify `repository = "https://github.com/RonHouben/wavecraft"` is present
4. Verify `edition`, `license`, `authors` are still present

**Expected Result**: No version, has repository, other fields intact.

**Status**: ✅ PASS

**Actual Result**: `engine/Cargo.toml` `[workspace.package]` section contains: `edition = "2024"`, `license = "MIT"`, `authors = ["Wavecraft Team"]`, `repository = "https://github.com/RonHouben/wavecraft"`. No `version` field present.

**Notes**: All original fields retained, only `version` removed and `repository` added.

---

### TC-004: All Publishable Crates Have Explicit Version

**Description**: Verify all 6 publishable crates have explicit `version = "0.7.1"` and required metadata.

**Steps**:
1. Check each crate's Cargo.toml for explicit `version = "0.7.1"`
2. Check each has `repository.workspace = true`
3. Check wavecraft-macros has `authors.workspace = true` and `description`
4. Check wavecraft-metering has `description`

**Expected Result**: All crates have explicit version and required metadata for crates.io.

**Status**: ✅ PASS

**Actual Result**: All 6 publishable crates verified:
- `wavecraft-protocol`: `version = "0.7.1"`, `repository.workspace = true` ✅
- `wavecraft-macros`: `version = "0.7.1"`, `repository.workspace = true`, `authors.workspace = true`, `description = "Procedural macros for Wavecraft plugin DSL"` ✅
- `wavecraft-metering`: `version = "0.7.1"`, `repository.workspace = true`, `description = "Real-time safe SPSC metering for Wavecraft audio plugins"` ✅
- `wavecraft-dsp`: `version = "0.7.1"`, `repository.workspace = true` ✅
- `wavecraft-bridge`: `version = "0.7.1"`, `repository.workspace = true` ✅
- `wavecraft-core`: `version = "0.7.1"`, `repository.workspace = true` ✅

**Notes**: Crates: wavecraft-protocol, wavecraft-macros, wavecraft-metering, wavecraft-dsp, wavecraft-bridge, wavecraft-core

---

### TC-005: Standalone Crate Excluded from Publishing

**Description**: Verify standalone crate has `publish = false` and explicit version.

**Steps**:
1. Read `engine/crates/standalone/Cargo.toml`
2. Verify `publish = false` is present
3. Verify it has an explicit version (not `version.workspace = true`)

**Expected Result**: `publish = false` present, explicit version.

**Status**: ✅ PASS

**Actual Result**: `standalone/Cargo.toml` has `version = "0.7.1"` (explicit) and `publish = false`. Does not use `version.workspace = true`.

**Notes**: Standalone correctly excluded from publishing while maintaining explicit version for local build compatibility.

---

### TC-006: xtask Version Reading Works

**Description**: Verify `read_workspace_version()` correctly reads version from `wavecraft-core/Cargo.toml`.

**Steps**:
1. Read `engine/xtask/src/lib.rs` and verify `read_workspace_version()` reads from `crates/wavecraft-core/Cargo.toml`
2. Run `cargo xtask bundle --debug` or verify the function works by checking the bundle command output references version 0.7.1

**Expected Result**: Function reads from `wavecraft-core/Cargo.toml` and returns "0.7.1".

**Status**: ✅ PASS

**Actual Result**: `read_workspace_version()` in `xtask/src/lib.rs` (line 82) reads from `crates/wavecraft-core/Cargo.toml` using `toml.get("package").and_then(|p| p.get("version"))`. This correctly targets the explicit version in wavecraft-core's `[package]` section.

**Notes**: Previously read from workspace root `Cargo.toml` via `toml.get("workspace").and_then(|w| w.get("package")).and_then(|p| p.get("version"))` which would fail with removed version.

---

### TC-007: Vite Version Reading Works

**Description**: Verify `getAppVersion()` in `vite.config.ts` correctly reads version from `wavecraft-core/Cargo.toml`.

**Steps**:
1. Read `ui/vite.config.ts` and verify regex matches format of `wavecraft-core/Cargo.toml`
2. Run `cd ui && npx vite build 2>&1 | head -5` or check that `__APP_VERSION__` resolves correctly

**Expected Result**: Version "0.7.1" is extracted correctly.

**Status**: ✅ PASS

**Actual Result**: `getAppVersion()` reads from `../engine/crates/wavecraft-core/Cargo.toml` using regex `/^\[package\]\s*\nname\s*=\s*"wavecraft-core"\s*\nversion\s*=\s*"([^"]+)"/m`. Runtime test with Node.js confirmed the regex correctly extracts `0.7.1` from the actual Cargo.toml file.

**Notes**: Regex verified by running a test script that applies the regex against the actual Cargo.toml contents. Output: `MATCH: version = 0.7.1`.

---

### TC-008: Visual UI Test — Version Badge

**Description**: Verify the version badge in the UI shows the correct version (not "dev").

**Steps**:
1. Start dev servers: `cargo xtask dev`
2. Navigate to http://localhost:5173
3. Check version badge displays "v0.7.1"

**Expected Result**: Version badge shows "v0.7.1".

**Status**: ✅ PASS

**Actual Result**: Version badge in footer shows **"v0.7.1 TEST"** which correctly reflects the version from `wavecraft-core/Cargo.toml`. The "TEST" suffix is expected for the dev build. Full UI is functional: WebSocket connected, meters displaying -60.0 dB L/R, IPC latency diagnostics showing ~0.7ms average, connection status showing "Connected (WebSocket)".

**Notes**: Screenshot captured as `tc-008-version-badge.png`. Dev servers started successfully via `cargo xtask dev` with Vite on port 5173 and WebSocket on port 9000. Previously blocked by sandbox EPERM restrictions on port binding.

---

### TC-009: Continuous Deploy Workflow Syntax Valid

**Description**: Verify the workflow YAML is valid and has the expected structure.

**Steps**:
1. Check workflow has 5 jobs: detect-changes, publish-cli, publish-engine, publish-npm-core, publish-npm-components
2. Verify publish-engine uses `cargo ws publish --from-git`
3. Verify publish-engine has dry-run step before actual publish
4. Verify git tag steps exist in publish-cli, publish-npm-core, publish-npm-components

**Expected Result**: Valid YAML with all expected jobs and steps.

**Status**: ✅ PASS

**Actual Result**: 
- 5 jobs present: detect-changes, publish-cli, publish-engine, publish-npm-core, publish-npm-components ✅
- publish-engine uses `cargo ws publish --from-git --yes --allow-branch main` ✅
- publish-engine has "Verify publishability (dry-run)" step before actual publish ✅
- publish-engine has `fetch-depth: 0` for git history access ✅
- publish-cli has "Create and push git tag" step: `wavecraft-cli-v$NEW_VERSION` ✅
- publish-npm-core has "Create and push git tag" step: `@wavecraft/core-v$NEW_VERSION` ✅
- publish-npm-components has "Create and push git tag" step: `@wavecraft/components-v$NEW_VERSION` ✅

**Notes**: All 5 jobs verified. All 3 non-engine publish jobs (cli, npm-core, npm-components) have git tag steps. The engine job uses cargo-workspaces which handles its own git tags.

---

### TC-010: Full Lint + Test Suite (cargo xtask check)

**Description**: Run the full pre-push CI validation suite.

**Steps**:
1. Run `cargo xtask check` from the engine directory

**Expected Result**: All lint checks and tests pass (~52 seconds).

**Status**: ✅ PASS

**Actual Result**: All checks passed in 46.0 seconds:
- Phase 1 Linting: PASSED (8.5s)
  - Rust formatting: OK
  - Clippy: OK (zero warnings)
  - ESLint: OK (zero warnings)
  - Prettier: OK
- Phase 2 Automated Tests: PASSED (37.4s)
  - Engine: 131 tests passed (8 crate test binaries, 1 integration test, 1 trybuild, 1 latency bench)
  - UI: 28 tests passed (6 test files via Vitest)

**Notes**: Complete output verified. All 131 engine tests + 28 UI tests passed with zero failures.

---

### TC-011: Engine Tests Pass

**Description**: Verify all Rust engine tests pass.

**Steps**:
1. Run `cd engine && cargo test --workspace`

**Expected Result**: All tests pass.

**Status**: ✅ PASS

**Actual Result**: All engine tests pass. Ran as part of TC-010 (`cargo xtask check`). 131 tests across all workspace crates: standalone (8+8+6+3), wavecraft-bridge (9), wavecraft-core (3+0+4+1), wavecraft-dsp (15+3), wavecraft-macros (0+3), wavecraft-metering (5), wavecraft-protocol (13), xtask (42+8).

**Notes**: Includes unit tests, integration tests, doc tests, trybuild tests, and latency benchmarks.

---

### TC-012: No Remaining workspace version References

**Description**: Verify no code still references `workspace.package.version` (the old pattern).

**Steps**:
1. Search codebase for `workspace.package.*version` or `version.workspace` in engine crates
2. Verify only plugin-template (not modified) still uses the pattern

**Expected Result**: No remaining references in engine crates (only plugin-template is acceptable).

**Status**: ✅ PASS

**Actual Result**: Searched for `version.workspace` across `engine/` directory. Found exactly 0 matches in engine crates. Only 2 matches found in `plugin-template/engine/` files (which is the scaffolding template and has its own workspace — not affected by this change):
- `plugin-template/engine/xtask/Cargo.toml:3` — `version.workspace = true`
- `plugin-template/engine/Cargo.toml:3` — `version.workspace = true`

**Notes**: The plugin-template files are correct and expected to use `version.workspace = true` since the template has its own workspace with a version defined.

---

### TC-013: Install cargo-workspaces Locally

**Description**: Install cargo-workspaces tool for local verification.

**Steps**:
1. Run `cargo install cargo-workspaces`
2. Verify `cargo ws --version` works

**Expected Result**: cargo-workspaces installs successfully.

**Status**: ✅ PASS

**Actual Result**: Successfully installed `cargo-workspaces v0.4.2`. `cargo ws --version` returns `cargo-workspaces 0.4.2`. Installed to `/Users/ronhouben/.cargo/bin/cargo-ws`.

**Notes**: Previously blocked by sandbox network restrictions. Installed 318 packages in 43.58s.

---

### TC-014: Verify Crate Metadata (cargo ws list)

**Description**: Verify cargo-workspaces can discover all publishable crates with correct metadata.

**Steps**:
1. Run `cd engine && cargo ws list`
2. Run `cd engine && cargo ws list -l`
3. Verify all 6 publishable crates listed with version 0.7.1
4. Verify standalone is excluded or marked as not publishable

**Expected Result**: All 6 publishable crates listed with correct versions.

**Status**: ✅ PASS

**Actual Result**: `cargo ws list -l` output:
```
wavecraft-protocol v0.7.1 crates/wavecraft-protocol
wavecraft-bridge   v0.7.1 crates/wavecraft-bridge
wavecraft-macros   v0.7.1 crates/wavecraft-macros
wavecraft-dsp      v0.7.1 crates/wavecraft-dsp
wavecraft-metering v0.7.1 crates/wavecraft-metering
wavecraft-core     v0.7.1 crates/wavecraft-core
```
All 6 publishable crates listed with version 0.7.1. Standalone crate correctly excluded (has `publish = false`).

**Notes**: Previously blocked by sandbox network restrictions. `cargo ws list` (without `-l`) also lists all 6 crates.

---

### TC-015: Dry-Run Publish Locally

**Description**: Test publishing without actually publishing to crates.io.

**Steps**:
1. Run `cd engine && cargo ws publish --from-git --dry-run --yes --no-git-push`
2. Verify all crates pass validation
3. Verify no errors about missing metadata or path dependencies

**Expected Result**: All crates pass validation in dry-run mode.

**Status**: ❌ FAIL

**Actual Result**: 3 of 6 crates passed dry-run, 3 failed:

| Crate | Status | Detail |
|-------|--------|--------|
| wavecraft-protocol | ✅ PASS | Packaged and dry-run uploaded successfully |
| wavecraft-bridge | ❌ FAIL | `dependency 'wavecraft-protocol' does not specify a version` |
| wavecraft-macros | ✅ PASS | Packaged and dry-run uploaded successfully |
| wavecraft-dsp | ❌ FAIL | `dependency 'wavecraft-macros' does not specify a version` |
| wavecraft-metering | ✅ PASS | Packaged and dry-run uploaded successfully |
| wavecraft-core | ❌ FAIL | `dependency 'nih_plug' does not specify a version` |

**Root Causes (2 issues):**

**Issue A — Missing version specifiers on path dependencies:**
Workspace-level `[workspace.dependencies]` defines wavecraft-* crates with only `path` but no `version`:
```toml
wavecraft-protocol = { path = "crates/wavecraft-protocol" }  # no version!
```
Should be:
```toml
wavecraft-protocol = { version = "0.7.1", path = "crates/wavecraft-protocol" }
```
Note: `cargo ws publish` may inject versions during actual publishing (dry-run doesn't rewrite manifests), but best practice is to include version specifiers.

**Issue B — nih_plug is NOT on crates.io (CRITICAL BLOCKER):**
`nih_plug` is specified as a git dependency only. `cargo search nih_plug` returns no results — it is NOT published on crates.io. This means `wavecraft-core` (which depends on nih_plug) **cannot be published to crates.io** until nih_plug is available there.

**Notes**: The dry-run warning states "Dry run doesn't check that all dependencies have been published" — so Issue A may be handled automatically by cargo-workspaces during real publishing. Issue B is a fundamental blocker that cargo-workspaces cannot resolve.

---

### TC-016: Check Crate Name Availability

**Description**: Verify crate names are available on crates.io (no squatting).

**Steps**:
1. For each of 6 crate names, run `cargo search <name> --limit 1`
2. Verify each name returns no existing crate

**Expected Result**: All 6 names available on crates.io.

**Status**: ✅ PASS

**Actual Result**: All 6 crate names return empty results on crates.io — all names are available:
- `wavecraft-protocol` — available ✅
- `wavecraft-macros` — available ✅
- `wavecraft-metering` — available ✅
- `wavecraft-dsp` — available ✅
- `wavecraft-bridge` — available ✅
- `wavecraft-core` — available ✅

**Notes**: Previously blocked by sandbox network restrictions. No name squatting detected.

---

## Issues Found

### Issue #1: Path Dependencies Missing Version Specifiers

- **Severity**: Medium
- **Test Case**: TC-015
- **Description**: Workspace-level `[workspace.dependencies]` in `engine/Cargo.toml` declares inter-crate dependencies with only `path` and no `version`. When `cargo publish` runs (even dry-run), it requires version specifiers for all dependencies.
- **Expected**: All workspace dependency declarations should include both `path` and `version` for publishable crates.
- **Actual**: e.g., `wavecraft-protocol = { path = "crates/wavecraft-protocol" }` — no version field.
- **Affected Crates**: wavecraft-bridge (depends on wavecraft-protocol), wavecraft-dsp (depends on wavecraft-macros)
- **Steps to Reproduce**:
  1. Run `cd engine && cargo ws publish --from-git --dry-run --yes --no-git-push`
  2. Observe errors: `dependency 'wavecraft-protocol' does not specify a version`
- **Suggested Fix**: Add version specifiers to all wavecraft-* workspace dependencies:
  ```toml
  wavecraft-protocol = { version = "0.7.1", path = "crates/wavecraft-protocol" }
  wavecraft-macros = { version = "0.7.1", path = "crates/wavecraft-macros" }
  wavecraft-dsp = { version = "0.7.1", path = "crates/wavecraft-dsp" }
  wavecraft-bridge = { version = "0.7.1", path = "crates/wavecraft-bridge" }
  wavecraft-metering = { version = "0.7.1", path = "crates/wavecraft-metering" }
  ```
- **Note**: `cargo ws publish` may auto-inject versions during actual (non-dry-run) publishing, but adding explicit versions is best practice and enables dry-run validation.

### Issue #2: nih_plug NOT on crates.io (CRITICAL BLOCKER)

- **Severity**: Critical
- **Test Case**: TC-015
- **Description**: `wavecraft-core` depends on `nih_plug` which is only available as a git dependency. `nih_plug` is NOT published on crates.io. Cargo requires all dependencies to be on crates.io for publishing.
- **Expected**: All dependencies resolvable from crates.io.
- **Actual**: `nih_plug` dependency specified as `git = "https://github.com/robbert-vdh/nih-plug.git"` with no crates.io version. `cargo search nih_plug` returns zero results.
- **Steps to Reproduce**:
  1. Run `cargo search nih_plug --limit 1` → empty results
  2. Run `cargo ws publish --from-git --dry-run` → `dependency 'nih_plug' does not specify a version`
- **Impact**: `wavecraft-core` **cannot be published to crates.io** until either:
  a. nih_plug is published to crates.io by its maintainer, OR
  b. wavecraft-core's nih_plug dependency is restructured (e.g., make nih_plug optional, or split wavecraft-core into a publishable core + non-publishable plugin integration crate)
- **Note**: The other 5 crates (wavecraft-protocol, wavecraft-macros, wavecraft-metering, wavecraft-bridge, wavecraft-dsp) do NOT depend on nih_plug and can potentially be published independently once Issue #1 is resolved. However, wavecraft-bridge depends on wavecraft-protocol (path only), and wavecraft-dsp depends on wavecraft-macros + wavecraft-protocol (path only).

## Testing Notes

- `cargo xtask check` is the most effective single command for validation — it covers linting (Rust fmt, Clippy, ESLint, Prettier) and all automated tests (Engine + UI) in ~24 seconds.
- Visual UI testing (TC-008) now passes — version badge shows "v0.7.1 TEST" correctly.
- The `plugin-template/` directory correctly retains `version.workspace = true` since it has its own independent workspace.
- Phase 4 verification (TC-013 through TC-016) was previously blocked by sandbox restrictions, now completed.
- TC-015 (dry-run publish) revealed **2 issues** that must be addressed before actual crates.io publishing:
  1. **Medium**: Path dependencies need version specifiers (fixable by coder agent)
  2. **Critical**: nih_plug is not on crates.io — blocks wavecraft-core publishing (upstream dependency issue)
- Crates that CAN potentially be published (no nih_plug dep): wavecraft-protocol, wavecraft-macros, wavecraft-metering
- Crates that need Issue #1 fix first: wavecraft-bridge, wavecraft-dsp
- Crate blocked by Issue #2: wavecraft-core

## Sign-off

- [x] All critical tests pass (except TC-015 which found real issues)
- [x] All high-priority tests pass
- [ ] Issues documented for coder agent (2 issues found)
- [ ] Ready for release: **NO** — Issue #2 (nih_plug not on crates.io) is a critical blocker for wavecraft-core publishing. Issue #1 (missing version specifiers) needs coder fix for remaining crates.
