# Low-Level Design: Audio Pipeline Fixes & Mocking Cleanup

## Related Documents

- [User Stories](./user-stories.md) — Requirements and acceptance criteria
- [High-Level Design](../../architecture/high-level-design.md) — Overall architecture, Dev Audio via FFI
- [Coding Standards](../../architecture/coding-standards.md) — Real-time safety, FFI patterns, TypeScript conventions

---

## 1. Architecture Overview

This milestone transforms the dev-mode audio pipeline from **metering-only** to **full-duplex audio I/O** with parameter synchronization. Today, `wavecraft start` opens an input-only cpal stream, processes audio through the user's `FfiProcessor`, computes meters, and broadcasts them over WebSocket. Audio output is discarded — the developer never hears their DSP. Parameter changes from the UI update `InMemoryParameterHost` via `RwLock`, but these values never reach the audio callback.

### Current Data Flow (Broken)

```
OS Mic ──► cpal input callback ──► FfiProcessor::process() ──► meter computation ──► WebSocket
                                         ▲                          │
                              (default params only)          (meters to browser UI)
                                                           
                                   ╳ NO audio output
                                   ╳ NO parameter sync from UI
```

### Target Data Flow (After This Milestone)

```
                          ┌──────────────────────────────────────────────────────┐
                          │              cpal Audio Callback Thread              │
                          │                                                      │
OS Mic ──► cpal input ──► │  deinterleave ──► inject params ──► FfiProcessor     │
                          │                   (AtomicF32 read)   ::process()     │
                          │                                          │           │
                          │                                          ▼           │
                          │                                  interleave ──► cpal output ──► Speakers
                          │                                          │           │
                          │                                   meter compute      │
                          │                                          │           │
                          └──────────────────────────────────────────┼───────────┘
                                                                     │
                                                              MeterUpdateNotification
                                                                     │
                                                                     ▼
                                                              WebSocket broadcast
                                                                     │
                                                                     ▼
                                                               Browser UI

        WebSocket thread                                       
        ────────────────                                       
        UI setParameter ──► IpcHandler ──► ParameterHost       
                                              │                
                                       Arc<AtomicF32> write    
                                              │                
                                    (read by audio callback    
                                     at next block boundary)   
```

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Audio I/O model | Separate input + output streams | cpal does not expose full-duplex on most backends; separate streams is the portable approach |
| Inter-stream buffer | SPSC ring buffer (`rtrb`) | Lock-free, real-time safe, already a dependency |
| Parameter bridge | `Arc<AtomicF32>` per parameter | Zero-allocation, zero-lock reads on audio thread; matches existing pattern in `wavecraft-nih_plug` |
| VTable extension | **No** — parameters injected externally | Avoids ABI-breaking change; keeps vtable versioned at v1; simpler for existing plugins |
| Meter computation point | After `process()` (from processed output) | Already correct in current code; unchanged |

---

## 2. Audio I/O Design (US-1)

### 2.1 Problem

`AudioServer::start()` calls `device.build_input_stream()` only. The input callback runs `processor.process()` on the mic data, computes meters, and discards the processed samples. There is no output stream.

### 2.2 Approach: Paired Input + Output Streams with SPSC Ring Buffer

cpal does not provide a true full-duplex stream on most platform backends. The standard approach is:

1. Open an **input stream** (mic capture)
2. Open an **output stream** (speaker playback)
3. Use a **lock-free ring buffer** to pass data from input callback → output callback

The input callback deinterleaves, processes via `FfiProcessor`, writes processed samples into the ring buffer, and computes meters. The output callback reads from the ring buffer and interleaves to the output device.

### 2.3 Detailed Design

#### AudioServer Changes

```
AudioServer {
    processor: Box<dyn DevAudioProcessor>,
    config: AudioConfig,
    input_device: Device,        // was: device
    output_device: Device,       // NEW
    input_config: StreamConfig,  // was: stream_config
    output_config: StreamConfig, // NEW
}
```

**Constructor changes:**

- Query both `default_input_device()` and `default_output_device()` from cpal
- Get default configs for both
- Reconcile sample rate: use the input device's sample rate; call `processor.set_sample_rate()` with it
- If input and output have different sample rates, log a warning and use the input rate for processing (output device typically resamples internally)

