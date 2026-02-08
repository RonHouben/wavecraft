## Summary

Fixes CLI `wavecraft start` failures when `ui/dist` is missing by embedding fallback UI assets in the SDK, and improves dev server startup with preflight port checks and strict UI port binding. Updates CLI/engine versions, expands documentation, and adds test/QA artifacts.

## Changes

- **Engine/DSP**: Embedded fallback UI assets and updated `include_dir!` path for `wavecraft-nih_plug`.
- **CLI**: Added preflight port checks, strict UI port binding, and improved startup/shutdown behavior in `wavecraft start`.
- **Build/Config**: Version bumps across CLI/engine crates and lockfiles.
- **Documentation**: Updated guides, coding standards, agent flow, high-level design, roadmap, test plan, and QA report.

## Commits

- d6ccb35 fix: enhance CLI `wavecraft start` with preflight port checks and strict UI port binding
- 37b8a83 fix: enhance dev server startup behavior for CLI `wavecraft start`
- 41b14ae fix: add QA report for CLI `start` UI asset fallback and dev server startup
- 11ff104 fix: update test plan dates and re-validation notes for CLI `start` UI asset fallback
- 06d4349 fix: update CLI and engine crate versions to 0.8.4 and 0.7.4, add fallback UI asset testing guidelines
- 5cb5e93 fix: add fallback UI assets and update asset paths in the plugin

## Related Documentation

- [Test Plan](./test-plan.md)
- [QA Report](./QA-report.md)

## Testing

- [x] Linting + tests: `cargo xtask ci-check` (run from `engine/`)
- [x] CLI tests: `cargo test --manifest-path cli/Cargo.toml`
- [x] `wavecraft-nih_plug` tests: `cargo test -p wavecraft-nih_plug --lib`
- [x] Manual: `wavecraft start` port-in-use fail-fast + strict port binding (TC-004/TC-005)

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No linting errors (`cargo xtask ci-check`)
