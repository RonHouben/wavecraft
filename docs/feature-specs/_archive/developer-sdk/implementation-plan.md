# Implementation Plan: Developer SDK

## Overview

This plan transforms VstKit from an internal project into a Developer SDK, following the architecture defined in [low-level-design-developer-sdk.md](./low-level-design-developer-sdk.md).

**Target Version:** `0.4.0` (vstkit-core initial SDK release)

**Estimated Duration:** 3-4 weeks across 4 phases

---

## Requirements Summary

From [user-stories.md](./user-stories.md):
1. SDK packaging via hybrid model (template repo + crates.io)
2. Clear boundaries between framework and user code
3. Developer workflow: zero to working plugin in < 30 minutes
4. Independent package versioning
5. Comprehensive documentation

---

## Phase 1: Crate Restructuring

**Goal:** Rename and reorganize crates with `vstkit-` prefix, establish clear SDK boundaries.

**Duration:** 1 week

### Step 1.1: Rename Protocol Crate

**Files:** `engine/crates/protocol/` → `engine/crates/vstkit-protocol/`

| Task | File | Action |
|------|------|--------|
| Rename directory | `crates/protocol/` | Rename to `crates/vstkit-protocol/` |
| Update Cargo.toml | `crates/vstkit-protocol/Cargo.toml` | Change `name = "protocol"` to `name = "vstkit-protocol"` |
| Update workspace | `engine/Cargo.toml` | Update `[workspace.dependencies]` path |
| Update dependents | All crates using `protocol` | Change to `vstkit-protocol` |

**Dependencies:** None  
**Risk:** Low — straightforward rename  
**Verification:** `cargo check --workspace`

### Step 1.2: Rename Bridge Crate

**Files:** `engine/crates/bridge/` → `engine/crates/vstkit-bridge/`

| Task | File | Action |
|------|------|--------|
| Rename directory | `crates/bridge/` | Rename to `crates/vstkit-bridge/` |
| Update Cargo.toml | `crates/vstkit-bridge/Cargo.toml` | Change `name = "bridge"` to `name = "vstkit-bridge"` |
| Update workspace | `engine/Cargo.toml` | Update `[workspace.dependencies]` path |
| Update dependents | `plugin`, `standalone` | Change to `vstkit-bridge` |

**Dependencies:** Step 1.1  
**Risk:** Low  
**Verification:** `cargo check --workspace`

### Step 1.3: Rename Metering Crate

**Files:** `engine/crates/metering/` → `engine/crates/vstkit-metering/`

| Task | File | Action |
|------|------|--------|
| Rename directory | `crates/metering/` | Rename to `crates/vstkit-metering/` |
| Update Cargo.toml | `crates/vstkit-metering/Cargo.toml` | Change `name = "metering"` to `name = "vstkit-metering"` |
| Update workspace | `engine/Cargo.toml` | Update `[workspace.dependencies]` path |
| Update dependents | `plugin` | Change to `vstkit-metering` |

**Dependencies:** None (parallel with 1.1, 1.2)  
**Risk:** Low  
**Verification:** `cargo check --workspace`

### Step 1.4: Rename DSP Crate

**Files:** `engine/crates/dsp/` → `engine/crates/vstkit-dsp/`

| Task | File | Action |
|------|------|--------|
| Rename directory | `crates/dsp/` | Rename to `crates/vstkit-dsp/` |
| Update Cargo.toml | `crates/vstkit-dsp/Cargo.toml` | Change `name = "dsp"` to `name = "vstkit-dsp"` |
| Update workspace | `engine/Cargo.toml` | Update `[workspace.dependencies]` path |
| Update dependents | `plugin` | Change to `vstkit-dsp` |

**Dependencies:** Step 1.1 (dsp depends on protocol)  
**Risk:** Low  
**Verification:** `cargo check --workspace`

### Step 1.5: Rename Plugin Crate to vstkit-core

**Files:** `engine/crates/plugin/` → `engine/crates/vstkit-core/`

