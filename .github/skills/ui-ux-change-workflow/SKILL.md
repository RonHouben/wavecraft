---
name: ui-ux-change-workflow
description: Execute Wavecraft UI/UX changes through a practical end-to-end flow: discovery, proposal, implementation, verification, and handoff with minimal-scope commits.
---

# UI/UX Change Workflow

Use this skill to implement UI/UX changes in `ui/` and `sdk-template/ui/` with tight scope, predictable verification, and clear handoff notes.

## When to use

- New UX behavior, component interaction changes, or layout updates
- Multi-file UI changes requiring clear sequencing
- Any UI update that needs disciplined implementation + verification

## Guardrails

- Never edit `docs/feature-specs/_archive/**`.
- Do not edit `docs/roadmap.md` (PO-owned).
- Use project standards from:
  - `docs/architecture/coding-standards.md`
  - `docs/architecture/coding-standards-typescript.md`
  - `docs/architecture/coding-standards-css.md`
  - `docs/architecture/coding-standards-testing.md`

## End-to-end workflow

### 1) Discovery

- [ ] Identify user-visible goal and impacted UI surfaces.
- [ ] Locate existing component and styling patterns to reuse.
- [ ] Define explicit non-goals to avoid scope creep.

### 2) Proposal

- [ ] Break work into small, verifiable steps.
- [ ] Decide minimal file set to touch.
- [ ] Call out accessibility and token-compliance implications up front.

### 3) Implementation

- [ ] Apply smallest possible code changes per step.
- [ ] Keep commits incremental and logically grouped.
- [ ] Reuse existing components/tokens before adding new variants.

### 4) Verification

- [ ] Run lint/typecheck/tests relevant to touched areas.
- [ ] Perform manual keyboard pass (tab order, activation, focus visibility, escape flows).
- [ ] Perform quick UX sanity checks (empty/loading/error/disabled states).
- [ ] Confirm no accidental regressions in nearby components.

### 5) Handoff

- [ ] Summarize what changed and why.
- [ ] List verification performed (automated + manual keyboard checks).
- [ ] Note follow-ups separately from completed scope.

## Practical rules

- Prefer minimal-scope edits over broad refactors during UX delivery.
- Keep behavior and styling changes reviewable in small diffs.
- If scope expands, split into follow-up tasks instead of bundling everything.
