# Test Plan: Template Processors Module (v0.11.0)

## Overview
- **Feature**: Template Processors Module
- **Spec Location**: `docs/feature-specs/template-processors-module/`
- **Date**: 2026-02-08 (Re-test #2 after doc fixes)
- **Tester**: Tester Agent
- **Branch**: `feature/template-processors-module`

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 11 |
| ❌ FAIL | 1 |
| ⏸️ BLOCKED | 0 |
| ⬜ NOT RUN | 0 |

## Prerequisites

- [x] `cargo xtask ci-check` passes (all lint + tests)
- [ ] macOS-only checks pass (if applicable): bundle, sign, install *(not required for this feature)*

## Test Cases

### TC-001: Automated CI Checks (Lint + Tests)

**Description**: Verify all lint and test suites pass across engine and UI.

**Steps**:
1. Run `cargo xtask ci-check` from the engine directory

**Expected Result**: All linting (ESLint, Prettier, cargo fmt, clippy) and tests (Vitest, cargo test) pass with exit code 0.

**Status**: ✅ PASS

**Actual Result**: All checks passed in 12.2s. Linting: Engine PASSED (fmt + clippy), UI PASSED (ESLint + Prettier). Tests: 148 engine tests (including doctests) + 28 UI tests all pass. Zero failures.

---

### TC-002: Version Bump Consistency

**Description**: Verify all crate versions bumped from 0.10.0 to 0.11.0

**Steps**:
1. Checked `engine/Cargo.toml` workspace version
2. Checked individual crate versions
3. Checked CLI dependency versions

**Expected Result**: All workspace and crate versions consistently show 0.11.0.

**Status**: ✅ PASS

**Actual Result**: `engine/Cargo.toml` shows `version = "0.11.0"`. All workspace crates compiled at 0.11.0 (confirmed in TC-005 build output: wavecraft-protocol, wavecraft-dsp, wavecraft-bridge, wavecraft-macros, wavecraft-core, wavecraft-metering, wavecraft-dev-server, wavecraft-nih_plug all at 0.11.0). CLI's own version remains 0.9.1 (correct — CLI version is managed by CI auto-bump, not manually).

---

### TC-003: Template Generation — Default Structure

**Description**: Verify `wavecraft create` generates the correct project structure with processors module.

**Steps**:
1. Cleaned previous test output
2. Ran: `cargo run --manifest-path cli/Cargo.toml -- create TestPlugin --output target/tmp/test-plugin`
3. Verified directory structure

**Expected Result**: `processors/` directory with `mod.rs` and `oscillator.rs`, plus updated `lib.rs`.

**Status**: ✅ PASS

**Actual Result**: All expected files exist:
- `target/tmp/test-plugin/engine/src/processors/mod.rs` ✓
- `target/tmp/test-plugin/engine/src/processors/oscillator.rs` ✓
- `target/tmp/test-plugin/engine/src/lib.rs` (contains `mod processors;`) ✓

---

### TC-004: No Unreplaced Template Variables

**Description**: Verify no `{{...}}` template placeholders remain in generated output.

**Steps**:
1. Ran: `grep -rn '{{' target/tmp/test-plugin/`

**Expected Result**: No matches found.

**Status**: ✅ PASS

**Actual Result**: "No unreplaced template variables found ✓"

---

### TC-005: Default Chain Compiles (Gain-Only)

**Description**: Verify the default template (gain-only signal chain) compiles successfully.

**Steps**:
1. Ran: `cd target/tmp/test-plugin/engine && cargo check`

**Expected Result**: Compilation succeeds with warnings about unused Oscillator.

**Status**: ✅ PASS

**Actual Result**: Compilation succeeded. Expected warnings generated:
- `unused import: processors::Oscillator`
- `struct OscillatorParams is never constructed`
- `struct Oscillator is never constructed`

These are all expected since Oscillator is imported but not used in the default gain-only chain.

---

### TC-006: Oscillator Chain Compiles

**Description**: Verify the oscillator-enabled signal chain compiles when uncommented.

**Steps**:
1. Uncommented oscillator signal chain, commented default chain in generated lib.rs
2. Ran: `cd target/tmp/test-plugin/engine && cargo check`

**Expected Result**: Compilation succeeds without errors.

**Status**: ✅ PASS

**Actual Result**: Compilation succeeded with zero warnings. The oscillator chain `SignalChain![InputGain, Oscillator, OutputGain]` compiles perfectly.

---

### TC-007: ProcessorParams Derive Macro Paths

**Description**: Verify the derive macro generates `::wavecraft::` paths for user project compatibility.

**Steps**:
1. Reviewed `engine/crates/wavecraft-macros/src/processor_params.rs`
2. Engine tests ran as part of TC-001

**Expected Result**: All 3 derive macro tests pass. Generated code uses `::wavecraft::` paths.

**Status**: ✅ PASS

**Actual Result**: All 3 tests pass (`test_simple_param_specs`, `test_multiple_params`, `test_default_value_calculation`). The macro generates `::wavecraft::ParamSpec`, `::wavecraft::ProcessorParams`, `::wavecraft::ParamRange` paths. Test file uses `extern crate wavecraft_dsp as wavecraft;` alias for compatibility.

---

### TC-008: Re-exports in wavecraft-nih_plug

**Description**: Verify `ParamSpec` and `ProcessorParams` derive macro are properly re-exported.

**Steps**:
1. Reviewed `engine/crates/wavecraft-nih_plug/src/lib.rs`

**Expected Result**: Both `ParamSpec` and the `ProcessorParams` derive macro are re-exported.

**Status**: ✅ PASS

**Actual Result**:
- Line 19: `pub use wavecraft_dsp::{ParamRange, ParamSpec, Processor, ProcessorParams, Transport};` — ParamSpec included ✓
- Line 33: `pub use wavecraft_macros::ProcessorParams;` — Derive macro re-exported ✓
- Comment at lines 31-32 explains coexistence of derive macro and trait in different namespaces ✓

---

### TC-009: Generated README Quality

**Description**: Verify the generated README has correct project structure, oscillator instructions, and adding-processor guide.

**Preconditions**: Previously ❌ FAIL (Issues #1 and #2) — Fixed in this iteration.

**Steps**:
1. Reviewed template README at `cli/sdk-templates/new-project/react/README.md`
2. Checked all code examples for compile-correctness

**Expected Result**: README contains all key sections with accurate, compilable code examples.

**Status**: ✅ PASS

**Actual Result**: All 3 previously reported issues are fixed:

1. **Development Workflow code block (lines 85–106)**: `wavecraft_processor!` only wraps built-in types (`InputGain => Gain`, `OutputGain => Gain`). `Oscillator` used directly with comment explaining why. ✓
2. **Enabling the Oscillator (lines 119–126)**: Shows `SignalChain![InputGain, Oscillator, OutputGain]` — direct use, no wrapper. ✓
3. **Adding a New Processor step 4 (lines 175–184)**: `Filter` used directly in `SignalChain![]` with explicit comment: "Custom processors are used directly — no wavecraft_processor! wrapper needed." ✓
4. **Core Traits — Processor trait (lines 323–330)**: Correct trait signature with `type Params`, proper `process()` signature, `set_sample_rate()`, `reset()` — all matching `wavecraft-dsp/src/traits.rs`. ✓

---

### TC-010: Template Oscillator Implementation Quality

**Description**: Verify the oscillator template has proper parameters, state management, and comments.

**Steps**:
1. Reviewed `cli/sdk-templates/new-project/react/engine/src/processors/oscillator.rs`

**Expected Result**: Oscillator has all required elements per user story 2 acceptance criteria.

**Status**: ✅ PASS

**Actual Result**: All acceptance criteria met:
- [x] `#[derive(ProcessorParams, Default, Clone)]` on `OscillatorParams` ✓
- [x] `frequency` param: range 20–5000 Hz, default 440, unit "Hz", factor 2.5 ✓
- [x] `level` param: range 0–1, default 0.5, unit "%" ✓
- [x] `set_sample_rate()` implemented ✓
- [x] `reset()` clears phase to 0.0 ✓
- [x] Phase accumulation with wraparound ✓
- [x] Educational comments throughout (phase formula, sample rate, state management) ✓
- [x] `use wavecraft::ProcessorParams;` import for derive macro ✓
- [x] Comment explaining prelude vs derive macro distinction ✓

---

### TC-011: Documentation Updates — SDK Getting Started

**Description**: Verify SDK getting started guide reflects the new processors module structure.

**Preconditions**: Previously ❌ FAIL (Issue #3) — Partially fixed in this iteration.

**Steps**:
1. Reviewed `docs/guides/sdk-getting-started.md` main lib.rs example (lines 210–228)
2. Reviewed oscillator code example (lines 241–290)
3. Reviewed "Adding a Processor to Your Project" subsection (lines 306–320)

**Expected Result**: All code examples correctly use custom processors directly (no `wavecraft_processor!` wrapping).

**Status**: ❌ FAIL

**Actual Result**: The **main** lib.rs example (lines 210–228) was correctly fixed — Oscillator is used directly without wrapping. However, the **"Adding a Processor to Your Project"** subsection (lines 315–319) was NOT fixed and still shows incorrect code:

```rust
// lines 315-319 — STILL INCORRECT
use processors::{Oscillator, Filter};
wavecraft_processor!(MyFilter => Filter);
// signal: SignalChain![InputGain, MyOscillator, MyFilter, OutputGain],
```

Two errors remain:
1. `wavecraft_processor!(MyFilter => Filter)` — `Filter` is a custom type, this won't compile
2. `MyOscillator` — references a non-existent wrapper type (no `wavecraft_processor!(MyOscillator => ...)` is defined)

See Issue #1 below.

---

### TC-012: Documentation Updates — High-Level Design

**Description**: Verify high-level design document reflects the new template structure.

**Steps**:
1. Reviewed `docs/architecture/high-level-design.md` User Project Structure section

**Expected Result**: HLD diagram includes `processors/` directory.

**Status**: ✅ PASS

**Actual Result**: HLD correctly shows:
```
└── processors/      ← Custom DSP processors
    ├── mod.rs        ← Module exports
    └── oscillator.rs ← Example: sine-wave oscillator
```

---

## Issues Found

### Issue #1: SDK Getting Started guide — "Adding a Processor" section still uses incorrect `wavecraft_processor!` wrapping

- **Severity**: Medium
- **Test Case**: TC-011
- **Description**: The "Adding a Processor to Your Project" subsection in `docs/guides/sdk-getting-started.md` (lines 315–319) still shows wrapping a custom `Filter` processor with `wavecraft_processor!`, which only supports built-in types (`Gain` and `Passthrough`). It also references `MyOscillator` which doesn't exist.
- **Expected**:
  ```rust
  use processors::{Oscillator, Filter};
  // Custom processors are used directly — no wavecraft_processor! wrapper needed.
  // signal: SignalChain![InputGain, Oscillator, Filter, OutputGain],
  ```
- **Actual**:
  ```rust
  use processors::{Oscillator, Filter};
  wavecraft_processor!(MyFilter => Filter);
  // signal: SignalChain![InputGain, MyOscillator, MyFilter, OutputGain],
  ```
- **Location**: `docs/guides/sdk-getting-started.md` lines 315–319
- **Steps to Reproduce**:
  1. Open `docs/guides/sdk-getting-started.md`
  2. Navigate to "Adding a Processor to Your Project" → step 4
  3. Observe incorrect `wavecraft_processor!` wrapping of custom type and non-existent `MyOscillator` reference
- **Suggested Fix**:
  - Remove the `wavecraft_processor!(MyFilter => Filter);` line
  - Change `MyOscillator` → `Oscillator` and `MyFilter` → `Filter`
  - Optionally add a comment: "Custom processors are used directly — no wavecraft_processor! wrapper needed."

## Testing Notes

- **Test run #1** (pre-fix): 9/12 PASS, 3/12 FAIL — Issues #1 (README `wavecraft_processor!`), #2 (README Processor trait), #3 (Getting Started guide `wavecraft_processor!`)
- **Test run #2** (this run, after doc fixes): 11/12 PASS, 1/12 FAIL — 2 of 3 issues fixed (TC-009 README ✓). 1 remaining issue in TC-011.
- **Remaining issue**: The "Adding a Processor to Your Project" step 4 in `sdk-getting-started.md` (lines 315–319) was missed during the fix pass. The main lib.rs example in the same file was corrected, but the shorter subsection still has the old pattern.
- The remaining fix is a **3-line documentation change** — no code changes needed.
- All automated checks pass (lint + tests). All template generation and compilation tests pass.
- The core implementation (derive macro paths, re-exports, oscillator template, version bumps, template code) is excellent and well-done.

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass (no critical issues)
- [x] Issues documented for coder agent
- [ ] Ready for release: **NO** — 1 medium-severity documentation issue needs fixing first
