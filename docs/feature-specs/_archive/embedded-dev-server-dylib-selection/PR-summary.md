## Summary

Improves embedded dev server reliability and developer workflow by fixing dylib selection, reusing shared engine crates for the CLI dev server, renaming the standalone crate to `wavecraft-dev-server`, and updating supporting docs/specs, roadmap, and QA artifacts.

## Changes

- **Engine/DSP**: Added `InMemoryParameterHost` + `PluginParamLoader` in `wavecraft-bridge`, moved synthetic meter generator to `wavecraft-metering`, added parameter FFI exports, updated dev server crate naming, and refreshed protocol/macro integrations.
- **UI**: Updated WebSocket transport comments to align with dev server naming.
- **Build/Config**: Cargo workspace and lockfiles updated for new crates/versions; CLI and xtask wiring adjusted; CI workflow tweaks applied.
- **Documentation**: Added/updated feature specs, test plans, QA reports, and roadmap/backlog entries; archived legacy specs where applicable.

## Commits

- 1d1013a fix: clean up backlog by removing completed SDK publication and Apple Developer Account items
- c764fab fix: document renaming of standalone crate to wavecraft-dev-server in roadmap
- 795b294 fix: add CLI `-v`/`--version` flag to verify installed Wavecraft CLI version
- da31f65 fix: update roadmap with CLI dev server reuse details and related improvements
- 655f79c fix: rename standalone crate to wavecraft-dev-server and update related references in code and documentation
- 70ccc18 fix: update references to standalone dev server to wavecraft-dev-server in documentation and code
- d7481c5 fix: rename standalone to wavecraft-dev-server and update related references in documentation
- 69bcafc fix: rename standalone to wavecraft-dev-server and update related documentation
- 600df3d feat: Implement standalone Wavecraft desktop application
- 3e613a2 fix: update test plan results and implementation progress for crate rename
- 8256e25 Refactor CLI Dev Server and Metering Utilities
- dd305fa fix: update dylib selection logic to parse Cargo.toml and prefer [lib].name
- d41657f feat(docs): add QA report for embedded dev server with findings and automated check results
- f75c083 refactor: enhance dylib selection logic and update test plan results
- 528ae62 refactor: improve path handling and library selection logic in dev server
- 8c8d4d3 feat(docs): add implementation and test plans for embedded dev server
- 64aedb7 refactor: clean up code formatting and improve error handling in dev server
- 672535e Refactor internal testing user stories and update dependencies
- 71a1672 feat(docs): add implementation plan and progress tracker for renaming `standalone` to `wavecraft-dev-server`
- bd2c99f feat(agents): remove model reference from PO, architect, and planner agents; update handoff labels
- f78a360 feat(docs): add low-level design for renaming `standalone` to `wavecraft-dev-server`
- a88467c Refactor embedded dev server implementation to reuse existing WebSocket server
- 8631a03 feat(cli): add implementation plan and progress tracking for embedded dev server
- 924bb6e feat(cli): add low-level design documentation for embedded dev server with plugin parameter discovery
- 3aa9445 feat(cli): add QA report for `wavecraft start` command implementation
- e963a35 feat(cli): add `wavecraft new` command for project creation with template support
- fd218fc feat(cli): introduce `wavecraft start` command for improved development experience

## Related Documentation

- [Embedded Dev Server Spec](../embedded-dev-server/low-level-design-embedded-dev-server.md)
- [Embedded Dev Server Test Plan](../embedded-dev-server/test-plan.md)
- [Standalone Rename Spec](../standalone-rename/low-level-design-standalone-rename.md)
- [Standalone Rename Test Plan](../standalone-rename/test-plan.md)

## Testing

- [x] `cargo xtask check` (engine lint + tests + UI tests)
- [x] `cargo run -p wavecraft-dev-server -- --help`
- [x] `cargo run --manifest-path engine/xtask/Cargo.toml -- dev` (smoke test)
- [x] `wavecraft start --install --port 9010 --ui-port 5174 --verbose`
- [x] `wavecraft start --no-install` (expected error when `ui/node_modules` missing)

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No linting errors (`cargo xtask check`)
