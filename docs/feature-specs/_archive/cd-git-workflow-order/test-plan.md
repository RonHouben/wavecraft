# Test Plan: CD Auto-Bump Tag-Only Publishing

## Overview
- **Feature**: CD Auto-Bump Tag-Only Publishing Fix
- **Spec Location**: `docs/feature-specs/cd-git-workflow-order/`
- **Date**: 2026-02-08
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 12 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 1 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask ci-check` passes (all lint + tests)
- [x] Branch: `fix/cd-tag-only-publishing` with commit `dda6776`

## Test Cases

### TC-001: CI Check — No Regressions

**Description**: Verify `cargo xtask ci-check` passes with no regressions.

**Steps**:
1. Run `cd engine && cargo xtask ci-check`

**Expected Result**: All lint and test phases pass.

**Status**: ✅ PASS

**Actual Result**: All checks passed in 12.4s:
- Linting: PASSED (6.0s) — cargo fmt, clippy, ESLint, Prettier all OK
- Automated Tests: PASSED (6.4s) — 165 engine tests + 28 UI tests all passed

---

### TC-002: No `git push origin main` Remaining

**Description**: Verify that no `git push origin main` command exists anywhere in the workflow file.

**Steps**:
1. Run `grep -n "git push origin main" .github/workflows/continuous-deploy.yml`

**Expected Result**: No matches (exit code 1).

**Status**: ✅ PASS

**Actual Result**: Zero matches. Exit code 1 (no match). All direct pushes to `main` have been removed.

---

### TC-003: No `git pull --rebase` Remaining

**Description**: Verify that no `git pull --rebase` commands remain in the workflow.

**Steps**:
1. Run `grep -n "git pull --rebase" .github/workflows/continuous-deploy.yml`

**Expected Result**: No matches (exit code 1).

**Status**: ✅ PASS

**Actual Result**: Zero matches. Exit code 1 (no match). All rebase steps have been removed.

---

### TC-004: All Git Push Commands Are Tag-Only

**Description**: Verify every `git push` in the workflow pushes only tags (not branches).

**Steps**:
1. Run `grep -n "git push" .github/workflows/continuous-deploy.yml`
2. Verify each line pushes a specific tag or `--tags`

**Expected Result**: All `git push` lines should push tags only:
- `publish-cli`: `git push origin "wavecraft-cli-v${{ ... }}"`
- `publish-engine`: `git push origin --tags`
- `publish-npm-core`: `git push origin "@wavecraft/core-v${{ ... }}"`
- `publish-npm-components`: `git push origin "@wavecraft/components-v${{ ... }}"`

**Status**: ✅ PASS

**Actual Result**: Exactly 4 `git push` lines found:
- L209: `git push origin "wavecraft-cli-v${{ steps.final.outputs.version }}"`
- L283: `git push origin --tags`
- L386: `git push origin "@wavecraft/core-v${{ steps.final.outputs.version }}"`
- L493: `git push origin "@wavecraft/components-v${{ steps.final.outputs.version }}"`

All are tag-only pushes. No branch pushes exist.

---

### TC-005: `publish-cli` — Local-Only Commit Pattern

**Description**: Verify the `publish-cli` job commits auto-bump locally without pushing.

**Steps**:
1. Read the "Commit auto-bump locally" step in `publish-cli` job
2. Verify step name is "Commit auto-bump locally" (not "Commit and push auto-bump")
3. Verify `git add cli/Cargo.toml` and `git commit` are present
4. Verify NOTE comment explains why no push
5. Verify no `git push origin main` follows

**Expected Result**: Step renamed, local commit preserved, push removed, explanatory comment present.

**Status**: ✅ PASS

**Actual Result**: 
- Step at L176 named "Commit auto-bump locally" ✓
- `git add cli/Cargo.toml` and `git commit` present ✓
- L183-184: NOTE comment explains branch protection prevents direct pushes ✓
- No `git push origin main` in the step ✓

---

### TC-006: `publish-engine` — `--no-git-push` Flag

**Description**: Verify the `publish-engine` job uses `--no-git-push` for `cargo ws publish`.

**Steps**:
1. Read the dry-run and actual publish steps in `publish-engine`
2. Verify both include `--no-git-push` flag
3. Verify a separate "Push tags only" step follows with `git push origin --tags`

**Expected Result**: Both `cargo ws publish` invocations have `--no-git-push`; explicit tag push step exists.

**Status**: ✅ PASS

**Actual Result**:
- Dry-run step (L255-264): `cargo ws publish --from-git --dry-run --yes --no-git-push --allow-branch main` ✓
- Actual publish step (L271-279): `cargo ws publish --from-git --yes --no-git-push --allow-branch main` ✓
- "Push tags only" step (L279-283): `git push origin --tags` with explanatory comment ✓

---

### TC-007: `publish-npm-core` — Local-Only Commit Pattern

**Description**: Verify the `publish-npm-core` job uses the same local-only commit pattern.

**Steps**:
1. Read the "Commit auto-bump locally" step in `publish-npm-core`
2. Verify same pattern as CLI (local commit, no push, NOTE comment)
3. Verify tag push step still pushes the core tag

**Expected Result**: Local-only commit pattern, core tag still pushed.

**Status**: ✅ PASS

**Actual Result**:
- Step at L353 named "Commit auto-bump locally" ✓
- `git add ui/packages/core/package.json` and `git commit` present ✓
- L360-361: NOTE comment present ✓
- L386: Tag push `git push origin "@wavecraft/core-v${{ steps.final.outputs.version }}"` ✓

---

### TC-008: `publish-npm-components` — Local-Only Commit Pattern

**Description**: Verify the `publish-npm-components` job uses the same local-only commit pattern.

**Steps**:
1. Read the "Commit auto-bump locally" step in `publish-npm-components`
2. Verify same pattern as CLI/core (local commit, no push, NOTE comment)
3. Verify tag push step still pushes the components tag

**Expected Result**: Local-only commit pattern, components tag still pushed.

**Status**: ✅ PASS

**Actual Result**:
- Step at L460 named "Commit auto-bump locally" ✓
- `git add ui/packages/components/package.json` and `git commit` present ✓
- L467-468: NOTE comment present ✓
- L493: Tag push `git push origin "@wavecraft/components-v${{ steps.final.outputs.version }}"` ✓

---

### TC-009: YAML Syntax Validity

**Description**: Verify the workflow YAML file is syntactically valid and parseable.

**Steps**:
1. Parse with `js-yaml` and verify all 5 jobs are present

**Expected Result**: YAML parses successfully with 5 jobs.

**Status**: ✅ PASS

**Actual Result**: `js-yaml` parsed successfully. Jobs found: `detect-changes`, `publish-cli`, `publish-engine`, `publish-npm-core`, `publish-npm-components`. All 5 expected jobs present.

---

### TC-010: CI Pipeline Guide — Auto-Bump Section Updated

**Description**: Verify the CI Pipeline Guide reflects tag-only publishing.

**Steps**:
1. Read "Auto-Bump Pattern" section in `docs/guides/ci-pipeline.md`
2. Verify step 3 says "Commit locally" (not "Commit + push")
3. Verify step 4 says "Push tag only"
4. Verify explanation of why local-only commit

**Expected Result**: Documentation describes the 4-step pattern: determine → bump → commit locally → push tag.

**Status**: ✅ PASS

**Actual Result**: Section at L386-399 correctly describes:
- Step 3: "Commit locally — Commit the version bump locally for publish tooling; version is **not** pushed to `main`" ✓
- Step 4: "Push tag only — After publishing, create a git tag and push it" ✓
- Explanation paragraph: "Branch protection rulesets on `main` prevent direct pushes" ✓

---

### TC-011: CI Pipeline Guide — Infinite Loop & Conflict Sections Updated

**Description**: Verify Infinite Loop Prevention and Git Conflict Prevention sections reflect new behavior.

**Steps**:
1. Read both sections in `docs/guides/ci-pipeline.md`
2. Verify infinite loop section says "no longer applies" + defense-in-depth
3. Verify conflict section says conflicts "no longer possible"

**Expected Result**: Both sections acknowledge the issue is resolved by tag-only publishing.

**Status**: ✅ PASS

**Actual Result**:
- L401-413: "Since auto-bump commits are no longer pushed to `main`, the infinite loop scenario...no longer applies. The `detect-changes` guard is kept as defense-in-depth" ✓
- L423-425: "Since no commits are pushed to `main`, parallel job conflicts for version bumps are no longer possible. Only tag pushes remain, and each job uses a unique tag prefix" ✓

---

### TC-012: Coding Standards — SDK Distribution Versioning Updated

**Description**: Verify the coding standards reflect tag-only publishing.

**Steps**:
1. Read "SDK Distribution Versioning" section in `docs/architecture/coding-standards.md`
2. Verify "What CI does" bullets mention local-only commits and tag-only pushes
3. Verify "Infinite loop prevention" bullet reflects new behavior

**Expected Result**: All three key bullets updated.

**Status**: ✅ PASS

**Actual Result** (L1008-1017):
- "Commits version bumps **locally only** (not pushed to `main`) — branch protection rulesets prevent direct pushes" ✓
- "Pushes **git tags only** for each published version (tags are not subject to branch protection)" ✓
- "The version in source files on `main` is the 'product baseline' — the registry holds the authoritative published version" ✓
- "Auto-bump commits are no longer pushed to `main`, so the infinite loop scenario does not arise" ✓

---

### TC-013: Post-Merge CD Pipeline Verification

**Description**: After merging to `main`, verify the CD pipeline runs without GH013 errors and publishes successfully.

**Steps**:
1. Merge the PR to `main`
2. Observe the Continuous Deploy workflow run in GitHub Actions
3. Verify all publish jobs complete (or skip) without `GH013` errors
4. Verify git tags are created on the remote for published packages
5. Verify published packages appear on crates.io/npm with correct versions

**Expected Result**: Full CD pipeline success.

**Status**: ⏸️ BLOCKED

**Actual Result**: Cannot be tested pre-merge. This test case requires the PR to be merged and the CD pipeline to actually run against the protected `main` branch.

**Notes**: This is the definitive verification of the fix (Step 13 in the implementation plan). All other test cases provide high confidence via static analysis, but only this test proves the fix works in production.

---

## Issues Found

No issues found.

## Testing Notes

- **All 4 publish jobs follow a consistent pattern**: local commit → publish → tag push. The implementation is symmetric and clean.
- **The `publish-engine` job differs slightly**: it uses `--no-git-push` flag on `cargo ws publish` and `git push origin --tags` (all tags at once) rather than individual tag pushes. This is correct because `cargo-workspaces` manages its own git commits/tags internally.
- **Explanatory comments** are added to all 4 commit steps explaining why no push occurs — good for maintainability.
- **Documentation is fully consistent** across both `ci-pipeline.md` and `coding-standards.md` — both reflect the new tag-only behavior.
- **YAML is syntactically valid** and parses with all 5 expected jobs.
- **The only remaining test** (TC-013) requires post-merge verification and is blocked by nature.

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] Issues documented for coder agent: None
- [x] Ready for release: YES (pending TC-013 post-merge verification)
