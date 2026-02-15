# Low-Level Design: Oscilloscope (v1)

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — system architecture and runtime constraints
- [Coding Standards](../../architecture/coding-standards.md) — repository-wide coding and API conventions
- [Roadmap](../../roadmap.md) — milestone tracking and sequencing

## Overview

This feature adds a **developer-focused, observation-only oscilloscope** to Wavecraft with two concrete deliverables:

1. An engine-side oscilloscope tap processor in `engine/crates/wavecraft-processors`
2. A reusable React oscilloscope component in `ui/packages/components`

The oscilloscope never alters audio. It captures a tap of the signal at a defined signal-chain position, stores fixed-size snapshots (1024 points), and exposes them to UI via IPC so rendering works in both browser-dev (WebSocket transport) and plugin WebView (native transport).

v1 prioritizes simplicity: one trigger mode required (default rising zero-crossing), channel view selector, and requestAnimationFrame-driven rendering at ~60 fps.

## Goals / Non-goals

### Goals

- Add a **passthrough oscilloscope tap** that does not modify sample values, timing, or routing.
- Support **tap-position concept**: inspect output of previous item in signal chain.
- Provide **channel view selection**:
  - Default: L/R overlay in one graph
  - Also support per-channel view (L only, R only)
- Implement **trigger mode** (required), defaulting to rising zero-crossing.
- Use **1024-point frames** for waveform snapshots.
- Render in UI using **requestAnimationFrame (~60 fps)**.
- Work identically in:
  - Browser-dev mode (WebSocket path)
  - Plugin WebView mode (native IPC path)
- No-signal UX: flat line + “No signal” label.
- Always enabled in runtime (no feature flags or build-time gates).

### Non-goals (v1)

- FFT/spectrum, spectrogram, or persistence trails
- Multi-trigger types beyond required default mode
- Timebase/zoom/pan controls
- History recording/export
- Advanced anti-aliased/vector rendering optimizations beyond basic canvas/SVG clarity

## Architecture

### High-level design

Add an **observation processor** (`OscilloscopeTap`) to the processor chain that reads the incoming buffer (post previous item), copies downsampled/selected samples into an internal frame buffer, and forwards audio unchanged.

A dedicated oscilloscope frame contract is added to IPC. UI polling retrieves the latest frame and renders continuously at RAF cadence.

### Engine responsibilities

- `OscilloscopeTap` runs in DSP graph as a passthrough processor.
- Captures 1024-point stereo frame snapshots at tap position.
- Maintains latest frame in lock-free or RT-safe shared state for host/UI reads.
- Trigger alignment (rising zero-crossing) is applied before publishing frame.
- Exposes “no signal” metadata when amplitude stays below threshold for full frame window.

### Bridge/protocol responsibilities

- Extend protocol with `getOscilloscopeFrame` method and typed frame payload.
- Extend bridge host contract to provide latest oscilloscope frame.
- Ensure both plugin host bridge and dev server host implement the new contract.

### UI responsibilities

- Add reusable `Oscilloscope` component in `@wavecraft/components`.
- Add core hook/client in `@wavecraft/core` to fetch oscilloscope frames.
- UI state handles:
  - Channel view selection (overlay/L/R)
  - Trigger mode selection (default rising zero-crossing for v1)
  - No-signal display state
- Rendering loop driven by `requestAnimationFrame`.

## Data flow

1. Audio enters processor chain.
2. Previous processor writes output buffer.
3. `OscilloscopeTap` reads that output as tap input, copies/normalizes window into 1024-point snapshot, then forwards the same samples unchanged.
4. Snapshot is published into latest-frame shared state (single latest frame semantics; stale frames can be overwritten).
5. UI render loop requests latest frame through IPC (`getOscilloscopeFrame`) at RAF cadence.
6. Frame is rendered in selected channel mode:
   - Overlay (L+R, default)
   - L only
   - R only
7. If frame reports no signal, UI renders flat center line and “No signal” label.

## API / Contract changes

### Rust protocol (`wavecraft-protocol`)

Add new IPC constants/types in `engine/crates/wavecraft-protocol/src/ipc.rs` and re-export from `lib.rs`:

- Method constant: `METHOD_GET_OSCILLOSCOPE_FRAME = "getOscilloscopeFrame"`
- Optional notification constant for future push mode (not required for v1): `NOTIFICATION_OSCILLOSCOPE_UPDATE` (reserved, not used in v1)
- Types:
  - `OscilloscopeTriggerMode` (v1 includes `RisingZeroCrossing`)
  - `OscilloscopeChannelView` (`Overlay`, `Left`, `Right`)
  - `OscilloscopeFrame`:
    - `points_l: [f32; 1024]` (or serialized vec length 1024)
    - `points_r: [f32; 1024]`
    - `sample_rate: f32`
    - `timestamp: u64`
    - `no_signal: bool`
    - `trigger_mode: OscilloscopeTriggerMode`
  - `GetOscilloscopeFrameResult { frame: Option<OscilloscopeFrame> }`

### Bridge contract (`wavecraft-bridge`)

Extend `ParameterHost` trait with:

- `fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame>;`

Extend `IpcHandler` dispatch with new method handler for `getOscilloscopeFrame`.

All `ParameterHost` implementations must be updated (plugin editor bridge, in-memory host, dev server host, tests/mocks).

### TypeScript core (`@wavecraft/core`)

Add matching types and method constants in core IPC types:

- `OscilloscopeFrame`, `GetOscilloscopeFrameResult`
- `METHOD_GET_OSCILLOSCOPE_FRAME`

Add API entrypoints:

- `getOscilloscopeFrame()` helper (parallel to `getMeterFrame`)
- `useOscilloscopeFrame()` hook (polling cadence suitable for RAF-driven UI)

## File impact map

