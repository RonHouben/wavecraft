# User Stories: CI Pipeline Optimization

## Overview

Optimize the CI pipeline to reduce execution time and artifact storage costs. This addresses two backlog items:
1. Cache inefficiency causing test job recompilation
2. Artifact retention causing storage limit issues

## Version

**Target Version:** `0.6.2` (patch bump from `0.6.1`)

**Rationale:** Infrastructure improvement with no user-facing changes. Reduces CI execution time and storage costs — a DevEx/maintenance improvement warranting a patch bump per coding standards.

---

## User Story 1: Faster CI Feedback Loop

**As a** Wavecraft contributor  
**I want** the test-engine job to use cached compilation artifacts  
**So that** I get faster CI feedback on my pull requests

### Acceptance Criteria
- [ ] `prepare-engine` job pre-compiles test binaries with `cargo test --workspace --no-run`
- [ ] `test-engine` job shows minimal/no "Compiling" output (uses cache)
- [ ] Total CI time reduced by ~2-3 minutes per PR (net savings after longer prepare job)
- [ ] All existing tests continue to pass

### Notes
- The `prepare-engine` job will take ~1-2 minutes longer
- The `test-engine` job will save ~3-5 minutes
- Net time savings: ~2-3 minutes per PR

---

## User Story 2: Sustainable Artifact Storage

**As a** repository maintainer  
**I want** artifact retention to be tiered based on trigger type  
**So that** we stay within GitHub's storage limits without losing release artifacts

### Acceptance Criteria
- [ ] Main branch push artifacts retained for 7 days (down from 30)
- [ ] Release tag (`v*`) artifacts retained for 90 days
- [ ] `build-plugin` job triggers on both main branch and release tags
- [ ] VST3 and CLAP artifacts both use tiered retention
- [ ] Storage usage reduced by ~75-80%

### Notes
- Current: 30-day retention on all artifacts → ~3-5 GB/month
- Proposed: 7 days (main) / 90 days (tags) → ~700 MB - 1.1 GB/month
- Release artifacts remain accessible longer than before (90 > 30 days)

---

## User Story 3: Faster Release Builds

**As a** release manager  
**I want** the release workflow to use Rust caching  
**So that** release builds complete faster after the first run

### Acceptance Criteria
- [ ] `release.yml` workflow includes `Swatinem/rust-cache@v2` action
- [ ] Cache configured with `workspaces: engine` setting
- [ ] Second release build shows cache hit in logs
- [ ] Release build time reduced from ~15 min to ~5 min (after cache warm)

### Notes
- First release build will still be ~15 minutes (cold cache)
- Subsequent builds benefit from cache
- Cache key based on Cargo.lock ensures correctness

---

## User Story 4: Documentation Accuracy

**As a** project maintainer  
**I want** the backlog updated to reflect CI improvements  
**So that** the documentation accurately represents the current state

### Acceptance Criteria
- [ ] Backlog CI/CD section notes that cache inefficiency is addressed
- [ ] Backlog CI/CD section notes that artifact retention is addressed
- [ ] No misleading "TODO" items remain for completed work

---

## Out of Scope

- External artifact storage (R2/S3) — deferred unless GitHub limits become critical
- Windows/Linux CI runners — macOS-only for now
- Release workflow signing/notarization changes

---

## Dependencies

- Implementation plan: [implementation-plan.md](implementation-plan.md)
- Low-level design: [low-level-design-ci-optimization.md](low-level-design-ci-optimization.md)

---

## Definition of Done

- [ ] All acceptance criteria met
- [ ] CI pipeline passes on feature branch
- [ ] Local CI test with `act` shows expected behavior
- [ ] Version bumped to 0.6.2 in `engine/Cargo.toml`
- [ ] QA approval received
