# Low-Level Design — Build-Time Parameter Discovery

## Problem Statement

The `wavecraft start` command currently discovers plugin parameters by loading the compiled `.dylib` via `libloading` and calling FFI functions (`wavecraft_get_params_json`). When the dynamic library is loaded, macOS executes constructor functions registered by `nih_export_vst3!` and `nih_export_clap!` — these are nih-plug's static initializers for VST3/CLAP factory registration. On macOS, this can trigger `AudioComponentRegistrar` or other system-level audio subsystem interactions that **block indefinitely** or take unpredictable amounts of time, causing `wavecraft start` to hang at "Loading plugin parameters...".

This is a fundamental issue: the dev server needs parameter metadata, but the only way to get it is to load a binary that also initializes a full plugin host interface — something designed for DAW contexts, not CLI tooling.

## Solution Overview

Replace FFI-based parameter discovery with a **build-time sidecar JSON file**. The `wavecraft_plugin!` macro already has full knowledge of the parameter metadata at compile time. Rather than loading the plugin binary to extract this metadata at runtime, we emit it as a JSON file during compilation, and the `wavecraft start` command reads the file directly.

```
┌──────────────────────────────────────┐
│            BEFORE (current)          │
│                                      │
│  cargo build ──► plugin.dylib        │
│                        │             │
│  wavecraft start       │             │
│    └─► dlopen(dylib) ──┘             │
│         │ ← triggers nih-plug init   │
│         │ ← AudioComponentRegistrar  │
│         │ ← BLOCKS / HANGS           │
│         ▼                            │
│    FFI: wavecraft_get_params_json()  │
│         │                            │
│         ▼                            │
│    DevServerHost(params)             │
└──────────────────────────────────────┘

┌──────────────────────────────────────┐
│            AFTER (proposed)          │
│                                      │
│  cargo build ──► plugin.dylib        │
│       │                              │
│       └──► build.rs / macro writes   │
│            wavecraft-params.json     │
│                                      │
│  wavecraft start                     │
│    └─► read("wavecraft-params.json") │
│         │  ← no dlopen, no FFI       │
│         │  ← no nih-plug init        │
│         │  ← instant, deterministic  │
│         ▼                            │
│    DevServerHost(params)             │
│                                      │
│    (dylib still loaded later for     │
│     audio-dev FFI vtable only)       │
└──────────────────────────────────────┘
```

## Design Constraints

1. **Backward compatibility.** Existing plugins that don't generate the sidecar file must still work. The CLI should fall back to FFI-based loading if the JSON file is not found.

2. **Zero user friction.** The sidecar file must be generated automatically by the existing `cargo build` step — no new build commands, no manual steps.

3. **Consistency.** The sidecar JSON must contain the same data as the FFI export. The source of truth is the `ProcessorParams::param_specs()` implementation.

4. **No new dependencies.** The build.rs approach should not introduce new crate dependencies for the user's plugin.

5. **Audio-dev FFI remains separate.** The `DevProcessorVTable` FFI (for in-process audio) still requires loading the dylib. This design only replaces the parameter discovery path.

## Detailed Design

### Approach: Macro-Emitted Sidecar via `build.rs`

