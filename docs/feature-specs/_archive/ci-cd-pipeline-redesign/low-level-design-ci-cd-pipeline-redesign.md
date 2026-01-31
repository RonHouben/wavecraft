# Low-Level Design: CI/CD Pipeline Redesign

**Feature:** CI/CD Pipeline Optimization  
**Author:** Architect  
**Date:** 2026-01-31  
**Status:** Draft

---

## 1. Overview

This design transforms the current monolithic CI pipeline into a staged, fail-fast architecture that:

1. Enables PR testing (critical gap fix)
2. Provides fast feedback (<2 min for lint/typecheck)
3. Reduces CI costs by ~88% for PRs through ubuntu usage
4. Eliminates duplicate workflows (`lint.yml` → absorbed into `ci.yml`)
5. Adds concurrency control to prevent wasted compute

### Key Architectural Decisions

| Decision | Rationale |
|----------|-----------|
| 3-stage pipeline | Fail-fast prevents 15 min wait for lint errors |
| Ubuntu for non-build jobs | 10x cheaper than macOS runners |
| Artifact sharing for UI build | Engine lint requires compiled UI; avoids rebuilding |
| Type-check produces artifacts | Fastest job builds UI, shares with dependents |
| Build only on `main` | Plugin bundle (~8 min) unnecessary for PR validation |

---

## 2. Pipeline Architecture

### 2.1 Stage Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CI PIPELINE STAGES                                 │
└─────────────────────────────────────────────────────────────────────────────┘

                         ┌─────────────────────┐
                         │      Trigger        │
                         │  (PR or push:main)  │
                         └──────────┬──────────┘
                                    │
            ┌───────────────────────┼───────────────────────┐
            │                       │                       │
            ▼                       ▼                       ▼
   ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
   │   lint-ui       │    │  typecheck-ui   │    │  lint-engine    │
   │   (ubuntu)      │    │   (ubuntu)      │    │   (ubuntu)      │
   │   ~30 sec       │    │   ~30 sec       │    │   ~90 sec       │
   │                 │    │  uploads: ui/   │    │  needs:         │
   │                 │    │  dist artifact  │    │  typecheck-ui   │
   └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
            │                      │                      │
            │         ┌────────────┴────────────┐         │
            │         │    ui-dist artifact     │─────────┘
            │         └─────────────────────────┘
            │                      │
            └──────────────────────┼──────────────────────┘
                                   │
                    ══════════════════════════════
                    ║   STAGE 1 GATE (all pass)  ║
                    ══════════════════════════════
                                   │
            ┌──────────────────────┼──────────────────────┐
            │                      │                      │
            ▼                      ▼                      │
   ┌─────────────────┐    ┌─────────────────┐            │
   │   test-ui       │    │  test-engine    │            │
   │   (ubuntu)      │    │   (ubuntu)      │            │
   │   ~60 sec       │    │   ~90 sec       │            │
   │                 │    │  needs:         │            │
   │                 │    │  typecheck-ui   │            │
   └────────┬────────┘    └────────┬────────┘            │
            │                      │                      │
            └──────────────────────┼──────────────────────┘
                                   │
                    ══════════════════════════════
                    ║   STAGE 2 GATE (all pass)  ║
                    ══════════════════════════════
                                   │
                                   │ if: github.ref == 'refs/heads/main'
                                   │
                                   ▼
                         ┌─────────────────┐
                         │  build-plugin   │
                         │   (macos)       │
                         │   ~8 min        │
                         │  uploads: VST3, │
                         │  CLAP artifacts │
                         └─────────────────┘
```

### 2.2 Job Dependency Graph

```
                   ┌─────────────┐
                   │ typecheck-ui│ ─────────────────────────┐
                   └──────┬──────┘                          │
                          │ uploads ui-dist                 │
                          │                                 │
            ┌─────────────┼─────────────┐                   │
            │             │             │                   │
            ▼             ▼             ▼                   │
      ┌───────────┐ ┌───────────┐ ┌───────────┐            │
      │  lint-ui  │ │lint-engine│ │ test-ui   │            │
      └─────┬─────┘ └─────┬─────┘ └─────┬─────┘            │
            │             │             │                   │
            │             │ downloads   │                   │
            │             │ ui-dist     │                   │
            │             │             │                   │
            └─────────────┼─────────────┘                   │
                          │                                 │
                          ▼                                 ▼
                    ┌───────────┐                    ┌───────────┐
                    │test-engine│                    │           │
                    └─────┬─────┘                    │           │
                          │                         │           │
                          │ downloads ui-dist       │           │
                          │                         │           │
                          └─────────────────────────┘           │
                                       │                        │
                                       ▼                        │
                          (all Stage 1 + Stage 2 complete)      │
                                       │                        │
                                       ▼                        │
                              ┌─────────────────┐               │
                              │  build-plugin   │◄──────────────┘
                              │   (main only)   │  downloads ui-dist
                              └─────────────────┘
