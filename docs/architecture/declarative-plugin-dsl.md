# Declarative Plugin DSL

Wavecraft provides a declarative domain-specific language (DSL) for defining plugins with minimal boilerplate. The DSL reduces plugin definitions from ~190 lines of manual implementation to ~9 lines.

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview, component design, data flows
- [SDK Architecture](./sdk-architecture.md) — Crate structure and distribution model
- [Coding Standards — Rust](./coding-standards-rust.md) — Rust conventions and patterns

---

## DSL Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          DECLARATIVE PLUGIN DSL                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   User Code (9 lines)              Generated Code (~400 lines)                  │
│   ────────────────────             ────────────────────────────                 │
│                                                                                 │
│   wavecraft_processor!(            → Plugin struct with metering                │
│       InputGain => Gain            → Default impl with meter channel            │
│   );                               → Plugin trait impl (name, vendor, I/O)      │
│                                    → Params struct via ProcessorParams          │
│   wavecraft_plugin! {              → process() with DSP routing                 │
│       name: "My Plugin",           → VST3Plugin impl (class ID)                 │
│       signal: SignalChain![        → ClapPlugin impl (CLAP ID)                  │
│           InputGain],              → nih_export_vst3!() (#[cfg] gated)          │
│   }                                → nih_export_clap!() (#[cfg] gated)          │
│                                    → FFI vtable export (always available)       │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Macro System

The DSL uses a two-layer macro system:

1. **`wavecraft_processor!`** (declarative macro) — Wraps **built-in** DSP processors only:

   ```rust
   wavecraft_processor!(InputGain => Gain);
   wavecraft_processor!(Bypass => Passthrough);
   ```

   - Creates newtype wrappers around built-in processors (`Gain`, `Passthrough`)
   - Delegates `Processor` trait implementation
   - Maintains type distinction for compile-time safety (wrapper name becomes parameter-ID prefix)
   - **Not for custom processors** — types implementing `Processor` directly go straight into `SignalChain![]`

2. **`wavecraft_plugin!`** (proc-macro) — Generates complete plugin implementation:

   ```rust
   wavecraft_plugin! {
       name: "My Plugin",
       signal: SignalChain![InputGain],
   }
   ```

   > **Note:** `vendor` and `url` are derived from `Cargo.toml` metadata. `email` is not exposed as a macro property and defaults internally to an empty string. The `signal` field requires `SignalChain![]` wrapper — bare processor names are not accepted.

   In addition to the nih-plug `Plugin` implementation, this macro also generates:
   - `nih_export_vst3!()` and `nih_export_clap!()` — Conditionally compiled with `#[cfg(not(feature = "_param-discovery"))]`. This allows `wavecraft start` to load the dylib for parameter discovery without triggering nih-plug's static initializers (which cause macOS `AudioComponentRegistrar` hangs during `dlopen`).
   - `wavecraft_get_params_json` / `wavecraft_free_string` — FFI exports for parameter discovery (always available)
   - `wavecraft_dev_create_processor` — FFI vtable export returning a `DevProcessorVTable` for dev audio processing (always available, see [Dev Audio via FFI](./development-workflows.md#dev-audio-via-ffi))

   All generated `extern "C"` functions use `catch_unwind` to prevent panics from unwinding across the FFI boundary.

3. **`#[derive(ProcessorParams)]`** — Auto-generates parameter metadata:

   ```rust
   use wavecraft::prelude::*;
   use wavecraft::ProcessorParams;  // derive macro (separate from trait in prelude)

   #[derive(ProcessorParams, Default)]
   struct GainParams {
       #[param(range = "-60.0..=24.0", default = 0.0, unit = "dB", group = "Input")]
       gain: f32,
   }
   ```

   > **Import note:** `use wavecraft::prelude::*` brings in the `ProcessorParams` _trait_.
   > The `#[derive(ProcessorParams)]` _derive macro_ requires `use wavecraft::ProcessorParams;` — trait and derive macro coexist in different namespaces.

## Parameter Runtime Discovery

The DSL supports runtime parameter discovery via the `ProcessorParams` trait:

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                       RUNTIME PARAMETER DISCOVERY                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   #[derive(ProcessorParams)]          ProcessorParams::param_specs()            │
│   struct GainParams {                 ──────────────────────────────►           │
│       #[param(range="...", unit="dB")]    &'static [ParamSpec]                  │
│       gain: f32,                                                                │
│   }                                   ┌──────────────────────────┐              │
│                                       │ ParamSpec {              │              │
│                                       │   name: "Gain",          │              │
│                                       │   id_suffix: "gain",     │              │
│                                       │   range: Linear {...},   │              │
│                                       │   default: 0.0,          │              │
│                                       │   unit: "dB",            │              │
│                                       │   group: Some("Input"),  │              │
│                                       │ }                        │              │
│                                       └──────────────────────────┘              │
│                                                   │                             │
│                                                   ▼                             │
│                                       ┌──────────────────────────┐              │
│                                       │ IPC: getAllParameters    │              │
│                                       │ → ParameterInfo[]        │              │
│                                       │ → React UI renders       │              │
│                                       │   grouped parameters     │              │
│                                       └──────────────────────────┘              │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Parameter ID Prefix Generation

When `SignalChain![]` contains multiple processors, each processor's parameters are namespaced with an ID prefix derived from the processor type name (lowercased):

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                       PARAMETER ID NAMESPACING                                  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   wavecraft_processor!(InputGain => Gain);                                      │
│   wavecraft_processor!(OutputGain => Gain);                                     │
│                                                                                 │
│   wavecraft_plugin! {                                                           │
│       name: "My Plugin",                                                        │
│       signal: SignalChain![InputGain, OutputGain],                               │
│   }                                                                             │
│                                                                                 │
│   Both InputGain and OutputGain wrap the same Gain processor                    │
│   (which has a `gain` parameter), but get distinct IDs:                         │
│                                                                                 │
│   InputGain  → prefix "inputgain"  → parameter ID: "inputgain_gain"            │
│   OutputGain → prefix "outputgain" → parameter ID: "outputgain_gain"           │
│                                                                                 │
│   Custom processors follow the same rule:                                       │
│   Oscillator → prefix "oscillator" → "oscillator_frequency",                   │
│                                       "oscillator_level"                        │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

This namespacing is performed by the `wavecraft_plugin!` proc-macro at compile time. The `type_prefix()` function converts the type name to snake_case for the prefix. These prefixed IDs are what appear in the generated `ParameterId` TypeScript union type (see below).

## TypeScript Parameter ID Codegen

The `wavecraft start` command extracts parameter IDs from the compiled plugin and generates a TypeScript module augmentation file at `ui/src/generated/parameters.ts`:

```typescript
// Auto-generated by `wavecraft start` — DO NOT EDIT
declare module '@wavecraft/core' {
  interface ParameterIdMap {
    __wavecraft_internal_augmented__: true;
    inputgain_gain: true;
    outputgain_gain: true;
  }
}

export {};
```

This augments the `ParameterIdMap` interface in `@wavecraft/core`, causing the `ParameterId` conditional type to resolve to `'inputgain_gain' | 'outputgain_gain'` instead of `string`. The result: IDE autocompletion and compile-time type checking for all `useParameter()` calls, `ParameterClient` methods, and component props — with zero developer effort.

The file is regenerated automatically on Rust source changes during development (via the hot-reload pipeline).

## UI Parameter Grouping

Parameters can be organized into groups for UI organization:

```typescript
// React hook: useParameterGroups()
const { groupedParams, ungroupedParams } = useParameterGroups();

// Renders ParameterGroup components for each group
// Ungrouped parameters appear in default "Parameters" section
```

The `ParameterGroup` component renders parameters within a named section, improving UI organization for plugins with many parameters.

## Design Decisions

1. **Compile-Time Code Generation** — All boilerplate is generated at compile time, not runtime. This means zero runtime overhead and full IDE support for generated types.

2. **Deterministic VST3 IDs** — The macro generates VST3 class IDs by hashing the plugin name and vendor. This ensures consistent IDs without manual management.

3. **Type-Safe Signal Routing** — The `signal` field accepts any type implementing `Processor`. This enables future composition via `Chain![]` or custom processor types.

4. **Optional Group Field** — The `group` parameter attribute is optional, maintaining backward compatibility while enabling UI organization.

5. **Runtime Discovery over Static Embedding** — Parameters are discovered at runtime via trait methods rather than embedded in static metadata. This simplifies the macro implementation and enables dynamic parameter sets.

## Achieved Code Reduction

| Before (Manual)       | After (DSL)  | Reduction |
| --------------------- | ------------ | --------- |
| Plugin struct         | Generated    | -20 lines |
| Default impl          | Generated    | -15 lines |
| Plugin trait impl     | Generated    | -80 lines |
| Params struct         | Generated    | -40 lines |
| VST3/CLAP impls       | Generated    | -20 lines |
| Export macros         | Generated    | -5 lines  |
| **Total: ~190 lines** | **~9 lines** | **95%**   |

## Known Limitations and Trade-offs (v0.9.0)

### Parameter Sync in DSL Plugins

The `wavecraft_plugin!` macro generates plugins where the `Processor::process()` method always receives **default parameter values**. This is a conscious design trade-off to keep the macro simple while supporting the most common use cases.

**What Works**:

- ✅ Host automation (parameters visible in DAW, automation recorded)
- ✅ UI parameter display and editing (sliders, knobs work correctly)
- ✅ IPC parameter sync (UI ↔ Host communication)
- ✅ Parameter values visible in DAW mixer/automation lanes

**What Doesn't Work**:

- ❌ DSP code reading parameter values in `process()`
- ❌ Parameter-driven effects (gain, filters, modulation)
- ❌ Parameter automation affecting audio output

**Example of the Limitation**:

```rust
#[derive(ProcessorParams, Default)]
struct GainParams {
    #[param(range = "-60.0..=24.0", default = 0.0)]
    gain: f32,
}

impl Processor for MyGain {
    type Params = GainParams;

    fn process(&mut self, buffer: &mut [&mut [f32]], ..., params: &Self::Params) {
        // ⚠️ params.gain will ALWAYS be 0.0 (default) in DSL-generated plugins
        // Host automation and UI updates don't reach here
        let gain_linear = db_to_linear(params.gain); // Always 1.0 (0 dB)
    }
}
```

**Workaround**: For parameter-driven DSP, implement the `Plugin` trait directly instead of using the macro:

```rust
// Manual implementation - full parameter control
impl Plugin for MyPlugin {
    fn process(&mut self, buffer: &mut Buffer, ...) {
        // Direct access to nih-plug parameters
        let gain_db = self.params.gain.value();
        let gain_linear = db_to_linear(gain_db);

        // Apply gain to audio
        for channel in buffer.iter_samples() {
            for sample in channel {
                *sample *= gain_linear;
            }
        }
    }
}
```

**Why This Limitation Exists**:

The macro bridges two parameter representations:

1. **nih-plug parameters** — Runtime trait objects (`FloatParam`, `BoolParam`) with atomic storage for host automation
2. **Processor parameters** — Plain typed structs (`f32`, `bool`) for DSP code

Full bidirectional sync requires:

- Parse user's parameter struct at compile time (complex proc-macro logic)
- Generate field-by-field conversion code
- Handle nested parameters, groups, and conditionals
- Manage parameter smoothing and sample-accurate automation

This is solvable but adds significant complexity (~1500 LOC proc-macro code vs current ~500 LOC). The limitation is acceptable for v0.9.0 because:

1. **Clear Workaround**: Manual `Plugin` implementation is well-documented and straightforward
2. **Target Use Case**: The DSL is designed for minimal plugins (test plugins, demos, fixed DSP)
3. **Incremental Release**: Full parameter sync can be added in 0.10.0 without breaking the existing API

**When to Use DSL vs Manual Implementation**:

| Use Case                            | Recommendation                       |
| ----------------------------------- | ------------------------------------ |
| Test plugins, demos                 | ✅ Use `wavecraft_plugin!` macro     |
| Fixed DSP (no parameter control)    | ✅ Use `wavecraft_plugin!` macro     |
| Passthrough, analyzers, visualizers | ✅ Use `wavecraft_plugin!` macro     |
| Gain, EQ, compression, modulation   | ❌ Implement `Plugin` trait manually |
| Custom parameter smoothing          | ❌ Implement `Plugin` trait manually |
| Sample-accurate automation          | ❌ Implement `Plugin` trait manually |

**Roadmap**: Full parameter sync is tracked in GitHub issues and targeted for a future release (0.10.0 or 1.0.0).
