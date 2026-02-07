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
| ❌ FAIL | 1 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] npm Trusted Publishing configured for `@wavecraft/*`
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

**Status**: ❌ FAIL

**Actual Result**: `publish-npm-components` succeeded, but `publish-npm-core` was skipped due to change detection and the `main` run still failed in `publish-npm-core` with an expired token being injected.

**Notes**:
- Branch run: 21779095466 (`fix/publish-packages-workflow`) — components published, core skipped.
- Main run: 21779071434 (`main`) — `publish-npm-core` failed with `Access token expired or revoked` and 404 on publish.

---

### TC-004: CI workflow validation (main)

**Description**: Validate `continuous-deploy.yml` on `main` uses OIDC (no token auth) and publishes npm packages successfully.

**Preconditions**:
- Workflow changes merged into `main`

**Steps**:
1. Trigger `continuous-deploy.yml` on `main`.
2. Verify `publish-npm-core` and `publish-npm-components` jobs succeed without token auth.

**Expected Result**: Jobs succeed and publish using OIDC.

**Status**: ❌ FAIL

**Actual Result**: `publish-npm-core` failed because token auth was still injected on `main`.

**Notes**: Run 21779071434 failed; update `main` workflow to remove token injection and re-run.

---

## Issues Found

- `publish-npm-core` on `main` still injects a token, causing publish to fail with `Access token expired or revoked`.

## Testing Notes


## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Issues documented for coder agent
- [ ] Ready for release: YES / NO
