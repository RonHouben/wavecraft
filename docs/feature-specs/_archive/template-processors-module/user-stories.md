# User Stories: Template Processors Module

## Overview

Improve the CLI-generated template structure by introducing a `processors/` module with a complete oscillator example. This change teaches proper code organization from day one while providing a more engaging example than the current silent gain processor.

**Problem Being Solved:**
- Current single-file template doesn't scale to real-world plugins (5-20+ processors)
- No example showing how to implement a custom `Processor` trait
- Gain example is passive (no sound) — less engaging for learning
- Unclear where to add new processors in a growing plugin

**Value Proposition:**
- Teaches proper Rust module organization patterns immediately
- Provides complete DSP implementation example with real audio generation
- Creates clear convention for where new processors belong
- More engaging learning experience (oscillator makes sound!)

---

## Version

**Target Version:** `0.11.0` (minor bump from `0.10.0`)

**Rationale:**  
This is a **breaking change to the template** (developers using the old template will see a different structure), even though existing plugins are unaffected. The new structure represents a significant pedagogical improvement and architectural pattern change. Per coding standards, this warrants a minor version bump.

---

## User Story 1: Beginner Learns Proper Organization

**As a** beginner audio plugin developer using Wavecraft for the first time  
**I want** the template to show me proper code organization patterns  
**So that** I don't have to refactor as my plugin grows

### Acceptance Criteria

- [ ] Template includes `engine/src/processors/` directory
- [ ] `processors/mod.rs` shows clear module export pattern
- [ ] `lib.rs` imports from `processors` module (not inline)
- [ ] Structure is self-explanatory (no additional docs needed to understand layout)
- [ ] Pattern scales naturally (adding `processors/filter.rs` is obvious)

### Notes

- Beginners often struggle with Rust module organization
- Teaching good patterns from day one prevents bad habits
- Real-world plugins have 5-20+ processors; single-file doesn't scale
- Industry convention: separate domain logic into modules

---

## User Story 2: Developer Sees Complete Processor Implementation

**As an** experienced developer evaluating Wavecraft  
**I want** to see a complete custom `Processor` trait implementation  
**So that** I understand how to write my own DSP code

### Acceptance Criteria

- [ ] Template includes `processors/oscillator.rs` with complete implementation
- [ ] Oscillator has parameter struct using `#[derive(ProcessorParams)]`
- [ ] Implementation includes `set_sample_rate()` for proper initialization
- [ ] Implementation includes `reset()` for state clearing
- [ ] Code is well-commented explaining key concepts (phase accumulation, parameter usage)
- [ ] Example shows how to manage processor state (phase tracking)

### Notes

- Current template only shows wrapper macros, not trait implementation
- Developers need to see the full pattern to understand the abstraction
- Oscillator is simple enough to understand but non-trivial enough to teach
- Shows real DSP concepts: phase accumulation, sample rate handling, state management

---

## User Story 3: New Users Experience Immediate Audio Feedback

**As a** first-time Wavecraft user following the Getting Started guide  
**I want** the template plugin to generate actual sound  
**So that** I can immediately verify my setup works

### Acceptance Criteria

- [ ] Template plugin generates audible sine wave by default
- [ ] Frequency parameter (e.g., 440Hz default) is adjustable via UI
- [ ] Level parameter controls oscillator volume
- [ ] Oscillator is disabled by default in favor chains (no unwanted sound in DAW)
- [ ] Clear comments indicate which processor to enable/disable

### Notes

- Current Gain example requires external audio to test
- Immediate audio feedback is more satisfying for learning
- Validates entire audio pipeline (DSP → host → speakers) without external input
- Frequency/level parameters demonstrate parameter system immediately

---

## User Story 4: Developer Understands Where to Add Processors

**As a** plugin developer building a complex effect  
**I want** a clear convention for organizing multiple processors  
**So that** my codebase stays maintainable as it grows

### Acceptance Criteria

- [ ] `processors/` folder establishes clear organizational pattern
- [ ] `mod.rs` shows explicit module exports (not glob imports)
- [ ] README or template documentation explains the pattern
- [ ] Easy to add new processors: create file, implement trait, export in mod.rs
- [ ] Pattern matches Rust community conventions

### Notes

- Real plugins have chains like: Input → EQ → Compressor → Limiter → Output
- Without clear pattern, codebases become messy quickly
- Rust culture values explicit module boundaries
- Pattern should feel idiomatic to Rust developers

---

## User Story 5: SDK Improvements Are Immediately Available to New Users

**As a** plugin developer starting a new project  
**I want** the latest template structure when I run `wavecraft create`  
**So that** I benefit from SDK improvements automatically

### Acceptance Criteria

- [ ] `wavecraft create` always uses latest embedded template
- [ ] Template version matches CLI version (no version skew)
- [ ] Existing plugins are unaffected (no forced migration)
- [ ] Documentation clearly explains the new structure

### Notes

- Template is embedded in CLI via `include_dir!`, so updates are automatic
- Existing plugins continue to work with old structure
- Migration guide optional (users can adopt new pattern gradually)

---

## Technical Requirements

### Template Structure

**Before (current):**
```
engine/
  src/
    lib.rs    # ~10 lines: processor wrapper + plugin declaration
```

**After (proposed):**
```
engine/
  src/
    lib.rs                  # ~15 lines: imports + plugin assembly
    processors/
      mod.rs                # ~5 lines: module exports
      oscillator.rs         # ~60 lines: complete Processor implementation
```

### File Breakdown

**lib.rs** (~15 lines):
- Module declaration: `mod processors;`
- Processor imports: `use processors::Oscillator;`
- Wrapper macros: `wavecraft_processor!` for built-ins
- Plugin declaration: `wavecraft_plugin!` with `SignalChain![]`

