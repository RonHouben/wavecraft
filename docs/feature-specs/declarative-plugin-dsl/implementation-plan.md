# Implementation Plan: Declarative Plugin DSL

## Overview

This plan implements a macro-based DSL that simplifies plugin creation from ~120 lines to ~8 lines. Users define processor types, optionally chain them, and the macro generates all nih-plug integration code.

**Target Version:** TBD (PO decision)  
**Estimated Effort:** 10-14 days  
**Feature Branch:** `feature/declarative-plugin-dsl`

---

## Prerequisites

Before starting implementation:
- [ ] Ensure `main` branch is up-to-date
- [ ] Feature branch created
- [ ] Low-level design reviewed and approved

---

## Phase 1: Core Traits & Infrastructure

**Goal:** Establish the foundational traits and create the proc-macro crate.

### Step 1.1: Create wavecraft-macros Crate

- **Action:** Create new proc-macro crate at `engine/crates/wavecraft-macros/`
- **Why:** Proc-macros must be in a separate crate
- **Dependencies:** None
- **Risk:** Low
- **Verification:** `cargo check -p wavecraft-macros` passes

**Files to create:**
```
engine/crates/wavecraft-macros/
├── Cargo.toml
└── src/
    └── lib.rs
```

**Cargo.toml:**
```toml
[package]
name = "wavecraft-macros"
version.workspace = true
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full", "parsing", "extra-traits"] }
quote = "1"
proc-macro2 = "1"
convert_case = "0.6"
```

### Step 1.2: Update Workspace Cargo.toml

**File:** `engine/Cargo.toml`

- **Action:** Add `wavecraft-macros` to workspace members
- **Why:** Workspace must include new crate
- **Dependencies:** Step 1.1
- **Risk:** Low

### Step 1.3: Extend Processor Trait with Params

**File:** `engine/crates/wavecraft-dsp/src/traits.rs`

- **Action:** Add `type Params` associated type to `Processor` trait
- **Why:** Links each processor to its parameter struct
- **Dependencies:** None
- **Risk:** Medium (breaking change to existing trait)
- **Verification:** Existing code updated to implement new trait

**Changes:**
```rust
pub trait Processor: Send + 'static {
    type Params: ProcessorParams + Default + Send + Sync + 'static;
    
    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport, params: &Self::Params);
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    fn reset(&mut self) {}
}
```

### Step 1.4: Create ProcessorParams Trait

**File:** `engine/crates/wavecraft-dsp/src/traits.rs`

- **Action:** Add `ProcessorParams` trait and supporting types
- **Why:** Defines parameter metadata contract
- **Dependencies:** None
- **Risk:** Low

**Add:**
```rust
pub trait ProcessorParams: Default + Send + Sync + 'static {
    fn param_specs() -> &'static [ParamSpec];
}

#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub name: &'static str,
    pub id_suffix: &'static str,
    pub range: ParamRange,
    pub default: f64,
    pub unit: &'static str,
}

#[derive(Debug, Clone)]
pub enum ParamRange {
    Linear { min: f64, max: f64 },
    Skewed { min: f64, max: f64, factor: f64 },
    Stepped { min: i32, max: i32 },
}

// Implement for unit type (no params)
impl ProcessorParams for () {
    fn param_specs() -> &'static [ParamSpec] { &[] }
}
```

### Step 1.5: Update Existing GainProcessor

**File:** `engine/crates/wavecraft-dsp/src/processor.rs`

- **Action:** Update `GainProcessor` to implement new `Processor` trait
- **Why:** Ensure existing code compiles with new trait
- **Dependencies:** Steps 1.3, 1.4
- **Risk:** Medium

---

## Phase 2: ProcessorParams Derive Macro

**Goal:** Create derive macro for automatic `ProcessorParams` implementation.

### Step 2.1: Implement ProcessorParams Derive

**File:** `engine/crates/wavecraft-macros/src/lib.rs`

- **Action:** Implement `#[derive(ProcessorParams)]` proc-macro
- **Why:** Automates param metadata generation
- **Dependencies:** Phase 1 complete
- **Risk:** Medium (proc-macro complexity)

**Implementation:**
1. Parse struct fields
2. Extract `#[param(...)]` attributes
3. Generate `param_specs()` implementation
4. Support field types: `f32`, `bool`, enums

### Step 2.2: Add Param Attribute Parsing

**File:** `engine/crates/wavecraft-macros/src/processor_params.rs` (NEW)

- **Action:** Create module for parsing `#[param(range = ..., default = ..., unit = "...")]`
- **Why:** Structured attribute parsing
- **Dependencies:** Step 2.1
- **Risk:** Low

