# Archived

This feature spec has been archived.

Canonical file: `docs/feature-specs/_archive/oscillator-passthrough-mix/low-level-design-oscillator-passthrough-mix.md`

Please read/update the archived file only.

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview, DSP/UI boundaries
- [Coding Standards](../../architecture/coding-standards.md) — Repository conventions
- [Roadmap](../../roadmap.md) — Milestone 18.11 context and tracking
- [SDK Architecture](../../architecture/sdk-architecture.md) — Crate/package boundaries
- [Development Workflows](../../architecture/development-workflows.md) — Build/test/dev workflows
- [Plugin Formats](../../architecture/plugin-formats.md) — Host/runtime constraints

---

## Problem Statement

In fresh generated projects, adding `Oscillator` to `SignalChain![]` can mute DAW passthrough in Ableton instead of layering oscillator + input.

Current behavior is caused by generator overwrite semantics in `Oscillator::process()`:

- when disabled: fills channels with `0.0` (hard mute)
- when enabled: writes oscillator samples into `*sample` (replaces input)

Because the DSP chain is in-place serial processing, replacement at any stage destroys upstream signal.

---

## Root Cause and Context

### Confirmed technical behavior

- `sdk-template/engine/src/lib.rs`: default chain includes `Oscillator` first in serial chain.
- `engine/crates/wavecraft-dsp/src/combinators/chain.rs`: `Chain::process()` is strictly in-place serial (`first` then `second`) on one shared buffer.
- `engine/crates/wavecraft-processors/src/oscillator.rs`:
  - `!enabled` path writes silence (`channel.fill(0.0)`)
  - enabled path sets `*sample = oscillator_sample * level`
- Therefore, oscillator currently acts as a **source replacement**, not **source augmentation**.

### Architectural constraint

Wavecraft DSP processors are in-place and real-time safe. No additional routing graph currently exists in template runtime path; behavior must be corrected within this model.

---

## Canonical Signal-Flow Semantics (Normative)

For Wavecraft serial chains, processor categories are defined as:

1. **Transform processors** (effects): modify existing buffer content.
2. **Generator processors** (sources): add synthetic content to existing buffer.
3. **Tap/observer processors**: observe only; must not modify buffer.

### Canonical contract for generator + passthrough coexistence

Given input sample $x[n]$, oscillator sample $o[n]$, oscillator level $g \in [0,1]$:

$$
y[n] = x[n] + g \cdot o[n]
$$

- If generator is disabled:
  $$
  y[n] = x[n]
  $$
  (strict passthrough; no forced silence)
- Generator MUST NOT overwrite buffer by default.
- Generator MUST preserve channel count and in-place processing semantics.
- Any clipping behavior remains downstream concern (e.g., gain/output stage); this fix is about **non-destructive coexistence**, not loudness normalization policy.

---

## Impacted Modules / Layers

| Layer                                    | Module(s)                                              | Impact                                                                                                  |
| ---------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------- |
| Template plugin graph                    | `sdk-template/engine/src/lib.rs`                       | No structural change required for bugfix; may add clarifying comment about additive generator semantics |
| DSP processor implementation             | `engine/crates/wavecraft-processors/src/oscillator.rs` | Primary fix location: replace overwrite/zero-fill behavior with additive/no-op behavior                 |
| DSP composition semantics                | `engine/crates/wavecraft-dsp/src/combinators/chain.rs` | No behavior change; referenced as invariant (in-place serial)                                           |
| Runtime plugin path (nih-plug codegen)   | `engine/crates/wavecraft-macros/src/plugin.rs`         | No direct change for this bug; confirms in-place buffer path and real-time constraints                  |
| Docs/guidance (optional but recommended) | `docs/guides/sdk-getting-started.md`                   | Clarify generator semantics to avoid future confusion                                                   |

---

## Implementation Options

### Option A — Fix Oscillator semantics in-place (**Recommended**)

**Change:**  
`Oscillator::process()` becomes additive generator:

- disabled: return without mutating buffer
- enabled: `*sample += osc_sample * level`

**Pros**

- Minimal surface area and low regression risk
- Preserves existing chain architecture
- Aligns with user expectations immediately
- Real-time safe (no allocations/locks)

**Cons**

- Clipping may be more likely at high levels (already true in additive systems)

---

### Option B — Add explicit Mixer processor in template and route oscillator separately

**Change:** introduce dedicated source bus + mixer stage.

**Pros**

