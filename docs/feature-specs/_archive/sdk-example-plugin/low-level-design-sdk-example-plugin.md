# Low-Level Design — SDK Example Plugin Crate

> **Feature folder:** `docs/feature-specs/sdk-example-plugin/`
> **Status:** Draft
> **Author:** Architect Agent

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
- [Development Workflows](../../architecture/development-workflows.md) — Browser dev mode, build system
- [SDK Architecture](../../architecture/sdk-architecture.md) — Crate structure, distribution model
- [Declarative Plugin DSL](../../architecture/declarative-plugin-dsl.md) — Macro system & parameter discovery

---

## 1. Goals

1. **`cargo xtask dev` works from the SDK root** — SDK developers can run a live UI + WebSocket + hot-reload session without creating a separate plugin project.
2. **Exercises the real plugin path** — The example crate uses `wavecraft_plugin!`, `wavecraft_processor!`, `SignalChain![]`, `_param-discovery`, and `cdylib` export, exactly as a generated project does. This makes it a first-class integration test of the SDK surface.
3. **Minimal footprint** — The example crate adds no new dependencies to the workspace; it reuses existing SDK crates via workspace path deps.
4. **No impact on generated projects** — All changes are additive or gated behind a `sdk_mode` path. The existing `wavecraft start` flow for user projects remains untouched.
5. **Template parity** — The example plugin mirrors the structure produced by `wavecraft create`, serving as a living reference implementation.

## 2. Non-Goals

- **Replacing `wavecraft create` for end users.** CLI scaffolding remains the intended workflow for plugin authors.
- **Publishing or distributing the example crate.** It is `publish = false`, dev-only.
- **Adding new processor types.** The example uses the same `Gain` / `Oscillator` processors the template already provides.
- **Changing the dev-server crate.** The dev-server is transport-agnostic and already works correctly when given valid `ProjectMarkers`.
- **Windows/Linux support.** Consistent with the project's macOS-first constraint, this design is validated on macOS only.

## 3. Design Overview

### 3.1 Current State (Problem)

```
SDK root (wavecraft/)
├── engine/
│   ├── Cargo.toml          ← [workspace] — NOT a [package]
│   └── crates/             ← library crates only, no cdylib
├── ui/
│   └── package.json
└── cli/
    └── src/
        └── project/
            └── detection.rs ← ProjectMarkers::detect() requires [package] in engine/Cargo.toml
```

When `cargo xtask dev` calls `wavecraft start`:

1. `StartCommand::execute()` calls `ProjectMarkers::detect(&cwd)` where `cwd` is the repo root
2. `detect()` reads `engine/Cargo.toml`, finds `[workspace]` instead of `[package]`
3. Returns error: _"Cannot run dev server in the SDK repository"_

### 3.2 Proposed State (Solution)

```
SDK root (wavecraft/)
├── engine/
│   ├── Cargo.toml          ← [workspace] members now includes "crates/wavecraft-example"
│   └── crates/
│       ├── wavecraft-example/      ← NEW: thin example plugin (cdylib)
│       │   ├── Cargo.toml          ← [package] name = "wavecraft-example"
│       │   └── src/
│       │       └── lib.rs          ← wavecraft_plugin! { ... }
│       ├── wavecraft-core/
│       ├── wavecraft-nih_plug/
│       └── ... (other crates unchanged)
├── ui/
│   └── package.json
└── cli/
    └── src/
        └── project/
            └── detection.rs ← ProjectMarkers::detect() now supports SDK mode
```

The detection flow becomes:

1. `detect()` reads `engine/Cargo.toml`
2. Detects `[workspace]` → **SDK mode**
3. Sets `engine_dir` to `engine/crates/wavecraft-example/` (the example plugin crate)
4. Sets `sdk_mode = true` on `ProjectMarkers`
5. All downstream code (dylib discovery, build, param extraction) operates on the example crate as if it were a normal plugin project

## 4. Crate Layout

### 4.1 `engine/crates/wavecraft-example/Cargo.toml`

```toml
[package]
name = "wavecraft-example"
description = "Example Wavecraft plugin for SDK development and testing"
publish = false
version.workspace = true
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true

[lib]
name = "wavecraft_example"
crate-type = ["cdylib"]

[dependencies]
wavecraft = { package = "wavecraft-nih_plug", path = "../wavecraft-nih_plug" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"

[features]
default = []
_param-discovery = []
```

