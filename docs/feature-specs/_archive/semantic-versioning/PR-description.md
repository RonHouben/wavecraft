# Semantic Versioning for VstKit

## Summary

Implements semantic versioning (SemVer) for VstKit plugins with a **single source of truth** in `engine/Cargo.toml`. The version automatically propagates to:
- Plugin metadata (VST3, CLAP, AU via nih-plug)
- React UI footer via build-time injection

**Bonus:** Also delivers browser development mode (partial Milestone 6) — UI now runs in browsers without IPC errors using environment detection and mock data.

## Changes

### Engine/Build System
- Added `toml` crate dependency to xtask
- Created `read_workspace_version()` helper function
- Updated `bundle` command to inject `VITE_APP_VERSION` env var during UI build
- Enhanced test coverage with descriptive `expect()` messages (46 tests)

### UI Components
- **New:** `VersionBadge.tsx` component — displays version in footer
- **New:** `environment.ts` — detects browser vs WKWebView environment
- Updated `IpcBridge.ts` — lazy initialization, graceful browser mode degradation
- Updated `hooks.ts` — environment-aware with mock data for browser mode
- Vite `define` block for `__APP_VERSION__` compile-time constant

### Documentation
- Updated `high-level-design.md` — versioning strategy, browser development mode
- Updated `coding-standards.md` — environment-aware hooks, build-time constants patterns
- Updated `roadmap.md` — Milestone 5 progress, partial Milestone 6 delivery
- Full feature spec (now archived): user stories, low-level design, implementation plan, test plan, QA report

## Commits

- `4a41ba3` feat: Implement semantic versioning across VstKit
- `1d5def3` feat: Enhance documentation with versioning strategy and environment-aware hooks
- `9580590` feat: Address QA findings and enhance documentation
- `4a00488` feat: Implement lazy IPC initialization and environment detection for browser compatibility
- `9aa22a6` feat: Complete implementation of semantic versioning with build-time version injection
- `09e69d3` feat: Add implementation plan and progress tracking
- `d5a5e91` feat: Add low-level design document
- `b0fb5b7` feat: Add user stories for semantic versioning

## Testing

### Automated Tests
- ✅ **35/35 UI unit tests passing** (including 10 new tests for VersionBadge, environment, IpcBridge)
- ✅ **46/46 xtask tests passing**
- ✅ **All linting checks passing** (ESLint, Prettier, Clippy, rustfmt)

### Manual Tests (8/8 passing)
- ✅ Dev mode fallback — `npm run dev` shows `vdev` in footer
- ✅ Production build — `cargo xtask bundle -f webview_editor` injects real version
- ✅ Plugin in DAW — footer shows `v0.1.0` in Ableton Live
- ✅ Version consistency — Cargo.toml matches UI display

## Related Documentation

- [Archived Feature Spec](docs/feature-specs/_archive/semantic-versioning/)
- [High-Level Design - Versioning](docs/architecture/high-level-design.md)
- [Coding Standards - Build-Time Constants](docs/architecture/coding-standards.md)

## Checklist

- [x] Code follows project coding standards
- [x] All tests passing (UI + Engine)
- [x] Linting clean (ESLint, Prettier, Clippy)
- [x] Documentation updated (architecture + feature specs)
- [x] QA review completed and approved
- [x] Feature spec archived
