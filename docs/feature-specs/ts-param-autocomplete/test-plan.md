# Test Plan: TypeScript Parameter ID Autocompletion

## Overview

- **Feature**: TypeScript Parameter ID Autocompletion (`ts-param-autocomplete`)
- **Spec Location**: `docs/feature-specs/ts-param-autocomplete/`
- **Date**: 2026-02-13
- **Tester**: Tester Agent

## Test Summary

| Status     | Count |
| ---------- | ----: |
| ✅ PASS    |     6 |
| ❌ FAIL    |     0 |
| ⏸️ BLOCKED |     0 |
| ⬜ NOT RUN |     0 |

## Prerequisites

- [x] Implementation merged into working tree for manual validation
- [x] Automated checks passed earlier in session (`cargo xtask ci-check --fix`, CLI checks, template validation)
- [x] Isolated manual test fixture created at `target/tmp/manual-param-autocomplete/`

## Test Cases

### TC-001: `wavecraft start` generates parameter type file on startup

**Description**: Verify startup path generates `ui/src/generated/parameters.ts` automatically.

**Preconditions**:

- Fresh generated plugin exists at `target/tmp/manual-param-autocomplete/`
- Dev servers can be started on non-default ports

**Steps**:

1. Create a fresh plugin fixture with `wavecraft create` into `target/tmp/manual-param-autocomplete`.
2. Start dev servers using:
   - `cargo run --manifest-path cli/Cargo.toml -- start --install --port 3011 --ui-port 5181`
3. Check for existence of `ui/src/generated/parameters.ts` in the generated plugin.

**Expected Result**: Generated file exists and is non-empty after startup.

**Status**: ✅ PASS

**Actual Result**:

- File exists: `target/tmp/manual-param-autocomplete/ui/src/generated/parameters.ts`
- File length observed: 15 lines.

**Notes**:

- Startup path generation is functional.

---

### TC-002: Generated file format and augmentation structure

**Description**: Verify generated file format follows module augmentation design.

**Preconditions**:

- TC-001 completed

**Steps**:

1. Inspect generated file contents.
2. Verify header and module augmentation shape.

**Expected Result**:

- Includes auto-generated header.
- Uses `declare module '@wavecraft/core'`.
- Defines `interface ParameterIdMap` keys.
- Ends with `export {};`.

**Status**: ✅ PASS

**Actual Result**:

- File contains expected structure:
  - header comment
  - `declare module '@wavecraft/core'`
  - `interface ParameterIdMap { ... }`
  - `export {};`

**Notes**:

- Structural format is correct.

---

### TC-003: Hot-reload regenerates types after Rust source change

**Description**: Verify hot-reload pipeline regenerates `parameters.ts` when `engine/src` changes.

**Preconditions**:

- Dev server running via TC-001

**Steps**:

1. Modify generated plugin `engine/src/lib.rs` signal chain from:
   - `SignalChain![InputGain, OutputGain, Oscillator]`
     to
   - `SignalChain![InputGain, OutputGain]`
2. Wait for rebuild cycle.
3. Re-open `ui/src/generated/parameters.ts`.

**Expected Result**:

- File is regenerated with updated parameter ID set.
- Rebuild logs show successful hot-reload.

**Status**: ✅ PASS

**Actual Result**:

- File updated successfully.
- Rebuild terminal output confirmed:
  - `Hot-reload complete — 2 parameters (2 removed)`
- Regenerated file reflected reduced parameter set.

**Notes**:

- Hot-reload integration path is functioning.

---

### TC-004: Type safety rejects invalid parameter IDs at compile time

**Description**: Verify invalid parameter IDs fail TypeScript checking.

**Preconditions**:

- Generated project switched to local package dependencies for current implementation under test:
  - `@wavecraft/core` -> `file:/Users/ronhouben/code/private/wavecraft/ui/packages/core`
  - `@wavecraft/components` -> `file:/Users/ronhouben/code/private/wavecraft/ui/packages/components`

**Steps**:

1. Add `ui/src/param-id-typecheck-test.tsx`:
   - `useParameter('nonexistent_param')`
2. Run `npm --prefix .../ui run typecheck`.

**Expected Result**:

- TypeScript compilation fails with invalid ID error.

**Status**: ✅ PASS

**Actual Result**:

