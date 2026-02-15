## Feature Name

`browser-dev-audio-start-silence`

## Problem Statement

In browser dev mode (`wavecraft start`), the UI can connect successfully (WebSocket + IPC request/response works) while audio remains silent.  
The primary failure mode is in `cli/src/commands/start.rs`:

- `load_parameters()` returns `(params, None)` on sidecar cache hit.
- `run_dev_servers()` only calls `try_start_audio_in_process()` when `loader: Some(...)`.
- Result: control-plane initializes, but audio runtime is skipped entirely.

This creates a misleading “connected” state with no explicit audio readiness signal.

Additional issues:

1. **No explicit audio readiness contract** from CLI/dev-server to UI.
2. **Metering contract mismatch**:
   - UI public API still supports polling `getMeterFrame` (`useMeterFrame`).
   - `DevServerHost::get_meter_frame()` returns `None` in browser dev mode.
   - Meter data is actually push-only via `meterUpdate`.
3. **Poor diagnostics** for input device absence, permissions, output fallback, and FFI/vtable failures.

---

## Scope

### In scope

1. Make audio startup deterministic and independent of parameter-cache hit path.
2. Add explicit runtime audio status model and IPC contract for status visibility.
3. Align metering contracts so polling and push are coherent.
4. Add actionable diagnostics for device/permission/runtime failures.
5. Preserve backwards compatibility for existing UI clients.

### Non-goals

1. No DSP algorithm changes.
2. No DAW/plugin runtime architecture changes (this is browser dev mode only).
3. No roadmap edits.
4. No breaking JSON-RPC changes for existing methods.

---

## Current-State Flow and Failure Points

### Startup sequence today

1. `StartCommand::execute()` → `run_dev_servers(...)`
2. `load_parameters(...)`:
   - cache miss → build + dylib load → returns `(params, Some(loader))`
   - cache hit → returns `(params, None)` **(critical)**
3. WebSocket server starts; UI connects (`useConnectionStatus` shows connected).
4. Audio start:
   - only attempted if `loader.is_some()`
   - cache-hit path logs skip and proceeds without audio
5. UI remains connected, parameters load, but no audio output and no explicit “audio not running” status.

### Failure points

1. **Cache-coupled loader lifecycle** in `cli/src/commands/start.rs` (`load_parameters` conflates metadata load and runtime loader ownership).
2. **No runtime state machine** for audio startup progression and failure reasons.
3. **Status blind spot in UI** (`useConnectionStatus` reports transport only).
4. **Metering mismatch** (`getMeterFrame` contract vs push-based `meterUpdate` reality).
5. **Diagnostics lost in console logs** instead of structured status codes.

---

## Proposed Architecture Changes (module/file granularity)

### 1) Decouple parameter metadata loading from runtime audio loader

#### `cli/src/commands/start.rs`

Introduce separation of concerns:

- `load_parameter_metadata(...) -> ParameterMetadataResult`
  - source: `CacheHit | DiscoveryBuild | FallbackBuild`
  - params only
- `load_runtime_plugin_loader(...) -> Option<PluginLoader>`
  - always attempts to load runtime dylib when `audio-dev` is enabled
  - independent from metadata source

New flow in `run_dev_servers(...)`:

1. Load metadata (may come from cache).
2. Start WebSocket + hot reload as today.
3. Always attempt runtime loader acquisition for audio path.
4. Attempt audio start with explicit status transitions.
5. Publish status transitions through dev-server status channel + IPC notification.

This removes the cache-hit silent-audio bug by construction.

---

### 2) Add explicit audio runtime state model

#### New file: `dev-server/src/audio/status.rs`

Define:

- `AudioRuntimePhase`:
  - `Disabled`
  - `Initializing`
  - `RunningFullDuplex`
  - `RunningInputOnly`
  - `Degraded`
  - `Failed`
- `AudioDiagnosticCode` (examples):
  - `LoaderUnavailable`
  - `VtableMissing`
  - `ProcessorCreateFailed`
  - `NoInputDevice`
  - `InputPermissionDenied`
  - `NoOutputDevice`
  - `StreamStartFailed`
  - `Unknown`
- `AudioRuntimeStatus`:
  - phase
  - diagnostic (optional structured code/message/hint)
  - sample_rate / buffer_size (optional)
  - updated_at

#### `dev-server/src/session.rs`

- Own shared `AudioRuntimeStatus` state (e.g. `Arc<RwLock<AudioRuntimeStatus>>`).
- Pass status handle to WS layer and host adapter.

#### `dev-server/src/ws/mod.rs`

- Add server-side broadcast helper:
  - `broadcast_audio_status_changed(...)`
- Broadcast on transitions:
  - initializing → running/degraded/failed.

---

### 3) IPC contract additions for status visibility

#### `engine/crates/wavecraft-protocol/src/ipc.rs`

Add:

- `METHOD_GET_AUDIO_STATUS = "getAudioStatus"`
- `NOTIFICATION_AUDIO_STATUS_CHANGED = "audioStatusChanged"`
- `GetAudioStatusResult { status: AudioRuntimeStatus }`
- Serialized types for `AudioRuntimeStatus`, `AudioRuntimePhase`, `AudioDiagnostic`.

#### `engine/crates/wavecraft-protocol/src/lib.rs`

- Re-export new status types/constants.

#### `engine/crates/wavecraft-bridge/src/host.rs`

- Extend `ParameterHost` trait:
  - `fn get_audio_status(&self) -> Option<AudioRuntimeStatus> { None }` (default method for compatibility)

#### `engine/crates/wavecraft-bridge/src/handler.rs`

- Add method dispatch for `getAudioStatus`.
- Return `GetAudioStatusResult`.

