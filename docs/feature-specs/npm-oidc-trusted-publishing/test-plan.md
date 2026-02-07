# Test Plan: npm OIDC Trusted Publishing

## Overview
- **Feature**: npm trusted publishing (OIDC) for `@wavecraft/core` and `@wavecraft/components`
- **Spec Location**: `docs/feature-specs/npm-oidc-trusted-publishing/`
- **Date**: 2026-02-07
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 2 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 1 |

## Prerequisites

- [ ] npm Trusted Publishing configured for `@wavecraft/*`
- [x] `continuous-deploy.yml` updated to use OIDC (no token auth)

## Test Cases

### TC-001: Local npm publish dry-run (core)

**Description**: Validate `@wavecraft/core` dry-run publish.

**Preconditions**:
- npm configured locally

**Steps**:
1. Run `npm publish --access public --dry-run` in `ui/packages/core`.

**Expected Result**: Dry-run completes successfully.

**Status**: ✅ PASS

**Actual Result**: Dry-run completed successfully.

**Notes**: npm warnings about `NODE_OPTIONS` and TypeScript peer version.

---

### TC-002: Local npm publish dry-run (components)

**Description**: Validate `@wavecraft/components` dry-run publish.

**Preconditions**:
- npm configured locally

**Steps**:
1. Run `npm publish --access public --dry-run` in `ui/packages/components`.

**Expected Result**: Dry-run completes successfully.

**Status**: ✅ PASS

**Actual Result**: Dry-run completed successfully.

**Notes**: npm warnings about `NODE_OPTIONS` and TypeScript peer version.

---

### TC-003: CI workflow validation (OIDC)

**Description**: Trigger Continuous Deploy on the fix branch and verify npm publish jobs succeed with OIDC.

**Preconditions**:
- Trusted Publisher configured for workflow filename

**Steps**:
1. Trigger `continuous-deploy.yml` on `fix/publish-packages-workflow`.
2. Verify `publish-npm-core` and `publish-npm-components` jobs succeed without token auth.

**Expected Result**: Jobs succeed and publish using OIDC.

**Status**: ⬜ NOT RUN

**Actual Result**: `publish-npm-core` and `publish-npm-components` were skipped because there were no changes under `ui/packages/*` on the branch.

**Notes**: Workflow run 21778865123 succeeded; npm jobs skipped by change detection.

---

## Issues Found

_None_

## Testing Notes


## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Issues documented for coder agent
- [ ] Ready for release: YES / NO