**Key points:**

- `crate-type = ["cdylib"]` — produces a loadable dylib, exactly like a generated plugin.
- `publish = false` — never goes to crates.io.
- `_param-discovery` feature — enables the fast-build path used by `wavecraft start`.
- Uses `wavecraft-nih_plug` via workspace-relative path dep (renamed to `wavecraft` for import parity with generated projects).

### 4.2 `engine/crates/wavecraft-example/src/lib.rs`

```rust
use wavecraft::prelude::*;

wavecraft_processor!(InputGain => Gain);
wavecraft_processor!(OutputGain => Gain);

wavecraft_plugin! {
    name: "Wavecraft Example",
    signal: SignalChain![InputGain, OutputGain],
}
```

Mirrors the default template output. SDK developers can add processors here to test new DSL features during development.

### 4.3 Workspace Registration

**`engine/Cargo.toml`** — Add to the existing `members` list:

```toml
[workspace]
members = ["crates/*", "xtask"]
```

No change needed — the `crates/*` glob already includes `crates/wavecraft-example`.

## 5. Changes to Detection / Start Flow

### 5.1 `ProjectMarkers` (detection.rs)

**Add `sdk_mode` field:**

```rust
pub struct ProjectMarkers {
    pub ui_dir: PathBuf,
    pub engine_dir: PathBuf,        // In SDK mode: points to wavecraft-example crate
    pub ui_package_json: PathBuf,
    pub engine_cargo_toml: PathBuf, // In SDK mode: wavecraft-example/Cargo.toml
    pub sdk_mode: bool,             // NEW
}
```

**Modify `detect()` to support SDK mode:**

```rust
pub fn detect(start_dir: &Path) -> Result<Self> {
    let ui_dir = start_dir.join("ui");
    let engine_dir = start_dir.join("engine");
    let ui_package_json = ui_dir.join("package.json");
    let engine_cargo_toml = engine_dir.join("Cargo.toml");

    // ... existing checks for ui_dir, engine_dir, ui_package_json, engine_cargo_toml ...

    // Check if this is the SDK workspace (has [workspace] in engine/Cargo.toml)
    if is_sdk_repo(&engine_cargo_toml)? {
        // SDK mode: redirect engine_dir to the example plugin crate
        let example_dir = engine_dir.join("crates").join("wavecraft-example");
        let example_cargo = example_dir.join("Cargo.toml");

        if !example_dir.is_dir() || !example_cargo.is_file() {
            bail!(
                "SDK mode detected but 'engine/crates/wavecraft-example/' is missing.\n\
                 This crate is required to run the dev server from the SDK repo."
            );
        }

        return Ok(Self {
            ui_dir,
            engine_dir: example_dir,
            ui_package_json,
            engine_cargo_toml: example_cargo,
            sdk_mode: true,
        });
    }

    Ok(Self {
        ui_dir,
        engine_dir,
        ui_package_json,
        engine_cargo_toml,
        sdk_mode: false,
    })
}
```

The already-defined `is_sdk_repo()` function (currently unused dead code) becomes active.

### 5.2 `dylib.rs` — No Changes Required

`find_plugin_dylib(engine_dir)` receives `engine_dir` which in SDK mode is `engine/crates/wavecraft-example/`. The function:

1. Calls `resolve_debug_dir(engine_dir)` — tries `engine_dir/target/debug/`, then falls back to `engine_dir/../target/debug/` (the workspace root target). Since the engine workspace builds everything into `engine/target/debug/`, the workspace fallback path resolves correctly.
2. Looks for `libwavecraft_example.dylib` — matches via `read_engine_crate_name()` which reads `wavecraft-example/Cargo.toml` → lib name `wavecraft_example`.

**No modifications needed.** The existing two-level target directory search handles workspace builds.

### 5.3 `start.rs` — Minimal Changes

The `load_parameters()` function builds with `--features _param-discovery` and uses `--package` from `read_engine_package_name()`. When `engine_dir` points to the example crate:

- `read_engine_package_name()` returns `"wavecraft-example"`
- Cargo build runs from the example crate directory with `--package wavecraft-example --features _param-discovery`

**One required change:** The `cargo build --lib` invocation currently uses `current_dir(engine_dir)`. Since the example crate is part of the engine workspace, Cargo will resolve workspace deps. However, the `--package` flag is already being passed, so this should work as-is.

