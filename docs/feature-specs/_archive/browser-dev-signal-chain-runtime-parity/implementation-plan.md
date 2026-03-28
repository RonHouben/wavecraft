# Implementation Plan: Browser Dev Signal-Chain Runtime Parity

## Overview

This change fixes the browser dev-mode reorder bug by making browser audio execution use the same slot-aware runtime semantics as native/plugin mode. Today, `setSignalChainOrder` updates control-plane state and UI notifications, but the audible/runtime topology in browser dev mode remains fixed because the dev FFI is processor-only and the dev server owns a fixed post-process `OscilloscopeTap`.

The approved direction is to converge browser dev mode onto a single slot-aware runtime model, reuse native/plugin slot-aware semantics, move reorderable tap ownership into the runtime, and keep the dev server as shell/transport rather than topology owner. The implementation below assumes a dev FFI contract update is acceptable and treats fail-fast versioning as the default path.

## Related Documents

- [Tap Processor System — Low-Level Design](../tap-processor-system/low-level-design-tap-processor-system.md) — canonical slot/tap semantics
- [Tap Processor System — Implementation Plan](../tap-processor-system/implementation-plan.md) — prior slot-aware runtime/codegen work
- [High-Level Design](../../architecture/high-level-design.md) — IPC, runtime ownership, and browser dev-mode architecture
- [Development Workflows](../../architecture/development-workflows.md) — browser dev-mode and FFI workflow context
- [Coding Standards](../../architecture/coding-standards.md) — overall repository conventions
- [Rust Coding Standards](../../architecture/coding-standards-rust.md) — Rust/FFI/real-time safety guidance
- [Testing & Quality Standards](../../architecture/coding-standards-testing.md) — validation expectations
- [Agent Development Flow](../../architecture/agent-development-flow.md) — feature-spec workflow and handoffs

## Problem Statement and Approved Decisions

### Problem statement

Browser dev mode currently exposes a slot-aware control plane but not a slot-aware audio runtime:

- `engine/crates/wavecraft-bridge/src/handler.rs`
  - `handle_set_signal_chain_order()`
  - `handle_json_multi()`
  - correctly accept `setSignalChainOrder` and emit `signalChainOrderChanged`
- `dev-server/src/host.rs`
  - `DevServerHost::set_signal_chain_order()`
  - delegates to `InMemoryParameterHost` and updates stored order state
- but `dev-server/src/audio/server/startup_wiring.rs`
  - creates a fixed `OscilloscopeTap::with_output(...)`
- and `dev-server/src/audio/server/input_pipeline.rs`
  - owns `oscilloscope_tap: OscilloscopeTap`
  - always calls `self.oscilloscope_tap.capture_stereo(left, right)` after the full DSP pass

So browser dev mode can report reordered slots while still processing/capturing audio with a fixed post-process oscilloscope position. That diverges from native/plugin semantics where taps belong to the runtime topology and move with slot order.

### Approved decisions

- Browser dev mode **must match native reorder semantics**
- A **dev FFI version update is approved if needed**
- The fix direction is:
  - converge browser dev mode onto a **single slot-aware runtime model**
  - **reuse native/plugin slot-aware semantics**
  - move reorderable tap ownership into the **runtime**
  - treat `dev-server` as **shell/transport**, not topology owner

## Goals and Non-Goals

### Goals

- Make browser dev audio execution honor the same signal-chain slot order as native/plugin mode
- Remove server-owned fixed oscilloscope topology from the audio callback path
- Reuse generated/native slot validation and ordering logic rather than reimplementing topology rules in `dev-server`
- Keep processor discovery (`wavecraft_get_processors_json`) processor-only
- Preserve current UI contract:
  - `getSignalChainOrder`
  - `setSignalChainOrder`
  - `signalChainOrderChanged`
- Keep FFI/version mismatch behavior fail-fast with actionable restart/upgrade guidance
- Preserve real-time safety: no locks, allocations, or syscalls on the audio callback

