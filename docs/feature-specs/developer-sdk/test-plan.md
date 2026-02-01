# Test Plan: Developer SDK

## Overview
- **Feature**: Developer SDK (Milestone 8, Phase 1)
- **Spec Location**: `docs/feature-specs/developer-sdk/`
- **Date**: February 1, 2026
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 0 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 15 |

## Prerequisites

- [ ] Docker is running: `docker info`
- [ ] CI image exists: `docker images | grep vstkit-ci`
- [ ] Local CI passes (see Phase 2)

## Test Cases

### TC-001: CI Pipeline Passes

**Description**: Verify the full CI pipeline passes after SDK restructuring

**Preconditions**:
- Docker Desktop is running
- vstkit-ci image is available

**Steps**:
1. Check Docker status: `docker info`
2. Verify CI image: `docker images | grep vstkit-ci`
3. Run full CI pipeline: 
   ```bash
   act -W .github/workflows/ci.yml \
       --container-architecture linux/amd64 \
       -P ubuntu-latest=vstkit-ci:latest \
       --pull=false \
       --artifact-server-path /tmp/act-artifacts
   ```

**Expected Result**: All CI jobs pass (check-ui, test-ui, prepare-engine, check-engine, test-engine)

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-002: Crate Names Verification

**Description**: Verify all crates have been renamed with vstkit-* prefix

**Preconditions**:
- None

**Steps**:
1. List crates: `ls -1 /Users/ronhouben/code/private/vstkit/engine/crates/`
2. Verify presence of:
   - vstkit-protocol
   - vstkit-bridge
   - vstkit-metering
   - vstkit-dsp
   - vstkit-core

**Expected Result**: All 5 SDK crates exist with correct naming

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-003: Workspace Compilation

**Description**: Verify entire workspace compiles successfully

**Preconditions**:
- None

**Steps**:
1. Clean build: `cd /Users/ronhouben/code/private/vstkit/engine && cargo clean`
2. Check workspace: `cargo check --workspace`

**Expected Result**: No compilation errors, all crates compile

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-004: Engine Test Suite

**Description**: Verify all Rust unit and integration tests pass

**Preconditions**:
- Workspace compiles (TC-003)

**Steps**:
1. Run engine tests: `cd /Users/ronhouben/code/private/vstkit/engine && cargo test --workspace`

**Expected Result**: 43+ tests pass, no failures

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-005: UI Test Suite

**Description**: Verify all React/TypeScript tests pass

**Preconditions**:
- None

**Steps**:
1. Run UI tests: `cd /Users/ronhouben/code/private/vstkit/ui && npm test`

**Expected Result**: 35+ tests pass, no failures

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-006: Plugin Bundle Build

**Description**: Verify plugin bundles (VST3/CLAP) build successfully

**Preconditions**:
- Workspace compiles (TC-003)

**Steps**:
1. Build bundles: `cd /Users/ronhouben/code/private/vstkit/engine && cargo xtask bundle --release`
2. Verify outputs exist:
   - `engine/target/bundled/vstkit-core.vst3/`
   - `engine/target/bundled/vstkit-core.clap`

**Expected Result**: Both VST3 and CLAP bundles created without errors

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-007: Plugin Code Signing

**Description**: Verify ad-hoc code signing works

**Preconditions**:
- Plugin bundles built (TC-006)

**Steps**:
1. Sign bundles: `cd /Users/ronhouben/code/private/vstkit/engine && cargo xtask sign --adhoc`
2. Verify signatures: `cargo xtask sign --verify`

**Expected Result**: Both bundles signed successfully, verification passes

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-008: SDK API Exports

**Description**: Verify vstkit_core::prelude module exports all necessary SDK types

**Preconditions**:
- Workspace compiles (TC-003)

**Steps**:
1. Check prelude exports: `grep -A 30 "pub mod prelude" /Users/ronhouben/code/private/vstkit/engine/crates/vstkit-core/src/lib.rs`
2. Verify these exports exist:
   - Processor trait
   - ParamSet trait
   - vstkit_params! macro
   - ParameterHost trait
   - Common types (Transport, ParamId, ParameterInfo, etc.)

**Expected Result**: All essential SDK types available in prelude

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-009: Template Repository Structure