**Verify:** The `--package` flag combined with `current_dir` pointing to a workspace member should work because Cargo walks up to find the workspace root. This is standard Cargo behavior.

### 5.4 `xtask dev` — No Changes Required

`cargo xtask dev` invokes `cargo run --manifest-path ../cli/Cargo.toml --features audio-dev -- start`. The CLI `start` command calls `ProjectMarkers::detect(&cwd)` where `cwd` is the SDK root. With the modified detection, this now succeeds in SDK mode.

### 5.5 Hot-Reload Watcher

The `DevSession` watches `engine_dir/src/` for changes. In SDK mode, this watches `engine/crates/wavecraft-example/src/`. This is correct — changes to the example plugin trigger rebuilds.

**Consideration:** SDK developers might also want to trigger rebuilds when changing SDK crate sources (e.g., `engine/crates/wavecraft-core/src/`). This is a **future enhancement**, not part of this design. For now, manually restarting the dev server after SDK crate changes is acceptable. (The example crate's dependency on SDK crates means `cargo build` will recompile transitively, but the file watcher won't detect source changes in dependency crates.)

## 6. Build & Load Steps

### Complete flow when running `cargo xtask dev` from SDK root:

```
1. cargo xtask dev
   └─► xtask::commands::dev::run()
       └─► cargo run --manifest-path ../cli/Cargo.toml --features audio-dev -- start

2. StartCommand::execute()
   └─► ProjectMarkers::detect(&cwd)           // cwd = SDK root
       ├─ finds engine/Cargo.toml → [workspace] → SDK mode
       └─ engine_dir = engine/crates/wavecraft-example/

3. load_parameters(engine_dir)
   ├─ try_read_cached_params() → cache miss (first run)
   ├─ cargo build --lib --features _param-discovery --package wavecraft-example
   │   ├─ current_dir = engine/crates/wavecraft-example/
   │   └─ output → engine/target/debug/libwavecraft_example.dylib
   ├─ find_plugin_dylib(engine_dir)
   │   └─ resolve_debug_dir → engine/target/debug/ (workspace fallback)
   │       → libwavecraft_example.dylib
   └─ PluginLoader::load() → extract parameters → write sidecar cache

4. run_dev_servers()
   ├─ Start WebSocket server (port 9000)
   ├─ Start DevSession (watches engine/crates/wavecraft-example/src/)
   ├─ Try audio in-process (loads vtable from dylib)
   └─ Start Vite UI dev server (port 5173)
```

## 7. Risks & Mitigations

| #   | Risk                                                | Impact                                                          | Likelihood | Mitigation                                                                                                                                                   |
| --- | --------------------------------------------------- | --------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | **Workspace target dir differs from crate-local**   | `find_plugin_dylib` can't find the built dylib                  | Low        | `resolve_debug_dir` already has a two-level fallback (crate-local → workspace). Validated by reading existing code.                                          |
| 2   | **`cargo build --package` from subdirectory**       | Build fails or builds the wrong crate                           | Low        | Cargo walks up to find workspace root. `--package wavecraft-example` is explicit. Standard Cargo behavior.                                                   |
| 3   | **Example crate drifts from template**              | Develops inconsistencies with what `wavecraft create` generates | Medium     | CI job compares example crate structure against template. Document the invariant: example MUST use the same macros and patterns as the template.             |
| 4   | **Hot-reload doesn't detect SDK crate changes**     | Developers change `wavecraft-core` but rebuild isn't triggered  | Medium     | Document limitation. Future: extend watcher to observe `engine/crates/*/src/`. Out of scope for this design.                                                 |
| 5   | **`sdk_mode` flag leaks into user-facing behavior** | Generated projects accidentally set `sdk_mode = true`           | Low        | `sdk_mode` is only `true` when `engine/Cargo.toml` contains `[workspace]`. Generated projects have `[package]` in their engine Cargo.toml.                   |
| 6   | **Sidecar cache path collision**                    | SDK builds clobber cache from a different plugin                | Low        | Sidecar goes in `resolve_debug_dir()/wavecraft-params.json`. In workspace mode this is `engine/target/debug/wavecraft-params.json`, unique to the workspace. |

## 8. Test Considerations

### 8.1 Unit Tests