- Cleaner conceptual model for multiple sources

**Cons**

- Larger architectural change for a high-priority bugfix
- Expands template complexity
- Requires new routing semantics beyond current serial model

---

### Option C — Introduce multi-bus graph semantics framework

**Pros**

- Future-proof for complex routing/modulation

**Cons**

- Significant scope increase
- Not justified for current bug milestone

---

## Recommended Direction

Adopt **Option A** now:

1. Make oscillator additive/non-destructive.
2. Keep serial in-place chain model unchanged.
3. Add regression tests that lock in coexistence semantics.
4. Optionally add a short docs note for "generator processors are additive by contract."

This resolves Milestone 18.11 without architectural churn.

---

## Real-Time Safety Constraints

Any implementation MUST preserve:

- no allocations in `process()`
- no locks / blocking calls
- no logging/syscalls in audio callback
- deterministic per-sample operation
- no additional temporary heap buffers

Option A satisfies all constraints.

---

## Acceptance-Test Framing

### 1) DSP unit tests (`wavecraft-processors`)

Add/replace tests in `oscillator.rs`:

- `oscillator_disabled_preserves_input_passthrough`
  - input buffer initialized non-zero
  - disabled oscillator
  - output equals input (within epsilon)
- `oscillator_enabled_adds_signal_without_removing_input`
  - input buffer non-zero constant
  - enabled oscillator
  - output differs from both pure input and pure oscillator-only overwrite pattern
- `oscillator_zero_input_still_generates_when_enabled`
  - ensures generator path still audible

### 2) Chain-level integration framing

Add chain behavior test (processor + gain/passthrough context):

- validates serial chain with oscillator keeps upstream signal present.

### 3) Template regression framing

In generated project validation flow:

- explicit case: oscillator in signal chain + DAW input present => both audible.
- verify removing oscillator does not "restore" passthrough (i.e., passthrough should already be present).

### 4) Host smoke test (primary target)

Manual Ableton test on macOS:

- route audible track into plugin
- enable oscillator output
- confirm simultaneous DAW input + oscillator
- check no silence/dropout regression in passthrough-only case

---

## Risks and Mitigations

1. **Perceived loudness/clipping increase**
   - _Mitigation:_ include gain-staged acceptance checks; keep behavior explicit as additive.
2. **Semantic drift for future custom generators**
   - _Mitigation:_ document generator contract (additive + disabled=no-op) in DSP docs/guidance.
3. **False negatives due unrelated parameter-sync limitations**
   - _Mitigation:_ keep core regression tests at DSP unit level with explicit params; treat host automation sync as separate concern.

---

## Planner Handoff Guidance

Planner should produce an implementation plan with these work packages:

1. **WP1 (Core fix):** Update `Oscillator::process()` semantics to additive/no-op.
2. **WP2 (Unit regression):** Add focused tests for passthrough preservation and additive coexistence.
3. **WP3 (Template/Docs alignment):** Add minimal comments/docs clarifying generator semantics.
4. **WP4 (Verification):** Include Ableton macOS manual smoke scenario in test plan.
5. **WP5 (Risk checks):** Validate no real-time safety regressions and no passthrough-only breakage.

**Definition of done for milestone 18.11:**

- oscillator no longer mutes DAW passthrough
- oscillator + passthrough audible simultaneously
- automated regression coverage in processor tests
- host smoke test evidence in test-plan artifacts

---

## Concise design summary

- Root cause is architectural misuse of in-place buffer semantics: oscillator replaces/zeros instead of augmenting.
- Canonical contract now defined: generator processors are additive; disabled state is passthrough (no-op).
- Recommended fix is minimal and local (Option A), with strong regression framing and no runtime architecture rewrite.

## Key decisions

- Keep current serial `SignalChain![]` model.
- Define generator coexistence mathematically as $y[n] = x[n] + g \cdot o[n]$.
- Treat mixer/multi-bus architecture as future work, not part of this bug milestone.

## Risks & mitigations

- Main risk is clipping from additive summing; mitigate with gain-stage checks and explicit expectations.
- Main process risk is confusing test results with unrelated parameter-sync concerns; mitigate by anchoring on DSP-level deterministic tests first.

## Next handoff recommendation

Hand off to **Planner** now to produce `implementation-plan.md` with the WP breakdown above, sequencing:

1. oscillator semantic fix, 2) regression tests, 3) template/docs note, 4) Ableton verification steps.