### Non-goals

- No backward-compatibility shim for old dev-runtime semantics beyond version-mismatch diagnostics
- No redesign of the UI signal-chain API or drag/drop UX
- No change to native/plugin runtime semantics beyond reuse/extraction needed for parity
- No support for hot-reloading a changed slot catalog in-place unless it falls out naturally and safely; restart guidance is acceptable
- No change to processor catalog semantics (taps remain excluded from generated processor registry)

## Architecture Strategy

### Target model

Browser dev mode should run a **slot-aware dev runtime** loaded via FFI. That runtime owns:

- processor execution
- tap placement
- signal-chain order application
- oscilloscope capture production

The dev server should own:

- transport/WebSocket shell
- device I/O
- parameter bridge plumbing
- cached host state and IPC exposure
- runtime lifecycle and restart decisions

### Practical implementation approach

The most practical path is to evolve the current dev FFI from “processor-only” into “dev runtime” semantics while keeping the loader flow simple:

1. **Extend the dev FFI contract** in `engine/crates/wavecraft-protocol/src/dev_audio_ffi.rs`
   - bump `DEV_PROCESSOR_VTABLE_VERSION` from `2` to `3`
   - add control-thread methods needed for browser parity
2. **Expose declared slot metadata from FFI**
   - add a dedicated export for the runtime’s declared/default slot order
   - keep `wavecraft_get_processors_json()` unchanged and processor-only
3. **Add runtime-side order application**
   - allow `dev-server` to push validated `SignalChainSlot[]` into the FFI runtime
4. **Add runtime-side oscilloscope frame retrieval**
   - remove server-owned `OscilloscopeTap`
   - poll the runtime for the latest oscilloscope frame on the non-RT side
5. **Make startup/reload seed host state from FFI-owned slot metadata**
   - host becomes a cache/projection of the runtime contract, not the topology source of truth

### Preferred ownership boundaries after the change

- **Source of truth for declared slots:** generated runtime / FFI
- **Source of truth for current active order:** generated runtime, mirrored in host for IPC reads/notifications
- **Source of truth for oscilloscope capture position:** generated runtime
- **Source of truth for transport notifications:** bridge/dev-server after successful runtime mutation

## Phased Implementation Plan

### Phase 1 — Lock the dev-runtime contract and metadata boundary

#### 1.1 Extend the dev FFI contract to support runtime parity

**Files**

- `engine/crates/wavecraft-protocol/src/dev_audio_ffi.rs`
- `engine/crates/wavecraft-protocol/src/lib.rs`
- `engine/crates/wavecraft-nih_plug/src/lib.rs`

**Symbols to touch**

- `DevProcessorVTable`
- `DEV_PROCESSOR_VTABLE_VERSION`

**Changes**

- Bump `DEV_PROCESSOR_VTABLE_VERSION` from `2` to `3`
- Extend `DevProcessorVTable` with control-thread methods required for slot-aware browser parity, expected to include:
  - `set_signal_chain_order(...)` or `set_signal_chain_order_json(...)`
  - `take_latest_oscilloscope_frame(...)` or `take_latest_oscilloscope_frame_json(...)`
- Keep existing methods:
  - `create`
  - `process`
  - `apply_plain_values`
  - `set_sample_rate`
  - `reset`
  - `drop`

**Why first**

- This is the hard boundary between generated runtime and `dev-server`; everything else depends on it.

**Complexity**

- Medium

**Risk**

- Medium: ABI change, but explicitly approved.

#### 1.2 Add a dedicated FFI export for declared slot metadata

**Files**

- `engine/crates/wavecraft-macros/src/plugin/codegen/ffi.rs`
- `engine/crates/wavecraft-bridge/src/plugin_loader.rs`

**Symbols to add/update**

- generated export such as `wavecraft_get_signal_chain_slots_json`
- `PluginLoader` loading path alongside existing params/processors metadata loaders

