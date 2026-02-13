## Summary

Refactor canonical sdk-template as single source of truth.

This change consolidates three overlapping plugin scaffold sources (`cli/sdk-templates/new-project/react/`, `engine/crates/wavecraft-example/`, and `ui/src/`) into a single canonical `sdk-template/` directory at the repository root. The canonical template now serves both as the CLI template source for `wavecraft create` and as the development target for `cargo xtask dev`.

## Changes

- **Canonical template foundation**
  - Created `sdk-template/` at repo root as the canonical plugin scaffold.
  - Switched CLI `include_dir!` embedding source to `../sdk-template`.
- **Developer workflow and SDK mode**
  - Added `scripts/setup-dev-template.sh` for dev-mode bootstrapping.
  - Rewired SDK-mode detection to `sdk-template/engine` and `sdk-template/ui`.
  - Added Vite aliases for SDK-mode package development.
- **Legacy path cleanup**
  - Deleted legacy scaffold paths:
    - `cli/sdk-templates/`
    - `engine/crates/wavecraft-example/`
    - `ui/src/`
  - Updated root `ui/` to operate as a pure npm package workspace.
- **CI, docs, and agent updates**
  - Updated CI workflows to align with canonical template layout.
  - Updated architecture/guides documentation and agent configuration references.

## Commits

- `ec96b62` docs: archive canonical-sdk-template feature spec and update roadmap
- `601b4ac` fix: add post-check validation to setup-dev-template.sh
- `5434a9d` fix: update stale ui/src references in docs and agent configs
- `ca73468` docs: update architecture and workflows for canonical sdk-template
- `9e25507` refactor: remove root ui app and keep ui as package workspace
- `5ae531f` refactor: migrate canonical sdk-template and sdk-mode detection
- `d601bd9` docs: add feature spec for canonical sdk-template migration

## Related Documentation

- [Implementation Plan](./implementation-plan.md)
- [Low-Level Design](./low-level-design-canonical-sdk-template.md)
- [Test Plan](./test-plan.md)
- [QA Report](./QA-report.md)

## Testing

- ✅ Automated checks pass: `cargo xtask ci-check`
- ✅ Manual smoke test passed:
  - `cargo xtask dev` starts WebSocket + Vite servers
  - Web UI loads and connects
  - HMR verified in `sdk-template/ui/src/App.tsx`
  - Engine hot-reload verified in `sdk-template/engine/src/lib.rs`
  - Package alias behavior verified in SDK mode
- ✅ Stale reference sweep clean for removed legacy paths

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No linting/test failures in validation run (`cargo xtask ci-check`)
