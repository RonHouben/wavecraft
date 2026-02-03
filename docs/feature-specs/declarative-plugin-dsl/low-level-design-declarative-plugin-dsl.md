# Low-Level Design: Declarative Plugin DSL

## Overview

This document describes the technical design for a declarative macro-based DSL that dramatically simplifies plugin creation. Users define processors with meaningful names, compose them into chains, and the macro generates all nih-plug boilerplate.

**Goal:** Reduce a simple gain plugin from ~120 lines to ~8 lines while maintaining full type safety and extensibility.

---

## Architecture

### Current State

```
┌─────────────────────────────────────────────────────────────────┐
│  User Code (~120 lines)                                         │
│                                                                 │
│  struct MyPlugin { params, processor, meter_*, ... }            │
│  struct MyPluginParams { #[id = "gain"] gain: FloatParam }      │
│  impl Default for MyPlugin { ... }                              │
│  impl Default for MyPluginParams { ... }                        │
│  impl Plugin for MyPlugin { NAME, VENDOR, process(), ... }      │
│  impl ClapPlugin for MyPlugin { ... }                           │
│  impl Vst3Plugin for MyPlugin { ... }                           │
│  nih_export_clap!(MyPlugin);                                    │
│  nih_export_vst3!(MyPlugin);                                    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Target State

```
┌─────────────────────────────────────────────────────────────────┐
│  User Code (~10 lines)                                          │
│                                                                 │
│  wavecraft_processor!(MyGain => Gain);                          │
│                                                                 │
│  wavecraft_plugin! {                                            │
│      name: "My Plugin",                                         │
│      vendor: "My Company",                                      │
│      signal: Chain![                                            │
│          MyGain { level: 0.0 },                                 │
│      ],                                                         │
│  }                                                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Macro Expansion
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Generated Code (~200 lines)                                    │
│                                                                 │
│  - Plugin struct with meter channels                            │
│  - Params struct with nih-plug attributes                       │
│  - All trait impls (Plugin, ClapPlugin, Vst3Plugin)             │
│  - DSP chain wiring                                             │
│  - Parameter → Processor binding                                │
│  - Export macros                                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Design

### 1. Core Traits

**File:** `engine/crates/wavecraft-dsp/src/traits.rs`

```rust
/// Trait for user-implemented DSP processors.
pub trait Processor: Send + 'static {
    /// The parameter struct for this processor.
    type Params: ProcessorParams + Default + Send + Sync + 'static;
    
    /// Process audio buffer with current parameter values.
    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport, params: &Self::Params);
    
    /// Called when sample rate changes.
    fn set_sample_rate(&mut self, _sample_rate: f32) {}
    
    /// Reset internal state (delay lines, filters, etc.)
    fn reset(&mut self) {}
}

/// Marker trait for processor parameter structs.
/// Automatically implemented by #[derive(ProcessorParams)].
pub trait ProcessorParams: Default + Send + Sync + 'static {
    /// Get parameter metadata for nih-plug integration.
    fn param_specs() -> &'static [ParamSpec];
}

/// Parameter specification for code generation.
#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub name: &'static str,
    pub id_suffix: &'static str,
    pub range: ParamRange,
    pub default: f64,
    pub unit: &'static str,
    pub smoothing: SmoothingSpec,
}

#[derive(Debug, Clone)]
pub enum ParamRange {
    Linear { min: f64, max: f64 },
    Skewed { min: f64, max: f64, factor: f64 },
    Stepped { min: i32, max: i32 },
}

#[derive(Debug, Clone, Default)]
pub struct SmoothingSpec {
    pub style: SmoothingStyle,
    pub duration_ms: f32,
}

#[derive(Debug, Clone, Default)]
pub enum SmoothingStyle {
    #[default]
    None,
    Linear,
    Logarithmic,
    Exponential,
}
```

---

### 2. ProcessorParams Derive Macro

**File:** `engine/crates/wavecraft-macros/src/processor_params.rs` (NEW crate)

