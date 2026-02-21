---
name: design-token-compliance
description: Ensure Wavecraft UI changes use existing design tokens and reusable patterns instead of ad-hoc values, with a fast normalization checklist for spacing, color, typography, and states.
---

# Design Token Compliance

Use this skill to audit and normalize UI styling in `ui/` and `sdk-template/ui/` so changes follow existing theme tokens and reusable component patterns.

## When to use

- Any Tailwind/className/style changes
- New components or component variants
- Refactors that touch spacing, color, typography, borders, shadows, or interaction states

## Guardrails

- Never edit `docs/feature-specs/_archive/**`.
- Do not edit `docs/roadmap.md` (PO-owned).
- Follow:
  - `docs/architecture/coding-standards.md`
  - `docs/architecture/coding-standards-typescript.md`
  - `docs/architecture/coding-standards-css.md`
  - `docs/architecture/coding-standards-testing.md`

## Audit + normalization workflow

- [ ] List changed UI files and collect new/modified class strings.
- [ ] Replace ad-hoc visual values with existing tokens/utilities first.
- [ ] Normalize spacing scale (avoid arbitrary one-off gaps/margins unless justified).
- [ ] Normalize typography scale/weight/line-height to established patterns.
- [ ] Normalize color usage for text/surfaces/borders using existing theme roles.
- [ ] Ensure state consistency (hover/focus/active/disabled/error) across similar controls.
- [ ] Reuse existing component patterns before creating new styling variants.
- [ ] Keep class composition readable; extract repeated patterns when reused.

## Anti-patterns to avoid

- Arbitrary values when an existing token exists (`text-[#...]`, `px-[13px]`, etc.)
- Inline styles for themeable properties
- Duplicate near-identical variants instead of shared patterns
- Inconsistent state styling between equivalent controls
- Introducing new “temporary” visual constants without rationale

## Done criteria

- Changed UI uses existing tokens/patterns by default.
- Any new token-like value is rare, justified, and intentionally scoped.
- Similar components have consistent spacing, type scale, and interaction states.
- Styling changes remain minimal-scope and easy to maintain.
