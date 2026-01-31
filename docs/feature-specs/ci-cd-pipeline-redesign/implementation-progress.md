# Implementation Progress: CI/CD Pipeline Redesign

**Status:** Phase 1 Complete  
**Last Updated:** 2026-01-31

---

## Phase Summary

| Phase | Status | Completed |
|-------|--------|-----------|
| Phase 1: Create new `ci.yml` | ✅ Complete | 2026-01-31 |
| Phase 2: Validate new pipeline | ⬜ Not Started | |
| Phase 3: Delete `lint.yml` | ⬜ Not Started | |
| Phase 4: Configure branch protection | ⬜ Not Started | |
| Phase 5: Final validation & cleanup | ⬜ Not Started | |

---

## Task Checklist

### Phase 1: Create New `ci.yml` (Keep `lint.yml` Active)

- [x] **1.1** Replace `.github/workflows/ci.yml` with staged pipeline
  - [x] Update workflow name to `CI`
  - [x] Enable `pull_request` trigger for `main` branch
  - [x] Add concurrency control block
  - [x] Create `typecheck-ui` job (ubuntu-latest)
  - [x] Create `lint-ui` job (ubuntu-latest)
  - [x] Create `lint-engine` job (ubuntu-latest, depends on typecheck-ui)
  - [x] Create `test-ui` job (ubuntu-latest, depends on Stage 1)
  - [x] Create `test-engine` job (ubuntu-latest, depends on Stage 1)
  - [x] Create `build-plugin` job (macos-latest, main only)
  - [x] Configure artifact upload in `typecheck-ui`
  - [x] Configure artifact download in `lint-engine`, `test-engine`, `build-plugin`
  - [ ] Commit and push changes

### Phase 2: Validate New Pipeline

- [ ] **2.1** Test clean PR scenario
  - [ ] Create test branch `test/ci-clean-validation`
  - [ ] Open PR
  - [ ] Verify all Stage 1+2 jobs pass
  - [ ] Verify `build-plugin` shows "Skipped"
  - [ ] Verify total time < 5 minutes
  - [ ] Close PR without merging
  - [ ] Delete test branch

- [ ] **2.2** Test fail-fast behavior (lint failure)
  - [ ] Create test branch `test/ci-lint-fail-validation`
  - [ ] Add intentional lint error
  - [ ] Open PR
  - [ ] Verify `lint-ui` fails
  - [ ] Verify Stage 2 jobs are skipped
  - [ ] Close PR without merging
  - [ ] Delete test branch

- [ ] **2.3** Test concurrency cancellation
  - [ ] Create test branch `test/ci-concurrency-validation`
  - [ ] Make two rapid pushes
  - [ ] Verify first run is cancelled
  - [ ] Close PR without merging
  - [ ] Delete test branch

- [ ] **2.4** Test main branch push (full pipeline)
  - [ ] Merge ci.yml changes to main
  - [ ] Verify all stages run
  - [ ] Verify `build-plugin` runs on macos-latest
  - [ ] Verify artifacts uploaded

### Phase 3: Delete `lint.yml` Workflow

- [ ] **3.1** Delete `.github/workflows/lint.yml`
  - [ ] Run `git rm .github/workflows/lint.yml`
  - [ ] Commit with message: `chore: remove lint.yml (absorbed into ci.yml)`
  - [ ] Push to main
  - [ ] Verify only `CI` workflow runs on next PR

### Phase 4: Configure Branch Protection

- [ ] **4.1** Configure branch protection rules
  - [ ] Go to Repository → Settings → Branches
  - [ ] Add/edit rule for `main` branch
  - [ ] Enable "Require a pull request before merging"
  - [ ] Enable "Require status checks to pass before merging"
  - [ ] Enable "Require branches to be up to date before merging"
  - [ ] Add required check: `TypeCheck UI`
  - [ ] Add required check: `Lint UI`
  - [ ] Add required check: `Lint Engine`
  - [ ] Add required check: `Test UI`
  - [ ] Add required check: `Test Engine`
  - [ ] Save rule

- [ ] **4.2** Verify branch protection works
  - [ ] Create test PR with passing CI
  - [ ] Verify merge button enabled
  - [ ] Create test PR with failing lint
  - [ ] Verify merge button disabled

### Phase 5: Final Validation & Cleanup

- [ ] **5.1** End-to-end validation
  - [ ] Create real feature PR
  - [ ] Verify all 5 required checks run
  - [ ] Verify build-plugin skipped on PR
  - [ ] Verify merge only possible after CI passes
  - [ ] Merge PR
  - [ ] Verify build-plugin runs on main

- [ ] **5.2** Clean up test branches
  - [ ] Delete `test/ci-clean-validation` (if still exists)
  - [ ] Delete `test/ci-lint-fail-validation` (if still exists)
  - [ ] Delete `test/ci-concurrency-validation` (if still exists)

---

## Verification Checklist

### Pipeline Behavior
- [ ] PRs trigger all Stage 1+2 jobs
- [ ] PRs do NOT trigger `build-plugin`
- [ ] Main push triggers full pipeline including build
- [ ] Lint failure stops Stage 2 execution
- [ ] Concurrency cancellation works

### Performance Targets
- [ ] Stage 1 completes in < 2 minutes
- [ ] Total PR time < 5 minutes
- [ ] Total main push time < 12 minutes

### Cost Optimization
- [ ] `typecheck-ui` runs on ubuntu
- [ ] `lint-ui` runs on ubuntu
- [ ] `lint-engine` runs on ubuntu
- [ ] `test-ui` runs on ubuntu
- [ ] `test-engine` runs on ubuntu
- [ ] Only `build-plugin` runs on macos

### Branch Protection
- [ ] All 5 required checks configured
- [ ] Merge blocked when CI fails
- [ ] Admins can bypass for emergencies

---

## Notes

### Phase 1 Implementation (2026-01-31)

**Changes Made:**
1. Replaced `.github/workflows/ci.yml` with new staged pipeline structure
   - Workflow name changed from "CI Build" to "CI"
   - Enabled PR testing via `pull_request` trigger
   - Added concurrency control to cancel stale runs
   - Migrated from single monolithic job to 6 specialized jobs

2. Verified `ui/package.json` has `typecheck` script (already present)

**Pipeline Architecture:**
- **Stage 1 (Fast Feedback):** `typecheck-ui`, `lint-ui`, `lint-engine` (all ubuntu)
- **Stage 2 (Tests):** `test-ui`, `test-engine` (both ubuntu, depend on Stage 1)
- **Stage 3 (Build):** `build-plugin` (macos, main only, depends on Stage 2)

**Key Features:**
- Artifact sharing: `typecheck-ui` builds and uploads UI dist, shared with engine jobs
- Fail-fast: Stage 2 only runs if Stage 1 passes
- Cost optimization: Ubuntu runners for all non-build jobs (~90% cheaper)
- PR efficiency: Skip `build-plugin` on PRs, ~8 min savings per PR

**Status:** Ready for Phase 2 validation (commit and push required)

---

## Completion

- [ ] All phases complete
- [ ] All verification checks pass
- [ ] Ready for Tester handoff
