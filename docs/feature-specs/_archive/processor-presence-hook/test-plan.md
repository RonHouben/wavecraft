# Test Plan: Processor Presence Hook

## Overview

- **Feature**: Processor Presence Hook
- **Spec Location**: `docs/feature-specs/processor-presence-hook/`
- **Date**: 2026-02-15
- **Tester**: Tester Agent

## Test Summary

| Status     | Count |
| ---------- | ----- |
| ✅ PASS    | 6     |
| ❌ FAIL    | 0     |
| ⏸️ BLOCKED | 0     |
| ⬜ NOT RUN | 0     |

## Scope Under Test

Feature-specific validation targets requested:

- Macro export behavior (`wavecraft_get_processors_json` / processor ID derivation)
- Loader parsing (`PluginParamLoader` processor metadata parsing)
- CLI code generation (`write_processor_types`, extraction path)
- Core hooks (`useHasProcessor`, `useAvailableProcessors`)
- End-to-end quality gate (`cargo xtask ci-check`)

## Prerequisites

- [x] Run focused Rust/UI tests for feature-touched areas
- [x] Run `cargo xtask ci-check` from `engine/` workspace

## Test Cases

### TC-001: Macro processor export + ID derivation tests

**Description**: Verify macro layer correctly parses signal chain processors and derives canonical IDs.  
**Command**: `cd engine && cargo test -p wavecraft-macros`  
**Expected Result**: Macro/unit tests pass, including processor ID derivation and signal-chain parsing.  
**Status**: ✅ PASS  
**Actual Result**: 8 tests passed (3 plugin-focused + 5 processor_params).  
**Notes**: Confirms new processor metadata preparation behavior in macro crate.

---

### TC-002: Bridge loader parsing for processor metadata

**Description**: Verify bridge/plugin-loader handles processor JSON parsing and error handling.  
**Command**: `cd engine && cargo test -p wavecraft-bridge`  
**Expected Result**: `plugin_loader` tests pass for processor JSON parse success/failure paths.  
**Status**: ✅ PASS  
**Actual Result**: 29 unit tests + 2 doctests passed. Includes:

- `test_parse_processors_json`
- `test_parse_processors_json_invalid`  
  **Notes**: Loader parsing path is validated.

---

### TC-003: CLI processor codegen and extraction paths

**Description**: Verify CLI test suite covers processor TS codegen behavior and extraction plumbing.  
**Command**: `cd cli && cargo test`  
**Expected Result**: CLI tests pass; processor codegen tests pass.  
**Status**: ✅ PASS  
**Actual Result**: 89 unit tests + integration tests passed. Includes:

- `writes_deterministic_sorted_processor_output`
- `writes_marker_only_for_empty_processor_list`
- `errors_on_reserved_processor_marker_id_collision`  
  **Notes**: Confirms deterministic and guarded processor TS generation.

---

### TC-004: Dev-server rebuild callback compatibility

**Description**: Verify rebuild pipeline changes (processor callback additions) do not regress existing behavior.  
**Command**: `cd dev-server && cargo test`  
**Expected Result**: Existing reload and server tests pass.  
**Status**: ✅ PASS  
**Actual Result**: 36 unit + 5 integration/doc tests passed.  
**Notes**: `reload_cancellation` tests remain green after callback struct expansion.

---

### TC-005: Core hook behavior tests (new feature hooks)

**Description**: Verify `useHasProcessor` and `useAvailableProcessors` behavior in isolation.  
**Command**: `cd ui && npm run test -- packages/core/src/hooks/useHasProcessor.test.ts packages/core/src/hooks/useAvailableProcessors.test.ts`  
**Expected Result**: Both hook test files pass.  
**Status**: ✅ PASS  
**Actual Result**: 2 files passed; 4 tests total passed.  
**Notes**: Feature hooks are functionally correct in targeted tests.

---

### TC-006: Full formal pipeline (`ci-check`)

**Description**: Validate repository-level quality gate after feature integration.  
**Command**: `cd engine && cargo xtask ci-check`  
**Expected Result**: All phases pass.  
**Status**: ✅ PASS  
**Actual Result**:

- Docs: pass
- UI dist build: pass
- Lint: pass
- Automated tests: pass
- Final result: `cargo xtask ci-check` passed after fixes.
  **Notes**: Previously failing Issue #1 and Issue #2 were fixed; full gate is now green.

## Issues Found

### Issue #1: Conditional React hook usage flagged by ESLint in Oscilloscope component

- **Severity**: High
- **Test Case**: TC-006
- **Description**: `react-hooks/rules-of-hooks` violations in `ui/packages/components/src/Oscilloscope.tsx` (lines 40/44/57).
- **Expected**: Hooks called unconditionally in stable order.
- **Actual**: ESLint reports conditional hook invocation.
- **Evidence**: `cargo xtask ci-check` lint phase output.
- **Suggested Fix**: Refactor `Oscilloscope` hook calls to be unconditional and move branching logic below hook declarations.
- **Retest status**: Resolved (hooks reordered; lint now passes).

### Issue #2: Oscilloscope tests not updated for new `useHasProcessor` dependency in mocked module

- **Severity**: Medium
- **Test Case**: TC-006
- **Description**: `Oscilloscope.test.tsx` mocks `@wavecraft/core` without `useHasProcessor`, causing runtime test failures.
- **Expected**: Mock includes all consumed exports.
- **Actual**: 5 test failures with “No `useHasProcessor` export is defined on the `@wavecraft/core` mock.”
- **Evidence**: `cargo xtask ci-check` automated test phase output.
- **Suggested Fix**: Update `vi.mock('@wavecraft/core', ...)` in `Oscilloscope.test.tsx` to include `useHasProcessor` (or partially mock with `importOriginal` and override selectively).
- **Retest status**: Resolved (focused test now passes 5/5).

## Sign-off

- [x] Focused feature tests executed
- [x] Full `ci-check` executed
- [x] Failures documented with actionable evidence
- [x] Ready for release: **YES**

**Recommendation**: Hand off to **Coder** for fixes, then re-run this test plan for confirmation.

## Recommendation

All previously identified issues are resolved and formal checks are passing. Recommend handoff to **QA** (approved path), and then proceed to final documentation/merge workflow.

## Todo list (final status)

- [x] Review feature implementation docs and define test scope.
- [x] Run focused tests for macro export, loader parsing, CLI codegen, and core hooks.
- [x] Run `cargo xtask ci-check` for full formal validation.
- [x] Document pass/fail outcomes and concrete failure evidence.
- [x] Prepare `test-plan.md` content for DocWriter persistence handoff.
- [x] Provide release recommendation (QA vs Coder).

## Retest Update (2026-02-15)

- Re-ran formal gate: `cargo xtask ci-check`
- Re-ran focused lint: `npm run lint -- packages/components/src/Oscilloscope.tsx` (from `ui/`)
- Re-ran focused test: `npm run test -- packages/components/src/Oscilloscope.test.tsx` (from `ui/`)

Results:

- Issue #2 (missing `useHasProcessor` mock export) is resolved.
  - Focused test file passes: 5/5.
  - UI automated tests in full gate pass.
- Issue #1 (hooks-order lint) is resolved; focused lint now passes.

Updated recommendation:

- Keep handoff to **Coder** until Issue #1 is fixed.
- QA handoff remains blocked.
