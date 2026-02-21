# UX Improvement Plan — UI/UX Refactor

## 1) Purpose and scope

This document defines a practical, incremental UX improvement plan for the `ui-ux-refactor` feature.

It translates prior UX findings, visual QA caveats, and architecture constraints into phased execution guidance that can be implemented safely in `ui/` and `sdk-template/ui/`.

**In scope:**
- Focus visibility and keyboard clarity
- Interaction-state clarity and consistency
- Design-token compliance and ad-hoc value reduction
- Visual hierarchy improvements in prioritized surfaces
- Reduced-motion and resize accessibility baseline
- Structural UX improvements that preserve existing architecture boundaries

**Out of scope:**
- DSP/audio behavior changes
- Framework rewrites or large visual rebranding
- Contract/layering changes that introduce new coupling between UI presentational components and transport/business logic

---

## 2) UX principles for this refactor

1. **Clarity first:** Every interactive element should clearly communicate intent and current state.
2. **Keyboard confidence:** Focus order and focus visibility must be deterministic and easy to track.
3. **State coherence:** Hover, active, focus, disabled, selected, and error states follow a shared state model.
4. **Token discipline:** Prefer existing spacing/color/typography/state tokens; avoid ad-hoc visual values unless explicitly justified.
5. **Progressive structure:** Improve hierarchy and readability in-place before introducing larger structural shifts.
6. **Accessibility baseline by default:** Reduced motion, semantics, and keyboard usability are baseline requirements, not optional polish.
7. **Architectural safety:** Preserve smart-vs-presentational layering and avoid coupling regressions while improving UX.
8. **Incremental delivery:** Ship in small, verifiable phases with visible quality gains each phase.

---

## 3) Current UX baseline summary (from findings)

Based on prior UX, visual QA, and architect findings:

- **Focus visibility gaps** exist across important controls, creating keyboard context loss risks.
- **Interaction states are subtle/inconsistent** in places, reducing learnability and confidence.
- **Token drift/ad-hoc values** are present, causing style inconsistency and maintainability drag.
- **Hierarchy can feel flat** in some surfaces, reducing scanability and task orientation.
- **Reduced-motion baseline** exists but should be hardened and consistently validated across changed surfaces.
- **Resize accessibility/ownership** needs clearer UX behavior and boundary ownership to prevent regressions.
- **Visual QA status:** baseline pass with caveats; priority caveats are focus and interaction-state clarity.
- **Architectural guardrail:** preserve component layering and avoid introducing new coupling while refactoring UX.

---

## 4) Improvement backlog by phase

### Phase A: immediate UX/a11y quick wins

**Objective:** remove highest-friction usability issues quickly and safely.

**Backlog items:**
- Standardize visible `:focus-visible` treatment across shared interactive controls.
- Fix keyboard path gaps in critical flows (tab order, Enter/Space activation, Escape close flows where applicable).
- Strengthen interaction-state contrast/readability for core controls (button, toggle, slider, selectable row/card).
- Resolve known visual QA caveats for focus and interaction states in prioritized journeys.
- Add/verify reduced-motion handling for changed transitions and micro-interactions.
- Validate resize affordance/accessibility behavior on targeted responsive/plugin-embedded surfaces.

### Phase B: consistency and component-system improvements

**Objective:** consolidate consistency so future UI work remains predictable.

**Backlog items:**
- Replace ad-hoc visual values with existing design tokens in targeted surfaces.
- Consolidate repeated class patterns into shared component/state variants.
- Define and apply a shared interaction-state matrix for core control families.
- Improve hierarchy through token-aligned typography, spacing, grouping, and emphasis.
- Reduce style leakage by tightening component boundaries and usage patterns.
- Document token and state usage conventions for touched components.

### Phase C: structural UX enhancements

**Objective:** improve long-term UX velocity by reducing architecture-linked friction.

**Backlog items:**
- Refactor selected flows to reinforce smart-vs-presentational boundaries.
- Reduce state fan-out in targeted surfaces to simplify behavior and UX predictability.
- Align IPC-facing naming usage in UI touchpoints with canonical contracts (without widening coupling).
- Clarify and enforce singular resize ownership in impacted integration paths.
- Add concise implementation notes for future contributors to prevent boundary regression.

