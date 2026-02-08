## Summary

Fix the Continuous Deployment pipeline that was failing to publish npm packages (`@wavecraft/core`, `@wavecraft/components`) due to version drift between the git repository and the npm registry. The root cause was that CD-created version bump commits never landed in git, causing subsequent runs to compute an already-published version.

## Changes

- **Build/Config**:
  - Replace naive single-patch-bump strategy with registry-aware version resolution for all publish jobs (npm + CLI)
  - npm jobs now query `npm view` for the latest published version and compute the next version from there
  - CLI job now queries the crates.io sparse index API for the latest published version
  - Add top-level `concurrency` group to prevent parallel CD runs from racing
  - Fix `publish-npm-components` conditional to run independently when core changes weren't needed
  - Bump `@wavecraft/core` and `@wavecraft/components` package.json versions to `0.7.4` (past npm's `0.7.3`)

- **Documentation**:
  - Add implementation plan documenting root cause analysis and 4-phase fix strategy
  - Add implementation progress tracker

## Commits

- `676d7f4` fix(cd): registry-aware version resolution for npm and crates.io publishing

## Related Documentation

- [Implementation Plan](./implementation-plan.md)
- [Implementation Progress](./implementation-progress.md)

## Testing

- [ ] CI pipeline passes (lint, tests)
- [ ] Trigger `workflow_dispatch` on main → all CD jobs pass or correctly skip
- [ ] Re-run `workflow_dispatch` → publish jobs skip (idempotent, versions already published)
- [ ] Verify npm versions are `>= 0.7.4` after successful run

## Checklist

- [x] Code follows project coding standards
- [x] Documentation updated
- [x] No linting errors (workflow YAML only, no source code changes)