```

### 2.3 Runner Assignment

| Job | Runner | Rationale |
|-----|--------|-----------|
| `typecheck-ui` | `ubuntu-latest` | Node.js only, no platform deps |
| `lint-ui` | `ubuntu-latest` | Node.js only, ESLint + Prettier |
| `lint-engine` | `ubuntu-latest` | Rust fmt/clippy work cross-platform |
| `test-ui` | `ubuntu-latest` | happy-dom (no real browser needed) |
| `test-engine` | `ubuntu-latest` | Rust unit tests are platform-agnostic |
| `build-plugin` | `macos-latest` | VST3/CLAP bundle creation requires macOS |

**Cost Impact:**

| Job | macOS Cost (per min) | Ubuntu Cost (per min) | Minutes | Savings |
|-----|---------------------|----------------------|---------|---------|
| typecheck-ui | $0.08 | $0.008 | ~0.5 | 90% |
| lint-ui | $0.08 | $0.008 | ~0.5 | 90% |
| lint-engine | $0.08 | $0.008 | ~1.5 | 90% |
| test-ui | $0.08 | $0.008 | ~1.0 | 90% |
| test-engine | $0.08 | $0.008 | ~1.5 | 90% |
| build-plugin | $0.08 | N/A | ~8.0 | 0% |

---

## 3. Workflow File Design

### 3.1 Complete `ci.yml` Structure

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  # ═══════════════════════════════════════════════════════════════════════════
  # STAGE 1: Fast Feedback (<2 minutes)
  # ═══════════════════════════════════════════════════════════════════════════

  typecheck-ui:
    name: TypeCheck UI
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: ui/package-lock.json

      - name: Install dependencies
        run: npm ci
        working-directory: ui

      - name: Type-check
        run: npm run typecheck
        working-directory: ui

      - name: Build UI
        run: npm run build
        working-directory: ui

      - name: Upload UI dist
        uses: actions/upload-artifact@v4
        with:
          name: ui-dist
          path: ui/dist
          retention-days: 1

  lint-ui:
    name: Lint UI
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: ui/package-lock.json

      - name: Install dependencies
        run: npm ci
        working-directory: ui

      - name: ESLint
        run: npm run lint
        working-directory: ui

      - name: Prettier
        run: npm run format:check
        working-directory: ui

  lint-engine:
    name: Lint Engine
    runs-on: ubuntu-latest
    needs: [typecheck-ui]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Download UI dist
        uses: actions/download-artifact@v4
        with:
          name: ui-dist
          path: ui/dist

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: engine

      - name: Check formatting
        run: cargo fmt --check
        working-directory: engine

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings
        working-directory: engine

  # ═══════════════════════════════════════════════════════════════════════════
  # STAGE 2: Tests (runs after Stage 1 passes)
  # ═══════════════════════════════════════════════════════════════════════════

  test-ui:
    name: Test UI
    runs-on: ubuntu-latest
    needs: [typecheck-ui, lint-ui, lint-engine]
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: ui/package-lock.json

      - name: Install dependencies
        run: npm ci
        working-directory: ui

      - name: Run tests
        run: npm test
        working-directory: ui

  test-engine:
    name: Test Engine
    runs-on: ubuntu-latest
    needs: [typecheck-ui, lint-ui, lint-engine]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Download UI dist
        uses: actions/download-artifact@v4
        with:
          name: ui-dist
          path: ui/dist

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: engine

      - name: Run tests
        run: cargo test --workspace
        working-directory: engine

  # ═══════════════════════════════════════════════════════════════════════════
  # STAGE 3: Build (main branch only, after tests pass)
  # ═══════════════════════════════════════════════════════════════════════════

  build-plugin:
    name: Build Plugin
    runs-on: macos-latest
    needs: [test-ui, test-engine]
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Download UI dist
        uses: actions/download-artifact@v4
        with:
          name: ui-dist
          path: ui/dist

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: engine

      - name: Build plugin bundles
        run: cargo xtask bundle --features webview_editor
        working-directory: engine

      - name: Ad-hoc sign (macOS)
        run: cargo xtask sign --adhoc
        working-directory: engine

      - name: Verify signatures
        run: cargo xtask sign --verify --verbose
        working-directory: engine

      - name: Upload VST3
        uses: actions/upload-artifact@v4
        with:
          name: vstkit-vst3-adhoc-signed
          path: engine/target/bundled/vstkit.vst3
          retention-days: 30

      - name: Upload CLAP
        uses: actions/upload-artifact@v4
        with:
          name: vstkit-clap-adhoc-signed
          path: engine/target/bundled/vstkit.clap
          retention-days: 30
```

