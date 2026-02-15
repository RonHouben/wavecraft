# Implementation Plan: browser-dev-audio-start-silence

## Overview

Fix browser dev mode startup so audio initialization is attempted deterministically even when parameter metadata comes from cache, and expose a first-class audio runtime status contract to the UI. Align metering behavior so both polling (`getMeterFrame`) and push (`meterUpdate`) are coherent, while enforcing strict current-SDK contracts. Add structured diagnostics to eliminate “silent success” ambiguity and fail fast with explicit errors when required contracts are missing.

## Requirements

1. Deterministic audio startup independent of parameter cache hit path.
2. Explicit audio runtime status contract from server to UI.
3. Metering contract alignment for both polling and push.
4. Actionable structured diagnostics for startup/device/permission/runtime failures.
5. Strict contract enforcement for current SDK clients/plugins; missing required contracts must fail startup with clear diagnostics.

---

## File-level change map

### CLI startup orchestration

- `cli/src/commands/start.rs`
  - Decouple parameter metadata loading from runtime loader acquisition.
  - Ensure audio start attempt occurs on cache-hit path.
  - Add explicit status transitions (`Initializing` → `Running*`/`Degraded`/`Failed`).
  - Update meter forwarding loop to keep host snapshot + broadcast existing push updates.

### Dev server audio status model & state ownership

- `dev-server/src/audio/status.rs` (new)
  - Add `AudioRuntimePhase`, `AudioDiagnosticCode`, `AudioRuntimeStatus`.
- `dev-server/src/session.rs`
  - Add shared status ownership (`Arc<RwLock<AudioRuntimeStatus>>` equivalent).
- `dev-server/src/ws/mod.rs`
  - Add broadcast support for `audioStatusChanged` transitions.
- `dev-server/src/host.rs`
  - Implement `get_audio_status()`.
  - Store latest meter snapshot and return it from `get_meter_frame()`.

### Protocol + bridge contract evolution

- `engine/crates/wavecraft-protocol/src/ipc.rs`
  - Add `getAudioStatus` method and `audioStatusChanged` notification constants.
  - Add status payload/result types.
- `engine/crates/wavecraft-protocol/src/lib.rs`
  - Re-export status types/constants.
- `engine/crates/wavecraft-bridge/src/host.rs`
  - Require host trait implementation of `get_audio_status() -> Option<_>`.
- `engine/crates/wavecraft-bridge/src/handler.rs`
  - Route `getAudioStatus` request and return status payload.

### UI core API/hook additions

- `ui/packages/core/src/types/ipc.ts`
  - Add TS status/diagnostic types + method/notification names.
- `ui/packages/core/src/ipc/IpcBridge.ts`
  - Add typed `getAudioStatus` call.
  - Handle `audioStatusChanged` notifications.
- `ui/packages/core/src/hooks/useAudioStatus.ts` (new)
  - Initial fetch on connect + subscription handling.
- `ui/packages/core/src/index.ts`
  - Export new hook/types.
- (Docs/comments) `useConnectionStatus` usage points
  - Clarify transport connectivity is not audio readiness.

---

## Implementation phases, tasks, and dependencies

### Phase 1 — Startup decoupling (highest-impact root-cause fix)

1. Refactor metadata loader:
   - Create metadata-only function returning params + source info.
2. Introduce runtime loader acquisition independent of metadata source.
3. Update startup flow to always attempt runtime loader and audio init in dev-audio mode.
4. Emit initial `Initializing` status before runtime attempt.
5. Remove the browser-dev audio-start-silence feature flag and any conditional feature-flag code path; make this behavior unconditionally enabled by default.

**Dependencies:** none (foundation)  
**Risk:** medium (central startup flow)  
**Exit criteria:** cache-hit path still attempts runtime loader/audio startup.

---

### Phase 2 — Audio runtime status model and propagation

1. Add status model types and diagnostics in `dev-server/src/audio/status.rs`.
2. Add shared status state in session lifecycle.
3. Add WS broadcast helper + transition publishing.
4. Thread status updates through startup success/failure branches.

**Dependencies:** Phase 1  
**Risk:** medium (state consistency and transition correctness)  
**Exit criteria:** transitions visible server-side and broadcast to connected clients.

---

### Phase 3 — Protocol + bridge implementation

1. Add JSON-RPC method/notification constants + payload structs in protocol crate.
2. Re-export status types/constants in protocol lib.
3. Extend bridge host trait with required `get_audio_status()` getter (no default compatibility fallback).
4. Add `getAudioStatus` dispatch handler.

**Dependencies:** Phase 2 for concrete status shape  
**Risk:** low-medium (contract serialization and handler wiring)  
**Exit criteria:** client can request status via `getAudioStatus`; missing host contract implementations fail explicitly.

---

### Phase 4 — Metering contract alignment (poll + push)

1. Add latest meter snapshot storage in dev-server host.
2. Update meter forwarding flow in CLI to:
   - update snapshot
   - continue sending `meterUpdate` push events
3. Change `get_meter_frame()` implementation to return snapshot (not permanent `None`).

**Dependencies:** Phase 1 (startup path), can overlap with Phase 3  
**Risk:** medium (synchronization correctness, stale snapshot edge cases)  
**Exit criteria:** polling path returns meaningful frame once data exists; push path unchanged.

---

### Phase 5 — UI status consumption (opt-in)

