## Summary

Fix browser-dev audio startup silence by introducing explicit audio runtime status contracts, deterministic startup diagnostics, stricter parameter/range handling, and improved CI/CD dry-run change detection/reporting.

## Changes

- **Engine/DSP**:
  - Added audio runtime status handling across protocol and bridge layers (`AudioRuntimeStatus`, diagnostics, status methods/notifications).
  - Updated parameter contracts and host/bridge behavior for declared `min`/`max` ranges, boolean handling, and canonical gain parameter IDs.
  - Extended DSP behavior for dev-mode processing and oscillator enable/silence behavior.
  - Added/updated Rust tests for audio startup failure diagnostics and parameter/range flows.

- **UI**:
  - Added/updated hooks and types for audio runtime status (`useAudioStatus`, IPC type extensions).
  - Integrated oscillator control and improved connection/runtime status display behavior.
  - Expanded component/core tests (parameter rendering, reconnect behavior, connection status, oscillator controls).

- **Build/Config**:
  - Improved `engine/xtask` CD dry-run reporting and change-detection flow.
  - Enhanced dev/preflight workflow behavior for UI artifacts and generated parameter handling.
  - Updated template/runtime glue in CLI/dev-server paths to support deterministic browser-dev startup.

- **Documentation**:
  - Updated architecture and workflow docs for audio status contracts and startup expectations.
  - Added/updated feature-spec documentation for browser-dev audio startup work.

## Commits

- `caf92d7` fix: enhance CD dry-run command with improved change detection and summary reporting
- `fb17e16` fix: streamline import statements in App component for clarity
- `3ec343e` feat: implement audio runtime status contract and improve startup diagnostics in browser dev mode
- `5914144` fix: update parameter handling to support boolean types and refactor related components
- `6b5ffa8` fix: enforce strict use of canonical gain parameter IDs and remove legacy support
- `751af0e` fix: correct default value for oscillator enabled parameter
- `47add8f` fix: refactor shutdown handling and improve gain parameter management
- `9482955` fix: apply input/output gain levels in dev-mode audio processing
- `4adbe87` feat: Add min and max range support for parameters
- `1415561` fix(dev): enhance preflight checks for UI artifacts and parameter caches
- `d570814` feat: implement OscillatorControl component and integrate into App
- `4bbd889` feat: add audio runtime status handling and parameter initialization

## Related Documentation

- [Feature Spec: Browser Dev Audio Start Silence](../browser-dev-audio-start-silence/)
- [Implementation Plan](../browser-dev-audio-start-silence/implementation-plan.md)
- [Implementation Progress](../browser-dev-audio-start-silence/implementation-progress.md)
- [Low-Level Design](../browser-dev-audio-start-silence/low-level-design-browser-dev-audio-start-silence.md)
- [Test Plan](../browser-dev-audio-start-silence/test-plan.md)

## Testing

- [x] Build/lint/tests executed for affected Rust + UI paths during implementation
- [x] Audio startup diagnostics validated for failure cases (including startup status coverage)
- [x] UI behavior validated for runtime/connection status and parameter updates
- [x] CI/CD dry-run behavior validated for change-detection summary improvements

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No known remaining lint/typecheck blockers for merged change set
