# Test Plan: Template Processors Module (v0.11.0)

## Overview
- **Feature**: Template Processors Module
- **Spec Location**: `docs/feature-specs/template-processors-module/`
- **Date**: 2026-02-08
- **Tester**: Tester Agent
- **Branch**: `feature/template-processors-module`

## Test Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 9 |
| ❌ FAIL | 3 |
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

**Actual Result**: All checks passed in 24.8s. Linting: Engine PASSED, UI PASSED. Tests: 142 engine tests + 28 UI tests all pass.

---

### TC-002: Version Bump Consistency

**Description**: Verify all crate versions bumped from 0.10.0 to 0.11.0

**Steps**:
1. Checked `engine/Cargo.toml` workspace version
2. Checked individual crate versions
3. Checked CLI dependency versions

**Expected Result**: All workspace and crate versions consistently show 0.11.0.

**Status**: ✅ PASS

**Actual Result**: All engine crate versions are 0.11.0 (workspace + wavecraft-protocol, wavecraft-dsp, wavecraft-bridge, wavecraft-macros, wavecraft-core, wavecraft-metering, wavecraft-dev-server). CLI depends on wavecraft versions 0.11.0. CLI's own version remains 0.9.1 (correct — CLI version is managed by CI auto-bump, not manually).

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

**Steps**:
1. Reviewed template README at `cli/sdk-templates/new-project/react/README.md`
2. Checked for key sections and code accuracy

**Expected Result**: README contains all key sections with accurate content.

**Status**: ❌ FAIL

**Actual Result**: README has all required sections (Project Structure, Enabling the Oscillator Example, Adding a New Processor) but contains **incorrect code examples** that would fail to compile. See Issue #1 and Issue #2.

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

**Steps**:
1. Reviewed `docs/guides/sdk-getting-started.md` for processors/ structure and trait API

**Expected Result**: Guide shows `processors/` directory in project structure, correct trait API.

**Status**: ❌ FAIL

**Actual Result**: The guide correctly shows the `processors/` directory structure and includes an accurate oscillator example. However, it contains incorrect `wavecraft_processor!` usage for custom processors. See Issue #3.

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

### Issue #1: README template shows non-compiling `wavecraft_processor!` usage for custom processors

- **Severity**: Medium
- **Test Case**: TC-009
- **Description**: The README template (`cli/sdk-templates/new-project/react/README.md`) shows wrapping custom processors with `wavecraft_processor!`, but the macro only supports built-in types (`Gain` and `Passthrough` — see `engine/crates/wavecraft-core/src/macros.rs` lines 58-98). These code examples would fail to compile if a user followed them.
- **Expected**: Documentation should use custom processors directly in `SignalChain![]` (matching the actual `lib.rs` template)
- **Actual**: Multiple incorrect examples throughout the README
- **Locations**:
  1. **Line 95**: `wavecraft_processor!({{plugin_name_pascal}}Oscillator => Oscillator);` — Won't compile
  2. **Line 102**: `signal: SignalChain![InputGain, {{plugin_name_pascal}}Oscillator, OutputGain],` — References non-existent wrapper
  3. **Lines 114–117** ("Enabling the Oscillator" section): Shows `SignalChain![InputGain, {{plugin_name_pascal}}Oscillator, OutputGain]` — Should be `SignalChain![InputGain, Oscillator, OutputGain]`
  4. **Line 177**: `wavecraft_processor!(MyFilter => Filter);` — Won't compile for custom `Filter` type
  5. **Line 180**: `signal: SignalChain![InputGain, {{plugin_name_pascal}}Oscillator, MyFilter, OutputGain],` — Uses non-existent wrappers
- **Suggested Fix**:
  - Remove `wavecraft_processor!` wrapping for custom processors in code examples
  - Use custom processor types directly in `SignalChain![]`
  - The "Development Workflow" section code block should match the actual generated `lib.rs`
  - "Enabling the Oscillator" → `signal: SignalChain![InputGain, Oscillator, OutputGain]`
  - "Adding a New Processor" step 4 → use `Filter` directly: `signal: SignalChain![InputGain, Oscillator, Filter, OutputGain]`

### Issue #2: README template shows incorrect Processor trait signature

- **Severity**: Medium
- **Test Case**: TC-009
- **Description**: The "Core Traits" section of the template README (lines 320–327) shows an outdated/incorrect Processor trait signature that doesn't match the actual trait in `engine/crates/wavecraft-dsp/src/traits.rs`.
- **Expected** (actual trait):
  ```rust
  pub trait Processor: Send + 'static {
      type Params: ProcessorParams + Default + Send + Sync + 'static;
      fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport, params: &Self::Params);
      fn set_sample_rate(&mut self, _sample_rate: f32) {}
      fn reset(&mut self) {}
  }
  ```
- **Actual** (in README template):
  ```rust
  pub trait Processor {
      fn prepare(&mut self, sample_rate: f32, max_block_size: usize);
      fn process(&mut self, transport: &Transport, buffer: &mut Buffer);
      fn reset(&mut self);
  }
  ```
- **Differences**:
  - Missing `Send + 'static` bound
  - Missing `type Params` associated type
  - Shows `prepare()` instead of `set_sample_rate()` (different name and signature)
  - `process()` has wrong argument order and types (`Buffer` vs `&mut [&mut [f32]]`, no `params` parameter)
- **Suggested Fix**: Replace the trait block in the README template with the actual trait signature from `wavecraft-dsp`.

### Issue #3: SDK Getting Started guide shows incorrect `wavecraft_processor!` usage

- **Severity**: Medium
- **Test Case**: TC-011
- **Description**: The SDK Getting Started guide (`docs/guides/sdk-getting-started.md`) shows wrapping custom processors with `wavecraft_processor!`, which won't compile.
- **Locations**:
  1. **Line 220**: `wavecraft_processor!(MyOscillator => Oscillator);` — Won't compile
  2. **Line 228**: `signal: SignalChain![InputGain, MyOscillator, OutputGain],` — References non-existent wrapper
  3. **Line 313**: `wavecraft_processor!(MyFilter => Filter);` — Won't compile
  4. **Line 314**: `signal: SignalChain![InputGain, MyOscillator, MyFilter, OutputGain],` — Uses non-existent wrappers
- **Suggested Fix**: Same approach as Issue #1 — use custom processors directly without `wavecraft_processor!` wrapping. The guide's signal chain examples should use `Oscillator` and `Filter` directly.

## Testing Notes

- The actual generated `lib.rs` template is **correct** — it uses `Oscillator` directly in `SignalChain![]` without wrapping it in `wavecraft_processor!`. The issues are all in the **documentation** (README template and SDK guide) which show incorrect patterns.
- The `wavecraft_processor!` macro only has match arms for `Gain` and `Passthrough` (in `engine/crates/wavecraft-core/src/macros.rs` lines 58-98). Any other type passed to it will cause a compile error.
- The core implementation (derive macro paths, re-exports, oscillator template, version bumps) is excellent and well-done.
- The `implementation-progress.md` correctly notes "wavecraft_processor! only supports built-in types" as a resolved issue, but this knowledge wasn't consistently applied to the README template and SDK guide code examples.
- All 3 issues are documentation fixes only — no code changes required.

## Sign-off

- [ ] All critical tests pass
- [x] All high-priority tests pass (no critical issues)
- [x] Issues documented for coder agent
- [ ] Ready for release: **NO** — 3 medium-severity documentation issues need fixing first