```rust
/// Derive macro for ProcessorParams trait.
/// 
/// # Example
/// ```rust
/// #[derive(ProcessorParams, Default)]
/// pub struct GainParams {
///     #[param(range = -24.0..=24.0, default = 0.0, unit = "dB")]
///     pub level: f32,
///     
///     #[param(default = false)]
///     pub bypass: bool,
/// }
/// ```
#[proc_macro_derive(ProcessorParams, attributes(param))]
pub fn derive_processor_params(input: TokenStream) -> TokenStream {
    // Parse struct fields
    // Extract #[param(...)] attributes
    // Generate ProcessorParams impl with param_specs()
}
```

**Generated code example:**

```rust
impl ProcessorParams for GainParams {
    fn param_specs() -> &'static [ParamSpec] {
        static SPECS: &[ParamSpec] = &[
            ParamSpec {
                name: "Level",
                id_suffix: "level",
                range: ParamRange::Linear { min: -24.0, max: 24.0 },
                default: 0.0,
                unit: "dB",
                smoothing: SmoothingSpec::default(),
            },
            ParamSpec {
                name: "Bypass",
                id_suffix: "bypass",
                range: ParamRange::Stepped { min: 0, max: 1 },
                default: 0.0,
                unit: "",
                smoothing: SmoothingSpec::default(),
            },
        ];
        SPECS
    }
}
```

---

### 3. Built-in Processors

**File:** `engine/crates/wavecraft-dsp/src/builtins/mod.rs`

```rust
pub mod gain;
pub mod filter;
pub mod compressor;
pub mod delay;
pub mod passthrough;

pub use gain::{GainDsp, GainParams};
pub use filter::{FilterDsp, FilterParams, FilterType};
pub use compressor::{CompressorDsp, CompressorParams};
pub use delay::{DelayDsp, DelayParams};
pub use passthrough::{PassthroughDsp, PassthroughParams};
```

**File:** `engine/crates/wavecraft-dsp/src/builtins/gain.rs`

```rust
use crate::traits::{Processor, ProcessorParams, Transport};

/// Parameters for gain processing.
#[derive(ProcessorParams, Default, Clone)]
pub struct GainParams {
    /// Gain level in decibels.
    #[param(range = -24.0..=24.0, default = 0.0, unit = "dB", smoothing = "logarithmic(50)")]
    pub level: f32,
    
    /// Bypass the gain stage.
    #[param(default = false)]
    pub bypass: bool,
}

/// Gain DSP implementation.
pub struct GainDsp {
    sample_rate: f32,
}

impl Default for GainDsp {
    fn default() -> Self {
        Self { sample_rate: 44100.0 }
    }
}

impl Processor for GainDsp {
    type Params = GainParams;
    
    fn process(&mut self, buffer: &mut [&mut [f32]], _transport: &Transport, params: &GainParams) {
        if params.bypass { return; }
        
        let gain = db_to_linear(params.level);
        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                *sample *= gain;
            }
        }
    }
    
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

/// Helper for converting dB to linear gain.
#[inline]
fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}
```

---

### 4. Processor Wrapper Macro

**File:** `engine/crates/wavecraft-core/src/macros.rs`

```rust
/// Create a user-named processor type wrapping a built-in DSP.
/// 
/// # Example
/// ```rust
/// wavecraft_processor!(InputGain => Gain);
/// wavecraft_processor!(OutputGain => Gain);
/// ```
/// 
/// # Expansion
/// ```rust
/// pub struct InputGain(wavecraft_dsp::builtins::GainDsp);
/// 
/// impl Processor for InputGain {
///     type Params = wavecraft_dsp::builtins::GainParams;
///     // delegates to inner
/// }
/// ```
#[macro_export]
macro_rules! wavecraft_processor {
    ($name:ident => Gain) => {
        pub struct $name($crate::__internal::GainDsp);
        
        impl Default for $name {
            fn default() -> Self {
                Self($crate::__internal::GainDsp::default())
            }
        }
        
        impl $crate::Processor for $name {
            type Params = $crate::__internal::GainParams;
            
            fn process(
                &mut self, 
                buffer: &mut [&mut [f32]], 
                transport: &$crate::Transport, 
                params: &Self::Params
            ) {
                self.0.process(buffer, transport, params)
            }
            
            fn set_sample_rate(&mut self, sample_rate: f32) {
                self.0.set_sample_rate(sample_rate)
            }
            
            fn reset(&mut self) {
                self.0.reset()
            }
        }
    };
    
    ($name:ident => Filter) => { /* similar */ };
    ($name:ident => Compressor) => { /* similar */ };
    ($name:ident => Delay) => { /* similar */ };
}
```

---

### 5. Chain Combinator

**File:** `engine/crates/wavecraft-dsp/src/combinators/chain.rs`

```rust
use crate::traits::{Processor, ProcessorParams, Transport};

