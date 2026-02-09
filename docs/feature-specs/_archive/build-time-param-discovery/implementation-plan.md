# Implementation Plan: Build-Time Parameter Discovery

## Overview

`wavecraft start` hangs at "Loading plugin parameters..." because loading the plugin dylib triggers nih-plug's VST3/CLAP static initializers (`nih_export_clap!`, `nih_export_vst3!`), which block on macOS audio subsystem services (`AudioComponentRegistrar`). The fix is to feature-gate the nih-plug export macros behind a `_param-discovery` Cargo feature. When building with this feature active, the dylib only contains param FFI functions — no plugin factory registrations, no static initializers. The CLI builds with this feature for fast param extraction, caches the result as a sidecar JSON file, and only does a full build (with nih-plug exports) if audio-dev is needed.

## Requirements

- `wavecraft start` must not hang when loading plugin parameters
- Zero user friction — no new manual steps required
- Backward-compatible with existing plugins that lack the `_param-discovery` feature
- Sidecar JSON cache to skip rebuild on subsequent runs when code hasn't changed
- Full nih-plug build still happens (in background) when `audio-dev` is needed
- Existing test suites and CI must continue to pass

## Architecture Changes

1. **`engine/crates/wavecraft-macros/src/plugin.rs`** — Wrap `nih_export_clap!` / `nih_export_vst3!` invocations in `#[cfg(not(feature = "_param-discovery"))]` guards.
2. **`cli/sdk-templates/new-project/react/engine/Cargo.toml.template`** — Add `[features]` section with `_param-discovery = []`.
3. **`cli/src/commands/start.rs`** — Replace the single-phase build+load with a two-phase flow: (1) build with `_param-discovery` for fast param extraction, (2) optionally build without it in the background for audio-dev.
4. **`engine/crates/wavecraft-bridge/src/plugin_loader.rs`** — Add `load_params_from_file()` static method for reading cached sidecar JSON.

## Implementation Steps

### Phase 1: Macro Feature Gate

#### Step 1.1: Wrap nih-plug export macros with `#[cfg]` guard

**File:** `engine/crates/wavecraft-macros/src/plugin.rs`

**Action:** Change the current unconditional export to a conditionally-compiled version with `#[cfg(not(feature = "_param-discovery"))]` guards.

**Dependencies:** None

---

### Phase 2: Template Update

#### Step 2.1: Add `_param-discovery` feature to template Cargo.toml

**File:** `cli/sdk-templates/new-project/react/engine/Cargo.toml.template`

**Action:** Add a `[features]` section:

```toml
[features]
default = []
_param-discovery = []   # Internal: used by `wavecraft start` for fast param loading
```

**Dependencies:** None (independent of Phase 1)

---

### Phase 3: Sidecar JSON Cache in `PluginParamLoader`

#### Step 3.1: Add `load_params_from_file()` to `PluginParamLoader`

**File:** `engine/crates/wavecraft-bridge/src/plugin_loader.rs`

**Action:** Add a new static method to `PluginParamLoader`:

```rust
/// Load parameters from a sidecar JSON file (bypasses FFI/dlopen).
pub fn load_params_from_file<P: AsRef<Path>>(
    json_path: P,
) -> Result<Vec<ParameterInfo>, PluginLoaderError> {
    let contents = std::fs::read_to_string(json_path.as_ref())?;
    let params: Vec<ParameterInfo> = serde_json::from_str(&contents)?;
    Ok(params)
}
```

**Dependencies:** None

---

#### Step 3.2: Add unit test for `load_params_from_file()`

**File:** `engine/crates/wavecraft-bridge/src/plugin_loader.rs` (test module)

**Action:** Add test that writes a JSON file, reads it back with `load_params_from_file()`, and verifies the deserialized values match.

**Dependencies:** Step 3.1

---

### Phase 4: CLI Two-Phase Build + Sidecar Cache

#### Step 4.1: Add sidecar JSON helper functions

**File:** `cli/src/commands/start.rs`

**Action:** Add helper functions for sidecar path, cache reading, and cache writing.

**Dependencies:** Step 3.1

---

#### Step 4.2: Refactor `run_dev_servers()` to use two-phase param loading

**File:** `cli/src/commands/start.rs`

**Action:** Replace the current single-phase build+load flow with the new two-phase flow.

**Dependencies:** Steps 4.1, 4.3

---

#### Step 4.3: Implement the `load_parameters()` function

**File:** `cli/src/commands/start.rs`

**Action:** Add the core two-phase param loading function with cache → fast build → fallback logic.

**Dependencies:** Step 4.1

---

### Phase 5: Testing

#### Step 6.1: Verify `_param-discovery` feature gate works (symbol check)

**Action:** Build a test plugin with `--features _param-discovery` and verify symbols with `nm -g`.

**Dependencies:** Steps 1.1, 2.1

---

#### Step 6.2: Integration test — `wavecraft start` loads params without hang

**Action:** Run `wavecraft start` and verify parameters load quickly with sidecar cache.

**Dependencies:** Steps 4.1–4.5

---

#### Step 6.3: Backward compatibility test — fallback for old plugins

**Action:** Test with a plugin that does NOT have the `_param-discovery` feature and verify fallback works.

**Dependencies:** Step 4.3

---

#### Step 6.5: Template validation test

**Action:** Run the standard template validation workflow with clippy and feature verification.

**Dependencies:** Steps 2.1, 2.2

---

#### Step 6.6: Run `cargo xtask ci-check`

**Action:** Run the full CI check suite to verify no regressions.

**Dependencies:** All previous steps

---

## Success Criteria

- [ ] `wavecraft start` loads parameters within 5 seconds (no hang)
- [ ] Sidecar JSON file is created at `target/debug/wavecraft-params.json`
- [ ] Subsequent runs with no code changes read from cache (< 1 second)
- [ ] Plugins built without `_param-discovery` fall back to current FFI loading
- [ ] `nm -g` on discovery-built dylib shows NO `clap_entry` / `GetPluginFactory` symbols
- [ ] `nm -g` on normally-built dylib shows ALL expected symbols
- [ ] `cargo xtask ci-check` passes
- [ ] Template validation (create → clippy → build) passes

## Documentation References

- [Low-Level Design](./low-level-design-build-time-param-discovery.md) — Full design analysis
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Development Workflows](../../architecture/development-workflows.md) — Build system and dev mode
