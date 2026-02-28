# User Stories: UI/UX Refactor

## Overview

This feature spec defines a focused UI/UX refactor for Wavecraft’s plugin UI layer to improve interaction clarity, visual consistency, accessibility, and maintainability without changing core audio behavior.

The refactor is based on completed review findings:

- **Architect findings:** parameter state fan-out, smart-vs-presentational boundaries, IPC string drift, resize ownership split, style leakage
- **UX findings:** focus visibility, interaction consistency, token drift, hierarchy improvements, reduced motion/accessibility baseline
- **Visual QA findings:** baseline visual pass with caveats; prioritize focus and interaction states

---

## Problem Statement

Wavecraft’s current UI experience is functionally complete but exhibits inconsistency and architectural drift that now slows feature development and risks UX regressions:

- Users can lose keyboard context due to inconsistent or weak focus visibility.
- Interaction states (hover/active/disabled/focus) are not consistently expressed across controls.
- Token usage has drifted, causing visual inconsistency and harder theming.
- Some UI state handling is over-coupled (state fan-out, blurred smart/presentational boundaries), reducing maintainability.
- IPC string drift and split resize ownership introduce avoidable integration risk.
- Style leakage makes components less predictable and less reusable.

This creates friction for both plugin users (clarity and confidence) and plugin developers (speed, reliability, extensibility).

---

## Goals

1. **Improve usability and accessibility** of primary UI interactions, with emphasis on visible focus and consistent control states.
2. **Unify visual language** through stronger design-token compliance and clearer hierarchy.
3. **Stabilize frontend architecture** by clarifying smart vs presentational boundaries and reducing parameter state fan-out pain.
4. **Reduce integration drift** by aligning IPC naming/contract usage and establishing clear resize ownership.
5. **Keep performance and behavior stable** while shipping improvements in incremental, testable phases.

## Non-Goals

- Rewriting the full UI framework or replacing React/Tailwind architecture.
- Introducing new DSP/audio features.
- Reworking plugin format support (VST3/CLAP/AU) or transport architecture fundamentals.
- Large-scale visual rebranding unrelated to usability/accessibility and consistency.

---

## User Personas

### Persona 1: Plugin User (Music Producer / Sound Designer)

- Uses the plugin in DAWs (primary: Ableton on macOS) during creative sessions.
- Values speed, clarity, and confidence while tweaking controls under time pressure.
- Needs predictable interactions, visible focus when using keyboard navigation, and readable UI hierarchy.

### Persona 2: Plugin Developer (Wavecraft SDK User)

- Builds and extends plugin UIs with Wavecraft’s React and core packages.
- Values maintainable patterns, clear boundaries, predictable tokens, and stable contracts.
- Needs confidence that new controls follow consistent states and accessibility baseline without ad-hoc fixes.

---

## Prioritized User Stories

### P0 — Story 1: Strong Focus Visibility Across Interactive Controls

**As a** plugin user  
**I want** clear and consistent focus indicators on all keyboard-focusable controls  
**So that** I can navigate and edit parameters confidently without losing context.

#### Acceptance Criteria

- [ ] All keyboard-focusable controls expose a visible focus state meeting baseline contrast expectations in both normal and high-density contexts.
- [ ] Focus styling is consistent across shared control classes/components (not ad-hoc per screen).
- [ ] Focus order is logical and deterministic in primary interaction flows.
- [ ] Visual QA confirms focus visibility improvements for key journeys and records before/after evidence.

---

### P0 — Story 2: Consistent Interaction States for Core Controls

**As a** plugin user  
**I want** controls to behave and look consistent across hover, active, disabled, and focus states  
**So that** interactions feel reliable and learnable.

#### Acceptance Criteria

- [ ] Core control set (buttons, sliders, toggles, selectable rows/cards) uses a shared interaction-state model.
- [ ] State transitions do not conflict (e.g., focus + disabled, active + hover) and are visually unambiguous.
- [ ] Visual QA caveats around interaction states are resolved for the prioritized control set.
- [ ] No regressions in existing baseline screenshot coverage for unaffected areas.

---

### P1 — Story 3: Token Drift Reduction and Style Containment