/// Chain two processors in series: A → B
pub struct Chain<A, B> 
where 
    A: Processor,
    B: Processor,
{
    pub a: A,
    pub b: B,
}

impl<A, B> Chain<A, B>
where
    A: Processor,
    B: Processor,
{
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

/// Combined params for a chain.
pub struct ChainParams<PA, PB> 
where
    PA: ProcessorParams,
    PB: ProcessorParams,
{
    pub a: PA,
    pub b: PB,
}

impl<PA, PB> Default for ChainParams<PA, PB>
where
    PA: ProcessorParams,
    PB: ProcessorParams,
{
    fn default() -> Self {
        Self {
            a: PA::default(),
            b: PB::default(),
        }
    }
}

impl<PA, PB> ProcessorParams for ChainParams<PA, PB>
where
    PA: ProcessorParams,
    PB: ProcessorParams,
{
    fn param_specs() -> &'static [ParamSpec] {
        // TODO: Combine specs from A and B with proper prefixing
        &[]
    }
}

impl<A, B> Processor for Chain<A, B>
where
    A: Processor,
    B: Processor,
{
    type Params = ChainParams<A::Params, B::Params>;
    
    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport, params: &Self::Params) {
        self.a.process(buffer, transport, &params.a);
        self.b.process(buffer, transport, &params.b);
    }
    
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.a.set_sample_rate(sample_rate);
        self.b.set_sample_rate(sample_rate);
    }
    
    fn reset(&mut self) {
        self.a.reset();
        self.b.reset();
    }
}
```

**Chain Macro:**

```rust
/// Chain multiple processors: Chain![A, B, C] → Chain<A, Chain<B, C>>
#[macro_export]
macro_rules! Chain {
    ($first:ty) => { $first };
    ($first:ty, $($rest:ty),+ $(,)?) => {
        $crate::combinators::Chain<$first, Chain![$($rest),+]>
    };
}
```

---

### 6. Plugin Macro (Core)

**File:** `engine/crates/wavecraft-macros/src/plugin.rs` (proc-macro crate)

The `wavecraft_plugin!` macro is the main entry point. It parses the DSL and generates:

1. Plugin struct with meter channels
2. Params struct with nih-plug attributes
3. `Plugin` trait impl
4. `ClapPlugin` trait impl  
5. `Vst3Plugin` trait impl
6. Export macros

**Input syntax:**

```rust
wavecraft_plugin! {
    name: "My Plugin",
    vendor: "My Company",
    url: "https://example.com",           // optional
    email: "contact@example.com",         // optional
    
    // Always use Chain![], even for single processor
    // Chain![A] compiles to just A (zero overhead)
    signal: Chain![
        InputGain { level: 0.0 },
        HighPass { frequency: 80.0 },
        OutputGain { level: -6.0 },
    ],
    
    // Plugin format IDs
    clap_id: "com.mycompany.myplugin",    // optional, generated if missing
    vst3_id: *b"MyPlug0000000000",         // optional, generated if missing
}
```

**Generated code structure:**

```rust
// 1. Plugin struct
pub struct __WavecraftPlugin {
    params: std::sync::Arc<__WavecraftParams>,
    processor: /* DSP type */,
    meter_producer: wavecraft_metering::MeterProducer,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    meter_consumer: std::sync::Arc<std::sync::Mutex<wavecraft_metering::MeterConsumer>>,
}

// 2. Params struct (flattened from processor params)
#[derive(nih_plug::prelude::Params)]
pub struct __WavecraftParams {
    #[id = "my_gain_level"]
    pub my_gain_level: nih_plug::prelude::FloatParam,
    // ... more params
}

// 3. Default impls
impl Default for __WavecraftPlugin { /* ... */ }
impl Default for __WavecraftParams { /* ... */ }

