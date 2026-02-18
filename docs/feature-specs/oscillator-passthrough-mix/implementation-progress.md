# Implementation Progress — oscillator-passthrough-mix

**Milestone:** 18.11  
**Date:** 2026-02-18  
**Branch:** `bugfix/oscillator-passthrough-mix`

## Summary

Implemented the oscillator passthrough mix bugfix with minimal scope:

- Updated `Oscillator::process()` to follow additive generator semantics.
  - **Enabled:** adds oscillator sample to existing input (`+=`)
  - **Disabled:** passthrough no-op (returns without mutating buffer)
- Added regression tests to lock in:
  - disabled oscillator preserves passthrough
  - enabled oscillator remains audible with silent input
  - enabled oscillator adds to existing non-zero input (no overwrite)
- Added a small template clarification comment in default `SignalChain` usage.

No routing framework expansion or chain architecture changes were introduced.

## Files Changed

- `engine/crates/wavecraft-processors/src/oscillator.rs`
  - Core behavior fix in `process()`
  - Regression test updates/additions
- `sdk-template/engine/src/lib.rs`
  - Comment clarifying oscillator additive/no-op generator semantics

## Verification

### Focused tests

- `cargo test --manifest-path engine/Cargo.toml -p wavecraft-processors`
  - ✅ Passed
  - `18 passed; 0 failed`

### Repo-level checks

- `cargo xtask ci-check --fix`
  - ✅ Passed
  - Documentation links: passed
  - Lint/typecheck: passed
  - Engine tests: passed
  - UI tests: passed (`20` files, `96` tests)

## Notes for Tester Handoff

- Manual host smoke in Ableton/macOS is still required for milestone acceptance:
  1. Load generated plugin with audible DAW input.
  2. Enable oscillator.
  3. Verify oscillator + passthrough are both audible simultaneously.
  4. Disable oscillator and verify passthrough remains (no mute).

## Scope Guard

Confirmed scope remained limited to Milestone 18.11 bugfix.
No mixer/multi-bus/routing framework work was introduced.

---

## Follow-up Regression Fix (2026-02-18)

### Regression Report

During manual Ableton validation, UI showed:

- `Error: Parameter not found: oscillator_enabled`
- `Error: Parameter not found: oscillator_waveform`
- `Error: Parameter not found: oscillator_frequency`
- `Error: Parameter not found: oscillator_level`

and the oscillator toggle was non-interactive.

### Root Cause

`wavecraft bundle` did not refresh generated TypeScript contract files (`ui/src/generated/parameters.ts` and `ui/src/generated/processors.ts`) before building and embedding UI assets.

This allowed stale generated metadata to be packaged, creating a UI↔runtime contract mismatch when processor/parameter sets diverged across builds.

### Fix Implemented

- Updated CLI bundle flow in `cli/src/commands/bundle_command.rs` to refresh generated parameter/processor typings before UI build:
  - Sidecar-first metadata load (`wavecraft-params.json`, `wavecraft-processors.json`) when available.
  - Automatic fallback to discovery build (`cargo build --lib --features _param-discovery -p <engine_package>`) plus subprocess extraction (`extract-params` / `extract-processors`) when sidecars are missing.
  - Regenerates:
    - `ui/src/generated/parameters.ts`
    - `ui/src/generated/processors.ts`

### Regression Tests Added/Updated

- Added integration regression coverage in `cli/tests/bundle_command.rs`:
  - `test_bundle_refreshes_generated_contract_types_from_sidecars`
    - Seeds stale generated files + sidecar metadata
    - Verifies `wavecraft bundle` rewrites generated contract files from sidecars before UI build
- Updated bundle command fixture setup to seed realistic sidecar metadata used by bundle tests.

### Verification Performed

