# Implementation Plan: CI/CD Pipeline Redesign

**Version:** 1.0  
**Date:** 2026-01-31  
**Status:** Ready for Implementation

---

## Overview

This plan transforms the current monolithic CI pipeline into a staged, fail-fast architecture that enables PR testing, provides fast feedback (<2 minutes), and reduces CI costs by ~88% for PRs through Ubuntu runner usage.

**Input Documents:**
- [User Stories](./user-stories.md)
- [Low-Level Design](./low-level-design-ci-cd-pipeline-redesign.md)

---

## Prerequisites

Before starting implementation, ensure:

1. **Repository access:** Write access to `.github/workflows/` directory
2. **GitHub permissions:** Admin access for branch protection configuration
3. **Existing state verified:**
   - `ui/package.json` has `typecheck` script (✅ already exists: `"typecheck": "tsc --noEmit"`)
   - Current `ci.yml` and `lint.yml` are at expected state (no pending changes)

---

## Implementation Steps

### Phase 1: Create New `ci.yml` (Keep `lint.yml` Active)

**Goal:** Deploy new staged pipeline alongside existing workflows for validation.

**Estimated Time:** 30 minutes

---

#### Step 1.1: Replace `ci.yml` with Staged Pipeline

**File:** `.github/workflows/ci.yml`

**Action:** Replace the entire contents of `ci.yml` with the new staged pipeline structure from the low-level design.

**Changes:**
1. Update workflow name to `CI`
2. Enable both `push` and `pull_request` triggers for `main` branch
3. Add concurrency control block
4. Create 6 jobs in 3 stages:
   - **Stage 1 (Fast Feedback):** `typecheck-ui`, `lint-ui`, `lint-engine`
   - **Stage 2 (Tests):** `test-ui`, `test-engine`
   - **Stage 3 (Build):** `build-plugin` (main only)

**Key Configuration Details:**

