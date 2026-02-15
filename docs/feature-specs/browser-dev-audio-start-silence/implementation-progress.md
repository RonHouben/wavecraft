# Implementation Progress — browser-dev-audio-start-silence

## Status

In progress.

### 2026-02-15 — Focused fix: apply input/output gain in dev-mode runtime output modifiers

- Traced end-to-end path for `input_gain_level` and `output_gain_level`:
  - UI slider (`ParameterSlider`) → `useParameter.setValue()`
  - `@wavecraft/core` `ParameterClient.setParameter()` JSON-RPC `setParameter`
  - `wavecraft-bridge` handler → `DevServerHost::set_parameter()`
  - host forwards successful writes into `AtomicParameterBridge` for audio-thread reads
- Root cause: `dev-server/src/audio/server.rs::apply_output_modifiers()` only handled oscillator controls and never applied gain parameters to runtime output, so gain slider updates were accepted but had no audible effect in browser dev mode.
- Implemented minimal fix in `apply_output_modifiers()`:
  - read `input_gain_level` and `output_gain_level` lock-free from `AtomicParameterBridge`
  - compute combined gain and apply it to the current output buffer every callback
  - preserve existing oscillator enabled/frequency/level behavior exactly (including mute-on-disabled and phase continuity)
  - robust defaults/fallbacks: missing/invalid gains resolve to unity (`1.0`), finite values clamped to `0.0..=2.0`
- Added regression tests in `dev-server/src/audio/server.rs`:
  - `output_modifiers_apply_input_and_output_gain_levels`
  - `output_modifiers_apply_gain_without_oscillator_params`

### 2026-02-15 — Focused fix: runtime oscillator frequency/level bridge in dev audio callback

- Root cause identified: dev-mode FFI processor path still processes with `from_param_defaults()` each block, so runtime slider updates for `oscillator_frequency`/`oscillator_level` were never injected into DSP state.
- Implemented a focused, robust bridge in `dev-server/src/audio/server.rs`:
  - kept existing `oscillator_enabled` on/off behavior intact.
  - added runtime application of `oscillator_frequency` and `oscillator_level` in the audio callback via lock-free `AtomicParameterBridge` reads.
  - maintains oscillator phase continuity across callbacks for stable tone generation.
- Added regression tests covering:
  - generated output from runtime frequency+level params,
  - level=0 silence behavior,
  - waveform changes when frequency changes.
- Follow-up (documented): replace this targeted bridge with full generic parameter injection into the FFI process path when the vtable contract is extended beyond defaults-only params.

### 2026-02-15 — Regression fix: stale parameter ID cache dropped `oscillator_enabled`

- Fixed stale parameter sidecar cache validation in `wavecraft start`:
  - cache now invalidates when any file under `engine/src` is newer than `wavecraft-params.json`.
  - cache now invalidates when the CLI binary is newer than the sidecar (tooling/schema evolution safety).
  - existing dylib mtime check remains in place.
- Regenerated template TypeScript parameter ID augmentation to include:
  - `oscillator_enabled`
- Result:
  - typed `ParameterId` union now recognizes `oscillator_enabled` in `sdk-template/ui/src/App.tsx`.
  - runtime parameter list refresh is no longer blocked by stale sidecar metadata after source/tooling changes.

### 2026-02-15 — Temporary output-device policy change (default output now, selector later)

- Updated dev audio startup to require and use the **system default output device** by default when available.
- Removed input-only/metering-only startup fallback from dev-mode audio server initialization:
  - missing/unusable default output device now fails startup with structured `noOutputDevice` diagnostic.
  - successful startup now reports `runningFullDuplex` (no degraded no-output branch).
- This is intentionally scoped as a temporary policy until a dedicated UI output-device selector is implemented.

### 2026-02-15 — QA medium findings remediation (no-output semantics + fail-fast wording)

> Note: superseded in part by the policy update above. No-output-device startup is now fail-fast (`failed` + `noOutputDevice`) rather than a running degraded mode.

