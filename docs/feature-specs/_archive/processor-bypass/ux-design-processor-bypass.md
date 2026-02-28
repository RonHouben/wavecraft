# UX Design Proposal: Processor Bypass

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Project conventions
- [Roadmap](../../roadmap.md) — Milestones and progress
- [SDK Architecture](../../architecture/sdk-architecture.md) — SDK/package boundaries
- [Development Workflows](../../architecture/development-workflows.md) — Dev/build/test flow
- [Plugin Formats](../../architecture/plugin-formats.md) — VST3/CLAP/AU integration details
- [User Stories: Processor Bypass](./user-stories.md) — Product requirements
- [Low-Level Design: Processor Bypass](./low-level-design-processor-bypass.md) — Runtime/API constraints

---

## Overview

This proposal defines a **non-prescriptive UI/UX contract** for exposing per-processor bypass controls in Wavecraft UIs.

The intent is to let plugin developers choose their own visuals (toggle, pill, switch, icon button, etc.) while maintaining consistent behavior for:

- parameter semantics (`{processor_id}_bypass`)
- interaction states and feedback
- accessibility and keyboard support
- per-instance independence in processor chains

The host/runtime contract remains parameter-driven (no new processor-level IPC methods).

---

## UX Goals and Non-Goals

### Goals

1. Make bypass obvious and fast for A/B listening.
2. Preserve per-instance clarity in chains with repeated processor types.
3. Keep behavior predictable under automation and async updates.
4. Enable design freedom while standardizing interaction + accessibility contracts.

### Non-Goals

- Prescribing exact visual style, component library, or layout skin.
- Introducing new transport/IPC methods.
- Defining DSP transition internals (covered by low-level design).

---

## Interaction Model

## Control semantics

- **Parameter ID**: `${processorId}_bypass`
- **State meaning**:
  - `false` (or numeric `< 0.5`) = processor **active**
  - `true` (or numeric `>= 0.5`) = processor **bypassed**

The UI treats bypass as a boolean control with optimistic but safe feedback.

## Recommended interaction behavior

- Pointer/click toggles state immediately.
- Keyboard activation uses **Space** and **Enter** on focused control.
- State changes should be reflected visually within one frame after local update.
- If a host/automation update arrives after local interaction, latest source-of-truth value wins and UI reconciles.

## State model (behavioral, not visual)

Each bypass control should support the following states:

1. **Active** (processor enabled)
2. **Bypassed**
3. **Pending/Loading** (outbound update in flight, optional if transport is near-instant)
4. **Disabled** (processor unavailable, no engine connection, or explicit UI lock)
5. **Error** (update rejected/time-out/bridge failure)

### State transitions

- `Active -> Pending -> Bypassed`
- `Bypassed -> Pending -> Active`
- `Pending -> Error` on failed write
- `Error -> Pending -> Active|Bypassed` on retry
- `Any -> Disabled` when control becomes unavailable

### Error recovery guidance

- Keep last known state visible.
- Show concise inline error affordance (icon/text/tooltip) tied to the control.
- Provide retry via re-toggle or dedicated retry action.

## Focus and keyboard behavior

- Bypass control must be focusable in normal tab order.
- Focus ring must remain visible in all visual themes.
- Avoid keyboard traps in chain rows/cards.
- If disabled, control should be skipped by tab order only when semantically disabled (`disabled`).

---

## Information Architecture for Per-Instance Chain Controls

The IA should be organized around **processor instances**, not processor types.

## Proposed hierarchy

1. **Signal Chain Region** (`role="region"` with label)
2. **Processor Instance Item** (one row/card per instance)
3. **Primary actions per instance**
   - bypass toggle
   - processor-local controls (existing params)
4. **Optional metadata**
   - instance display name (e.g., "Input Trim")
   - processor type tag (e.g., "Gain")

## Per-instance identity requirements

To satisfy independence (US6), each UI item should carry:

- stable `processorId` (instance ID)
- derived `bypassParameterId` = `${processorId}_bypass`
- distinct accessible label using instance display name

Even if two processors share the same type, they must render as separate instance entries with separate bypass controls and labels.

## Suggested chain ordering

- Match runtime signal path order top-to-bottom or left-to-right.
- Keep bypass control in a consistent location per instance for scanability.
- Prefer grouping bypass with processor title to strengthen identity mapping.

---

## Accessibility Considerations

## Semantics and ARIA

- Prefer native `<button>` with toggle semantics or equivalent accessible control.
- If using a button-like toggle, expose pressed state via `aria-pressed`.
- Accessible name should include processor instance name, e.g.:
  - `aria-label="Bypass Input Trim"`
- Optional helper text can describe current state for screen readers.

## Focus order

- Focus should follow chain order.
- Within a processor item, bypass should appear before secondary controls when it is the primary action.
- Do not reorder visually and semantically in conflicting ways.

## Contrast and non-color cues

