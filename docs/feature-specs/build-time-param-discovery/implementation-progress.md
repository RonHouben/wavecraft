# Implementation Progress: Build-Time Parameter Discovery

## Status: In Progress

## Phase 1: Macro Feature Gate
- [x] **Step 1.1** — Wrap `nih_export_clap!` / `nih_export_vst3!` in `#[cfg(not(feature = "_param-discovery"))]` in `engine/crates/wavecraft-macros/src/plugin.rs`
- [ ] **Step 1.2** _(contingency)_ — Module-wrapped exports if Step 1.1 fails

## Phase 2: Template Update
- [x] **Step 2.1** — Add `[features]` section with `_param-discovery = []` to `cli/sdk-templates/new-project/react/engine/Cargo.toml.template`
- [ ] **Step 2.2** — Regenerate template `Cargo.lock`

## Phase 3: Sidecar JSON Cache in `PluginParamLoader`
- [x] **Step 3.1** — Add `load_params_from_file()` method to `engine/crates/wavecraft-bridge/src/plugin_loader.rs`
- [x] **Step 3.2** — Add unit test for `load_params_from_file()`

## Phase 4: CLI Two-Phase Build + Sidecar Cache
- [x] **Step 4.1** — Add sidecar JSON helper functions (`sidecar_json_path`, `try_read_cached_params`, `write_sidecar_cache`) in `cli/src/commands/start.rs`
- [x] **Step 4.2** — Refactor `run_dev_servers()` to use two-phase param loading
- [x] **Step 4.3** — Implement `load_parameters()` function with cache → fast build → fallback
- [x] **Step 4.4** — Verify audio-dev vtable loading works from discovery build
- [x] **Step 4.5** — Add `use` imports for new types

## Phase 5: Invalidate Sidecar on Source Changes
- [x] **Step 5.1** — Conservative mtime-based cache invalidation (implemented in Step 4.1)

## Phase 6: Testing
- [ ] **Step 6.1** — Verify `_param-discovery` feature gate (symbol check with `nm -g`)
- [ ] **Step 6.2** — Integration test: `wavecraft start` loads params without hang
- [ ] **Step 6.3** — Backward compatibility test: fallback for old plugins
- [ ] **Step 6.4** — Cache invalidation test
- [ ] **Step 6.5** — Template validation test (`wavecraft create` → clippy → build)
- [x] **Step 6.6** — Run `cargo xtask ci-check`
