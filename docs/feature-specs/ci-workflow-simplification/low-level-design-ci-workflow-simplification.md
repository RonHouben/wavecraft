# CI Workflow Simplification — Low-Level Design

**Status:** Draft  
**Created:** 2026-02-05  
**Author:** Architect Agent

---

## Problem Statement

When a PR is merged to `main`, three workflows run simultaneously:

| Workflow | Jobs | Est. Time | Purpose |
|----------|------|-----------|---------|
| **CI** | 7 jobs | ~5-8 min | Lint, test, build validation |
| **Template Validation** | 1 job | ~4-6 min | Scaffold & compile test plugin |
| **Continuous Deploy** | 5 jobs | ~2-10 min | Publish changed packages |

This is redundant:
- CI and Template Validation already ran on the PR (merge requirement)
- Re-running them post-merge wastes ~10-14 minutes of CI time per merge
- No code changes occur between PR approval and merge

---

## Goals

| Goal | Priority |
|------|----------|
| Eliminate redundant validation runs on merge to main | High |
| Maintain PR-based quality gates (no regression) | High |
| Reduce CI minutes consumption | High |
| Preserve ability to manually trigger validation | Medium |
| Handle edge case of direct-to-main commits | Low |

---

## Non-Goals

- Changing the structure of individual workflows
- Modifying job dependencies within workflows
- Adding new validation jobs

---

## Current Trigger Configuration

```yaml
# ci.yml
on:
  push:
    branches: [main]      # ← REDUNDANT after PR merge
  pull_request:
    branches: [main]      # ← REQUIRED quality gate

# template-validation.yml
on:
  push:
    branches: [main]      # ← REDUNDANT after PR merge
  pull_request:
    branches: [main]      # ← REQUIRED quality gate

# continuous-deploy.yml
on:
  push:
    branches: [main]      # ← CORRECT: deploy on merge
  workflow_dispatch:       # ← CORRECT: manual trigger
```

---

## Proposed Trigger Configuration

```yaml
# ci.yml — CHANGE: Remove push trigger
on:
  pull_request:
    branches: [main]
  workflow_dispatch:       # ← ADD: Allow manual runs

# template-validation.yml — CHANGE: Remove push trigger
on:
  pull_request:
    branches: [main]
  workflow_dispatch:       # ← ADD: Allow manual runs

# continuous-deploy.yml — NO CHANGE
on:
  push:
    branches: [main]
  workflow_dispatch:
```

---

## Architecture

### Before (Current State)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    CURRENT WORKFLOW TRIGGERS                            │
└─────────────────────────────────────────────────────────────────────────┘

     PR Created/Updated ────────────►  CI ✅
                                       Template Validation ✅
                                              │
                                              ▼
                                       ┌──────────────┐
                                       │  PR Approved │
                                       │  Checks Pass │
                                       └──────┬───────┘
                                              │
                                        merge │
                                              ▼
     Push to main ─────────────────►  CI ❌ (redundant)
                                      Template Validation ❌ (redundant)
                                      Continuous Deploy ✅

     Total workflows on merge: 3
     Redundant CI minutes: ~10-14 min per merge
```

### After (Proposed State)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                   SIMPLIFIED WORKFLOW TRIGGERS                          │
└─────────────────────────────────────────────────────────────────────────┘

     PR Created/Updated ────────────►  CI ✅ (quality gate)
                                       Template Validation ✅ (quality gate)
                                              │
                                              ▼
                                       ┌──────────────┐
                                       │  PR Approved │
                                       │  Checks Pass │
                                       └──────┬───────┘
                                              │
                                        merge │
                                              ▼
     Push to main ─────────────────►  Continuous Deploy ✅ (only)

     Total workflows on merge: 1
     Redundant CI minutes: 0
```

---

## File Changes

### 1. `.github/workflows/ci.yml`

**Current:**
```yaml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
```

**Proposed:**
```yaml
on:
  pull_request:
    branches: [main]
  workflow_dispatch:  # Manual trigger for emergencies
```

### 2. `.github/workflows/template-validation.yml`

