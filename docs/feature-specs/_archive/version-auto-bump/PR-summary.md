## Summary

This bugfix resolves continuous-deploy pipeline failures caused by local crate versions falling behind published versions on crates.io after branch protection prevented auto-bump commits from being pushed to main. The fix implements "Option A" logic (auto-bump when local is behind published) in all three Rust publish jobs (CLI, engine, dev-server), replacing the previous behavior of exiting with error code 1.

All affected crate versions have been bumped from 0.12.1 to 0.12.6 to align with the current published versions.

## Changes

- **Build/Config**: Updated `.github/workflows/continuous-deploy.yml` to auto-bump versions instead of failing when local version is behind published version (18 lines changed)
- **Engine**: Bumped all engine crate versions and workspace version from 0.12.1 to 0.12.6:
  - `engine/Cargo.toml` (workspace version)
  - `engine/crates/wavecraft-bridge/Cargo.toml`
  - `engine/crates/wavecraft-dsp/Cargo.toml`
  - `engine/crates/wavecraft-macros/Cargo.toml`
  - `engine/crates/wavecraft-metering/Cargo.toml`
  - `engine/crates/wavecraft-protocol/Cargo.toml`
- **Dev Server**: Bumped dev-server version from 0.12.1 to 0.12.6 in `dev-server/Cargo.toml`
- **Lockfiles**: Updated all Cargo.lock files (cli, dev-server, engine) to reflect new version dependencies

## Commits

```
d8dcc9c fix: implement auto-bump when local version behind published
```

## Related Documentation

This is a bugfix with no associated feature documentation (using lightweight workflow per [agent-development-flow.md](../../architecture/agent-development-flow.md#lightweight-workflow-for-bug-fixes)).

## Testing

- [x] Build passes: Verified locally with `cargo build`
- [x] Linting passes: `cargo fmt --check` and `cargo clippy` pass
- [x] Tests pass: All unit tests pass
- [x] CI verification: continuous-deploy workflow logic reviewed and tested with version comparison scenarios

**Verification method:**

- Reviewed the workflow changes to ensure auto-bump logic correctly handles `local < published` case
- Verified all version numbers are consistent across workspace and crates
- Confirmed Cargo.lock files are updated

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed (N/A for CI workflow fix)
- [x] Documentation updated (N/A for bugfix)
- [x] No linting errors
