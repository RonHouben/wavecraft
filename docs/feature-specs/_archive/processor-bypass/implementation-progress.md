# Implementation Progress — Processor Bypass

## Summary

Implemented end-to-end processor bypass as a parameter-level feature with no new IPC methods.

## Completed

- Added framework-level bypass wrapper in DSP combinators:
  - `Bypassed<P>` and `BypassedParams<P::Params>`
  - Auto-injected `bypass` param (`Stepped 0..1`, default `0`)
  - Dry passthrough when bypassed by skipping child `process()`
- Updated `SignalChain!` to wrap every processor instance in `Bypassed<...>` automatically.
- Updated macro generation to include bypass in:
  - runtime parameter construction
  - FFI-discovered parameter metadata (`wavecraft_get_params_json`)
- Added processor-instance-specific bypass display names (`"{ProcessorName} Bypass"`).
- Fixed repeated-type processor ID collisions in macro-generated metadata/runtime IDs:
  - Added deterministic per-instance ID prefixing in `wavecraft-macros`.
  - Repeated processor types now emit unique IDs (`gain`, `gain_2`, `gain_3`, ...).
  - This applies to both processor metadata IDs and derived parameter IDs (including bypass IDs).
- Added UI/core helper APIs:
  - `isBypassParameterId()`
  - `getProcessorBypassParamId()`
  - `useProcessorBypass()` hook
- Updated SDK template to remove static `BypassStage` example and clarify dynamic bypass behavior.

## Decisions

1. **No new IPC methods**
   - Bypass is exposed through existing parameter APIs (`getParameter`, `setParameter`, `getAllParameters`).
2. **Per-instance bypass IDs**

- IDs follow existing naming and are emitted as `{processor_id}_bypass`.
- For repeated identical processor types, instance IDs are now deterministic and unique by ordered suffixing: first instance keeps legacy base ID, later instances are suffixed (`gain`, `gain_2`, ...).
- Example: `SignalChain![Gain, Gain]` now yields `gain_bypass` and `gain_2_bypass`.

3. **Realtime behavior (current)**

- Steady-state bypass uses dry passthrough with child DSP skipped.
- Bypass toggles now use a bounded short transition (fade-out + fade-in) to avoid hard discontinuities/clicks.

## Remaining

- Manual DAW validation remains a follow-up outside this coding pass.

## Regression Risk Fix (2026-02-21)

- Addressed tester-found collision risk where repeated identical processor types could produce duplicate bypass IDs.
- Updated macro code paths to use the same per-instance prefix strategy for:
  - Runtime parameter ID generation (`runtime_params.rs`)
  - FFI parameter metadata generation (`metadata.rs`)
  - Processor metadata IDs (`metadata.rs`)
- Added/updated macro tests to prove deterministic uniqueness for repeated types.

## QA Remediation (2026-02-21)

Addressed QA blocking findings from `QA-report.md`:

1. **Critical (real-time safety): runtime split path no longer depends on alloc/leak `param_specs()` calls**

- Added `ProcessorParams::plain_value_count()` (defaulting to `param_specs().len()`).
- Updated runtime split paths to use `plain_value_count()`:
  - `BypassedParams::apply_plain_values()`
  - `ChainParams::apply_plain_values()`
- Added regression tests that intentionally panic on `param_specs()` and verify runtime splitting still works, proving split logic no longer routes through alloc/leak metadata helpers.

2. **High (acceptance gap): click/pop-safe bypass transitions**

- Implemented bounded transition state machine in `Bypassed<P>` with short fade-out/fade-in on bypass toggles.
- Transition length is sample-rate aware and bounded (min/max clamp) to keep behavior deterministic and real-time safe.
- Added edge tests for both toggle directions ensuring smooth bounded sample-to-sample changes and correct eventual steady-state output.

3. **Behavior preserved**

- No new IPC methods were added.
- Per-instance bypass ID behavior remains unchanged (`{processor_id}_bypass` with suffixes for repeated processor types).
- Existing bypass and chain behavior tests remain in place and continue to validate baseline behavior.

### Compatibility Notes

- **Preserved:** Non-repeated processor type IDs remain unchanged.
- **Preserved:** For repeated types, the **first** instance keeps the legacy base ID (e.g., `gain`).
- **Changed (unavoidable for correctness):** Subsequent repeated instances now use suffixed IDs (e.g., `gain_2`, `gain_3`).

## Validation Run

- Targeted Rust tests passed:
  - `cargo test --manifest-path engine/Cargo.toml -p wavecraft-dsp`
  - `cargo test --manifest-path engine/Cargo.toml -p wavecraft-macros`
  - `cargo test --manifest-path engine/Cargo.toml -p wavecraft-bridge`
- Targeted UI tests passed:
  - `npm --prefix ui run test -- packages/core/src/hooks/useParameter.test.ts packages/core/src/hooks/useProcessorBypass.test.ts packages/core/src/processors/bypass.test.ts`
- Full local validation passed:
  - `cargo xtask ci-check`
