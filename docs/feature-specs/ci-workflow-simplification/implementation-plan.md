# Implementation Plan: CI Workflow Simplification

## Overview

Remove redundant `push` triggers from CI and Template Validation workflows so they only run on pull requests. This eliminates duplicate validation runs when PRs are merged to main, saving ~10-14 CI minutes per merge.

## Requirements

- CI workflow runs only on PRs and manual dispatch (not on merge to main)
- Template Validation workflow runs only on PRs and manual dispatch (not on merge to main)
- Continuous Deploy workflow unchanged (runs on merge to main)
- Manual `workflow_dispatch` trigger added for emergency runs
- Documentation updated to reflect new trigger behavior

## Architecture Changes

| File | Change |
|------|--------|
| `.github/workflows/ci.yml` | Remove `push.branches`, add `workflow_dispatch` |
| `.github/workflows/template-validation.yml` | Remove `push.branches`, add `workflow_dispatch` |
| `docs/guides/ci-pipeline.md` | Update documentation to reflect new triggers |

## Implementation Steps

### Phase 1: Verify Prerequisites

1. **Verify branch protection** (Manual)
   - Action: Confirm branch protection requires status checks before merge
   - Why: Ensures PRs cannot merge without passing CI
   - Dependencies: None
   - Risk: Low

### Phase 2: Update Workflow Triggers

2. **Update ci.yml triggers** (File: `.github/workflows/ci.yml`)
   - Action: Remove `push.branches: [main]`, add `workflow_dispatch`
   - Why: Eliminate redundant post-merge runs
   - Dependencies: Step 1 verified
   - Risk: Low

3. **Update template-validation.yml triggers** (File: `.github/workflows/template-validation.yml`)
   - Action: Remove `push.branches: [main]`, add `workflow_dispatch`
   - Why: Eliminate redundant post-merge runs
   - Dependencies: Step 1 verified
   - Risk: Low

### Phase 3: Update Documentation

4. **Update ci-pipeline.md** (File: `docs/guides/ci-pipeline.md`)
   - Action: Update "Triggers" section to reflect PR-only behavior
   - Why: Keep documentation accurate
   - Dependencies: Steps 2-3 complete
   - Risk: Low

### Phase 4: Verification

5. **Create test PR** (Manual)
   - Action: Create branch, make trivial change, open PR
   - Why: Verify CI and Template Validation run on PR
   - Dependencies: Steps 2-4 complete
   - Risk: Low

6. **Merge test PR** (Manual)
   - Action: Merge the test PR
   - Why: Verify only Continuous Deploy runs on merge
   - Dependencies: Step 5 complete
   - Risk: Low

## Testing Strategy

- **Automated tests:** None required (workflow configuration change)
- **Manual tests:**
  1. Open PR → Verify CI and Template Validation trigger
  2. Merge PR → Verify only Continuous Deploy triggers
  3. Manual dispatch → Verify CI can be manually triggered from Actions tab

## Risks & Mitigations

- **Risk**: Direct commits to main bypass validation
  - Mitigation: Accept risk; direct commits are rare and from trusted admins

- **Risk**: Branch protection not enforced
  - Mitigation: Verify settings before deployment (Step 1)

## Success Criteria

- [ ] CI workflow triggers only on `pull_request` and `workflow_dispatch`
- [ ] Template Validation triggers only on `pull_request` and `workflow_dispatch`
- [ ] Merging a PR triggers only Continuous Deploy
- [ ] Documentation reflects new trigger behavior
- [ ] Manual dispatch works for both CI and Template Validation