- TypeScript failed as expected:
  - `TS2345: Argument of type '"nonexistent_param"' is not assignable ...`

**Notes**:

- Compile-time guard behavior is working (against current local SDK packages).

---

### TC-005: Generated IDs match expected processor-based naming convention

**Description**: Verify generated IDs are human-readable and aligned with expected processor-derived naming convention.

**Preconditions**:

- Generated file available from TC-001/TC-003
- Template signal chain includes `InputGain`, `OutputGain`, optional `Oscillator`

**Steps**:

1. Inspect generated keys in `ParameterIdMap`.
2. Compare to expected convention (processor-wrapper-derived names, e.g. `input_gain_level`, `oscillator_frequency`).

**Expected Result**:

- IDs should be readable processor-derived IDs (matching feature intent and user-story examples).

**Status**: ✅ PASS (retested after fix)

**Actual Result**:

- Generated IDs are now processor-derived and human-readable, e.g.:
  - `'input_gain_level'`
  - `'output_gain_level'`
  - `'oscillator_frequency'`
  - `'oscillator_level'`

**Notes**:

- Root cause was in `wavecraft_plugin!` FFI param export path deriving a prefix from
  the full `SignalChain![...]` macro expression string.
- Fix implemented by parsing `SignalChain!` processor types and generating IDs using
  per-processor prefixes.
- Retest evidence:
  - `wavecraft extract-params` output now returns `input_gain_level`,
    `output_gain_level`, `oscillator_frequency`, `oscillator_level`.
  - `wavecraft start` regenerated `ui/src/generated/parameters.ts` with the same IDs.

---

### TC-006: `cargo xtask dev` path validation

**Description**: Verify feature behavior via `cargo xtask dev` path.

**Preconditions**:

- Test environment includes both generated plugin and SDK repository

**Steps**:

1. Attempt `xtask dev` in generated plugin (`target/tmp/manual-param-autocomplete/engine`).
2. Attempt `engine/xtask dev` in SDK repository.

**Expected Result**:

- At least one supported `xtask dev` workflow should be verifiable for this feature.

**Status**: ✅ PASS

**Actual Result**:

- Generated plugin `xtask` only exposes `bundle`, no `dev` command.
- SDK repo `cargo xtask dev` now starts correctly in SDK mode and reaches:
  - `✓ Loaded 4 parameters`
  - `✓ WebSocket server running`
  - `✓ All servers running!`
  - Vite serving UI on `http://localhost:5173/`

**Notes**:

- The root cause was dylib discovery preferring an existing but empty
  `sdk-template/engine/target/debug` directory over the actual build output directory.
- Fix applied in `cli/src/project/dylib.rs` to search all candidate debug directories
  and select a directory containing the plugin library.
- `cargo xtask dev` is now a valid workflow for validating this feature in SDK mode.

## Issues Found

### Issue #1: Generated parameter IDs were malformed and not processor-derived (Resolved)

- **Severity**: High
- **Test Case**: TC-005
- **Description**: Generated keys in `ui/src/generated/parameters.ts` include macro-expression-like prefixes rather than expected human-readable processor-derived IDs.
- **Expected**: IDs similar to `input_gain_level`, `output_gain_level`, `oscillator_frequency`.
- **Actual (before fix)**: IDs like:
  - `signal_chain! [_input_gain, _output_gain, _oscillator]_frequency`
  - `signal_chain! [_input_gain, _output_gain]_level`
- **Steps to Reproduce**:
  1. Generate plugin via `wavecraft create`.
  2. Run `wavecraft start` in generated plugin.
  3. Open `ui/src/generated/parameters.ts`.
- **Evidence**:
  - File content captured during manual run from:
    - `target/tmp/manual-param-autocomplete/ui/src/generated/parameters.ts`
- **Resolution**:
  - Fixed in `engine/crates/wavecraft-macros/src/plugin.rs` by parsing
    `SignalChain!` types and mapping `ParamSpec` values with per-processor prefixes.
  - Manual retest confirms generated IDs now follow expected naming.

## Testing Notes

- `wavecraft start` startup generation and hot-reload regeneration are both operational.
- Type safety enforcement works with local SDK package wiring and now aligns with ergonomic IDs.
- No runtime crashes encountered during manual validation.

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] Issues documented for coder agent
- [x] Ready for release: YES
