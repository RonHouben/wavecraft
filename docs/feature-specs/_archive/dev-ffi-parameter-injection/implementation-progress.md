# Implementation Progress — Dev FFI Parameter Injection v2 and DSP Unification

## Date

2026-02-21

## Completed in this pass

### Phase 1 — Protocol: Introduce FFI v2 ABI ✅

- Added `apply_plain_values` function pointer to `DevProcessorVTable` in `wavecraft-protocol`.
- Bumped `DEV_PROCESSOR_VTABLE_VERSION` from `1` to `2`.
- Updated ABI comments to document v2 behavior.

### Phase 2 — Macros: Generate v2-compatible processor wrapper ✅

- Updated generated dev FFI wrapper to store per-instance state:
  - processor instance
  - mutable params cache (`ProcessorParams`)
- `create()` now initializes the params cache once from defaults.
- `process()` now uses cached params (no per-callback `from_param_defaults()`).
- Added generated `apply_plain_values(instance, values_ptr, len)` and wired it into the returned vtable.
- Added/updated macro generation assertions to lock in v2 wrapper behavior.

### Phase 3 — Bridge: strict v2 by default + explicit v1 compat toggle ✅

- Loader now enforces strict v2 by default.
- Added explicit opt-in migration toggle: `WAVECRAFT_DEV_FFI_V1_COMPAT=1`.
- Added actionable mismatch diagnostics including remediation guidance.
- Added unit tests for compat env parsing and mismatch diagnostic variants.

### Phase 4 — Dev-server block-boundary injection ✅ (core path)

- Added `apply_plain_values(&[f32])` to `DevAudioProcessor` and implemented it in `FfiProcessor`.
- Added ordered dense snapshot support to `AtomicParameterBridge`:
  - `parameter_count()`
  - `copy_all_to(&mut [f32])`
- Audio callback now:
  1. snapshots parameter values once per block,
  2. calls `apply_plain_values` once per block,
  3. runs processor `process`.
- Compatibility path is explicit and opt-in:
  - `output_modifiers` parameter-semantic DSP is only applied when `WAVECRAFT_DEV_FFI_V1_COMPAT=1`.

## Remaining work

### Phase 5 — Remove DSP duplication by default ✅

- Made the migration path explicit in code and usage:
  - renamed `apply_output_modifiers` → `apply_v1_compat_output_modifiers`
  - switched `InputCallbackPipeline` to track compat state as `Option<f32>` phase only when opt-in is enabled
  - default path keeps strict v2 behavior (`apply_plain_values` + processor `process`) with no legacy post-process path state.
- Compatibility behavior remains explicit opt-in only via `WAVECRAFT_DEV_FFI_V1_COMPAT=1` for migration burn-in.
- Full sunset/removal of the compat branch remains a follow-up after migration parity confidence.

### Phase 6 — Documentation closure (pending)

- Update `docs/architecture/development-workflows.md` with final v2 flow and migration closure details.

## Notes

- This pass intentionally stopped at a coherent checkpoint after Phase 4 because Phase 5 is gated on parity validation and migration burn-in.
- Compatibility remains explicit opt-in only, aligned with pre-1.0 strict policy.
