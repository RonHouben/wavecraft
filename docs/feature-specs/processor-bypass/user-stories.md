# User Stories: Processor Bypass

## Overview

Plugin developers need the ability to bypass (disable) individual processors in the signal chain from the UI. When bypassed, a processor passes audio through unmodified (dry passthrough). This enables end users to A/B compare the effect of each processor independently without removing it from the chain.

Bypass is implemented as a **parameter-level feature** — each processor instance automatically gets a bypass parameter injected by the framework. This aligns with the existing compile-time `SignalChain!` architecture and avoids dynamic graph mutation. Bypass state is per-instance, persisted with DAW sessions, and automatable from the host.

### Current State

- The signal chain is compile-time composed via the `SignalChain!` macro (serial `Chain<A, B>` composition).
- No generic bypass mechanism exists across processors.
- The `Oscillator` processor has a manual `enabled` parameter, but this is hand-rolled and not a framework-level pattern.
- A `BypassStage` exists in the SDK template as an explicit `Passthrough` processor in the chain — this is a static no-op stage, not a toggleable bypass.
- The UI knows about processor presence via codegen (`useHasProcessor`, `useAvailableProcessors`) but has no concept of processor bypass state.
- IPC methods cover parameters, meters, and audio status — no processor-level control methods exist.

### Scope

This feature covers:

- Framework-level bypass parameter injection for all processors
- Dry passthrough behavior when bypassed
- IPC support for reading/writing bypass state
- UI hooks/API for bypass control
- DAW automation and session persistence of bypass state

This feature does **not** cover:

- Specific UI component design for bypass controls (left to architect/designer)
- Wet/dry mix or partial bypass
- Dynamic signal chain reordering or processor removal at runtime

---

## User Story 1: Automatic Bypass Parameter per Processor

**As a** plugin developer using the Wavecraft SDK
**I want** each processor in my signal chain to automatically have a bypass parameter
**So that** I don't have to manually implement bypass logic for every processor I create

### Acceptance Criteria

- [ ] Every processor in a `SignalChain!` declaration automatically gets a bypass parameter without developer intervention
- [ ] The bypass parameter is a boolean (on/off) — `true` = bypassed, `false` = active
- [ ] The bypass parameter follows the existing parameter naming convention (e.g., `{processor_id}:bypass`)
- [ ] The bypass parameter is discoverable via `getAllParameters` like any other parameter
- [ ] Existing processor code (custom `Processor` trait implementations) continues to work without modification
- [ ] The SDK template demonstrates bypass usage out of the box

### Notes

- This is a framework-level concern — individual processor authors should not need to implement bypass logic
- The bypass parameter is injected by the `SignalChain!` macro or a wrapping combinator, not by the processor itself
- The existing `Oscillator.enabled` parameter is a separate concern and should remain as-is (processor-specific enable/disable logic)

---

## User Story 2: Dry Passthrough When Bypassed

**As an** end user of a Wavecraft-based plugin
**I want** a bypassed processor to pass audio through unmodified
**So that** I can hear the signal without that processor's effect and A/B compare

### Acceptance Criteria

- [ ] When a processor is bypassed, its `process()` method is **not called**
- [ ] Audio input is passed directly to the output unmodified (dry passthrough)
- [ ] Bypassing a processor introduces no audible artifacts (no clicks, pops, or volume changes)
- [ ] Bypassing/unbypassing works correctly mid-stream (while audio is playing)
- [ ] Zero additional CPU cost for a bypassed processor (its DSP is skipped, not computed and discarded)
- [ ] Bypass works correctly for all built-in processors (Gain, Filter, Saturator, Passthrough, Oscillator, OscilloscopeTap)

### Notes

- The bypass check must be real-time safe (no allocations or locks) — reading an atomic boolean is sufficient
- Consider whether `OscilloscopeTap` bypass should also stop sending oscilloscope data or only skip audio processing
- Passthrough processor bypass is a no-op (bypassing a passthrough still passes audio through) — this is expected and correct

---

## User Story 3: Bypass State Persistence

**As an** end user of a Wavecraft-based plugin
**I want** the bypass state of each processor to be saved and restored with my DAW session
**So that** my bypass settings are preserved when I reopen a project

### Acceptance Criteria

