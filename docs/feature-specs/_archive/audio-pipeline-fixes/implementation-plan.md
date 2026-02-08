# Implementation Plan: Audio Pipeline Fixes & Mocking Cleanup

## Overview

This plan transforms the dev-mode audio pipeline from metering-only to full-duplex audio I/O with parameter synchronization, removes synthetic meter infrastructure, and adds connection-aware parameter loading in the UI. It spans 4 user stories across 4 phases, targeting version `0.10.0`.

## Requirements

- **US-1:** Audio output in dev mode — paired input/output cpal streams with SPSC ring buffer
- **US-2:** Parameter changes reach DSP — `AtomicParameterBridge` for lock-free UI→audio param sync
- **US-3:** Remove `MeterGenerator` — delete synthetic metering, fallback to zeros
- **US-4:** UI parameter retry — `useAllParameters` re-fetches on WebSocket connection

## Architecture Changes

Refer to the [Low-Level Design](./low-level-design-audio-pipeline-fixes.md) for full architectural diagrams and rationale.

| Component | Change Summary |
|-----------|---------------|
| `wavecraft-dev-server/src/audio_server.rs` | Add output stream, SPSC ring buffer, pre-allocated buffers |
| `wavecraft-dev-server/src/ffi_processor.rs` | Fix `Vec` allocation in `process()` — use `[*mut f32; 2]` |
| `wavecraft-dev-server/src/atomic_params.rs` (new) | `AtomicF32` + `AtomicParameterBridge` |
| `wavecraft-dev-server/src/lib.rs` | Export new `atomic_params` module |
| `wavecraft-dev-server/Cargo.toml` | Add `rtrb` dependency (for audio feature) |
| `wavecraft-metering/src/dev.rs` | **Delete file** |
| `wavecraft-metering/src/lib.rs` | Remove `pub mod dev;` |
| `cli/src/dev_server/host.rs` | Remove `MeterGenerator`, add `AtomicParameterBridge` |
| `cli/src/commands/start.rs` | Wire `AtomicParameterBridge` to `AudioServer` and `DevServerHost` |
| `ui/packages/core/src/hooks/useAllParameters.ts` | Add `useConnectionStatus` dependency for retry |

---

## Implementation Steps

### Phase 1: Fix Real-Time Safety Violations (Foundation)

These fixes are prerequisites for Phases 2 and 3 because the audio callback must be allocation-free before adding the output stream and parameter reads.

#### Step 1.1: Fix `FfiProcessor::process()` allocation

**File:** `engine/crates/wavecraft-dev-server/src/ffi_processor.rs`
**Dependencies:** None
**Risk:** Low

**What to change:**

Replace the `Vec<*mut f32>` allocation in `FfiProcessor::process()` (line ~73) with a stack-allocated `[*mut f32; 2]` array. The current code:

```rust
let mut ptrs: Vec<*mut f32> = channels.iter_mut().map(|ch| ch.as_mut_ptr()).collect();
```

allocates on every audio callback invocation, violating real-time safety.

