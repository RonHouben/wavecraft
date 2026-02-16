# Test Plan: Oscillator Waveform Selector

## Overview

- **Feature**: Oscillator waveform selector (Sine, Square, Saw, Triangle)
- **Spec Location**: `docs/feature-specs/oscillator-waveform-selector/`
- **Date**: 2026-02-16
- **Tester**: Tester Agent
- **Scope**:
  - UI waveform dropdown behavior
  - Runtime audio behavior in browser dev mode
  - Oscilloscope and metering response
  - Regression coverage in engine/dev-server/UI tests

## Test Summary

| Status     | Count |
| ---------- | ----- |
| ✅ PASS    | 0     |
| ❌ FAIL    | 0     |
| ⏸️ BLOCKED | 0     |
| ⬜ NOT RUN | 9     |

## Prerequisites

- [ ] `cargo xtask ci-check --fix` passes
- [ ] Dev server starts with `cargo xtask dev`
- [ ] Feature is available in UI (`Waveform` select in oscillator control)

## Test Cases

### TC-001: Waveform selector renders with expected options

**Description**: Verify oscillator UI shows waveform selector and all 4 options.

**Preconditions**:

- Dev server is running.
- Oscillator control is visible.

**Steps**:

1. Open UI in browser dev mode.
2. Locate `Oscillator signal` panel.
3. Open the `Waveform` dropdown.
4. Verify options include `Sine`, `Square`, `Saw`, `Triangle`.

**Expected Result**: Dropdown is visible and all 4 waveform options are present.

**Status**: ⬜ NOT RUN

**Actual Result**: _Pending execution._

**Notes**: _None._

---

### TC-002: Switching waveform changes oscilloscope shape

**Description**: Verify selecting different waveform updates the displayed shape.

**Preconditions**:

- Oscillator output is ON.
- Frequency and level are non-zero.

**Steps**:

1. Select `Sine` and observe scope shape.
2. Select `Square` and observe scope shape.
3. Select `Saw` and observe scope shape.
4. Select `Triangle` and observe scope shape.

**Expected Result**: Scope shape clearly changes according to selected waveform.

**Status**: ⬜ NOT RUN

**Actual Result**: _Pending execution._

**Notes**: _None._

---

### TC-003: Square waveform sounds consistent with expected timbre

**Description**: Verify square does not indicate accidental dual-source mixing.

**Preconditions**:

- Audio output enabled.
- Oscillator ON.

**Steps**:

1. Set waveform to `Square`.
2. Keep frequency around 220–440 Hz and level around 0.5.
3. Listen for stable square-like timbre.
4. Compare briefly against `Sine` and `Saw`.

**Expected Result**: Audible timbre difference is consistent with single selected waveform.

**Status**: ⬜ NOT RUN

**Actual Result**: _Pending execution._

**Notes**: _None._

---

### TC-004: Oscillator output toggle mutes/unmutes signal

**Description**: Verify ON/OFF control still works after waveform implementation.

**Preconditions**:

- Oscillator panel visible.

**Steps**:

1. Turn oscillator ON and verify signal status / meter activity.
2. Turn oscillator OFF.
3. Confirm output drops to silence / no signal.
4. Turn ON again and confirm signal returns.

**Expected Result**: Toggle reliably controls output state and UI status.

**Status**: ⬜ NOT RUN

**Actual Result**: _Pending execution._

**Notes**: _None._

---

### TC-005: Frequency and level controls work for all waveforms

**Description**: Verify existing controls remain functional with waveform selection.

**Preconditions**:

- Oscillator ON.

**Steps**:

1. Set each waveform (`Sine`, `Square`, `Saw`, `Triangle`).
2. Adjust frequency slider and confirm pitch change.
3. Adjust level slider to low and high values.

**Expected Result**: Frequency and level updates apply consistently for each waveform.

**Status**: ⬜ NOT RUN

**Actual Result**: _Pending execution._

**Notes**: _None._

---

### TC-006: Engine waveform unit/integration tests

**Description**: Validate oscillator waveform tests in Rust processor crate.

**Preconditions**:

- Rust toolchain available.

**Steps**:

1. Run waveform-focused processor tests.

**Expected Result**: All waveform tests pass.

**Status**: ⬜ NOT RUN

**Actual Result**: _Pending execution._

**Notes**: _None._

---

### TC-007: Dev-server runtime waveform tests

**Description**: Validate browser-dev runtime output modifier waveform behavior.

**Preconditions**:

- Rust toolchain available.

**Steps**:

1. Run `output_modifiers_*` test subset in dev-server crate.

**Expected Result**: Runtime waveform and gain behavior tests pass.

**Status**: ⬜ NOT RUN

**Actual Result**: _Pending execution._

**Notes**: _None._

---

### TC-008: UI component tests for waveform controls

**Description**: Validate waveform select and oscillator control UI tests.

**Preconditions**:

- Node dependencies installed.

**Steps**:

1. Run UI tests for `ParameterSelect`, `OscillatorControl`, and app rendering mapping.

**Expected Result**: UI tests pass and verify enum control behavior.

**Status**: ⬜ NOT RUN

**Actual Result**: _Pending execution._

**Notes**: _None._

---

### TC-009: Full local CI verification

**Description**: Run full local CI check to confirm no regressions.

**Preconditions**:

- Workspace builds successfully.

**Steps**:

1. Run `cargo xtask ci-check --fix` from workspace root.

**Expected Result**: Docs, lint, and all tests pass.

**Status**: ⬜ NOT RUN

**Actual Result**: _Pending execution._

**Notes**: _None._

## Issues Found

_No issues recorded yet._

## Testing Notes

- Manual interactive checks (TC-001..TC-005) should be executed against active `cargo xtask dev` session.
- If browser interaction tooling is unavailable, record those as BLOCKED and provide explicit user-run steps.

## Sign-off

- [ ] All critical tests pass
- [ ] All high-priority tests pass
- [ ] Issues documented for coder agent
- [ ] Ready for release: YES / NO
