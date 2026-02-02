# Low-Level Design: Developer SDK

## Overview

This document defines the architecture for transforming VstKit from an internal project into a reusable **Developer SDK** that enables other developers to build VST3/CLAP audio plugins with Rust + React.

**Related Documents:**
- [User Stories](./user-stories.md) — Requirements and acceptance criteria
- [High-Level Design](../../architecture/high-level-design.md) — Current architecture
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions

---

## 1. SDK Packaging Strategy

### 1.1 Analysis of Options

| Approach | Description | Pros | Cons |
|----------|-------------|------|------|
| **Template Repository** | Users clone/fork a starter repo | Simple, familiar (nih-plug model), full control | Manual updates, version drift, no central management |
| **CLI Scaffolding** | `cargo vstkit new my-plugin` generates project | Great DX, version-aware, customizable | Complex to build/maintain, another binary to distribute |
| **Crates.io Library** | Users add `vstkit = "0.4"` to Cargo.toml | Standard Rust workflow, automatic updates | UI layer doesn't fit (npm/Vite), complex for beginners |
| **Hybrid (Recommended)** | Template repo + vstkit-core crate + xtask tooling | Best of both, incremental adoption | Two distribution channels to maintain |

### 1.2 Recommended Approach: Hybrid Model

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           VSTKIT SDK DISTRIBUTION                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   ┌───────────────────────┐     ┌───────────────────────┐                       │
│   │  TEMPLATE REPOSITORY  │     │    CRATES.IO / GIT    │                       │
│   │  vstkit-plugin-template│     │                       │                       │
│   │                        │     │  vstkit-core (lib)    │  ← Framework code     │
│   │  ├── engine/          │────▶│  vstkit-bridge        │    (user depends on)  │
│   │  │   └── Cargo.toml   │     │  vstkit-protocol      │                       │
│   │  │       deps: vstkit-*│     │  vstkit-metering      │                       │
│   │  │                    │     │  vstkit-dsp           │                       │
│   │  ├── ui/              │     │  vstkit-xtask (bin)   │  ← Build tooling      │
│   │  │   └── package.json │     │                       │                       │
│   │  │       @vstkit/ui   │────▶│  @vstkit/ui (npm)     │  ← UI components      │
│   │  │                    │     │  @vstkit/ipc (npm)    │                       │
│   │  └── README.md        │     │                       │                       │
│   │      Getting started  │     └───────────────────────┘                       │
│   └───────────────────────┘                                                     │
│              │                                                                  │
│              │  User customizes:                                                │
│              │  - DSP code (src/dsp/)                                           │
│              │  - UI components (ui/src/)                                       │
│              │  - Plugin metadata (Cargo.toml)                                  │
│              ▼                                                                  │
│   ┌───────────────────────┐                                                     │
│   │   USER'S PLUGIN       │                                                     │
│   │   my-awesome-plugin   │                                                     │
│   └───────────────────────┘                                                     │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

