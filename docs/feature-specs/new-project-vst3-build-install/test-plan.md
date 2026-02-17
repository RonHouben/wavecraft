# Test Plan: New Project VST3 Build + Install

## Overview

- **Feature**: `new-project-vst3-build-install`
- **Spec Location**: `docs/feature-specs/new-project-vst3-build-install/`
- **Date**: 2026-02-17
- **Tester**: Tester Agent
- **Branch Context**: current feature branch state after recent edits (including template validation flow and getting-started docs updates)

## Test Summary

| Status     | Count |
| ---------- | ----: |
| ✅ PASS    |     3 |
| ❌ FAIL    |     0 |
| ⏸️ BLOCKED |     0 |
| ⬜ NOT RUN |     0 |

## Prerequisites

- [x] Run core feature validation commands from workspace root
- [x] Capture concrete command evidence for each test case
- [x] Include template validation and broad repo verification

## Test Cases

### TC-001: CLI bundle command tests

**Description**: Verify CLI bundle command behavior and diagnostics for generated/new-project workflows.

**Command**:  
`cargo test --manifest-path cli/Cargo.toml --test bundle_command -- --nocapture`

**Expected Result**:

- Bundle command tests execute successfully.
- Install flag behavior and invalid-context diagnostics are covered and passing.

**Actual Result**:

- Test binary ran successfully.
- 5/5 tests passed.

**Evidence Snippet**:

- `test_help_shows_bundle_command ... ok`
- `test_bundle_help_shows_install_flag ... ok`
- `test_bundle_install_invalid_context_has_actionable_message ... ok`
- `test_bundle_without_install_invalid_context_has_actionable_message ... ok`
- `test_bundle_install_detects_project_root_from_subdirectory ... ok`
- `test result: ok. 5 passed; 0 failed`

**Status**: ✅ PASS

---

### TC-002: Template validation path (includes bundle/install contract)

**Description**: Validate end-to-end generated plugin path, including engine/UI validation and canonical bundle/install flow.

**Command**:  
`cargo xtask validate-template`

**Expected Result**:

- Template generation succeeds.
- Engine and UI validation pass.
- CLI bundle contract (bundle and `bundle --install`) passes.
- Overall validation reports success.

**Actual Result**:

- All validation stages passed.
- Bundle artifacts generated and install path validated.

**Evidence Snippet**:

- `Engine validation passed`
- `UI validation passed`
- `CLI bundle contract validation passed`
- `Installed VST3: /Users/ronhouben/Library/Audio/Plug-Ins/VST3/my_plugin.vst3`
- `✅ All validation checks passed (86.6s)`
- `Template validation successful!`

**Status**: ✅ PASS

---

### TC-003: Full repository validation sweep

**Description**: Re-run broad project validation after recent edits, including docs and template-related checks.

**Command**:  
`cargo xtask ci-check --full`

**Expected Result**:

- Documentation, linting, tests, template validation, and CD dry-run all pass.

**Actual Result**:

- All phases passed successfully.

**Evidence Snippet**:

- `Documentation: PASSED (0.2s)`
- `Linting: PASSED (11.0s)`
- `Automated Tests: PASSED (11.3s)`
- `Template Validation: PASSED (85.7s)`
- `CD Dry-Run: PASSED (0.0s)`
- `Total time: 126.6s`
- `All checks passed! Ready to push.`

**Status**: ✅ PASS

## Issues / Blockers

No functional failures detected in required verification scope.

### Notes

- Template validation output includes npm audit warnings (`11 moderate severity vulnerabilities`) during dependency installation. These did not fail validation and are not blockers for this feature verification pass.

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] Required commands re-run with evidence captured
- [x] Ready for coder/QA handoff
- **Release Readiness (for tested scope)**: **YES**