- Canonicalized no-output-device startup semantics in `cli/src/commands/start.rs`:
  - successful input-only startup is now emitted as:
    - `phase = Degraded`
    - `diagnostic.code = NoOutputDevice`
    - with explicit message + actionable hint
  - full-duplex startup remains `RunningFullDuplex` without diagnostics.
- Replaced stale startup log wording in fail-fast branches:
  - removed `"Continuing without audio..."`/`"Continuing without audio processing..."` style messaging.
  - replaced with explicit abort wording aligned to strict SDK dev-mode fail-fast policy.
- Added deterministic automated coverage for the no-output-device branch:
  - `status_for_running_audio_marks_input_only_as_degraded_with_no_output_device`
  - `status_for_running_audio_marks_full_duplex_as_running`
- Added feature-level QA evidence artifact:
  - `docs/feature-specs/browser-dev-audio-start-silence/test-plan.md`
  - includes manual matrix status, blocked row details, and retest criteria.

## Completed in this implementation pass

### 2026-02-15 — Integration validation for forced startup failures

- Fixed startup-failure status propagation for runtime loader/vtable failures in `wavecraft start`:
  - runtime loader acquisition failures now set explicit `failed` audio runtime status with structured diagnostics.
  - `audioStatusChanged` is now broadcast for these failures before startup aborts (strict fail-fast retained).
- Added integration-level server validation for forced startup-failure diagnostics:
  - new `dev-server/tests/audio_status_startup_failures.rs` starts a real WS server and validates broadcast delivery to connected clients.
  - covers forced failure diagnostics: `loaderUnavailable`, `vtableMissing`, `processorCreateFailed`.
  - verifies emitted state is `failed` with the expected diagnostic code for each case, and host state matches broadcast payload.
- Added CLI unit coverage for deterministic loader/vtable diagnostic mapping:
  - `classify_runtime_loader_error_maps_vtable_missing`
  - `classify_runtime_loader_error_maps_loader_unavailable`

### 2026-02-15 — Always-on audio status UI + runtime panic fix

- Removed UI opt-in consumption path for runtime audio status:
  - `ConnectionStatus` now always subscribes to `useAudioStatus()` and always renders an audio status badge.
  - Audio readiness/degraded/failed state is now visible by default without any template-app integration step.
- Removed hidden UI gating in status rendering:
  - no native-transport early return; transport and audio badges are always rendered.
- Fixed dev-server runtime panic during manual `wavecraft start --verbose` flows:
  - replaced `tokio::sync::RwLock::blocking_*` usage in `dev-server/src/host.rs` with `std::sync::RwLock` guarded reads/writes.
  - this removes `Cannot block the current thread from within a runtime` panics when host status/meter access occurs on Tokio runtime threads.
- Added tests:
  - `ui/packages/components/src/ConnectionStatus.test.tsx` for always-visible transport/audio badges and degraded/startup states.
  - `dev-server/src/host.rs` async-runtime regression test to verify status set/get works inside Tokio runtime context.

### 2026-02-14 — Delta pass: startup/status hardening + coherence verification

- Added targeted startup diagnostic classification hardening in CLI:
  - extracted `classify_audio_init_error()` in `cli/src/commands/start.rs` to make startup-failure mapping deterministic and testable.
  - added edge-case tests for strict mapping:
    - permission denied → `inputPermissionDenied`
    - no input device → `noInputDevice`
    - unknown backend error → `unknown`
- Expanded UI hook coverage for startup/status transitions in `useAudioStatus`:
  - added transition test: `initializing` → `runningFullDuplex` via `audioStatusChanged` notification.
  - added malformed contract payload test: malformed `audioStatusChanged` payload is ignored and prior valid status is retained.
- Performed coherence/regression verification after recent edits to:
  - `engine/crates/wavecraft-bridge/src/plugin_loader.rs`
  - `ui/packages/core/src/hooks/useAudioStatus.test.ts`
- Executed targeted and full checks (see Validation run section) with all checks passing.

### 2026-02-14 — Policy shift: remove backward compatibility priority (SDK dev mode)

