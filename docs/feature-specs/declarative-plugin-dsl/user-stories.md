# User Stories: Declarative Plugin DSL

## Overview

This feature introduces a macro-based DSL that dramatically simplifies plugin creation. Users define processor types, compose them into signal chains, and the macro generates all nih-plug boilerplate code. The goal is to reduce a simple gain plugin from ~120 lines to ~12 lines while maintaining full type safety and extensibility.

## Version

**Target Version:** `0.6.0` (minor bump from `0.5.0`)

**Rationale:** This is a significant new feature that introduces a new public API (macros, traits, built-in processors). It changes how users interact with the SDK and represents a major DX improvement. Minor version bump is appropriate per semantic versioning.

---

## User Story 1: Minimal Plugin Setup

**As a** plugin developer new to Wavecraft  
**I want** to create a working audio plugin with minimal code  
**So that** I can get started quickly and see results in my DAW within minutes

### Acceptance Criteria

- [ ] A complete gain plugin can be written in ≤12 lines of code
- [ ] The following code compiles and produces a working VST3/CLAP plugin:
  ```rust
  use wavecraft::prelude::*;
  
  wavecraft_processor!(MyGain => Gain);
  
  wavecraft_plugin! {
      name: "My First Plugin",
      vendor: "My Company",
      signal: Chain![
          MyGain { level: 0.0 },
      ],
  }
  ```
- [ ] Plugin loads in Ableton Live without errors
- [ ] Gain parameter is exposed and automatable
- [ ] Meter displays audio levels correctly

### Notes

- This is the "Getting Started" path—optimized for first-time users
- No DSP knowledge required for this tier
- Built-in `Gain` processor handles all audio processing

---

## User Story 2: Signal Chain Composition

**As a** plugin developer building a channel strip  
**I want** to chain multiple processors together  
**So that** I can create complex signal flows without manual wiring

### Acceptance Criteria

- [ ] Multiple processors can be chained using `Chain![]` macro
- [ ] The following code produces a working channel strip:
  ```rust
  wavecraft_processor!(InputGain => Gain);
  wavecraft_processor!(HighPass => Filter);
  wavecraft_processor!(Dynamics => Compressor);
  wavecraft_processor!(OutputGain => Gain);
  
  wavecraft_plugin! {
      name: "Channel Strip",
      vendor: "My Company",
      signal: Chain![
          InputGain { level: 0.0 },
          HighPass { frequency: 80.0, filter_type: HighPass },
          Dynamics { threshold: -18.0, ratio: 4.0 },
          OutputGain { level: -6.0 },
      ],
  }
  ```
- [ ] Audio flows through processors in declared order
- [ ] Each processor's parameters are independently controllable
- [ ] Parameter IDs are prefixed with processor name (e.g., `input_gain_level`)

### Notes

- `Chain![A]` (single processor) compiles to just `A` with zero overhead
- Processor order determines signal flow
- Each processor type can only appear once unless using different user-defined types

---

## User Story 3: User-Defined Processor Types

**As a** plugin developer  
**I want** to give my processors meaningful names  
**So that** my code is self-documenting and parameter IDs are semantic

### Acceptance Criteria

- [ ] `wavecraft_processor!(MyName => BuiltIn)` creates a new type wrapping the built-in
- [ ] The new type inherits all parameters from the built-in processor
- [ ] Parameter IDs use the user-defined name: `my_name_level`, not `gain_level`
- [ ] IDE autocomplete works for parameter fields
- [ ] Compile error if user tries to use same processor type twice in a chain

### Notes

- This is the "naming" layer—users own their processor names
- Encourages semantic naming: `InputTrim`, `OutputFader` instead of `Gain`, `Gain`

---

## User Story 4: Enum Parameters

**As a** plugin developer building a filter  
**I want** to use enum parameters for filter type selection  
**So that** users can choose between LowPass, HighPass, BandPass, etc.

### Acceptance Criteria

- [ ] Built-in `Filter` processor includes `filter_type` enum parameter
- [ ] Enum values are: `LowPass`, `HighPass`, `BandPass` (minimum)
- [ ] Enum parameter appears as dropdown in DAW
- [ ] Enum parameter is automatable
- [ ] Custom processors can define their own enum parameters:
  ```rust
  #[derive(ProcessorParams)]
  pub struct MyFilterParams {
      #[param(range = 20.0..=20000.0)]
      pub frequency: f32,
      
      #[param(enum)]
      pub filter_type: FilterType,
  }
  
  #[derive(ParamEnum)]
  pub enum FilterType {
      LowPass,
      HighPass,
      BandPass,
  }
  ```

### Notes

- Enums map to integer parameters internally for DAW compatibility
- Display names can be customized with attributes if needed

---

## User Story 5: Type-Safe Parameters

**As a** plugin developer  
**I want** compile-time validation of parameter names  
**So that** typos are caught at build time, not runtime

### Acceptance Criteria

- [ ] Using an invalid parameter name produces a compile error:
  ```rust
  MyGain { gain: 0.0 }  // ERROR: GainParams has `level`, not `gain`
  ```
- [ ] Error message lists available parameters
- [ ] IDE shows autocomplete for valid parameter names
- [ ] All parameters from `ProcessorParams` are available

### Notes

- This leverages Rust's type system via associated types
- Each processor has `type Params` that defines valid fields

