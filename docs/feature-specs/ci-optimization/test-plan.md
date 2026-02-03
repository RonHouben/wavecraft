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
| ⬜ NOT RUN | 7 |

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
- [ ] Performance testing complete
- [ ] Ready for QA: **PENDING PERFORMANCE TESTS**

---

## Performance Testing

### Overview

Compare CI pipeline execution times between the OLD setup (without test pre-compilation) and the NEW setup (with test pre-compilation) across different cache states to validate the expected time savings.

**Test Matrix:**

| Scenario | Cache State | OLD Setup | NEW Setup |
|----------|-------------|-----------|-----------|
| First PR run | Cold (empty) | PT-001 | PT-002 |
| Subsequent PR run | Warm (populated) | PT-003 | PT-004 |

---

## Cold Cache Tests (First PR Run Simulation)

### PT-001: OLD Setup - Cold Cache

**Description**: Measure CI execution times with the OLD setup (no test pre-compilation) starting from empty cache

**Preconditions**:
- Docker running with wavecraft-ci:latest image
- Clean state (no cached artifacts)

**Steps**:
1. Checkout main branch (or temporarily remove pre-compilation step):
   ```bash
   git stash  # Save current changes temporarily
   ```
2. Comment out the "Pre-compile test binaries" step in `.github/workflows/ci.yml`
3. Clear local act cache:
   ```bash
   rm -rf ~/.cache/act
   rm -rf /tmp/act-artifacts
   ```
4. Run full CI pipeline with timing:
   ```bash
   time act -W .github/workflows/ci.yml \
     --container-architecture linux/amd64 \
     -P ubuntu-latest=wavecraft-ci:latest \
     --pull=false \
     --artifact-server-path /tmp/act-artifacts 2>&1 | tee /tmp/ci-old-cold.log
   ```
5. Extract job timings:
   ```bash
   grep -E "Success.*\[.*s\]" /tmp/ci-old-cold.log | grep -E "(Prepare Engine|Check Engine|Test Engine)"
   ```

**Expected Result**: 
- prepare-engine: ~3-4 min (clippy only, no pre-compilation)
- test-engine: ~5-7 min (full compilation + test execution)
- All jobs pass

**Status**: ⬜ NOT RUN

**Actual Result**: 

| Job | Step | Duration |
|-----|------|----------|
| prepare-engine | Build with clippy | |
| prepare-engine | **Total** | |
| check-engine | Clippy | |
| check-engine | **Total** | |
| test-engine | Run tests (compile + run) | |
| test-engine | **Total** | |
| **Pipeline Total** | | |

**Notes**: 

---

### PT-002: NEW Setup - Cold Cache

**Description**: Measure CI execution times with the NEW setup (with test pre-compilation) starting from empty cache

**Preconditions**:
- Feature branch checked out with all changes
- Docker running with wavecraft-ci:latest image
- Clean state (no cached artifacts)

**Steps**:
1. Restore feature branch changes:
   ```bash
   git stash pop  # Restore new setup changes
   ```
2. Clear local act cache:
   ```bash
   rm -rf ~/.cache/act
   rm -rf /tmp/act-artifacts
   ```
3. Run full CI pipeline with timing:
   ```bash
   time act -W .github/workflows/ci.yml \
     --container-architecture linux/amd64 \
     -P ubuntu-latest=wavecraft-ci:latest \
     --pull=false \
     --artifact-server-path /tmp/act-artifacts 2>&1 | tee /tmp/ci-new-cold.log
   ```
4. Extract job timings:
   ```bash
   grep -E "Success.*\[.*s\]" /tmp/ci-new-cold.log | grep -E "(Prepare Engine|Check Engine|Test Engine)"
   ```

**Expected Result**: 
- prepare-engine: ~4-6 min (clippy + test pre-compilation)
- test-engine: ~1-2 min (uses cached test binaries, run only)
- All jobs pass

**Status**: ⬜ NOT RUN

**Actual Result**: 

| Job | Step | Duration |
|-----|------|----------|
| prepare-engine | Build with clippy | |
| prepare-engine | Pre-compile test binaries | |
| prepare-engine | **Total** | |
| check-engine | Clippy | |
| check-engine | **Total** | |
| test-engine | Run tests | |
| test-engine | **Total** | |
| **Pipeline Total** | | |

**Notes**: 

---

## Warm Cache Tests (Subsequent PR Run Simulation)

### PT-003: OLD Setup - Warm Cache

**Description**: Measure CI execution times with OLD setup when caches are already populated (simulates subsequent PR runs)

**Preconditions**:
- PT-001 completed (cache now populated)
- Old setup still configured (no pre-compilation step)

**Steps**:
1. Ensure old setup is configured (pre-compilation step removed/commented)
2. Do NOT clear cache (use cache from PT-001)
3. Run full CI pipeline with timing:
   ```bash
   time act -W .github/workflows/ci.yml \
     --container-architecture linux/amd64 \
     -P ubuntu-latest=wavecraft-ci:latest \
     --pull=false \
     --artifact-server-path /tmp/act-artifacts 2>&1 | tee /tmp/ci-old-warm.log
   ```