### 3.2 Trigger Configuration Explained

```yaml
on:
  push:
    branches: [main]      # Full pipeline: lint → test → build
  pull_request:
    branches: [main]      # Lint + test only (build skipped via `if:`)
```

The `build-plugin` job has `if: github.ref == 'refs/heads/main'` which evaluates to `false` on PRs, causing it to skip.

### 3.3 Concurrency Control Explained

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

- **Group format:** `CI-refs/heads/feature-branch` or `CI-refs/pull/123/merge`
- **Behavior:** New commit to same branch cancels in-progress run
- **Isolation:** Different branches run independently

---

## 4. Artifact Sharing Strategy

### 4.1 Problem Statement

The Engine (Rust) embeds the UI (React) at compile time. Specifically:
- `cargo xtask bundle` requires `ui/dist/` to exist
- `cargo clippy` compiles the full workspace, which includes the embed step
- Without UI artifacts, Engine lint fails

### 4.2 Solution: Upload/Download Artifacts

```
┌───────────────┐          ┌──────────────────────────────────┐
│ typecheck-ui  │─────────►│        ui-dist artifact          │
│   (builds UI) │  upload  │  contents: ui/dist/**            │
└───────────────┘          │  retention: 1 day                │
                           └──────────────────────────────────┘
                                          │
            ┌─────────────────────────────┼─────────────────────────────┐
            │ download                    │ download                    │ download
            ▼                             ▼                             ▼
   ┌─────────────────┐           ┌─────────────────┐           ┌─────────────────┐
   │  lint-engine    │           │  test-engine    │           │  build-plugin   │
   │  (needs clippy) │           │  (needs build)  │           │  (needs bundle) │
   └─────────────────┘           └─────────────────┘           └─────────────────┘
```

### 4.3 Upload Configuration

```yaml
- name: Upload UI dist
  uses: actions/upload-artifact@v4
  with:
    name: ui-dist
    path: ui/dist
    retention-days: 1           # Short retention, only needed for same workflow run
```

### 4.4 Download Configuration

```yaml
- name: Download UI dist
  uses: actions/download-artifact@v4
  with:
    name: ui-dist
    path: ui/dist               # Restores to original location
```

### 4.5 Cache vs Artifact Trade-offs

| Aspect | Cache | Artifact |
|--------|-------|----------|
| **Purpose** | Speed up repeated builds | Share files between jobs |
| **Persistence** | Across workflow runs | Within single workflow run |
| **Key mechanism** | Content hash (Cargo.lock, package-lock.json) | Explicit upload/download |
| **Use for UI dist** | ❌ Not appropriate | ✅ Correct choice |
| **Use for node_modules** | ✅ Via `setup-node` cache | ❌ Too large |
| **Use for cargo target** | ✅ Via `Swatinem/rust-cache` | ❌ Too large |

**Decision:** Use artifacts for UI dist (small, workflow-scoped), cache for dependencies (large, cross-workflow).

---

## 5. Branch Protection Configuration

### 5.1 Required Status Checks

Configure these checks as required for PR merges to `main`:

| Check Name | Purpose | Blocks Merge If |
|------------|---------|-----------------|
| `TypeCheck UI` | Type correctness | Type errors |
| `Lint UI` | Code style (UI) | ESLint/Prettier failures |
| `Lint Engine` | Code style (Engine) | fmt/clippy failures |
| `Test UI` | UI unit tests | Test failures |
| `Test Engine` | Engine unit tests | Test failures |

**Note:** `Build Plugin` is NOT a required check because it only runs on `main`.

### 5.2 GitHub Settings Path

```
Repository → Settings → Branches → Branch protection rules → main
```

### 5.3 Recommended Configuration

```
☑ Require a pull request before merging
  ☑ Require approvals: 1 (optional, based on team size)
  ☐ Dismiss stale pull request approvals when new commits are pushed

☑ Require status checks to pass before merging
  ☑ Require branches to be up to date before merging
  Required checks:
    - TypeCheck UI
    - Lint UI
    - Lint Engine
    - Test UI
    - Test Engine

☐ Require conversation resolution before merging (optional)

☑ Require linear history (recommended for clean git log)

☐ Include administrators (disable to allow force-push for emergencies)
```

---

## 6. Migration Plan

### 6.1 Step-by-Step Migration

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           MIGRATION SEQUENCE                                 │
└─────────────────────────────────────────────────────────────────────────────┘

  Step 1                    Step 2                    Step 3
  ──────                    ──────                    ──────
  Create new ci.yml         Test & validate           Cleanup
  (keep lint.yml)           (both active)             (delete lint.yml)

  ┌─────────────┐           ┌─────────────┐           ┌─────────────┐
  │ ci.yml (new)│           │ ci.yml (new)│           │ ci.yml      │
  │ lint.yml    │    →      │ lint.yml    │     →     │             │
  │ release.yml │           │ release.yml │           │ release.yml │
  └─────────────┘           └─────────────┘           └─────────────┘
                                   │
                            Validate:
                            ✓ PR triggers work
                            ✓ All jobs pass
                            ✓ Artifacts shared
                            ✓ Build skips on PR
