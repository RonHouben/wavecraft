# Implementation Plan: Oscillator Passthrough Mix (Milestone 18.11)

## Overview

This plan implements a **targeted bug fix** for Milestone 18.11: enabling `Oscillator` in generated projects must not mute DAW passthrough.  
The fix is scoped to oscillator processing semantics and regression coverage, avoiding architecture changes (no routing graph/multi-bus redesign).  
Implementation follows the in-place serial DSP model and real-time safety constraints.

## Requirements

- With oscillator enabled, incoming DAW audio remains audible.
- Oscillator output and DAW input are audible simultaneously.
- Disabling oscillator preserves passthrough (no forced silence).
- Regression coverage prevents reintroduction.
- Validate behavior in primary target environment: **macOS + Ableton Live**.
- Keep scope to Milestone 18.11 bug fix only (no architecture overreach).

## Scope Boundaries (Milestone 18.11)

### In Scope

- `Oscillator::process()` behavior correction from overwrite/zero-fill to additive/no-op semantics.
- Unit/integration regression tests for passthrough + oscillator coexistence.
- Minimal template/docs clarification where needed.

### Out of Scope

- New mixer processor or multi-bus routing framework.
- Broad DSP architecture refactors.
- Cross-platform DAW parity expansion.

## Impacted Files / Modules

1. `engine/crates/wavecraft-processors/src/oscillator.rs`
   - Primary bug fix and processor-level regression tests.
2. `sdk-template/engine/src/lib.rs`
   - Optional comment clarification about additive generator semantics in default chain.
3. `docs/guides/sdk-getting-started.md` (optional, minimal wording update only if needed)
   - Clarify oscillator is additive (non-replacing) when enabled.
4. **Reference-only invariant (no behavior change expected):**
   - `engine/crates/wavecraft-dsp/src/combinators/chain.rs` (serial in-place chain semantics remain unchanged).

## Implementation Steps

### Phase 1: Baseline + Guardrails

1. **Confirm current oscillator semantics and failing behavior**  
   **File:** `engine/crates/wavecraft-processors/src/oscillator.rs`
   - Action: Verify `!enabled` branch zero-fills buffer and enabled branch overwrites samples.
   - Why: Establish exact bug mechanism before code changes.
   - Dependencies: None
   - Risk: Low

2. **Lock scope to Milestone 18.11**  
   **Files:** feature docs + roadmap context
   - Action: Confirm no architectural work is bundled (e.g., no new routing graph).
   - Why: Prevent overreach; ensure fast, safe bugfix delivery.
   - Dependencies: Step 1
   - Risk: Low

### Phase 2: Core DSP Fix (Primary Change)

3. **Change disabled oscillator behavior to passthrough no-op**  
   **File:** `engine/crates/wavecraft-processors/src/oscillator.rs` (`Oscillator::process`)
   - Action: Replace disabled branch behavior:
     - from: zero-fill each channel
     - to: return without mutating buffer
   - Why: Disabled generator must not mute existing signal.
   - Dependencies: Phase 1
   - Risk: Medium (behavioral change to existing tests)

4. **Change enabled oscillator behavior to additive mix**  
   **File:** `engine/crates/wavecraft-processors/src/oscillator.rs` (`Oscillator::process`)
   - Action: Replace per-sample assignment:
     - from: `*sample = osc_sample * level`
     - to: `*sample += osc_sample * level`
   - Why: Generator must augment input instead of replacing it.
   - Dependencies: Step 3
   - Risk: Medium (possible louder output / clipping at extremes)

5. **Keep chain combinator unchanged; treat as invariant**  
   **File:** `engine/crates/wavecraft-dsp/src/combinators/chain.rs`
   - Action: No behavioral changes.
   - Why: Existing serial in-place model is correct; bug is oscillator semantics.
   - Dependencies: Step 4
   - Risk: Low

### Phase 3: Regression Test Coverage

6. **Update existing disabled behavior test**  
   **File:** `engine/crates/wavecraft-processors/src/oscillator.rs` (tests module)
   - Action: Replace/rename current silence assertion test with passthrough-preservation assertion:
     - prior intent: disabled → silence
     - new intent: disabled → output equals original input
   - Why: Locks in corrected contract.
   - Dependencies: Phase 2
   - Risk: Low

7. **Add additive coexistence test for enabled oscillator + non-zero input**  
   **File:** `engine/crates/wavecraft-processors/src/oscillator.rs` (tests module)
   - Action: Add test case where input buffer starts non-zero and enabled oscillator modifies output additively.
   - Why: Prevent reintroduction of overwrite behavior.
   - Dependencies: Step 6
   - Risk: Low

8. **Add enabled-on-silent-input sanity test**  
   **File:** `engine/crates/wavecraft-processors/src/oscillator.rs` (tests module)
   - Action: Ensure oscillator still produces audible output from zero input.
   - Why: Confirms generator path still functions after additive change.
   - Dependencies: Step 7
   - Risk: Low

