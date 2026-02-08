# QA Report: Template Processors Module

**Date**: 2026-02-08
**Reviewer**: QA Agent
**Status**: FAIL

## Summary

| Severity | Count |
|----------|-------|
| Critical | 0 |
| High | 0 |
| Medium | 3 |
| Low | 2 |

**Overall**: FAIL (3 Medium findings require coder attention)

## Automated Check Results

**Note:** Automated checks (linting, type-checking, tests) were run by the Tester agent via `cargo xtask ci-check` prior to QA review. Results documented in test-plan.md.

- Linting: ✅ PASSED (Engine + UI, 12.1s)
- Tests: ✅ PASSED (148 engine tests + 28 UI tests)

## Findings

| ID | Severity | Category | Description | Location | Recommendation |
|----|----------|----------|-------------|----------|----------------|
| 1 | Medium | DSP Correctness | Oscillator stereo phase inconsistency | `cli/sdk-templates/.../oscillator.rs:69-79` | Save/restore phase per channel |
| 2 | Medium | DSP Robustness | Division by zero when `sample_rate` is 0.0 | `cli/sdk-templates/.../oscillator.rs:67` | Add early return guard |
| 3 | Medium | Documentation | README "Plugin Configuration" has stale signal chain | `cli/sdk-templates/.../README.md:274` | Update to match new pattern |
| 4 | Low | Documentation | Markdown formatting: `---` and heading on same line | `cli/sdk-templates/.../README.md:43` | Split onto separate lines |
| 5 | Low | Documentation | README references non-existent `dev-audio.rs` | `cli/sdk-templates/.../README.md:243-247` | Remove or note as future work |

---

### Finding 1: Oscillator Stereo Phase Inconsistency

**Severity**: Medium  
**Category**: DSP Correctness  
**Location**: `cli/sdk-templates/new-project/react/engine/src/processors/oscillator.rs` lines 69–79

**Description:**

The oscillator's `process()` method iterates channels in sequence, advancing `self.phase` through all samples of channel 0, then continuing through channel 1. This means channel 1 (Right) receives a phase-shifted version of the sine wave relative to channel 0 (Left).

For a 440 Hz sine at 44100 Hz with a 512-sample buffer, the right channel would be offset by ~5 cycles — producing an audibly different waveform.

```rust
// Current code: phase drifts across channels
for channel in buffer.iter_mut() {
    for sample in channel.iter_mut() {
        *sample = (self.phase * std::f32::consts::TAU).sin() * params.level;
        self.phase += phase_delta;
        // ...
    }
}
```

**Impact**: As a teaching example, this introduces a subtle DSP bug that could confuse beginners learning multi-channel processing. The left and right outputs will not be identical, which is unexpected for a mono sine generator.

**Recommendation**: Save the phase before the channel loop and restore it for each channel:

```rust
let start_phase = self.phase;
for channel in buffer.iter_mut() {
    self.phase = start_phase;
    for sample in channel.iter_mut() {
        *sample = (self.phase * std::f32::consts::TAU).sin() * params.level;
        self.phase += phase_delta;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }
}
```

After the loop, `self.phase` retains the correct position for the next buffer (the end-of-last-channel position is correct since all channels process the same number of samples).

**Standard**: Coding standards — Real-Time Safety: "DSP traits enforce the contract" (correctness is part of the contract).

---

### Finding 2: Division by Zero When `sample_rate` Is 0.0

**Severity**: Medium  
**Category**: DSP Robustness  
**Location**: `cli/sdk-templates/new-project/react/engine/src/processors/oscillator.rs` line 67

**Description:**

The `Oscillator` struct derives `Default`, which initializes `sample_rate` to `0.0`. The `process()` method computes:

```rust
let phase_delta = params.frequency / self.sample_rate;
```

If `process()` is called before `set_sample_rate()`, `self.sample_rate` is `0.0`, producing `phase_delta = ±inf`. Subsequent phase accumulation yields `NaN` output, which propagates through downstream processors and can cause issues in hosts.

**Impact**: While hosts should always call `set_sample_rate()` before `process()`, defensive coding in a teaching template sets a better example. NaN propagation is a known footgun in audio DSP.

**Recommendation**: Add a guard at the top of `process()`:

```rust
fn process(&mut self, buffer: &mut [&mut [f32]], _transport: &Transport, params: &Self::Params) {
    if self.sample_rate == 0.0 {
        return; // Not yet initialized — leave buffer unchanged
    }
    // ...
}
```

**Standard**: Coding standards — Real-Time Safety checklist, high-level design — "DSP traits enforce the contract."

---

### Finding 3: README "Plugin Configuration" Has Stale Signal Chain