| Test                             | Location       | What It Validates                                                                                               |
| -------------------------------- | -------------- | --------------------------------------------------------------------------------------------------------------- |
| `test_sdk_repo_detection`        | `detection.rs` | `detect()` returns `sdk_mode = true` when `engine/Cargo.toml` has `[workspace]` and `wavecraft-example/` exists |
| `test_sdk_mode_missing_example`  | `detection.rs` | `detect()` returns error when `[workspace]` present but `wavecraft-example/` missing                            |
| `test_plugin_project_unchanged`  | `detection.rs` | `detect()` returns `sdk_mode = false` for normal plugin projects with `[package]`                               |
| `test_dylib_discovery_workspace` | `dylib.rs`     | `find_plugin_dylib()` finds dylib via workspace fallback path                                                   |

### 8.2 Integration Tests

| Test                         | How                                                           | What It Validates                                                   |
| ---------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------- |
| `cargo xtask dev` smoke test | Run from SDK root, verify servers start                       | End-to-end SDK mode detection, build, param loading, server startup |
| Parameter extraction         | Start dev server, connect WebSocket, send `getParameter`      | Example plugin exposes expected parameters (InputGain, OutputGain)  |
| Hot-reload                   | Edit `wavecraft-example/src/lib.rs`, verify rebuild triggered | File watcher detects changes in example crate source                |

### 8.3 CI Validation

- **Template parity check:** CI job runs `clippy` on `wavecraft-example` alongside the existing template validation workflow.
- **Build verification:** `cargo build -p wavecraft-example --features _param-discovery` runs in the standard CI pipeline (already covered by workspace `cargo build`).
- **Lint:** `cargo clippy -p wavecraft-example -- -D warnings` added to the lint step.

### 8.4 Updating Existing Tests

The existing `test_sdk_repo_detection` test in `detection.rs` currently expects an error when `[workspace]` is detected. This test must be updated to:

1. Create a `wavecraft-example` directory structure inside the temp dir
2. Assert `sdk_mode == true` instead of asserting an error

## 9. Files Changed (Summary)

| File                                         | Change Type  | Description                                                                           |
| -------------------------------------------- | ------------ | ------------------------------------------------------------------------------------- |
| `engine/crates/wavecraft-example/Cargo.toml` | **New**      | Example plugin crate manifest                                                         |
| `engine/crates/wavecraft-example/src/lib.rs` | **New**      | Example plugin using SDK macros                                                       |
| `cli/src/project/detection.rs`               | **Modified** | Add `sdk_mode` field, SDK workspace detection in `detect()`, activate `is_sdk_repo()` |
| `cli/src/project/detection.rs` (tests)       | **Modified** | Update `test_sdk_repo_detection`, add `test_sdk_mode_missing_example`                 |

**Files explicitly NOT changed:**

- `cli/src/project/dylib.rs` — existing workspace fallback handles SDK mode
- `cli/src/commands/start.rs` — `--package` flag and `engine_dir` abstraction handle SDK mode
- `engine/xtask/src/commands/dev.rs` — no changes needed
- `engine/Cargo.toml` — `crates/*` glob auto-includes the new crate
- `dev-server/` — transport-agnostic, no changes needed

## 10. Future Extensions

- **Watch all SDK crates:** Extend `DevSession` file watcher to observe `engine/crates/*/src/` in SDK mode for full hot-reload coverage.
- **Example processor gallery:** Add commented-out processors in the example crate demonstrating each built-in DSP type.
- **`cargo xtask dev --example <name>`:** Support multiple example plugins for testing different DSL features.

---

## Appendix: Decision Log

| Decision                                  | Rationale                                                              | Alternatives Considered                                                  |
| ----------------------------------------- | ---------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| Example crate inside `engine/crates/`     | Workspace glob auto-includes it; consistent with existing crate layout | Top-level `example/` dir (requires workspace changes, breaks convention) |
| Modify `ProjectMarkers::detect()`         | Single code path for project resolution; minimal change surface        | Separate `--sdk-mode` CLI flag (more invasive, user-facing complexity)   |
| `publish = false`                         | Development-only artifact                                              | Not applicable                                                           |
| Don't watch SDK crate sources (initially) | Keeps scope small; `cargo build` handles transitive recompilation      | Watch all `engine/crates/*/src/` (future enhancement)                    |
