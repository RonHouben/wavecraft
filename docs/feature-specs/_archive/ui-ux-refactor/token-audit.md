# Token Audit — ui-ux-refactor (Phase 2.1)

## Scope

- `ui/packages/components/src/**`
- `sdk-template/ui/src/**`

## Findings

### Blocking (fixed in this phase)

1. `ui/packages/components/src/Meter.tsx`
   - `bg-[#333]` used for meter track background (L/R)
   - Replaced with `bg-plugin-surface`

2. `ui/packages/components/src/Meter.tsx`
   - Non-reduced-motion width/shadow transitions (`transition-*` without `motion-safe:`)
   - Updated to `motion-safe:transition-*` + `motion-safe:duration-*`

### Non-blocking

- Inline `style={{ width: ... }}` in `Meter.tsx` is retained intentionally for dynamic bar-width percentages and is not a token violation.

## Summary

- Ad-hoc color token drift in touched UX surfaces has been removed.
- Transition behavior now respects reduced-motion in updated meter interactions.
