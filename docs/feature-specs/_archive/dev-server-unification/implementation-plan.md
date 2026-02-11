# Implementation Plan: Dev Server Unification

## Overview

Unify the CLI's `dev_server` module with the `wavecraft-dev-server` engine crate into a single top-level `dev-server/` crate. This eliminates code duplication, clarifies ownership, and creates a clean public API for the `wavecraft start` command.

**Current state:**

- **CLI dev_server module** (`cli/src/dev_server/`): 5 files, ~1,100 LOC — orchestration layer (DevServerHost, DevSession, FileWatcher, RebuildPipeline, BuildGuard)
- **Engine crate** (`engine/crates/wavecraft-dev-server/`): 9 files, ~1,200 LOC — infrastructure primitives (WsServer, AudioServer, FfiProcessor, AtomicParameterBridge) + standalone binary (main.rs, app.rs, webview.rs, assets.rs)

**Target state:**

- Single `dev-server/` crate at repository root
- Clean builder API for CLI integration
- Standalone binary removed (superseded by `wavecraft start`)
- Feature flag structure preserved (`audio` → audio I/O capabilities)

**Branch strategy:**

- Current branch: `fix/hotreloading` (active hot-reload fixes)
- This plan: NEW branch `feat/dev-server-unification` (create from `main` after hot-reload merge)

---

## Target Structure

```
wavecraft/
├── dev-server/              # NEW: unified dev server crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # Public API (DevServer builder)
│       ├── builder.rs       # Builder pattern for DevServer
│       ├── session.rs       # DevSession lifecycle
│       ├── host.rs          # DevServerHost (ParameterHost impl)
│       ├── ws/              # WebSocket server
│       │   └── mod.rs       # WsServer, WsHandle
│       ├── audio/           # Audio processing (feature-gated)
│       │   ├── mod.rs
│       │   ├── server.rs    # AudioServer, AudioHandle
│       │   ├── ffi_processor.rs  # FfiProcessor (C-ABI wrapper)
│       │   └── atomic_params.rs  # AtomicParameterBridge
│       └── reload/          # Hot-reload system
│           ├── mod.rs
│           ├── watcher.rs   # FileWatcher
│           ├── rebuild.rs   # RebuildPipeline
│           └── guard.rs     # BuildGuard
├── cli/
│   └── src/
│       ├── dev_server/      # REMOVED
│       └── commands/
│           └── start.rs     # Thin wrapper (~200-250 lines, down from ~610)
├── engine/crates/
│   └── wavecraft-dev-server/  # REMOVED
```

---

## Phase 1: Create `dev-server/` Crate Skeleton

**Goal:** Establish new crate structure and configure workspace membership

**Estimated time:** 1-2 hours | **Risk:** Low

### Tasks

1. Create `dev-server/` directory at repository root with subdirectories (`src/`, `src/ws/`, `src/audio/`, `src/reload/`)

2. Create `dev-server/Cargo.toml` with merged dependencies from both sources:
   - Core: `wavecraft-bridge`, `wavecraft-protocol`
   - WebSocket: `tokio`, `tokio-tungstenite`, `futures-util`
   - File watching: `notify`, `notify-debouncer-full`
   - Process management: `command-group`, `nix` (unix)
   - Concurrency: `parking_lot`, `atomic_float`
   - Audio (optional): `cpal`, `rtrb`
   - Features: `default = ["audio"]`, `audio = ["cpal", "rtrb"]`

3. Add `dev-server` to workspace members (root or engine `Cargo.toml`)

4. Create `dev-server/src/lib.rs` skeleton with empty module declarations

### Testing Checkpoint

- [ ] `cargo build -p dev-server` succeeds (empty modules)
- [ ] Workspace recognizes new crate

---

## Phase 2: Migrate Infrastructure from Engine Crate

**Goal:** Move WebSocket server, audio processing, and atomic params from `engine/crates/wavecraft-dev-server/` to `dev-server/`

**Estimated time:** 4-6 hours | **Risk:** Low-Medium

### File Migration Map

| Source                                                    | Destination                             | LOC  |
| --------------------------------------------------------- | --------------------------------------- | ---- |
| `engine/crates/wavecraft-dev-server/src/ws_server.rs`     | `dev-server/src/ws/mod.rs`              | ~311 |
| `engine/crates/wavecraft-dev-server/src/audio_server.rs`  | `dev-server/src/audio/server.rs`        | ~329 |
| `engine/crates/wavecraft-dev-server/src/ffi_processor.rs` | `dev-server/src/audio/ffi_processor.rs` | ~251 |
| `engine/crates/wavecraft-dev-server/src/atomic_params.rs` | `dev-server/src/audio/atomic_params.rs` | ~150 |

