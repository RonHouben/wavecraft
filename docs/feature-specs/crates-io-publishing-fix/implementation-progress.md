# Implementation Progress — crates.io Publishing Fix

## Status

- [x] Add versioned path dependencies in `cli/Cargo.toml`
- [x] Make `wavecraft-dev-server` publishable
- [x] Add CLI publish preflight (`cargo publish --dry-run`) in workflow
- [x] Add workflow guardrails for unpublished deps
- [x] Run local CLI publish dry‑run
- [ ] Validate CI `publish-cli` job success

## Notes

- Published `wavecraft-dev-server` v0.7.3 to crates.io.
- `cargo publish --manifest-path cli/Cargo.toml --dry-run --allow-dirty` now succeeds.
