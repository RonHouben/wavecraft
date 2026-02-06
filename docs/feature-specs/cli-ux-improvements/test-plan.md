# Test Plan: CLI UX Improvements

## Overview
- **Feature**: CLI UX Improvements
- **Spec Location**: `docs/feature-specs/cli-ux-improvements/`
- **Date**: 2026-02-06
- **Tester**: Tester Agent
- **Branch**: `feature/cli-ux-improvements`

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 0 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 10 |

## Prerequisites

- [ ] `cargo xtask check` passes (all lint + tests)
- [ ] CLI builds successfully
- [ ] Implementation progress document reviewed

## Test Cases

### TC-001: Help Command Displays Correctly

**Description**: Verify that `wavecraft --help` and `wavecraft new --help` display correctly and are discoverable.

**Preconditions**:
- CLI is built

**Steps**:
1. Run `wavecraft --help`
2. Run `wavecraft help`
3. Run `wavecraft new --help`
4. Run `wavecraft` with no arguments

**Expected Result**: 
- All help commands display usage information
- Available commands listed (new, help)
- `wavecraft new --help` shows available options
- No arguments displays brief usage message

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-002: No Interactive Prompts During Project Generation

**Description**: Verify that `wavecraft new` creates a project without prompting for any user input.

**Preconditions**:
- CLI is built
- Test directory is clean

**Steps**:
1. Run `wavecraft new test-plugin --output /tmp/test-no-prompts --no-git`
2. Observe the process (should complete immediately without waiting for input)
3. Check generated project exists

**Expected Result**: 
- No prompts for vendor, email, or URL
- Project created successfully
- Success message displayed

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-003: Default Values Used in Generated Project

**Description**: Verify that default placeholder values are used when no flags provided.

**Preconditions**:
- CLI is built

**Steps**:
1. Run `wavecraft new test-defaults --output /tmp/test-defaults --no-git`
2. Check `engine/Cargo.toml` for vendor metadata
3. Check `engine/src/lib.rs` for plugin metadata

**Expected Result**: 
- Vendor defaults to "Your Company"
- Email field is empty or not set
- URL field is empty or not set
- Generated files compile correctly

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-004: Optional Vendor Flags Override Defaults

**Description**: Verify that --vendor, --email, and --url flags work correctly.

**Preconditions**:
- CLI is built

**Steps**:
1. Run:
   ```bash
   wavecraft new test-custom \
     --vendor "Acme Audio" \
     --email "dev@acme.com" \
     --url "https://acme.com" \
     --output /tmp/test-custom \
     --no-git
   ```
2. Check generated files contain the custom values

**Expected Result**: 
- Custom vendor, email, and URL appear in generated files
- No prompts displayed

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-005: SDK Version Auto-Detection

**Description**: Verify that generated projects use the SDK version matching the CLI version.

**Preconditions**:
- CLI is built
- CLI version is 0.7.1

**Steps**:
1. Run `wavecraft --version` to confirm CLI version
2. Run `wavecraft new test-sdk-version --output /tmp/test-sdk-version --no-git`
3. Check `engine/Cargo.toml` for SDK dependency

**Expected Result**: 
- Generated project uses git tag matching CLI version (e.g., `tag = "0.7.1"`)
- No `v` prefix in version

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-006: Local SDK Mode Works from Repo Root

**Description**: Verify that --local-sdk flag generates path dependencies when run from wavecraft repo root.

**Preconditions**:
- CLI is built
- Running from wavecraft repository root
- `engine/crates` directory exists

**Steps**:
1. From repo root, run:
   ```bash
   cargo run --manifest-path cli/Cargo.toml -- new test-local-sdk \
     --output /tmp/test-local-sdk --no-git --local-sdk
   ```
2. Check `engine/Cargo.toml` in generated project

**Expected Result**: 
- Dependencies use `path = "..."` instead of `git = "..."`
- Path points to absolute location of wavecraft repo's `engine/crates`
- Project created successfully

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-007: Local SDK Error Outside Repo

**Description**: Verify that --local-sdk displays a clear error when run outside the wavecraft repository.

**Preconditions**:
- CLI is built
- Running from directory outside wavecraft repo (e.g., /tmp)

**Steps**:
1. `cd /tmp`
2. Run:
   ```bash
   /path/to/wavecraft new test-error --no-git --local-sdk
   ```
3. Check error message

**Expected Result**: 
- Clear error message: "Error: --local-sdk must be run from the wavecraft repository root."
- Mentions "Could not find: engine/crates"
- Exit code is non-zero

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-008: Internal Flags Hidden from Help

**Description**: Verify that --local-sdk is hidden from help output and --sdk-version is completely removed.

**Preconditions**:
- CLI is built

**Steps**:
1. Run `wavecraft new --help`
2. Check output for presence of flags

**Expected Result**: 
- `--local-sdk` is NOT shown in help output
- `--sdk-version` is NOT shown in help output (removed completely)
- Standard flags shown: --vendor, --email, --url, --output, --no-git

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-009: Generated Project Compiles

**Description**: Verify that a generated project compiles successfully.

**Preconditions**:
- CLI is built
- Project generated from TC-002

**Steps**:
1. Use project from TC-002 (`/tmp/test-no-prompts`)
2. Run:
   ```bash
   cd /tmp/test-no-prompts
   cd ui && npm install && cd ..
   cargo check --manifest-path engine/Cargo.toml
   ```

**Expected Result**: 
- UI dependencies install successfully
- Engine code compiles without errors
- No warnings about missing or incorrect dependencies

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-010: CI Workflow Compatibility

**Description**: Verify that the updated CI workflow syntax is correct.

**Preconditions**:
- Changes to `.github/workflows/template-validation.yml` reviewed

**Steps**:
1. Review `.github/workflows/template-validation.yml`
2. Check that it uses `--local-sdk` without path argument
3. Verify vendor/email/url flags are removed
4. Check syntax is valid YAML

**Expected Result**: 
- Workflow uses: `wavecraft new test-plugin --no-git --local-sdk`
- No `--vendor`, `--email`, `--url` flags in workflow
- YAML syntax is valid

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

## Issues Found

_No issues found yet. This section will be updated during testing._

## Testing Notes

### Test Environment
- macOS
- Rust toolchain: stable
- Node.js: 20+
- Git available

### Coverage
- Story 1 (Help Command): TC-001
- Story 2 (Remove Prompts): TC-002, TC-003, TC-004
- Story 3 (Clean Interface): TC-005, TC-006, TC-007, TC-008
- Story 4 (PATH Troubleshooting): Documentation only, verified by review

### Out of Scope
- PATH troubleshooting testing (user-facing documentation, no code to test)
- Windows/Linux testing (macOS is primary platform)

## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Issues documented for coder agent
- [ ] Ready for release: PENDING