// 4. Plugin trait impl
impl nih_plug::prelude::Plugin for __WavecraftPlugin {
    const NAME: &'static str = "My Plugin";
    const VENDOR: &'static str = "My Company";
    // ... full impl
}

// 5. Format-specific impls
impl nih_plug::prelude::ClapPlugin for __WavecraftPlugin { /* ... */ }
impl nih_plug::prelude::Vst3Plugin for __WavecraftPlugin { /* ... */ }

// 6. Exports
nih_plug::nih_export_clap!(__WavecraftPlugin);
nih_plug::nih_export_vst3!(__WavecraftPlugin);
```

---

### 7. Parameter ID Generation

Parameter IDs are generated from the processor name (snake_case) and param field name:

| Processor | Field | Generated ID | Display Name |
|-----------|-------|--------------|--------------|
| `InputGain` | `level` | `input_gain_level` | "Input Gain Level" |
| `HighPass` | `frequency` | `high_pass_frequency` | "High Pass Frequency" |
| `OutputGain` | `level` | `output_gain_level` | "Output Gain Level" |

**Conversion rules:**
- Type name to snake_case: `InputGain` → `input_gain`
- Concatenate: `{processor}_{field}`
- Display name: Title case with spaces

---

### 8. Crate Structure

```
engine/crates/
├── wavecraft-macros/          # NEW: Proc-macro crate
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── plugin.rs          # wavecraft_plugin! proc-macro
│       └── processor_params.rs # ProcessorParams derive
│
├── wavecraft-dsp/
│   ├── src/
│   │   ├── lib.rs
│   │   ├── traits.rs          # Processor, ProcessorParams traits
│   │   ├── builtins/          # NEW: Built-in processors
│   │   │   ├── mod.rs
│   │   │   ├── gain.rs
│   │   │   ├── filter.rs
│   │   │   ├── compressor.rs
│   │   │   ├── delay.rs
│   │   │   └── passthrough.rs
│   │   └── combinators/       # NEW: Chain, Parallel
│   │       ├── mod.rs
│   │       ├── chain.rs
│   │       └── parallel.rs
│
├── wavecraft-core/
│   └── src/
│       ├── lib.rs
│       ├── prelude.rs         # Re-exports everything
│       └── macros.rs          # wavecraft_processor! declarative macro
```

---

## API Design

### Tier 1: Minimal Plugin (Built-in DSP)

```rust
use wavecraft::prelude::*;

wavecraft_processor!(MyGain => Gain);

wavecraft_plugin! {
    name: "Simple Gain",
    signal: Chain![
        MyGain { level: 0.0 },
    ],
}
```

### Tier 2: Chained Built-in DSP

```rust
use wavecraft::prelude::*;

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

### Tier 3: Custom Processor with Built-in Params

```rust
use wavecraft::prelude::*;

#[derive(Processor)]
#[wavecraft(params = GainParams)]  // Use built-in params
pub struct MySaturator;

impl MySaturator {
    fn process_impl(&mut self, buffer: &mut [&mut [f32]], params: &GainParams) {
        // Custom saturation using params.level as drive
        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                let drive = 1.0 + params.level.abs() / 6.0;
                *sample = (*sample * drive).tanh();
            }
        }
    }
}

wavecraft_plugin! {
    name: "Saturator",
    signal: Chain![
        MySaturator { level: 6.0 },
    ],
}
```

### Tier 4: Fully Custom Processor

```rust
use wavecraft::prelude::*;

#[derive(ProcessorParams, Default)]
pub struct MySynthParams {
    #[param(range = 20.0..=20000.0, default = 440.0, unit = "Hz")]
    pub frequency: f32,
    
    #[param(range = 0.0..=1.0, default = 0.5)]
    pub pulse_width: f32,
}

pub struct MySynth {
    phase: f32,
    sample_rate: f32,
}

impl Default for MySynth {
    fn default() -> Self {
        Self { phase: 0.0, sample_rate: 44100.0 }
    }
}

impl Processor for MySynth {
    type Params = MySynthParams;
    
    fn process(&mut self, buffer: &mut [&mut [f32]], _transport: &Transport, params: &MySynthParams) {
        let phase_inc = params.frequency / self.sample_rate;
        
        for channel in buffer.iter_mut() {
            for sample in channel.iter_mut() {
                // Pulse wave
                *sample = if self.phase < params.pulse_width { 1.0 } else { -1.0 };
                self.phase = (self.phase + phase_inc) % 1.0;
            }
        }
    }
    
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
    
    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

wavecraft_plugin! {
    name: "Simple Synth",
    signal: Chain![
        MySynth { frequency: 440.0, pulse_width: 0.5 },
    ],
}
```