- Active/bypassed/disabled/error states must not rely on color alone.
- Add iconography, text labels, and/or shape/weight differences.
- Maintain WCAG-appropriate contrast for label, control boundary, and focus indicator.

## Motion guidance

- If animating toggle transitions, keep duration short and subtle.
- Respect `prefers-reduced-motion` by reducing or removing non-essential animation.
- Avoid motion that obscures whether processor is active vs bypassed.

## Assistive feedback for async/error states

- For async errors, provide polite live announcement only when state changes to error.
- Keep announcements concise and actionable (e.g., "Could not update bypass for Input Trim. Try again.").

---

## Design Token & Reuse Guidance

This section is intentionally token-oriented and avoids hard-coded visuals.

## Token usage principles

- Use existing semantic tokens for:
  - surface/background
  - text
  - border/divider
  - focus ring
  - status (disabled/error)
- Reuse existing spacing and typography scales; avoid arbitrary pixel values.
- Keep state styling consistent with other boolean/toggle controls in the plugin UI.

## Reuse recommendations

- Prefer a shared `ToggleableParameterControl`/`BypassControl` pattern over bespoke one-offs.
- Centralize state class recipes (default/hover/focus/active/disabled/error).
- Reuse processor list item/container primitives where available.
- Keep processor title + bypass pairing structurally consistent across all chain items.

## Avoid

- Ad-hoc one-off spacing/color values when tokens already exist.
- Different bypass interaction patterns in different processor rows without clear product reason.
- Hidden focus styles or color-only selected/bypassed cues.

---

## Non-Prescriptive Component API Contracts (Future Implementation)

The following contracts define behavior, not appearance.

## Core control contract

```ts
export interface ProcessorBypassControlState {
  processorId: string;
  bypassParameterId: string; // `${processorId}_bypass`
  bypassed: boolean;
  disabled?: boolean;
  loading?: boolean;
  error?: string | null;
}

export interface ProcessorBypassControlActions {
  onToggle(nextBypassed: boolean): Promise<void> | void;
  onRetry?(): Promise<void> | void;
}

export interface ProcessorBypassControlA11y {
  ariaLabel?: string; // default: `Bypass ${processorDisplayName}`
  ariaDescribedBy?: string;
}

export type ProcessorBypassControlProps = ProcessorBypassControlState &
  ProcessorBypassControlActions &
  ProcessorBypassControlA11y & {
    processorDisplayName: string;
    processorTypeName?: string;
    // Visual freedom: render prop or slot can be added without changing behavior contract
  };
```

## Optional hook-level contract

```ts
export interface UseProcessorBypassResult {
  processorId: string;
  bypassParameterId: string;
  bypassed: boolean;
  loading: boolean;
  error: string | null;
  setBypassed(nextBypassed: boolean): Promise<void>;
  toggle(): Promise<void>;
  clearError(): void;
}
```

## Chain item composition contract

```ts
export interface ProcessorChainItemViewModel {
  processorId: string;
  processorDisplayName: string;
  processorTypeName: string;
  orderIndex: number;
  bypass: {
    parameterId: string;
    bypassed: boolean;
    loading: boolean;
    disabled: boolean;
    error: string | null;
  };
}
```

These contracts permit any UI style while ensuring functional consistency.

---

## Acceptance Mapping to User Stories

## Priority mapping (US5, US6)

### US5 — UI Bypass Control via IPC

- Uses existing parameter channels (`getParameter`, `setParameter`, `getAllParameters`), no new method assumptions.
- Bypass control model explicitly binds to `${processorId}_bypass`.
- Contracts support immediate UI feedback and reconciliation with automation updates.
- Async/error states account for bridge or write failures while preserving predictable UI behavior.

### US6 — Per-Instance Bypass Independence

- IA is instance-centric, not type-centric.
- Every chain item carries a unique `processorId` and derived unique `bypassParameterId`.
- Accessible naming requires instance display labels, preventing ambiguity between repeated processor types.

## Additional story alignment

- **US1:** Standardized bypass parameter discoverability and naming expectations reflected in contracts.
- **US2:** UX supports real-time toggling and A/B listening without prescribing DSP internals.
- **US3/US4:** Host-driven persistence/automation assumptions are respected via source-of-truth reconciliation behavior.

---

## Implementation Notes for Future UI Work

1. Start with hook-driven state (`useParameter` or a dedicated wrapper hook).
2. Build one reusable bypass control primitive with strict accessibility semantics.
3. Integrate control into processor chain row/card primitive.
4. Add keyboard and screen-reader regression tests for active/bypassed/disabled/error states.
5. Validate behavior under DAW automation bursts to ensure stable reconciliation.

---

## Open UX Questions

1. Should bypass controls be always visible or progressively disclosed in compact layouts?
2. Should error messages be inline, tooltip-based, or summarized in a status region?
3. For dense chains, should multi-select bypass operations exist as a future enhancement?

(These are follow-ups and not blockers for initial implementation.)
