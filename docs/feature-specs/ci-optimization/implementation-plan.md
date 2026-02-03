# Implementation Plan: CI Pipeline Optimization

**Feature:** ci-optimization  
**Based on:** [Low-Level Design](low-level-design-ci-optimization.md)  
**Date:** 2026-02-03  
**Estimated Effort:** ~1 hour

---

## Overview

Implement three CI improvements to reduce execution time and artifact storage:
1. Pre-compile test binaries in prepare-engine job
2. Implement tiered artifact retention (7 days main / 90 days tags)
3. Add Rust caching to release workflow

---

## Requirements

- [ ] Test Engine job should not recompile artifacts already built by Prepare Engine
- [ ] Main branch artifacts retained for 7 days (down from 30)
- [ ] Release tag artifacts retained for 90 days
- [ ] Release workflow uses Rust cache for faster builds
- [ ] Build-plugin job triggers on both main branch and release tags

---

## Architecture Changes

| File | Change |
|------|--------|
| `.github/workflows/ci.yml` | Add test pre-compilation, update retention, update trigger condition |
| `.github/workflows/release.yml` | Add rust-cache action |
| `docs/backlog.md` | Mark CI items as addressed |

---

## Implementation Steps

### Phase 1: CI Workflow Cache Optimization

#### Step 1.1: Add Test Binary Pre-compilation
**File:** `.github/workflows/ci.yml`  
**Location:** `prepare-engine` job, after clippy step (~line 107)

**Action:** Add new step to pre-compile test binaries:
```yaml
- name: Pre-compile test binaries
  run: cargo test --workspace --no-run
  working-directory: engine
```

**Why:** This compiles test binaries with the test profile and caches them via rust-cache. The `test-engine` job will then use cached binaries instead of recompiling.

**Dependencies:** None  
**Risk:** Low — additive change, doesn't affect existing steps

---

### Phase 2: Artifact Retention Optimization

#### Step 2.1: Update Build-Plugin Trigger Condition
**File:** `.github/workflows/ci.yml`  
**Location:** `build-plugin` job `if` condition (~line 199)

**Action:** Change from:
```yaml
if: github.ref == 'refs/heads/main'
```
To:
```yaml
if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')
```

**Why:** Allows build-plugin to run on release tags, not just main branch, so we can apply different retention policies.

**Dependencies:** None  
**Risk:** Low — extends functionality, doesn't break existing behavior

---

#### Step 2.2: Update VST3 Artifact Retention
**File:** `.github/workflows/ci.yml`  
**Location:** VST3 upload step (~line 230)

**Action:** Change from:
```yaml
retention-days: 30
```
To:
```yaml
retention-days: ${{ startsWith(github.ref, 'refs/tags/v') && 90 || 7 }}
```

**Why:** 90 days for releases (long-term access), 7 days for main (short-term debugging).

**Dependencies:** Step 2.1 (tag trigger must be enabled first)  
**Risk:** Low — GitHub Actions expression syntax is well-documented

---

#### Step 2.3: Update CLAP Artifact Retention
**File:** `.github/workflows/ci.yml`  
**Location:** CLAP upload step (~line 237)

**Action:** Change from:
```yaml
retention-days: 30
```
To:
```yaml
retention-days: ${{ startsWith(github.ref, 'refs/tags/v') && 90 || 7 }}
```

**Why:** Same rationale as VST3.

**Dependencies:** Step 2.1  
**Risk:** Low

---

### Phase 3: Release Workflow Optimization

#### Step 3.1: Add Rust Cache to Release Workflow
**File:** `.github/workflows/release.yml`  
**Location:** After `Install Rust` step (~line 15)

**Action:** Add new step:
```yaml
- name: Cache cargo
  uses: Swatinem/rust-cache@v2
  with:
    workspaces: engine
```

**Why:** Release builds currently have no caching, causing cold builds (~15 min). This reduces to ~5 min after first cached run.

**Dependencies:** None  
**Risk:** Low — additive change

---

### Phase 4: Documentation Update

#### Step 4.1: Update Backlog
**File:** `docs/backlog.md`  
**Location:** CI/CD Optimization section

**Action:** Add note that items have been addressed:
- "CI pipeline cache optimization" → Mark as addressed with link to PR
- "GitHub artifacts storage alternative" → Mark as partially addressed (retention reduced, external storage deferred)

**Why:** Keep backlog accurate.

**Dependencies:** All previous steps  
**Risk:** None

---

## Testing Strategy

### Automated Validation
1. **Push to feature branch** — Verify CI runs successfully
2. **Check test-engine logs** — Should show minimal/no "Compiling" output
3. **Verify artifact retention** — Check Actions UI shows 7-day retention

### Manual Validation
1. **Create test tag** — `git tag v0.0.0-test && git push origin v0.0.0-test`
2. **Verify 90-day retention** — Check artifact in Actions UI
3. **Delete test tag** — `git push origin :refs/tags/v0.0.0-test`

### Metrics to Capture
| Metric | Before | After (Expected) |
|--------|--------|------------------|
| test-engine duration | ~5-7 min | ~1-2 min |
| Artifact retention (main) | 30 days | 7 days |
| Artifact retention (tags) | 30 days | 90 days |
| Release build duration | ~15 min | ~5 min (cached) |

---

## Risks & Mitigations

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Cache key mismatch between jobs | Low | Swatinem/rust-cache uses consistent keys per workflow |
| GitHub expression syntax error | Low | Test locally with `act` if available, or test in draft PR |
| 7-day retention too short | Low | Can manually trigger build-plugin if artifact needed |

---

## Success Criteria

- [ ] `test-engine` job completes in <2 minutes (down from 5-7)
- [ ] Main branch artifacts show 7-day retention in Actions UI
- [ ] Release tag artifacts show 90-day retention
- [ ] `release.yml` shows cache hits after first run
- [ ] No CI failures introduced by changes

---

## Rollback Plan

All changes are additive YAML modifications. If issues arise:
1. Revert the commit
2. Push to main
3. CI returns to previous behavior immediately

---

## File Change Summary

| File | Lines Added | Lines Modified | Lines Removed |
|------|-------------|----------------|---------------|
| `.github/workflows/ci.yml` | 3 | 3 | 0 |
| `.github/workflows/release.yml` | 4 | 0 | 0 |
| `docs/backlog.md` | 2 | 2 | 0 |
| **Total** | **9** | **5** | **0** |

---

## Changelog

| Date | Author | Change |
|------|--------|--------|
| 2026-02-03 | Planner | Initial implementation plan |