**`AudioHandle` changes:**

```
AudioHandle {
    _input_stream: Stream,
    _output_stream: Stream,
}
```

Both streams must be kept alive. Dropping either stops that half.

#### SPSC Ring Buffer

Use `rtrb::RingBuffer` (already a dependency of `wavecraft-metering`).

- **Capacity:** `buffer_size * num_channels * 4` samples — enough for ~4 audio blocks of latency headroom
- **Data format:** Interleaved `f32` samples (matches cpal output format)
- **Producer:** Input callback writes processed, interleaved samples
- **Consumer:** Output callback reads interleaved samples

If the ring buffer underflows (output runs ahead of input), write silence. If it overflows (input runs ahead), drop oldest samples. Both are non-blocking, real-time safe operations.

#### Input Callback (Pseudocode)

```
fn input_callback(data: &[f32], ring_producer, processor, meter_tx, ...) {
    // 1. Deinterleave cpal input → left[], right[]
    // 2. processor.process(&mut [&mut left, &mut right])
    // 3. Compute meters from left[], right[] (post-processing)
    // 4. Send meter update (non-blocking, via tokio mpsc channel)
    // 5. Interleave left[], right[] → interleaved[]
    // 6. ring_producer.write_chunk(interleaved) — non-blocking
    //    If ring is full, samples are dropped (acceptable — temporary glitch)
}
```

#### Output Callback (Pseudocode)

```
fn output_callback(data: &mut [f32], ring_consumer) {
    // 1. ring_consumer.read_chunk(data.len()) → samples
    // 2. If enough samples: copy to data
    // 3. If underflow: fill data with 0.0 (silence)
    //    No allocations, no locks.
}
```

#### Pre-allocated Buffers (Real-Time Safety)

The current implementation allocates `Vec`s inside the input callback:

```rust
// CURRENT (violates real-time safety):
let mut left = vec![0.0f32; num_samples];
let mut right = vec![0.0f32; num_samples];
```

This must be fixed. Options:

**Option A — Pre-allocate outside callback, move into closure:**

Allocate fixed-size buffers before `build_input_stream()` and move them into the closure. The buffers are reused every callback invocation. This requires knowing the maximum buffer size upfront.

```
let mut left_buf = vec![0.0f32; max_buffer_size];
let mut right_buf = vec![0.0f32; max_buffer_size];
// ... move into closure, slice as needed
```

**Option B — Use `rtrb::Producer::write_chunk_uninit()`:**

Write directly into the ring buffer's slots without intermediate buffers. More complex but avoids all temporary buffers.

**Recommendation:** Option A. It is straightforward, the `buffer_size` in `AudioConfig` provides the upper bound, and one allocation at startup is acceptable. Document the pre-allocation in the code.

### 2.4 Device Selection and Configuration

