# Test Plan: CI Build Stage Removal

## Overview
- **Feature**: CI Build Stage Removal
- **Spec Location**: `docs/feature-specs/ci-build-stage-removal/`
- **Date**: 2026-02-06
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 0 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 9 |

## Prerequisites

- [x] `cargo xtask check` passes (confirmed by user)
- [ ] Feature branch `feature/ci-build-stage-removal` exists  
- [ ] Version bumped to 0.7.2 in Cargo.toml
- [ ] Commits pushed to remote

## Test Cases

### TC-001: Local Validation - Linting and Tests

**Description**: Verify all local checks pass before creating PR

**Preconditions**:
- Working directory is at repository root
- All changes committed

**Steps**:
1. Run `cargo xtask check` to execute all lint and test checks
2. Verify exit code is 0
3. Verify no errors in output

**Expected Result**: All checks pass with no errors

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-002: Workflow File Changes

**Description**: Verify the build-plugin job has been completely removed from ci.yml

**Preconditions**:
- Changes committed to `.github/workflows/ci.yml`

**Steps**:
1. Open `.github/workflows/ci.yml`
2. Search for "build-plugin" string
3. Search for "STAGE 3" comment block
4. Verify lines 218-272 have been removed

**Expected Result**: 
- No "build-plugin" job definition exists
- No "STAGE 3" section header exists
- File syntax is valid YAML

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-003: Documentation Updates

**Description**: Verify all references to build-plugin removed from ci-pipeline.md

**Preconditions**:
- Changes committed to `docs/guides/ci-pipeline.md`

**Steps**:
1. Open `docs/guides/ci-pipeline.md`
2. Search for "build-plugin" string
3. Verify workflow diagram shows only 6 jobs
4. Verify Jobs table does not include build-plugin
5. Verify "Release Artifacts" section removed or updated
6. Verify "Local Testing" table does not include build-plugin

**Expected Result**: 
- No references to build-plugin remain
- Diagram reflects actual workflow (6 validation jobs)
- All tables and sections consistent with current implementation

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-004: Version Verification

**Description**: Verify version 0.7.2 is set correctly in workspace

**Preconditions**:
- Version bump commit exists

**Steps**:
1. Check `engine/Cargo.toml` for `[workspace.package]` version
2. Verify version is "0.7.2"
3. Check that crate versions inherit from workspace

**Expected Result**: Workspace version is 0.7.2

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-005: PR Creation

**Description**: Create PR for feature branch and verify it triggers CI correctly

**Preconditions**:
- All local checks pass
- Branch pushed to remote
- GitHub CLI authenticated

**Steps**:
1. Create PR-summary.md in feature-specs folder
2. Run `gh pr create` with auto-generated title and body
3. Verify PR is created successfully
4. Note PR number and URL

**Expected Result**: PR created and visible on GitHub

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-006: GitHub CI Workflow Execution

**Description**: Verify CI workflow runs successfully on the PR

**Preconditions**:
- PR created and CI triggered

**Steps**:
1. Navigate to PR's "Checks" tab on GitHub
2. Wait for CI workflow to complete
3. Verify all jobs execute:
   - check-docs
   - check-ui
   - test-ui
   - prepare-engine
   - check-engine
   - test-engine
4. Verify NO "build-plugin" job appears

**Expected Result**: 
- All 6 validation jobs execute and pass
- No "build-plugin" job in the workflow run
- Green checkmarks for all jobs

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-007: Workflow Stage Count

**Description**: Verify the workflow has been simplified to 2 stages (not 3)

**Preconditions**:
- CI workflow running on PR

**Steps**:
1. View CI workflow run on GitHub Actions UI
2. Count the number of stage groupings visible
3. Verify stage structure matches:
   - Stage 1: Preparation (check-docs)
   - Stage 2: Validation (5 parallel jobs)
   - No Stage 3

**Expected Result**: Only 2 stages visible, no orphaned Stage 3

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-008: Version Display in UI

**Description**: Verify version badge shows "v0.7.2" when UI is running

**Preconditions**:
- Dev servers can be started
- Browser access available

**Steps**:
1. Run `cargo xtask dev` to start dev servers
2. Open browser to `http://localhost:5173`
3. Locate version badge in UI
4. Verify text displays "v0.7.2"

**Expected Result**: Version badge shows "v0.7.2"

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: Manual visual testing or Playwright MCP may be used

---

### TC-009: Documentation Link Validation

**Description**: Verify no broken links in updated documentation

**Preconditions**:
- Documentation changes committed

**Steps**:
1. Run `bash scripts/check-links.sh` if available
2. Manually verify links in ci-pipeline.md point to valid sections
3. Check that workflow file links are correct

**Expected Result**: All documentation links are valid

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

## Issues Found

_No issues found yet. This section will be populated during testing._

## Testing Notes

**Implementation Summary:**
- Removed lines 218-272 from `.github/workflows/ci.yml` (Stage 3 header + build-plugin job)
- Updated `docs/guides/ci-pipeline.md` to remove build-plugin references
- Version bumped to 0.7.2 in workspace Cargo.toml
- All changes committed in 2 commits

**Testing Strategy:**
1. Phase 1: Local validation (lint, tests, file verification)
2. Phase 2: PR creation and GitHub CI observation
3. Phase 3: Runtime verification (version display, link checking)

## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Issues documented for coder agent
- [ ] Ready for release: YES / NO