**Changes**

- Export declared/default slot order from generated code as JSON using existing `SignalChainSlot` protocol shape
- Load that slot list in `PluginLoader` during startup/reload
- Keep `wavecraft_get_processors_json()` processor-only

**Why**

- `dev-server` must stop inferring topology from processor-only metadata or hard-coded taps.

**Complexity**

- Medium

**Risk**

- Low

#### 1.3 Keep naming pragmatically stable, but rename internal runtime types

**Files**

- `engine/crates/wavecraft-macros/src/plugin/codegen/ffi.rs`
- `dev-server/src/audio/ffi_processor.rs`

**Symbols to rename internally**

- `__DevProcessorInstance` → `__DevRuntimeInstance`
- `FfiProcessor` may remain public for now, but internal comments/docs should describe it as runtime-backed

**Changes**

- Keep external loader continuity where practical
- Rename internal symbols/comments so the code reflects actual ownership

**Why**

- Reduces future confusion; today “processor” is underselling what browser parity requires.

**Complexity**

- Low

**Risk**

- Low

### Phase 2 — Generate runtime-owned slot/order behavior in the FFI path

#### 2.1 Reuse generated slot validation instead of duplicating it in `dev-server`

**Files**

- `engine/crates/wavecraft-macros/src/plugin/codegen/params.rs`
- `engine/crates/wavecraft-macros/src/plugin/codegen/ffi.rs`

**Symbols to touch**

- `__WavecraftParams`
- `SignalChainOrderAccess::set_order`
- generated FFI order setter

**Changes**

- Route incoming dev-runtime reorder requests through generated `SignalChainOrderAccess::set_order`
- Do not add a separate validation implementation in `dev-server`
- Ensure the same duplicate/omission/type validation used by native/plugin mode applies in browser dev mode

**Why**

- Prevents semantic drift. One validation path, one set of rules.

**Complexity**

- Medium

**Risk**

- Low

#### 2.2 Expose runtime-owned oscilloscope retrieval

**Files**

- `engine/crates/wavecraft-macros/src/plugin/codegen/ffi.rs`
- possibly `engine/crates/wavecraft-processors/src/oscilloscope.rs` if a poll/drain hook is needed

**Symbols to add/update**

- generated vtable implementation for latest oscilloscope frame retrieval
- any runtime-side storage needed to move frame data from RT-safe tap capture to non-RT polling

**Changes**

- The generated runtime must surface the latest oscilloscope frame without relying on a server-owned tap
- Retrieval should be non-RT and polling-friendly, matching current UI IPC cadence
- If frame serialization is used across FFI, keep it isolated to non-audio code paths

**Why**

- This is the key piece that lets the runtime own tap placement while the dev server remains a shell.

**Complexity**

- High

**Risk**

- High: must preserve RT safety and avoid churn in frame semantics.

#### 2.3 Ensure the generated runtime’s default slot order is the exported browser default

**Files**

- `engine/crates/wavecraft-macros/src/plugin/codegen/params.rs`
- `engine/crates/wavecraft-macros/src/plugin/codegen/ffi.rs`

**Symbols to touch**

- `__initial_order`
- default slot generation helpers

**Changes**

- Reuse the generated declaration-order slot list as the exported browser slot order
- Avoid duplicating slot list assembly logic

**Why**

- Prevents startup drift between native and browser default order.

**Complexity**

- Low

**Risk**

- Low

### Phase 3 — Refactor the dev-server audio path to stop owning topology

#### 3.1 Update the FFI wrapper to drive a slot-aware runtime

**Files**

- `dev-server/src/audio/ffi_processor.rs`

**Symbols to touch**

- `DevAudioProcessor`
- `FfiProcessor::new()`
- helper methods around supported features/version gating

**Changes**

- Extend the trait wrapper with control-thread methods corresponding to new vtable capabilities
- Expected additions:
  - `set_signal_chain_order(...)`
  - `take_latest_oscilloscope_frame(...)`
