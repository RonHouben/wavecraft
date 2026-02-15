# Test Evidence — browser-dev-audio-start-silence

Date: 2026-02-15  
Owner: Coder agent

## Scope

Validate startup/audio status behavior in browser dev mode after QA medium findings:

1. Canonical no-output-device semantics
2. Fail-fast startup wording/behavior consistency
3. Deterministic automated coverage for no-output-device branch

### Temporary policy note (2026-02-15)

- Browser dev mode now targets **audible output by default** using the system default output device.
- If no usable default output device exists, startup now fails with explicit diagnostics instead of entering input-only degraded mode.
- UI output-device selection is deferred to a future feature.

## Automated evidence

| Area                                  | Command                                                                                                | Result  | Notes                                                                             |
| ------------------------------------- | ------------------------------------------------------------------------------------------------------ | ------- | --------------------------------------------------------------------------------- |
| CLI startup status semantics          | `cargo test --manifest-path cli/Cargo.toml start::tests::status_for_running_audio -- --nocapture`      | ✅ Pass | Covers full-duplex runtime status behavior (no degraded no-output running state). |
| CLI startup diagnostic mapping        | `cargo test --manifest-path cli/Cargo.toml start::tests::classify_audio_init_error -- --nocapture`     | ✅ Pass | Confirms no-output startup failures map to `noOutputDevice`.                      |
| Dev-server startup failure broadcasts | `cargo test --manifest-path dev-server/Cargo.toml --test audio_status_startup_failures -- --nocapture` | ✅ Pass | Confirms explicit `failed` diagnostics are broadcast.                             |
| Repo checks                           | `cargo xtask ci-check`                                                                                 | ✅ Pass | Confirms no regressions across lint/tests/build.                                  |

## Manual matrix (browser dev mode, macOS)

| Scenario                         | Expected runtime phase/diagnostic                                 | Status                    | Evidence / blocker                                                                                         |
| -------------------------------- | ----------------------------------------------------------------- | ------------------------- | ---------------------------------------------------------------------------------------------------------- |
| Input + output devices available | `runningFullDuplex`, no diagnostic                                | ⚪ Not rerun in this pass | Covered by existing startup baseline; no code path change for this branch beyond status helper extraction. |
| No output device                 | startup aborts (`failed` + `noOutputDevice`)                      | ⚪ Pending rerun          | Requires controllable no-output hardware state on local machine or virtual device setup.                   |
| No input device                  | `failed` + `noInputDevice` (or mapped startup failure diagnostic) | ⚪ Not rerun in this pass | Existing fail-fast behavior retained; classification path unchanged except log wording.                    |
| Mic permission denied            | `failed` + `inputPermissionDenied`                                | ⚪ Not rerun in this pass | Existing fail-fast behavior retained; classification path unchanged except log wording.                    |
| Runtime loader/vtable failure    | `failed` + `loaderUnavailable` / `vtableMissing`                  | ✅ Covered                | Verified by automated integration + unit tests.                                                            |

## Blocked rows and retest criteria

### Blocked/Pending

- **No output device** startup-abort scenario is pending manual rerun due environment setup dependency.

### Retest criteria to close blocked row

1. Start browser dev mode with no output device available.
2. Confirm startup aborts with explicit `noOutputDevice` diagnostic.
3. Re-enable output device and restart; confirm `runningFullDuplex`.

## Pass/Fail gate

This feature slice is considered QA-complete for these findings when:

- automated tests above are green,
- pending no-output-device manual row is executed and evidence attached,
- no stale "Continuing without audio..." wording remains in startup/audio fail-fast paths.
