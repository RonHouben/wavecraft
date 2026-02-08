## Summary

Follow-up fix for PR #38. The initial CD fix's validation run revealed two additional issues:

1. **Idempotency bug**: When `local == registry` version, the resolution logic treated it as "needs bump" instead of "skip". This caused unwanted publishes of `@wavecraft/core` 0.7.5 and CLI 0.8.5.

2. **Branch protection conflict**: The "Commit version bump" steps tried to `git push` directly to main, which is blocked by repository rules requiring PRs and status checks.

**Solution**: Switch to a **publish-only model** — CD only publishes when the local version is explicitly ahead of the registry. No auto-bumping, no commits to main. Developers bump versions in PRs.

## Changes

- **Build/Config** (`.github/workflows/continuous-deploy.yml`):
  - Replace auto-bump logic with skip-when-not-ahead for all 3 publish jobs (CLI, npm-core, npm-components)
  - Remove all "Commit version bump" steps — no more direct pushes to main
  - Condition build/publish/tag steps on `skip != 'true'` output
  - Remove "Pull latest changes" step from components job (no longer needed)
  - Sync `@wavecraft/core` version to 0.7.5 and CLI to 0.8.5 (match registry)

## Commits

- `b3f7f9c` fix(cd): switch to publish-only model, remove auto-bump commits

## Related Documentation

- [Implementation Progress](./implementation-progress.md) — Phase 5 added for this follow-up

## Testing

- [ ] Merge PR and trigger `workflow_dispatch` → all jobs skip (versions match registry)
- [ ] Re-trigger → still skips (idempotent)
- [ ] Bump a version in a new PR → CD publishes only the bumped package

## Checklist

- [x] Code follows project coding standards
- [x] Documentation updated (implementation-progress.md Phase 5)
- [x] No linting errors (workflow YAML only)