**Current:**
```yaml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
```

**Proposed:**
```yaml
on:
  pull_request:
    branches: [main]
  workflow_dispatch:  # Manual trigger for emergencies
```

### 3. `.github/workflows/continuous-deploy.yml`

**No changes required.** Current configuration is already correct.

---

## Edge Cases

### Direct Commits to Main (Admin Bypass)

**Scenario:** An admin pushes directly to main (bypassing PR).

**Impact:** Code reaches main without CI or Template Validation.

**Mitigation Options:**

| Option | Implementation | Recommendation |
|--------|----------------|----------------|
| A. Accept risk | None | ✅ **Recommended** — Direct commits are rare; trust the committer |
| B. Add skip logic | Check if commit is from merged PR, skip validation if so | ❌ Over-engineering |
| C. Keep push trigger | Revert to current behavior | ❌ Defeats purpose |

**Recommendation:** Accept option A. Direct commits to main should be:
1. Rare (emergencies, automated commits)
2. From trusted admins
3. Small/low-risk changes

If direct commits become problematic, revisit with option B.

### Automated Commits (`[skip ci]`)

The Continuous Deploy workflow already handles this correctly:
- Bot commits include `[skip ci]` in the message
- GitHub Actions skips workflows for these commits
- No infinite loop risk

---

## Branch Protection Prerequisites

For this simplification to be safe, ensure branch protection rules require:

- [x] **Require status checks to pass** — PR must pass CI and Template Validation
- [x] **Require branches to be up to date** — Prevents merging stale branches
- [ ] **Require linear history** — Optional, but recommended

**Verification command:**
```bash
gh api repos/RonHouben/wavecraft/branches/main/protection | jq '.required_status_checks'
```

---

## Impact Analysis

### CI Minutes Savings

| Metric | Current | Proposed | Savings |
|--------|---------|----------|---------|
| Workflows per merge | 3 | 1 | 66% reduction |
| CI minutes per merge | ~15-24 min | ~5-10 min | ~10-14 min |
| Monthly (20 merges) | ~480 min | ~200 min | ~280 min |

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Direct commit bypasses validation | Low | Medium | Admin discipline; workflow_dispatch fallback |
| Branch protection misconfigured | Low | High | Verify settings before deployment |
| Manual dispatch forgotten | Low | Low | Documented in CONTRIBUTING.md |

---

## Implementation Plan

### Step 1: Verify Branch Protection

Before making changes, confirm branch protection requires status checks:

```bash
gh api repos/RonHouben/wavecraft/branches/main/protection
```

### Step 2: Update ci.yml

Remove `push.branches` trigger, add `workflow_dispatch`.

### Step 3: Update template-validation.yml

Remove `push.branches` trigger, add `workflow_dispatch`.

### Step 4: Test with a PR

1. Create a test branch
2. Make a trivial change
3. Open PR → Verify CI and Template Validation run
4. Merge PR → Verify only Continuous Deploy runs

### Step 5: Update Documentation

Update [ci-pipeline.md](../../guides/ci-pipeline.md) to reflect the simplified triggers.

---

## Rollback Plan

If issues arise, revert by adding back the push trigger:

```yaml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
```

This is a safe, reversible change.

---

## Related Documents

- [CI Pipeline Guide](../../guides/ci-pipeline.md) — User-facing documentation
- [High-Level Design](../../architecture/high-level-design.md) — Build system architecture
- [Agent Development Flow](../../architecture/agent-development-flow.md) — Testing workflow

---

## Appendix: Complete Workflow Inventory

| Workflow | Trigger | Purpose | Post-Change |
|----------|---------|---------|-------------|
| ci.yml | PR, ~~push~~, dispatch | Lint, test, build | PR + manual only |
| template-validation.yml | PR, ~~push~~, dispatch | Scaffold validation | PR + manual only |
| continuous-deploy.yml | push, dispatch | Publish packages | No change |
| release.yml | tags (`v*`), dispatch | Signed release build | No change |
| cli-release.yml | tags (`cli-v*`), dispatch | Manual CLI release | No change |