#### `dev-server/src/host.rs`

- Implement `get_audio_status()` using shared runtime status state.

---

### 4) Metering contract alignment (poll + push compatibility)

#### `dev-server/src/host.rs`

- Store latest meter frame snapshot (updated by meter-forwarding task).
- `get_meter_frame()` returns latest snapshot instead of always `None`.

#### `cli/src/commands/start.rs` (meter forwarding task)

- In task draining meter ring buffer:
  1. update host’s latest meter snapshot
  2. broadcast existing `meterUpdate` notification

This keeps push path intact while making polling hooks valid again.

---

### 5) UI status API and hooks

#### `ui/packages/core/src/types/ipc.ts`

- Add TypeScript interfaces for `AudioRuntimeStatus`, `AudioRuntimePhase`, `AudioDiagnostic`.
- Add method/notification string constants.

#### `ui/packages/core/src/ipc/IpcBridge.ts`

- Add typed helper for `getAudioStatus`.
- Handle `audioStatusChanged` notification dispatch.

#### New hook: `ui/packages/core/src/hooks/useAudioStatus.ts`

- Behavior:
  - fetch `getAudioStatus` on connect
  - subscribe to `audioStatusChanged`
  - expose `{ phase, diagnostic, isReady, isDegraded }`

#### `ui/packages/core/src/index.ts`

- Export new hook/types.

#### Existing `useConnectionStatus`

- Keep transport-only semantics (no breaking change).
- Document that transport connected != audio ready.

---

## Sequence Changes

### Revised startup sequence

1. CLI loads parameter metadata (cache/build).
2. CLI starts WS + host + session.
3. Status: `Initializing` broadcast.
4. CLI always attempts runtime loader acquisition (audio-dev mode).
5. If loader/vtable/processor succeeds:
   - start audio server
   - status `RunningFullDuplex` or `RunningInputOnly`
6. If any failure:
   - status `Degraded` or `Failed` with diagnostic code/hint
7. UI:
   - transport connected state from existing hook
   - audio readiness from `useAudioStatus`.

---

## API/Contract Changes Summary

### New JSON-RPC method

- `getAudioStatus` → returns structured runtime status.

### New notification

- `audioStatusChanged` with same payload as status result.

### Metering behavior

- `getMeterFrame` returns latest snapshot in browser dev mode (no longer permanently null).
- Existing `meterUpdate` notification unchanged.

### Compatibility guarantee

- Existing methods and notifications unchanged.
- Older UI clients continue functioning.
- New clients can feature-detect `getAudioStatus` (handle method-not-found).

---

## Migration Strategy and Backward Compatibility

1. **Phase 1 (server first):**
   - implement runtime decoupling + status method/notification.
   - keep existing UI behavior untouched.
2. **Phase 2 (UI opt-in):**
   - add `useAudioStatus` and optional badges/warnings.
3. **Fallback behavior:**
   - if `getAudioStatus` not supported, UI falls back to transport-only.
4. **No breaking changes** to existing request/response schemas.

---

## Test Strategy

### Unit tests

#### `cli/src/commands/start.rs`

- cache-hit metadata path still attempts runtime loader acquisition.
- audio start attempted on cache hit (not skipped).
- status transitions emitted on success/failure.

#### `engine/crates/wavecraft-protocol/src/ipc.rs`

- serialize/deserialize new status types and constants.

#### `engine/crates/wavecraft-bridge/src/handler.rs`

- `getAudioStatus` method dispatch success and fallback behavior.

#### `dev-server/src/host.rs`

- meter snapshot set/get behavior.
- audio status getter behavior.

### Integration tests

#### `dev-server/tests/`

- startup with simulated cache hit + mock loader: reaches running status.
- startup with forced loader/vtable failure: status becomes degraded/failed with diagnostic.
- `audioStatusChanged` is broadcast to connected clients.
- `getMeterFrame` returns non-null once meter frames are produced.

### Manual verification (macOS browser dev mode)

1. Run `wavecraft start` from repository root.
2. Open browser UI:
   - transport connected true
   - audio status visible and accurate.
3. Test scenarios:
   - normal input/output device present
   - no input device
   - microphone permission denied/revoked
   - no output device (input-only degradation)
4. Confirm:
   - no silent-success state
   - diagnostics actionable
   - metering works via push and poll path.

---

## Rollout and Risk Mitigation

### Rollout plan

1. Land internal startup decoupling first (highest impact, lowest API risk).
2. Add status IPC contracts and default host method.
3. Add UI hook as non-breaking enhancement.
4. Add docs for interpreting transport vs audio readiness.

### Risks and mitigations

1. **Risk:** status drift from real runtime state  
   **Mitigation:** single status owner + explicit transition points only.
2. **Risk:** extra synchronization overhead  
   **Mitigation:** status/meter snapshot updates occur off audio callback path; audio thread remains lock-free.
3. **Risk:** mixed-version client/server behavior  
   **Mitigation:** method-not-found fallback and additive protocol evolution.
4. **Risk:** false permission diagnosis across platforms  
   **Mitigation:** diagnostic codes normalized from cpal errors with conservative mapping and raw error passthrough.

---

## Acceptance Criteria

1. Cache-hit startup path cannot skip audio initialization attempt.
2. UI can distinguish:
   - transport connected
   - audio running/degraded/failed.
3. `getMeterFrame` is meaningful in browser dev mode once audio frames exist.
4. Structured diagnostics are available for device/permission/startup failures.
5. Existing clients remain functional without changes.

---

## Notes

- This design keeps real-time constraints intact: no additional blocking/allocating work added to audio callback.
- The major architectural correction is uncoupling **metadata cache** from **runtime loader ownership**.