```yaml
# Triggers (enable PR testing)
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

# Concurrency control
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

**Job Runner Assignments:**
| Job | Runner |
|-----|--------|
| `typecheck-ui` | `ubuntu-latest` |
| `lint-ui` | `ubuntu-latest` |
| `lint-engine` | `ubuntu-latest` |
| `test-ui` | `ubuntu-latest` |
| `test-engine` | `ubuntu-latest` |
| `build-plugin` | `macos-latest` |

**Job Dependencies:**
- `lint-engine` needs: `[typecheck-ui]` (requires UI artifact)
- `test-ui` needs: `[typecheck-ui, lint-ui, lint-engine]`
- `test-engine` needs: `[typecheck-ui, lint-ui, lint-engine]`
- `build-plugin` needs: `[test-ui, test-engine]` with `if: github.ref == 'refs/heads/main'`

**Artifact Sharing:**
- `typecheck-ui` uploads `ui-dist` artifact after building UI
- `lint-engine`, `test-engine`, and `build-plugin` download `ui-dist` artifact

**Expected Outcome:**
- New `ci.yml` committed but both workflows run in parallel temporarily
- PRs now trigger the new CI workflow

**Verification:**
- [ ] File syntax is valid YAML (GitHub Actions validates on push)
- [ ] Both `CI` and `Lint` workflows appear in Actions tab

---

### Phase 2: Validate New Pipeline

**Goal:** Confirm all scenarios work correctly before removing `lint.yml`.

**Estimated Time:** 45 minutes (including waiting for CI runs)

---

#### Step 2.1: Test Clean PR Scenario

**Action:** Create a test branch with a trivial change.

```bash
git checkout -b test/ci-clean-validation
echo "// CI validation test" >> ui/src/App.tsx
git add . && git commit -m "test: validate clean PR"
git push -u origin test/ci-clean-validation
```

**Expected Result:**
- [ ] `typecheck-ui` passes and uploads artifact (< 1 min)
- [ ] `lint-ui` passes (< 1 min)
- [ ] `lint-engine` passes and downloads artifact (< 2 min)
- [ ] `test-ui` passes (< 2 min)
- [ ] `test-engine` passes (< 2 min)
- [ ] `build-plugin` shows "Skipped" (not run on PR)
- [ ] Total PR time < 5 minutes

**Cleanup:** Close PR without merging, delete branch.

---

#### Step 2.2: Test Fail-Fast Behavior (Lint Failure)

**Action:** Create a test branch with an intentional lint error.

```bash
git checkout -b test/ci-lint-fail-validation
echo "const unused_var:any = 1;" >> ui/src/test-lint-error.ts
git add . && git commit -m "test: validate lint failure stops pipeline"
git push -u origin test/ci-lint-fail-validation
```

**Expected Result:**
- [ ] `lint-ui` fails (ESLint `no-explicit-any` error)
- [ ] `test-ui` and `test-engine` are skipped (not started)
- [ ] `build-plugin` is skipped

**Cleanup:** Close PR without merging, delete branch.

---

#### Step 2.3: Test Concurrency Cancellation

**Action:** Make two rapid pushes to the same branch.

```bash
git checkout -b test/ci-concurrency-validation
git commit --allow-empty -m "test: push 1 for concurrency"
git push -u origin test/ci-concurrency-validation
git commit --allow-empty -m "test: push 2 for concurrency (should cancel first)"
git push  # Push immediately while first run is still in progress
```

**Expected Result:**
- [ ] First workflow run shows "Cancelled"
- [ ] Second workflow run completes normally

**Cleanup:** Close PR without merging, delete branch.

---

#### Step 2.4: Test Main Branch Push (Full Pipeline)

**Action:** After validating PR scenarios, merge the `ci.yml` changes to `main`.

**Expected Result:**
- [ ] All Stage 1 jobs pass
- [ ] All Stage 2 jobs pass
- [ ] `build-plugin` runs on `macos-latest`
- [ ] VST3 and CLAP artifacts uploaded successfully
- [ ] Total time < 12 minutes

---

### Phase 3: Delete `lint.yml` Workflow

**Goal:** Remove duplicate workflow after new pipeline is validated.

**Estimated Time:** 10 minutes

---

#### Step 3.1: Delete `lint.yml` File

**File:** `.github/workflows/lint.yml`

**Action:** Delete the file.

```bash
git rm .github/workflows/lint.yml
git commit -m "chore: remove lint.yml (absorbed into ci.yml)"
```

**Expected Outcome:**
- Only `ci.yml` and `release.yml` remain in `.github/workflows/`
- Lint workflow no longer appears in Actions tab
- No duplicate lint runs on PRs

**Verification:**
- [ ] File deleted from repository
- [ ] Next PR only shows `CI` workflow (not `Lint`)

---

### Phase 4: Configure Branch Protection

**Goal:** Require CI checks to pass before PR merge.

**Estimated Time:** 15 minutes

---

#### Step 4.1: Configure Branch Protection Rules

**Location:** GitHub Repository → Settings → Branches → Branch protection rules

**Action:** Add or edit rule for `main` branch.

**Configuration:**

| Setting | Value |
|---------|-------|
| Branch name pattern | `main` |
| Require a pull request before merging | ☑ Enabled |
| Require status checks to pass before merging | ☑ Enabled |
| Require branches to be up to date before merging | ☑ Enabled |

**Required Status Checks (add all 5):**
1. `TypeCheck UI`
2. `Lint UI`
3. `Lint Engine`
4. `Test UI`
5. `Test Engine`

**Do NOT add:**
- `Build Plugin` (only runs on `main`, would block PRs)

**Optional Settings:**
- Require linear history: ☑ Recommended
- Include administrators: ☐ Disabled (allows emergency bypasses)

**Expected Outcome:**
- PRs cannot merge until all 5 required checks pass
- Merge button disabled until CI passes

**Verification:**
- [ ] Create a test PR with passing CI
- [ ] Confirm merge button is enabled
- [ ] Create a test PR with failing lint
- [ ] Confirm merge button is disabled with message about failing checks

---

### Phase 5: Final Validation & Cleanup

**Goal:** Confirm complete system works end-to-end.

**Estimated Time:** 20 minutes

---

#### Step 5.1: End-to-End Validation

**Action:** Create a real feature PR (small documentation change).

```bash
git checkout -b docs/ci-pipeline-complete
# Add completion note to implementation-progress.md
git add . && git commit -m "docs: mark CI/CD redesign complete"
git push -u origin docs/ci-pipeline-complete
```

**Verify:**
- [ ] CI runs all 5 required checks
- [ ] Build-plugin is skipped on PR
- [ ] Branch protection shows all checks required
- [ ] Merge is only possible after CI passes
- [ ] After merge, `build-plugin` runs on `main`

---

#### Step 5.2: Clean Up Test Branches

**Action:** Delete all test branches created during validation.

```bash
git push origin --delete test/ci-clean-validation
git push origin --delete test/ci-lint-fail-validation
git push origin --delete test/ci-concurrency-validation
```

---

## Testing Checkpoints

### After Phase 1 Completion
- [ ] New `ci.yml` deployed
- [ ] Both `CI` and `Lint` workflows run on PRs (temporarily)
- [ ] No syntax errors in workflow file

### After Phase 2 Completion
- [ ] Clean PR triggers all Stage 1+2 jobs
- [ ] Lint failure triggers fail-fast (Stage 2 skipped)
- [ ] Concurrency cancellation works
- [ ] Main branch push triggers full pipeline including build

### After Phase 3 Completion
- [ ] `lint.yml` deleted
- [ ] Only one lint runs per PR (in `CI` workflow)

### After Phase 4 Completion
- [ ] Branch protection rules active
- [ ] Merge blocked when CI fails
- [ ] 5 required checks configured

### After Phase 5 Completion
- [ ] Full end-to-end flow validated
- [ ] Test branches cleaned up
- [ ] Ready for handoff to Tester agent

---

## Rollback Instructions

### If New `ci.yml` Has Critical Issues (Phase 1-2)

```bash
# Revert to previous ci.yml
git checkout HEAD~1 -- .github/workflows/ci.yml
git commit -m "revert: restore previous ci.yml"
git push
```

### If `lint.yml` Was Deleted Prematurely (Phase 3)

```bash
# Restore lint.yml from git history
git checkout HEAD~N -- .github/workflows/lint.yml  # N = commits since deletion
git commit -m "revert: restore lint.yml"
git push
```

### If Branch Protection Blocks Urgent Hotfix (Phase 4)

1. Go to: Repository → Settings → Branches → Edit rule for `main`
2. Uncheck "Require status checks to pass before merging"
3. Save rule
4. Merge hotfix
5. Re-enable branch protection

### Complete Rollback (All Phases)

If all changes need to be reverted:

```bash
# Revert to state before CI/CD redesign
git revert <commit-hash-of-ci.yml-change>
git revert <commit-hash-of-lint.yml-deletion>  # if applicable
git push

