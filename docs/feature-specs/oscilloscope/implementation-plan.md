# Implementation Plan: Oscilloscope (v1)

## Planning Checklist (this document)

- [x] Requirements analyzed from `docs/feature-specs/oscilloscope/low-level-design-oscilloscope.md`
- [x] Affected files mapped across engine/protocol/bridge/dev-server/ui-core/ui-components
- [x] Phased implementation order defined with explicit dependencies
- [x] Per-phase testing strategy and verification checkpoints included
- [x] Realtime safety and transport-performance risk controls included
- [x] Single-PR delivery strategy included
- [x] Validation commands included
- [x] Definition of Done checklist included

## Overview

This plan delivers a **read-only, passthrough oscilloscope** end-to-end from Rust DSP to React UI using the existing JSON-RPC transport path.  
The implementation is structured to protect audio-thread correctness first, then add contract/bridge plumbing, then UI APIs/components, and finally integration hardening and CI validation.

## Scope and Constraints

### In scope (v1)

- Add `OscilloscopeTap` processor (1024-point snapshots, stereo)
- Add IPC method `getOscilloscopeFrame`
- Support channel view: `overlay` (default), `left`, `right`
- Support trigger mode control with v1 default/only mode: rising zero-crossing
- RAF-driven rendering in UI (`~60fps`)
- No-signal state: flat line + “No signal”
- Works in both:
  - browser-dev mode (WebSocket transport)
  - plugin WebView mode (native transport)

### Out of scope (v1)

- FFT/spectrum/spectrogram
- history, persistence trails, export
- zoom/pan/timebase controls
- push-notification oscilloscope streaming (keep pull-based `getOscilloscopeFrame`)

---

## Dependency Graph (Phase Order)

1. **Phase 1: Protocol + Bridge Contract**
2. **Phase 2: Engine Tap + Host Wiring**
3. **Phase 3: Dev Server Wiring**
4. **Phase 4: UI Core API + Hook**
5. **Phase 5: UI Component**
6. **Phase 6: Integration Hardening + CI**

Hard dependencies:

- Phase 2 depends on Phase 1 (trait + method contract must exist)
- Phase 3 depends on Phase 1 and partially Phase 2 (frame type + producer path)
- Phase 4 depends on Phase 1
- Phase 5 depends on Phase 4
- Phase 6 depends on all prior phases

---

## Phase 1 — Protocol and Bridge Contract Foundation

**Goal:** Introduce stable oscilloscope IPC contract and bridge dispatch path first.

### File-by-file tasks

1. **`engine/crates/wavecraft-protocol/src/ipc.rs`**
   - Add:
     - `METHOD_GET_OSCILLOSCOPE_FRAME`
     - `OscilloscopeTriggerMode` enum (`RisingZeroCrossing`)
     - `OscilloscopeChannelView` enum (`Overlay`, `Left`, `Right`)
     - `OscilloscopeFrame` payload
     - `GetOscilloscopeFrameResult { frame: Option<OscilloscopeFrame> }`
   - Add serialization tests for new types and method behavior.

2. **`engine/crates/wavecraft-protocol/src/lib.rs`**
   - Re-export new oscilloscope constants/types from `ipc`.

3. **`engine/crates/wavecraft-bridge/src/host.rs`**
   - Extend `ParameterHost` trait:
     - `fn get_oscilloscope_frame(&self) -> Option<OscilloscopeFrame>;`
   - Update `impl<T: ParameterHost> ParameterHost for Arc<T>` passthrough.

4. **`engine/crates/wavecraft-bridge/src/handler.rs`**
   - Add dispatch branch for `METHOD_GET_OSCILLOSCOPE_FRAME`.
   - Add `handle_get_oscilloscope_frame()` returning `GetOscilloscopeFrameResult`.
   - Extend existing mock host tests to validate:
     - success with frame
     - success with `None` frame

5. **`engine/crates/wavecraft-bridge/src/in_memory_host.rs`**
   - Add optional oscilloscope provider abstraction parallel to existing `MeterProvider`.
   - Implement `get_oscilloscope_frame()` in `InMemoryParameterHost`.
   - Add unit tests for provider and retrieval behavior.

### Testing in this phase