1. Add TS status/diagnostic types and contract constants.
2. Add `IpcBridge` typed status request + notification handling.
3. Implement `useAudioStatus` hook:
   - fetch on connect
   - subscribe to status change notifications
   - expose readiness/degraded convenience booleans
4. Export hook/types and document semantics relative to transport connection.

**Dependencies:** Phase 3  
**Risk:** low (additive API surface)  
**Exit criteria:** UI can distinguish transport connected vs audio ready/degraded/failed.

---

### Phase 6 — Verification and hardening

1. Unit tests for startup decoupling and transition emission.
2. Protocol serialization/deserialization tests for new status types.
3. Bridge handler tests for method dispatch and explicit contract-error behavior.
4. Dev-server tests for meter snapshot behavior and status getter.
5. Integration tests for:
   - cache-hit + runtime success
   - forced runtime failure with diagnostic
   - status broadcast delivery
6. Manual macOS dev-mode verification for device/permission scenarios.

**Dependencies:** Phases 1–5  
**Risk:** low-medium (test harness coverage depth)  
**Exit criteria:** all planned scenarios covered; no silent-success state reproducible.

---

## Rollout order and fail-fast policy

### Rollout order (recommended)

1. **Server internals first:** Phase 1 + 2 (fix root cause and status source of truth)
2. **Protocol/bridge next:** Phase 3 (enable status retrieval/notification contract)
3. **Metering alignment:** Phase 4 (restore poll+push coherence)
4. **UI opt-in:** Phase 5 (consume status while preserving strict contract semantics)
5. **Validation pass:** Phase 6 (tests + manual scenarios)

### Fail-fast policy (with explicit opt-in escape hatch only)

- `getAudioStatus` is a required contract in current SDK mode; unsupported implementations are treated as explicit errors.
- Runtime loader/vtable acquisition failures are startup-fatal and must surface clear diagnostics.
- Output-device constraints may still yield `RunningInputOnly`/`Degraded` when runtime initialization succeeds and the condition is recoverable.
- Existing `meterUpdate` push behavior remains, but absence of required startup/runtime contracts is never silently tolerated.
- Any temporary compatibility behavior is allowed only behind an explicit, documented opt-in escape hatch.

---

## Risk areas and mitigations

1. **Status drift from real runtime**
   - Mitigation: single status owner + explicit transition points only.
2. **Concurrency overhead or RT-safety regression**
   - Mitigation: keep status/snapshot updates outside audio callback hot path.
3. **Protocol mixed-version behavior**

- Mitigation: treat mixed/unsupported versions as explicit contract errors with clear upgrade diagnostics.

4. **Diagnostic misclassification across platforms**
   - Mitigation: conservative mapping + include raw underlying error context.
5. **Regression in startup orchestration**
   - Mitigation: dedicated cache-hit startup tests and branch-level logging assertions.

---

## Verification strategy

### Unit tests

- `cli/src/commands/start.rs`
  - cache-hit path still performs runtime loader/audio init attempt
  - status transitions emitted for success/failure
- `engine/crates/wavecraft-protocol/src/ipc.rs`
  - status type serde round-trip and constants sanity
- `engine/crates/wavecraft-bridge/src/handler.rs`
  - `getAudioStatus` dispatch and explicit error behavior when contract requirements are not met
- `dev-server/src/host.rs`
  - meter snapshot set/get
  - `get_audio_status` getter behavior

### Integration tests

- `dev-server/tests/*` (new/updated)
  - startup success from cache-hit path
  - forced loader/vtable/processor failure is startup-fatal with explicit diagnostic
  - `audioStatusChanged` notification broadcast
  - `getMeterFrame` returns non-null after frame production

### Manual verification (macOS)

1. `wavecraft start`
2. Open browser UI and validate:
   - transport connected true
   - audio status correct and transitions visible
3. Scenario matrix:
   - normal input/output
   - no input device
   - mic permission denied/revoked
   - no output device
4. Confirm:
   - no “connected but silent without explanation”
   - diagnostics actionable
   - metering works for both push and poll consumers

---

## Acceptance criteria mapping to low-level design

| AC ID | Acceptance criterion                                                                                     | Low-level design mapping                                                                           |
| ----- | -------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| AC-1  | Cache-hit startup cannot skip audio init attempt                                                         | “Problem Statement” + “Current-State Flow and Failure Points” + “Proposed Architecture Changes §1” |
| AC-2  | UI can distinguish transport connected vs audio runtime state                                            | “Additional issues #1” + “Proposed Architecture Changes §2, §3, §5”                                |
| AC-3  | `getMeterFrame` returns meaningful data in browser dev mode when available                               | “Additional issues #2” + “Proposed Architecture Changes §4”                                        |
| AC-4  | Structured diagnostics available for startup/device/permission/runtime failures                          | “Additional issues #3” + “Proposed Architecture Changes §2”                                        |
| AC-5  | Required runtime/protocol contracts are explicitly enforced; violations fail fast with clear diagnostics | “Scope / Non-goals” + “Proposed Architecture Changes §2, §3” + “Migration Strategy”                |

---

## Definition of done

- All acceptance criteria AC-1..AC-5 satisfied.
- Unit/integration tests added and passing.
- Startup fails explicitly when required runtime/protocol contracts are missing.
- Manual macOS scenario matrix executed successfully.
- `implementation-plan.md` persisted under the feature folder.
- No edits to `docs/roadmap.md`.
- No edits to any files under `docs/feature-specs/_archive/`.
