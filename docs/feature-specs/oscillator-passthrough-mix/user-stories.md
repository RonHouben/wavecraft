# Archived

This feature spec has been archived.

Canonical file: `docs/feature-specs/_archive/oscillator-passthrough-mix/user-stories.md`

Please read/update the archived file only.

**User problem:**

- A developer/scenario expects a basic synth+input layering behavior in the default chain setup
- In Ableton, enabling `Oscillator` causes incoming track audio to disappear
- Removing `Oscillator` restores passthrough, confirming a chain behavior bug

**Expected product behavior:**

- Oscillator signal and DAW input are both audible simultaneously
- Oscillator must augment the signal path, not replace/block it
- Newly scaffolded projects should behave correctly without requiring users to redesign DSP routing

---

## User Story 1: Hear DAW Input While Oscillator Is Enabled

**As a** plugin developer testing a newly scaffolded Wavecraft plugin in Ableton  
**I want** DAW input to continue passing through when `Oscillator` is in the `SignalChain`  
**So that** I can test effects and layering workflows without broken audio routing

### Acceptance Criteria

- [ ] With `Oscillator` enabled in the chain, incoming DAW audio remains audible
- [ ] Removing `Oscillator` does not change passthrough presence (only tonal mix content)
- [ ] Behavior is reproducible in a fresh `wavecraft create` project
- [ ] No silence/dropout regression is introduced in normal passthrough-only setups

### Notes

- This is a correctness bug in signal flow behavior, not a feature request for new synthesis capabilities
- Validation should be done in the primary target environment: macOS + Ableton Live

---

## User Story 2: Hear Oscillator and DAW Input Simultaneously

**As a** sound designer prototyping tones over external audio  
**I want** oscillator output to mix with incoming DAW signal  
**So that** both sources are audible at the same time

### Acceptance Criteria

- [ ] Oscillator output is audible when enabled
- [ ] DAW input is simultaneously audible when oscillator is enabled
- [ ] Combined level is perceptually consistent with expected mix behavior (no hard replacement)
- [ ] Existing gain controls still affect output predictably

### Notes

- Implementation detail (e.g., additive mix vs configurable dry/wet) is an architecture decision for Architect/Coder
- PO acceptance is behavior-oriented: no source should be unintentionally muted by enabling oscillator

---

## User Story 3: New Users Don’t Hit a “Why Is My Input Gone?” Trap

**As a** first-time Wavecraft user  
**I want** template signal-chain behavior to be intuitive and non-destructive  
**So that** my first DAW test build inspires confidence instead of confusion

### Acceptance Criteria

- [ ] Default/new-project workflow docs and examples do not imply mutually exclusive audio paths when oscillator is enabled
- [ ] If relevant, template comments/config defaults clarify intended layering behavior
- [ ] Test plan includes explicit regression case for “oscillator enabled + DAW input present”

### Notes

- This is high user-impact because it affects first impressions of generated projects
- If architectural constraints require explicit mixer processor, story remains valid but should be delivered with clear defaults

---

## Scope and Constraints

### In Scope

- Fix signal-flow behavior where oscillator blocks passthrough in generated projects
- Ensure oscillator + DAW input are both audible in primary dev target (macOS + Ableton)
- Add regression coverage for this behavior

### Out of Scope

- Advanced synth engine redesign
- Full routing matrix/editor UX
- Cross-platform host parity beyond current primary target

---

## Risks and Product Considerations

| Risk                                               | Impact                          | Mitigation                                                |
| -------------------------------------------------- | ------------------------------- | --------------------------------------------------------- |
| Fix is done host-specific instead of chain-correct | Regression in other hosts/modes | Validate at signal-chain logic level + Ableton smoke test |
| Level summing causes clipping                      | Poor perceived quality          | Include metering/clipping check in test plan              |
| Quick patch breaks template simplicity             | DX regression                   | Prefer minimal, understandable default behavior           |

---

## Priority Recommendation

**Priority:** High

**Rationale:**

- **User Impact:** High — this breaks core expected audio behavior in first-run projects
- **Strategic Fit:** High — directly affects Wavecraft’s “simple and professional” promise
- **Risk:** Medium — touches audio path logic and needs careful regression validation
- **Dependencies:** Should proceed through Architect next for signal-path decision and acceptance-test framing

---

## Suggested Handoff

1. **Architect**: define canonical signal-path behavior when generator and input coexist
2. **Planner**: produce implementation plan across template/engine/runtime layers as needed
3. **Coder**: implement minimal fix + regression tests
4. **Tester/QA**: verify in Ableton and automated checks before closure