- Gate behavior on vtable version `== 3`; do not attempt partial fallback for reorder parity
- Add tests for:
  - order setter dispatch
  - oscilloscope retrieval dispatch
  - version mismatch behavior

**Why**

- This is the `dev-server` runtime adapter.

**Complexity**

- Medium

**Risk**

- Medium

#### 3.2 Remove server-owned fixed `OscilloscopeTap` from startup wiring

**Files**

- `dev-server/src/audio/server/startup_wiring.rs`
- `dev-server/src/audio/server/device_setup.rs`

**Symbols to remove/update**

- `create_oscilloscope_channel(...)`
- `OscilloscopeTap::with_output(...)`
- `InputStreamBuildContext::oscilloscope_tap`

**Changes**

- Stop creating the oscilloscope tap in `start_audio_io()`
- Stop threading an `OscilloscopeTap` through device setup and input callback construction
- If an oscilloscope consumer thread remains necessary, it should consume runtime-polled frames, not a server-owned tap

**Why**

- This is the direct fix for split ownership.

**Complexity**

- Medium

**Risk**

- Medium

#### 3.3 Remove fixed post-process oscilloscope capture from the input callback

**Files**

- `dev-server/src/audio/server/input_pipeline.rs`

**Symbols to remove/update**

- `InputCallbackPipeline::oscilloscope_tap`
- `self.oscilloscope_tap.capture_stereo(left, right)`

**Changes**

- Delete server-owned oscilloscope capture from `process_callback()`
- Keep:
  - input routing
  - plain-value injection
  - runtime `process()`
  - output modifiers
  - meter update generation
  - ring-buffer output routing
- Do not add new topology logic here

**Why**

- The callback path must become topology-agnostic.

**Complexity**

- Low

**Risk**

- Low

### Phase 4 — Synchronize host state with runtime state and protect reload behavior

#### 4.1 Seed `DevServerHost` signal-chain order from loaded slot metadata

**Files**

- `dev-server/src/host.rs`
- `engine/crates/wavecraft-bridge/src/plugin_loader.rs`
- `dev-server/src/reload/rebuild.rs`

**Symbols to add/update**

- startup construction path for `DevServerHost`
- slot metadata load result handling
- host initialization/replacement flow

**Changes**

- Initialize the in-memory host with the declared slot order loaded from FFI
- On startup, browser `getSignalChainOrder` should reflect the runtime’s default order without requiring a UI write first
- On rebuild, re-load declared slots alongside parameters/processors

**Why**

- The host should reflect runtime contract, not invent it.

**Complexity**

- Medium

**Risk**

- Medium

#### 4.2 Push reorder mutations into the runtime before broadcasting success

**Files**

- `dev-server/src/host.rs`
- a small coordination layer near dev-server startup/runtime ownership if direct host access is awkward
- `engine/crates/wavecraft-bridge/src/handler.rs` only if sequencing needs adjustment

**Symbols to update**

- `DevServerHost::set_signal_chain_order()`

**Changes**

- On `set_signal_chain_order(order)`:
  1. validate via existing host/generated rules
  2. apply the order to the loaded runtime
  3. only then persist/mirror the order in host state
  4. allow `IpcHandler::handle_json_multi()` to emit `signalChainOrderChanged`
- If runtime application fails, return an error and do not broadcast

**Why**

- Browser notifications must describe actual runtime state.

**Complexity**

- High

**Risk**

- High: requires clean coordination between host and live runtime handle.

#### 4.3 Mirror runtime oscilloscope frames into host cache for existing IPC consumers

**Files**

- `dev-server/src/host.rs`
- runtime polling task location in `dev-server` startup/audio orchestration

**Symbols to reuse**

- `DevServerHost::set_latest_oscilloscope_frame()`
- `ParameterHost::get_oscilloscope_frame()`

**Changes**

