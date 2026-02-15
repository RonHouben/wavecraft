# Test Plan: npm Cohort Lockstep Versioning

## Overview

- **Feature**: npm cohort lockstep versioning
- **Spec Location**: `docs/feature-specs/npm-cohort-lockstep-versioning/`
- **Date**: 2026-02-15
- **Tester**: Tester Agent

## Test Summary

| Status     | Count |
| ---------- | ----- |
| ✅ PASS    | 7     |
| ❌ FAIL    | 0     |
| ⏸️ BLOCKED | 0     |
| ⬜ NOT RUN | 0     |

## Prerequisites

- [x] `cargo xtask ci-check --full` passes
- [x] Workflow and docs changes available in working tree

## Test Cases

### TC-001: Baseline project validation (`ci-check --full`)

**Description**: Validate repository remains green with implementation merged in working tree.

**Steps**:

1. Run `cargo xtask ci-check --full`.
2. Review lint, tests, template validation, and CD dry-run sections.

**Expected Result**: All phases pass.

**Status**: ✅ PASS

**Actual Result**: All phases passed (documentation, linting, automated tests, template validation, CD dry-run).

---

### TC-002: Cohort trigger when either package changes

**Description**: Confirm workflow computes cohort activation from either npm package path filter.

**Steps**:

1. Review `detect-changes.outputs.npm-cohort` expression in `.github/workflows/continuous-deploy.yml`.
2. Validate expression includes `steps.filter.outputs.npm-core == 'true' || steps.filter.outputs.npm-components == 'true'`.

**Expected Result**: Cohort turns true when either package changes.

**Status**: ✅ PASS

**Actual Result**: Expression correctly ORs npm-core and npm-components flags (plus force inputs).

---

### TC-003: Single target version propagation

**Description**: Confirm one cohort target version is computed and consumed by both publish jobs.

**Steps**:

1. Review `publish-npm-cohort-prepare` outputs.
2. Confirm `publish-npm-core` and `publish-npm-components` reference `needs.publish-npm-cohort-prepare.outputs.version`.
3. Run local simulation of target computation (same shell logic used in workflow).

**Expected Result**: One target version fans out to both npm jobs.

**Status**: ✅ PASS

**Actual Result**: Workflow wiring is correct. Local simulation produced:

- core_local=0.7.5
- components_local=0.7.4
- core_published=0.7.29
- components_published=0.7.24
- target=0.7.29

---

### TC-004: Ordered publish semantics

**Description**: Ensure components publish happens only after core publish path.

**Steps**:

1. Review `publish-npm-components.needs`.
2. Review components job `if` guard requiring prepare success and core result success/skipped.

**Expected Result**: `@wavecraft/core` is always published/skipped before `@wavecraft/components`.

**Status**: ✅ PASS

**Actual Result**: Components job requires `publish-npm-core` in `needs` and gate conditions.

---

### TC-005: Idempotent rerun behavior

**Description**: Validate rerun-safety (already-published versions and existing tags).

**Steps**:

1. Review prepare outputs `core-already-published` / `components-already-published`.
2. Review publish steps that skip when already published.
3. Review tag creation steps that skip when tag already exists.
4. Run local simulation of `already_published` booleans.

**Expected Result**: Safe reruns with no duplicate publish/tag failures.

**Status**: ✅ PASS

**Actual Result**:

- Core publish and components publish each guard on already-published flags.
- Tag creation for both packages checks remote tag existence before creating.
- Local simulation: `core_already_published=true`, `components_already_published=false` for computed target.

---

### TC-006: Deprecated force input mapping + warnings

**Description**: Confirm legacy force inputs remain functional and emit migration warning.

**Steps**:

1. Review workflow_dispatch inputs (`force-publish-npm-core`, `force-publish-npm-components`) and canonical input (`force-publish-npm-cohort`).
2. Review `npm-cohort` output expression includes legacy + canonical force inputs.
3. Simulate warning branch with `LEGACY_FORCE=true`.

**Expected Result**: Legacy flags still trigger cohort and emit warning.

**Status**: ✅ PASS

**Actual Result**: Warning message and cohort force notice emitted as expected:
`::warning::Deprecated inputs force-publish-npm-core / force-publish-npm-components were used...`

---

### TC-007: Post-publish verification logic

**Description**: Validate verification checks exist for package presence and peer dependency alignment.

**Steps**:

1. Review `Post-publish verify cohort alignment` step.
2. Confirm checks:
   - `@wavecraft/core@<target>` exists
   - `@wavecraft/components@<target>` exists
   - components peer dependency on `@wavecraft/core` equals `^<target>`

**Expected Result**: Workflow fails if cohort version or peer alignment drifts.

**Status**: ✅ PASS

**Actual Result**: Checks are implemented and fail-fast with clear messages.

## Issues Found

None.

## Testing Notes

- End-to-end registry publish and GitHub tag push were not executed locally (requires actual workflow run context and publish credentials/trust setup).
- Functional confidence is based on:
  1. Full local `cargo xtask ci-check --full` pass
  2. Direct workflow logic inspection
  3. Practical shell simulation of target computation, trigger semantics, and warning branches.

## Sign-off

- [x] All critical tests pass
- [x] All high-priority tests pass
- [x] Issues documented for coder agent (none)
- [x] Ready for release: YES