**As a** plugin developer  
**I want** standardized token usage and reduced style leakage  
**So that** UI changes remain coherent and easier to maintain.

#### Acceptance Criteria

- [ ] Color, spacing, typography, and state styling in refactored surfaces are token-based (no new ad-hoc style constants unless documented exception).
- [ ] Style leakage hotspots identified in review are addressed with containment-friendly patterns.
- [ ] New/updated components document expected token hooks or usage guidance.
- [ ] Refactor avoids introducing component-level visual one-offs that bypass design-token intent.

---

### P1 — Story 4: Clarify Smart vs Presentational Component Boundaries

**As a** plugin developer  
**I want** predictable boundaries between stateful orchestration and presentational rendering  
**So that** components are easier to reason about, test, and evolve.

#### Acceptance Criteria

- [ ] Refactored flows separate data/side-effect orchestration from pure rendering responsibilities.
- [ ] Presentational components can be reused in isolation without hidden state dependencies.
- [ ] State fan-out pain points are reduced in targeted surfaces through clearer data flow ownership.
- [ ] Developer notes include boundary rules for future component additions in this area.

---

### P2 — Story 5: Contract and Ownership Consistency (IPC + Resize)

**As a** plugin developer  
**I want** IPC naming consistency and clear resize ownership  
**So that** integration issues and regressions are reduced.

#### Acceptance Criteria

- [ ] IPC string usage in targeted UI paths aligns with canonical contract naming.
- [ ] Any contract aliasing/compatibility behavior is explicit and documented.
- [ ] Resize ownership in targeted UI/plugin boundary is singular and unambiguous.
- [ ] Integration behavior remains stable in browser-dev mode and plugin host mode.

---

### P2 — Story 6: Accessibility Baseline Including Reduced Motion

**As a** plugin user with accessibility needs  
**I want** the UI to respect reduced-motion preferences and accessible interaction patterns  
**So that** the plugin remains comfortable and usable in longer sessions.

#### Acceptance Criteria

- [ ] Motion-heavy transitions in targeted surfaces provide reduced-motion-friendly behavior.
- [ ] Semantic and keyboard interaction baselines are validated for updated controls.
- [ ] No accessibility regressions are introduced in refactored areas.
- [ ] QA notes explicitly include accessibility baseline checks for changed surfaces.

---

## Phased Scope

### Phase 1 — Foundation

**Objective:** establish stable baseline and remove highest-risk UX friction.

Includes:

- Focus visibility improvements (P0 Story 1)
- Interaction-state consistency for core controls (P0 Story 2)
- Initial token normalization on impacted controls/surfaces
- Visual QA caveat closure for focus and interaction states

### Phase 2 — UX Consistency

**Objective:** scale consistency and maintainability patterns.

Includes:

- Broader token drift reduction and style containment (P1 Story 3)
- Accessibility baseline hardening (including reduced motion) in refactored surfaces (P2 Story 6)
- Extended interaction model consistency across additional components

### Phase 3 — Structural UX

**Objective:** address architectural friction affecting long-term UX velocity.

Includes:

- Smart vs presentational boundary refactors in targeted flows (P1 Story 4)
- Parameter state fan-out simplification in prioritized surfaces
- IPC naming consistency and resize ownership alignment in targeted integration paths (P2 Story 5)

---

## Definition of Done

This feature is considered done when:

- [ ] All Phase 1 acceptance criteria are complete and validated.
- [ ] Priority P0 stories are verified by visual QA with caveats resolved for focus and interaction states.
- [ ] Accessibility baseline checks are executed for changed surfaces, with no critical regressions.
- [ ] Refactored surfaces demonstrate consistent token usage and interaction-state behavior.
- [ ] Structural changes applied in-scope have clear ownership boundaries (state, resize, IPC naming) and documented rationale.
- [ ] No roadmap changes are required as part of this documentation task (roadmap remains untouched).

---

## Notes

- This is a **refactor and quality** initiative: user-visible improvements are expected, but behavioral parity for core plugin functions is maintained.
- Scope should remain incremental: prioritize high user impact (focus/interaction) before deeper structural cleanup.