- Reuse existing host frame cache and `get_oscilloscope_frame()` IPC surface
- Replace the frame producer with a non-RT polling path from the FFI runtime
- Keep UI/public protocol unchanged

**Why**

- This minimizes UI churn while changing ownership underneath.

**Complexity**

- Medium

**Risk**

- Medium

#### 4.4 Fail fast on slot-catalog hot-reload drift

**Files**

- `dev-server/src/reload/rebuild.rs`
- `dev-server/src/host.rs`
- `engine/crates/wavecraft-bridge/src/plugin_loader.rs`

**Changes**

- Detect changes to declared slot IDs/types on rebuild
- If the slot catalog changed while a runtime is already active, return a restart-required error similar in spirit to existing parameter-schema drift handling
- Preserve current order only when the slot catalog is unchanged

**Why**

- Topology hot-reload is a separate feature; this bug fix should not accidentally promise it.

**Complexity**

- Medium

**Risk**

- Low

### Phase 5 — Keep bridge/UI surfaces stable, expand tests to cover real parity

#### 5.1 Preserve existing bridge and UI ordering API

**Files expected to remain mostly unchanged**

- `engine/crates/wavecraft-bridge/src/handler.rs`
- `ui/packages/core/src/ipc/SignalChainOrderClient.ts`
- `ui/packages/core/src/hooks/useSignalChainOrder.ts`

**Changes**

- Prefer no production API changes here
- Update tests or comments only if necessary

**Why**

- The bug is below the control plane; the slot-aware UI API is already the correct shape.

**Complexity**

- Low

**Risk**

- Low

#### 5.2 Strengthen backend/browser tests to assert runtime semantics, not just notification semantics

**Files**

- `ui/tests/visual/signalChain.backend.spec.ts`
- optionally a new browser-dev parity test such as `ui/tests/visual/signalChain.runtime-parity.spec.ts`
- `dev-server/tests/` integration coverage as needed

**Test additions**

- Existing test already proves:
  - backend order updates
  - `signalChainOrderChanged` notification
  - rendered slot order
- Add parity coverage that proves:
  - moving `oscilloscope_tap` changes observed frame semantics in browser dev mode
  - the frame reflects boundary-relative placement, not fixed post-process capture
- Good deterministic scenario:
  - feed a test tone
  - move `oscilloscope_tap` before vs after a clearly visible stage such as `soft_clip` or `tone_filter`
  - assert frame characteristics differ in the expected direction

**Why**

- The current test catches control-plane correctness, not the actual bug.

**Complexity**

- Medium

**Risk**

- Medium

#### 5.3 Add unit/integration tests around FFI versioning and runtime application

**Files**

- `engine/crates/wavecraft-bridge/src/plugin_loader.rs`
- `dev-server/src/audio/ffi_processor.rs`
- `dev-server/src/host.rs`

**Coverage**

- vtable v3 accepted, older versions rejected with actionable error
- runtime order setter invoked on reorder
- host order not updated/broadcast on runtime apply failure
- oscilloscope frame polling path works without server-owned tap

**Complexity**

- Medium

**Risk**

- Low

## Testing and Validation Strategy

### Rust unit and integration testing

#### FFI / protocol / loader

- `engine/crates/wavecraft-protocol/src/dev_audio_ffi.rs`
  - validate v3 contract shape
- `engine/crates/wavecraft-bridge/src/plugin_loader.rs`
  - add/update tests for vtable version mismatch
  - add slot-metadata export loading tests

#### Dev runtime adapter

- `dev-server/src/audio/ffi_processor.rs`
  - tests for new control-thread methods
  - tests that unsupported older vtables fail fast

#### Host/runtime coordination

- `dev-server/src/host.rs`
  - reorder success updates mirrored state
  - reorder failure does not mutate mirrored state
  - existing oscilloscope getter continues to return cached runtime-driven frame
  - slot-catalog drift returns restart guidance

#### Dev audio server

