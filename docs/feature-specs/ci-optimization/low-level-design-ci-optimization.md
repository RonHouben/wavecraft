# Low-Level Design: CI Pipeline Optimization

**Feature:** ci-optimization  
**Author:** Architect Agent  
**Date:** 2026-02-03  
**Status:** Draft

---

## 1. Overview

This document describes architectural changes to the CI pipeline to address two issues from the backlog:

1. **Cache inefficiency:** Test Engine job recompiles instead of using cached artifacts from Check Engine
2. **Artifact storage bloat:** 30-day retention on every main branch push exceeds GitHub's free tier limits

**Goals:**
- Reduce CI execution time by 3-5 minutes per PR
- Reduce artifact storage by ~80%
- Maintain fast feedback on lint/format failures

**Non-Goals:**
- Changing the release workflow signing/notarization flow
- Adding external artifact storage (R2/S3) — deferred unless needed

---

## 2. Current State Analysis

### 2.1 Pipeline Structure

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CURRENT CI PIPELINE                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  UI TRACK                       ENGINE TRACK                                │
│                                                                              │
│  check-ui ──► test-ui           prepare-engine ──► check-engine             │
│       │                                │                 │                   │
│       │                                │                 ▼                   │
│       │                                │           test-engine              │
│       │                                │                 │                   │
│       └────────────────┬───────────────┴─────────────────┘                  │
│                        ▼                                                     │
│                  build-plugin (main only)                                   │
│                        │                                                     │
│                        ▼                                                     │
│                  Upload artifacts (30-day retention)                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Problem 1: Cargo Profile Mismatch

| Job | Command | Cargo Profile | Artifacts |
|-----|---------|---------------|-----------|
| prepare-engine | `cargo clippy --workspace --all-targets --no-deps` | check | `.rlib`, metadata |
| check-engine | `cargo clippy --workspace --all-targets -- -D warnings` | check | Uses cached ✅ |
| test-engine | `cargo test --workspace` | test | **Recompiles** ❌ |

**Root Cause:** `cargo test` requires test binaries compiled with `--test` profile. The `clippy` check profile doesn't produce these, so `test-engine` rebuilds everything.

**Impact:** ~3-5 minutes wasted per PR.

### 2.3 Problem 2: Artifact Storage Accumulation

| Setting | Current | Impact |
|---------|---------|--------|
| Retention (main) | 30 days | ~1-2 GB/month accumulation |
| Retention (tags) | 30 days | Same as main (no differentiation) |
| Upload trigger | Every main push | ~2-3 pushes/day × 30-50 MB = 60-150 MB/day |

**Impact:** Hits 500 MB free tier limit, causing pipeline failures.

---

## 3. Proposed Changes

### 3.1 Change 1: Pre-compile Test Binaries in Prepare Job

**Approach:** Add `cargo test --no-run` to `prepare-engine` job after clippy.

```yaml
# prepare-engine job
- name: Build with clippy (compile for lint compatibility)
  run: cargo clippy --workspace --all-targets --no-deps
  working-directory: engine

# NEW: Pre-compile test binaries so test-engine doesn't rebuild
- name: Pre-compile test binaries
  run: cargo test --workspace --no-run
  working-directory: engine
```

**Why This Works:**
- `cargo test --no-run` compiles test binaries without running them
- These binaries are cached by `Swatinem/rust-cache@v2`
- `test-engine` uses the same cache key, finds pre-compiled binaries
- Test execution becomes near-instant (~10-30 seconds)

**Trade-off:** `prepare-engine` takes ~1-2 minutes longer, but saves 3-5 minutes in `test-engine`. Net savings: ~2-3 minutes per PR.

### 3.2 Change 2: Tiered Artifact Retention

**Approach:** Different retention based on trigger:

| Trigger | Artifacts | Retention |
|---------|-----------|-----------|
| PR | None | N/A |
| Main branch push | VST3 + CLAP | 7 days |
| Release tag (`v*`) | VST3 + CLAP | 90 days |

**Implementation:**

```yaml
build-plugin:
  name: Build Plugin
  runs-on: macos-latest
  needs: [test-ui, test-engine]
  # Build on main AND release tags
  if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')
  steps:
    # ... existing build steps ...

    - name: Upload VST3
      uses: actions/upload-artifact@v4
      with:
        name: wavecraft-vst3-adhoc-signed
        path: engine/target/bundled/wavecraft.vst3
        # 90 days for releases, 7 days for main
        retention-days: ${{ startsWith(github.ref, 'refs/tags/v') && 90 || 7 }}

    - name: Upload CLAP
      uses: actions/upload-artifact@v4
      with:
        name: wavecraft-clap-adhoc-signed
        path: engine/target/bundled/wavecraft.clap
        retention-days: ${{ startsWith(github.ref, 'refs/tags/v') && 90 || 7 }}
```

**Storage Calculation:**

| Scenario | Current | Proposed | Savings |
|----------|---------|----------|---------|
| Daily main pushes (3×) | 90-150 MB × 30 days = 2.7-4.5 GB | 90-150 MB × 7 days = 630 MB-1 GB | ~75% |
| Monthly releases (1×) | 50 MB × 30 days = 50 MB | 50 MB × 90 days = 50 MB | 0% |
| **Total monthly** | ~3-5 GB | ~700 MB - 1.1 GB | **~75-80%** |