**Supported attributes:**
- `range = MIN..=MAX` (linear)
- `range_skewed = MIN..=MAX, factor = F` (skewed)
- `default = VALUE`
- `unit = "STRING"`
- `smoothing = "linear(ms)" | "logarithmic(ms)"`

### Step 2.3: Test ProcessorParams Derive

- **Action:** Create test cases for derive macro
- **Why:** Verify correct code generation
- **Dependencies:** Steps 2.1, 2.2
- **Risk:** Low

**Test file:** `engine/crates/wavecraft-macros/tests/processor_params.rs`

```rust
#[derive(ProcessorParams, Default)]
struct TestParams {
    #[param(range = 0.0..=1.0, default = 0.5)]
    value: f32,
}

#[test]
fn test_param_specs_generated() {
    let specs = TestParams::param_specs();
    assert_eq!(specs.len(), 1);
    assert_eq!(specs[0].name, "Value");
}
```

---

## Phase 3: Built-in Processors

**Goal:** Create ready-to-use processor implementations.

### Step 3.1: Create Builtins Module Structure

**File:** `engine/crates/wavecraft-dsp/src/builtins/mod.rs` (NEW)

- **Action:** Create module structure for built-in processors
- **Why:** Organize built-in DSP code
- **Dependencies:** Phase 2 complete
- **Risk:** Low

```
engine/crates/wavecraft-dsp/src/builtins/
├── mod.rs
├── gain.rs
├── filter.rs
├── passthrough.rs
└── (compressor.rs, delay.rs deferred)
```

### Step 3.2: Implement GainDsp with ProcessorParams

**File:** `engine/crates/wavecraft-dsp/src/builtins/gain.rs`

- **Action:** Create `GainDsp` and `GainParams` using derive macro
- **Why:** First built-in processor
- **Dependencies:** Step 3.1, Phase 2
- **Risk:** Low

### Step 3.3: Implement FilterDsp

**File:** `engine/crates/wavecraft-dsp/src/builtins/filter.rs`

- **Action:** Create basic filter with frequency, resonance, type params
- **Why:** Common DSP building block
- **Dependencies:** Step 3.1
- **Risk:** Medium (DSP algorithm)

**Params:**
- `frequency: f32` (20-20000 Hz)
- `resonance: f32` (0-1)
- `filter_type: FilterType` (enum: LowPass, HighPass, BandPass)

### Step 3.4: Implement PassthroughDsp

**File:** `engine/crates/wavecraft-dsp/src/builtins/passthrough.rs`

- **Action:** Create no-op processor with no params
- **Why:** Useful for testing, placeholder
- **Dependencies:** Step 3.1
- **Risk:** Low

### Step 3.5: Add wavecraft-macros Dependency to wavecraft-dsp

**File:** `engine/crates/wavecraft-dsp/Cargo.toml`

- **Action:** Add dependency on wavecraft-macros
- **Why:** Built-ins use derive macros
- **Dependencies:** Phase 2
- **Risk:** Low

---

## Phase 4: Chain Combinator

**Goal:** Provide the single DSP composition pattern (`Chain![]` is used for all plugins, even single-processor).

### Step 4.1: Create Combinators Module

**File:** `engine/crates/wavecraft-dsp/src/combinators/mod.rs` (NEW)

- **Action:** Create module for processor combinators
- **Why:** Organize combinator types
- **Dependencies:** Phase 1 complete
- **Risk:** Low

### Step 4.2: Implement Chain Struct

**File:** `engine/crates/wavecraft-dsp/src/combinators/chain.rs`

- **Action:** Create `Chain<A, B>` struct implementing `Processor`
- **Why:** Enables serial processor composition
- **Dependencies:** Step 4.1
- **Risk:** Medium (generic bounds)

**Key aspects:**
- `Chain<A, B>` holds two processors
- `process()` calls A then B on same buffer
- `Params` combines both param sets

### Step 4.3: Implement Chain! Macro

**File:** `engine/crates/wavecraft-dsp/src/combinators/mod.rs`

- **Action:** Create `Chain![]` declarative macro
- **Why:** `Chain![A]` returns `A`, `Chain![A, B, C]` nests as `Chain<A, Chain<B, C>>`
- **Dependencies:** Step 4.2
- **Risk:** Low

```rust
#[macro_export]
macro_rules! Chain {
    // Single processor: no wrapping, zero overhead
    ($single:ty) => { $single };
    // Multiple: nest into Chain<A, Chain<B, ...>>
    ($first:ty, $($rest:ty),+ $(,)?) => {
        $crate::combinators::Chain<$first, Chain![$($rest),+]>
    };
}
```

**Note:** This means `Chain![MyGain { level: 0.0 }]` compiles to exactly `MyGain`—no wrapper overhead for single-processor plugins.

