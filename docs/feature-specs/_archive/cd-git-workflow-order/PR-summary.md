## Summary

Fixes the `GH013` error in the Continuous Deploy pipeline where `github-actions[bot]` attempted to push auto-bump commits directly to `main`, which is blocked by branch protection rulesets.

The fix changes the CD pipeline from a **commit-based** publishing model to a **tag-only** model: version bumps are committed locally (not pushed to `main`), and only git tags are pushed for each published version. Tags are not subject to branch protection rules.

## Changes

- **CI/CD**: Updated `.github/workflows/continuous-deploy.yml`
  - Replaced `git push origin main` with local-only commits + `git push origin <tag>`
  - Removed the `github-actions[bot]` author check from `detect-changes` (no longer needed since auto-bump commits are never pushed to `main`)
  - Added explicit `--no-verify` flag to git commits to skip hooks in CI
- **Documentation**: Updated `docs/guides/ci-pipeline.md`
  - Documented the tag-only publishing model
  - Updated the "Infinite Loop Prevention" section to reflect the new approach
- **Documentation**: Updated `docs/architecture/coding-standards.md`
  - Updated the SDK Distribution Versioning section to describe tag-only commits and branch protection rationale

## Commits

- `dda6776` fix: CD auto-bump uses tag-only publishing to avoid branch protection errors
- `1d0a099` docs: add feature spec for CD tag-only publishing fix

## Related Documentation

- [Implementation Plan](./implementation-plan.md)
- [Low-Level Design](./low-level-design-cd-git-workflow-order.md)
- [Implementation Progress](./implementation-progress.md)
- [Test Plan](./test-plan.md)

## Testing

- [x] `cargo xtask ci-check` passes (lint + tests)
- [x] Workflow YAML is valid (no syntax errors)
- [ ] CD pipeline runs successfully on merge (verify in GitHub Actions)

## Checklist

- [x] Code follows project coding standards
- [x] Documentation updated
- [x] No linting errors (`cargo xtask ci-check`)