- Unit:
  - protocol serde tests (`wavecraft-protocol`)
  - bridge handler tests (`wavecraft-bridge`)
  - in-memory host tests (`wavecraft-bridge`)
- Integration:
  - none yet (contract-only phase)

### Verification checkpoint

- `getOscilloscopeFrame` method compiles through protocol + bridge layers.
- All `ParameterHost` implementers fail-fast at compile time until updated (expected, intentional).

### Exit criteria

- New protocol types/constants compile and are re-exported.
- Bridge dispatch includes `getOscilloscopeFrame`.
- Bridge and protocol tests pass.

---

## Phase 2 — Engine Tap Processor and Plugin Host Wiring

**Goal:** Produce RT-safe oscilloscope frames without altering audio output.

### File-by-file tasks

1. **`engine/crates/wavecraft-processors/src/oscilloscope.rs` (new)**
   - Implement `OscilloscopeTap`:
     - passthrough processing (input == output)
     - fixed-size preallocated snapshot buffers (1024)
     - rising zero-crossing trigger alignment
     - no-signal detection threshold
     - latest-frame publication (drop/overwrite acceptable)

2. **`engine/crates/wavecraft-processors/src/lib.rs`**
   - Export `OscilloscopeTap` (and config types if needed).

3. **`engine/crates/wavecraft-nih_plug/src/editor/bridge.rs`**
   - Extend `PluginEditorBridge` to provide `get_oscilloscope_frame()`.
   - Thread-safe read of latest oscilloscope frame source (parallel to meter consumer pattern).
   - Keep `get_audio_status()` behavior unchanged (`None` in plugin host).

4. **Engine-side wiring (where processor chain is assembled)**
   - Ensure oscilloscope tap is inserted at required tap position (post previous processor output).
   - Confirm frame source is accessible from editor bridge.
   - If chain assembly lives outside files above, include corresponding chain builder file(s) in implementation PR.

### Testing in this phase

- Unit (`wavecraft-processors`):
  - passthrough invariance
  - 1024 frame length guarantee
  - trigger alignment behavior
  - no-signal threshold behavior
  - stereo channel integrity (distinct L/R preserved)
- Integration (`wavecraft-nih_plug`):
  - bridge returns latest frame when available
  - returns `None` when unavailable

### Verification checkpoint

- Audio output remains unchanged by oscilloscope tap under test vectors.
- Frame generation deterministic and bounded-time for fixed 1024-window.

### Exit criteria

- `OscilloscopeTap` integrated and exported.
- Plugin host implementation compiles with extended `ParameterHost` trait.
- Processor/bridge tests pass.

---

## Phase 3 — Dev Server Support (Browser Mode)

**Goal:** Ensure browser-dev host can serve oscilloscope frames over existing request/response IPC.

### File-by-file tasks

1. **`dev-server/src/host.rs`**
   - Add `latest_oscilloscope_frame: Arc<RwLock<Option<OscilloscopeFrame>>>`.
   - Add setter `set_latest_oscilloscope_frame(...)`.
   - Implement `get_oscilloscope_frame()` in `ParameterHost` impl.
   - Add tests paralleling existing meter frame tests.

2. **`dev-server/src/audio/server.rs`**
   - Add oscilloscope capture feed from processed stereo buffers.
   - Keep RT constraints:
     - no allocations in callback
     - fixed-size buffers reused
     - non-blocking publication
   - Ensure meter path remains intact; oscilloscope path must not regress meter cadence.

3. **`dev-server/src/ws/mod.rs`** (optional/no behavior change expected)
   - No protocol transport changes required for pull-based method.
   - Only touch if dispatch glue requires explicit method registration (confirm by compile/tests).

### Testing in this phase

- Unit (`dev-server/src/host.rs` tests):
  - set/get oscilloscope frame behavior
- Integration (`dev-server` tests):
  - `getOscilloscopeFrame` round-trip over WebSocket JSON-RPC
  - null frame response behavior when no data yet

### Verification checkpoint

- Browser-dev host serves oscilloscope frame through same IPC path as meters/params.
- No regressions in existing audio status/meter behavior.

### Exit criteria

- Dev server compiles with new trait method.
- New dev-server tests pass.
- Existing dev-server tests remain green.

---

## Phase 4 — UI Core (`@wavecraft/core`) API and Hook