```

### 6.2 Detailed Steps

#### Step 1: Deploy New `ci.yml` (Keep Both Workflows)

1. Replace `.github/workflows/ci.yml` with the new staged pipeline
2. Keep `.github/workflows/lint.yml` temporarily (runs in parallel)
3. Push to a feature branch, open PR to test

**Expected behavior:**
- Both `CI` and `Lint` workflows run
- `CI` has new staged structure
- No branch protection changes yet

#### Step 2: Validate New Pipeline

Run these test scenarios:

| Scenario | Expected Result |
|----------|-----------------|
| PR with clean code | All Stage 1 + Stage 2 jobs green, no build |
| PR with lint error | Stage 1 fails, Stage 2 skipped |
| PR with test failure | Stage 1 green, Stage 2 red |
| Push to main | Full pipeline including build-plugin |
| Rapid pushes to PR | In-progress run cancelled |

**Validation checklist:**
- [ ] `typecheck-ui` uploads artifact
- [ ] `lint-engine` downloads and uses artifact
- [ ] `test-engine` downloads and uses artifact
- [ ] `build-plugin` has `if:` condition (skipped on PR)
- [ ] Concurrency cancels old runs

#### Step 3: Enable Branch Protection

1. Go to Repository → Settings → Branches → Add rule
2. Branch name pattern: `main`
3. Enable required status checks (see Section 5.3)
4. Save rule

#### Step 4: Delete `lint.yml`

After 2-3 successful PRs through the new pipeline:

```bash
git rm .github/workflows/lint.yml
git commit -m "chore: remove lint.yml (absorbed into ci.yml)"
git push
```

#### Step 5: Update Documentation

- [ ] Update `docs/roadmap.md` (PO responsibility)
- [ ] Archive feature spec to `docs/feature-specs/_archive/`

### 6.3 Rollback Strategy

If the new pipeline has critical issues:

**Immediate rollback (< 5 min):**
```bash
# Revert ci.yml to previous version
git revert HEAD  # if ci.yml was the last commit
# OR
git checkout HEAD~1 -- .github/workflows/ci.yml
git commit -m "revert: restore previous ci.yml"
git push
```

**If lint.yml was already deleted:**
```bash
# Restore from git history
git checkout HEAD~N -- .github/workflows/lint.yml  # N = commits since deletion
git commit -m "revert: restore lint.yml"
git push
```

**Branch protection:**
- Can be disabled instantly via GitHub UI
- Settings → Branches → Edit rule → Uncheck "Require status checks"

---

## 7. Testing the Pipeline

### 7.1 Test Scenarios Matrix

| # | Scenario | Trigger | Expected Behavior |
|---|----------|---------|-------------------|
| 1 | Clean PR | `pull_request` | All jobs green, build skipped |
| 2 | PR with TypeScript error | `pull_request` | `typecheck-ui` red, tests skipped |
| 3 | PR with ESLint error | `pull_request` | `lint-ui` red, tests skipped |
| 4 | PR with Rust fmt error | `pull_request` | `lint-engine` red, tests skipped |
| 5 | PR with UI test failure | `pull_request` | Stage 1 green, `test-ui` red |
| 6 | PR with Engine test failure | `pull_request` | Stage 1 green, `test-engine` red |
| 7 | Push clean to main | `push` | Full pipeline + build artifacts uploaded |
| 8 | Rapid pushes (same branch) | `push` × 2 | First run cancelled, second completes |
| 9 | Parallel PRs (different branches) | 2 × `pull_request` | Both run independently |

### 7.2 Manual Test Procedure

**Test 1: Clean PR**
```bash
git checkout -b test/ci-clean
echo "// test" >> ui/src/App.tsx  # trivial change
git add . && git commit -m "test: clean PR"
git push -u origin test/ci-clean
# Open PR, observe all checks pass, build-plugin shows "skipped"
```

**Test 2: Lint Failure (Fail-Fast)**
```bash
git checkout -b test/ci-lint-fail
echo "const x:any = 1;" >> ui/src/App.tsx  # ESLint error
git add . && git commit -m "test: lint failure"
git push -u origin test/ci-lint-fail
# Open PR, observe lint-ui fails, test jobs skipped
```

**Test 3: Concurrency Cancellation**
```bash
git checkout -b test/ci-concurrency
git commit --allow-empty -m "test: push 1"
git push
git commit --allow-empty -m "test: push 2"  # immediately
git push
# Observe first run cancelled in GitHub Actions UI
```

### 7.3 Verification Checklist

```
Stage 1 (Fast Feedback)
  [ ] typecheck-ui completes in <1 min
  [ ] lint-ui completes in <1 min
  [ ] lint-engine completes in <2 min
  [ ] lint-engine correctly downloads ui-dist artifact
  [ ] Stage 1 failure stops Stage 2

Stage 2 (Tests)
  [ ] test-ui runs after Stage 1
  [ ] test-engine downloads ui-dist artifact
  [ ] Both tests run in parallel

Stage 3 (Build)
  [ ] build-plugin skipped on PR (shows "Skipped" badge)
  [ ] build-plugin runs on push to main
  [ ] VST3/CLAP artifacts uploaded

General
  [ ] Concurrency cancels in-progress runs
  [ ] Total PR time <5 minutes
  [ ] Total main push time <10 minutes
```

---

## 8. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Ubuntu lint-engine fails due to platform differences | Low | Medium | Rust fmt/clippy are platform-agnostic; test in migration |
| Artifact upload/download adds latency | Medium | Low | ~10 sec overhead, acceptable for cost savings |
| Branch protection blocks urgent hotfix | Low | High | Admins can bypass; document emergency procedure |
| Concurrency cancels wanted runs | Low | Low | Only affects same branch; different branches isolated |

---

## 9. Future Considerations

### 9.1 Not In Scope (Deferred)

- **Windows CI:** Add when Windows support is prioritized
- **Release workflow changes:** `release.yml` works, just needs credentials
- **Parallelized Rust tests:** `cargo-nextest` for faster test execution
- **Merge queue:** GitHub merge queue for high-traffic repos

### 9.2 Potential Optimizations

If CI times need further reduction:

1. **Turborepo/nx for UI caching:** Cache lint/test results based on file changes
2. **Cargo build caching (sccache):** Distributed compilation cache
3. **Self-hosted runners:** For very high CI volume (not recommended for small teams)

---

## 10. Documentation Links

- [High-Level Design](../../architecture/high-level-design.md) — System architecture
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
- [Roadmap](../../roadmap.md) — Milestone tracking
- [User Stories](./user-stories.md) — Requirements for this feature
