## Summary

Fix CLI publishability by removing a git-only `nih_plug_xtask` dependency from the CLI dependency graph, ensuring the CLI can be packaged/published without pulling non-publishable git-only crates.

## Changes

- **Build/Config**
  - Updated `cli/Cargo.toml` dependency wiring to avoid git-only `nih_plug_xtask` in publish-facing paths.
  - Regenerated `cli/Cargo.lock` to reflect the corrected dependency graph.
- **CLI Command Logic**
  - Updated `cli/src/commands/bundle_command.rs` to align bundle behavior with the new dependency setup.
- **Tests**
  - Updated `cli/tests/bundle_command.rs` for deterministic coverage of the adjusted bundle/publish behavior.

## Commits

- `ed91002` Fix CLI publishability by removing git-only nih_plug_xtask dep

## Related Documentation

- Feature folder: `docs/feature-specs/publish-cli-nih-plug-xtask-version/`

## Testing

- [x] Commit-level validation via automated CLI bundle test updates in `cli/tests/bundle_command.rs`
- [ ] Build passes: `cargo xtask build`
- [ ] Linting passes: `cargo xtask lint`
- [ ] Tests pass: `cargo xtask test`

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated (`PR-summary.md`)
- [ ] No linting errors (`cargo xtask lint`)
