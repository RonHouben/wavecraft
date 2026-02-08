# Test Plan: CD CLI Cascade Publish

## Overview
- **Feature**: CD CLI Cascade Publish + CI Auto-Bump
- **Spec Location**: `docs/feature-specs/cd-cli-cascade-publish/`
- **Date**: 2026-02-08
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 10 |
| ❌ FAIL | 2 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

> **Note**: TC-005 is marked FAIL for the missing upstream failure guards (Issue #2, Medium). TC-006 is marked FAIL for the `sed` pattern corrupting dependency versions (Issue #1, Critical).

## Prerequisites

- [x] `cargo xtask ci-check` passes (all lint + tests)

## Test Cases

### TC-001: YAML Syntax Validation

**Description**: Verify the modified `continuous-deploy.yml` is syntactically valid YAML.

**Steps**:
1. Parse the workflow file with a YAML parser
2. Verify no syntax errors

**Expected Result**: YAML parses without errors.

**Status**: ✅ PASS

**Actual Result**: `npx js-yaml .github/workflows/continuous-deploy.yml` parses without errors.

---

### TC-002: Workflow Structure — Job Count Preserved

**Description**: Verify the workflow still contains exactly 5 jobs with the correct names.

**Steps**:
1. Parse the YAML and list all job names
2. Compare against expected: `detect-changes`, `publish-cli`, `publish-engine`, `publish-npm-core`, `publish-npm-components`

**Expected Result**: Exactly 5 jobs with the documented names.

**Status**: ✅ PASS

**Actual Result**: 5 jobs found: `detect-changes`, `publish-cli`, `publish-engine`, `publish-npm-components`, `publish-npm-core`.

---

### TC-003: Loop Guard — `[auto-bump]` Skip Condition

**Description**: Verify the `detect-changes` job has the `[auto-bump]` skip guard as an `if` condition.

**Steps**:
1. Read the `detect-changes` job definition
2. Verify `if` condition contains `!contains(github.event.head_commit.message, '[auto-bump]')`

**Expected Result**: The `if` condition is present and correctly formed.

**Status**: ✅ PASS

**Actual Result**: `if: "!contains(github.event.head_commit.message, '[auto-bump]')"` — present and correctly formed.

---

### TC-004: Aggregate Output — `any_sdk_changed`

**Description**: Verify the `detect-changes` job exposes an `any_sdk_changed` output that OR-combines all four filter outputs.

**Steps**:
1. Verify `any_sdk_changed` is declared in the job `outputs`
2. Verify the `aggregate` step ORs cli, engine, npm-core, npm-components
3. Verify it writes to `$GITHUB_OUTPUT`

**Expected Result**: Output declared and computed correctly from all 4 filters.

**Status**: ✅ PASS

**Actual Result**: `any_sdk_changed` output declared, aggregate step ORs all 4 filter outputs (cli, engine, npm-core, npm-components) correctly.

---

### TC-005: CLI Cascade Trigger — `publish-cli` Dependencies

**Description**: Verify the `publish-cli` job depends on all publish jobs and triggers on `any_sdk_changed`.

**Steps**:
1. Verify `needs` includes: `detect-changes`, `publish-engine`, `publish-npm-core`, `publish-npm-components`
2. Verify `if` condition references `any_sdk_changed == 'true'`
3. Verify `always()` is used (to run even if upstream jobs were skipped)

**Expected Result**: CLI depends on all 4 jobs and triggers on aggregate flag.

**Status**: ❌ FAIL

**Actual Result**: CLI correctly depends on all 4 jobs and uses `any_sdk_changed`. However, the `if` condition is `needs.detect-changes.outputs.any_sdk_changed == 'true' && always()` which **lacks upstream failure guards**. The LLD specified `!cancelled()` and individual result checks like `(needs.publish-engine.result == 'success' || needs.publish-engine.result == 'skipped')`. Without these, if an upstream publish job fails, CLI will still attempt to publish. See **Issue #2**.

---

### TC-006: CLI Auto-Bump Logic

**Description**: Verify the `publish-cli` job has the three-step auto-bump pattern: determine → auto-bump → commit+push.

**Steps**:
1. Verify "Determine publish version" step compares local vs crates.io
2. Verify "Auto-bump patch version" step runs only when `version == ''`
3. Verify it uses `sed -i` to update `cli/Cargo.toml`
4. Verify "Commit and push auto-bump" step has `[auto-bump]` in commit message
5. Verify it runs `git pull --rebase origin main` before push
6. Verify "Set final version" step computes the version from either determine or bump outputs

**Expected Result**: Three-step auto-bump pattern with loop guard marker and rebase.

**Status**: ❌ FAIL

**Actual Result**: Three-step pattern is present (determine → auto-bump → commit+push → set final). The bump uses `sed -i` correctly on CI (Ubuntu). **However**, the `sed` pattern `s/^version = ".*"/version = "$NEW"/` matches ALL lines starting with `version = ` in `cli/Cargo.toml`, not just the `[package]` version. The file has 7 matching lines (package version + 6 dependency versions). This would corrupt dependency version declarations. See **Issue #1**.

---

### TC-007: CLI Skip Guards Removed

**Description**: Verify the `publish-cli` job no longer has `if: steps.version.outputs.skip != 'true'` guards on downstream steps.

**Steps**:
1. Search for `skip` references in the publish-cli job
2. Verify none of the build/publish/tag steps are conditionally skipped

**Expected Result**: No `skip` guards on dry-run, auth, publish, or tag steps.

**Status**: ✅ PASS

**Actual Result**: No steps contain `skip` guards. All downstream steps run unconditionally.

---

### TC-008: npm-core Auto-Bump Logic

**Description**: Verify the `publish-npm-core` job has the three-step auto-bump pattern.

**Steps**:
1. Verify "Determine publish version" step sets `version` output only when local > latest
2. Verify "Auto-bump patch version" step runs only when `version == ''`
3. Verify it uses `npm version "$NEW" --no-git-tag-version`
4. Verify "Commit and push auto-bump" step has `[auto-bump]` in commit message
5. Verify `git pull --rebase origin main` before push
6. Verify "Set final version" step concatenates version/bump outputs
7. Verify downstream steps have NO `skip` guards
8. Verify git tag step uses `${{ steps.final.outputs.version }}`

**Expected Result**: Auto-bump logic correct, skip guards removed, tag uses final version.

**Status**: ✅ PASS

**Actual Result**: Three-step auto-bump present. Uses `npm version --no-git-tag-version` correctly. Commit message includes `[auto-bump]`. `git pull --rebase origin main` before push. Tag uses `${{ steps.final.outputs.version }}`. No skip guards.

---

### TC-009: npm-components Auto-Bump Logic

**Description**: Verify the `publish-npm-components` job has the three-step auto-bump pattern (same as npm-core but for components).

**Steps**:
1. Verify same three-step pattern as TC-008 but for `@wavecraft/components`
2. Verify working directory is `ui/packages/components`
3. Verify git add targets `ui/packages/components/package.json`
4. Verify commit message references `@wavecraft/components`
5. Verify tag uses `@wavecraft/components-v${{ steps.final.outputs.version }}`

**Expected Result**: Identical pattern to npm-core, scoped to components package.

**Status**: ✅ PASS

**Actual Result**: Same three-step pattern. Working directory `ui/packages/components`. Git add targets correct file. Commit message references `@wavecraft/components`. Tag format `@wavecraft/components-v...`. No skip guards.

---

### TC-010: Git Tag Consistency

**Description**: Verify all three auto-bump jobs (CLI, npm-core, npm-components) create git tags using the computed `final` version, and use `git pull --rebase` before tagging.

**Steps**:
1. Check CLI tag step uses `${{ steps.final.outputs.version }}`
2. Check npm-core tag step uses `${{ steps.final.outputs.version }}`
3. Check npm-components tag step uses `${{ steps.final.outputs.version }}`
4. Verify all tag steps call `git pull --rebase origin main` before tagging

**Expected Result**: All tags reference the final computed version and rebase first.

**Status**: ✅ PASS

**Actual Result**: All 3 tag steps use `${{ steps.final.outputs.version }}` and `git pull --rebase origin main` before tagging.

---

### TC-011: Coding Standards Documentation

**Description**: Verify the SDK Distribution Versioning section was added correctly to coding standards.

**Steps**:
1. Verify "SDK Distribution Versioning (CI Auto-Bump)" subsection exists
2. Verify it documents the two version domains (Product vs Distribution)
3. Verify it documents the `[auto-bump]` infinite loop prevention
4. Verify it documents what developers should/should not do
5. Verify it documents what CI does
6. Verify it is placed after the existing Versioning section and before Comments and Documentation

**Expected Result**: Complete documentation section covering all aspects of the new workflow.

**Status**: ✅ PASS

**Actual Result**: Section "SDK Distribution Versioning (CI Auto-Bump)" added at line 982 of coding-standards.md. Contains: two version domains table, how it works (6 steps), what developers should do, what CI does, infinite loop prevention. Correctly placed between "VersionBadge" paragraph and "Comments and Documentation" section.

---

### TC-012: Automated CI Checks Pass

**Description**: Verify `cargo xtask ci-check` passes with all changes.

**Steps**:
1. Run `cargo xtask ci-check`
2. Verify lint (ESLint, Prettier, cargo fmt, clippy) passes
3. Verify automated tests (Engine + UI) pass

**Expected Result**: All checks pass cleanly.

**Status**: ✅ PASS

**Actual Result**: `cargo xtask ci-check` completed in 19.6s. Lint (cargo fmt, clippy, ESLint, Prettier) all passed. Tests (165 Rust tests, 28 UI tests) all passed.

---

## Issues Found

### Issue #1: `sed` Pattern Corrupts Dependency Versions in CLI Cargo.toml

- **Severity**: Critical
- **Test Case**: TC-006
- **Description**: The CLI auto-bump step uses `sed -i "s/^version = \".*\"/version = \"$NEW\"/" cli/Cargo.toml` to bump the package version. This regex matches **all** lines starting with `version = "` — not just the `[package]` version.
- **Expected**: Only the `[package]` version on line 3 is updated.
- **Actual**: `cli/Cargo.toml` has **7 lines** matching `^version = "`:
  - Line 3: `version = "0.8.5"` (package version — should be bumped)
  - Line 35: `version = "0.8"` (toml dependency — **should NOT change**)
  - Line 38: `version = "1"` (tokio dependency — **should NOT change**)
  - Line 44: `version = "0.7.4"` (wavecraft-protocol — **should NOT change**)
  - Line 48: `version = "0.7.4"` (wavecraft-bridge — **should NOT change**)
  - Line 52: `version = "0.7.4"` (wavecraft-metering — **should NOT change**)
  - Line 56: `version = "0.7.4"` (wavecraft-dev-server — **should NOT change**)
- **Steps to Reproduce**:
  1. Run: `grep -n '^version = ' cli/Cargo.toml`
  2. Observe 7 matching lines
  3. Simulate: `sed "s/^version = \".*\"/version = \"0.8.6\"/" cli/Cargo.toml` (dry run)
  4. Observe all 7 lines changed to `version = "0.8.6"`
- **Suggested Fix**: Target only the `[package]` section version. Options:
  - **Option A (simplest)**: Use `sed` with line-number targeting: `sed -i '3s/^version = ".*"/version = "'$NEW'"/' cli/Cargo.toml` — fragile if line numbers change.
  - **Option B (recommended)**: Use a two-pass approach — `sed '/^\[package\]/,/^\[/{s/^version = ".*"/version = "'"$NEW"'"/}' cli/Cargo.toml` to scope the replacement to the `[package]` section only.
  - **Option C**: Use `cargo set-version` from `cargo-edit` (most robust but requires installing the tool in CI).

---

### Issue #2: Missing Upstream Failure Guards on `publish-cli` Job

- **Severity**: Medium
- **Test Case**: TC-005
- **Description**: The `publish-cli` job's `if` condition is:
  ```yaml
  if: needs.detect-changes.outputs.any_sdk_changed == 'true' && always()
  ```
  The LLD specification (lines 304-316 of `low-level-design-cd-cli-cascade-publish.md`) requires defensive upstream failure guards:
  ```yaml
  if: |
    !cancelled() &&
    needs.detect-changes.outputs.any_sdk_changed == 'true' &&
    (needs.publish-engine.result == 'success' || needs.publish-engine.result == 'skipped') &&
    (needs.publish-npm-core.result == 'success' || needs.publish-npm-core.result == 'skipped') &&
    (needs.publish-npm-components.result == 'success' || needs.publish-npm-components.result == 'skipped')
  ```
- **Expected**: `publish-cli` should only run if upstream jobs **succeeded or were skipped** (not if they failed or were cancelled).
- **Actual**: With `always()`, `publish-cli` will run even if an upstream publish job **fails**. This could result in publishing a CLI that references engine crate versions not yet on crates.io.
- **Steps to Reproduce**:
  1. Open `.github/workflows/continuous-deploy.yml`
  2. Find `publish-cli` job's `if` condition (line ~67)
  3. Compare to LLD specification (lines 304-316)
- **Suggested Fix**: Replace the `if` condition with the LLD-specified defensive pattern that checks `!cancelled()` and individual upstream job results.

## Testing Notes

This feature modifies a GitHub Actions workflow file. The core behavior (auto-bumping, publishing, infinite loop prevention) can only be fully verified by running the CD pipeline in GitHub. The test cases here focus on:

1. **Static validation**: YAML syntax, workflow structure, correct conditions/outputs
2. **Logic verification**: Auto-bump patterns, skip guard removal, cascade dependencies
3. **Documentation**: Coding standards completeness
4. **CI health**: Existing tests still pass

Post-merge verification scenarios (engine-only change triggers CLI, auto-bump commit skips pipeline, etc.) are tracked in the implementation-progress.md.

## Sign-off

- [ ] All critical tests pass — **NO** (Issue #1 is Critical)
- [x] All high-priority tests pass
- [x] Issues documented for coder agent
- [ ] Ready for release: **NO** — Critical Issue #1 must be fixed before merge
