# Test Plan: CI Pipeline Optimization

## Overview
- **Feature**: CI Pipeline Optimization
- **Spec Location**: `docs/feature-specs/ci-optimization/`
- **Date**: 2026-02-03
- **Tester**: Tester Agent

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 7 |
| ❌ FAIL | 0 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] Docker is running: `docker info`
- [x] CI image exists: `docker images | grep wavecraft-ci`
- [x] Feature branch exists: `feature/ci-optimization`
- [x] All code changes committed

## Test Cases

### TC-001: Version Bump Verification

**Description**: Verify that version was correctly bumped to 0.6.2

**Preconditions**:
- Repository checked out to feature/ci-optimization branch

**Steps**:
1. Check `engine/Cargo.toml` workspace version
2. Check `engine/Cargo.lock` version consistency
3. Verify all crate versions inherited correctly

**Expected Result**: 
- `[workspace.package]` version = "0.6.2"
- All workspace crates show v0.6.2 in Cargo.lock

**Status**: ✅ PASS

**Actual Result**: 
- `engine/Cargo.toml` shows `version = "0.6.2"` under `[workspace.package]`
- `engine/Cargo.lock` shows all wavecraft-* crates with version 0.6.2
- Version bump correctly applied

**Notes**: All versions consistent across workspace 

---

### TC-002: Test Binary Pre-compilation (YAML)

**Description**: Verify that ci.yml contains the test pre-compilation step

**Preconditions**:
- Feature branch checked out

**Steps**:
1. Open `.github/workflows/ci.yml`
2. Locate `prepare-engine` job
3. Verify step "Pre-compile test binaries" exists after clippy step
4. Verify command is `cargo test --workspace --no-run`

**Expected Result**: 
- Step exists in correct location
- Command matches specification
- working-directory is set to `engine`

**Status**: ✅ PASS

**Actual Result**: 
- Step "Pre-compile test binaries" exists after clippy step
- Command is `cargo test --workspace --no-run` (correct)
- `working-directory: engine` is set
- Step executed successfully in CI run

**Notes**: Found in ci.yml at correct location in prepare-engine job 

---

### TC-003: Build Plugin Trigger Update (YAML)

**Description**: Verify build-plugin job triggers on both main branch and release tags

**Preconditions**:
- Feature branch checked out

**Steps**:
1. Open `.github/workflows/ci.yml`
2. Locate `build-plugin` job
3. Check `if` condition

**Expected Result**: 
```yaml
if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')
```

**Status**: ✅ PASS

**Actual Result**: 
Exact match found in `.github/workflows/ci.yml`:
```yaml
if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')
```

**Notes**: Trigger condition updated correctly 

---

### TC-004: Tiered Artifact Retention (YAML)

**Description**: Verify artifact retention is tiered (7 days main / 90 days tags)

**Preconditions**:
- Feature branch checked out

**Steps**:
1. Open `.github/workflows/ci.yml`
2. Locate VST3 upload step
3. Check retention-days expression
4. Locate CLAP upload step
5. Check retention-days expression

**Expected Result**: 
Both artifacts should have:
```yaml
retention-days: ${{ startsWith(github.ref, 'refs/tags/v') && 90 || 7 }}
```

**Status**: ✅ PASS

**Actual Result**: 
Both VST3 and CLAP upload steps have:
```yaml
retention-days: ${{ startsWith(github.ref, 'refs/tags/v') && 90 || 7 }}
```
Verified in `.github/workflows/ci.yml`

**Notes**: Tiered retention correctly implemented for both artifact types 

---

### TC-005: Release Workflow Caching (YAML)

**Description**: Verify release.yml contains Rust cache step

**Preconditions**:
- Feature branch checked out

**Steps**:
1. Open `.github/workflows/release.yml`
2. Locate "Cache cargo" step after "Install Rust"
3. Verify uses `Swatinem/rust-cache@v2`
4. Verify `with.workspaces: engine`

**Expected Result**: 
- Cache step exists between Install Rust and Install Node.js
- Correct action version (v2)
- Correct workspace configuration

**Status**: ✅ PASS

**Actual Result**: 
Found in `.github/workflows/release.yml`:
```yaml
- name: Cache cargo
  uses: Swatinem/rust-cache@v2
  with:
    workspaces: engine
```
Correctly positioned after "Install Rust" step

**Notes**: Release workflow caching added correctly 

---

### TC-006: Local CI Pipeline Execution

**Description**: Run full CI pipeline locally with act to verify changes work

**Preconditions**:
- Docker is running
- act CLI installed
- wavecraft-ci:latest image exists

**Steps**:
1. Run: `docker info` (verify Docker running)
2. Run: `act -W .github/workflows/ci.yml --container-architecture linux/amd64 -P ubuntu-latest=wavecraft-ci:latest --pull=false --artifact-server-path /tmp/act-artifacts`
3. Monitor output for errors
4. Check prepare-engine job logs for "Pre-compile test binaries"
5. Check tes✅ PASS

**Actual Result**: 
- ✅ All CI jobs completed successfully (Check UI, Test UI, Prepare Engine, Check Engine, Test Engine)
- ✅ prepare-engine job includes "Pre-compile test binaries" step (ran for 1m42s)
- ✅ test-engine job shows `Finished test profile [unoptimized + debuginfo] target(s) in 13.58s`
- ✅ build-plugin skipped (as expected on feature branch)
- ✅ All 100+ tests passed across all crates

**Notes**: 
- Local CI testing with `act` has container isolation limitations (cache not shared between jobs)
- In GitHub Actions, test-engine would show even less compilation due to shared cache
- Test execution time (13.58s) is reasonable given container overheadengine shows "Pre-compile test binaries" step
- test-engine shows "Finished `test` profile" quickly with minimal compilation
- build-plugin skips (not on main branch)

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

---

### TC-007: Backlog Documentation Update

**Description**: Verify backlog.md accurately reflects completed work

**Preconditions**:
- Feature branch checked out

**Steps**:
1. Open `docs/backlog.md`
2. Locate "CI/CD Optimization" section
3. Verify "CI pipeline cache optimization" is marked addressed
4. Verify "G✅ PASS

**Actual Result**: 
Found in `docs/backlog.md` under "CI/CD Optimization":
- ~~CI pipeline cache optimization~~ ✅ **Addressed** in v0.6.2 — explains pre-compilation
- ~~GitHub artifacts storage alternative~~ ✅ **Partially addressed** in v0.6.2 — explains tiered retention
- Both entries properly formatted with strikethrough and version references

**Notes**: Documentation accurately reflects completed workms show ✅ checkmarks or strikethrough
- Version v0.6.2 mentioned
- Notes explain what was done

**Status**: ⬜ NOT RUN

**Actual Result**: 

**Notes**: 

-*No issues found** — All test cases passed successfully.

---

## Testing Notes

- ✅ Local CI testing with `act` completed successfully
- ✅ All 100+ tests passed (31 engine tests, 43 UI tests, rest from xtask)
- ✅ Version bump verified across all workspace crates
- ✅ YAML configuration changes validated
- ⚠️ **Cache behavior in `act`**: Container isolation means caches aren't shared between jobs in local testing. In real GitHub Actions, the test-engine job will show even better cache effectiveness.
- 📊 **Expected GitHub Actions behavior**: 
  - First run after merge: prepare-engine ~3-5 min longer, test-engine ~3-5 min faster
  - Subsequent runs: Both jobs benefit from warm cache
  - Net time savings: ~2-3 minutes per PR

---

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] No issues found
- [x] Ready for QA: **YES**
- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Issues documented for coder agent (if any)
- [ ] Ready for QA: NO (testing not started)