---

## Error Handling

### Compile-Time Errors

The macro should emit helpful errors:

```rust
// Missing required field
wavecraft_plugin! {
    vendor: "My Company",  // ERROR: missing `name` field
    signal: Chain![
        MyGain { level: 0.0 },
    ],
}
// Error: wavecraft_plugin! requires `name` field

// Invalid param field
wavecraft_plugin! {
    name: "Test",
    signal: Chain![
        MyGain { gain: 0.0 },  // ERROR: GainParams has `level`, not `gain`
    ],
}
// Error: MyGain does not have parameter `gain`. Available: level, bypass

// Duplicate processor without distinct types
wavecraft_plugin! {
    name: "Test",
    signal: Chain![
        MyGain { level: 0.0 }, 
        MyGain { level: -6.0 },  // ERROR: duplicate type
    ],
}
// Error: Processor type `MyGain` appears twice. Define separate types:
//        wavecraft_processor!(InputGain => Gain);
//        wavecraft_processor!(OutputGain => Gain);
```

---

## UI Integration

Parameter IDs are stable and predictable, enabling direct UI binding:

```tsx
// React UI (auto-generated IDs)
<ParameterSlider id="input_gain_level" />
<ParameterSlider id="high_pass_frequency" />
<ParameterSlider id="dynamics_threshold" />
<ParameterSlider id="output_gain_level" />
```

---

## Migration Path

Existing manual plugins continue to work. The macro is purely additive.

Users can incrementally adopt:
1. Start with `wavecraft_plugin!` for new plugins
2. Gradually refactor existing plugins if desired
3. Mix: use macro for simple parts, manual for complex

---

## Testing Strategy

### Unit Tests

1. **Processor trait**: Test built-in processors in isolation
2. **Chain combinator**: Test chaining works correctly
3. **Param generation**: Test ID generation and metadata

### Integration Tests

1. **Macro expansion**: Use `cargo expand` to verify generated code
2. **Plugin loading**: Test generated plugins load in nih-plug test harness
3. **Parameter sync**: Test UI param IDs match generated IDs

### Template Tests

1. **Template builds**: Verify `wavecraft-plugin-template` compiles
2. **Template runs**: Verify bundled plugin loads in DAW

---

## Dependencies

### New Crate: `wavecraft-macros`

```toml
[package]
name = "wavecraft-macros"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[dependencies]
syn = { version = "2", features = ["full", "parsing", "extra-traits"] }
quote = "1"
proc-macro2 = "1"
convert_case = "0.6"  # For snake_case/PascalCase conversion
```

### Updated: `wavecraft-dsp/Cargo.toml`

```toml
[dependencies]
wavecraft-macros = { path = "../wavecraft-macros" }
```

### Updated: `wavecraft-core/Cargo.toml`

```toml
[dependencies]
wavecraft-dsp = { path = "../wavecraft-dsp" }
wavecraft-macros = { path = "../wavecraft-macros" }
```

---

## Decisions

1. **Parallel combinator**: **Deferred** — Not included in initial release. Chain covers most use cases.

2. **Enum parameters**: **Included** — Support for enum params like `FilterType::LowPass`.

3. **Parameter groups in UI**: **Included** — Params grouped by processor in collapsible UI sections.

4. **Preset compatibility**: **Documented as breaking** — Renaming processors changes param IDs and breaks presets. This is documented behavior.

---

## References

- [nih-plug documentation](https://github.com/robbert-vdh/nih-plug)
- [Rust procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html)
- [syn crate](https://docs.rs/syn/latest/syn/)
- [quote crate](https://docs.rs/quote/latest/quote/)
