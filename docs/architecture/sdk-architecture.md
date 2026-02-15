# SDK Architecture

Wavecraft is designed as a **Developer SDK** that enables other developers to build VST3/CLAP audio plugins with Rust + React. This document covers the SDK's crate structure, distribution model, npm packages, public API surface, and design principles.

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview, component design, data flows
- [Declarative Plugin DSL](./declarative-plugin-dsl.md) — DSL architecture and macro system
- [Coding Standards — Rust](./coding-standards-rust.md) — Rust conventions and patterns
- [Development Workflows](./development-workflows.md) — Build commands, dev servers, CI/CD
- [SDK Getting Started](../guides/sdk-getting-started.md) — Building plugins with Wavecraft SDK

---

## SDK Distribution Model

Wavecraft distributes its SDK through two channels:

1. **Rust Crates** — Audio engine, DSP, and plugin framework (via Git tags, crates.io in Phase 2)
2. **npm Packages** — React UI components, IPC hooks, and utilities (via npm)

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         WAVECRAFT SDK DISTRIBUTION                              │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   ┌───────────────────────┐       ┌───────────────────────┐                     │
│   │  EMBEDDED TEMPLATE    │       │    GIT-ONLY           │                     │
│   │  sdk-template/        │       │                       │                     │
│   │                       │       │  wavecraft-nih_plug   │  ← User depends     │
│   │  ├── engine/          │──────▶│  (Cargo rename:       │    (git tag)        │
│   │  │   └── Cargo.toml   │       │   wavecraft = {...})  │                     │
│   │  │   wavecraft = {    │       │                       │                     │
│   │  │     package =      │       └───────────┬───────────┘                     │
│   │  │     "wavecraft-    │                   │                                 │
│   │  │     nih_plug"...}  │                   │ depends on                      │
│   │  │                    │                   ▼                                 │
│   │  │                    │       ┌───────────────────────┐                     │
│   │  │                    │       │    CRATES.IO          │                     │
│   │  │                    │       │                       │                     │
│   │  │                    │       │  wavecraft-core       │  ← Publishable      │
│   │  │                    │       │  wavecraft-dsp        │                     │
│   │  │                    │       │  wavecraft-protocol   │                     │
│   │  │                    │       │  wavecraft-metering   │                     │
│   │  │                    │       │  wavecraft-bridge     │                     │
│   │  │                    │       │  wavecraft-macros     │                     │
│   │  │                    │       └───────────────────────┘                     │
│   │  │                    │                                                     │
│   │  │                    │       ┌───────────────────────┐                     │
│   │  ├── ui/              │       │        NPM            │                     │
│   │  │   └── package.json │──────▶│                       │                     │
│   │  │                    │       │  @wavecraft/core      │  ← UI Framework    │
│   │  └── README.md        │       │  @wavecraft/components│    (user depends)  │
│   └───────────────────────┘       └───────────────────────┘                     │
│              │                                                                  │
│              │  User customizes:                                                │
│              │  - DSP code (Processor trait impl)                               │
│              │  - Parameters (ProcessorParams derive macro)                     │
│              │  - UI components (React, using @wavecraft/components)            │
│              ▼                                                                  │
│   ┌───────────────────────┐                                                     │
│   │   USER'S PLUGIN       │                                                     │
│   │   my-awesome-plugin   │                                                     │
│   └───────────────────────┘                                                     │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## SDK Crate Structure (Rust)

All SDK crates use the `wavecraft-*` naming convention for clear identification:

| Crate                  | Purpose                                                                                                                                                                                                                                                                                       | Publishable                     | User Interaction                                                                                         |
| ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `wavecraft-nih_plug`   | nih-plug integration, WebView editor, plugin exports                                                                                                                                                                                                                                          | ❌ Git only                     | **Primary dependency** — users import via Cargo rename: `wavecraft = { package = "wavecraft-nih_plug" }` |
| `wavecraft-core`       | Core SDK types, declarative macros, no nih_plug dependency                                                                                                                                                                                                                                    | ✅ crates.io                    | Re-exported via wavecraft-nih_plug                                                                       |
| `wavecraft-macros`     | Procedural macros: `ProcessorParams` derive, `wavecraft_plugin!` proc-macro                                                                                                                                                                                                                   | ✅ crates.io                    | Used indirectly via wavecraft-nih_plug                                                                   |
| `wavecraft-protocol`   | IPC contracts, parameter types, JSON-RPC definitions, FFI vtable contract (`DevProcessorVTable`)                                                                                                                                                                                              | ✅ crates.io                    | Implements `ParamSet` trait                                                                              |
| `wavecraft-bridge`     | IPC handler, `ParameterHost` trait, `PluginParamLoader` (dlopen + parameter metadata + processor metadata + dev vtable loading)                                                                                                                                                               | ✅ crates.io                    | CLI uses for plugin loading                                                                              |
| `wavecraft-metering`   | Real-time safe SPSC ring buffer for audio → UI metering                                                                                                                                                                                                                                       | ✅ crates.io                    | Uses `MeterProducer` in DSP                                                                              |
| `wavecraft-dsp`        | DSP primitives, `Processor` trait, built-in processors                                                                                                                                                                                                                                        | ✅ crates.io                    | Implements `Processor` trait                                                                             |
| `wavecraft-processors` | Reusable SDK-owned processor implementations (for example, `Oscillator`) used by templates and plugin projects; complements `wavecraft-dsp` primitives                                                                                                                                        | ✅ crates.io                    | Imported by user plugins when they want SDK-provided processors                                          |
| `wavecraft-dev-server` | Unified dev server at `dev-server/` (repo root): WebSocket server, `DevAudioProcessor` trait, `FfiProcessor` wrapper, `AudioServer` (full-duplex), `AtomicParameterBridge`, hot-reload, file watching. Feature-gated audio (`default = ["audio"]`). CLI uses with `default-features = false`. | ❌ Standalone (publish = false) | CLI uses for dev mode                                                                                    |
| `sdk-template`         | Canonical plugin scaffold at repository root. Used both for CLI template embedding and SDK-mode development (`cargo xtask dev` after running `scripts/setup-dev-template.sh`).                                                                                                                | ❌ Internal scaffold            | Source of truth for generated projects and SDK contributor workflow                                      |

> **Why the split?** The `nih_plug` crate cannot be published to crates.io (it has unpublished dependencies). By isolating nih_plug integration in `wavecraft-nih_plug` (git-only), all other crates become publishable. User projects depend on `wavecraft-nih_plug` via git tag, while the ecosystem gains crates.io discoverability for the rest of the SDK. The `wavecraft-dev-server` crate lives at the repository root (`dev-server/`) because it bridges CLI and engine concerns and is never distributed to end users — it's an internal development tool.

## npm Package Structure (UI)

The UI SDK is distributed as npm packages, enabling standard JavaScript/TypeScript dependency management:

| Package                 | Purpose                                   | Exports                                                                                                                                                                                                                                                                          |
| ----------------------- | ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `@wavecraft/core`       | IPC bridge, React hooks, utilities, types | `useParameter`, `useAllParameters`, `useMeterFrame`, `useAudioStatus`, `useHasProcessor`, `useAvailableProcessors`, `IpcBridge`, `Logger`, `ParameterId`, `ParameterIdMap`, `ProcessorId`, `ProcessorIdMap`, `AudioRuntimeStatus`, `AudioRuntimePhase`, `AudioDiagnostic`, types |
| `@wavecraft/components` | Pre-built React components                | `Meter`, `ParameterSlider`, `ParameterGroup`, `ParameterToggle`, `VersionBadge`, `ConnectionStatus`, `LatencyMonitor`, `ResizeHandle`, `ResizeControls`                                                                                                                          |

**Subpath Exports:**

The `@wavecraft/core` package supports subpath exports for tree-shaking and avoiding side effects:

```typescript
// Main entry — full SDK (IPC, hooks, utilities)
import { useParameter, IpcBridge, Logger } from '@wavecraft/core';

// Subpath: Pure audio math utilities (no IPC side effects)
import { linearToDb, dbToLinear, getMeterFrame } from '@wavecraft/core/meters';
```

**Package Dependencies:**

```
@wavecraft/components
    └── @wavecraft/core (peer dependency)
            └── react (peer dependency)
```

**User Plugin Usage:**

```typescript
// In user's plugin UI
import { useParameter, useAllParameters, Logger } from '@wavecraft/core';
import { Meter, ParameterSlider, ParameterGroup } from '@wavecraft/components';

function MyPluginUI() {
  const allParams = useAllParameters();
  return (
    <ParameterGroup name="Input">
      {allParams.map(p => <ParameterSlider key={p.id} parameter={p} />)}
    </ParameterGroup>
  );
}
```

**Runtime Status Contracts (by design):**

- Transport/runtime connectivity (for reconnect UX) is exposed separately from audio runtime readiness.
- `useConnectionStatus()` answers: _"Is the transport connected?"_
- `useAudioStatus()` answers: _"Is audio runtime initialized/running, and if not, why?"_
- Pre-1.0, this contract is strict current-version required: incompatible/missing runtime status expectations fail fast with actionable diagnostics instead of silent compatibility fallbacks.

### Processor presence contract (v1)

Processor presence in v1 is codegen-first and local: `wavecraft start` generates `ui/src/generated/processors.ts`, which registers and types available processors at startup for `@wavecraft/core` hooks (`useHasProcessor`, `useAvailableProcessors`). No new runtime JSON-RPC endpoint is introduced for processor presence in v1.

## Public API Surface (Rust)

The SDK exposes a minimal, stable API through the `wavecraft::prelude` module (where `wavecraft` is the Cargo rename for `wavecraft-nih_plug`):

```rust
// wavecraft::prelude re-exports (via wavecraft-nih_plug)
pub use nih_plug::prelude::*;  // From wavecraft-nih_plug
pub use wavecraft_dsp::{Processor, ProcessorParams, Transport, builtins};
pub use wavecraft_processors::{Oscillator, OscillatorParams};
pub use wavecraft_protocol::{ParamId, ParameterInfo, ParameterType, db_to_linear};
pub use wavecraft_metering::{MeterConsumer, MeterFrame, MeterProducer, create_meter_channel};
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use crate::editor::WavecraftEditor;
pub use crate::util::calculate_stereo_meters;

// Declarative macros (from wavecraft-core, re-exported)
pub use wavecraft_core::{wavecraft_processor, wavecraft_plugin};
```

> **Note:** The `wavecraft-nih_plug` crate also exports a hidden `__nih` module containing all nih_plug types needed by proc-macro generated code. This allows the `wavecraft_plugin!` macro to reference types like `Plugin`, `Params`, and `FloatParam` through a stable path.

**Key Traits:**

1. **`Processor`** — Core DSP abstraction for audio processing:

   ```rust
   pub trait Processor: Send + 'static {
       type Params: ProcessorParams + Default + Send + Sync + 'static;

       fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport, params: &Self::Params);
       fn set_sample_rate(&mut self, _sample_rate: f32) {}
       fn reset(&mut self) {}
   }
   ```

2. **`ProcessorParams`** — Parameter metadata for runtime discovery (typically via `#[derive(ProcessorParams)]`):

   ```rust
   pub trait ProcessorParams: Default + Send + Sync + 'static {
       fn param_specs() -> &'static [ParamSpec];
   }
   ```

3. **`ParameterHost`** — Backend trait for parameter management (framework-provided):
   ```rust
   pub trait ParameterHost: Send + Sync {
       fn get_parameters(&self) -> Vec<ParameterInfo>;
       fn get_parameter(&self, id: &str) -> Option<ParameterInfo>;
       fn set_parameter(&self, id: &str, value: f32) -> bool;
   }
   ```

**Macros:**

- **`wavecraft_processor!`** — Creates named wrappers around built-in DSP processors (not for custom types):

  ```rust
  wavecraft_processor!(InputGain => Gain);
  wavecraft_processor!(OutputGain => Gain);
  ```

- **`wavecraft_plugin!`** — Generates complete plugin implementation from minimal DSL:

  ```rust
  wavecraft_plugin! {
      name: "My Plugin",
      vendor: "Wavecraft",
      signal: SignalChain![InputGain, MyProcessor, OutputGain],
  }
  ```

  Custom processors go directly in `SignalChain![]` — no wrapper needed.

- **`#[derive(ProcessorParams)]`** — Auto-generates parameter metadata from struct definition:

  ```rust
  use wavecraft::ProcessorParams; // derive macro import

  #[derive(ProcessorParams, Default)]
  struct MyParams {
      #[param(range = "-60.0..=24.0", default = 0.0, unit = "dB")]
      gain: f32,
  }
  ```

- **`wavecraft_params!`** — Declarative parameter definition (legacy, prefer derive macro):
  ```rust
  wavecraft_params! {
      Gain: { id: 0, name: "Gain", range: -24.0..=24.0, default: 0.0, unit: "dB" },
      Mix: { id: 1, name: "Mix", range: 0.0..=1.0, default: 1.0, unit: "%" },
  }
  ```

## User Project Structure

The template provides a standardized project structure:

```
my-plugin/
├── engine/
│   ├── Cargo.toml           ← Depends on wavecraft-nih_plug (git tag, Cargo rename)
│   └── src/
│       ├── lib.rs           ← Plugin assembly (signal chain + metadata)
│       └── processors/      ← Custom DSP processors
│           ├── mod.rs        ← Module exports
│           └── example_processor.rs ← Minimal custom processor example
│
├── ui/
│   ├── package.json         ← Depends on @wavecraft/core + @wavecraft/components
│   ├── vite.config.ts
│   └── src/
│       ├── App.tsx          ← User's custom UI (imports from npm packages)
│       └── generated/       ← Build artifacts (gitignored)
│           ├── parameters.ts ← ParameterId types (auto-generated by `wavecraft start`)
│           └── processors.ts ← Processor presence/types (auto-generated by `wavecraft start`)
│
└── xtask/                   ← Build automation (bundle, dev, etc.)
```

**Template `Cargo.toml` Dependencies:**

```toml
[dependencies]
# Single SDK dependency — Cargo rename gives us `use wavecraft::prelude::*`
wavecraft = { package = "wavecraft-nih_plug", git = "https://github.com/RonHouben/wavecraft", tag = "wavecraft-cli-v0.7.1" }
```

**Template `package.json` Dependencies:**

```json
{
  "dependencies": {
    "@wavecraft/core": "^0.7.0",
    "@wavecraft/components": "^0.7.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  }
}
```

**Template UI Example:**

```tsx
// my-plugin/ui/src/App.tsx
import { useAllParameters, useMeterFrame } from '@wavecraft/core';
import { Meter, ParameterSlider, VersionBadge } from '@wavecraft/components';

export function App() {
  const params = useAllParameters();
  const meters = useMeterFrame();

  return (
    <div>
      <VersionBadge />
      <Meter leftDb={meters?.leftDb} rightDb={meters?.rightDb} />
      {params.map((p) => (
        <ParameterSlider key={p.id} parameter={p} />
      ))}
    </div>
  );
}
```

### Ownership Boundary: SDK vs User Processors

- **SDK-owned**: `engine/crates/wavecraft-processors` contains reusable processors maintained by Wavecraft and versioned with the SDK.
- **User-owned**: `my-plugin/engine/src/processors/` in generated projects contains plugin-specific processors authored by plugin developers.
- **Relationship**: User plugins can combine both in `SignalChain![]` (SDK processors from `wavecraft-processors` + custom project-local processors).

## SDK Design Principles

1. **Minimal Boilerplate** — Users implement traits and use macros; framework handles nih-plug integration, WebView setup, and IPC.

2. **Clear Boundaries** — SDK code vs user code is explicit through crate structure and trait contracts.

3. **Composition over Inheritance** — Users compose their plugins from framework components rather than subclassing.

4. **Type Safety** — Compile-time guarantees through Rust's type system; generic `WavecraftEditor<P: Params>` works with any parameter type.

5. **Real-Time Safety by Design** — DSP traits enforce the contract; metering uses proven lock-free patterns.

6. **Type-Safe Parameter IDs** — `wavecraft start` generates TypeScript module augmentation that narrows `ParameterId` from `string` to a literal union of the plugin's actual parameter IDs, providing IDE autocompletion and compile-time type checking with zero developer effort.

## Testability & Environment

- Avoid tests that manipulate global state (e.g., environment variables) directly. Code that depends on environment configuration should separate construction from environment reading to make it testable (example: `SigningConfig::new()` + `SigningConfig::from_env()`).
- Tests should prefer dependency injection and pure constructors to remain deterministic and parallelizable; use of test-only serialisation (e.g., `serial_test`) is discouraged as a primary fix for global-state tests.
- The `xtask::sign` commands accept a `SigningConfig` to allow production behavior (`from_env()`) while enabling pure, side-effect-free unit tests via `new()`.