- `dev-server/src/audio/server/input_pipeline.rs`
  - no direct oscilloscope capture remains
- `dev-server/src/audio/server/startup_wiring.rs`
  - no server-owned `OscilloscopeTap` construction remains

### Browser/UI validation

#### Automated

- Extend `ui/tests/visual/signalChain.backend.spec.ts`
- Add a runtime-parity visual/integration spec if one test becomes too overloaded

#### Manual

Run browser dev mode and verify:

1. `getSignalChainOrder` returns declared default order on first load
2. dragging slots still updates UI immediately
3. `signalChainOrderChanged` only reflects successful runtime changes
4. moving `oscilloscope_tap` changes observed waveform semantics relative to processor boundaries
5. native/plugin mode and browser dev mode now agree on reorder behavior for the same slot sequence

### Full validation pass

- `cargo xtask ci-check`
- browser-dev visual verification in VS Code Simple Browser per project testing workflow
- targeted regression check on hot reload:
  - unchanged slot catalog: continue working
  - changed slot catalog: clear restart-required guidance

## Risks, Rollout Notes, and Approval Checkpoints

### Key risks

- **FFI scope creep**
  - The contract can balloon if too much state is pushed over ABI.
  - Mitigation: keep new control-thread methods narrowly focused on order application and oscilloscope polling.

- **RT-safety regression**
  - Moving oscilloscope ownership into runtime could accidentally introduce non-RT work on the callback.
  - Mitigation: all serialization, parsing, and polling must remain off the audio thread.

- **State sync bugs**
  - Host state and runtime state could diverge if update sequencing is wrong.
  - Mitigation: runtime apply must happen before mirrored host mutation and notification broadcast.

- **Hot-reload ambiguity**
  - Slot schema changes during rebuild can produce half-valid UI/runtime state.
  - Mitigation: explicitly reject slot-catalog drift unless restart logic is intentionally implemented.

### Rollout notes

- This is a **clean-break dev-runtime contract change**
- Expected developer-visible outcome:
  - older generated plugins fail with a clear vtable version mismatch
  - restarting `wavecraft start` after rebuilding with the updated SDK resolves the mismatch
- No user-facing UI API migration should be required

### Approval checkpoints

#### Checkpoint A — Contract review before coding Phase 2

Approve:

- exact v3 FFI additions
- JSON vs structured payload choice for slot/order/frame transport
- whether external symbol names stay stable or are renamed

#### Checkpoint B — Runtime parity review after Phase 4

Approve once:

- server-owned `OscilloscopeTap` is gone from the callback path
- reorder writes are applied to the runtime before notification
- browser dev mode behavior matches native semantics in a targeted parity scenario

#### Checkpoint C — Pre-merge verification

Approve once:

- automated tests pass
- manual browser-dev verification is recorded
- version-mismatch/restart guidance is clear and actionable

## Exit Criteria

- [ ] Browser dev mode no longer owns a fixed post-process oscilloscope tap in `dev-server`
- [ ] The dev FFI contract is updated and version-gated for slot-aware runtime parity
- [ ] Declared/default signal-chain slots are loaded from FFI, not inferred from server-owned topology
- [ ] `setSignalChainOrder` applies to the live browser dev runtime before host/UI notification
- [ ] `getOscilloscopeFrame` in browser dev mode reflects runtime-owned tap placement
- [ ] Browser dev reorder semantics match native/plugin semantics for the same slot order
- [ ] Slot-catalog hot-reload drift fails fast with restart guidance rather than silently diverging
- [ ] Existing UI slot-order APIs remain intact
- [ ] `cargo xtask ci-check` passes
- [ ] Manual browser-dev parity verification is completed and recorded

## Implementation Sequencing Summary

Recommended coding order:

1. FFI contract + loader metadata
2. generated runtime FFI hooks
3. `dev-server` runtime adapter
4. remove server-owned oscilloscope topology
5. host/runtime synchronization and error handling
6. parity-focused tests and manual validation
