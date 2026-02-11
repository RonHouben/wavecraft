# PR Summary: Dev Server Unification and Hot-Reload Enhancements

## Summary

This PR completes the **Dev Server Unification** milestone and adds comprehensive **Rust hot-reload** capabilities:

- **Created a unified `dev-server/` crate** at the repository root, merging the CLI's `dev_server` module and the engine's `wavecraft-dev-server` crate
- **Eliminated ~1,100 LOC of code duplication**, clarified ownership, and established a clean builder API for the `wavecraft start` command
- **Implemented Rust hot-reload functionality** with parameter preservation and WebSocket-based UI synchronization
- **Added subprocess-based parameter extraction** to resolve `dlopen` hangs during plugin hot-reload
- **Preserved feature-gated audio support** with the `audio` feature flag for FFI-based audio processing
- **Completed comprehensive testing** with 17/17 test cases passing, ensuring no breaking changes to the public API
- **Updated agent configurations** with improved model selections and enhanced QA processes
- **Refactored documentation** for better clarity and consistency across architecture and SDK docs

## Changes

### Dev Server Architecture (`dev-server/`)
- Created new standalone `dev-server/` crate with clean builder API
- Moved WebSocket server, hot-reload, audio server, and session management into unified location
- Implemented `DevServerHost` trait for dependency injection
- Added feature-gated audio support (`audio` feature flag)
- Removed duplicate `wavecraft-dev-server` crate from `engine/crates/`
- Removed duplicate `dev_server` module from CLI

**Files:**
- `dev-server/Cargo.toml` (new)
- `dev-server/src/lib.rs`, `host.rs`, `session.rs` (new)
- `dev-server/src/ws/mod.rs` (unified WebSocket server)
- `dev-server/src/audio/` (feature-gated: `server.rs`, `ffi_processor.rs`, `atomic_params.rs`)
- `dev-server/src/reload/` (`mod.rs`, `watcher.rs`, `rebuild.rs`, `guard.rs`)
- `engine/crates/wavecraft-dev-server/` (removed ~1,100 LOC)
- `cli/src/dev_server/mod.rs` (removed)

### CLI Updates (`cli/`)
- Refactored `cli/src/commands/start.rs` to use new `dev-server` crate builder API
- Added `extract-params` hidden subcommand for subprocess-based parameter discovery
- Implemented `project/param_extract.rs` for subprocess parameter extraction
- Added `project/dylib.rs` for plugin dylib management and loading
- Feature-gated audio dependencies with `audio` feature flag
- Updated `Cargo.toml` dependencies to use new `dev-server` crate

**Files:**
- `cli/src/commands/start.rs` (445 line refactor)
- `cli/src/commands/extract_params.rs` (new, 73 lines)
- `cli/src/project/param_extract.rs` (new, 126 lines)
- `cli/src/project/dylib.rs` (new, 153 lines)
- `cli/Cargo.toml` (dependency updates, audio feature gating)

### Hot-Reload Enhancements (`engine/crates/`)
- Added `load_params_only()` to `PluginLoader` for optimized parameter retrieval
- Implemented parameter preservation in `InMemoryParameterHost` during hot-reload
- Added sidecar cache updates after hot-reload for faster subsequent starts
- Enhanced rebuild diagnostics and error reporting

**Files:**
- `engine/crates/wavecraft-bridge/src/plugin_loader.rs` (+51 lines)
- `engine/crates/wavecraft-bridge/src/in_memory_host.rs` (parameter preservation)
- `engine/crates/wavecraft-bridge/src/host.rs` (+28 lines)

### UI Updates (`ui/`)
- Enhanced `useAllParameters` hook to react to hot-reload notifications
- Added unique keys to parameter groups for efficient re-rendering
- Implemented parameter re-fetch on hot-reload events

**Files:**
- `ui/packages/core/src/hooks/useAllParameters.ts` (+10 lines)
- `ui/packages/core/src/hooks/useAllParameters.test.ts` (+47 lines)
- `ui/src/App.tsx` (key management)

### Agent Configuration (`.github/agents/`)
- Updated all agent configurations with Claude Sonnet 4.5 and GPT-5.2-Codex
- Standardized research rules for Search agent delegation
- Enhanced documentation delegation patterns
- Improved table formatting and clarity
- Added CLI testing guidelines to Coder agent

**Files:**
- `.github/agents/QA.agent.md` (76 line update)
- `.github/agents/coder.agent.md` (92 line update)
- `.github/agents/tester.agent.md` (91 line update)
- `.github/agents/architect.agent.md`, `planner.agent.md`, `po.agent.md`, etc.

### Documentation Updates (`docs/`)
- Refactored high-level design document with clearer diagrams and structure
- Enhanced SDK architecture documentation with dev-server details
- Updated development workflows for browser dev mode
- Added Rust coding standards for real-time safety
- Archived dev-server-unification feature specs with implementation plan and test plan
- Created comprehensive documentation for subprocess parameter extraction
- Added hot-reload investigation reports and bugfix plans