**Severity**: Medium  
**Category**: Documentation Accuracy  
**Location**: `cli/sdk-templates/new-project/react/README.md` line 274

**Description:**

The "Plugin Configuration → engine/src/lib.rs" section shows:

```rust
wavecraft_plugin! {
    name: "{{plugin_name_title}}",
    signal: SignalChain![{{plugin_name_pascal}}Gain],  // Processor chain entry point
}
```

This uses the **old** single-wrapper pattern (`{{plugin_name_pascal}}Gain`), which was updated everywhere else in the README (Development Workflow section at lines 85–106 correctly shows `InputGain, OutputGain`). The actual `lib.rs` template uses `SignalChain![InputGain, OutputGain]`.

**Impact**: Inconsistent documentation within the same README. A reader encounters the new correct pattern first, then sees the old pattern in the Plugin Configuration section, creating confusion.

**Recommendation**: Update line 274 to match the new pattern:

```rust
wavecraft_plugin! {
    name: "{{plugin_name_title}}",
    signal: SignalChain![InputGain, OutputGain],
}
```

**Note**: This is pre-existing content that was not updated alongside the Development Workflow changes in this feature. While not introduced by this feature, it should be fixed for consistency since the surrounding README content was significantly rewritten.

**Standard**: Coding standards — Documentation References: "Always link to relevant documentation" / keep documentation consistent.

---

### Finding 4: Markdown Formatting — Heading on Same Line as Rule

**Severity**: Low  
**Category**: Documentation Formatting  
**Location**: `cli/sdk-templates/new-project/react/README.md` line 43

**Description:**

```markdown
--- ## Project Structure
```

The horizontal rule (`---`) and heading (`## Project Structure`) are on the same line. Most markdown parsers render this as a horizontal rule followed by literal text `## Project Structure` (not a proper heading). They should be on separate lines.

**Note**: Pre-existing issue, not introduced by this feature.

**Recommendation**: Split onto two lines:
```markdown
---

## Project Structure
```

---

### Finding 5: README References Non-Existent `dev-audio.rs`

**Severity**: Low  
**Category**: Documentation Accuracy  
**Location**: `cli/sdk-templates/new-project/react/README.md` lines 243–247

**Description:**

The README states:
> "The template includes `engine/src/bin/dev-audio.rs` and the necessary configuration in `engine/Cargo.toml`"

However, no `engine/src/bin/dev-audio.rs` file exists in the template, and the template `Cargo.toml.template` has no `[[bin]]` section (though it does include optional `wavecraft-dev-server` and `cpal` dependencies). This reference appears to describe a planned or partially implemented feature.

**Note**: Pre-existing issue, not introduced by this feature.

**Recommendation**: Either remove this section or add a note that the dev-audio binary setup is a future enhancement. The optional dependencies in Cargo.toml.template should also be cleaned up if dev-audio is not yet implemented.

---

## Positive Observations

1. **Derive macro path change is well-executed** — Switching from `wavecraft_dsp::` to `::wavecraft::` paths with the `extern crate` alias in tests is a clean approach. The comment in `processor_params.rs` tests explaining the alias is clear and helpful.

2. **Dual-namespace re-export is correctly documented** — The `ProcessorParams` trait (from `wavecraft_dsp`) and derive macro (from `wavecraft_macros`) coexisting via different namespaces is well-commented in `wavecraft-nih_plug/src/lib.rs` (lines 31–33).

3. **Template code quality is excellent** — The oscillator implementation is well-commented with educational content explaining key DSP concepts (phase accumulation, sample rate, state management). The 4-step guide in `processors/mod.rs` is concise and actionable.

4. **Version consistency is verified** — All workspace crates correctly show 0.11.0. CLI version (0.9.1) is correctly managed separately by CI auto-bump.

5. **Real-time safety in template code** — No heap allocations, no locks, no syscalls, no panics in the `process()` method. Uses only stack-allocated variables and in-place mutation. Adheres to coding standards.

6. **User story acceptance criteria substantially met** — All 5 user stories' core requirements are fulfilled. The processors module pattern, oscillator example, signal chain comments, and documentation updates are comprehensive.

## Architectural Concerns

No architectural concerns. The changes are well-scoped to template files and a targeted derive macro path fix. Domain boundaries are maintained.

## Handoff Decision

**Target Agent**: Coder
**Reasoning**: Three Medium findings require code changes — two in the oscillator template (stereo phase fix, division guard) and one documentation update (stale signal chain in README). The two Low findings are pre-existing issues that should ideally be fixed while the README is being touched. All fixes are straightforward and do not require architectural review.