| Task | File | Action |
|------|------|--------|
| Rename directory | `crates/plugin/` | Rename to `crates/vstkit-core/` |
| Update Cargo.toml | `crates/vstkit-core/Cargo.toml` | Change `name = "vstkit"` to `name = "vstkit-core"` |
| Update lib name | `crates/vstkit-core/Cargo.toml` | Change `lib.name = "vstkit"` to `lib.name = "vstkit_core"` |
| Update workspace | `engine/Cargo.toml` | Update `[workspace.dependencies]` path |

**Dependencies:** Steps 1.1-1.4  
**Risk:** Medium — this is the main plugin crate  
**Verification:** `cargo xtask bundle`, test in DAW

### Step 1.6: Update xtask References

**Files:** `engine/xtask/src/`

| Task | File | Action |
|------|------|--------|
| Update crate references | `commands/*.rs` | Change `vstkit` to `vstkit-core` in bundle commands |
| Update paths | `lib.rs` | Update any hardcoded crate paths |

**Dependencies:** Step 1.5  
**Risk:** Low  
**Verification:** `cargo xtask bundle`

### Step 1.7: Phase 1 Integration Test

| Task | Action |
|------|--------|
| Run all tests | `cargo xtask test` |
| Run linters | `cargo xtask lint` |
| Build plugin | `cargo xtask bundle --install` |
| Test in DAW | Load in Ableton Live, verify functionality |

**Dependencies:** Steps 1.1-1.6  
**Verification:** All tests pass, plugin works in DAW

---

## Phase 2: API Extraction

**Goal:** Extract public APIs, traits, and create user extension points.

**Duration:** 1 week

### Step 2.1: Create Processor Trait in vstkit-dsp

**File:** `engine/crates/vstkit-dsp/src/traits.rs`

```rust
/// Trait for user-implemented DSP processors
pub trait Processor: Send + 'static {
    /// Process a buffer of audio samples
    fn process(&mut self, buffer: &mut [&mut [f32]], transport: &Transport);
    
    /// Called when sample rate changes
    fn set_sample_rate(&mut self, sample_rate: f32) {}
    
    /// Reset processor state (e.g., on transport stop)
    fn reset(&mut self) {}
}
```

| Task | File | Action |
|------|------|--------|
| Create traits module | `vstkit-dsp/src/traits.rs` | Define `Processor` trait |
| Export from lib | `vstkit-dsp/src/lib.rs` | Add `pub mod traits; pub use traits::*;` |
| Move gain to examples | `vstkit-dsp/src/gain.rs` | Keep as reference implementation |

**Dependencies:** Phase 1 complete  
**Risk:** Medium — defines core abstraction  
**Verification:** Existing plugin compiles with new trait

### Step 2.2: Create ParamSet Trait in vstkit-protocol

**File:** `engine/crates/vstkit-protocol/src/params.rs`

```rust
/// Trait for user-defined parameter sets
pub trait ParamSet: 'static + Send + Sync {
    const COUNT: usize;
    fn spec(id: ParamId) -> Option<&'static ParamSpec>;
    fn iter() -> impl Iterator<Item = &'static ParamSpec>;
}
```

| Task | File | Action |
|------|------|--------|
| Define ParamSet trait | `vstkit-protocol/src/params.rs` | Add trait definition |
| Make PARAM_SPECS generic | `vstkit-protocol/src/params.rs` | Remove hardcoded params |
| Export trait | `vstkit-protocol/src/lib.rs` | Add to public exports |

**Dependencies:** Phase 1 complete  
**Risk:** Medium — affects parameter system  
**Verification:** Plugin compiles, parameters work

### Step 2.3: Create vstkit_params! Macro

**File:** `engine/crates/vstkit-protocol/src/macros.rs`

```rust
/// Macro for defining parameters with minimal boilerplate
#[macro_export]
macro_rules! vstkit_params {
    ($($name:ident: { 
        id: $id:expr, 
        name: $label:expr, 
        range: $min:expr..=$max:expr, 
        default: $default:expr, 
        unit: $unit:expr $(,)? 
    }),* $(,)?) => {
        // Generate enum and ParamSet impl
    };
}
```

| Task | File | Action |
|------|------|--------|
| Create macros module | `vstkit-protocol/src/macros.rs` | Define `vstkit_params!` |
| Export macro | `vstkit-protocol/src/lib.rs` | Add `pub mod macros;` |
| Add tests | `vstkit-protocol/src/macros.rs` | Unit tests for macro expansion |