---

## User Story 6: Parameter Groups in UI

**As an** end user of a Wavecraft plugin  
**I want** parameters grouped by processor in the UI  
**So that** I can easily find and adjust related controls

### Acceptance Criteria

- [ ] UI displays parameters in collapsible groups
- [ ] Group names match processor names (Title Case): "Input Gain", "High Pass"
- [ ] Groups are expanded by default
- [ ] Groups can be collapsed/expanded by clicking header
- [ ] Parameter order within group matches declaration order

### Notes

- Requires IPC protocol update to include `group` field
- Groups improve UX for plugins with many parameters

---

## User Story 7: Custom DSP Implementation

**As an** advanced plugin developer  
**I want** to write my own DSP algorithm  
**So that** I can create unique audio effects not covered by built-ins

### Acceptance Criteria

- [ ] Custom processor can be created by implementing `Processor` trait
- [ ] Custom parameters defined with `#[derive(ProcessorParams)]`
- [ ] The following code works:
  ```rust
  #[derive(ProcessorParams, Default)]
  pub struct MySaturatorParams {
      #[param(range = 0.0..=1.0, default = 0.5)]
      pub drive: f32,
      
      #[param(range = 0.0..=1.0, default = 1.0)]
      pub mix: f32,
  }
  
  pub struct MySaturator {
      sample_rate: f32,
  }
  
  impl Processor for MySaturator {
      type Params = MySaturatorParams;
      
      fn process(&mut self, buffer: &mut [&mut [f32]], _transport: &Transport, params: &MySaturatorParams) {
          for channel in buffer.iter_mut() {
              for sample in channel.iter_mut() {
                  let saturated = (*sample * (1.0 + params.drive * 10.0)).tanh();
                  *sample = *sample * (1.0 - params.mix) + saturated * params.mix;
              }
          }
      }
  }
  
  wavecraft_plugin! {
      name: "Saturator",
      signal: Chain![
          MySaturator { drive: 0.5, mix: 1.0 },
      ],
  }
  ```
- [ ] Custom processor receives correct sample rate via `set_sample_rate()`
- [ ] Custom processor can be mixed with built-in processors in chain

### Notes

- This is the "advanced" path for users who need custom algorithms
- Must follow real-time safety rules (no allocations, locks, syscalls)
- Documented in separate "Custom DSP Guide"

---

## User Story 8: Built-in Processor Library

**As a** plugin developer  
**I want** access to common DSP building blocks  
**So that** I don't have to implement standard effects from scratch

### Acceptance Criteria

- [ ] `Gain` processor available with `level` (dB) and `bypass` parameters
- [ ] `Filter` processor available with `frequency`, `resonance`, `filter_type` parameters
- [ ] `Passthrough` processor available for testing (no parameters)
- [ ] All built-in processors implement `Processor` trait
- [ ] All built-in processors are real-time safe

### Notes

- Compressor and Delay are deferred to future release
- Built-ins serve as reference implementations for custom processors

---

## User Story 9: Documentation and Guides

**As a** plugin developer  
**I want** clear documentation for the new macro system  
**So that** I can learn how to use it effectively

### Acceptance Criteria

- [ ] "Getting Started" guide updated with macro-based examples
- [ ] "Custom DSP Guide" created for advanced users
- [ ] "Signal Chains Guide" documents `Chain![]` composition
- [ ] Warning about preset breaking changes documented:
  > ⚠️ Renaming a processor (e.g., `InputGain` → `InputTrim`) changes parameter IDs and will break existing presets.
- [ ] High-level design document updated with macro architecture

### Notes

- Three-tier documentation: Getting Started → Chains → Custom DSP
- Each tier increases in complexity

---

## User Story 10: Template Update

**As a** plugin developer cloning the template  
**I want** the template to use the new macro system  
**So that** I start with the simplified workflow by default

### Acceptance Criteria

- [ ] `wavecraft-plugin-template` uses `wavecraft_plugin!` macro
- [ ] Template compiles with `cargo xtask bundle`
- [ ] Template plugin loads in DAW
- [ ] Template demonstrates single-processor pattern
- [ ] README updated with new syntax examples

### Notes

- Template is the first-run experience—must be polished
- Should demonstrate the simplest possible plugin

---

## Out of Scope (Deferred)

The following are explicitly **not** included in this release:

1. **Parallel combinator** (`Parallel![A, B]`) — Deferred to future release
2. **Parameter ID override** — Users cannot override generated IDs
3. **Preset migration tooling** — Users must manually update presets if they rename processors

---

## Technical Notes

### Parameter ID Generation

| Processor | Field | Generated ID | Display Name |
|-----------|-------|--------------|--------------|
| `InputGain` | `level` | `input_gain_level` | "Input Gain Level" |
| `HighPass` | `frequency` | `high_pass_frequency` | "High Pass Frequency" |

### Breaking Change Policy

Renaming a processor changes all its parameter IDs. This is documented as expected behavior. Users who need stable IDs across renames should plan their processor names carefully before shipping.

---

## Definition of Done

- [ ] All acceptance criteria met
- [ ] All tests passing (`cargo xtask test`)
- [ ] No lint errors (`cargo xtask lint`)
- [ ] Documentation updated
- [ ] Template updated and tested in DAW
- [ ] Version bumped to 0.6.0
- [ ] PR reviewed and approved
