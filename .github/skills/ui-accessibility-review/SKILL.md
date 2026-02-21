---
name: ui-accessibility-review
description: Review Wavecraft UI changes for practical accessibility quality (semantic HTML, keyboard/focus behavior, contrast, ARIA, reduced motion) and provide fix-ready findings.
---

# UI Accessibility Review

Use this skill to run a focused accessibility review for React/TypeScript/Tailwind changes in `ui/` and `sdk-template/ui/`.

## When to use

- Any PR that changes UI components, forms, controls, or interaction flows
- New component creation or major visual restyling
- Before handoff when keyboard/focus behavior might be affected

## Guardrails

- Never edit `docs/feature-specs/_archive/**`.
- Do not edit `docs/roadmap.md` (PO-owned).
- Align with:
  - `docs/architecture/coding-standards.md`
  - `docs/architecture/coding-standards-typescript.md`
  - `docs/architecture/coding-standards-css.md`
  - `docs/architecture/coding-standards-testing.md`

## Review workflow (checklist)

- [ ] Identify changed UI files in `ui/` and `sdk-template/ui/`.
- [ ] Verify semantic structure first (`button`, `label`, `nav`, `main`, headings in logical order).
- [ ] Validate full keyboard path: tab order, enter/space activation, escape/close flows, no keyboard traps.
- [ ] Check focus visibility on all interactive elements (clear, persistent, non-color-only cues).
- [ ] Confirm accessible names for controls (visible label, `aria-label`, or `aria-labelledby`).
- [ ] Confirm ARIA is minimal and correct; prefer native semantics over custom roles.
- [ ] Validate color contrast for text, icons, and control states (default/hover/focus/disabled/error).
- [ ] Verify reduced motion support (`prefers-reduced-motion`) for transitions/animations.
- [ ] Re-test with keyboard only after fixes.

## Done criteria

- No critical keyboard or focus issues remain.
- Interactive controls have clear role/name/state.
- Contrast and focus indicators are usable in all relevant states.
- Motion-heavy behavior has a reduced-motion fallback.
- Findings are actionable, file-specific, and prioritized (critical/high/medium/low).

## Common pitfalls

- Clickable `div`/`span` instead of `button`/`a`
- Hidden focus outlines without replacement
- Placeholder used as label
- Incorrect ARIA (`aria-hidden` on focusable nodes, invalid role/state pairings)
- Color-only error/success cues
- Animation that cannot be reduced or disabled
