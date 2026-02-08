# Implementation Progress: Audio Pipeline Fixes & Mocking Cleanup

## Status: Complete (All Phases Implemented)

---

## Phase 1: Fix Real-Time Safety Violations

- [x] **Step 1.1** — Fix `FfiProcessor::process()` `Vec` allocation → `[*mut f32; 2]` array
  - File: `engine/crates/wavecraft-dev-server/src/ffi_processor.rs`
  - Changed `Vec<*mut f32>` to `[*mut f32; 2]` stack-allocated array
- [x] **Step 1.2** — Pre-allocate buffers in `AudioServer` input callback
  - File: `engine/crates/wavecraft-dev-server/src/audio_server.rs`
  - Buffers pre-allocated outside callback via `move` closure capture

## Phase 2: Audio Output Stream (US-1)

- [x] **Step 2.1** — Add `rtrb` dependency to `wavecraft-dev-server` (audio feature)
  - File: `engine/crates/wavecraft-dev-server/Cargo.toml`
  - Added `rtrb = { version = "0.3", optional = true }`, included in `audio` feature
- [x] **Step 2.2** — Add output device, SPSC ring buffer, and output stream to `AudioServer`
  - File: `engine/crates/wavecraft-dev-server/src/audio_server.rs`
  - Full rewrite: SPSC ring buffer connects input→output, graceful fallback to metering-only if no output device
- [x] **Step 2.3** — Update `try_start_audio_in_process()` for new `AudioHandle` type
  - File: `cli/src/commands/start.rs`
  - Updated to pass `Arc<AtomicParameterBridge>`, added `has_output()` status message

## Phase 3: Parameter Sync + Mocking Removal (US-2  + US-3)

- [x] **Step 3.1** — Create `AtomicParameterBridge` module (`AtomicF32` + bridge struct)
  - File: `engine/crates/wavecraft-dev-server/src/atomic_params.rs` (new)
  - `AtomicParameterBridge` with `HashMap<String, Arc<AtomicF32>>`, `write()`/`read()` with `Ordering::Relaxed`, 5 tests
- [x] **Step 3.2** — Export `atomic_params` module from `lib.rs`
  - File: `engine/crates/wavecraft-dev-server/src/lib.rs`
  - Exported under `#[cfg(feature = "audio")]`
- [x] **Step 3.3** — Delete `MeterGenerator` (`dev.rs` + `pub mod dev;`)
  - Deleted: `engine/crates/wavecraft-metering/src/dev.rs`
  - Modified: `engine/crates/wavecraft-metering/src/lib.rs` — removed `pub mod dev;`
- [x] **Step 3.4** — Redesign `DevServerHost` — remove `MeterGenerator`, add `AtomicParameterBridge`
  - File: `cli/src/dev_server/host.rs`
  - Removed all MeterGenerator usage, added `param_bridge: Option<Arc<AtomicParameterBridge>>` (cfg-gated), `set_parameter()` writes to bridge, `get_meter_frame()` returns `None`
- [x] **Step 3.5** — Wire `AtomicParameterBridge` into `start.rs` and `AudioServer`
  - File: `cli/src/commands/start.rs`
  - Creates `AtomicParameterBridge` from discovered params, passes to both `DevServerHost` and `AudioServer`
- [x] **Step 3.6** — Version bump `0.9.0` → `0.10.0`
  - File: `engine/Cargo.toml` + all crate `Cargo.toml` files
  - All workspace and inter-crate versions bumped to `0.10.0`

## Phase 4: UI Parameter Retry (US-4)

- [x] **Step 4.1** — Add connection-aware retry to `useAllParameters` hook
  - File: `ui/packages/core/src/hooks/useAllParameters.ts`
  - Added `useConnectionStatus()` dependency, reloads parameters when `connected` transitions to `true`

---

## Additional Changes

- Removed unused `wavecraft-metering` dependency from `cli/Cargo.toml`
- Added `#[cfg_attr(feature = "audio-dev", allow(dead_code))]` to `DevServerHost::new()` (used by tests and non-audio-dev builds)
- Updated CLI crate dependency versions to `0.10.0`

## Verification

- [x] `cargo xtask ci-check` passes — 146 engine tests + 28 UI tests, all linting green (13.8s)
- [x] `cargo test` in CLI — 57 tests passed
- [ ] Manual: `wavecraft start` → audio output audible
- [ ] Manual: UI slider → parameter changes (no crash)
- [ ] Manual: Meters show zeros when no vtable
- [ ] Manual: Parameters load after WebSocket connects