**Goal:** Expose typed oscilloscope client API to UI consumers.

### File-by-file tasks

1. **`ui/packages/core/src/types/ipc.ts`**
   - Add `METHOD_GET_OSCILLOSCOPE_FRAME` constant.
   - Option A (LLD-aligned): add oscilloscope interfaces here.
   - Option B (preferred with existing structure): keep method constant here and add types in dedicated file (next step).

2. **`ui/packages/core/src/types/oscilloscope.ts` (new)**
   - Define:
     - `OscilloscopeTriggerMode`
     - `OscilloscopeChannelView`
     - `OscilloscopeFrame`
     - `GetOscilloscopeFrameResult`

3. **`ui/packages/core/src/oscilloscope-ipc.ts` (new)**
   - Add `getOscilloscopeFrame(): Promise<OscilloscopeFrame | null>`
   - Use `IpcBridge.invoke<GetOscilloscopeFrameResult>(METHOD_GET_OSCILLOSCOPE_FRAME)`.

4. **`ui/packages/core/src/hooks/useOscilloscopeFrame.ts` (new)**
   - Hook for polling-safe retrieval, compatible with RAF-driven consumers.
   - Include connection-aware behavior and cleanup on unmount.

5. **`ui/packages/core/src/index.ts`**
   - Export new types, constant, API helper, and hook.

6. **Tests**
   - `ui/packages/core/src/hooks/useOscilloscopeFrame.test.ts` (new)
   - Update/extend bridge method coverage tests (if in `IpcBridge` tests, add method coverage)

### Testing in this phase

- Unit:
  - type/contract consistency tests (if present in core test suite)
  - hook lifecycle tests (mount, fetch, cleanup)
  - null/error resilience behavior
- Integration:
  - IPC invoke path for `getOscilloscopeFrame` in UI-core test environment

### Verification checkpoint

- Core package exports compile and are consumable by components package.
- Hook behaves under connected/disconnected and null-frame conditions.

### Exit criteria

- `@wavecraft/core` builds (`build:lib`) and tests pass.
- Public exports include oscilloscope APIs with no type regressions.

---

## Phase 5 — UI Component (`@wavecraft/components`) Implementation

**Goal:** Deliver reusable oscilloscope visualization with required UX controls and states.

### File-by-file tasks

1. **`ui/packages/components/src/Oscilloscope.tsx` (new)**
   - Implement reusable component:
     - consumes `getOscilloscopeFrame` or `useOscilloscopeFrame`
     - RAF render loop (`requestAnimationFrame`)
     - channel view selector (`overlay`, `left`, `right`)
     - trigger mode control (v1 single mode visible/defaulted)
     - no-signal UI (flat center line + label)

2. **`ui/packages/components/src/index.ts`**
   - Export `Oscilloscope`.

3. **`ui/packages/components/src/Oscilloscope.test.tsx` (new)**
   - Test:
     - default overlay mode
     - mode switching
     - default trigger mode
     - no-signal rendering
     - render loop cleanup on unmount

4. **`ui/test/mocks/ipc.ts`**
   - Add oscilloscope frame mock helpers (parallel to `setMockMeterFrame`) for component tests.

### Testing in this phase

- Unit:
  - component behavior tests
  - mock IPC integration tests for expected rendering states
- Integration:
  - package-level build/test ensuring `@wavecraft/components` consumes new `@wavecraft/core` exports cleanly

### Verification checkpoint

- Component renders consistently in test environment and uses repo styling patterns.
- RAF loop cleanup confirmed (no timer/animation leaks in tests).

### Exit criteria

- Component exported and test-covered.
- Components package builds and tests pass.

---

## Phase 6 — End-to-End Verification, Performance Guardrails, and CI

**Goal:** Validate behavior across browser-dev and plugin WebView, and enforce safety/performance constraints.

### Cross-layer verification tasks

1. Browser-dev verification:
   - live oscilloscope updates from dev server audio path
   - no-signal state visible when source inactive
2. Plugin WebView verification:
   - live updates via native transport
   - channel selector and trigger mode UI behavior
3. Regression check:
   - meter and parameter interactions unaffected
   - no build/lint/test regressions in workspace

### Realtime safety controls (must pass)