**Dependencies:** Step 2.2  
**Risk:** High — macro development is complex  
**Verification:** Test with sample parameter set

### Step 2.4: Create vstkit_plugin! Macro

**File:** `engine/crates/vstkit-core/src/macros.rs`

```rust
/// Macro for defining a complete VstKit plugin
#[macro_export]
macro_rules! vstkit_plugin {
    (
        name: $name:expr,
        vendor: $vendor:expr,
        // ... other fields
        params: [$($param:ty),*],
        processor: $processor:ty,
    ) => {
        // Generate nih-plug Plugin impl
    };
}
```

**Subtasks (Coder):**
- [ ] Create `engine/crates/vstkit-core/src/macros.rs` and implement a **thin** `vstkit_plugin!` macro that:
  - Generates the minimal nih-plug plugin skeleton and descriptor (name/vendor/version/url/email)
  - Registers parameters from the provided `params` list (uses `vstkit-protocol` ParamSet infrastructure)
  - Wires the provided `processor` type into the plugin audio callbacks (construct & call in process)
  - Keeps expansion explicit (prefer generated glue, not hidden behaviour)
- [ ] Add `trybuild` compile-time tests under `engine/crates/vstkit-core/tests/` that assert macro expands for minimal and full variants
- [ ] Add runnable doc examples in `macros.rs` (doc tests) showing minimal usage and a full-featured example
- [ ] Re-export macro in `vstkit-core/src/lib.rs` (e.g., `pub mod macros; pub use macros::vstkit_plugin;`)
- [ ] Update `vstkit-plugin-template/engine/src/lib.rs` to use the macro and verify the example plugin builds
- [ ] Update docs (low-level design and implementation plan) to mark the macro implemented and add usage notes
- [ ] Run `cargo xtask test` and `cargo xtask bundle` and perform a DAW load test to verify end-to-end behavior

| Task | File | Action |
|------|------|--------|
| Create macros module | `vstkit-core/src/macros.rs` | Define `vstkit_plugin!` |
| Export macro | `vstkit-core/src/lib.rs` | Add to public exports |
| Create prelude | `vstkit-core/src/prelude.rs` | Re-export common types |

**Dependencies:** Steps 2.1-2.3  
**Risk:** High — integrates all components  
**Verification:** Create minimal plugin using macro (see subtasks)

### Step 2.5: Extract ParameterHost Trait

**File:** `engine/crates/vstkit-bridge/src/host.rs`

| Task | File | Action |
|------|------|--------|
| Extract trait | `vstkit-bridge/src/host.rs` | Move `ParameterHost` to own module |
| Make generic | `vstkit-bridge/src/handler.rs` | Use trait bounds instead of concrete type |
| Document trait | `vstkit-bridge/src/host.rs` | Add rustdoc comments |

**Dependencies:** Phase 1 complete  
**Risk:** Low — trait already exists  
**Verification:** Plugin compiles, IPC works

### Step 2.6: Phase 2 Integration Test

| Task | Action |
|------|--------|
| Create test plugin | Use macros to define minimal plugin |
| Run all tests | `cargo xtask test` |
| Build plugin | `cargo xtask bundle --install` |
| Test in DAW | Verify macro-based plugin works |

**Dependencies:** Steps 2.1-2.5  
**Verification:** Macro-based plugin loads and functions in DAW

---

## Phase 3: Template Repository

**Goal:** Create standalone template repository for new plugin projects.

**Duration:** 1 week

### Step 3.1: Create Template Repository Structure

**Location:** New repository `vstkit-plugin-template`

```
vstkit-plugin-template/
├── engine/
│   ├── Cargo.toml
│   ├── .cargo/
│   │   └── config.toml        # xtask alias
│   └── src/
│       ├── lib.rs             # Plugin entry point
│       ├── params.rs          # User parameters
│       └── dsp.rs             # User DSP code
├── ui/
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── tailwind.config.js
│   └── src/
│       ├── App.tsx
│       ├── main.tsx
│       └── index.css
├── .gitignore
├── README.md
└── LICENSE
```

