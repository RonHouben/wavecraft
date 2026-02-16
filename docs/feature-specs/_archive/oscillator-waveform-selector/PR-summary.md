## Summary

This PR adds a full waveform selector flow for the oscillator processor (Sine, Square, Saw, Triangle) across DSP, protocol, dev-server runtime, and UI components.

It also hardens enum parameter handling by validating macro enum defaults at compile time and updates the enum UI control behavior so missing variant metadata does not produce misleading fallback options.

## Changes

- **Engine / DSP / Protocol**
  - Added oscillator waveform parameter support and waveform sample generation for all four waveform types.
  - Added enum-parameter metadata support (`variants`) in protocol/type plumbing and bridge mappings.
  - Added macro validation for enum defaults in `ProcessorParams` derive:
    - default must be finite
    - default must be an integer index
    - default must be in range of declared variants

- **Dev Server Runtime**
  - Updated dev-server audio runtime to use shared waveform generation helpers from `wavecraft-processors` (`Waveform`, `generate_waveform_sample`) for waveform-shape parity with engine DSP.
  - Added/updated tests to assert runtime waveform-change behavior.

- **UI**
  - Added `ParameterSelect` for enum parameters.
  - Updated `ParameterGroup` and oscillator controls to render/use waveform enum selection.
  - Changed `ParameterSelect` behavior for enum params with no variants:
    - no synthetic/fallback options are rendered
    - select is disabled
    - helper text is shown: "No variants available"
    - warning is logged for diagnostics

- **Documentation**
  - Added/updated feature documents in `docs/feature-specs/oscillator-waveform-selector/`:
    - implementation plan
    - test plan
    - QA report

## Key Files Changed

- `engine/crates/wavecraft-processors/src/oscillator.rs`
- `engine/crates/wavecraft-macros/src/processor_params.rs`
- `engine/crates/wavecraft-protocol/src/ipc.rs`
- `engine/crates/wavecraft-nih_plug/src/lib.rs`
- `dev-server/src/audio/server.rs`
- `ui/packages/components/src/ParameterSelect.tsx`
- `ui/packages/components/src/ParameterSelect.test.tsx`
- `ui/packages/components/src/ParameterGroup.tsx`
- `ui/packages/components/src/OscillatorControl.tsx`
- `ui/packages/core/src/types/parameters.ts`

## Commits

- `bec8fcd` fix(parameter-select): correct hasNoVariants condition to ensure proper variant check
- `8c8f562` feat(parameter-select): enhance variant handling with warning and helper text for missing variants
- `07cfce1` feat(oscillator): refactor waveform sample generation and update related tests
- `e3d1f1c` feat(qa): add QA report for oscillator waveform selector with validation results
- `1587830` feat(tests): add comprehensive test plan for oscillator waveform selector
- `2d8b0c0` feat(oscillator): add oscillator waveform parameter and update related tests
- `991ed16` test(ts_codegen): add test for number value type emission in enum parameters
- `1fb153c` style: format code for better readability in various files
- `ff64b7b` feat(oscillator): add waveform selection and related UI components
- `6fb91e5` feat(oscillator): implement selectable waveform types with UI integration

## Related Documentation

- [Implementation Plan](./implementation-plan.md)
- [Test Plan](./test-plan.md)
- [QA Report](./QA-report.md)

## Validation Evidence

- ✅ Full local checks passed: `cargo xtask ci-check --fix`
- ✅ Engine waveform tests passed (`wavecraft-processors`, oscillator test suite)
- ✅ Dev-server runtime waveform tests passed (`output_modifiers_*` coverage)
- ✅ UI tests passed including:
  - `ParameterSelect.test.tsx`
  - `OscillatorControl.test.tsx`
  - `AppParameterRendering.test.tsx`
- ✅ Manual local validation confirmed by user (waveform selector behavior and audio output)

## Intentionally Accepted Behavior

- Enum parameters without `variants` are intentionally treated as non-interactive in `ParameterSelect` (disabled control + helper text), rather than inventing fallback options.
- Dev-mode runtime currently includes focused oscillator parameter bridging (`oscillator_*`) as an intentional transitional path while broader generic FFI parameter injection evolves.

## Checklist

- [x] Feature implementation completed
- [x] Tests updated and passing
- [x] QA recheck completed (PASS)
- [x] `cargo xtask ci-check --fix` passes
- [x] PR summary created for workflow compliance