- `cargo test --manifest-path cli/Cargo.toml --test bundle_command` ✅
- `cargo fmt --manifest-path cli/Cargo.toml` ✅
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` ✅
- `cargo test --manifest-path cli/Cargo.toml` ✅
- `cargo xtask ci-check --fix` ✅
- Generated project workflow validation:
  - `cargo run --manifest-path cli/Cargo.toml -- bundle --install` from `target/tmp/osc-mix-test` ✅
  - Confirmed contract refresh path executed (`sidecars missing -> discovery build -> generated types synced`) and bundle/install completed.

---

## Second Follow-up: stale sidecar cache path in manual Ableton retest (2026-02-18)

### New Manual Failure Report

Manual Ableton retest still showed:

- `Error: Parameter not found: oscillator_enabled`
- `Error: Parameter not found: oscillator_waveform`
- `Error: Parameter not found: oscillator_frequency`
- `Error: Parameter not found: oscillator_level`

with oscillator UI controls non-reactive, despite audio passthrough working.

### Root Cause (why first follow-up was insufficient)

The first follow-up ensured `wavecraft bundle` regenerates `ui/src/generated/*.ts` from metadata sidecars. However, `bundle` accepted existing sidecars (`target/debug/wavecraft-params.json` and `wavecraft-processors.json`) without validating freshness.

For older generated projects (e.g., long-lived manual test projects), sidecars can be stale relative to newer `engine/src` changes. In that case, bundle regenerated TypeScript contracts from stale sidecars, preserving the UI↔runtime mismatch.

### Additional Fix Implemented

`cli/src/commands/bundle_command.rs` now validates sidecar freshness before using cache:

- Treats sidecars as stale when either sidecar is older than:
  - plugin dylib
  - newest file under `engine/src`
  - currently running CLI binary
- On stale cache, bundle logs reason and falls back to discovery build (`--features _param-discovery`) + subprocess metadata extraction.

This prevents silently shipping/installing stale parameter contracts from outdated sidecar caches.

### Regression Coverage Added

Added unit-level regression tests in bundle command module:

- `stale_sidecars_are_ignored_when_engine_source_is_newer`
  - Reproduces exact stale cache condition and asserts sidecars are ignored.
- `fresh_sidecars_are_used_for_contract_refresh`
  - Confirms fresh sidecars are still used (no unnecessary discovery build).

### Verification Performed

- `cargo test --manifest-path cli/Cargo.toml --test bundle_command` ✅
- `cargo xtask ci-check --fix` ✅

---

## Third Follow-up: runtime parameter ID mismatch + missing resize handle (2026-02-18)

### New Manual Failure Report

Manual Ableton retest (after rebuild/install + host restart) still showed:

- Audio passthrough works.
- Oscillator UI errors:
  - `Parameter not found: oscillator_enabled`
  - `Parameter not found: oscillator_waveform`
  - `Parameter not found: oscillator_frequency`
  - `Parameter not found: oscillator_level`
- Oscillator toggle remained non-reactive.
- Resize handle was missing from bottom-right (regression).

### Root Cause (why previous bundle fixes still failed)

There were two distinct issues:

1. **Runtime ID mismatch in macro-generated plugin params**
   - `wavecraft_plugin!` generated nih-plug `param_map()` IDs as `param_0`, `param_1`, ...
   - UI/generated contracts expected canonical IDs like `oscillator_enabled`.
   - Result: runtime bridge could not resolve oscillator IDs, so controls rendered `Parameter not found`.

2. **Template UI regression for resize affordance**
   - `sdk-template/ui/src/App.tsx` rendered oscillator/meter/latency panels but did not render `ResizeHandle`, so the resize grip disappeared in generated plugin UIs.

Previous bundle-sidecar freshness fixes were necessary but insufficient because they only affected how generated TS contract files are refreshed; they did **not** change runtime IDs exposed by the plugin.

### Additional Fixes Implemented

- **Macro/runtime fix** (`engine/crates/wavecraft-macros/src/plugin.rs`):
  - `__WavecraftParams` now stores canonical runtime IDs and groups alongside param pointers.
  - Generated `param_map()` now emits those canonical IDs (e.g. `oscillator_enabled`) instead of `param_N` placeholders.
  - This aligns runtime bridge IDs with generated TS contract IDs.

- **UI compatibility hardening** (`ui/packages/components/src/OscillatorControl.tsx`):
  - Added ID resolution fallback logic for oscillator controls:
    - canonical ID match first
    - explicit legacy aliases (`param_0..param_3`)
    - suffix-based fallback (`*_enabled`, etc.)
  - Prevents brittle failures when older runtime variants are encountered.

- **Resize handle restoration** (`sdk-template/ui/src/App.tsx`):
  - Re-added `ResizeHandle` component rendering in generated template app layout.

### Regression Coverage Added

- `engine/crates/wavecraft-macros/src/plugin.rs`
  - `generated_param_map_uses_prefixed_runtime_ids_instead_of_param_indexes`
  - Guards against reintroducing `param_N` runtime IDs.

- `ui/packages/components/src/OscillatorControl.test.tsx`
  - Added fallback coverage for legacy runtime IDs (`param_0..param_3`).

- `ui/packages/components/src/TemplateApp.test.tsx`
  - Ensures template app renders both oscillator control and resize handle.

---

## Fourth Follow-up: parameter plumbing + enum metadata + UI semantics polish (2026-02-18)

### New Manual Retest Issues (Ableton)

After prior fixes, manual retest still reported 5 remaining issues:

1. Oscillator toggle set to ON did not produce audible tone.
2. Waveform dropdown rendered empty.
3. Frequency slider values were unstable and occasionally snapped toward 0 Hz.
4. Signal badge could show active signal while oscillator toggle was OFF, which was misleading.
5. Resize handle remained hard to see/use in bottom-right corner.

### Root Causes

1. **Processor params were rebuilt from defaults, not live host/UI values**
   - Macro-generated plugin code used `ProcessorParams::from_param_defaults()` without applying current nih-plug parameter values.
   - Result: DSP oscillator processor received default params regardless of UI toggle/controls.

2. **Runtime bridge metadata did not expose enum variants**
   - Runtime parameter introspection hardcoded `param_type: Float` and `variants: None`.
   - Result: enum-backed `oscillator_waveform` had no options in `ParameterSelect`.

3. **Host callback forwarded normalized values to UI as-if plain values**
   - `param_value_changed()` used normalized values directly in JS notifications.
   - Result: UI slider state drift/snap behavior when round-tripping values.

4. **Signal badge wording implied oscillator-only signal source**
   - Component displayed generic output metering as “Oscillator signal.”
   - Result: badge could appear contradictory when oscillator was OFF but passthrough signal existed.

5. **Resize handle contrast/hit-area were too subtle for host embedding context**
   - Lightweight transparent styling reduced discoverability and interaction confidence.

### Fixes Implemented

- **Live param application path added end-to-end**
  - Added `ProcessorParams::apply_plain_values(&mut self, values: &[f32])` (default no-op).
  - Implemented for:
    - `GainParams`
    - `ChainParams<PA, PB>` (recursive split by child param counts)
    - `OscillatorParams`
  - Updated derive macro (`#[derive(ProcessorParams)]`) to auto-generate field mapping in param order.
  - Updated plugin macro `build_processor_params()` to collect current plain values from nih params and apply them before DSP processing.

- **Runtime enum/bool metadata restored for UI controls**
  - Bridge now inspects `step_count()` and `normalized_value_to_string()` to infer:
    - bool-like stepped params (`param_type: Bool`)
    - enum variants for stepped params (`param_type: Enum`, `variants: Some([...])`)

- **Normalized -> plain conversion fixed for host->UI param updates**
  - `param_value_changed()` now converts callback normalized values via `preview_plain()` before notifying UI.

- **Frequency max range set to full audible spectrum**
  - Oscillator frequency parameter max raised to `20_000.0` Hz.

- **Signal badge semantics clarified**
  - Label changed to **“Output signal (post-chain)”**.
  - Status logic now prioritizes oscillator state:
    - OFF -> `Off`
    - ON + output -> `Signal at output`
    - ON + no output -> `On (no output)`

- **Resize handle visibility improved**
  - Stronger anchored styling (`bottom-2 right-2`), larger hit area, persistent contrast background/border, and clearer icon state.

### Regression Coverage Added/Updated

- `engine/crates/wavecraft-dsp/src/combinators/chain.rs`
  - `test_chain_apply_plain_values_splits_by_child_param_count`

- `engine/crates/wavecraft-processors/src/oscillator.rs`
  - `apply_plain_values_updates_all_fields`
  - `frequency_param_uses_full_audible_range`

- `engine/crates/wavecraft-macros/src/plugin.rs`
  - Strengthened generated-code assertion to ensure live-value application call is present.

- `ui/packages/components/src/OscillatorControl.test.tsx`
  - Updated semantics expectations for output signal labeling.
  - Added explicit OFF-state regression check when output meter has signal.

- `ui/packages/components/src/ResizeHandle.test.tsx` (new)
  - Verifies visible anchored styling classes.
  - Verifies drag path triggers `requestResize` with minimum bounds.

### Verification Performed

- `cargo xtask ci-check --fix` ✅
  - Documentation links: passed
  - Lint/typecheck: passed
  - Engine tests: passed
  - UI tests: passed (`22` files, `101` tests)