### Import Updates

- `crate::atomic_params::*` → `crate::audio::atomic_params::*`
- `crate::ffi_processor::*` → `crate::audio::ffi_processor::*`
- All audio modules guarded with `#[cfg(feature = "audio")]`

### Testing Checkpoint

- [ ] `cargo build -p dev-server` succeeds
- [ ] `cargo build -p dev-server --no-default-features` succeeds (audio disabled)
- [ ] `cargo test -p dev-server` passes (existing tests from engine crate)

---

## Phase 3: Migrate Orchestration from CLI

**Goal:** Move hot-reload system and DevServerHost from `cli/src/dev_server/` to `dev-server/`

**Estimated time:** 8-10 hours | **Risk:** Medium

### File Migration Map

| Source                                            | Destination                        | LOC  |
| ------------------------------------------------- | ---------------------------------- | ---- |
| `cli/src/dev_server/host.rs`                      | `dev-server/src/host.rs`           | ~207 |
| `cli/src/dev_server/rebuild.rs` (BuildGuard)      | `dev-server/src/reload/guard.rs`   | ~45  |
| `cli/src/dev_server/rebuild.rs` (RebuildPipeline) | `dev-server/src/reload/rebuild.rs` | ~430 |
| `cli/src/dev_server/watcher.rs`                   | `dev-server/src/reload/watcher.rs` | ~205 |
| `cli/src/dev_server/session.rs`                   | `dev-server/src/session.rs`        | ~150 |

### Import Updates

- `super::host::DevServerHost` → `crate::host::DevServerHost`
- `wavecraft_dev_server::ws_server::WsServer` → `crate::ws::WsServer`
- `#[cfg(feature = "audio-dev")]` → `#[cfg(feature = "audio")]`
- `wavecraft_dev_server::audio_server::AudioHandle` → `crate::audio::AudioHandle`
- `wavecraft_dev_server::atomic_params::AtomicParameterBridge` → `crate::audio::AtomicParameterBridge`

### Key Design Decision: Sidecar Cache Writer

`RebuildPipeline` currently calls `crate::commands::start::write_sidecar_cache` (CLI-specific). Replace with dependency injection:

```rust
pub struct RebuildPipeline {
    // ... existing fields ...
    sidecar_writer: Option<Box<dyn Fn(&[ParameterInfo]) -> Result<()> + Send + Sync>>,
}
```

This keeps CLI-specific file I/O out of the dev-server crate.

### Testing Checkpoint

- [ ] `cargo build -p dev-server` succeeds
- [ ] `cargo test -p dev-server` passes (all migrated tests)
- [ ] `cargo clippy -p dev-server` passes

---

## Phase 4: Create Builder API

**Goal:** Design and implement a clean builder API for CLI integration

**Estimated time:** 4-6 hours | **Risk:** Low-Medium

### Target API

```rust
use dev_server::{DevServer, AudioConfig};

let dev_server = DevServer::builder()
    .parameters(params)
    .ws_port(9000)
    .engine_dir(engine_dir)
    .verbose(true)
    .audio_config(AudioConfig {
        sample_rate: 44100.0,
        buffer_size: 512,
    })
    .plugin_loader(loader)
    .sidecar_writer(Box::new(move |params| {
        write_sidecar_cache(&engine_dir, params)
    }))
    .build()?;

let (session, shutdown_rx) = dev_server.start().await?;
```

### Tasks

1. Create `dev-server/src/builder.rs` — `DevServerBuilder` with fluent API
2. Create `DevServer` struct in `lib.rs` — owns components, provides `start()` method
3. Export builder through `lib.rs`

### Testing Checkpoint

- [ ] Builder API compiles with all configurations
- [ ] Documentation examples in rustdoc compile

---

## Phase 5: Update CLI `start.rs`

**Goal:** Replace ~610-line implementation with thin wrapper using `dev-server` crate

**Estimated time:** 6-8 hours | **Risk:** Medium-High

### Tasks

1. **Update `cli/Cargo.toml`** — Replace `wavecraft-dev-server` dependency with `dev-server`

   ```toml
   [dependencies.dev-server]
   path = "../dev-server"

   [features]
   audio-dev = ["dev-server/audio"]
   ```

2. **Simplify `start.rs`** (~200-250 lines)
   - **Keep in CLI:** project detection, dependency prompts, port checks, parameter loading/caching, UI server spawning, shutdown orchestration
   - **Remove from CLI:** DevServerHost creation, IpcHandler, WsServer, DevSession, FileWatcher, RebuildPipeline, audio initialization