---

## 5) Acceptance criteria per phase

### Phase A acceptance criteria

- Focus visibility is clear and consistent on all keyboard-focusable controls in prioritized surfaces.
- Keyboard-only navigation is deterministic, with no blocked or ambiguous interaction points.
- Core interaction states are visually distinguishable and no longer subject to current QA caveats.
- Reduced-motion behavior is present for updated motion-heavy interactions.
- No regressions in unaffected baseline UI screenshots for nearby surfaces.

### Phase B acceptance criteria

- Touched surfaces are token-compliant for color, spacing, typography, and interaction states.
- Ad-hoc visual values are removed from refactored areas unless documented with rationale.
- Shared controls implement a common state model with consistent behavior and visuals.
- Hierarchy/readability improvements are visible and consistent across targeted screens.
- Component style leakage is reduced through reusable, containment-friendly patterns.

### Phase C acceptance criteria

- Refactored flows clearly separate orchestration/state logic from presentational rendering.
- Targeted fan-out pain points are simplified with clearer ownership and fewer side-effect touchpoints.
- IPC naming usage in targeted UI areas aligns with canonical contract definitions.
- Resize ownership is singular, explicit, and stable across browser-dev and plugin-host contexts.
- Structural changes preserve layering and do not introduce new coupling regressions.

---

## 6) Visual QA and accessibility verification checklist

- [ ] Before/after screenshots captured for all changed primary journeys.
- [ ] Visual caveats for focus and interaction states are re-checked and closed.
- [ ] Keyboard-only pass completed (tab order, activation keys, escape flows, no traps).
- [ ] Focus indicators remain visible and non-color-only across themes/densities.
- [ ] Semantic controls used (native elements first; ARIA only when necessary).
- [ ] Accessible names verified for changed controls.
- [ ] Contrast checks run for default/hover/focus/disabled/error states.
- [ ] Reduced-motion preference respected on changed transitions/animations.
- [ ] Resize behavior tested for accessibility and usability in targeted layouts.
- [ ] Regression check confirms unaffected nearby UI remains visually stable.

---

## 7) Risks, dependencies, and out-of-scope

### Risks

- **Coupling regression risk:** UX fixes that accidentally entangle presentational components with data/transport logic.
- **Inconsistent rollout risk:** partial updates that improve one control family while leaving adjacent controls inconsistent.
- **Token substitution risk:** replacing ad-hoc values without validating visual intent can cause subtle regressions.
- **Resize behavior risk:** unclear ownership can create jitter, conflicts, or inaccessible resizing behavior.

### Dependencies

- Existing design tokens and Tailwind/theme utility conventions.
- Prioritized UI surfaces and control inventory from `user-stories.md`.
- Visual QA baseline artifacts and caveat list.
- Alignment between UI implementers and architecture constraints for layering and boundaries.

### Out-of-scope (for this plan)

- Non-UI engine/transport rewrites.
- New product features unrelated to UX quality and consistency goals.
- Full-system redesign beyond incremental refactor scope.

---

## 8) Handoff notes for planner/coder/tester

### Planner handoff

- Break each phase into small, independently verifiable tasks by control family and surface.
- Prioritize Phase A items that directly close visual QA caveats (focus + interaction states).
- Include explicit “layering safety” checks for any Phase C structural task.

### Coder handoff

- Implement phase work in minimal-scope PRs with clear before/after behavior notes.
- Reuse existing tokens/components first; document any justified exceptions.
- Preserve smart-vs-presentational boundaries and avoid contract/coupling regressions.
- Include keyboard and reduced-motion checks in local verification for each changed surface.

### Tester handoff

- Validate against the checklist in Section 6 for every phase increment.
- Prioritize focus visibility and interaction-state clarity in primary user journeys.
- Record caveat closure evidence (screenshots + brief notes) and flag any layering-related regressions.
- Confirm resize behavior and accessibility in both browser-dev and plugin-host relevant contexts.