### Engine / Protocol / Bridge

- `engine/crates/wavecraft-processors/src/lib.rs` — export new processor
- `engine/crates/wavecraft-processors/src/oscilloscope.rs` — new passthrough tap processor (new)
- `engine/crates/wavecraft-protocol/src/ipc.rs` — add oscilloscope method/constants/types
- `engine/crates/wavecraft-protocol/src/lib.rs` — re-export new protocol types/constants
- `engine/crates/wavecraft-bridge/src/host.rs` — extend `ParameterHost` trait
- `engine/crates/wavecraft-bridge/src/handler.rs` — route and serve `getOscilloscopeFrame`
- `engine/crates/wavecraft-bridge/src/in_memory_host.rs` — trait impl update + provider support
- `engine/crates/wavecraft-nih_plug/src/editor/bridge.rs` — plugin host implementation for oscilloscope frame

### Dev server (browser-dev support)

- `dev-server/src/host.rs` — store/provide latest oscilloscope frame
- `dev-server/src/ws/mod.rs` — no protocol change needed for request/response path (optional future broadcast reserved)
- `dev-server/src/audio/server.rs` — feed oscilloscope tap/source frame path in dev runtime

### UI core package (`@wavecraft/core`)

- `ui/packages/core/src/types/ipc.ts` — add method constant + frame types
- `ui/packages/core/src/index.ts` — export new types/method/hook
- `ui/packages/core/src/oscilloscope-ipc.ts` — new request helper (new)
- `ui/packages/core/src/hooks/useOscilloscopeFrame.ts` — new hook (new)
- Tests:
  - `ui/packages/core/src/hooks/useOscilloscopeFrame.test.ts`
  - `ui/packages/core/src/IpcBridge.test.ts` updates for method coverage

### UI components package (`@wavecraft/components`)

- `ui/packages/components/src/Oscilloscope.tsx` — reusable component (new)
- `ui/packages/components/src/index.ts` — export component
- `ui/packages/components/src/Oscilloscope.test.tsx` — component behavior tests (new)

## Realtime safety constraints

- Audio thread rules remain strict:
  - No allocation
  - No locks
  - No blocking I/O/syscalls
  - No logging in callback/process path
- `OscilloscopeTap` must be pure passthrough for output buffer.
- Frame capture uses preallocated storage and lock-free / wait-free publication pattern.
- Dropped or overwritten oscilloscope frames are acceptable; audio correctness is not.
- Trigger detection must run in bounded, deterministic time over fixed window (1024 points).
- UI polling rate must never back-pressure audio thread state production.

## UX behavior

- Default render mode: **L/R overlay** in one graph.
- Channel selector options:
  - Overlay (default)
  - Left
  - Right
- Trigger mode:
  - Required control present
  - v1 default and only implemented mode: **Rising zero-crossing**
- Rendering:
  - Driven by requestAnimationFrame (~60 fps)
  - Most recent available frame displayed each repaint
- No signal:
  - Draw center flat line
  - Show “No signal” label
- Scope is always available when UI is connected; no feature toggle.

## Testing strategy

### Engine tests

- Unit tests for `OscilloscopeTap`:
  - Passthrough invariance: output equals input bit-for-bit/tolerance
  - Frame length exactly 1024
  - Trigger alignment (rising zero-crossing behavior)
  - No-signal detection threshold behavior
  - Stereo capture integrity (L/R distinctness preserved)

### Bridge/protocol tests

- Protocol serialization/deserialization tests for new oscilloscope types.
- IPC handler tests:
  - `getOscilloscopeFrame` success
  - no-frame (`null`) response behavior
- Trait impl tests for all host implementations compile and return expected types.

### Dev server tests

- Host storage/retrieval tests for oscilloscope frame.
- Browser-dev integration tests validating `getOscilloscopeFrame` round-trip over WebSocket.

### UI tests

- Core hook tests for polling cadence/error resilience.
- Component tests:
  - Overlay/L/R mode switching
  - Trigger mode default state
  - No-signal rendering (flat line + label)
  - Rendering loop lifecycle (mount/unmount cleanup)

### End-to-end validation

- Browser-dev run: visible oscilloscope updates with live audio.
- Plugin WebView run: visible oscilloscope updates in DAW.
- Re-run standard checks: `cargo xtask ci-check`.

## Risks / mitigations

- **Risk: hidden RT regressions from capture work**
  - Mitigation: fixed-size preallocation, bounded loops, no dynamic memory in process path.
- **Risk: transport rate mismatch at 60 fps**
  - Mitigation: single latest-frame semantics; UI drops stale frames.
- **Risk: trigger instability on low-level/noisy signals**
  - Mitigation: deterministic threshold + explicit no-signal fallback.
- **Risk: host implementation drift (multiple `ParameterHost` impls)**
  - Mitigation: compiler-enforced trait extension + integration tests per host.
- **Risk: browser-dev and plugin behavior divergence**
  - Mitigation: shared IPC method and shared core hook/component usage in both modes.

## Rollout notes

- Always enabled from initial merge (no feature flag).
- Backward compatibility:
  - Pre-1.0 contract policy applies; clients and engine in repo move together.
  - New IPC method is additive and low-risk for existing calls.
- Rollout order:
  1. Protocol + bridge trait/handler
  2. Engine tap processor + host wiring
  3. Core TS types/hook/client
  4. Reusable component + tests
  5. Browser-dev and plugin verification
- v1 scope guard: avoid adding FFT/history/advanced controls in same PR stream.

## Implementation checklist (status)

- [x] Requirements mapped to architecture and constraints
- [x] Concrete contract/API changes specified
- [x] File impact map scoped across engine, bridge, dev-server, and UI packages
- [x] Realtime safety rules and failure modes documented
- [x] Test strategy and rollout plan documented