- No allocations/locks/logging/syscalls in audio callback hot paths.
- Fixed-size bounded loops only for capture/trigger scan.
- Single-latest-frame semantics (drop stale frames instead of blocking).
- Confirm oscilloscope code does not alter output sample values.

### Transport performance controls (must pass)

- Keep pull-based frame retrieval lightweight.
- UI uses latest frame, never queues unbounded backlog.
- Detect and avoid over-polling that can flood bridge:
  - RAF render should coalesce to most recent available frame
  - avoid awaiting multiple overlapping in-flight requests

### Exit criteria

- Browser-dev and plugin modes manually validated.
- `cargo xtask ci-check` passes.
- Focused oscilloscope tests pass in all affected layers.

---

## Explicit Validation Commands

Run from repo root unless noted.

```bash
cargo xtask ci-check
```

```bash
cargo xtask ci-check -F
```

Focused Rust tests:

```bash
cargo test -p wavecraft-protocol oscilloscope
cargo test -p wavecraft-bridge oscilloscope
cargo test -p wavecraft-processors oscilloscope
cargo test --manifest-path dev-server/Cargo.toml oscilloscope
```

Focused UI tests:

```bash
npm --prefix ui run test -- packages/core/src/hooks/useOscilloscopeFrame.test.ts
npm --prefix ui run test -- packages/components/src/Oscilloscope.test.tsx
```

Optional targeted package builds:

```bash
npm --prefix ui run build:lib
cargo test -p wavecraft-nih_plug
```

---

## Single-PR Delivery Strategy

The oscilloscope v1 implementation will be delivered in **one end-to-end PR**.

### Scope included in the single PR

- Phase 1: Protocol + Bridge Contract
- Phase 2: Engine Tap + Plugin Host Wiring
- Phase 3: Dev Server Integration
- Phase 4: UI Core API + Hook
- Phase 5: UI Component
- Phase 6: Integration verification and CI hardening

### Reviewability controls inside one PR

To keep one PR manageable and safe:

1. **Structured commit sequence** in dependency order (protocol → engine → dev-server → ui-core → ui-components → integration).
2. **Per-phase verification evidence** included in PR description (tests run and results).
3. **Realtime-safety checklist** explicitly confirmed for audio-thread paths.
4. **Focused reviewer guidance** by file group (engine/protocol/bridge/dev-server/ui).

### Merge gate for single PR

Do not merge until all Definition of Done items are checked and both validation commands pass:

- `cargo xtask ci-check`
- `cargo xtask ci-check -F`

---

## Risks and Mitigations

### Risk: RT regression in audio callback
- **Mitigation:** preallocated buffers only, bounded loops, zero blocking operations
- **Detection:** processor unit tests + code review checklist + focused callback-path scrutiny

### Risk: IPC/transport overhead at RAF cadence
- **Mitigation:** latest-frame semantics, no unbounded queueing, avoid overlapping requests
- **Detection:** manual stress in browser-dev and plugin modes

### Risk: host implementation drift after trait extension
- **Mitigation:** compiler-enforced trait method update across all `ParameterHost` impls
- **Detection:** workspace build + bridge tests

### Risk: browser-dev vs plugin behavior divergence
- **Mitigation:** shared method contract + shared core API; only transport differs
- **Detection:** explicit dual-mode verification in Phase 6

---

## Definition of Done

- [ ] Protocol includes oscilloscope method/constants/types and re-exports
- [ ] Bridge trait/handler/in-memory host support `getOscilloscopeFrame`
- [ ] `OscilloscopeTap` implemented with passthrough invariance and 1024-point frames
- [ ] Plugin host bridge returns oscilloscope frames
- [ ] Dev server host/audio path provides oscilloscope frames in browser mode
- [ ] `@wavecraft/core` exports typed oscilloscope API + hook
- [ ] `@wavecraft/components` exports reusable `Oscilloscope` component
- [ ] Per-phase unit/integration tests added and passing
- [ ] Manual verification completed in browser-dev and plugin WebView
- [ ] Realtime safety checklist satisfied
- [ ] Transport performance controls validated
- [ ] `cargo xtask ci-check` passes
- [ ] `cargo xtask ci-check -F` passes
- [ ] Single end-to-end PR completed and approved without scope creep
