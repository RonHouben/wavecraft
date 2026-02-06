## Summary

Fixes the CI publish failure where `cargo publish` failed because version 0.7.1 already exists on crates.io. The root cause was unreliable version detection using `cargo search` which has indexing delays.

## Changes

- **Build/Config**: 
  - `.github/workflows/continuous-deploy.yml` - Replaced `cargo search` with sparse index API for reliable version checking
  - `cli/Cargo.toml` - Bumped version from 0.7.1 to 0.7.2

## Commits

- `dfe0ba2` fix(ci): improve crates.io version detection and bump CLI to 0.7.2

## Testing

- [x] Workflow syntax validated
- [x] Sparse index API verified locally: `curl -s "https://index.crates.io/wa/ve/wavecraft" | tail -1 | jq -r '.vers'`

## Checklist

- [x] Code follows project coding standards
- [x] No linting errors
- [x] CI workflow logic is correct