| Task | File | Action |
|------|------|--------|
| Create repo structure | All | Copy and adapt from VstKit |
| Create Cargo.toml | `engine/Cargo.toml` | Depend on vstkit-* via git |
| Create lib.rs | `engine/src/lib.rs` | Minimal plugin using macros |
| Create package.json | `ui/package.json` | Depend on @vstkit/* (or copy) |
| Create README | `README.md` | Getting started guide |

**Dependencies:** Phase 2 complete  
**Risk:** Medium — new repo setup  
**Verification:** Clone template, build plugin

### Step 3.2: Configure Git Dependencies

**File:** `vstkit-plugin-template/engine/Cargo.toml`

```toml
[dependencies]
vstkit-core = { git = "https://github.com/vstkit/vstkit", branch = "main" }
vstkit-protocol = { git = "https://github.com/vstkit/vstkit", branch = "main" }
vstkit-dsp = { git = "https://github.com/vstkit/vstkit", branch = "main" }
```

| Task | File | Action |
|------|------|--------|
| Add git deps | `Cargo.toml` | Point to VstKit repo |
| Test fetch | — | Verify deps resolve |
| Document | `README.md` | Explain dependency model |

**Dependencies:** Step 3.1  
**Risk:** Low  
**Verification:** `cargo check` in template

### Step 3.3: Copy UI Layer to Template

Since we're not publishing @vstkit/ui to npm yet, copy the UI components:

| Task | File | Action |
|------|------|--------|
| Copy vstkit-ipc | `ui/src/lib/vstkit-ipc/` | Copy IPC client code |
| Copy components | `ui/src/components/` | Copy Meter, ParameterSlider, etc. |
| Update imports | All UI files | Adjust import paths |
| Test build | — | `npm run build` |

**Dependencies:** Step 3.1  
**Risk:** Medium — must keep in sync with VstKit  
**Verification:** UI builds and runs

### Step 3.4: Create Example Plugin

**File:** `vstkit-plugin-template/engine/src/lib.rs`

```rust
use vstkit_core::prelude::*;

vstkit_params! {
    Gain: { id: 0, name: "Gain", range: -24.0..=24.0, default: 0.0, unit: "dB" },
}

vstkit_plugin! {
    name: "My Plugin",
    vendor: "My Company",
    url: "https://example.com",
    email: "contact@example.com",
    version: env!("CARGO_PKG_VERSION"),
    
    audio: { inputs: 2, outputs: 2 },
    params: [Gain],
    processor: GainProcessor,
}

struct GainProcessor;

impl vstkit_dsp::Processor for GainProcessor {
    fn process(&mut self, buffer: &mut [&mut [f32]], _transport: &Transport) {
        // Simple gain implementation
    }
}
```

| Task | File | Action |
|------|------|--------|
| Implement example | `engine/src/lib.rs` | Minimal gain plugin |
| Test build | — | `cargo xtask bundle` |
| Test in DAW | — | Load in Ableton |

**Dependencies:** Steps 3.1-3.3  
**Risk:** Medium — validates entire SDK  
**Verification:** Plugin works in DAW

### Step 3.5: Create Getting Started README

**File:** `vstkit-plugin-template/README.md`

| Section | Content |
|---------|---------|
| Prerequisites | Rust, Node.js, macOS |
| Quick Start | Clone, install deps, run dev |
| Project Structure | Explain engine/ and ui/ |
| Customization | How to add parameters, change DSP |
| Building | `cargo xtask bundle` |
| Testing | Dev workflow with hot reload |
| Troubleshooting | Common issues |

**Dependencies:** Steps 3.1-3.4  
**Risk:** Low  
**Verification:** Follow guide from scratch

### Step 3.6: Phase 3 Integration Test

| Task | Action |
|------|--------|
| Fresh clone | Clone template to new directory |
| Follow README | Execute all getting started steps |
| Build plugin | `cargo xtask bundle --install` |
| Test in DAW | Verify plugin works |
| Time to first plugin | Should be < 30 minutes |

**Dependencies:** Steps 3.1-3.5  
**Verification:** Complete workflow works end-to-end

---

## Phase 4: Documentation & Polish

**Goal:** Complete documentation and prepare for SDK release.

**Duration:** 1 week

### Step 4.1: Update Architecture Docs

**Files:** `docs/architecture/`

| Task | File | Action |
|------|------|--------|
| Update high-level design | `high-level-design.md` | Add SDK distribution section |
| Update coding standards | `coding-standards.md` | Add SDK-specific patterns |
| Create SDK overview | `sdk-overview.md` | New doc explaining SDK usage |

**Dependencies:** Phase 3 complete  
**Risk:** Low  
**Verification:** Docs accurate and helpful

### Step 4.2: Generate API Documentation

| Task | Action |
|------|--------|
| Add rustdoc comments | All public APIs in vstkit-* crates |
| Generate docs | `cargo doc --workspace --no-deps` |
| Host on GitHub Pages | Or link to docs.rs |

**Dependencies:** Phase 2 complete  
**Risk:** Low  
**Verification:** All public items documented

### Step 4.3: Create Concept Guides

**Files:** `docs/concepts/`

| Guide | Content |
|-------|---------|
| `architecture.md` | How VstKit components fit together |
| `parameters.md` | Parameter system deep dive |
| `ui-bridge.md` | UI ↔ Engine communication |
| `real-time-safety.md` | Audio thread constraints |

**Dependencies:** Phase 3 complete  
**Risk:** Medium — significant writing  
**Verification:** Review for clarity

### Step 4.4: Update Roadmap

**File:** `docs/roadmap.md`

| Task | Action |
|------|--------|
| Mark M8 complete | Update status to ✅ |
| Add changelog entry | Document SDK release |
| Archive feature spec | Move to `_archive/developer-sdk/` |

**Dependencies:** All phases complete  
**Risk:** Low  
**Verification:** Roadmap accurate

### Step 4.5: Version Bump

**File:** `engine/Cargo.toml`

| Task | Action |
|------|--------|
| Bump version | `0.3.1` → `0.4.0` |
| Update changelogs | Add SDK release notes |
| Tag release | `git tag v0.4.0` |

**Dependencies:** All tests passing  
**Risk:** Low  
**Verification:** Version displays correctly

### Step 4.6: Final Integration Test

| Task | Action |
|------|--------|
| Run full test suite | `cargo xtask test` |
| Run linters | `cargo xtask lint` |
| Build from template | Fresh clone and build |
| Test in DAW | Full functionality check |
| Review documentation | Walkthrough as new user |

**Dependencies:** Steps 4.1-4.5  
**Verification:** Everything works, docs are clear

---

## Testing Strategy

### Unit Tests
- Macro expansion tests for `vstkit_params!` and `vstkit_plugin!`
- Trait implementation tests for `Processor` and `ParamSet`
- All existing tests continue to pass

### Integration Tests
- Template builds successfully
- Plugin loads in DAW
- Parameters work (get/set)
- Metering works
- UI renders correctly

### End-to-End Test
- New developer follows README
- Time to first working plugin < 30 minutes
- No undocumented steps required

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Macro complexity | High | Start with simple macros, iterate |
| Breaking existing plugin | High | Continuous testing after each step |
| Template drift from SDK | Medium | Automation to sync templates |
| Documentation gaps | Medium | Dogfood by following docs ourselves |

---

## Success Criteria

### Phase 1 Complete When:
- [ ] All crates renamed with `vstkit-` prefix
- [ ] All tests pass
- [ ] Plugin works in DAW

### Phase 2 Complete When:
- [ ] `Processor` trait defined and documented
- [ ] `ParamSet` trait defined and documented
- [ ] `vstkit_params!` macro works
- [ ] `vstkit_plugin!` macro works
- [ ] Minimal plugin buildable with macros

### Phase 3 Complete When:
- [ ] Template repository created
- [ ] Getting started README written
- [ ] Fresh clone → working plugin in < 30 minutes

### Phase 4 Complete When:
- [ ] All public APIs documented
- [ ] Concept guides written
- [ ] Version bumped to 0.4.0
- [ ] Roadmap updated

---

## Handoff

**Next Step:** Hand off to **Coder agent** to begin Phase 1 implementation.

*"Implement Phase 1 of Developer SDK: Crate restructuring. Start with Step 1.1 (rename protocol crate). Reference implementation plan in `/docs/feature-specs/developer-sdk/implementation-plan.md`."*
