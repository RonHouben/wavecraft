# User Stories: Audio Pipeline Fixes & Mocking Cleanup

## Overview

The dev-mode audio pipeline (`wavecraft start`) has two critical gaps that prevent plugin developers from testing real audio behavior during development. Additionally, unused synthetic metering infrastructure adds unnecessary complexity. This milestone fixes the audio pipeline and cleans up the mocking infrastructure, ensuring beta testers get a working development experience.

## Version

**Target Version:** `0.10.0` (minor — significant functionality change, new audio pipeline behavior)

**Rationale:** This is a minor bump because it fundamentally changes how the dev server processes audio — from metering-only to full audio I/O with parameter sync. While no public API is broken, the behavioral change is significant enough to warrant a minor version. This also prepares the version lineage for v1.0.0-beta.

---

## User Story 1: Audio Output in Dev Mode

**As a** plugin developer using `wavecraft start`
**I want** to hear my processed audio output through my speakers/headphones
**So that** I can verify my DSP code works correctly during browser-based development

### Context

Currently, `AudioServer` only opens an input stream (`build_input_stream()`) — it reads microphone data and computes meters, but never calls `processor.process()` to apply the user's DSP. There is no output stream. A developer building a gain plugin would see meters move but hear nothing.

### Acceptance Criteria

- [ ] `wavecraft start` opens both an input and output audio stream via cpal
- [ ] Audio flows: OS input → user's `Processor::process()` → OS output
- [ ] Meters are computed from the *processed* output (not the raw input)
- [ ] When no FFI vtable is found (metering-only fallback), behavior is unchanged
- [ ] Audio latency is acceptable for monitoring (< 20ms round-trip)
- [ ] No audio glitches or dropouts under normal operation
- [ ] Real-time safety maintained — no allocations or locks on the audio thread

### Notes

- The `AudioServer` in `wavecraft-dev-server` currently only uses `build_input_stream()`
- Need to add `build_output_stream()` or use a full-duplex stream
- cpal supports both input-only and output-only streams; for effects plugins, we need both
- For generators/oscillators (future), need output-only mode with empty input buffers
- This story focuses on the effects path (input → process → output)

---

## User Story 2: Parameter Changes Reach DSP in Dev Mode

**As a** plugin developer adjusting parameters in the browser UI
**I want** my parameter changes to affect the audio processing in real-time
**So that** I can hear the effect of my DSP parameters during development

### Context

Two separate issues exist:

1. **Dev server path:** When running `wavecraft start`, WebSocket `setParameter` messages update the `InMemoryParameterHost` state, but the `AudioServer`'s processing callback doesn't read these values. The `FfiProcessor` processes audio with whatever defaults were set at creation.

2. **Plugin macro path (documented limitation):** The `wavecraft_plugin!` macro generates code where `Processor::process()` always receives default parameter values. This is a known limitation documented in the high-level design and is **out of scope** for this milestone.

### Acceptance Criteria

- [ ] WebSocket `setParameter` updates are received by the audio processing callback
- [ ] Parameter values are passed to `FfiProcessor::process()` (or equivalent) on the audio thread
- [ ] Communication between WebSocket thread and audio thread is lock-free (no mutexes)
- [ ] Parameter updates are applied at block boundaries (not sample-accurate — acceptable for dev mode)
- [ ] Moving a gain slider in the browser UI audibly changes the output level
- [ ] Multiple parameters can be updated simultaneously without data races

### Notes

- Need a lock-free mechanism to pass parameter state from WS thread → audio thread
- Options: `Arc<AtomicF32>` per parameter, or a lock-free parameter snapshot struct
- The FFI vtable's `process()` function doesn't currently accept parameters — may need vtable extension or a separate parameter-passing mechanism
- The existing `ParameterHost` trait has `set_parameter()` — need to bridge this to the audio callback
- Sample-accurate automation is not required for dev mode; block-level updates are sufficient

---

## User Story 3: Remove Synthetic Meter Generator

**As a** SDK maintainer
**I want** the synthetic meter generator and related mocking infrastructure removed
**So that** the codebase is leaner and there's no confusion about what's real vs. fake

### Context

The `MeterGenerator` in `wavecraft-metering/src/dev.rs` generates fake animated meter data for UI development. Now that we have real FFI audio processing with actual meter computation, this synthetic infrastructure is unused and adds maintenance burden. The `InMemoryDevHost` in the CLI's `dev_server/host.rs` uses this generator as a fallback.

### Acceptance Criteria

- [ ] `wavecraft-metering/src/dev.rs` is removed (the `MeterGenerator` struct)
- [ ] `dev` module is removed from `wavecraft-metering/src/lib.rs` exports
- [ ] CLI `dev_server/host.rs` `InMemoryDevHost` is updated to not use synthetic meters
- [ ] If FFI vtable is not available, meters show zeros (silent) rather than fake animation
- [ ] All tests still pass after removal
- [ ] No broken imports or unused dependencies remain

### Notes

- The `MockTransport` in `@wavecraft/core` tests is **not** in scope — that's a legitimate test double for unit testing `IpcBridge`
- The `test/mocks/ipc.ts` module is **not** in scope — that's test infrastructure
- Only remove *production* mocking that simulates audio signals to the UI
- The fallback behavior when no FFI vtable exists should show "no audio" status, not fake meters

---

## User Story 4: UI Parameter Load Retry on Connection

**As a** plugin developer opening the browser UI
**I want** parameters to load correctly even if the WebSocket isn't connected yet
**So that** I don't see an empty UI and have to manually refresh

### Context

`useAllParameters()` fires a single `getAllParameters` request on mount. If the WebSocket hasn't connected yet, the request silently fails, and parameters never load. The user must refresh the page to retry.

### Acceptance Criteria

- [ ] `useAllParameters()` retries the parameter fetch when WebSocket connection is established
- [ ] Parameters appear within 1 second of connection becoming available
- [ ] No duplicate requests are fired when already connected
- [ ] Works correctly when the connection drops and reconnects (parameters re-fetched)
- [ ] No polling or busy-waiting — event-driven retry based on connection state

### Notes

- The `useConnectionStatus()` hook already exposes connection state
- Options: have `useAllParameters` depend on connection status, or have the WebSocket transport emit a "connected" event that triggers re-fetch
- The reconnection flow (exponential backoff) already exists in `WebSocketTransport`
- This should be a small change — likely adding a dependency on `useConnectionStatus` in the `useAllParameters` hook

---

## Priority & Sequencing

| Story | Priority | Effort | Dependencies |
|-------|----------|--------|-------------|
| US-1: Audio output | Critical | Medium | None |
| US-2: Params reach DSP | Critical | Medium | US-1 (audio must flow first) |
| US-3: Remove mocking | Low | Small | US-1 (real audio replaces fake) |
| US-4: UI param retry | Minor | Small | None (can be done in parallel) |

**Recommended implementation order:** US-1 → US-2 → US-3, with US-4 in parallel at any point.

---

## Out of Scope

- **Plugin macro parameter sync** — The `wavecraft_plugin!` macro limitation where `Processor::process()` receives default values is a documented known limitation. Fixing it requires significant proc-macro work (~1500 LOC) and is targeted for a future release (0.11.0 or 1.0.0).
- **Generator/oscillator mode** — Output-only audio stream for synth plugins. The current design focuses on effects (input → process → output). Generator support can be added later.
- **Sample-accurate automation** — Dev mode uses block-level parameter updates, not per-sample automation. Full sample-accurate automation is a post-V1 concern.
- **WASM audio input** — Browser-based audio processing via WebAssembly. This remains in the backlog as a separate feature.