9. **(Optional) Add chain-context regression test in processors crate**  
   **File:** `engine/crates/wavecraft-processors/src/oscillator.rs` tests or `engine/crates/wavecraft-processors/tests/*`
   - Action: Validate oscillator behavior when composed serially with another processor.
   - Why: Increase confidence in real chain usage.
   - Dependencies: Step 8
   - Risk: Medium (depends on existing crate test conventions)

### Phase 4: Template/Docs Alignment (Minimal)

10. **Template comment clarification (minimal)**  
    **File:** `sdk-template/engine/src/lib.rs`
    - Action: Add/adjust inline comment to clarify oscillator is additive with passthrough.
    - Why: Avoid first-run confusion for new users.
    - Dependencies: Phase 2
    - Risk: Low

11. **Guide wording touch-up only if needed**  
    **File:** `docs/guides/sdk-getting-started.md`
    - Action: Minimal wording update near oscillator examples to reflect additive semantics.
    - Why: Align docs with delivered behavior, without broad rewrite.
    - Dependencies: Step 10
    - Risk: Low

### Phase 5: Verification + Handoff Readiness

12. **Run focused automated checks**
    - Action:
      - `cargo test -p wavecraft-processors`
      - `cargo xtask ci-check`
    - Why: Validate processor correctness and no broad regressions.
    - Dependencies: Phases 2–4
    - Risk: Low

13. **Manual Ableton smoke (macOS primary target)**
    - Action:
      - Build/install plugin from generated project.
      - Route audible DAW input through plugin.
      - Enable oscillator and confirm simultaneous DAW + oscillator audio.
      - Disable oscillator and confirm passthrough remains.
    - Why: Acceptance in primary host environment.
    - Dependencies: Step 12
    - Risk: Medium (host/runtime variability)

14. **Document results for Tester/QA**  
    **Artifact:** feature `test-plan.md` execution notes
    - Action: Record test evidence and edge-case outcomes.
    - Why: Enables clean Tester/QA handoff.
    - Dependencies: Step 13
    - Risk: Low

## Testing Strategy

### Unit Tests

- **Primary:** `engine/crates/wavecraft-processors/src/oscillator.rs`
  - disabled preserves input passthrough
  - enabled adds to existing input (non-destructive)
  - enabled generates output from silent input
  - existing waveform-output tests remain valid

### Integration Tests

- Processor-chain context test (within processors crate test scope) to confirm behavior in serial flow.
- Full repo regression via `cargo xtask ci-check`.

### Manual Host Smoke (Ableton, macOS)

- Fresh generated plugin scenario:
  1. `wavecraft create ...`
  2. Build/install
  3. Load in Ableton
  4. Verify oscillator enabled does not remove DAW input
  5. Verify passthrough-only path remains stable

## Acceptance Criteria Mapping

| Acceptance Criterion                                                 | Verification                                                |
| -------------------------------------------------------------------- | ----------------------------------------------------------- |
| Oscillator enabled keeps DAW input audible                           | Ableton smoke + additive unit test                          |
| Oscillator audible while DAW input audible                           | Ableton smoke + enabled additive unit test                  |
| Removing/disabling oscillator does not "restore" missing passthrough | disabled passthrough-preservation unit test + Ableton smoke |
| Reproducible in fresh generated project                              | generated-project manual workflow                           |
| No passthrough-only regression                                       | ci-check + disabled passthrough unit test                   |

## Risks & Mitigations

1. **Risk:** Additive mix can increase level/clipping at high settings  
   **Mitigation:** Keep fix scoped to coexistence semantics; validate with gain staging during smoke tests; defer loudness policy changes.

2. **Risk:** Over-fixing into architecture changes (mixer/multi-bus)  
   **Mitigation:** Explicitly no chain architecture modifications in this milestone.

3. **Risk:** Regression hidden by only host-level testing  
   **Mitigation:** Prioritize deterministic unit tests in `oscillator.rs` first, then host smoke.

4. **Risk:** Documentation drift  
   **Mitigation:** Minimal template/guide note aligned with actual behavior.

## Rollback Plan

- Rollback scope is isolated to oscillator semantics and related tests/docs.
- If unexpected regressions appear, revert the oscillator commit(s) only:
  - `engine/crates/wavecraft-processors/src/oscillator.rs`
  - associated oscillator regression tests
  - optional template/docs wording updates
- No protocol/API/schema changes are involved, so rollback is low-risk and straightforward.

## Sequencing for Coder Handoff

1. Implement `Oscillator::process()` fix (disabled no-op, enabled additive).
2. Update/add oscillator regression tests.
3. Add minimal template/docs clarifications.
4. Run focused tests (`wavecraft-processors`, then `cargo xtask ci-check`).
5. Execute Ableton smoke and capture evidence.
6. Hand off to Tester with explicit test checklist and expected outcomes.

## Success Criteria

- [ ] Oscillator no longer mutes DAW passthrough when enabled.
- [ ] Oscillator and DAW input are simultaneously audible.
- [ ] Automated regression tests enforce additive/no-op semantics.
- [ ] `cargo xtask ci-check` passes.
- [ ] Ableton smoke test on macOS passes for fresh generated plugin workflow.
- [ ] Changes remain narrowly scoped to Milestone 18.11 bugfix.