**processors/mod.rs** (~5 lines):
- Public exports: `pub mod oscillator;`
- Re-export types if needed: `pub use oscillator::Oscillator;`

**processors/oscillator.rs** (~60 lines):
- Parameter struct with `#[derive(ProcessorParams)]`
- Oscillator struct with phase state
- `Processor` trait implementation
- Comment blocks explaining:
  - Phase accumulation formula
  - Sample rate dependency
  - State management (reset behavior)
  - Parameter usage pattern

### Oscillator Implementation Details

**Parameters:**
- `frequency`: 20Hz - 20kHz, default 440Hz (A4 note)
- `level`: 0.0 - 1.0, default 0.5 (attenuated for safety)

**State:**
- `phase: f32` — current phase position (0.0 to 1.0)
- Stored in struct, reset on `reset()` call

**DSP:**
- Phase increment per sample: `frequency / sample_rate`
- Output: `(phase * 2π).sin() * level`
- Phase wrap: `phase %= 1.0`

### Signal Chain Configuration

The template should demonstrate configuration flexibility:

```rust
// Option 1: Simple gain only (silent, requires external input)
signal: SignalChain![InputGain, OutputGain],

// Option 2: Oscillator example (generates tone)
signal: SignalChain![InputGain, MyOscillator, OutputGain],
```

Default should be **Option 1** (gain only) to avoid unexpected sound in DAW, with clear comments explaining how to enable the oscillator example.

---

## Documentation Requirements

### Files to Update

| File | Changes Needed |
|------|----------------|
| `docs/guides/sdk-getting-started.md` | Show new template structure, explain processors/ module |
| `docs/architecture/high-level-design.md` | Update template structure diagram |
| `docs/architecture/coding-standards.md` | Add section on processor organization patterns |
| `cli/sdk-templates/new-project/react/README.md` | Explain processors/ folder, show how to add new processors |

### Documentation Sections to Add

**In SDK Getting Started:**
- "Understanding the Template Structure" section
- "Adding Your First Processor" walkthrough
- "Organizing Complex Plugins" best practices

**In Template README:**
- "Project Structure" section explaining each directory
- "Adding Processors" step-by-step guide
- "Enabling the Oscillator Example" instructions

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Template comprehension | >80% of testers understand where to add processors | User testing questionnaire |
| Code organization quality | Zero "where should this go?" questions from testers | User testing feedback |
| Engagement | ≥50% of testers try modifying the oscillator | Git commit analysis or survey |
| Migration friction | Zero breaking changes to existing plugins | Compatibility testing |

---

## Risks and Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| **Increased cognitive load** | Beginners overwhelmed by more files | Medium | Comprehensive comments, clear README |
| **Oscillator confusion** | Users don't know how to disable sound | Low | Clear comments in lib.rs, documentation |
| **Pattern not followed** | Users still put everything in lib.rs | Low | Strong example, clear documentation |
| **Breaking existing workflows** | Users on old template are disrupted | Low | No migration required, version-gated |

---

## Dependencies

**Blocks:**
- None — this is a template-only change

**Blocked By:**
- Milestone 18 (Audio Pipeline Fixes) — should complete M18 first for prioritization clarity

**Related:**
- Milestone 19 (User Testing) — ideally complete this before M19 so testers evaluate the new structure
- Milestone 20 (V1.0 Release) — should be in V1.0 for best first impression

---

## Out of Scope

**Explicitly NOT included in this feature:**

- Migration tool for existing plugins (users can adopt pattern manually if desired)
- Multiple processor examples (oscillator only; users can reference OSS examples for more)
- Advanced DSP examples (filters, effects) — keep template simple
- UI changes for oscillator controls (parameter auto-discovery handles this)
- Breaking changes to existing macro API (template-only change)

---

## Priority Recommendation

**Priority:** Medium (Quality-of-Life Improvement)

**Reasoning:**

| Factor | Assessment |
|--------|------------|
| **User Impact** | Medium-High — improves learning curve for all new users |
| **Strategic Fit** | High — aligns with "simple and professional" SDK vision |
| **Effort** | Low-Medium — template changes + docs (~80 lines of new code) |
| **Risk** | Low — no breaking changes, well-understood requirements |
| **Dependencies** | None blocking |

**Recommended Placement:**
- **After M18** (Audio Pipeline Fixes) — M18 is higher priority (fixes critical audio gaps)
- **Before or during M19** (User Testing) — better to test new structure with beta users
- **Definitely before M20** (V1.0) — want this quality improvement in the first stable release

**Timeline Estimate:** 3-5 days (implementation 1-2 days, testing/docs 2-3 days)

---

## Open Questions

1. **Should we include multiple processor examples?** (e.g., both oscillator + filter)
   - **Recommendation:** No, keep it simple. One complete example is enough.

2. **Should oscillator be enabled by default in signal chain?**
   - **Recommendation:** No, default to silent gain chain. Oscillator should be opt-in via uncommenting to avoid surprising users with unexpected sound.

3. **Should we provide a migration guide for existing plugins?**
   - **Recommendation:** Optional documentation only. No automated migration. Let users adopt gradually.

4. **Should processors/ be a flat structure or nested?**
   - **Recommendation:** Flat for template. Real plugins can nest as needed (e.g., `processors/filters/lowpass.rs`).

---

## Next Steps

Once approved by PO:

1. **Architect:** Review this spec and create low-level design for implementation details
2. **Planner:** Create step-by-step implementation plan
3. **Coder:** Implement template changes, oscillator example, documentation updates
4. **Tester:** Validate template generation, test compilation, verify documentation
5. **QA:** Code review for quality, ensure coding standards compliance
6. **PO:** Archive spec and update roadmap upon completion
