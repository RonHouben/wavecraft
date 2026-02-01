## Summary

Remove the `webview_editor` feature flag and make React UI the default (and only) plugin editor implementation. This simplifies the codebase by removing conditional compilation and the legacy egui fallback editor.

**Key Changes:**
- Deleted egui fallback editor (`editor/egui.rs`)
- Removed `webview_editor` feature flag from all Cargo configs
- Simplified build commands — React UI builds by default
- Updated CI workflows to remove feature flag arguments
- Bumped version to 0.2.0

## Changes

### Engine/DSP
- Removed `webview_editor` feature from `engine/crates/plugin/Cargo.toml`
- Removed `nih_plug_egui` dependency (no longer needed)
- Deleted `engine/crates/plugin/src/editor/egui.rs` (legacy fallback)
- Removed `#[cfg(feature = "webview_editor")]` conditionals from:
  - `lib.rs` — Editor selection
  - `editor/mod.rs` — Module exports
  - `editor/assets.rs`, `editor/bridge.rs`, `editor/macos.rs`, `editor/webview.rs`
- Added justification comments to `#[allow(dead_code)]` annotations

### Build System
- `engine/xtask/src/commands/bundle.rs` — Always build React UI (removed feature check)
- `engine/xtask/src/commands/release.rs` — Removed feature flag argument
- `.github/workflows/ci.yml` — Removed `--features webview_editor`
- `.github/workflows/release.yml` — Removed `--features webview_editor`

### Documentation
- `README.md` — Removed feature flag section, simplified build commands
- `docs/architecture/high-level-design.md` — Removed feature flags section
- `docs/guides/macos-signing.md` — Removed feature flag from commands
- `docs/architecture/coding-standards.md` — Added versioning guidelines
- `.github/agents/coder.agent.md` — Added version bumping workflow
- `docs/roadmap.md` — Updated task status, added resize handle visibility task

## Commits

- `2871de8` docs: add user stories for React UI default feature
- `06b4f1c` docs: add low-level design for React UI default feature
- `c55ce50` docs: add implementation plan and progress
- `17efa96` feat: make React UI the default by removing webview_editor feature flag
- `3cf06f9` feat: update version to 0.2.0; add versioning guidelines
- `24537f0` feat: update roadmap with status and resize handle task
- `ef729e7` style: add justification comment to dead_code annotation
- `26bfe78` feat: update tester role to include local CI pipeline

## Related Documentation

- [User Stories](./user-stories.md)
- [Low-Level Design](./low-level-design-react-ui-default.md)
- [Implementation Plan](./implementation-plan.md)
- [Implementation Progress](./implementation-progress.md)
- [Test Plan](./test-plan.md)
- [QA Report](./QA-report.md)

## Testing

- [x] Build passes: `cargo xtask bundle` (no feature flags)
- [x] Build passes: `cargo xtask bundle --release`
- [x] Linting passes: `cargo xtask lint`
- [x] UI tests pass: `cargo xtask test --ui` (35 tests)
- [x] Engine tests pass: `cargo xtask test --engine` (8 tests)
- [x] TypeScript passes: `npm run typecheck`
- [x] Manual verification: Plugin loads in Ableton Live with React UI
- [x] Version badge displays v0.2.0

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No linting errors (`cargo xtask lint`)
- [x] QA review passed