- Use `cpal::default_host()` for both input and output 
- Input: `host.default_input_device()` (already done)
- Output: `host.default_output_device()` (new)
- If no output device is available, fall back to metering-only mode with a clear log message
- Both streams use `f32` sample format (cpal's default)

### 2.5 Fallback Behavior

When `FfiProcessor` is not available (no vtable exported), behavior is unchanged:
- Input stream only, metering only
- No output stream opened 
- This preserves backward compatibility with older SDK plugins

---

## 3. Parameter Sync Mechanism (US-2)

### 3.1 Problem

The `InMemoryParameterHost` stores parameter values in a `RwLock<HashMap<String, f32>>`. The IPC handler's `setParameter()` writes to this map. The audio callback in `AudioServer` has no access to these values — `FfiProcessor::process()` is called with whatever the processor was initialized with.

Additionally, the FFI vtable's `process()` signature does not accept parameters:

```rust
pub process: extern "C" fn(
    instance: *mut c_void,
    channels: *mut *mut f32,
    num_channels: u32,
    num_samples: u32,
),
```

### 3.2 Design: Atomic Parameter Snapshot

**Core insight:** We do NOT need to extend the FFI vtable. The user's `Processor::process()` inside the dylib reads `params: &Self::Params` — but as documented in the high-level design, the `wavecraft_plugin!` macro always passes default values. This is a known limitation (out of scope for this milestone per user stories).

However, we can still make parameter changes audible for the **built-in processors** (Gain, Passthrough) by injecting parameter effects externally — i.e., applying gain changes in the audio callback *around* the processor call. This gives developers immediate audio feedback for basic parameter changes without requiring vtable modification.

**Alternative (applied here):** Since the macro limitation means the processor's own `process()` won't use the parameters anyway, the real value of parameter sync in dev mode is:

1. **UI ↔ Host round-trip verification** — Developers can verify `setParameter` / `getParameter` / `getAllParameters` work correctly
2. **Meter response** — When a gain parameter changes, the processed audio changes, and meters reflect it
3. **Future readiness** — When the macro limitation is fixed (v0.11+), the parameter bridge is already in place

**For this milestone**, the architecture focuses on making the `ParameterHost` values readable from the audio thread via lock-free atomics, and storing the bridge so it can be used when the vtable is extended in the future.

#### 3.2.1 `AtomicParameterBridge`

A new struct in `wavecraft-dev-server` that maps parameter IDs to `AtomicF32` values:

```
┌─────────────────────────────────────────────────────────────┐
│                    AtomicParameterBridge                     │
│                                                             │
│  HashMap<String, Arc<AtomicF32>>  (built once at startup)   │
│                                                             │
│  write(id, value)  ← called from WS thread (non-RT)        │
│  snapshot() → Vec<(String, f32)>  ← called from audio (RT) │
│  read(id) → f32    ← called from audio thread (RT-safe)    │
└─────────────────────────────────────────────────────────────┘
```

- **Construction:** At startup, create one `Arc<AtomicF32>` per parameter (from the loaded `ParameterInfo` list). Store in a `HashMap<String, Arc<AtomicF32>>`. The `HashMap` itself is never mutated after construction — only the `AtomicF32` values change.
- **Write path (WS thread):** `bridge.write("gain", 0.75)` → `AtomicF32::store(0.75, Ordering::Relaxed)`
- **Read path (audio thread):** `bridge.read("gain")` → `AtomicF32::load(Ordering::Relaxed)`
- Relaxed ordering is sufficient: parameter updates are not synchronization points. A one-block delay in parameter propagation is acceptable for dev mode.

#### 3.2.2 Integration with ParameterHost

The `DevServerHost` (in `cli/src/dev_server/host.rs`) will hold an `Arc<AtomicParameterBridge>`. When `set_parameter()` is called (from the IPC handler on the WS thread), it:

1. Updates the `InMemoryParameterHost` values (existing behavior — for `getParameter` / `getAllParameters` responses)
2. Writes to the `AtomicParameterBridge` (new — for audio thread reads)

The `AudioServer` receives a clone of `Arc<AtomicParameterBridge>` at construction time.

#### 3.2.3 Audio Callback Parameter Read

The audio callback reads parameter values at block boundaries:

```
fn input_callback(..., param_bridge: &AtomicParameterBridge) {
    // Read all parameters once per block (block-level updates, not sample-accurate)
    // In the future, these values will be passed to the processor.
    // For now, they are available for external gain application or logging.
    
    // 1. Deinterleave
    // 2. processor.process(...)
    // 3. (Future: apply parameter-driven gain externally if needed)
    // 4. Compute meters
    // 5. Write to output ring buffer
}
```

#### 3.2.4 `AtomicF32` Implementation

Use `std::sync::atomic::AtomicU32` with `f32::to_bits()` / `f32::from_bits()` for lock-free f32 storage. This is the standard pattern used throughout the Rust audio ecosystem (including nih-plug).

```rust
struct AtomicF32(AtomicU32);

impl AtomicF32 {
    fn new(val: f32) -> Self { Self(AtomicU32::new(val.to_bits())) }
    fn load(&self) -> f32 { f32::from_bits(self.0.load(Ordering::Relaxed)) }
    fn store(&self, val: f32) { self.0.store(val.to_bits(), Ordering::Relaxed); }
}
```

This is real-time safe: no allocations, no locks, single atomic load per parameter per block.

### 3.3 VTable Evolution (Future, Not This Milestone)

The current `DevProcessorVTable` (`version: 1`) does not accept parameters in `process()`. When the macro limitation is resolved (v0.11+), the vtable will be extended:

```rust
// Future vtable v2 (NOT implemented now):
pub struct DevProcessorVTable {
    pub version: u32,  // → 2
    pub create: ...,
    pub process: ...,
    pub set_sample_rate: ...,
    pub reset: ...,
    pub drop: ...,
    // NEW in v2:
    pub set_parameter: extern "C" fn(instance: *mut c_void, id: *const c_char, value: f32),
}
```

The version field enables graceful fallback: CLI checks `version >= 2` before calling `set_parameter`. Plugins compiled with v1 continue to work.

**This milestone does NOT change the vtable.** The `AtomicParameterBridge` infrastructure is placed now so that the audio callback can read parameter values; the actual injection into the processor will happen when vtable v2 ships.

---

## 4. Mocking Removal (US-3)

### 4.1 What Gets Deleted

| File | Action | Reason |
|------|--------|--------|
| `engine/crates/wavecraft-metering/src/dev.rs` | **Delete entirely** | `MeterGenerator` is synthetic fake data |
| `engine/crates/wavecraft-metering/src/lib.rs` | Remove `pub mod dev;` line | Module no longer exists |

### 4.2 What Gets Modified

| File | Change |
|------|--------|
| `cli/src/dev_server/host.rs` | Remove `MeterGenerator` usage, `MeterGeneratorProvider`, `Arc<RwLock<MeterGenerator>>` field. Replace with real meter data source (or `None` when no audio). |

### 4.3 Fallback Behavior Change

**Before:** When no FFI vtable is available, `DevServerHost` uses `MeterGenerator` to produce fake animated meter data → UI shows bouncing meters.

**After:** When no FFI vtable is available, `get_meter_frame()` returns `None` → UI shows zero/silent meters. This is the honest representation of "no audio processing is happening."

### 4.4 DevServerHost Redesign

The `DevServerHost` will be simplified:

```
DevServerHost {
    inner: InMemoryParameterHost,     // existing — parameter metadata & values
    param_bridge: Arc<AtomicParameterBridge>,  // NEW — lock-free param access for audio
    // REMOVED: meter_generator, MeterGeneratorProvider
}
```

When audio is running, meters come from the `AudioServer` → `meter_tx` → WebSocket broadcast pipeline (already implemented). The `ParameterHost::get_meter_frame()` method on `DevServerHost` will return `None` — meter data is pushed via notifications, not polled via `getMeterFrame` requests.

**Rationale:** In the current architecture, meter data flows through two separate paths:

1. **Push path (notifications):** Audio callback → `meter_tx` → tokio task → WebSocket broadcast → `meterUpdate` notification → browser `useMeterFrame()` hook
2. **Pull path (request/response):** Browser `getMeterFrame` request → `IpcHandler` → `ParameterHost::get_meter_frame()` → response

The push path (1) is the one that actually delivers real-time meter data. The pull path (2) was only used to serve the synthetic `MeterGenerator` data. With MeterGenerator removed, the pull path returns `None`, which is correct — real meters flow through notifications.

### 4.5 Out of Scope

These legitimate test doubles are **not removed**:

- `ui/packages/core/src/transports/MockTransport.ts` — test double for `IpcBridge` unit tests
- `ui/test/mocks/ipc.ts` — test infrastructure for React component tests
- `engine/crates/wavecraft-bridge/src/in_memory_host.rs` — `InMemoryParameterHost` is production code used by the dev server, not a mock

---

## 5. UI Hook Changes (US-4)

### 5.1 Problem

`useAllParameters()` calls `reload()` once on mount via `useEffect`. If the WebSocket hasn't connected yet (common on page load), the `ParameterClient.getAllParameters()` call throws (`IpcBridge: Transport not connected`), the error is caught, and `params` stays empty. There is no retry.

### 5.2 Design: Connection-Aware Retry

Add `useConnectionStatus()` as a dependency of `useAllParameters()`. When connection status transitions from disconnected → connected, trigger a reload.

```
useAllParameters() {
    const { connected } = useConnectionStatus();
    
    // Existing: load on mount
    useEffect(() => { reload(); }, [reload]);
    
    // NEW: reload when connection is (re-)established
    useEffect(() => {
        if (connected) {
            reload();
        }
    }, [connected, reload]);
    
    // Existing: subscribe to parameterChanged notifications
    useEffect(() => { ... }, []);
}
```

### 5.3 Deduplication Guarantee

When the WebSocket is already connected at mount time:
- First `useEffect` fires → `reload()` → parameters loaded
- Second `useEffect` also fires (because `connected` is `true`) → `reload()` again

This results in a duplicate request on initial load. This is acceptable because:
1. `getAllParameters` is idempotent
2. It happens once (not repeatedly)
3. The second call overwrites the first result with identical data

If deduplication is desired, a `hasLoaded` ref can prevent the connection-effect from firing when data is already present. However, this adds complexity for negligible benefit.

### 5.4 Reconnection Behavior

When the WebSocket drops and reconnects:
- `useConnectionStatus` polling detects `connected: false` → `connected: true` transition
- The connection-aware `useEffect` fires → `reload()` → parameters re-fetched
- This handles the case where parameter state may have changed during disconnection

### 5.5 No Busy-Waiting

The retry is event-driven via React's dependency tracking. `useConnectionStatus` polls the transport every 1 second (existing behavior), but `useAllParameters` does NOT poll — it reacts to state changes in the connection status hook.

**Alternative considered:** Add a `connected` event to `WebSocketTransport` and subscribe via `IpcBridge.on('connected', ...)`. This is cleaner but requires new plumbing through the transport → bridge → hook chain. The `useConnectionStatus` dependency approach is simpler and sufficient for dev mode.

---

## 6. Real-Time Safety Analysis

### 6.1 Audio Thread (Input Callback)

| Operation | RT-Safe? | Notes |
|-----------|----------|-------|
| Deinterleave into pre-allocated buffers | ✅ | No allocation; buffers allocated at startup |
| `FfiProcessor::process()` | ✅ | FFI call through vtable; user's DSP runs |
| `AtomicF32::load()` per parameter | ✅ | Single atomic read; `Ordering::Relaxed` |
| Meter computation (peak, RMS) | ✅ | Arithmetic only |
| `meter_tx.send()` (tokio mpsc) | ✅ | Non-blocking; drops if receiver is behind |
| `ring_producer.push()` / `write_chunk()` | ✅ | Lock-free SPSC; drops on overflow |

### 6.2 Audio Thread (Output Callback)

| Operation | RT-Safe? | Notes |
|-----------|----------|-------|
| `ring_consumer.read_chunk()` | ✅ | Lock-free SPSC; returns silence on underflow |
| Fill output buffer | ✅ | memcpy from ring or zero-fill |

### 6.3 WebSocket Thread (Parameter Write)

| Operation | RT-Safe? | N/A — not audio thread |
|-----------|----------|------------------------|
| `RwLock::write()` on `InMemoryParameterHost` | N/A | WS thread may block; acceptable |
| `AtomicF32::store()` on parameter bridge | Lock-free | But not on audio thread anyway |

### 6.4 Current Violation to Fix

The current `audio_server.rs` input callback contains:

```rust
let mut left = vec![0.0f32; num_samples];   // ALLOCATION on audio thread
let mut right = vec![0.0f32; num_samples];  // ALLOCATION on audio thread
let mut ptrs: Vec<*mut f32> = ...;          // ALLOCATION (in FfiProcessor::process)
```

These must be replaced with pre-allocated buffers moved into the closure. The `FfiProcessor::process()` method also allocates a `Vec<*mut f32>` for channel pointers — this should be pre-allocated as well (a fixed `[*mut f32; 2]` array for stereo, or a small pre-allocated vec).

---

## 7. Complete Data Flow Diagrams

### 7.1 Audio Data Flow

```
┌───────────────────────────────────────────────────────────────────────┐
│                        AUDIO DATA FLOW                                │
│                                                                       │
│  ┌──────────┐   interleaved    ┌────────────────────────────────┐     │
│  │ OS Input │ ───────────────► │     Input Callback             │     │
│  │ (Mic)    │   f32 samples    │                                │     │
│  └──────────┘                  │  1. Deinterleave → L[], R[]    │     │
│                                │  2. AtomicF32 param reads      │     │
│                                │  3. FfiProcessor::process()    │     │
│                                │  4. Compute peak/RMS meters    │     │
│                                │  5. meter_tx.send(notification)│     │
│                                │  6. Interleave → ring_producer │     │
│                                └────────────────┬───────────────┘     │
│                                                 │                     │
│                                          SPSC Ring Buffer             │
│                                          (rtrb, lock-free)            │
│                                                 │                     │
│                                ┌────────────────┴───────────────┐     │
│                                │     Output Callback            │     │
│  ┌───────────┐  interleaved    │                                │     │
│  │ OS Output │ ◄────────────── │  1. ring_consumer.read_chunk() │     │
│  │ (Speakers)│   f32 samples   │  2. Copy to output (or silence)│     │
│  └───────────┘                 └────────────────────────────────┘     │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### 7.2 Parameter Sync Flow

```
┌───────────────────────────────────────────────────────────────────────┐
│                      PARAMETER SYNC FLOW                              │
│                                                                       │
│  Browser UI                                                           │
│  ┌─────────────────┐                                                  │
│  │ ParameterSlider  │                                                  │
│  │ onChange(0.75)   │                                                  │
│  └────────┬────────┘                                                  │
│           │ setParameter("gain", 0.75)                                │
│           ▼                                                           │
│  ┌─────────────────┐                                                  │
│  │ WebSocket        │                                                  │
│  │ Transport       │                                                  │
│  └────────┬────────┘                                                  │
│           │ JSON-RPC over ws://                                       │
│           ▼                                                           │
│  ┌─────────────────┐                                                  │
│  │ WsServer         │  ──► IpcHandler::handle_json()                  │
│  │ (tokio thread)  │                                                  │
│  └────────┬────────┘                                                  │
│           │                                                           │
│           ▼                                                           │
│  ┌─────────────────────────────────────────────┐                      │
│  │ DevServerHost::set_parameter("gain", 0.75)  │                      │
│  │                                              │                      │
│  │  ① InMemoryParameterHost.write(RwLock)      │  ← for IPC queries  │
│  │  ② AtomicParameterBridge.write(AtomicF32)   │  ← for audio thread │
│  └─────────────────────────────────────────────┘                      │
│                          │                                            │
│                   ②: AtomicF32::store(0.75, Relaxed)                  │
│                          │                                            │
│                          ▼                                            │
│  ┌─────────────────────────────────────────────┐                      │
│  │ Audio Input Callback (next block)           │                      │
│  │                                              │                      │
│  │  param_bridge.read("gain") → 0.75           │  ← AtomicF32::load  │
│  │  (available for future vtable v2 injection) │                      │
│  └─────────────────────────────────────────────┘                      │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

### 7.3 UI Parameter Load Flow (with retry)

```
┌───────────────────────────────────────────────────────────────────────┐
│                 UI PARAMETER LOAD (WITH RETRY)                        │
│                                                                       │
│  Component Mount                                                      │
│  ┌──────────────────────────────┐                                     │
│  │ useAllParameters()           │                                     │
│  │                              │                                     │
│  │  useEffect([], reload)  ─────┼──► getAllParameters()               │
│  │                              │     │                               │
│  │                              │     ├── connected? → SUCCESS        │
│  │                              │     └── not connected? → catch →    │
│  │                              │         error set, params empty     │
│  │                              │                                     │
│  │  useConnectionStatus()  ─────┼──► { connected: false }            │
│  │                              │                                     │
│  │  useEffect([connected],  ────┼──► (no-op while disconnected)      │
│  │    reload)                   │                                     │
│  └──────────────────────────────┘                                     │
│                                                                       │
│  WebSocket connects (1-5 seconds later)                               │
│  ┌──────────────────────────────┐                                     │
│  │ useConnectionStatus()        │                                     │
│  │  polls every 1s              │                                     │
│  │  → connected: true           │    state change triggers re-render  │
│  │                              │                                     │
│  │  useEffect([connected],  ────┼──► reload() → getAllParameters()    │
│  │    reload)                   │     → SUCCESS → params populated    │
│  └──────────────────────────────┘                                     │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

---

## 8. Affected Crates and Packages

### 8.1 Rust Crates

| Crate | File(s) | Changes |
|-------|---------|---------|
| `wavecraft-dev-server` | `src/audio_server.rs` | Add output stream, SPSC ring buffer, pre-allocated buffers, accept `AtomicParameterBridge` |
| `wavecraft-dev-server` | `src/ffi_processor.rs` | Fix `Vec` allocation in `process()` — use pre-allocated channel pointer array or accept a buffer |
| `wavecraft-dev-server` | `src/lib.rs` | Export `AtomicParameterBridge` (new module) |
| `wavecraft-dev-server` | `src/atomic_params.rs` (new) | `AtomicParameterBridge` struct + `AtomicF32` |
| `wavecraft-metering` | `src/dev.rs` | **Delete file** |
| `wavecraft-metering` | `src/lib.rs` | Remove `pub mod dev;` |
| `wavecraft-bridge` | `src/in_memory_host.rs` | No changes (still used for IPC queries) |

### 8.2 CLI

| File | Changes |
|------|---------|
| `cli/src/dev_server/host.rs` | Remove `MeterGenerator` usage; add `AtomicParameterBridge`; update `set_parameter` to write atomics |
| `cli/src/commands/start.rs` | Pass `AtomicParameterBridge` to `AudioServer`; update `try_start_audio_in_process()` |

### 8.3 UI Packages

| Package | File | Changes |
|---------|------|---------|
| `@wavecraft/core` | `src/hooks/useAllParameters.ts` | Add `useConnectionStatus` dependency for connection-aware retry |

### 8.4 No Changes Required

| Component | Reason |
|-----------|--------|
| `wavecraft-protocol` | VTable v1 unchanged; `MeterUpdateNotification` unchanged |
| `wavecraft-bridge/handler.rs` | IPC dispatch unchanged |
| `wavecraft-nih_plug` | Plugin mode unaffected |
| `@wavecraft/components` | Components consume hooks; no internal changes needed |

---

## 9. Implementation Notes

### 9.1 FfiProcessor Allocation Fix

The current `FfiProcessor::process()` allocates on every call:

```rust
let mut ptrs: Vec<*mut f32> = channels.iter_mut().map(|ch| ch.as_mut_ptr()).collect();
```

This should be replaced with a stack-allocated array. Since Wavecraft targets stereo (2 channels), a `[*mut f32; 2]` is sufficient. For future multi-channel support, a small `SmallVec<[*mut f32; 8]>` could be used, but for now a fixed array avoids the dependency.

Alternatively, the channel pointer array can be pre-allocated in the audio callback (which already knows the channel count) and passed into the processor via a new method signature change on `DevAudioProcessor`:

```rust
pub trait DevAudioProcessor: Send + 'static {
    fn process(&mut self, channels: &mut [&mut [f32]]);
    // ...
}
```

The trait signature is fine — the issue is only in `FfiProcessor::process()`. The fix is internal to that method.

### 9.2 Thread Model Summary

| Thread | Owns | Accesses (shared) |
|--------|------|-------------------|
| Main thread | `PluginParamLoader`, startup orchestration | N/A |
| Tokio runtime (WS) | `WsServer`, `IpcHandler<DevServerHost>` | `Arc<AtomicParameterBridge>` (writes) |
| cpal input callback | `FfiProcessor`, pre-allocated buffers, `rtrb::Producer` | `Arc<AtomicParameterBridge>` (reads) |
| cpal output callback | `rtrb::Consumer` | N/A |
| Tokio task (meter fwd) | meter_rx receiver | WsHandle (broadcasts) |

### 9.3 Startup Sequence (Updated)

```
1. Build plugin (cargo build)
2. Load dylib → PluginParamLoader (params + optional vtable)
3. Create AtomicParameterBridge from parameter metadata
4. Create DevServerHost(params, Arc<AtomicParameterBridge>)
5. Create IpcHandler(DevServerHost)
6. Start WsServer
7. If vtable available:
   a. Create FfiProcessor from vtable
   b. Create AudioServer(processor, config, Arc<AtomicParameterBridge>)
   c. Create SPSC ring buffer
   d. Start input stream (mic → process → ring buffer)
   e. Start output stream (ring buffer → speakers)
   f. Start meter forwarding task
