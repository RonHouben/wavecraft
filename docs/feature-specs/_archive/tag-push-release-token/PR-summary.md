## Summary

Fixes CD release tag push failures caused by insufficient workflow-related push permissions when using the default token. The workflow now uses `RELEASE_TAG_TOKEN` for tag pushes and hardens auth by using a per-command authenticated push remote (no persistent git URL rewrite).

## Changes

- **Build/Config**
  - Updated `.github/workflows/continuous-deploy.yml` to switch tag push authentication to `RELEASE_TAG_TOKEN`.
  - Hardened tag push auth to per-command `TAG_PUSH_REMOTE` usage instead of persistent local git config URL rewrite.
- **Documentation**
  - Updated `docs/guides/ci-pipeline.md` to document `RELEASE_TAG_TOKEN` requirements and per-command authenticated push behavior.

## Commits

- `b35e18d` fix(continuous-deploy): use RELEASE_TAG_TOKEN for authenticated tag pushes

## Related Documentation

- [CI Pipeline Guide](../../guides/ci-pipeline.md)

## Testing

- [ ] Validate CD workflow YAML syntax and expression usage
- [ ] Verify no persistent git URL rewrite is used for tag pushes
- [ ] Verify release tag push uses `RELEASE_TAG_TOKEN`

## Checklist

- [x] Code follows project coding standards
- [x] Documentation updated
- [x] Scope limited to CD tag push auth fix + docs follow-up
- [ ] CI pipeline run confirms tag push succeeds in release flow
