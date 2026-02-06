# Remove Redundant Build-Plugin Job from CI Workflow

## Summary

This PR removes the unused `build-plugin` job from the CI workflow (`.github/workflows/ci.yml`). The job was configured to run only on the `main` branch (`if: github.ref == 'refs/heads/main'`), but the workflow itself only triggers on pull requests to `main`. This meant the job never executed - it was dead code.

## Changes

### CI/CD
- **Removed** `build-plugin` job from `.github/workflows/ci.yml` (52 lines deleted)
- **Removed** "STAGE 3: Build" section header
- Workflow now runs 6 validation jobs only (check-docs, check-ui, test-ui, prepare-engine, check-engine, test-engine)

### Documentation
- **Updated** `docs/guides/ci-pipeline.md`:
  - Removed build-plugin from workflow diagrams
  - Removed build-plugin from jobs table
  - Removed "Release Artifacts" section (CI no longer produces plugin artifacts)
  - Removed build-plugin from "Local Testing" table

### Version
- **Bumped** all crate versions to 0.7.2 in workspace Cargo.toml

### Testing
- **Added** comprehensive test plan in `docs/feature-specs/ci-build-stage-removal/test-plan.md`

## Commits

```
9d1823a test: Add test plan for CI build stage removal
becac8b chore: Update crate versions to 0.7.2
2328033 feat: Remove redundant build-plugin job from CI workflow
1b62f69 feat: Add implementation plan and progress tracking for CI build stage removal
bf5e09b arch: Add low-level design for CI build stage removal
```

## Related Documentation

- **Low-Level Design**: [`docs/feature-specs/ci-build-stage-removal/low-level-design-ci-build-stage-removal.md`](../docs/feature-specs/ci-build-stage-removal/low-level-design-ci-build-stage-removal.md)
- **Implementation Plan**: [`docs/feature-specs/ci-build-stage-removal/implementation-plan.md`](../docs/feature-specs/ci-build-stage-removal/implementation-plan.md)
- **Implementation Progress**: [`docs/feature-specs/ci-build-stage-removal/implementation-progress.md`](../docs/feature-specs/ci-build-stage-removal/implementation-progress.md)
- **Test Plan**: [`docs/feature-specs/ci-build-stage-removal/test-plan.md`](../docs/feature-specs/ci-build-stage-removal/test-plan.md)

## Testing

### Pre-Validation Completed
- ✅ `cargo xtask check` passes (all lint + tests)
- ✅ No `build-plugin` or `STAGE 3` references in ci.yml
- ✅ No `build-plugin` references in ci-pipeline.md
- ✅ Version 0.7.2 correctly set in workspace

### Verification Required
- [ ] CI workflow runs successfully on this PR
- [ ] All 6 validation jobs execute (no errors)
- [ ] No `build-plugin` job appears in GitHub Actions UI
- [ ] Version badge shows "v0.7.2" when UI is running

## Breaking Changes

None. This is a pure cleanup with zero impact:
- The removed job never executed (conflicting trigger conditions)
- No other jobs depend on `build-plugin`
- Plugin builds happen in the `release` workflow, not CI

## Checklist

- [x] Code follows project style guidelines
- [x] All tests pass locally (`cargo xtask check`)
- [x] Documentation updated
- [x] Version bumped appropriately (0.7.1 → 0.7.2)
- [ ] CI passes on GitHub (will verify after PR creation)
- [ ] Manual testing completed (visual verification of UI version)