- Removed compatibility layers for legacy browser-dev startup paths; current SDK contracts are now required.
- Enforced strict `ParameterHost` contract: `get_audio_status()` is now mandatory (no trait default fallback).
- Removed UI silent fallback behavior in `useAudioStatus()`:
  - when transport is connected but `getAudioStatus` fails, hook now reports explicit `failed` status with diagnostic instead of silently ignoring errors.
- Enforced strict runtime loader/vtable behavior in CLI audio-dev startup:
  - runtime loader acquisition now returns an error (startup fails) on missing/invalid dev vtable path.
  - audio startup failure in this path now aborts startup with explicit error instead of degrading silently.
- Removed old parameter discovery compatibility fallback in `wavecraft start`:
  - discovery build now requires `_param-discovery`; no fallback branch to standard build for older plugins.

### Phase 1 — Startup decoupling (root-cause fix)

- Refactored CLI startup to decouple **parameter metadata loading** from **runtime loader acquisition**:
  - Metadata now loads via `load_parameter_metadata()`.
  - Runtime audio loader now attempts independently via `load_runtime_plugin_loader()`.
- Removed cache-hit audio skip behavior:
  - Cache-hit metadata no longer implies "no loader/no audio".
  - `wavecraft start` now always attempts runtime loader/audio startup in audio-dev mode.

### Phase 2 — Audio runtime status model + propagation (server-side)

- Added audio runtime status model to protocol and host integration:
  - `AudioRuntimePhase`, `AudioDiagnosticCode`, `AudioDiagnostic`, `AudioRuntimeStatus`.
- Added status transition publishing from CLI startup flow:
  - `Initializing` before runtime attempt.
  - `RunningFullDuplex` / `RunningInputOnly` on success.
  - `Degraded` / `Failed` with diagnostics on failure paths.
- Added server broadcast support for status transitions:
  - `audioStatusChanged` notification broadcast via WS handle.

### Phase 3 — Protocol + bridge wiring

- Added IPC contract additions:
  - Method: `getAudioStatus`
  - Notification: `audioStatusChanged`
  - Result: `GetAudioStatusResult`
- Extended bridge host trait with explicit contract:
  - `get_audio_status() -> Option<AudioRuntimeStatus>` is required for all implementers.
- Added bridge handler dispatch for `getAudioStatus`.

### Phase 4 — Metering poll/push alignment

- Added latest meter snapshot storage in `DevServerHost`.
- Updated meter forwarding loop to:
  1. update host snapshot
  2. continue existing `meterUpdate` broadcast
- Updated `get_meter_frame()` to return latest snapshot instead of always `None`.

### Phase 5 — UI opt-in status consumption (initial)

- Added core TS status contract types/constants for audio status.
- Added `IpcBridge.getAudioStatus()` typed helper.
- Added new `useAudioStatus()` hook with:
  - fetch-on-connect (`getAudioStatus`)
  - live updates via `audioStatusChanged`
  - derived flags: `isReady`, `isDegraded`
- Kept `useConnectionStatus` semantics transport-only; audio readiness is explicitly reported via `useAudioStatus()`.

## Validation run in this pass

- Targeted Rust tests:
  - `cargo test --manifest-path engine/Cargo.toml -p wavecraft-protocol -p wavecraft-bridge`
  - `cargo test --manifest-path dev-server/Cargo.toml`
  - `cargo test --manifest-path cli/Cargo.toml`
- Targeted UI tests:
  - `cargo xtask test --ui`
- Full pre-handoff validation:
  - `cargo xtask ci-check --fix`

All above commands passed.

### 2026-02-14 — Commands run in this delta pass

- Targeted CLI test (new startup diagnostic classification tests):
  - `cargo test --manifest-path cli/Cargo.toml start::tests::classify_audio_init_error -- --nocapture`
- Targeted UI hook test:
  - `cd ui && npm test -- useAudioStatus.test.ts`
- Targeted protocol/bridge regression check:
  - `cargo test --manifest-path engine/Cargo.toml -p wavecraft-bridge -p wavecraft-protocol`
- Full repo check:
  - `cargo xtask ci-check`

All commands passed.

## Remaining work

1. Manual scenario matrix validation (macOS) for:
   - no input device
   - mic permission denied
   - no output device (input-only/degraded)
