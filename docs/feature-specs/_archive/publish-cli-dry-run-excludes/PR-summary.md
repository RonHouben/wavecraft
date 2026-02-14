## Summary

Fixes `publish-cli` dry-run failures in CD when `force-publish-cli` is triggered.

Root cause: the workflow stages `sdk-template/` into `cli/sdk-template/` before `cargo publish --dry-run`, but copied workspace manifests (`Cargo.toml`, `engine/Cargo.toml`, `engine/xtask/Cargo.toml`) can create packaging/dependency resolution issues during the dry-run path.

Fix: exclude those manifest files from the staging `rsync` step in `.github/workflows/continuous-deploy.yml`.

## Changes

- **Build/Config**
  - Updated `.github/workflows/continuous-deploy.yml` in `publish-cli` template staging:
    - `--exclude Cargo.toml`
    - `--exclude engine/Cargo.toml`
    - `--exclude engine/xtask/Cargo.toml`
- **Documentation**
  - Added this PR summary artifact at `docs/feature-specs/publish-cli-dry-run-excludes/PR-summary.md`.

## Commits

- `c8f0fb1` fix(ci): exclude Cargo.toml files from CLI publish process

## Related Documentation

- [Agent Development Flow](../../architecture/agent-development-flow.md)
- [Development Workflows](../../architecture/development-workflows.md)

## Testing

- Verified workflow diff includes only the intended `rsync` exclude additions for CLI template staging.
- Reasoning check: the excluded files are workspace-level manifests not required for packaging staged template content.
- No local runtime/unit tests were executed for this CI workflow-only change.

## Checklist

- [x] Code follows project coding standards
- [x] Documentation updated
- [x] Change scope is minimal and focused
- [ ] Full repo CI checks run locally (`cargo xtask ci-check --fix`)