**Files:**
- `docs/architecture/high-level-design.md` (394 line refactor)
- `docs/architecture/sdk-architecture.md` (+46 lines)
- `docs/architecture/development-workflows.md` (+101 lines)
- `docs/architecture/coding-standards-rust.md` (+65 lines)
- `docs/feature-specs/_archive/dev-server-unification/` (implementation-plan.md, test-plan.md)
- `docs/feature-specs/_archive/rust-hot-reload/` (implementation-plan.md, test-plan.md, etc.)
- `docs/roadmap.md` (milestone tracking updates)

### Build & CI (`.github/workflows/`, `Cargo.toml`)
- Updated CI/CD workflows for new crate structure
- Version bumps to 0.12.1 across all wavecraft crates
- Feature flag management for audio dependencies
- Updated Cargo.lock files for dependency resolution

**Files:**
- `.github/workflows/continuous-deploy.yml` (77 line update)
- `engine/Cargo.toml` (version and workspace updates)
- `cli/Cargo.toml` (dev-server dependency)
- `engine/Cargo.lock`, `cli/Cargo.lock`, `dev-server/Cargo.lock`

## Commits

```
cf7880f feat: unify dev server architecture and enhance testing framework
f13a592 Refactor documentation for high-level design and SDK architecture
95e34d2 fix: remove unused parking_lot dependency and update WebSocket client handling
d6fafbf fix: update test plan to reflect resolution of deprecated test helper function issue
39688e3 fix: enable feature gating for audio dependencies in CLI
9d3e07f docs: cleaned feature specs up
5100652 fix: update test commands to use cargo_bin! macro for consistency
9f42f66 Remove wavecraft-dev-server module and related files
414a539 feat: add implementation plan for dev server unification
b118a26 Refactor agent documentation for clarity and consistency
dfb8a3b feat: implement subprocess-based parameter extraction for hot-reload
9206c91 feat: Update agent models and improve QA processes
4de4aed feat: add Rust hot-reload functionality for development mode
```

## Related Documentation

- [Implementation Plan](./implementation-plan.md) — Dev server unification architecture and migration strategy
- [Test Plan](./test-plan.md) — Comprehensive test cases (17/17 passed)
- [Subprocess Parameter Extraction Design](../rust-hot-reload/subprocess-parameter-extraction-design.md)
- [Hot-Reload Implementation Plan](../rust-hot-reload/implementation-plan.md)
- [Development Workflows](../../architecture/development-workflows.md) — Browser dev mode documentation
- [SDK Architecture](../../architecture/sdk-architecture.md) — Updated with dev-server details
- [High-Level Design](../../architecture/high-level-design.md) — Refactored architecture overview

## Testing

- ✅ **Build passes**: `cargo xtask build` (all crates including new `dev-server/`)
- ✅ **Linting passes**: `cargo xtask lint` (Rust + TypeScript)
- ✅ **Tests pass**: `cargo xtask test` (engine + UI)
- ✅ **Integration tests**: 17/17 test cases passed (documented in test-plan.md)
- ✅ **Manual verification**:
  - `wavecraft start` launches successfully with new dev-server
  - Hot-reload preserves parameter values
  - WebSocket communication works correctly
  - Parameter changes sync between UI and engine
  - Audio feature flag works correctly
  - Subprocess parameter extraction resolves dlopen hangs
- ✅ **QA approved**: All quality checks passed (no linting errors, clean architecture)

## Checklist

- ✅ Code follows project coding standards
- ✅ Tests added/updated as needed (17 comprehensive test cases)
- ✅ Documentation updated (architecture, SDK, development workflows)
- ✅ No linting errors (`cargo xtask lint` passes)
- ✅ Version bumped to 0.12.1
- ✅ Feature specs archived to `_archive/dev-server-unification/`
- ✅ Roadmap updated with milestone completion
- ✅ No breaking changes to public API
- ✅ ~1,100 LOC removed through deduplication

## Impact

**Lines of Code:**
- Added: 11,926 insertions
- Removed: 6,586 deletions
- Net change: +5,340 lines (includes new dev-server crate, hot-reload infrastructure, and comprehensive documentation)
- Code duplication eliminated: ~1,100 LOC

**Files Changed:** 96 files
- 39 new files (dev-server crate, parameter extraction, test plans, documentation)
- 13 deleted files (duplicate wavecraft-dev-server crate)
- 44 modified files (CLI refactor, agent updates, documentation improvements)

**Key Improvements:**
- Cleaner architecture with single source of truth for dev server logic
- More maintainable codebase through elimination of duplication
- Better testability with dependency injection via `DevServerHost` trait
- Enhanced developer experience with hot-reload parameter preservation
- Resolved critical dlopen hang during hot-reload via subprocess extraction