- [ ] Bypass state is included in the plugin's saved state (DAW session save)
- [ ] Bypass state is correctly restored when reopening a DAW session
- [ ] Bypass state is preserved across DAW undo/redo operations
- [ ] Default bypass state is `false` (processor active) for new instances

### Notes

- Since bypass is implemented as a standard parameter, persistence should be handled automatically by the nih-plug parameter state system
- No additional serialization work should be needed if the parameter is correctly registered with the host

---

## User Story 4: DAW Automation of Bypass

**As an** end user of a Wavecraft-based plugin
**I want** to automate the bypass state of each processor from my DAW
**So that** I can enable/disable processors at specific points in my arrangement

### Acceptance Criteria

- [ ] Bypass parameters are exposed to the DAW as automatable parameters
- [ ] Bypass automation is sample-accurate (or as accurate as the host allows for boolean params)
- [ ] Bypass automation is visible in the DAW's automation lanes alongside other parameters
- [ ] Writing automation for bypass works via the standard DAW automation recording workflow
- [ ] Bypass parameter names are clear and distinguishable in the DAW's parameter list (e.g., "InputTrim Bypass", "ExampleProcessor Bypass")

### Notes

- Boolean parameters in nih-plug are typically displayed as automation-compatible on/off values
- The parameter name shown in the DAW should include the processor's display name for clarity, since multiple processors in the chain will each have a bypass parameter
- Verify behavior in Ableton Live (primary target)

---

## User Story 5: UI Bypass Control via IPC

**As a** plugin developer using the Wavecraft SDK
**I want** to read and toggle the bypass state of each processor from the React UI
**So that** I can build custom bypass controls in my plugin's interface

### Acceptance Criteria

- [ ] The bypass parameter is accessible via the existing `setParameter` / `getParameter` IPC methods (no new IPC methods needed)
- [ ] The bypass state is included in `getAllParameters` responses
- [ ] The existing `useParameter` hook works with bypass parameters (e.g., `useParameter('{processor_id}:bypass')`)
- [ ] Toggling bypass via the UI provides immediate visual and audible feedback (no perceptible delay)
- [ ] Bypass state updates from DAW automation are reflected in the UI in real-time

### Notes

- Since bypass is a standard parameter, all existing parameter infrastructure (IPC, hooks, state management) should work without modification
- The UI framework should provide the processor-bypass parameter IDs via codegen so developers can reference them type-safely
- Specific bypass UI component design is out of scope for this story — developers use existing parameter hooks and build their own controls

---

## User Story 6: Per-Instance Bypass Independence

**As a** plugin developer using the Wavecraft SDK
**I want** each processor instance in the signal chain to have its own independent bypass state
**So that** bypassing one processor does not affect any other processor in the chain

### Acceptance Criteria

- [ ] If a chain contains multiple processors of the same type (e.g., two `Gain` processors), each has an independent bypass parameter
- [ ] Bypassing processor A does not affect the bypass state of processor B
- [ ] Bypass parameters are uniquely identified by processor instance ID, not processor type
- [ ] The parameter list clearly distinguishes bypass parameters for different instances (e.g., "InputTrim Bypass" vs "OutputTrim Bypass")

### Notes

- This is naturally handled by the existing per-instance `wavecraft_processor!` naming (each instance has a unique ID like `input_trim`, `output_trim`)
- The bypass parameter ID should incorporate the processor instance ID to guarantee uniqueness

---

## Dependencies

- **Existing architecture**: `SignalChain!` macro, `Processor` trait, `Chain<A, B>` combinator, parameter system
- **Existing IPC**: `setParameter`, `getParameter`, `getAllParameters` — no new methods needed
- **Existing UI**: `useParameter` hook, codegen for processor IDs

## Risks

| Risk                                                                       | Likelihood | Impact | Mitigation                                                                               |
| -------------------------------------------------------------------------- | ---------- | ------ | ---------------------------------------------------------------------------------------- |
| Click/pop artifacts on bypass toggle                                       | Medium     | High   | Use crossfade or zero-crossing detection for bypass transitions                          |
| Parameter count explosion with many processors                             | Low        | Medium | Bypass params are lightweight booleans; DAWs handle hundreds of params                   |
| Bypass of metering/analysis processors (OscilloscopeTap) may confuse users | Low        | Low    | Document expected behavior; consider whether analysis processors should be bypass-exempt |
