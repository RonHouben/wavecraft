# User Stories: Runtime-Reorderable SignalChain

## Overview

Wavecraft needs a runtime-reorderable SignalChain so plugin users can change processor order directly in the UI and hear the updated routing immediately. Today, processor order is compile-time fixed via `signal: SignalChain![...]`, which blocks common creative workflows like quickly auditioning processor placement.

This feature introduces a drag-and-drop `SignalChain` UI component where all registered processors are shown as expanded cards, each with bypass controls, and users can reorder them at runtime. The UI becomes the single source of truth for order intent, while the engine applies validated updates at audio block boundaries with a short crossfade to avoid audible artifacts.

### Current State

- Processor order is fixed at compile time through `signal: SignalChain![...]`.
- Runtime reordering is not possible.
- UI cannot authoritatively control processor order.
- There is no keyboard-accessible reorder flow.
- Processor order persistence per preset is not supported as a dedicated runtime order contract.

### Scope

This feature covers:

- Runtime processor reordering from the plugin UI (drag-and-drop + keyboard)
- Engine-side runtime order application at block boundaries with short crossfade
- Persistence of processor order in DAW plugin state (per preset)
- `SignalChain` UI with expanded processor cards and inline bypass toggle visibility
- Macro API shift from compile-time chain field (`signal`) to unordered processor registration (`processors`)

This feature does **not** cover:

- Dynamic add/remove of processors at runtime
- Non-serial routing (parallel/split/merge graph editing)
- Per-card collapse/expand state persistence

---

## User Story 1: Reorder Processors by Drag-and-Drop

**As an** end user of a Wavecraft-based plugin
**I want** to drag and drop processors in the SignalChain UI
**So that** I can quickly audition different processing orders while designing sound

### Acceptance Criteria

- [ ] The SignalChain UI renders all registered processors as draggable cards in order
- [ ] Dragging a processor to a new position updates the chain order in the UI immediately
- [ ] The engine applies the new order without requiring plugin reload or DAW restart
- [ ] Reordering updates are propagated through the runtime IPC contract and acknowledged
- [ ] If reorder validation fails, the UI shows a clear error and keeps a valid previous order

### Notes

- The UI is authoritative for user reorder intent; the engine validates and applies
- Reorder payload must always be a full valid permutation of all registered processor IDs

---

## User Story 2: Keyboard-Accessible Reordering

**As an** end user who relies on keyboard navigation
**I want** to reorder processors with keyboard controls
**So that** SignalChain editing is accessible without a pointer device

### Acceptance Criteria

- [ ] Focusable drag handles exist for each processor card
- [ ] `Space`/`Enter` picks up and drops the focused processor
- [ ] Arrow keys move the active processor up/down in the list
- [ ] `Escape` cancels the active reorder operation and restores prior order
- [ ] Reorder actions provide ARIA announcements/instructions for assistive technologies

### Notes

- Keyboard interaction must match documented SignalChain interaction behavior
- Pointer and keyboard flows should produce identical persisted order results

---

## User Story 3: Artifact-Free Runtime Application

**As an** end user
**I want** processor reorder changes to be applied without clicks or pops
**So that** I can reorder during playback without breaking the listening experience

### Acceptance Criteria

- [ ] New order is applied only at audio block boundaries
- [ ] A short crossfade is used on reorder transitions to prevent audible clicks
- [ ] Audio thread remains real-time safe (no allocations, locks, or blocking)
- [ ] Reorder logic does not regress CPU behavior beyond expected bounded overhead
- [ ] Repeated rapid reorder operations remain stable during sustained playback

### Notes

- Implementation follows Wavecraft real-time safety constraints
- Transition behavior should feel immediate while avoiding zipper/click artifacts

---

## User Story 4: Order Persistence Per Preset

**As an** end user
**I want** processor order to persist with plugin state
**So that** my signal chain layout survives DAW save/load and preset recall

### Acceptance Criteria