4. Extract job timings

**Expected Result**: 
- prepare-engine: faster (dependencies cached)
- test-engine: still needs recompilation (test profile not cached by check profile)
- Improvement over cold cache, but test-engine still slow

**Status**: ⬜ NOT RUN

**Actual Result**: 

| Job | Step | Duration |
|-----|------|----------|
| prepare-engine | Build with clippy | |
| prepare-engine | **Total** | |
| check-engine | Clippy | |
| check-engine | **Total** | |
| test-engine | Run tests (compile + run) | |
| test-engine | **Total** | |
| **Pipeline Total** | | |

**Notes**: 

---

### PT-004: NEW Setup - Warm Cache

**Description**: Measure CI execution times with NEW setup when caches are already populated (simulates subsequent PR runs)

**Preconditions**:
- PT-002 completed (cache now populated with test binaries)
- New setup configured (with pre-compilation step)

**Steps**:
1. Restore feature branch changes if needed
2. Do NOT clear cache (use cache from PT-002)
3. Run full CI pipeline with timing:
   ```bash
   time act -W .github/workflows/ci.yml \
     --container-architecture linux/amd64 \
     -P ubuntu-latest=wavecraft-ci:latest \
     --pull=false \
     --artifact-server-path /tmp/act-artifacts 2>&1 | tee /tmp/ci-new-warm.log
   ```
4. Extract job timings

**Expected Result**: 
- prepare-engine: fast (cache hit on clippy + test binaries)
- test-engine: very fast (cache hit, run only)
- Maximum benefit of the optimization

**Status**: ⬜ NOT RUN

**Actual Result**: 

| Job | Step | Duration |
|-----|------|----------|
| prepare-engine | Build with clippy | |
| prepare-engine | Pre-compile test binaries | |
| prepare-engine | **Total** | |
| check-engine | Clippy | |
| check-engine | **Total** | |
| test-engine | Run tests | |
| test-engine | **Total** | |
| **Pipeline Total** | | |

**Notes**: 

---

## Performance Analysis

### PT-005: Cold Cache Comparison

**Description**: Compare OLD vs NEW setup with cold cache (first PR run)

**Preconditions**:
- PT-001 (OLD cold) completed
- PT-002 (NEW cold) completed

**Expected Result**: 
- prepare-engine: NEW is ~1-2 min slower (pre-compilation overhead)
- test-engine: NEW is ~3-5 min faster (no recompilation)
- Net savings: ~2-3 minutes

**Status**: ⬜ NOT RUN

**Actual Result**: 

| Metric | OLD (Cold) | NEW (Cold) | Difference | % Change |
|--------|------------|------------|------------|----------|
| prepare-engine | | | | |
| test-engine | | | | |
| **Total** | | | | |

**Conclusion**: 

---

### PT-006: Warm Cache Comparison

**Description**: Compare OLD vs NEW setup with warm cache (subsequent PR runs)

**Preconditions**:
- PT-003 (OLD warm) completed
- PT-004 (NEW warm) completed

**Expected Result**: 
- prepare-engine: Similar (both benefit from dependency cache)
- test-engine: NEW significantly faster (test binaries cached)
- Maximum optimization benefit visible

**Status**: ⬜ NOT RUN

**Actual Result**: 

| Metric | OLD (Warm) | NEW (Warm) | Difference | % Change |
|--------|------------|------------|------------|----------|
| prepare-engine | | | | |
| test-engine | | | | |
| **Total** | | | | |

**Conclusion**: 

---

### PT-007: Overall Performance Summary

**Description**: Comprehensive summary of all performance test results

**Status**: ⬜ NOT RUN

**Summary Table**:

| Scenario | OLD Setup | NEW Setup | Time Saved | Notes |
|----------|-----------|-----------|------------|-------|
| Cold Cache (First PR) | | | | |
| Warm Cache (Subsequent PRs) | | | | |

**Key Findings**:
1. 
2. 
3. 

**Recommendation**: 

---

### Performance Testing Limitations

1. **Local `act` environment**: Container isolation means caches aren't shared between jobs the same way as GitHub Actions. Each job runs in a separate container.
2. **Cache key differences**: `act` uses local filesystem caching which may behave differently than GitHub's cache service
3. **Hardware variance**: Local machine performance may vary; focus on relative differences, not absolute times
4. **Network factors**: Crate downloads may affect timing; warm cache tests mitigate this
5. **Profile differences**: The key insight is that `cargo clippy` (check profile) doesn't cache test binaries (test profile), which is why pre-compilation helps

### Testing Order

For accurate results, execute tests in this order:

1. **PT-001** (OLD cold) → clears cache, runs old setup
2. **PT-003** (OLD warm) → immediately after, uses PT-001's cache
3. Clear cache, switch to new setup
4. **PT-002** (NEW cold) → clears cache, runs new setup
5. **PT-004** (NEW warm) → immediately after, uses PT-002's cache
6. **PT-005, PT-006, PT-007** → analysis