8. Start Vite UI server
9. Wait for Ctrl+C
```

---

## 10. Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| **Input/output sample rate mismatch** | Medium | Low | Use input device's sample rate for processing; log warning if output differs. Most macOS setups use a unified sample rate via CoreAudio. |
| **Ring buffer sizing** — too small causes dropouts, too large adds latency | Medium | Medium | Default to `buffer_size * channels * 4`. Make configurable via `AudioConfig`. Log underflow/overflow counts for debugging. |
| **cpal callback thread starvation** — output runs ahead of input | Low | Low | Output callback writes silence on underflow. Occasional silence glitches are acceptable in dev mode. |
| **Parameter ordering** — AtomicF32 updates not visible to audio thread | Low | Very Low | Relaxed ordering is sufficient for block-level parameter updates. One-block delay is imperceptible. |
| **FfiProcessor panic in audio callback** | High | Low | `FfiProcessor` vtable functions are wrapped in `catch_unwind` (macro-generated). If panic occurs, audio buffer is left unmodified. |
| **Removing MeterGenerator breaks tests** | Low | Low | Remove the `wavecraft-metering/src/dev.rs` tests along with the file. Update `cli/src/dev_server/host.rs` tests to not expect synthetic meters. |
| **useAllParameters double-fetch on startup** | Very Low | Certain | Idempotent operation, negligible cost. Could add `hasLoaded` ref guard if needed, but unnecessary. |
| **cpal no output device available** | Medium | Low | Graceful fallback: log warning, continue in metering-only mode. Same pattern as current no-input-device fallback. |
| **Breaking wavecraft-metering public API** | Medium | Low | `MeterGenerator` is only used by `cli/src/dev_server/host.rs`. Not part of the SDK's public surface for end users. Verify no references in `wavecraft-nih_plug` or published crate APIs. |

---

## 11. Testing Strategy

### 11.1 Unit Tests

| Test | Crate | Description |
|------|-------|-------------|
| `AtomicParameterBridge` write/read | `wavecraft-dev-server` | Verify atomic write on one thread, read returns latest value on another |
| `AtomicParameterBridge` multi-param | `wavecraft-dev-server` | Multiple parameters updated concurrently |
| `DevServerHost` without MeterGenerator | `cli` | `get_meter_frame()` returns `None` |
| `DevServerHost` param sync | `cli` | `set_parameter()` updates both `InMemoryParameterHost` and `AtomicParameterBridge` |
| `useAllParameters` retry | `@wavecraft/core` | Mock `useConnectionStatus` transition; verify `reload()` called |
| Removed `dev.rs` tests | `wavecraft-metering` | Deleted with the module |

### 11.2 Integration Tests

| Test | Description |
|------|-------------|
| `wavecraft start` with audio plugin | Plugin with vtable → audio output audible, meters show real data |
| `wavecraft start` without vtable | Older plugin → metering-only, meters show zeros, no crash |
| Parameter slider → audio change | Move slider in browser → `setParameter` → `AtomicParameterBridge` → verify atomic value updated |
| UI reconnection | Kill WS server → restart → parameters re-fetched automatically |

### 11.3 Manual Testing

| Test | Method |
|------|--------|
| Audio output audible | Run `wavecraft start` → speak into mic → hear processed audio through speakers |
| Latency acceptable | Subjective test — monitor feedback should feel "live" (< 20ms) |
| No glitches | Run for 60 seconds under normal conditions; no clicks/pops |
| UI parameters load on cold start | Open browser before WS connects → parameters appear within 1-2 seconds |

---

## 12. Migration and Backward Compatibility

- **FFI vtable v1 unchanged:** Existing plugins compiled with current SDK continue to work. No recompilation required.
- **MeterGenerator removal:** Only affects internal CLI code. Not exposed in published crate APIs.
- **UI hook change:** `useAllParameters` API (return type, behavior) is additive — existing consumers get better behavior without code changes.
- **Version bump to 0.10.0:** Signifies behavioral change (audio output in dev mode). No breaking API changes.
