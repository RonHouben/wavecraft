# Implementation Progress — Template Processors Module

## Status: Complete

## Changes Made

### Template Files (New)
- `cli/sdk-templates/new-project/react/engine/src/processors/oscillator.rs` — Sine-wave oscillator example with `ProcessorParams` derive
- `cli/sdk-templates/new-project/react/engine/src/processors/mod.rs` — Module exports with 4-step guide for adding new processors

### Template Files (Modified)
- `cli/sdk-templates/new-project/react/engine/src/lib.rs` — Updated to use `processors/` module, three-processor signal chain (gain-only default, oscillator commented out)
- `cli/sdk-templates/new-project/react/README.md` — Updated project structure, key files table, development workflow, oscillator enable instructions, "Adding a New Processor" guide

### SDK Code Changes
- `engine/crates/wavecraft-macros/src/processor_params.rs` — Changed derive macro to generate `::wavecraft::` paths instead of `wavecraft_dsp::`, enabling use in user projects
- `engine/crates/wavecraft-macros/tests/processor_params.rs` — Added `extern crate wavecraft_dsp as wavecraft;` alias for test compatibility
- `engine/crates/wavecraft-nih_plug/src/lib.rs` — Added `ParamSpec` to re-exports, added `ProcessorParams` derive macro re-export
- `engine/crates/wavecraft-nih_plug/src/prelude.rs` — No changes needed (derive macro not re-exportable alongside same-named trait)

### Version Bumps
- `engine/Cargo.toml` workspace version: 0.10.0 → 0.11.0
- All crate versions bumped: wavecraft-protocol, wavecraft-dsp, wavecraft-bridge, wavecraft-macros, wavecraft-core, wavecraft-metering, wavecraft-dev-server
- CLI dependency versions bumped accordingly

### Documentation Updates
- `docs/guides/sdk-getting-started.md` — Updated project structure, Processor trait signature, parameter definition examples
- `docs/architecture/high-level-design.md` — Updated User Project Structure diagram

## Issues Encountered & Resolved

1. **`wavecraft_processor!` only supports built-in types** — The macro only matches `Gain` and `Passthrough`. Custom processors like `Oscillator` must be used directly in `SignalChain![]`, not wrapped.

2. **`ProcessorParams` derive macro not found via prelude** — The `wavecraft::prelude::*` imports the trait but not the derive macro. Resolved by having users add `use wavecraft::ProcessorParams;` alongside the prelude import.

3. **Derive macro generates `wavecraft_dsp::` paths** — User projects don't depend on `wavecraft_dsp` directly. Fixed the macro to generate `::wavecraft::` paths (the Cargo rename for `wavecraft-nih_plug`). Added crate alias in tests.

4. **Version mismatch across workspace** — Bumping the workspace version wasn't sufficient; each crate had hardcoded `version = "0.10.0"`. Updated all crate Cargo.toml files and CLI dependencies.

## Validation

- ✅ SDK workspace compiles (`cargo check --workspace`)
- ✅ Engine lint passes (`cargo xtask lint --engine`)
- ✅ Engine tests pass (`cargo xtask test --engine`)
- ✅ CLI tests pass
- ✅ Derive macro tests pass (3/3)
- ✅ Generated project compiles (gain-only chain)
- ✅ Generated project compiles (oscillator-enabled chain)
- ✅ No unreplaced template variables in generated output