**Rationale:**
1. **Template repository** provides the fastest path from zero to working plugin
2. **Crates.io packages** enable proper dependency management and updates
3. **npm packages** solve the UI distribution problem (React doesn't fit crates.io)
4. **xtask binary** provides consistent tooling regardless of project structure

### 1.3 Distribution Channels

| Artifact | Distribution | Version Strategy |
|----------|--------------|------------------|
| `vstkit-template` | GitHub template repo | Tagged releases (v0.4.0) |
| `vstkit-core` | crates.io (future: git initially) | SemVer, follows framework |
| `vstkit-protocol` | crates.io | SemVer |
| `vstkit-bridge` | crates.io | SemVer |
| `vstkit-metering` | crates.io | SemVer |
| `vstkit-dsp` | crates.io | SemVer, optional |
| `@vstkit/ui` | npm | SemVer, matches Rust |
| `@vstkit/ipc` | npm | SemVer, matches Rust |

**Phase 1 (MVP):** Git dependencies only, no crates.io/npm publishing  
**Phase 2:** Publish to crates.io and npm for proper ecosystem integration

---

## 2. Crate Boundary Definition

### 2.1 Current Crate Audit

| Crate | Current Purpose | SDK Role | Changes Needed |
|-------|-----------------|----------|----------------|
| `protocol` | IPC types, parameter definitions | **Framework** | Rename to `vstkit-protocol`, make generic |
| `bridge` | IPC handler implementation | **Framework** | Rename to `vstkit-bridge`, extract trait |
| `metering` | SPSC ring buffer for meters | **Framework** | Rename to `vstkit-metering`, unchanged |
| `dsp` | Gain algorithm, processor trait | **Framework + Example** | Split into `vstkit-dsp` (traits) + example impl |
| `plugin` | nih-plug integration, WebView | **Framework** | Rename to `vstkit-core`, extract user extension points |
| `standalone` | Desktop testing app | **Template** | Move to template, user can customize |
| `xtask` | Build tooling | **Framework** | Rename to `vstkit-xtask`, make installable |

### 2.2 New Crate Structure

```
vstkit/                          ← Framework repository
├── crates/
│   ├── vstkit-core/             ← Main framework crate (plugin + WebView)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs           ← Re-exports, plugin macro
│   │       ├── plugin.rs        ← nih-plug integration
│   │       ├── editor/          ← WebView editor infrastructure
│   │       └── host.rs          ← ParameterHost trait
│   │
│   ├── vstkit-protocol/         ← IPC contracts, parameter types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── params.rs        ← ParamSpec, ParamId (user extends)
│   │       └── ipc.rs           ← JSON-RPC types
│   │
│   ├── vstkit-bridge/           ← IPC handler (framework-provided)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── handler.rs       ← IpcHandler + ParameterHost trait
│   │
│   ├── vstkit-metering/         ← Real-time safe metering
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs           ← SPSC ring buffer, MeterFrame
│   │
│   └── vstkit-dsp/              ← DSP primitives (optional)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── traits.rs        ← Processor trait (user implements)
│
├── xtask/                       ← Build tooling (installable)
│   ├── Cargo.toml
│   └── src/
│       └── ...
│
└── ui/                          ← @vstkit/ui and @vstkit/ipc packages
    ├── packages/
    │   ├── ui/                  ← React components (@vstkit/ui)
    │   │   ├── package.json
    │   │   └── src/
    │   │       ├── components/  ← Meter, ParameterSlider, etc.
    │   │       └── index.ts
    │   │
    │   └── ipc/                 ← IPC client (@vstkit/ipc)
    │       ├── package.json
    │       └── src/
    │           ├── IpcBridge.ts
    │           ├── hooks.ts
    │           └── index.ts
    │
    └── package.json             ← Workspace root
```

### 2.3 User Project Structure (Template)

```
my-plugin/                       ← User's plugin project
├── engine/
│   ├── Cargo.toml               ← Depends on vstkit-* crates
│   └── src/
│       ├── lib.rs               ← Plugin entry point, uses vstkit::plugin! macro
│       ├── params.rs            ← User's parameter definitions
│       └── dsp.rs               ← User's DSP implementation
│
├── ui/
│   ├── package.json             ← Depends on @vstkit/ui, @vstkit/ipc
│   ├── vite.config.ts
│   └── src/
│       ├── App.tsx              ← User's custom UI layout
│       ├── components/          ← User's custom components
│       └── main.tsx
│
├── .cargo/
│   └── config.toml              ← Alias: xtask = "vstkit-xtask"
│
└── README.md
```

---

## 3. API Design

### 3.1 Plugin Entry Point

Users should be able to create a plugin with minimal boilerplate:

```rust
// my-plugin/engine/src/lib.rs
use vstkit_core::prelude::*;

// Define parameters
vstkit_params! {
    Gain: { id: 0, name: "Gain", range: -24.0..=24.0, default: 0.0, unit: "dB" },
    Mix: { id: 1, name: "Mix", range: 0.0..=1.0, default: 1.0, unit: "%" },
}

// Define plugin metadata
vstkit_plugin! {
    name: "My Awesome Plugin",
    vendor: "My Company",
    url: "https://example.com",
    email: "support@example.com",
    version: env!("CARGO_PKG_VERSION"),
    
    // Audio configuration
    audio: {
        inputs: 2,
        outputs: 2,
    },
    
    // Parameter set
    params: [Gain, Mix],
    
    // DSP processor
    processor: MyProcessor,
}

// Implement DSP
struct MyProcessor {
    gain: f32,
    mix: f32,
}

impl vstkit_dsp::Processor for MyProcessor {
    fn process(&mut self, buffer: &mut AudioBuffer, params: &ParamState) {
        self.gain = params.get(Gain);
        self.mix = params.get(Mix);
        
        for sample in buffer.iter_mut() {
            *sample *= vstkit_dsp::db_to_linear(self.gain);
        }
    }
}
```

### 3.2 Parameter Definition API

```rust
// vstkit-protocol/src/params.rs

/// Trait for user-defined parameter sets
pub trait ParamSet: 'static + Send + Sync {
    /// Number of parameters
    const COUNT: usize;
    
    /// Get parameter spec by ID
    fn spec(id: ParamId) -> Option<&'static ParamSpec>;
    
    /// Iterate all parameter specs
    fn iter() -> impl Iterator<Item = &'static ParamSpec>;
}

/// Parameter specification
#[derive(Debug, Clone)]
pub struct ParamSpec {
    pub id: ParamId,
    pub name: &'static str,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub unit: &'static str,
    pub flags: ParamFlags,
}

/// Macro for defining parameters
#[macro_export]
macro_rules! vstkit_params {
    ($($name:ident: { id: $id:expr, name: $label:expr, range: $min:expr..=$max:expr, default: $default:expr, unit: $unit:expr $(,)? } ),* $(,)?) => {
        // Generates enum + ParamSet impl
    };
}
```

### 3.3 UI Extension Points

```typescript
// User's App.tsx
import { VstKitProvider, Meter, ParameterSlider, useParameter } from '@vstkit/ui';
import { IpcBridge } from '@vstkit/ipc';

function App() {
  return (
    <VstKitProvider>
      {/* User can use built-in components */}
      <Meter />
      
      {/* User can customize sliders */}
      <ParameterSlider 
        id="gain" 
        className="my-custom-slider"
        renderValue={(v) => `${v.toFixed(1)} dB`}
      />
      
      {/* User can create custom components */}
      <MyCustomKnob parameterId="mix" />
    </VstKitProvider>
  );
}

// Custom component using VstKit hooks
function MyCustomKnob({ parameterId }: { parameterId: string }) {
  const { value, setValue, spec } = useParameter(parameterId);
  
  return (
    <div className="knob">
      {/* Custom rendering */}
    </div>
  );
}
```

---

## 4. Build Tooling

### 4.1 xtask Commands

The `vstkit-xtask` binary provides all build operations:

```bash
# Installation (one-time)
cargo install vstkit-xtask

# Or via cargo alias in user's .cargo/config.toml
[alias]
xtask = "run --manifest-path xtask/Cargo.toml --"

# Commands
cargo xtask new my-plugin          # Scaffold new project (future)
cargo xtask build                   # Build plugin (debug)
cargo xtask build --release         # Build plugin (release)
cargo xtask bundle                  # Create VST3/CLAP bundles
cargo xtask dev                     # Run dev servers (UI + WS)
cargo xtask test                    # Run all tests
cargo xtask lint                    # Run linters
cargo xtask sign                    # macOS code signing
cargo xtask install                 # Install to plugin directories
```

### 4.2 Build Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                        cargo xtask bundle                           │
└─────────────────────────────────────────────────────────────────────┘
                                   │
         ┌─────────────────────────┼─────────────────────────┐
         │                         │                         │
         ▼                         ▼                         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  1. Build UI    │     │  2. Build Rust  │     │  3. Bundle      │
│                 │     │                 │     │                 │
│  npm run build  │     │  cargo build    │     │  nih-plug       │
│  → ui/dist/     │     │  --release      │     │  bundler        │
│                 │     │  → target/      │     │  → VST3/CLAP    │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         │                       │                       │
         └───────────────────────┴───────────────────────┘
                                 │
                                 ▼
                    ┌─────────────────────────┐
                    │  target/bundled/        │
                    │  ├── VstKit.vst3/       │
                    │  └── VstKit.clap        │
                    └─────────────────────────┘
```

---

## 5. Developer Workflow

### 5.1 Getting Started (Target: < 30 minutes)

```bash
# 1. Clone template (5 min)
git clone https://github.com/vstkit/vstkit-plugin-template my-plugin
cd my-plugin

# 2. Install dependencies (5 min)
cd ui && npm install && cd ..

# 3. Run dev environment (2 min)
cargo xtask dev
# Opens browser at localhost:5173 with hot reload
# WebSocket connects to Rust engine

# 4. Modify DSP code, see changes (ongoing)
# Edit engine/src/dsp.rs
# UI updates via WebSocket

# 5. Build and test in DAW (5 min)
cargo xtask bundle --install
# Opens Ableton Live with plugin loaded
```

### 5.2 Development Cycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                      DEVELOPER WORKFLOW                             │
└─────────────────────────────────────────────────────────────────────┘

  ┌─────────┐                                                          
  │  START  │                                                          
  └────┬────┘                                                          
       │                                                               
       ▼                                                               
  ┌─────────────────┐                                                  
  │ cargo xtask dev │  ← Starts UI server + WebSocket bridge           
  └────────┬────────┘                                                  
           │                                                           
           │  Opens browser at localhost:5173                          
           │  Hot reload enabled                                       
           │                                                           
           ▼                                                           
  ┌─────────────────────────────────────────┐                          
  │              EDIT LOOP                   │                          
  │                                          │                          
  │   ┌─────────────┐     ┌─────────────┐   │                          
  │   │  Edit UI    │     │  Edit DSP   │   │                          
  │   │  (tsx/css)  │     │  (Rust)     │   │                          
  │   └──────┬──────┘     └──────┬──────┘   │                          
  │          │                   │          │                          
  │          │  Hot reload       │  Restart │                          
  │          │  (instant)        │  server  │                          
  │          │                   │          │                          
  │          └─────────┬─────────┘          │                          
  │                    │                    │                          
  │                    ▼                    │                          
  │          ┌─────────────────┐            │                          
  │          │  Test in browser │            │                          
  │          │  (real engine)   │            │                          
  │          └─────────────────┘            │                          
  │                    │                    │                          
  │              Repeat │                    │                          
  │                    │                    │                          
  └────────────────────┼────────────────────┘                          
                       │                                               
                       │  Ready to test in DAW?                        
                       │                                               
                       ▼                                               
  ┌────────────────────────────────┐                                   
  │  cargo xtask bundle --install  │                                   
  └────────────────────────────────┘                                   
                       │                                               
                       ▼                                               
  ┌─────────────────────────────────┐                                  
  │  Test in DAW (Ableton Live)     │                                  
  │  - Parameter automation         │                                  
  │  - State save/restore           │                                  
  │  - Host compatibility           │                                  
  └─────────────────────────────────┘                                  
```

---

## 6. Versioning Strategy

### 6.1 Independent Package Versioning

Each VstKit package maintains its **own independent version**, following standard ecosystem practices:

```
vstkit-core         = "0.4.0"   ← Core framework, most active
vstkit-protocol     = "0.2.1"   ← Stable, changes rarely
vstkit-bridge       = "0.3.0"   ← Follows protocol changes
vstkit-metering     = "0.1.0"   ← Stable since M3
vstkit-dsp          = "0.1.0"   ← Traits rarely change
@vstkit/ui          = "0.5.0"   ← UI evolves independently
@vstkit/ipc         = "0.3.0"   ← Matches bridge protocol
```

**Rationale:**
- Packages evolve at different rates (UI changes more often than metering)
- Users only get breaking changes for packages they use
- Follows Rust/npm ecosystem conventions
- Avoids unnecessary version bumps for unchanged packages

### 6.2 Compatibility Matrix

Each `vstkit-core` release documents compatible versions of other packages:

```toml
# vstkit-core 0.4.0 compatibility
[dependencies]
vstkit-protocol = ">=0.2.0, <0.3.0"
vstkit-bridge = ">=0.3.0, <0.4.0"
vstkit-metering = ">=0.1.0"
vstkit-dsp = ">=0.1.0"
```

The template repository pins known-good combinations.

### 6.3 Semantic Versioning Policy

| Change Type | Version Bump | User Impact |
|-------------|--------------|-------------|
| Bug fix, docs | Patch (0.4.1) | Safe to update automatically |
| New feature, deprecation | Minor (0.5.0) | Review changelog, update recommended |
| Breaking API change | Major (1.0.0) | Migration guide required |

### 6.3 User Update Workflow

```toml
# User's Cargo.toml - each package versioned independently
[dependencies]
vstkit-core = "0.4"       # Accepts 0.4.x patches
vstkit-protocol = "0.2"   # Stable, rarely changes
vstkit-dsp = "0.1"        # Optional, for DSP traits

# To update a specific package:
# 1. Check CHANGELOG.md for that package
# 2. Update version constraint
# 3. cargo update -p vstkit-core
```

### 6.4 Breaking Change Policy

Before 1.0:
- Minor versions may have breaking changes
- Deprecation warnings precede removal by 1 minor version
- Migration guides in CHANGELOG.md

After 1.0:
- Only major versions have breaking changes
- Minimum 6 months deprecation period
- Automated migration tools (codemods) where feasible

---

## 7. Documentation Requirements

### 7.1 Documentation Structure

```
docs/
├── getting-started.md           ← 15-minute quickstart
├── concepts/
│   ├── architecture.md          ← How VstKit works
│   ├── parameters.md            ← Parameter system deep dive
│   ├── ui-bridge.md             ← UI ↔ Engine communication
│   └── real-time-safety.md      ← Audio thread constraints
│
├── guides/
│   ├── custom-dsp.md            ← Implementing your DSP
│   ├── custom-ui.md             ← Customizing the React UI
│   ├── macos-signing.md         ← Code signing and notarization
│   └── debugging.md             ← Common issues and solutions
│
├── api/
│   ├── rust/                    ← Generated from rustdoc
│   └── typescript/              ← Generated from TypeDoc
│
└── examples/
    ├── gain-plugin/             ← Minimal example
    ├── eq-plugin/               ← Multi-band EQ
    └── delay-plugin/            ← Stateful DSP example
```

### 7.2 Documentation Hosting

**Phase 1:** GitHub repository + mdBook  
**Phase 2:** Dedicated docs site (vstkit.dev) with search

---

## 8. Target Developer Persona

### 8.1 Primary Persona: "The Rust Audio Developer"

**Profile:**
- Intermediate Rust experience (comfortable with traits, lifetimes, async)
- Some DSP knowledge (knows what a filter is, can implement basic effects)
- Familiar with web development (React basics, npm/node)
- May have used nih-plug or other Rust audio tools

**Needs:**
- Clear project structure
- Type-safe parameter system
- Real-time safety guidance
- Working examples to learn from

**Pain Points:**
- UI development is tedious (wants React, not raw graphics)
- Plugin hosting is complex (code signing, bundling)
- Testing requires a DAW

### 8.2 Secondary Persona: "The Web Developer with Audio Interest"

**Profile:**
- Strong React/TypeScript skills
- New to Rust (willing to learn for audio quality)
- Basic audio knowledge (uses DAWs as a hobby)

**Needs:**
- Familiar React development experience
- Rust concepts explained in web terms
- Copy-paste examples that work

**Pain Points:**
- Rust learning curve
- Real-time constraints are foreign
- Audio math is intimidating

### 8.3 What We're NOT Optimizing For

- **Complete beginners** — Some programming experience required
- **C++/JUCE experts** — They have their ecosystem, though they're welcome
- **Linux-only developers** — macOS is the primary platform

---

## 9. Migration Path (VstKit Internal → SDK)

### 9.1 Phase 1: Restructure (This PR)

1. ✅ Create feature spec and user stories
2. Create low-level design (this document)
3. No code changes yet — architecture validation only

### 9.2 Phase 2: Crate Extraction (Future PR)

1. Rename crates with `vstkit-` prefix
2. Extract traits and interfaces
3. Create template repository
4. Split UI into npm packages

### 9.3 Phase 3: Documentation (Future PR)

1. Write getting-started guide
2. Create example plugins
3. Set up docs site

### 9.4 Phase 4: Publishing (Future PR)

1. Publish to crates.io
2. Publish to npm
3. Announce SDK availability

---

## 10. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| **API instability** | Users hit breaking changes frequently | Deprecation warnings, migration guides, conservative 1.0 release |
| **Package version mismatch** | Incompatible combinations | Compatibility matrix in docs, template pins known-good versions |
| **nih-plug breaking changes** | SDK must adapt | Pin nih-plug version, document upgrade path |
| **Documentation debt** | Users can't onboard | Budget docs time, treat as blocking for releases |
| **Scope creep** | SDK never ships | Phased approach, MVP first |

---

## 11. Success Metrics

### Phase 1 (Investigation) — Complete When:
- [x] User stories created
- [x] Low-level design documented
- [ ] Design reviewed and approved

### Phase 2 (Implementation) — Success Criteria:
- [ ] New developer creates working plugin in < 30 minutes
- [ ] Zero breaking changes without deprecation warning
- [ ] 100% of public APIs documented
- [ ] At least 3 example plugins

### Long-term (6 months post-launch):
- [ ] > 10 external projects using VstKit
- [ ] Community contributions (PRs, plugins)
- [ ] < 1 week average issue response time

---

## 12. Open Questions

1. **Git vs crates.io for initial release?**  
   Recommendation: Git dependencies first (simpler), crates.io once API is stable.

2. **Mono-repo vs multi-repo for template?**  
   Recommendation: Mono-repo template (simpler for users), SDK is separate repo.

3. **Should xtask be installable via cargo install?**  
   Recommendation: Yes, but also support local xtask in template for offline use.

4. **How to handle nih-plug version pinning?**  
   Recommendation: Pin to specific commit in SDK, document in release notes.

5. **Should we provide a proc-macro for plugin definition?**  
   Recommendation: Yes (`vstkit_plugin!`), but keep it thin — users can always write manual impl.

---

## 13. Handoff

**Next Step:** Review with Product Owner, then hand off to **Planner agent** to create implementation plan.

*"Create implementation plan for VstKit Developer SDK Phase 2, based on low-level design in `/docs/feature-specs/developer-sdk/low-level-design-developer-sdk.md`. Focus on crate extraction and template repository creation."*