**Specific changes:**
- Replace the `Vec` with a fixed-size `[*mut f32; 2]` array (wavecraft targets stereo)
- Add an early return guard if `channels.len() > 2` (log error via `tracing::error!` — this shouldn't happen but guards against future multi-channel)
- Populate `ptrs[0]` and `ptrs[1]` directly from `channels[0].as_mut_ptr()` and `channels[1].as_mut_ptr()`
- Add a code comment documenting why a fixed array is used and the stereo assumption

**Tests to verify:**
- Existing `test_ffi_processor_lifecycle` passes unchanged
- Existing `test_ffi_processor_empty_channels_noop` passes unchanged

#### Step 1.2: Pre-allocate buffers in `AudioServer` input callback

**File:** `engine/crates/wavecraft-dev-server/src/audio_server.rs`
**Dependencies:** None (independent of Step 1.1)
**Risk:** Low

**What to change:**

The current input callback allocates `Vec`s on every invocation (lines ~105-106):

```rust
let mut left = vec![0.0f32; num_samples];
let mut right = vec![0.0f32; num_samples];
```

**Specific changes:**
- Before `build_input_stream()`, pre-allocate two buffers using `self.config.buffer_size` as the upper bound:
  ```
  let mut left_buf = vec![0.0f32; buffer_size as usize];
  let mut right_buf = vec![0.0f32; buffer_size as usize];
  ```
- Move these buffers into the closure
- Inside the callback, slice them to the actual `num_samples`: `&mut left_buf[..num_samples]`
- Zero-fill the active slice at the start of each callback (to clear stale data)
- Also pre-allocate the `channels` vec outside the closure: `let mut channels_buf: Vec<&mut [f32]> = Vec::with_capacity(2);` — or better, restructure to avoid the `Vec<&mut [f32]>` entirely by calling `processor.process()` with a locally-constructed slice reference

**Note:** The `channels: Vec<&mut [f32]>` construction also allocates. Since `processor.process()` takes `&mut [&mut [f32]]`, construct it on the stack using a fixed-size array pattern. The simplest approach: build `[left_slice, right_slice]` as a local array and call `processor.process(&mut channels_array)`.

**Tests to verify:**
- Compile check — `cargo build -p wavecraft-dev-server --features audio`
- Manual test: `wavecraft start` still captures mic and shows meters

---

### Phase 2: Audio Output Stream (US-1)

This phase adds the output stream, SPSC ring buffer, and full-duplex audio flow.

#### Step 2.1: Add `rtrb` dependency to `wavecraft-dev-server`

**File:** `engine/crates/wavecraft-dev-server/Cargo.toml`
**Dependencies:** None
**Risk:** None

**What to change:**
- Add `rtrb = { version = "0.3", optional = true }` in `[dependencies]`
- Add `"rtrb"` to the `audio` feature list: `audio = ["cpal", "anyhow", "rtrb"]`

The `rtrb` crate is already used by `wavecraft-metering` at version 0.3, so no version conflict.

#### Step 2.2: Add output device and stream to `AudioServer`

**File:** `engine/crates/wavecraft-dev-server/src/audio_server.rs`
**Dependencies:** Step 2.1, Step 1.2
**Risk:** Medium — cpal device enumeration can fail; sample rate mismatch between input and output devices

**What to change:**

**Struct changes:**
- Rename `device` → `input_device`
- Add `output_device: Device`
- Rename `stream_config` → `input_config`
- Add `output_config: StreamConfig`

**Constructor (`AudioServer::new()`) changes:**
- After getting `default_input_device()`, also call `host.default_output_device()`
- If no output device: log a warning via `tracing::warn!` and store `output_device` as an `Option<Device>`. Fall back to metering-only mode (current behavior)
- Get `default_output_config()` from the output device
- Log both input and output device names and sample rates
- If input and output sample rates differ, log a warning (most macOS setups use unified CoreAudio sample rate)

**`AudioHandle` struct changes:**
- Rename `_stream` → `_input_stream`
- Add `_output_stream: Option<Stream>` (optional — `None` when no output device)

**`AudioServer::start()` changes:**

*SPSC ring buffer setup:*
- Create an `rtrb::RingBuffer::new(capacity)` where `capacity = buffer_size * num_channels * 4` (4 blocks of headroom, as specified in the low-level design)
- This produces a `(Producer<f32>, Consumer<f32>)` pair

*Input callback modifications:*
- After processing and meter computation (existing steps), add:
  1. Interleave `left_buf` + `right_buf` into a temporary interleaved buffer (pre-allocated before the closure, same size as the ring buffer write chunk)
  2. Write interleaved samples to `ring_producer` using `write_chunk_uninit()` or a loop of `push()` calls
  3. If the ring buffer is full, samples are silently dropped (acceptable — temporary glitch)

*Output callback (new):*
- Call `self.output_device.build_output_stream()` (if output device exists)
- The callback receives `data: &mut [f32]` (output buffer to fill)
- Read from `ring_consumer`: attempt to read `data.len()` samples
- If enough samples available: copy to `data`
- If underflow: fill `data` with `0.0` (silence)
- No allocations, no locks

*Pre-allocated interleave buffer:*
- Before the input closure, allocate: `let mut interleave_buf = vec![0.0f32; buffer_size as usize * 2];` (stereo interleaved max size)
- Move into the input closure

**Fallback behavior:**
- If `output_device` is `None`, skip output stream creation entirely. Return `AudioHandle` with `_output_stream: None`. The input-only metering behavior is preserved.

**Tests to add:**
- `AudioServer::new()` with mocked cpal host (if feasible) — or rely on integration test
- Unit test for interleave/deinterleave helper functions if extracted

#### Step 2.3: Update `try_start_audio_in_process()` in CLI

**File:** `cli/src/commands/start.rs`
**Dependencies:** Step 2.2
**Risk:** Low

**What to change:**
- The `AudioHandle` type changes (now has `_input_stream` + `_output_stream`). This is internal to `AudioServer` and flows through opaquely, so `start.rs` likely needs no changes beyond ensuring it compiles.
- If `AudioServer::start()` signature changes (e.g., to accept the `AtomicParameterBridge`), wire it in Phase 3.
- Print an additional status line when output device is detected: `"Audio: Input (mic) + Output (speakers)"`
- If no output device, print: `"Audio: Input only (metering mode)"`

**Tests to verify:**
- `cargo build -p wavecraft --features audio-dev` compiles
- Manual: `wavecraft start` → speak into mic → hear audio through speakers

---

### Phase 3: Parameter Sync (US-2) + Mocking Removal (US-3)

These are grouped because removing `MeterGenerator` from `DevServerHost` and adding `AtomicParameterBridge` both modify the same files. Doing them together avoids double-editing `host.rs` and `start.rs`.

#### Step 3.1: Create `AtomicParameterBridge` module

**File:** `engine/crates/wavecraft-dev-server/src/atomic_params.rs` (new file)
**Dependencies:** None
**Risk:** Low

**What to create:**

An `AtomicF32` struct and an `AtomicParameterBridge` struct as specified in the low-level design §3.2.

**`AtomicF32`:**
- Wraps `std::sync::atomic::AtomicU32`
- `new(val: f32)` — stores `val.to_bits()`
- `load(&self) -> f32` — `f32::from_bits(self.0.load(Ordering::Relaxed))`
- `store(&self, val: f32)` — `self.0.store(val.to_bits(), Ordering::Relaxed)`

**Note:** The `wavecraft-dev-server` Cargo.toml already has `atomic_float = "1.1"` as a dependency. Check if this crate provides an `AtomicF32` type that can be reused instead of rolling a custom one. If `atomic_float::AtomicF32` provides `load`/`store` with `Ordering` parameters, use that. Otherwise, implement the custom `AtomicF32` as described above.

**`AtomicParameterBridge`:**
- Field: `params: HashMap<String, Arc<AtomicF32>>` (or using the `atomic_float` type)
- `new(parameters: &[ParameterInfo]) -> Self` — creates one `Arc<AtomicF32>` per parameter, initialized to `param.default`
- `write(&self, id: &str, value: f32)` — looks up the `Arc<AtomicF32>` and calls `store(value)`
- `read(&self, id: &str) -> Option<f32>` — looks up the `Arc<AtomicF32>` and calls `load()`
- The `HashMap` is immutable after construction (only the `AtomicF32` values change), making reads lock-free
- Implement `Send + Sync` (should be auto-derived since `Arc<AtomicF32>` is `Send + Sync`)

**Tests to write:**
- `test_write_and_read` — write a value, read it back
- `test_read_unknown_param` — returns `None`
- `test_default_values` — all params start at their default values
- `test_concurrent_write_read` — spawn two threads, one writing, one reading (verify no panic, no data race)

#### Step 3.2: Export `atomic_params` module

**File:** `engine/crates/wavecraft-dev-server/src/lib.rs`
**Dependencies:** Step 3.1
**Risk:** None

**What to change:**
- Add `#[cfg(feature = "audio")] pub mod atomic_params;` (same feature gate as `audio_server` and `ffi_processor`)
- The module must be gated behind the `audio` feature because it's only used by the audio path

#### Step 3.3: Delete `MeterGenerator`

**Files:**
- `engine/crates/wavecraft-metering/src/dev.rs` — **Delete entire file**
- `engine/crates/wavecraft-metering/src/lib.rs` — Remove the `pub mod dev;` line (line 6)

**Dependencies:** None (can be done in parallel with Step 3.1)
**Risk:** Low — verify no other consumers reference `wavecraft_metering::dev`

**Pre-check:** Search for any imports of `wavecraft_metering::dev` or `MeterGenerator` outside of the two known consumers:
- `cli/src/dev_server/host.rs` (handled in Step 3.4)
- `engine/crates/wavecraft-metering/src/lib.rs` (the `pub mod dev;` line)

If any other consumers exist, address them. Based on current analysis, there are none.

**Tests affected:**
- The tests inside `dev.rs` are deleted with the file
- Run `cargo test -p wavecraft-metering` to confirm no broken references

#### Step 3.4: Redesign `DevServerHost` — remove `MeterGenerator`, add `AtomicParameterBridge`

**File:** `cli/src/dev_server/host.rs`
**Dependencies:** Step 3.1, Step 3.3
**Risk:** Medium — this is a central integration point

**What to change:**

*Remove:*
- Remove `use wavecraft_metering::dev::MeterGenerator;` import
- Remove the `MeterGeneratorProvider` struct entirely (and its `MeterProvider` impl)
- Remove the `meter_generator: Arc<RwLock<MeterGenerator>>` field from `DevServerHost`
- Remove the `tick_meters()` method
- Remove the `Arc::new(MeterGeneratorProvider { ... })` construction in `DevServerHost::new()`
- Change `InMemoryParameterHost::with_meter_provider(...)` back to `InMemoryParameterHost::new(...)` (no more meter provider)

*Add:*
- Add `use wavecraft_dev_server::atomic_params::AtomicParameterBridge;` (gated behind `#[cfg(feature = "...")]` if needed — check if the CLI always compiles with audio features)
- Add field: `param_bridge: Option<Arc<AtomicParameterBridge>>` to `DevServerHost`
  - `Option` because when no vtable is loaded (metering-only mode), there's no audio thread to read params
  - Even without audio, having the bridge doesn't hurt — but the `Option` keeps the constructor simple when the bridge isn't provided
- Update constructor: `DevServerHost::new(parameters: Vec<ParameterInfo>, param_bridge: Option<Arc<AtomicParameterBridge>>)`
- Add a public getter: `pub fn param_bridge(&self) -> Option<&Arc<AtomicParameterBridge>>`

*Update `set_parameter()`:*
- After the existing `self.inner.set_parameter(id, value)?`, add:
  ```
  if let Some(bridge) = &self.param_bridge {
      bridge.write(id, value);
  }
  ```
- This writes to the atomic bridge on every parameter change from the WebSocket thread

*Update `get_meter_frame()`:*
- `self.inner.get_meter_frame()` will now return `None` (since no `MeterProvider` is set)
- This is the correct behavior: real meters flow through the push path (notifications), not the pull path

**Tests to update:**
- `test_get_meter_frame` — update expectation: now returns `None` instead of a synthetic frame
- Add: `test_set_parameter_updates_bridge` — create `DevServerHost` with a bridge, `set_parameter()`, verify `bridge.read()` returns new value
- Add: `test_without_bridge` — create `DevServerHost` with `None` bridge, `set_parameter()` succeeds (no panic)
- All existing parameter tests should pass (constructor signature change needs argument updates)

#### Step 3.5: Wire `AtomicParameterBridge` into `start.rs`

**File:** `cli/src/commands/start.rs`
**Dependencies:** Step 3.2, Step 3.4
**Risk:** Medium — orchestration changes

**What to change:**

In `run_dev_servers()`, after loading parameters (step 3 in the existing code, around line ~285):

1. **Create the bridge:**
   ```
   let param_bridge = Arc::new(AtomicParameterBridge::new(&params));
   ```

2. **Pass bridge to `DevServerHost`:**
   ```
   let host = DevServerHost::new(params, Some(Arc::clone(&param_bridge)));
   ```

3. **Pass bridge to `AudioServer`** (in `try_start_audio_in_process()`):
   - Update `try_start_audio_in_process()` signature to accept `Arc<AtomicParameterBridge>`
   - Pass `Arc::clone(&param_bridge)` to `AudioServer::new()` or `AudioServer::start()`
   - Inside the audio callback, the bridge is available for reading parameter values

**`AudioServer` constructor/start changes:**
- `AudioServer::new()` or `start()` accepts an additional `param_bridge: Arc<AtomicParameterBridge>` parameter
- The bridge is moved into the input callback closure
- Inside the input callback, call `param_bridge.read("gain")` etc. at the start of each block
- For this milestone, the read values are not injected into the processor (vtable v2 not yet implemented). They are simply available and can be logged in verbose mode for verification. The infrastructure is in place for future use.

**Drop ordering:**
- Ensure `_audio_handle` is declared after `loader` in `run_dev_servers()` (already the case)
- The `param_bridge` `Arc` can be declared anywhere — it's reference-counted and safe to drop in any order

**Tests to verify:**
- `cargo build -p wavecraft --features audio-dev` compiles
- Manual: `wavecraft start` → move slider in UI → verify no crash (atomic write/read works)

#### Step 3.6: Version bump

**File:** `engine/Cargo.toml`
**Dependencies:** After all other steps, but logically belongs to this phase
**Risk:** None

**What to change:**
- Bump `[workspace.package] version` from `"0.9.0"` to `"0.10.0"`
- This propagates to all workspace crates and the UI via the build-time injection

---

### Phase 4: UI Parameter Retry (US-4)

This phase is independent of Phases 1-3 and can be implemented in parallel.

#### Step 4.1: Add connection-aware retry to `useAllParameters`

**File:** `ui/packages/core/src/hooks/useAllParameters.ts`
**Dependencies:** None
**Risk:** Low

**What to change:**

Add a `useConnectionStatus()` dependency that triggers `reload()` when the connection is (re-)established.

*Import:*
- Add `import { useConnectionStatus } from './useConnectionStatus';`

*Inside `useAllParameters()`:*
- Add: `const { connected } = useConnectionStatus();`
- Add a new `useEffect` that depends on `[connected, reload]`:
  ```
  useEffect(() => {
      if (connected) {
          reload();
      }
  }, [connected, reload]);
  ```

*Placement:*
- Add the new `useEffect` after the existing "Load on mount" `useEffect` and before the "Subscribe to parameter changes" `useEffect`

*Deduplication note (from low-level design §5.3):*
- When WebSocket is already connected at mount, both effects fire → duplicate `reload()`. This is acceptable: `getAllParameters` is idempotent, happens once, and the second call overwrites with identical data. No `hasLoaded` guard needed.

**Tests to write:**

Create or update test in `ui/packages/core/src/hooks/useAllParameters.test.ts`:
- `test: re-fetches parameters when connection transitions to connected` — mock `useConnectionStatus` to return `connected: false`, render hook, then update mock to `connected: true`, verify `reload()` called again
- `test: does not fetch when disconnected` — mock `useConnectionStatus` to return `connected: false`, verify no successful parameter load (catch error)
- Verify existing tests still pass

**Note on hook testing:** Check if there's an existing test file for `useAllParameters`. If not, create one following the testing patterns in the codebase (Vitest + React Testing Library with `renderHook`).

---

## Testing Strategy

### Per-Phase Testing

| Phase | Automated Tests | Manual Tests |
|-------|----------------|--------------|
| Phase 1 (RT safety) | Existing `ffi_processor` tests pass; `cargo build --features audio` | `wavecraft start` still works |
| Phase 2 (Audio output) | New interleave/deinterleave tests (if extracted) | Speak into mic → hear through speakers |
| Phase 3 (Param sync + mocking) | `AtomicParameterBridge` unit tests; `DevServerHost` tests updated; `cargo test -p wavecraft-metering` | Move UI slider → verify atomic value (verbose log); meters show zeros when no vtable |
| Phase 4 (UI retry) | `useAllParameters` hook test with mocked connection status | Open browser before WS connects → params appear within 1-2s |

### End-to-End Validation

After all phases, run the full validation:

1. `cargo xtask ci-check` — all lints and tests pass
2. `wavecraft start` with a test plugin:
   - Audio output audible
   - Meters reflect processed audio
   - UI slider changes are responsive
   - Parameters load after WebSocket connects
3. `wavecraft start` **without** vtable (older plugin):
   - Metering-only mode, meters show zeros
   - No crash, clear log message

---

## Ordering & Parallelism

```
Phase 1 ─────────────────┐
  Step 1.1 (FfiProcessor) ─┼── can be parallel
  Step 1.2 (AudioServer)  ─┤
                            │
Phase 4 ────────────────── │── can be parallel with Phases 1-3
  Step 4.1 (useAllParams)  │
                            │
Phase 2 ◄──────────────────┘
  Step 2.1 (rtrb dep)
  Step 2.2 (output stream) ← depends on 1.2, 2.1
  Step 2.3 (CLI update)    ← depends on 2.2
                            │
Phase 3 ◄──────────────────┘
  Step 3.1 (AtomicParamBridge) ─┐
  Step 3.3 (Delete dev.rs)     ─┼── can be parallel
                                │
  Step 3.2 (Export module)      ← depends on 3.1
  Step 3.4 (DevServerHost)     ← depends on 3.1, 3.3
  Step 3.5 (Wire start.rs)     ← depends on 3.2, 3.4, 2.2
  Step 3.6 (Version bump)      ← depends on all above
```

**Recommended execution order:** 1.1 → 1.2 → 2.1 → 2.2 → 2.3 → 3.1 + 3.3 (parallel) → 3.2 → 3.4 → 3.5 → 3.6, with 4.1 at any point.

---

## Risks & Mitigations

| Risk | Severity | Phase | Mitigation |
|------|----------|-------|------------|
| **cpal no output device** | Medium | 2 | Graceful fallback: log warning, continue metering-only. `output_device` is `Option<Device>` |
| **Input/output sample rate mismatch** | Medium | 2 | Use input device's sample rate for processing. Log warning if output differs. macOS CoreAudio typically uses unified rate |
| **Ring buffer sizing too small → dropouts** | Medium | 2 | Default `buffer_size * channels * 4`. Log underflow count for debugging. Can make configurable via `AudioConfig` |
| **Ring buffer underflow (silence glitches)** | Low | 2 | Output callback fills with zeros. Occasional silence is acceptable in dev mode |
| **Breaking `wavecraft-metering` public API** | Medium | 3 | `MeterGenerator` is only used by CLI's `host.rs`. Verify no references in `wavecraft-nih_plug`, published crate APIs, or user-facing templates |
| **`AtomicParameterBridge` not visible to audio thread** | Low | 3 | `Relaxed` ordering is sufficient for block-level parameter updates. One-block delay is imperceptible |
| **`DevServerHost` constructor signature change** | Low | 3 | Update all call sites (`start.rs`). Only one caller |
| **`useAllParameters` double-fetch on startup** | Very Low | 4 | Idempotent operation, negligible cost. Documented as acceptable in low-level design §5.3 |
| **`atomic_float` crate vs. custom `AtomicF32`** | Low | 3 | Check if `atomic_float::AtomicF32` API matches needs. If it has `load(Ordering)`/`store(val, Ordering)`, use it directly. Otherwise implement custom wrapper |
| **Feature gate mismatch** | Low | 3 | The CLI compiles with `audio-dev` feature. Ensure `atomic_params` module is gated under `audio` feature in `wavecraft-dev-server` and that the CLI's feature enables it |

---

## Success Criteria

- [ ] `wavecraft start` opens both input and output audio streams; processed audio is audible through speakers
- [ ] Pre-allocated buffers in audio callback — no `Vec` allocations on the audio thread
- [ ] SPSC ring buffer connects input→output with correct interleaving
- [ ] `AtomicParameterBridge` created at startup with one `AtomicF32` per parameter
- [ ] `DevServerHost::set_parameter()` writes to both `InMemoryParameterHost` and `AtomicParameterBridge`
- [ ] Audio callback can read parameter values via `AtomicParameterBridge` (lock-free)
- [ ] `wavecraft-metering/src/dev.rs` deleted; no `MeterGenerator` references remain
- [ ] Metering-only fallback shows zero meters (not fake animated data)
- [ ] `useAllParameters()` retries parameter fetch when WebSocket connection is established
- [ ] `cargo xtask ci-check` passes (all lints + tests green)
- [ ] Version bumped to `0.10.0` in `engine/Cargo.toml`