# Manually disable branch protection via GitHub UI
```

---

## Estimated Effort Summary

| Phase | Description | Estimated Time |
|-------|-------------|----------------|
| Phase 1 | Create new `ci.yml` | 30 minutes |
| Phase 2 | Validate new pipeline | 45 minutes |
| Phase 3 | Delete `lint.yml` | 10 minutes |
| Phase 4 | Configure branch protection | 15 minutes |
| Phase 5 | Final validation & cleanup | 20 minutes |
| **Total** | | **~2 hours** |

**Note:** Phase 2 time includes waiting for CI runs to complete (~5-10 minutes each).

---

## Success Criteria

From user stories Definition of Done:

- [ ] All acceptance criteria met
- [ ] PR triggers CI and shows pass/fail status
- [ ] CI completes lint+test in under 5 minutes for PRs
- [ ] `lint.yml` deleted
- [ ] Branch protection enabled (requires CI to pass)
- [ ] Documentation updated

---

## Handoff Notes

### For Coder Agent

1. Start with Phase 1 — this is the largest code change
2. Use the complete `ci.yml` from [Low-Level Design Section 3.1](./low-level-design-ci-cd-pipeline-redesign.md#31-complete-ciyml-structure)
3. Do NOT delete `lint.yml` until Phase 2 validation is complete
4. Branch protection (Phase 4) requires GitHub admin access — ensure you have permissions

### For Tester Agent

After implementation, test plan should cover:
- Scenario 1: Clean PR → all checks pass
- Scenario 2: Lint failure → fail-fast behavior
- Scenario 3: Test failure → Stage 2 fails, build skipped
- Scenario 4: Main push → full pipeline including build
- Scenario 5: Rapid pushes → concurrency cancellation
- Scenario 6: Branch protection → blocked merge on failure

---

## Documentation Links

- [User Stories](./user-stories.md) — Requirements
- [Low-Level Design](./low-level-design-ci-cd-pipeline-redesign.md) — Technical specification
- [High-Level Design](../../architecture/high-level-design.md) — System architecture
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
- [Roadmap](../../roadmap.md) — Milestone tracking