3. **Delete `cli/src/dev_server/` module** — entire directory removed

### Testing Checkpoint

- [ ] `cargo build -p wavecraft` succeeds
- [ ] `wavecraft start` works end-to-end in test project
- [ ] Hot-reload triggers rebuild and updates UI
- [ ] Audio works (audio-dev feature)
- [ ] Ctrl+C shuts down cleanly

---

## Phase 6: Remove Old Locations

**Goal:** Delete obsolete code and clean up workspace

**Estimated time:** 1-2 hours | **Risk:** Low

### Tasks

1. Delete standalone binary files from engine crate (`main.rs`, `app.rs`, `webview.rs`, `assets.rs`)
2. Delete entire `engine/crates/wavecraft-dev-server/` directory
3. Remove from engine workspace (`engine/Cargo.toml`)

### Testing Checkpoint

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes
- [ ] No broken imports

---

## Phase 7: Update All References

**Goal:** Update documentation, templates, and remaining references

**Estimated time:** 2-3 hours | **Risk:** Low

### Tasks

1. Update `cli/sdk-templates/new-project/react/engine/Cargo.toml.template` (if references exist)
2. Update `docs/architecture/high-level-design.md` — repository structure diagram
3. Update `docs/architecture/development-workflows.md` — dev server references
4. Grep for remaining `wavecraft-dev-server` references: `rg -l "wavecraft-dev-server"`
5. Update `cli/src/template/mod.rs` — `SDK_CRATES` array (if `wavecraft-dev-server` is listed)

### Testing Checkpoint

- [ ] `rg "wavecraft-dev-server"` returns only expected hits (archives, docs)
- [ ] `cargo doc --workspace --no-deps` succeeds

---

## Phase 8: Final Verification

**Goal:** Comprehensive end-to-end testing

**Estimated time:** 4-6 hours | **Risk:** Medium-High

### Checklist

- [ ] `wavecraft create TestPlugin --output /tmp/test-unification` succeeds
- [ ] `cd /tmp/test-unification && wavecraft start` works
- [ ] Hot-reload: edit source → verify rebuild triggers
- [ ] Audio capture: verify mic input processed (audio-dev)
- [ ] UI updates: verify browser reflects parameter changes
- [ ] Ctrl+C exits cleanly
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo fmt --all --check` passes
- [ ] Feature flag matrix:
  - [ ] `cargo build -p dev-server` (default: audio on)
  - [ ] `cargo build -p dev-server --no-default-features` (audio off)
  - [ ] `cargo build -p wavecraft` (default: audio-dev on)
  - [ ] `cargo build -p wavecraft --no-default-features` (audio-dev off)

---

## Summary

### Total Estimated Time: 30-45 hours

| Phase | Description            | Hours | Risk        |
| ----- | ---------------------- | ----- | ----------- |
| 1     | Crate skeleton         | 1-2   | Low         |
| 2     | Migrate infrastructure | 4-6   | Low-Medium  |
| 3     | Migrate orchestration  | 8-10  | Medium      |
| 4     | Builder API            | 4-6   | Low-Medium  |
| 5     | Update CLI             | 6-8   | Medium-High |
| 6     | Remove old code        | 1-2   | Low         |
| 7     | Update references      | 2-3   | Low         |
| 8     | Final verification     | 4-6   | Medium-High |

### Key Risks & Mitigations

1. **Breaking hot-reload** → Test incrementally after each phase; preserve exact logic
2. **Audio thread safety** → Copy audio code verbatim; don't modify real-time paths
3. **CLI coupling** → Use dependency injection (callbacks) for CLI-specific utilities
4. **Feature flag confusion** → CLI `audio-dev` maps to dev-server `audio` cleanly
5. **Incomplete migration** → Compile after each phase; grep for stale references

### Dependencies

```
Phase 1 (skeleton)
  ↓
Phase 2 (infrastructure) ──────┐
  ↓                            │
Phase 3 (orchestration) ───────┤
  ↓                            │
Phase 4 (builder API) ─────────┤
  ↓                            │
Phase 5 (CLI update) ──────────┘
  ↓
Phase 6 (cleanup)
  ↓
Phase 7 (references) ← can start during Phase 3-4
  ↓
Phase 8 (verification)
```

### Post-Merge Tasks

1. **PO:** Update roadmap (`docs/roadmap.md`) — mark complete, add changelog entry
2. **PO:** Archive feature spec to `docs/feature-specs/_archive/dev-server-unification/`
3. Update getting started guide if applicable