### 3.3 Change 3: Add Caching to Release Workflow

**Current:** `release.yml` has no Rust caching, causing cold builds (~10-15 min).

**Proposed:**

```yaml
# release.yml - after checkout, before build
- name: Cache cargo
  uses: Swatinem/rust-cache@v2
  with:
    workspaces: engine
```

**Impact:** Release builds drop from ~15 min to ~5 min (after first cached run).

---

## 4. Architecture Diagrams

### 4.1 Optimized Pipeline Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        OPTIMIZED CI PIPELINE                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  UI TRACK                       ENGINE TRACK                                │
│                                                                              │
│  check-ui ──► test-ui           prepare-engine ──► check-engine             │
│       │                          (clippy +          (uses cache)            │
│       │                           test --no-run)          │                  │
│       │                                │                  ▼                  │
│       │                                │            test-engine             │
│       │                                │            (runs only,             │
│       │                                │             no rebuild)            │
│       │                                │                  │                  │
│       └────────────────┬───────────────┴──────────────────┘                 │
│                        ▼                                                     │
│                  build-plugin                                               │
│                  (main + tags only)                                         │
│                        │                                                     │
│                        ▼                                                     │
│                  Upload artifacts                                           │
│                  (7 days main / 90 days tags)                               │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Cache Flow

```
┌──────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  prepare-engine  │     │  check-engine   │     │  test-engine    │
│                  │     │                 │     │                 │
│  cargo clippy    │────►│  cargo clippy   │     │  cargo test     │
│  (check profile) │cache│  (check profile)│     │  (test profile) │
│                  │     │  CACHE HIT ✅   │     │                 │
│  cargo test      │     │                 │     │  CACHE HIT ✅   │
│  --no-run        │─────┼─────────────────┼────►│  (runs only)    │
│  (test profile)  │cache│                 │     │                 │
└──────────────────┘     └─────────────────┘     └─────────────────┘
```

---

## 5. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Cache key collision across profiles | Low | Medium | Swatinem/rust-cache handles profile separation; test both profiles in same job |
| Longer prepare-engine job | Certain | Low | Net time savings still positive (~2-3 min) |
| 7-day retention too short for debugging | Low | Low | Can manually re-run build-plugin if needed |
| Tag detection expression error | Low | High | Test with manual `workflow_dispatch` trigger first |

---

## 6. Testing Strategy

### 6.1 Validation Steps

1. **Cache effectiveness:** After implementing, check `test-engine` logs for "Compiling" lines. Should see only test execution, no compilation.

2. **Artifact retention:** Create test tag, verify 90-day retention shown in Actions UI.

3. **Storage monitoring:** Check repository settings → Actions → Artifacts after 1 week.

### 6.2 Rollback Plan

All changes are additive YAML modifications. Rollback by reverting the commit.

---

## 7. Implementation Checklist

| # | Task | File | Lines Changed |
|---|------|------|---------------|
| 1 | Add `cargo test --no-run` to prepare-engine | `.github/workflows/ci.yml` | +3 |
| 2 | Update `if` condition for build-plugin to include tags | `.github/workflows/ci.yml` | ~1 |
| 3 | Add dynamic retention-days to VST3 upload | `.github/workflows/ci.yml` | ~1 |
| 4 | Add dynamic retention-days to CLAP upload | `.github/workflows/ci.yml` | ~1 |
| 5 | Add rust-cache to release.yml | `.github/workflows/release.yml` | +4 |
| 6 | Update backlog to mark items addressed | `docs/backlog.md` | ~5 |

**Total:** ~15 lines changed across 3 files.

---

## 8. Future Considerations

### 8.1 Deferred: Combine check-engine + test-engine

If further optimization is needed, these jobs could be combined into one. Trade-off: loses parallel feedback on lint failures.

**Recommendation:** Defer unless pipeline time becomes critical.

### 8.2 Deferred: External Artifact Storage

If 500 MB limit is still hit after these changes:
- Cloudflare R2: 10 GB free, no egress fees
- AWS S3: Pay-as-you-go, ~$0.023/GB/month

**Recommendation:** Defer. Current changes should provide sufficient headroom.

### 8.3 Deferred: Artifact Cleanup Workflow

Scheduled workflow to delete old artifacts proactively:

```yaml
name: Cleanup Artifacts
on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
jobs:
  cleanup:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/github-script@v7
        with:
          script: |
            const artifacts = await github.rest.actions.listArtifactsForRepo({
              owner: context.repo.owner,
              repo: context.repo.repo,
            });
            // Delete artifacts older than 7 days...
```

**Recommendation:** Defer. GitHub's built-in retention should suffice.

---

## 9. References

- [GitHub Actions Artifact Storage Limits](https://docs.github.com/en/billing/managing-billing-for-github-actions/about-billing-for-github-actions)
- [Swatinem/rust-cache Documentation](https://github.com/Swatinem/rust-cache)
- [Cargo Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Current CI Workflow](.github/workflows/ci.yml)
- [Current Release Workflow](.github/workflows/release.yml)

---

## Changelog

| Date | Author | Change |
|------|--------|--------|
| 2026-02-03 | Architect | Initial draft |