### Step 4.4: Test Chain Combinator

- **Action:** Write tests for processor chaining
- **Why:** Verify chain processes in correct order
- **Dependencies:** Steps 4.2, 4.3
- **Risk:** Low

---

## Phase 5: wavecraft_processor! Macro

**Goal:** Enable user-defined processor types wrapping built-ins.

### Step 5.1: Implement wavecraft_processor! Macro

**File:** `engine/crates/wavecraft-core/src/macros.rs`

- **Action:** Create declarative macro for wrapping built-in DSP
- **Why:** Users create named types like `InputGain` wrapping `Gain`
- **Dependencies:** Phase 3 complete
- **Risk:** Medium

**Syntax:**
```rust
wavecraft_processor!(InputGain => Gain);
```

**Generated:**
```rust
pub struct InputGain(wavecraft_dsp::builtins::GainDsp);
impl Processor for InputGain { /* delegates */ }
```

### Step 5.2: Support All Built-in Types

- **Action:** Add match arms for `Gain`, `Filter`, `Passthrough`
- **Why:** Each built-in needs a wrapper pattern
- **Dependencies:** Step 5.1
- **Risk:** Low

### Step 5.3: Test wavecraft_processor! Macro

- **Action:** Write tests verifying wrapper generation
- **Why:** Ensure correct delegation
- **Dependencies:** Step 5.2
- **Risk:** Low

---

## Phase 6: wavecraft_plugin! Macro

**Goal:** The main macro that generates complete plugin code.

### Step 6.1: Design Macro Input Parsing

**File:** `engine/crates/wavecraft-macros/src/plugin.rs` (NEW)

- **Action:** Implement input parsing for plugin DSL
- **Why:** Parse name, vendor, dsp, etc.
- **Dependencies:** Phases 1-5 complete
- **Risk:** High (complex proc-macro)

**Parse structure:**
```rust
struct PluginDef {
    name: String,
    vendor: String,
    url: Option<String>,
    email: Option<String>,
    processors: Vec<ProcessorDef>,  // Always a vec (Chain![]), even if one
    clap_id: Option<String>,
    vst3_id: Option<[u8; 16]>,
}

struct ProcessorDef {
    processor_type: Type,
    params: StructInit,
}
```

**Note:** No `enum DspDef` needed—we always parse `Chain![...]`, simplifying the macro.

### Step 6.2: Generate Plugin Struct

- **Action:** Generate plugin struct with meter channels
- **Why:** Core plugin infrastructure
- **Dependencies:** Step 6.1
- **Risk:** Medium

**Generated:**
```rust
pub struct __WavecraftPlugin {
    params: Arc<__WavecraftParams>,
    processor: /* DSP type */,
    meter_producer: MeterProducer,
    meter_consumer: Arc<Mutex<MeterConsumer>>,
}
```

### Step 6.3: Generate Params Struct

- **Action:** Generate nih-plug Params struct from processor params
- **Why:** Flattened params with proper IDs
- **Dependencies:** Step 6.1
- **Risk:** High (ID generation, nih-plug attributes)

**Key logic:**
1. Collect all processors in chain
2. For each, get `Params::param_specs()`
3. Generate `#[id = "processor_param"]` for each
4. Generate `FloatParam::new()` with range/default

### Step 6.4: Generate Plugin Trait Impl

- **Action:** Generate `impl Plugin for __WavecraftPlugin`
- **Why:** Core nih-plug integration
- **Dependencies:** Steps 6.2, 6.3
- **Risk:** High

**Generate:**
- `const NAME`, `VENDOR`, `URL`, `EMAIL`, `VERSION`
- `const AUDIO_IO_LAYOUTS` (stereo)
- `fn params()`, `fn editor()`
- `fn initialize()`, `fn reset()`, `fn process()`

### Step 6.5: Generate Format Impls & Exports

- **Action:** Generate `ClapPlugin`, `Vst3Plugin` impls and export macros
- **Why:** Complete plugin compilation
- **Dependencies:** Step 6.4
- **Risk:** Medium

**Generate:**
```rust
impl ClapPlugin for __WavecraftPlugin { /* ... */ }
impl Vst3Plugin for __WavecraftPlugin { /* ... */ }
nih_export_clap!(__WavecraftPlugin);
nih_export_vst3!(__WavecraftPlugin);
```

### Step 6.6: Add Error Messages

- **Action:** Implement helpful compile-time error messages
- **Why:** Good DX when macro usage is wrong
- **Dependencies:** Step 6.5
- **Risk:** Medium

**Errors to handle:**
- Missing required fields (name, signal)
- Invalid param field names
- Type mismatches

---

## Phase 7: Integration & Template

**Goal:** Update template and verify end-to-end.