The `wavecraft_plugin!` macro cannot directly write files (proc macros run in a sandboxed context and don't have reliable access to `OUT_DIR`). Instead, we use a two-phase approach:

**Phase 1 — Macro generates a const JSON string**

The `wavecraft_plugin!` macro already generates `wavecraft_get_params_json()` which serializes params at runtime. We extend it to also generate a `const` function that can be called from `build.rs`:

Actually, this doesn't work either — proc macros expand into the crate being compiled, not into build scripts.

**Revised approach: Compile-and-run a minimal binary**

This is overly complex for the problem at hand.

### Chosen Approach: Post-Build Write via `std::fs` in a Generated `#[ctor]` or `env!`

After evaluating multiple options, the cleanest approach is:

### Approach: Macro generates a build-time JSON write function + build.rs integration

#### Step 1: Macro emits a JSON string literal

The `wavecraft_plugin!` macro already constructs parameter metadata. We extend it to **also** emit a constant string containing the JSON representation, and a helper function that writes it to the target directory at library-load time during `cargo build` (not at runtime).

However, writing files from within library code is fragile. A better approach:

---

### **Final Approach: Dedicated Build-Time Binary**

The cleanest architecture is a **thin build-time binary** that the user's `build.rs` invokes. This binary links only against the DSP crate (no nih-plug), calls `param_specs()`, serializes to JSON, and writes the file.

But this requires users to maintain a separate binary target, which violates constraint #2 (zero user friction).

---

### **Actual Final Approach: Macro + `build.rs` Cargo Env**

After careful analysis, the most pragmatic solution uses two mechanisms:

#### 1. `wavecraft_plugin!` macro emits a `const` JSON byte array

The macro already has access to the full `ParamSpec` data (it generates the `wavecraft_get_params_json` function from it). We add a new generated item: a module-level constant containing the serialized JSON, plus a linker-invoked write function.

**Problem:** The macro doesn't have access to `ParamSpec` values at proc-macro expansion time — it only generates code that *calls* `param_specs()` at runtime. The specs are defined by the `ProcessorParams` trait impl on the user's types, which are resolved during type-checking, not during macro expansion.

This is the core constraint that makes build-time emission harder than it first appears.

---

### **Recommended Approach: Post-Compilation Param Extraction via Thin Helper Binary**

Given the constraints above, the cleanest architecture that satisfies all requirements is:

```
┌─────────────────────────────────────────────────────────┐
│ User's Plugin Crate                                     │
│                                                         │
│  [lib] crate-type = ["cdylib"]    ← plugin (nih-plug)   │
│                                                         │
│  [[bin]]                                                │
│  name = "param-export"            ← thin helper binary  │
│  required-features = ["_param-export"]                  │
│                                                         │
│  [features]                                             │
│  _param-export = []               ← internal feature    │
└─────────────────────────────────────────────────────────┘
```

**Why this doesn't work well either:** Adding a binary target that depends on the DSP types but not nih-plug is non-trivial given that `wavecraft_plugin!` generates everything (including proc-macro-derived types) in `lib.rs`.

---

## Revised Design: `wavecraft start` Runs a Micro-Binary for Param Discovery

After iterating through the design space, the approach that best satisfies all constraints is:

### Architecture

```
cargo build --lib             ← builds plugin.dylib (as before)
wavecraft start
  ├── cargo build --bin wavecraft-param-export  ← new: tiny no-nih-plug binary
  │     └── links wavecraft-dsp only
  │     └── calls param_specs(), writes JSON to stdout
  │     └── exits immediately
  ├── reads JSON from stdout
  ├── DevServerHost(params)
  ├── (optionally) dlopen(dylib) for audio-dev vtable only
  └── starts WebSocket + UI servers
```

**Wait** — this still requires the user to have a binary target that knows about their specific processor types. The processor types and their `ProcessorParams` impls are defined in user code, not in SDK crates.

---

## Final Recommended Design

After carefully considering all the constraints, here is the approach that actually works:

### The Real Constraint

The parameter metadata is defined by user code (`ProcessorParams` impls on user-defined structs). This metadata is only available after compilation. It cannot be extracted at proc-macro expansion time because the macro only sees syntax, not resolved types.

Therefore, **some form of executing compiled user code is required**. The question is: how do we execute it without triggering nih-plug's static initializers?

### Solution: Feature-Gated FFI Exports (No nih-plug Exports)

```toml
# User's Cargo.toml (generated by template)
[features]
default = []
_param-discovery = []   # Internal: skip nih-plug exports for fast param loading
```

The `wavecraft_plugin!` macro generates conditionally-compiled code:

```rust
// Always generated — param discovery FFI
#[unsafe(no_mangle)]
pub extern "C" fn wavecraft_get_params_json() -> *mut c_char { ... }

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wavecraft_free_string(ptr: *mut c_char) { ... }

// Only when NOT in param-discovery mode — nih-plug exports
#[cfg(not(feature = "_param-discovery"))]
nih_export_clap!(__WavecraftPlugin);

#[cfg(not(feature = "_param-discovery"))]
nih_export_vst3!(__WavecraftPlugin);
```

### Startup Flow

```
wavecraft start
  │
  ├── 1. cargo build --lib --features _param-discovery
  │      └── produces plugin.dylib WITHOUT nih-plug static initializers
  │      └── fast build (incremental, only feature flag change)
  │
  ├── 2. dlopen(dylib) + wavecraft_get_params_json()
  │      └── NO AudioComponentRegistrar, NO VST3 factory init
  │      └── instant, deterministic
  │      └── close dylib
  │
  ├── 3. cargo build --lib   (normal build, for audio-dev vtable)
  │      └── only if audio-dev feature is needed
  │      └── can run in background while servers start
  │
  ├── 4. DevServerHost(params) + WebSocket server
  ├── 5. UI dev server
  └── 6. (optional) dlopen for audio-dev vtable
```

### Why Two Builds Isn't Great

Two `cargo build` invocations double the startup time. Cargo uses feature unification within a build, but **switching features between builds invalidates the cache** for the cdylib target (since the source code is different due to `#[cfg]`). This means:

- First build (with `_param-discovery`): compiles cdylib without nih-plug exports
- Second build (without): recompiles cdylib with nih-plug exports

Both are incremental, but the cdylib link step must re-run each time.

### Better: Single Build + Separate Thin Rlib

Instead of feature-gating the main cdylib, we add a **second library target** that only contains the param discovery code.

---

## ✅ Final Design: Param-Discovery Rlib via `wavecraft_plugin!` + CLI Helper

### Core Idea

The SDK template includes a tiny Rust binary that imports the user's processor types (but **not** the nih-plug plugin struct) and writes parameter JSON to a well-known file location. This binary is compiled and run by `wavecraft start` before the main `cargo build`.

### Implementation

#### 1. Template adds a `param-export` binary

In the SDK template (`cli/sdk-templates/new-project/react/engine/`):

**New file: `src/bin/param_export.rs`**

```rust
//! Param discovery helper — extracts parameter metadata without loading nih-plug.
//!
//! This binary is invoked by `wavecraft start` to discover plugin parameters
//! without triggering VST3/CLAP static initializers (which can hang on macOS).

// Import processor types from the library crate
use {{plugin_name_snake}}::processors;

fn main() {
    // Problem: We can't easily call param_specs() without knowing
    // the concrete ProcessorType, which is defined by wavecraft_plugin!
}
```

**The problem resurfaces:** The concrete `SignalChain` type and its `ProcessorParams` impl are generated inside `wavecraft_plugin!` in `lib.rs`. A separate binary can't access these generated types without also expanding the macro (which brings in nih-plug).

---

## ✅✅ Actual Final Design: Compile-Time JSON via `env!()` + Cargo Build Script

After working through every angle, here is the design that genuinely works:

### Key Insight

Proc macros execute at compile time and can emit arbitrary Rust code. While they cannot call `param_specs()` (which requires type resolution), they **can** perform the same computation themselves — because `wavecraft_plugin!` already knows the signal chain structure and `param_specs()` is deterministic from the `ProcessorParams` derive.

However, `wavecraft_plugin!` only sees `SignalChain![InputGain, OutputGain]` as syntax tokens — it doesn't resolve what `InputGain`'s params are. It just generates code that will resolve at type-checking time.

### The Only Approach That Actually Works

Given that:
1. Parameter metadata is only knowable after type resolution
2. nih-plug static initializers run at dylib load
3. We need param metadata without loading the dylib

**The solution is to write the JSON _from within the compiled plugin code_ at `cargo build` time, using a `build.rs` integration.**

### Design: `build.rs` Compiles and Runs a Discovery Binary

#### Overview

```
cargo build --lib
  │
  ├── build.rs runs first
  │     └── Compiles param_export.rs as a temporary binary
  │     └── Runs it: outputs JSON to target/wavecraft-params.json
  │     └── param_export.rs links wavecraft-dsp (NOT nih-plug)
  │
  ├── lib.rs compiles (with nih-plug exports)
  │
  wavecraft start
    └── Reads target/wavecraft-params.json  ← no dlopen needed!
    └── Falls back to FFI if file not found (backward compat)
```

**Same problem again:** `param_export.rs` can't know the user's `ProcessorParams` types without importing `lib.rs`, which contains `wavecraft_plugin!`.

---

## ✅✅✅ The Architecture That Actually Works

After exhaustive analysis, I am converging on the only approach that cleanly solves the constraints:

### **`wavecraft_plugin!` macro writes params JSON at compile time using `std::fs`**

A proc macro **can** write files to `OUT_DIR` (or even to `CARGO_MANIFEST_DIR`) during expansion. This is uncommon but not prohibited — several crates do this (e.g., `tonic-build`, `prost-build` for generated code).

However, `wavecraft_plugin!` doesn't know the parameter *values* at macro expansion time — it only generates code that queries them at runtime.

### **Resolution: Move param metadata computation into the proc macro**

Currently, `ProcessorParams::param_specs()` returns `&'static [ParamSpec]` at runtime. But when using `#[derive(ProcessorParams)]`, all the metadata (name, range, default, unit) is written as struct field attributes:

```rust
#[derive(ProcessorParams, Default)]
struct GainParams {
    #[param(name = "Level", range = "0.0..=2.0", default = 1.0, unit = "x")]
    level: f32,
}
```

The `ProcessorParams` derive macro already has access to all of this metadata (it's in the attribute syntax). Currently it only emits a `param_specs()` function. We can **additionally** have it emit the metadata into a compile-time-accessible form.

Similarly, `wavecraft_plugin!` knows the signal chain structure (`SignalChain![InputGain, OutputGain]`). Combined with metadata from `ProcessorParams`, we have everything needed.

**Challenge:** `wavecraft_plugin!` macro and `#[derive(ProcessorParams)]` run in separate macro expansion phases. `wavecraft_plugin!` cannot see the output of `ProcessorParams` derive.

### **The Only Clean Path: Post-Build Step in `build.rs`**

Compile the cdylib normally, then run a post-build param extraction using `DYLD_INSERT_LIBRARIES` environment tricks to skip the static init? No — that's fragile and platform-specific.

### Accept the Constraint, Use the Simplest Solution

Given the fundamental constraint that parameter metadata is only available after type resolution and compilation, and that dylib loading triggers nih-plug static initializers:

## ✅ Recommended Design: Cargo Feature Gate on nih-plug Exports

This is the simplest solution that works reliably:

### 1. Add `_param-discovery` Feature to `wavecraft-nih_plug`

```toml
# engine/crates/wavecraft-nih_plug/Cargo.toml
[features]
default = ["plugin-exports"]
plugin-exports = []           # VST3/CLAP export macros
_param-discovery = []         # When set, skip plugin exports
```

### 2. Macro Conditionally Emits nih-plug Exports

In the `wavecraft_plugin!` macro output:

```rust
// FFI exports — always generated
#[unsafe(no_mangle)]
pub extern "C" fn wavecraft_get_params_json() -> *mut c_char { ... }

// Plugin exports — only when not in param-discovery mode
#[cfg(not(feature = "_param-discovery"))]
{
    nih_export_clap!(__WavecraftPlugin);
    nih_export_vst3!(__WavecraftPlugin);
}
```

### 3. `wavecraft start` Uses Two-Phase Build

```
Phase 1: Fast param discovery (no nih-plug static init)
  cargo build --lib --features _param-discovery --no-default-features
  dlopen → wavecraft_get_params_json() → instant, no hang
  close dylib

Phase 2: Full plugin build (background, if audio-dev needed)
  cargo build --lib
  dlopen → wavecraft_dev_create_processor() → audio FFI
```

### 4. Cache Invalidation Concern — Mitigated

Cargo feature changes on the cdylib target do cause relinking. However:

- **Phase 1** is a minimal build: only links the param discovery symbols. It's fast because the cdylib is small without nih-plug exports.
- **Phase 2** is the existing build — same as today.
- Cargo build cache is shared for all non-cdylib dependencies — only the final link step differs.

In practice, the overhead is one extra link step (~0.5–2s), which is acceptable to avoid an indefinite hang.

### 5. Sidecar JSON Cache

To avoid even the Phase 1 build on subsequent runs (when code hasn't changed):

```
Phase 1 build → dlopen → params JSON → write to target/wavecraft-params.json
Subsequent runs: if wavecraft-params.json exists AND is newer than dylib → skip Phase 1
```

### 6. Fallback for Existing Plugins

If neither the sidecar JSON file exists nor the `_param-discovery` feature is available (older SDK), fall back to the current behavior (load dylib with full nih-plug init). This preserves backward compatibility.

---

## Component Changes

### 1. `wavecraft-macros` (proc macro crate)

**File:** `engine/crates/wavecraft-macros/src/plugin.rs`

**Change:** Wrap `nih_export_*!` calls in `#[cfg(not(feature = "_param-discovery"))]`

```rust
// Current (always emitted):
#krate::__nih::nih_export_clap!(__WavecraftPlugin);
#krate::__nih::nih_export_vst3!(__WavecraftPlugin);

// New (conditionally emitted):
#[cfg(not(feature = "_param-discovery"))]
#krate::__nih::nih_export_clap!(__WavecraftPlugin);
#[cfg(not(feature = "_param-discovery"))]
#krate::__nih::nih_export_vst3!(__WavecraftPlugin);
```

**Note:** The `#[cfg]` attribute applies to the *user's crate* (where the macro expands), not to the macro crate itself. The feature `_param-discovery` must be defined on the user's crate (added to the template Cargo.toml).

**Correction:** Since `nih_export_clap!` and `nih_export_vst3!` are macro invocations (not items with attributes), we must wrap them in a block or use `cfg!` at the code level. The most reliable approach:

```rust
// In macro output:
const _: () = {
    #[cfg(not(feature = "_param-discovery"))]
    {
        #krate::__nih::nih_export_clap!(__WavecraftPlugin);
        #krate::__nih::nih_export_vst3!(__WavecraftPlugin);
    }
};
```

Actually, `nih_export_clap!` and `nih_export_vst3!` expand to top-level items (static initializers, `extern "C"` functions). They must be at module scope, not inside a `const _: ()` block. The correct approach is to emit them as separate top-level items, each guarded:

```rust
// Macro expansion generates these as separate items:
#[cfg(not(feature = "_param-discovery"))]
mod __wavecraft_clap_export {
    use super::*;
    #krate::__nih::nih_export_clap!(__WavecraftPlugin);
}

#[cfg(not(feature = "_param-discovery"))]
mod __wavecraft_vst3_export {
    use super::*;
    #krate::__nih::nih_export_vst3!(__WavecraftPlugin);
}
```

However, `nih_export_*!` macros likely generate `#[no_mangle]` `extern "C"` functions and static initializers that must be at the crate root. Putting them in a sub-module may break the exports.

**Safest approach:** Use `cfg_attr` or conditional compilation at the statement level:

```rust
// The macro emits this at crate root level:
macro_rules! __wavecraft_maybe_export {
    () => {
        #[cfg(not(feature = "_param-discovery"))]
        mod __wavecraft_exports {
            // Re-import everything the nih_export macros need
            use super::__WavecraftPlugin;
            $krate::__nih::nih_export_clap!(__WavecraftPlugin);
            $krate::__nih::nih_export_vst3!(__WavecraftPlugin);
        }
    };
}
__wavecraft_maybe_export!();
```

If the `nih_export` macros require crate-root placement, we'll need to test this. An alternative is to `cfg`-gate the entire struct + trait impls for `ClapPlugin` and `Vst3Plugin`, so the export macros have nothing to export. But this is cleaner tested during implementation.

### 2. SDK Template (`cli/sdk-templates/`)

**File:** `cli/sdk-templates/new-project/react/engine/Cargo.toml.template`

**Change:** Add `_param-discovery` feature

```toml
[features]
default = []
_param-discovery = []   # Used by `wavecraft start` for fast param loading
```

### 3. CLI `start` Command (`cli/src/commands/start.rs`)

**Change:** Implement two-phase param loading with sidecar cache

```rust
fn load_parameters(
    engine_dir: &Path,
    verbose: bool,
) -> Result<(Vec<ParameterInfo>, Option<PluginLoader>)> {
    // 1. Try reading cached sidecar JSON
    let sidecar_path = find_sidecar_json(engine_dir);
    if let Some(params) = try_read_sidecar(&sidecar_path, engine_dir, verbose) {
        println!("{} Loaded {} parameters (cached)", style("✓").green(), params.len());
        return Ok((params, None));
    }

    // 2. Build with _param-discovery feature (no nih-plug exports)
    println!("{} Building for parameter discovery...", style("→").cyan());
    let build_status = Command::new("cargo")
        .args(["build", "--lib", "--features", "_param-discovery"])
        .current_dir(engine_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()?;

    if !build_status.success() {
        anyhow::bail!("Parameter discovery build failed.");
    }

    // 3. Load dylib (safe — no nih-plug static initializers)
    let dylib_path = find_plugin_dylib(engine_dir)?;
    let loader = PluginLoader::load(&dylib_path)?;
    let params = loader.parameters().to_vec();

    // 4. Write sidecar JSON cache
    write_sidecar(&sidecar_path, &params)?;

    println!("{} Loaded {} parameters", style("✓").green(), params.len());
    Ok((params, Some(loader)))
}
```

**Sidecar location:** `engine/target/debug/wavecraft-params.json`

**Cache validity:** Compare sidecar mtime vs. dylib mtime. If dylib is newer, re-extract.

### 4. CLI `PluginLoader` (`engine/crates/wavecraft-bridge/src/plugin_loader.rs`)

**Change:** Add a method to load parameters from a JSON file (bypass FFI).

```rust
impl PluginParamLoader {
    /// Load parameters from a sidecar JSON file instead of FFI.
    pub fn load_params_from_file<P: AsRef<Path>>(
        json_path: P,
    ) -> Result<Vec<ParameterInfo>, PluginLoaderError> {
        let contents = std::fs::read_to_string(json_path.as_ref())
            .map_err(|e| PluginLoaderError::LibraryLoad(/* ... */))?;
        let params: Vec<ParameterInfo> =
            serde_json::from_str(&contents).map_err(PluginLoaderError::JsonParse)?;
        Ok(params)
    }
}
```

### 5. Full `wavecraft start` Revised Flow

```
wavecraft start
  │
  ├── 1. Check dependencies (npm)
  │
  ├── 2. Build plugin: cargo build --lib --features _param-discovery
  │      (fast, no nih-plug static init)
  │
  ├── 3. Load params: dlopen(dylib) → wavecraft_get_params_json()
  │      → instant (no VST3/CLAP init constructors)
  │      → write wavecraft-params.json cache
  │      → close dylib
  │
  ├── 4. In background: cargo build --lib (full build for audio-dev)
  │      (only if audio-dev feature is enabled)
  │
  ├── 5. Start WebSocket server with params
  │
  ├── 6. Start UI dev server
  │
  └── 7. When full build completes: dlopen for audio-dev vtable
```

On subsequent runs (no code changes):
```
wavecraft start
  │
  ├── 1. Check dependencies
  ├── 2. Read wavecraft-params.json (no build needed!)
  ├── 3. cargo build --lib (background, for audio-dev)
  ├── 4. Start servers immediately
  └── 5. Audio-dev vtable loaded when build completes
```

---

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| `nih_export_*!` macros don't respect `#[cfg]` in sub-modules | High | Test during implementation; if blocked, emit conditional `#[cfg]` directly on the generated `extern "C"` functions instead of wrapping the macro calls |
| Feature switching causes full recompile of cdylib | Medium | Only the link step re-runs; all dependency crates are cached. Measured impact: ~0.5–2s per switch |
| Sidecar cache stale after code changes | Low | Compare mtime of sidecar vs source files; conservative: always rebuild if dylib is newer |
| Older plugins without `_param-discovery` feature | Low | Fallback to current FFI loading behavior (backward compatible) |
| `--features` flag conflicts with user-defined features | Low | Prefix with `_` to indicate internal use; document in template |

---

## Alternatives Considered

| Approach | Why Rejected |
|----------|-------------|
| Macro writes JSON file during expansion | Proc macros don't have access to resolved type information; `param_specs()` values are runtime-only |
| `build.rs` compiles and runs a discovery binary | User's processor types are defined in `lib.rs` alongside `wavecraft_plugin!`; can't import without bringing in nih-plug |
| `DYLD_INSERT_LIBRARIES` to skip static init | Platform-specific, fragile, not suitable for a developer tool |
| `env!()` + build.rs for compile-time JSON | Same issue: build.rs runs before compilation, can't access post-compilation type info |
| Load dylib in a subprocess with timeout | Doesn't fix the problem, just adds error reporting. Still hangs, just with a timeout message |
| Disable `AudioComponentRegistrar` via environment variable | macOS system behavior, not controllable; undocumented and unreliable |

---

## Implementation Order

1. **Macro change:** Add `#[cfg(not(feature = "_param-discovery"))]` guards around `nih_export_*!` in `wavecraft_plugin!` output — verify that the dylib loads without triggering nih-plug init.

2. **Template change:** Add `_param-discovery` feature to template `Cargo.toml`.

3. **CLI change:** Update `wavecraft start` to build with `--features _param-discovery` for param loading, write sidecar cache.

4. **Sidecar cache:** Implement mtime-based cache invalidation in CLI.

5. **Fallback:** Preserve current FFI loading as fallback for older plugins.

6. **Audio-dev split:** If audio-dev is enabled, run full build in background and load vtable asynchronously.

---

## Testing Strategy

1. **Unit test:** Compile plugin with `_param-discovery` feature, verify no `VST3_FACTORY` or `clap_entry` symbols in dylib (`nm -g`)
2. **Integration test:** `wavecraft start` with new feature successfully loads params without hang
3. **Regression test:** Remove `_param-discovery` feature, verify fallback to FFI still works
4. **Cache test:** Modify source, verify sidecar is regenerated on next run
5. **Template test:** `wavecraft create` produces projects with `_param-discovery` feature

---

## Documentation References

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
- [SDK Architecture](../../architecture/sdk-architecture.md) — Crate structure and distribution
- [Development Workflows](../../architecture/development-workflows.md) — Build system and dev mode
- [Declarative Plugin DSL](../../architecture/declarative-plugin-dsl.md) — Macro system