- [ ] Processor order is serialized into plugin state per preset
- [ ] Reopening a DAW project restores the exact saved processor order
- [ ] Loading a preset restores its associated processor order
- [ ] First-run/default state falls back to registration order when no persisted order exists
- [ ] Invalid persisted order data is rejected safely with fallback to a valid default order

### Notes

- Persistence must be deterministic and resilient to malformed state payloads

---

## User Story 5: SDK API Uses Unordered Processor Registration

**As an** SDK developer
**I want** plugin declaration to register processors via `processors: [...]` instead of compile-time `signal: SignalChain![...]`
**So that** runtime UI can own and control processor order

### Acceptance Criteria

- [ ] `wavecraft_plugin!` supports `processors: [...]` as the canonical field
- [ ] `signal: SignalChain![...]` is removed for this feature scope
- [ ] Registration order provides initial fallback order only
- [ ] Existing processor parameter contracts remain stable and order-independent
- [ ] Documentation and templates reflect the new plugin declaration pattern

### Notes

- This is a pre-1.0 breaking change and is acceptable under current product policy
- Compile-time order is intentionally replaced by runtime order ownership

---

## User Story 6: Stable Processor Identity Across Reordering

**As an** SDK developer
**I want** processor instances and IDs to remain stable when reordered
**So that** stateful processors and parameter mappings continue to work correctly

### Acceptance Criteria

- [ ] Reordering does not recreate processor instances unnecessarily
- [ ] Processor IDs remain stable and continue to identify the same processor instance
- [ ] Parameter IDs and bypass IDs remain unchanged after reorder
- [ ] UI and engine use the same canonical processor ID set for validation
- [ ] Unknown/duplicate/missing IDs in reorder payloads are rejected with explicit errors

### Notes

- Stability protects filter history/envelopes and other internal processor state
- ID stability is required for reliable UI-state and automation mapping

---

## User Story 7: Bypass Control Visible in SignalChain Cards

**As an** end user
**I want** bypass toggles visible on each processor card in the SignalChain
**So that** I can compare effect combinations quickly while I reorder processors

### Acceptance Criteria

- [ ] Each processor card in SignalChain shows an accessible bypass control
- [ ] Bypass state changes are reflected immediately in card visuals and processing behavior
- [ ] Bypass controls remain usable before, during, and after reorder operations
- [ ] Bypass state continues to persist correctly with plugin state
- [ ] DAW automation updates bypass state in UI correctly for each processor card

### Notes

- Bypass visibility is part of the default SignalChain card presentation
- Reorder capability must not regress existing bypass behavior

---

## Dependencies

- Runtime processor-order IPC contract (`getProcessorOrder`, `setProcessorOrder`, `processorOrderChanged`)
- Engine runtime order application path with permutation validation
- Plugin state persistence for processor order
- UI SignalChain component and processor component registry
- Macro API support for `processors: [...]` registration

## Risks

| Risk                                                            | Likelihood | Impact | Mitigation                                                                  |
| --------------------------------------------------------------- | ---------- | ------ | --------------------------------------------------------------------------- |
| Audible artifact during reorder under playback load             | Medium     | High   | Apply reorder at block boundaries + short crossfade + stress testing        |
| Invalid order payloads desync UI and engine                     | Medium     | High   | Strict full-permutation validation + explicit error contract                |
| Accessibility regressions in drag-and-drop interactions         | Medium     | Medium | Keyboard-first acceptance criteria + ARIA instruction/announcement coverage |
| Breaking change confusion for SDK developers (`signal` removal) | Medium     | Medium | Clear migration notes in docs and template examples                         |

## Related Documents

- [Low-Level Design: Runtime-Reorderable SignalChain](./low-level-design-ui-signal-chain-reorder.md)
- [UI Design Specification: SignalChain](./ui-design-signal-chain.md)
- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards](../../architecture/coding-standards.md)
- [Roadmap](../../roadmap.md)
