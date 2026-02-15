## Summary

Implement npm cohort lockstep versioning in CD so `@wavecraft/core` and `@wavecraft/components` publish against a single computed target version per run, with ordered/idempotent behavior and post-publish alignment verification. Also adds implementation/test documentation and completes archive move semantics by removing the duplicate non-archived feature folder.

## Changes

- **Build/Config**
  - Refactored `.github/workflows/continuous-deploy.yml` for cohort-based npm version computation and fan-out.
  - Added `publish-npm-cohort-prepare` orchestration and dependency-safe publish ordering (`core` before `components`).
  - Added rerun-safe/tag-safe guards and explicit registry-availability gating for npm publish jobs.
  - Preserved legacy manual-dispatch force inputs with deprecation warnings while keeping `force-publish-npm-cohort` canonical.

- **Documentation**
  - Updated `docs/guides/ci-pipeline.md`.
  - Updated `docs/architecture/versioning-and-distribution.md`.
  - Added/updated archived feature docs:
    - `docs/feature-specs/_archive/npm-cohort-lockstep-versioning/implementation-progress.md`
    - `docs/feature-specs/_archive/npm-cohort-lockstep-versioning/test-plan.md`
    - `docs/feature-specs/_archive/npm-cohort-lockstep-versioning/PR-summary.md`
  - Completed archive closure by removing duplicate source feature docs from:
    - `docs/feature-specs/npm-cohort-lockstep-versioning/`

- **Project Tracking**
  - Updated `docs/roadmap.md` on this branch as part of the existing change set.

## Commits

- `7a4448e` feat(docs): add implementation progress and test plan for npm cohort lockstep versioning
- `3259c5a` feat(ci): implement npm cohort lockstep versioning for synchronized package publishing

## Related Documentation

- [Implementation Progress](./implementation-progress.md)
- [Test Plan](./test-plan.md)

## Testing

- [x] Full repository validation passes: `cargo xtask ci-check --full`
- [x] Workflow logic reviewed for trigger semantics, ordering, idempotency, and verification behavior
- [x] Local shell simulation performed for cohort target computation branches

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No blocking CI issues in local full check (`cargo xtask ci-check --full`)
