## Summary

Fix `cli/build.rs` path resolution so template staging works in both workspace and published crate layouts during `publish-cli` dry-runs.

The build script now:
- resolves `sdk-template` from `../sdk-template` (workspace/development), or
- falls back to `sdk-template` (packaged tarball layout),
- and declares both paths with `rerun-if-changed` so Cargo rebuild behavior is correct in either environment.

## Changes

- **Build/Config**:
  - Updated `cli/build.rs` to use `resolve_template_source()`.
  - Added support for both workspace and packaged template locations.
  - Added `cargo:rerun-if-changed=sdk-template` in addition to the existing workspace path.
  - Kept CI workflow exclusions unchanged.

## Commits

- `b3dcd66` fix(cli): resolve sdk-template path in build script for workspace and packaged layouts

## Related Documentation

- No feature-specific implementation-plan/LLD docs for this bugfix.

## Testing

- [ ] Build passes: `cargo xtask build`
- [ ] Linting passes: `cargo xtask lint`
- [ ] Tests pass: `cargo xtask test`

## Checklist

- [x] Code follows project coding standards
- [x] Documentation updated (`PR-summary.md`)
- [x] No changes to `.github/workflows/continuous-deploy.yml` excludes
- [ ] Full repo checks run (`cargo xtask ci-check --fix`)
