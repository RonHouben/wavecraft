# Implementation Plan: CI Build Stage Removal

## Overview

Remove the redundant `build-plugin` job from `.github/workflows/ci.yml` that never executes due to conflicting trigger conditions (workflow runs on PRs, job requires main branch). This is a safe cleanup with zero breaking change risk.

## Requirements

- Remove the `build-plugin` job definition (lines 227-272)
- Update CI documentation to reflect the actual workflow
- Verify no job dependencies are broken
- Maintain all existing validation checks

## Architecture Changes

| File | Change | Lines |
|------|--------|-------|
| `.github/workflows/ci.yml` | Delete `build-plugin` job, remove stage 3 header | 227-272 |
| `docs/guides/ci-pipeline.md` | Update diagram, remove build-plugin references | Multiple |

## Implementation Steps

### Phase 1: Verification (Pre-Removal Checks)

1. **Confirm no job dependencies** (Manual check)
   - Action: Verify no jobs use `needs: [build-plugin]`
   - Why: Ensure removal won't break job graph
   - Dependencies: None
   - Risk: Low (already verified in LLD)

2. **Confirm job doesn't run on PRs** (Manual check)
   - Action: Check recent CI runs to confirm `build-plugin` shows as "skipped"
   - Why: Final safety check before removal
   - Dependencies: None
   - Risk: Low

### Phase 2: Remove Job Definition

3. **Remove Stage 3 header comment** (File: [.github/workflows/ci.yml](.github/workflows/ci.yml#L218-L222))
   - Action: Delete the "STAGE 3: Build" comment block
   - Lines to remove:
     ```yaml
     # ═══════════════════════════════════════════════════════════════════════════
     # STAGE 3: Build (main branch only, after tests pass)
     # ═══════════════════════════════════════════════════════════════════════════
     ```
   - Why: No remaining jobs in "Stage 3" after removal
   - Dependencies: None
   - Risk: Low

4. **Remove `build-plugin` job** (File: [.github/workflows/ci.yml](.github/workflows/ci.yml#L224-L272))
   - Action: Delete entire `build-plugin` job definition
   - Lines to remove: 224-272 (including blank line before job)
   - Why: Job never executes; dead code
   - Dependencies: Step 3
   - Risk: Low (job never runs anyway)

### Phase 3: Update Documentation

5. **Update workflow diagram** (File: [docs/guides/ci-pipeline.md](docs/guides/ci-pipeline.md#L12-L40))
   - Action: Remove `build-plugin` box from diagram, simplify flow
   - Why: Diagram should reflect actual workflow
   - Dependencies: Step 4
   - Risk: Low

6. **Remove build-plugin from Jobs table** (File: [docs/guides/ci-pipeline.md](docs/guides/ci-pipeline.md#L50-L58))
   - Action: Delete `build-plugin` row from Engine Pipeline table
   - Why: Job no longer exists
   - Dependencies: Step 5
   - Risk: Low

7. **Update Design Principles section** (File: [docs/guides/ci-pipeline.md](docs/guides/ci-pipeline.md))
   - Action: Review "Artifact Sharing" section; clarify ui-dist is still shared but engine-target note about build-plugin is obsolete
   - Why: Reflects accurate artifact usage
   - Dependencies: Step 6
   - Risk: Low

8. **Update Artifacts section** (File: [docs/guides/ci-pipeline.md](docs/guides/ci-pipeline.md#L94-L106))
   - Action: Remove "Release Artifacts" section (ad-hoc signed artifacts no longer produced by CI)
   - Why: CI no longer produces plugin artifacts; release workflow handles this
   - Dependencies: Step 7
   - Risk: Low

9. **Update Local Testing table** (File: [docs/guides/ci-pipeline.md](docs/guides/ci-pipeline.md#L225-L235))
   - Action: Remove `build-plugin` row from "What Can Be Tested Locally" table
   - Why: Job no longer exists
   - Dependencies: Step 8
   - Risk: Low

### Phase 4: Validation

10. **Verify YAML syntax** (Manual validation)
    - Action: Run `act --list` or let GitHub validate on push
    - Why: Ensure workflow YAML is valid after edits
    - Dependencies: Step 4
    - Risk: Low

11. **Create test PR** (Manual validation)
    - Action: Push changes to feature branch, open PR
    - Why: Verify CI runs successfully without build-plugin
    - Dependencies: Steps 1-9
    - Risk: Low

12. **Verify all checks pass** (Manual validation)
    - Action: Confirm all validation jobs run: check-docs, check-ui, test-ui, prepare-engine, check-engine, test-engine
    - Why: Ensure no regressions in validation pipeline
    - Dependencies: Step 11
    - Risk: Low

## Testing Strategy

### Automated Validation
- CI workflow runs on PR (all lint + test jobs)
- No dedicated tests needed (removing dead code)

### Manual Validation
- Review GitHub Actions run to confirm:
  - All 6 jobs execute (check-docs, check-ui, test-ui, prepare-engine, check-engine, test-engine)
  - No job dependency errors
  - No reference to missing `build-plugin` job

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| macOS build issues caught later | Low | Medium | Release workflow runs on macOS; template validation on macOS |
| Accidental removal of other jobs | Very Low | High | Review diff carefully; job has unique name |
| Documentation inconsistency | Low | Low | Update all references in single phase |

## Success Criteria

- [ ] `build-plugin` job removed from `.github/workflows/ci.yml`
- [ ] Stage 3 comment block removed
- [ ] CI workflow diagram updated in `docs/guides/ci-pipeline.md`
- [ ] All job table references updated
- [ ] CI runs successfully on PR (all 6 validation jobs pass)
- [ ] No job dependency warnings in GitHub Actions UI

## Estimated Effort

| Phase | Estimated Time |
|-------|----------------|
| Phase 1: Verification | 5 minutes |
| Phase 2: Remove Job | 5 minutes |
| Phase 3: Update Documentation | 15 minutes |
| Phase 4: Validation | 10 minutes (+ CI run time ~5 min) |
| **Total** | **~35 minutes** |

## Notes

- This is a low-risk change since the job being removed never executes
- All validation capabilities remain intact (lint, tests, type-checking)
- Plugin artifact generation moves exclusively to Release workflow (as architecturally intended)
