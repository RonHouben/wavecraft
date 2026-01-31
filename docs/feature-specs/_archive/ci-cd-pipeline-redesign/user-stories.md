# CI/CD Pipeline Redesign — User Stories

**Epic:** CI/CD Pipeline Optimization  
**Goal:** Fast, reliable, cost-efficient CI that catches issues early and provides clear feedback

---

## Context

The current CI/CD setup has grown organically and now has several architectural issues:
- Monolithic 15-minute pipeline with no fail-fast behavior
- Duplicate linting across two workflows (`ci.yml` and `lint.yml`)
- PR trigger disabled on main CI (PRs only get lint checks)
- Expensive macOS runners used where ubuntu would work
- No concurrency control (duplicate runs on rapid pushes)

See [Architect's Analysis](./architecture-analysis.md) for detailed technical review.

---

## User Stories

### Story 1: Enable PR Testing

**As a** developer  
**I want** tests to run automatically on pull requests  
**So that** I catch issues before merging to main

#### Acceptance Criteria
- [ ] CI runs lint + tests on all PRs targeting `main`
- [ ] PRs cannot merge with failing CI checks (branch protection)
- [ ] Plugin build only runs on push to `main` (not on PRs)
- [ ] PR check results are visible in GitHub PR UI

#### Notes
- This addresses the current gap where PRs only get lint checks
- Plugin build is expensive (~8 min) and not needed for PR validation

---

### Story 2: Fast Feedback Stage (<2 minutes)

**As a** developer  
**I want** lint and type-check results within 2 minutes  
**So that** I get immediate feedback on basic issues without waiting for the full pipeline

#### Acceptance Criteria
- [ ] Lint (UI + Engine) and type-check run in parallel as Stage 1
- [ ] Stage 1 completes in under 2 minutes
- [ ] If Stage 1 fails, subsequent stages are skipped (no wasted compute)
- [ ] Clear failure messages identify which check failed

#### Notes
- Currently developers wait 15 min to discover a formatting issue
- Fail-fast saves both time and CI costs

---

### Story 3: Staged Pipeline (Fail-Fast)

**As a** developer  
**I want** the CI pipeline to stop early when a stage fails  
**So that** I don't wait 15 minutes to discover a lint error

#### Acceptance Criteria
- [ ] Pipeline has 3 stages: Fast Feedback → Tests → Build
- [ ] Stage 2 (Tests) only runs if Stage 1 passes
- [ ] Stage 3 (Build) only runs if Stage 2 passes AND on `main` branch
- [ ] Each stage clearly reports pass/fail in GitHub Actions UI

#### Notes
- Staged dependencies via `needs:` in GitHub Actions
- Build stage conditional: `if: github.ref == 'refs/heads/main'`

---

### Story 4: Reduce CI Costs (Ubuntu for Non-Build Jobs)

**As a** project maintainer  
**I want** CI jobs to run on ubuntu where possible  
**So that** we minimize GitHub Actions costs (10x cheaper than macOS)

#### Acceptance Criteria
- [ ] `lint-ui` runs on ubuntu
- [ ] `lint-engine` runs on ubuntu (with UI artifact sharing)
- [ ] `typecheck-ui` runs on ubuntu
- [ ] `test-ui` runs on ubuntu
- [ ] `test-engine` runs on ubuntu (unless macOS-specific tests exist)
- [ ] Only `build-plugin` runs on macOS (required for bundle creation)

#### Cost Impact
| Scenario | Current | Proposed | Savings |
|----------|---------|----------|---------|
| Per PR | ~$0.32 | ~$0.04 | 88% |
| Per main push | ~$1.20 | ~$0.68 | 43% |

#### Notes
- Engine lint requires UI artifacts to compile
- Solution: Build UI in typecheck stage, upload artifact, download in lint-engine

---

### Story 5: Eliminate Duplicate Linting Workflow

**As a** developer  
**I want** a single source of truth for linting  
**So that** I don't get confused about which workflow owns what

#### Acceptance Criteria
- [ ] Delete `lint.yml` workflow file
- [ ] All linting handled within `ci.yml`
- [ ] No duplicate job names or purposes across workflows
- [ ] Documentation updated to reflect single CI workflow

#### Notes
- Currently `lint.yml` and `ci.yml` both run linting
- This creates confusion about ownership and triggers

---

### Story 6: Add Concurrency Control

**As a** developer  
**I want** new CI runs to cancel in-progress runs for the same branch  
**So that** rapid pushes don't waste compute on outdated commits

#### Acceptance Criteria
- [ ] Pushing a new commit cancels any in-progress CI for that branch
- [ ] Concurrent runs on different branches are not affected
- [ ] Concurrency group includes workflow name and ref

#### Implementation
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

---

### Story 7: Add TypeScript Type-Check Job

**As a** developer  
**I want** TypeScript type errors caught in a separate fast job  
**So that** I get immediate feedback on type issues (fastest check)

#### Acceptance Criteria
- [ ] Add `type-check` npm script to `ui/package.json`
- [ ] New `typecheck-ui` job runs `npm run type-check`
- [ ] Job completes in under 30 seconds
- [ ] Type errors clearly reported in CI output

#### Notes
- Type-checking is faster than full ESLint
- Developers often have type errors before style issues
- This job also builds UI artifacts that can be shared with other jobs

---

## Priority Order

| Priority | Story | Rationale |
|----------|-------|-----------|
| 1 | Story 1: Enable PR Testing | Critical gap — PRs merge without test validation |
| 2 | Story 3: Staged Pipeline | Foundation for fail-fast behavior |
| 3 | Story 6: Concurrency Control | Quick win, prevents waste |
| 4 | Story 2: Fast Feedback Stage | Improves developer experience |
| 5 | Story 5: Eliminate Duplicate Linting | Cleanup, reduces confusion |
| 6 | Story 7: TypeScript Type-Check | Adds fastest feedback loop |
| 7 | Story 4: Reduce CI Costs | Optimization, lower priority |

---

## Out of Scope

- Changes to `release.yml` (works correctly, just needs Apple credentials)
- Windows/Linux CI runners (macOS is primary target)
- Automated deployment (not applicable for audio plugins)

---

## Definition of Done

- [ ] All acceptance criteria met
- [ ] PR triggers CI and shows pass/fail status
- [ ] CI completes lint+test in under 5 minutes for PRs
- [ ] `lint.yml` deleted
- [ ] Branch protection enabled (requires CI to pass)
- [ ] Documentation updated