**Description**: Verify template repository has correct structure

**Preconditions**:
- None

**Steps**:
1. Check template exists: `ls -la /Users/ronhouben/code/private/vstkit/vstkit-plugin-template/`
2. Verify directories:
   - `engine/` (Rust plugin code)
   - `ui/` (React frontend)
   - `xtask/` (Build scripts)
3. Verify key files:
   - `README.md`
   - `Cargo.toml`
   - `engine/Cargo.toml`
   - `ui/package.json`

**Expected Result**: All required directories and files present

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-010: Template Plugin Compilation

**Description**: Verify template plugin compiles successfully

**Preconditions**:
- Template structure verified (TC-009)

**Steps**:
1. Navigate to template: `cd /Users/ronhouben/code/private/vstkit/vstkit-plugin-template/engine`
2. Check template: `cargo check`
3. Build template: `cargo build --release`

**Expected Result**: Template plugin compiles without errors

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-011: Template UI Build

**Description**: Verify template UI builds successfully

**Preconditions**:
- Template structure verified (TC-009)

**Steps**:
1. Navigate to template UI: `cd /Users/ronhouben/code/private/vstkit/vstkit-plugin-template/ui`
2. Install dependencies: `npm ci`
3. Build UI: `npm run build`
4. Verify output: `ls -la dist/`

**Expected Result**: UI builds, dist/ folder contains index.html, JS, and CSS files

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-012: Template Bundle Build

**Description**: Verify template plugin can be bundled

**Preconditions**:
- Template plugin compiles (TC-010)
- Template UI builds (TC-011)

**Steps**:
1. Navigate to template: `cd /Users/ronhouben/code/private/vstkit/vstkit-plugin-template/engine`
2. Build bundles: `cargo xtask bundle`
3. Verify outputs exist in `target/bundled/`

**Expected Result**: Template plugin bundles successfully

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-013: Documentation Completeness

**Description**: Verify SDK documentation is complete and accessible

**Preconditions**:
- None

**Steps**:
1. Check architecture docs: `cat /Users/ronhouben/code/private/vstkit/docs/architecture/high-level-design.md | grep -A 50 "SDK Architecture"`
2. Check getting started guide: `ls -la /Users/ronhouben/code/private/vstkit/docs/guides/sdk-getting-started.md`
3. Check README links: `grep "SDK Getting Started" /Users/ronhouben/code/private/vstkit/README.md`

**Expected Result**: 
- SDK Architecture section exists in high-level-design.md
- SDK Getting Started guide exists
- README links to SDK documentation

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-014: Version Consistency

**Description**: Verify version is 0.4.0 across all SDK components

**Preconditions**:
- None

**Steps**:
1. Check workspace version: `grep "^version" /Users/ronhouben/code/private/vstkit/engine/Cargo.toml`
2. Check main plugin: `cargo metadata --no-deps | grep vstkit-core | grep version`
3. Check UI package: `grep "version" /Users/ronhouben/code/private/vstkit/ui/package.json`

**Expected Result**: All versions show 0.4.0

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-015: Plugin Functionality (DAW Test)

**Description**: Verify the renamed plugin still works correctly in a DAW

**Preconditions**:
- Plugin built and signed (TC-006, TC-007)

**Steps**:
1. Install plugin: `cd /Users/ronhouben/code/private/vstkit/engine && cargo xtask install`
2. Open Ableton Live
3. Create a new audio track
4. Load "vstkit-core" plugin
5. Verify:
   - UI renders correctly
   - Parameter slider works
   - Meters display audio levels
   - Version badge shows "v0.4.0"
   - Resize handle works

**Expected Result**: Plugin loads and functions normally, all UI elements work

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

## Issues Found

*No issues yet — testing in progress*

---

## Testing Notes

### Phase 1: CI Pipeline
Starting with automated CI validation to catch any build/test failures.

### Phase 2: SDK Structure
Verifying the crate restructuring and API exports are correct.

### Phase 3: Template Validation
Testing the template repository compiles and works independently.

### Phase 4: Integration Testing
End-to-end testing with DAW to ensure SDK changes don't break functionality.

---

## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Issues documented for coder agent
- [ ] Ready for release: YES / NO
