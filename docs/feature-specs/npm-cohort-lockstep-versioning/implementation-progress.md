# Implementation Progress — npm cohort lockstep versioning

## Status

- ✅ Refactored `.github/workflows/continuous-deploy.yml` to use npm cohort lockstep publishing
- ✅ Added `publish-npm-cohort-prepare` job to compute one target version per run
- ✅ Updated npm publish order and dependency alignment policy (core -> components)
- ✅ Added idempotent publish/tag behavior for reruns and partial recovery
- ✅ Added post-publish cohort verification
- ✅ Added migration path for deprecated `workflow_dispatch` npm split force inputs
- ✅ Updated CI and versioning docs
- ✅ QA follow-up: removed silent npm lookup fallback, added retry-aware registry checks, and gated npm publish jobs on explicit registry availability

## Files touched

- `.github/workflows/continuous-deploy.yml`
- `docs/guides/ci-pipeline.md`
- `docs/architecture/versioning-and-distribution.md`

## Notes

- Lockstep cohort target is computed as the highest semver across local and published versions for both npm packages.
- npm registry lookup failures no longer silently default to `0.0.0`; prepare now marks registry state explicitly and blocks downstream npm publish jobs when availability is unknown.
- Legacy force inputs (`force-publish-npm-core`, `force-publish-npm-components`) are still accepted and emit a warning.
- Canonical force input is now `force-publish-npm-cohort`.