### Step 7.1: Update Prelude Exports

**File:** `engine/crates/wavecraft-core/src/prelude.rs`

- **Action:** Re-export all macro and DSP types
- **Why:** Single import for users
- **Dependencies:** Phases 1-6
- **Risk:** Low

```rust
pub use wavecraft_macros::{wavecraft_plugin, ProcessorParams};
pub use wavecraft_dsp::{Processor, Transport, Chain};
pub use wavecraft_dsp::builtins::*;
pub use crate::wavecraft_processor;
```

### Step 7.2: Update Plugin Template

**File:** `wavecraft-plugin-template/engine/src/lib.rs`

- **Action:** Replace boilerplate with macro usage
- **Why:** Demonstrate new DX
- **Dependencies:** Step 7.1
- **Risk:** Medium

**Before:** ~120 lines  
**After:**
```rust
use wavecraft::prelude::*;

wavecraft_processor!(MyGain => Gain);

wavecraft_plugin! {
    name: "My Plugin",
    vendor: "My Company",
    signal: Chain![
        MyGain { level: 0.0 },
    ],
}
```

### Step 7.3: Verify Template Builds

- **Action:** Build template plugin
- **Why:** Ensure macro generates valid code
- **Dependencies:** Step 7.2
- **Risk:** Medium
- **Verification:** `cargo xtask bundle` succeeds in template

### Step 7.4: Test Plugin in DAW

- **Action:** Load generated plugin in DAW
- **Why:** End-to-end verification
- **Dependencies:** Step 7.3
- **Risk:** Low

---

## Phase 8: Documentation

**Goal:** Update guides for new workflow.

### Step 8.1: Update SDK Getting Started

**File:** `docs/guides/sdk-getting-started.md`

- **Action:** Rewrite with macro-based examples
- **Why:** Reflect new simplified DX
- **Dependencies:** Phase 7 complete
- **Risk:** Low

### Step 8.2: Create Custom DSP Guide

**File:** `docs/guides/custom-dsp-guide.md` (NEW)

- **Action:** Document `Processor` trait, real-time safety, full examples
- **Why:** Advanced users need this
- **Dependencies:** Phase 7 complete
- **Risk:** Low

### Step 8.3: Create DSP Chains Guide

**File:** `docs/guides/dsp-chains-guide.md` (NEW)

- **Action:** Document `Chain![]` macro and processor composition
- **Why:** Intermediate complexity level
- **Dependencies:** Phase 7 complete
- **Risk:** Low

### Step 8.4: Update High-Level Design

**File:** `docs/architecture/high-level-design.md`

- **Action:** Add section on macro architecture
- **Why:** Keep architecture docs current
- **Dependencies:** Phase 7 complete
- **Risk:** Low

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Proc-macro complexity | High | High | Start simple, iterate. Use syn/quote docs. |
| Breaking Processor trait | Medium | Medium | Update all usages in same PR |
| Parameter routing bugs | Medium | High | Extensive unit tests, manual DAW testing |
| Compile time increase | Low | Low | Proc-macros are fast; monitor with `cargo build --timings` |
| IDE support for macros | Medium | Medium | Use `cargo expand` for debugging; document limitations |

---

## Success Criteria

- [ ] Simple gain plugin compiles in < 10 lines
- [ ] Chained plugins work correctly
- [ ] Generated param IDs match expected format
- [ ] Plugin loads in DAW without errors
- [ ] Parameter automation works
- [ ] Meters display correctly
- [ ] Template updated and builds
- [ ] Documentation complete

---

## Dependencies Summary

```
Phase 1 (Traits)
    │
    ├──► Phase 2 (ProcessorParams Derive)
    │        │
    │        └──► Phase 3 (Built-ins)
    │                 │
    └──► Phase 4 (Chain) ◄───┘
              │
              └──► Phase 5 (wavecraft_processor!)
                        │
                        └──► Phase 6 (wavecraft_plugin!)
                                  │
                                  └──► Phase 7 (Integration)
                                            │
                                            └──► Phase 8 (Docs)
```

---

## Estimated Timeline

| Phase | Duration | Cumulative |
|-------|----------|------------|
| Phase 1: Core Traits | 1 day | Day 1 |
| Phase 2: ProcessorParams Derive | 2 days | Day 3 |
| Phase 3: Built-in Processors | 2 days | Day 5 |
| Phase 4: Chain Combinator | 1 day | Day 6 |
| Phase 5: wavecraft_processor! | 1 day | Day 7 |
| Phase 6: wavecraft_plugin! | 4 days | Day 11 |
| Phase 7: Integration | 2 days | Day 13 |
| Phase 8: Documentation | 1 day | Day 14 |

**Total: ~14 days**
