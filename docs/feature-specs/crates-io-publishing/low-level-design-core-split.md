# Low-Level Design: wavecraft-core Crate Split

**Status:** Draft  
**Created:** 2026-02-06  
**Updated:** 2026-02-06  
**Author:** Architect Agent  
**Parent Feature:** crates-io-publishing  
**Prerequisite for:** [crates.io Publishing LLD](./low-level-design-crates-io-publishing.md)

---

## Problem Statement

`wavecraft-core` is the main SDK crate that users depend on. It **cannot** be published to crates.io because it depends on `nih_plug`, which is distributed exclusively via Git and is **not on crates.io**.

crates.io enforces a strict rule: every transitive dependency of a published crate must itself be on crates.io. There is no workaround — wrapping `nih_plug` in another crate, re-exporting it, or feature-gating it does not bypass this constraint. If any crate in the dependency tree is Git-only, the root crate is unpublishable.

### Current Dependency Graph (Simplified)

```
wavecraft-core (UNPUBLISHABLE)
├── nih_plug ⚠️ (Git-only, NOT on crates.io)
├── wavecraft-protocol ✅
├── wavecraft-dsp ✅
├── wavecraft-metering ✅
├── wavecraft-bridge ✅
├── wavecraft-macros ✅ (proc-macro, no nih_plug link dep)
├── wry ✅
├── include_dir ✅
├── paste ✅
└── platform deps ✅ (objc2, windows)
```

The only unpublishable dependency is `nih_plug`. Everything else is on crates.io.

---

## Goals

| Goal | Priority |
|------|----------|
| Publish `wavecraft-core` to crates.io as the main SDK entry point | **Critical** |
| User-facing import: `use wavecraft::prelude::*` | **Critical** |
| Remove all `::nih_plug::` references from proc-macro generated code | High |
| Keep nih_plug integration functional for actual plugin builds | High |
| Users depend on a single crate in their `Cargo.toml` | High |
| Clean architectural boundary between publishable SDK and host-specific glue | High |
| No duplication of code between crates | Medium |

---

## Non-Goals

- Getting `nih_plug` published to crates.io (out of our control)
- Supporting alternative plugin frameworks (JUCE, etc.)
- Renaming the CLI crate (remains `wavecraft` on crates.io)

---

## Solution: Split wavecraft-core into Two Crates

### Architectural Principle

**Unpublishable code depends on publishable code. Never the reverse.**

The split creates a clear, enforceable boundary:

```
┌─────────────────────────────────────────────────────────────────────────┐
│               CRATE SPLIT ARCHITECTURE                                  │
└─────────────────────────────────────────────────────────────────────────┘

  PUBLISHABLE (crates.io)              GIT-ONLY (depends on publishable)
  ─────────────────────────            ──────────────────────────────────

  ┌─────────────────────┐             ┌───────────────────────────────┐
  │  wavecraft-core     │             │  wavecraft-nih_plug           │
  │  (SDK library)      │◄────────────│  (nih_plug integration)       │
  │                     │  depends on │                               │
  │  • wavecraft-dsp    │             │  • nih_plug (git)             │
  │  • wavecraft-macros │             │  • wavecraft-core             │
  │  • wavecraft-       │             │                               │
  │    protocol         │             │  Contains:                    │
  │  • wavecraft-       │             │  • Plugin trait impl          │
  │    metering         │             │  • Editor (WKWebView)         │
  │  • wavecraft-bridge │             │  • WavecraftParams            │
  │                     │             │  • calculate_stereo_meters    │
  │  Contains:          │             │  • wavecraft_plugin! (decl)   │
  │  • wavecraft_       │             │  • nih_plug re-exports (__nih)│
  │    processor! macro │             │  • WavecraftPlugin (ref impl) │
  │  • DSP re-exports   │             │                               │
  │  • Protocol types   │             │  NOT on crates.io             │
  │  • Bridge types     │             │  publish = false              │
  │  • Metering types   │             └───────────────────────────────┘
  │  • paste re-export  │                          ▲
  └─────────────────────┘                          │
                                                   │ depends on
                                                   │ (via Cargo rename)
                                        ┌──────────┴──────────┐
                                        │  plugin-template    │
                                        │  (user project)     │
                                        │                     │
                                        │  Cargo.toml:        │
                                        │  wavecraft =        │
                                        │    { package =      │
                                        │    "wavecraft-      │
                                        │     nih_plug" }     │
                                        │                     │
                                        │  lib.rs:            │
                                        │  use wavecraft::    │
                                        │    prelude::*;      │
                                        └─────────────────────┘
```

### Naming

| Crate | Cargo Name | Rust Identifier | Published |
|-------|------------|-----------------|-----------|
| SDK library | `wavecraft-core` | `wavecraft_core` | ✅ crates.io |
| nih_plug integration | `wavecraft-nih_plug` | `wavecraft_nih_plug` | ❌ Git-only |
| User alias (Cargo rename) | `wavecraft` | `wavecraft` | — (alias only) |

**Why `wavecraft-nih_plug`?**
- Explicitly names the plugin framework it integrates with
- Follows crate naming convention (`wavecraft-*`)
- Unambiguous — users know exactly what this crate wraps
- The underscore mirrors `nih_plug`'s own naming convention

### User-Facing Import: `use wavecraft::prelude::*`

Achieved via **Cargo package rename** in the user's `Cargo.toml`:

```toml
[dependencies]
wavecraft = { package = "wavecraft-nih_plug", git = "...", tag = "v0.7.1" }
```

This means:
- `wavecraft::prelude::*` resolves to `wavecraft_nih_plug::prelude::*`
- Users never type `wavecraft_nih_plug` in Rust code
- Template scaffolds this automatically

**Note:** The `wavecraft` name on crates.io is reserved for the CLI binary (`cargo install wavecraft`). The Cargo rename is a *local alias* in the user's project — it does not conflict with the CLI's crates.io registration.

---

## Removing `::nih_plug::` from Proc-Macro Output

### Current State

The proc-macro (`wavecraft-macros/src/plugin.rs`) generates ~40 `::nih_plug::prelude::*` tokens:

```rust
// Current proc-macro output (simplified):
impl ::nih_plug::prelude::Plugin for __WavecraftPlugin { ... }
impl ::nih_plug::prelude::ClapPlugin for __WavecraftPlugin { ... }
impl ::nih_plug::prelude::Vst3Plugin for __WavecraftPlugin { ... }
::nih_plug::nih_export_clap!(__WavecraftPlugin);
::nih_plug::nih_export_vst3!(__WavecraftPlugin);
```

This forces users to have `nih_plug` resolvable in their crate scope — leaking an implementation detail.

### Proposed: Route Through `wavecraft-nih_plug` Re-exports

`wavecraft-nih_plug` exposes a hidden re-export module:

```rust
// wavecraft-nih_plug/src/lib.rs

/// Hidden re-exports for proc-macro generated code.
/// DO NOT use directly — these are an implementation detail.
#[doc(hidden)]
pub mod __nih {
    // Traits
    pub use nih_plug::prelude::{
        Plugin, ClapPlugin, Vst3Plugin, Params, Param, Editor,
    };
    // Types
    pub use nih_plug::prelude::{
        FloatParam, FloatRange, ParamPtr, AudioIOLayout, MidiConfig,
        Buffer, AuxiliaryBuffers, ProcessStatus, BufferConfig,
        AsyncExecutor, Vst3SubCategory, ClapFeature,
    };
    // Export macros
    pub use nih_plug::{nih_export_clap, nih_export_vst3};
}
```

The proc-macro then generates `::wavecraft_nih_plug::__nih::Plugin` instead of `::nih_plug::prelude::Plugin`:

```rust
// NEW proc-macro output (simplified):
impl ::wavecraft_nih_plug::__nih::Plugin for __WavecraftPlugin { ... }
impl ::wavecraft_nih_plug::__nih::ClapPlugin for __WavecraftPlugin { ... }
impl ::wavecraft_nih_plug::__nih::Vst3Plugin for __WavecraftPlugin { ... }
::wavecraft_nih_plug::__nih::nih_export_clap!(__WavecraftPlugin);
::wavecraft_nih_plug::__nih::nih_export_vst3!(__WavecraftPlugin);
```

### Why `::wavecraft_nih_plug::` and Not `::wavecraft::`?

The proc-macro generates tokens that resolve in the **consumer crate's** scope. The path must match the crate's real identity:

| Consumer | `Cargo.toml` entry | `::wavecraft_nih_plug::` resolves? | `::wavecraft::` resolves? |
|----------|---------------------|------------------------------------|---------------------------|
| User plugin (with rename) | `wavecraft = { package = "wavecraft-nih_plug" }` | ❌ (replaced by alias) | ✅ |
| User plugin (no rename) | `wavecraft-nih_plug = { git = "..." }` | ✅ | ❌ |
| wavecraft-nih_plug itself (tests) | N/A (is the crate) | ✅ (via auto extern) | ❌ |

The Cargo rename (`wavecraft = { package = "..." }`) **replaces** the original name — `::wavecraft_nih_plug::` no longer resolves in the consumer.

**Problem:** We can't use `::wavecraft_nih_plug::` if the template uses a rename, and we can't use `::wavecraft::` for internal tests.

**Solution:** The proc-macro defaults to `::wavecraft_nih_plug::` (works for internal tests and non-renamed usage). For Cargo-renamed consumers, the proc-macro accepts an optional `crate` attribute:

```rust
// User plugin (template-generated):
wavecraft_plugin! {
    name: "My Plugin",
    vendor: "My Company",
    signal: MyGain,
    crate: wavecraft,         // ← tells the macro which path to use  
}
```

If omitted, the macro defaults to `::wavecraft_nih_plug::`. The plugin template always includes `crate: wavecraft`.

### Benefits

1. **Users never see `nih_plug`** — not in `Cargo.toml`, not in `use` statements, not in macro output
2. **Decoupled from nih_plug internals** — if nih_plug renames a module, only `wavecraft-nih_plug`'s `__nih` needs updating
3. **Clean compile errors** — errors reference `wavecraft::__nih::Plugin` rather than a mystery `::nih_plug::` crate
4. **Testable internally** — `wavecraft-nih_plug`'s own tests work without the `crate` attribute

### Declarative Macro: Uses `$crate`

The declarative `wavecraft_plugin!` macro (the one in `macros.rs`) can use `$crate::` which automatically resolves correctly regardless of renaming:

```rust
// In wavecraft-nih_plug/src/macros.rs:
#[macro_export]
macro_rules! wavecraft_plugin {
    (...) => {
        // $crate:: resolves to wavecraft_nih_plug (or wavecraft if renamed)
        impl $crate::__nih::Plugin for $ident { ... }
        $crate::__nih::nih_export_vst3!(...);
        $crate::__nih::nih_export_clap!(...);
    }
}
```

`$crate` is a special metavariable in declarative macros that always resolves to the defining crate — **it works regardless of how the consumer renames the dependency**. This is strictly superior to hardcoded paths.

---

## Module Migration Plan

### What Stays in wavecraft-core (Publishable)

These modules have **zero** `nih_plug` imports:

| Module/Item | Current Location | Rationale |
|-------------|------------------|-----------|
| `wavecraft_processor!` macro | `src/macros.rs` | Only references `::wavecraft_dsp::` — no nih_plug tokens |
| `paste` re-export | `src/lib.rs` | Used by macros, crates.io dependency |
| SDK prelude (partial) | `src/prelude.rs` | Re-exports from wavecraft-dsp, protocol, metering, bridge |

**New `wavecraft-core/src/prelude.rs`:**
```rust
//! SDK prelude — publishable types for building Wavecraft plugins.

// DSP traits and types
pub use wavecraft_dsp::{Chain, ParamRange, ParamSpec, Processor, ProcessorParams, Transport};
pub use wavecraft_dsp::builtins::{GainDsp, PassthroughDsp};

// Protocol types
pub use wavecraft_protocol::{ParamId, ParameterInfo, ParameterType, db_to_linear};

// Metering
pub use wavecraft_metering::{MeterConsumer, MeterFrame, MeterProducer, create_meter_channel};

// Derive macros (proc-macro — publishable, no nih_plug link dependency)
pub use wavecraft_macros::ProcessorParams as DeriveProcessorParams;

// Declarative macro
pub use crate::wavecraft_processor;
```

Note: `wavecraft_plugin!` (proc-macro) is **NOT** re-exported from wavecraft-core. It moves to `wavecraft-nih_plug::prelude`.

**New `wavecraft-core/src/lib.rs`:**
```rust
//! Wavecraft Core — Audio plugin SDK library.
//!
//! This crate provides the publishable SDK components:
//! - DSP traits and built-in processors (via wavecraft-dsp)
//! - Parameter types and protocol (via wavecraft-protocol)
//! - IPC bridge types (via wavecraft-bridge)
//! - Real-time metering (via wavecraft-metering)
//! - Declarative macros (wavecraft_processor!)
//! - ProcessorParams derive macro (via wavecraft-macros)
//!
//! For nih-plug integration (Plugin trait impl, Editor, exports),
//! use `wavecraft-nih_plug` which depends on this crate.

#![allow(clippy::crate_in_macro_def)]

pub mod macros;
pub mod prelude;

// Re-export paste for macro usage
pub use paste;
```

### What Moves to wavecraft-nih_plug (Git-only)

Every module that imports `nih_plug`:

| Module | Current Location | nih_plug Usage |
|--------|------------------|----------------|
| `WavecraftPlugin` struct + impls | `core/src/lib.rs` | `Plugin`, `Vst3Plugin`, `ClapPlugin`, `Buffer`, `ProcessStatus` |
| `WavecraftParams` | `core/src/params.rs` | `Params` derive, `FloatParam`, `FloatRange`, `SmoothingStyle` |
| `WavecraftEditor` | `core/src/editor/mod.rs` | `Editor` trait, `Params`, `ParentWindowHandle`, `GuiContext` |
| `PluginEditorBridge` | `core/src/editor/bridge.rs` | `Params`, `GuiContext`, `ParamPtr` |
| `WebViewHandle` / `WebViewConfig` | `core/src/editor/webview.rs` | `Params`, `ParentWindowHandle`, `GuiContext` |
| Asset embedding | `core/src/editor/assets.rs` | No direct nih_plug import, but used exclusively by editor |
| macOS native code | `core/src/editor/macos.rs` | Used exclusively by editor |
| Windows native code | `core/src/editor/windows.rs` | Used exclusively by editor |
| JS bridge helpers | `core/src/editor/js/` | Used exclusively by editor |
| `calculate_stereo_meters` | `core/src/util.rs` | `nih_plug::prelude::Buffer` parameter type |
| `wavecraft_plugin!` (decl macro) | `core/src/macros.rs` | Generates `nih_plug::prelude::Plugin`, etc. |

### Declarative Macro Split Detail

**Current `macros.rs` (327 lines) contains two macros:**
1. `wavecraft_processor!` — References only `::wavecraft_dsp::*`. **Stays in wavecraft-core.**
2. `wavecraft_plugin!` (declarative) — Generates `nih_plug::prelude::*` tokens. **Moves to wavecraft-nih_plug.** After the move, it uses `$crate::__nih::` instead of `nih_plug::prelude::`.

**After split:**

```
wavecraft-core/src/macros.rs:
  └── wavecraft_processor!           (no nih_plug, publishable)

wavecraft-nih_plug/src/macros.rs:
  └── wavecraft_plugin! (decl)       (uses $crate::__nih::, git-only)
```

---

## New Crate: wavecraft-nih_plug

### Cargo.toml

```toml
[package]
name = "wavecraft-nih_plug"
version = "0.7.1"
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Wavecraft nih-plug integration — Plugin, Editor, and host glue"
publish = false   # ← CANNOT publish (nih_plug is Git-only)

[lib]
name = "wavecraft_nih_plug"
crate-type = ["cdylib", "rlib"]

[features]
default = []
assert_process_allocs = ["nih_plug/assert_process_allocs"]

[dependencies]
# Publishable SDK (from crates.io in Phase 2, path for now)
wavecraft-core.workspace = true
wavecraft-protocol.workspace = true
wavecraft-dsp.workspace = true
wavecraft-metering.workspace = true
wavecraft-bridge.workspace = true

# Git-only dependency (the reason this crate is unpublishable)
nih_plug.workspace = true

# Same deps as current wavecraft-core
wry = { version = "0.47", default-features = false, features = ["os-webview"] }
raw-window-handle = "0.6"
serde_json = "1.0"
include_dir = "0.7"
paste = "1.0"

[dev-dependencies]
trybuild = "1.0"

[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-foundation = "0.2"
objc2-app-kit = "0.2"
objc2-web-kit = "0.2"
block2 = "0.5"

[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_Foundation",
    "Win32_Graphics_Gdi",
    "Win32_System_Com",
    "Win32_System_LibraryLoader",
    "Win32_System_WinRT",
    "Win32_UI_WindowsAndMessaging",
] }
webview2-com = "0.33"
```

### Module Structure

```
engine/crates/wavecraft-nih_plug/
├── Cargo.toml
└── src/
    ├── lib.rs              # WavecraftPlugin + Plugin/Vst3/Clap impls + __nih module
    ├── prelude.rs          # Re-exports wavecraft_core::prelude + nih_plug types
    ├── macros.rs           # wavecraft_plugin! declarative macro (uses $crate::__nih::)
    ├── params.rs           # WavecraftParams (nih_plug Params bridge)
    ├── util.rs             # calculate_stereo_meters(buffer: &Buffer)
    └── editor/
        ├── mod.rs          # WavecraftEditor<P: Params>
        ├── assets.rs       # Static UI asset embedding
        ├── bridge.rs       # PluginEditorBridge<P: Params>
        ├── webview.rs      # WebViewHandle trait, WebViewConfig
        ├── macos.rs        # macOS WKWebView impl
        ├── windows.rs      # Windows WebView2 impl
        └── js/
            ├── mod.rs      # JavaScript injection helpers
            └── ...
```

### lib.rs: Hidden Re-exports for Macro Support

```rust
//! Wavecraft nih-plug integration.
//!
//! This crate bridges the publishable Wavecraft SDK (wavecraft-core)
//! with nih-plug for actual VST3/CLAP plugin compilation.

pub mod editor;
pub mod macros;
pub mod prelude;
pub mod util;

mod params;

pub use paste;

// Re-export wavecraft-core for macro access
pub use wavecraft_core;

/// Hidden re-exports used by macro-generated code.
/// This module is an implementation detail — do not depend on it directly.
#[doc(hidden)]
pub mod __nih {
    // Traits
    pub use nih_plug::prelude::{
        Plugin, ClapPlugin, Vst3Plugin, Params, Param, Editor,
        InitContext, ProcessContext,
    };
    // Types
    pub use nih_plug::prelude::{
        FloatParam, FloatRange, ParamPtr, AudioIOLayout, MidiConfig,
        Buffer, AuxiliaryBuffers, ProcessStatus, BufferConfig,
        AsyncExecutor, Vst3SubCategory, ClapFeature, SmoothingStyle,
    };
    // Export macros
    pub use nih_plug::{nih_export_clap, nih_export_vst3};
}
```

### Prelude Re-exports

```rust
//! Prelude for nih-plug plugin authors.
//!
//! This re-exports everything from wavecraft-core's prelude plus
//! nih-plug types needed for actual plugin compilation.
//!
//! ```rust
//! use wavecraft::prelude::*;  // via Cargo rename
//! ```

// Everything from the publishable SDK
pub use wavecraft_core::prelude::*;

// nih-plug essentials (for users who need direct nih-plug access)
pub use nih_plug::prelude::*;
pub use nih_plug::{nih_export_clap, nih_export_vst3};

// Editor (platform-specific)
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use crate::editor::WavecraftEditor;

// Utility functions that depend on nih_plug::Buffer
pub use crate::util::calculate_stereo_meters;

// Re-export the wavecraft_plugin! proc-macro (from wavecraft-macros)
pub use wavecraft_macros::wavecraft_plugin;

// Re-export the nih-aware declarative macro
pub use crate::wavecraft_plugin_decl;
```

---

## Proc-Macro Changes (`wavecraft-macros`)

### Current: Generates `::nih_plug::` Paths

```rust
// wavecraft-macros/src/plugin.rs (current)
let expanded = quote! {
    impl ::nih_plug::prelude::Plugin for __WavecraftPlugin { ... }
    ::nih_plug::nih_export_clap!(__WavecraftPlugin);
};
```

### After: Generates `::wavecraft_nih_plug::__nih::` Paths (with Override)

The proc-macro accepts an optional `crate` field. If present, it uses that path; otherwise defaults to `::wavecraft_nih_plug`:

```rust
// wavecraft-macros/src/plugin.rs (new)
struct PluginDef {
    name: LitStr,
    vendor: LitStr,
    url: Option<LitStr>,
    email: Option<LitStr>,
    signal: Expr,
    krate: Option<syn::Path>,   // NEW: optional `crate` override
}

pub fn wavecraft_plugin_impl(input: TokenStream) -> TokenStream {
    let plugin_def = parse_macro_input!(input as PluginDef);
    
    // Default to ::wavecraft_nih_plug:: if no crate override
    let krate = plugin_def.krate
        .unwrap_or_else(|| syn::parse_quote!(::wavecraft_nih_plug));
    
    let expanded = quote! {
        impl #krate::__nih::Plugin for __WavecraftPlugin { ... }
        #krate::__nih::nih_export_clap!(__WavecraftPlugin);
        #krate::__nih::nih_export_vst3!(__WavecraftPlugin);
        
        fn editor(...) {
            #krate::editor::create_webview_editor(...)
        }
    };
}
```

**Template-generated user code:**

```rust
use wavecraft::prelude::*;

wavecraft_processor!(MyGain => Gain);

wavecraft_plugin! {
    name: "My Plugin",
    vendor: "My Company",
    signal: MyGain,
    crate: wavecraft,    // matches the Cargo rename
}
```

**Internal test code (no rename needed):**

```rust
// In wavecraft-nih_plug's own tests:
wavecraft_plugin! {
    name: "Test Plugin",
    vendor: "Test",
    signal: TestProcessor,
    // No `crate:` field → defaults to ::wavecraft_nih_plug::
}
```

---

## Dependency Graph: Before and After

### Before (Current)

```
                  ┌─────────────────┐
                  │  wavecraft-core │ ← UNPUBLISHABLE
                  │  (cdylib + rlib)│
                  └────────┬────────┘
                           │
          ┌────────────────┼────────────────┬──────────────┐
          ▼                ▼                ▼              ▼
   wavecraft-dsp    wavecraft-bridge  wavecraft-metering  nih_plug ⚠️
          │                │
          ▼                ▼
   wavecraft-protocol  wavecraft-protocol
   wavecraft-macros¹

   ¹ wavecraft-macros is a proc-macro; no nih_plug link dependency
```

### After (Proposed)

```
                  ┌──────────────────────┐
                  │ wavecraft-nih_plug   │ ← Git-only (publish = false)
                  │ (cdylib + rlib)      │
                  └────────┬─────────────┘
                           │
          ┌────────────────┼───────────────────┐
          ▼                ▼                   ▼
   wavecraft-core    nih_plug ⚠️          (platform deps)
   (rlib only)        (Git-only)
          │
          ├── wavecraft-dsp ✅
          ├── wavecraft-bridge ✅
          ├── wavecraft-metering ✅
          ├── wavecraft-macros ✅ (proc-macro)
          └── wavecraft-protocol ✅

   ALL ✅ crates publishable to crates.io
```

Note: `wavecraft-core` changes from `cdylib + rlib` to **`rlib` only**. The `cdylib` output (the .dylib/.dll that DAW hosts load) is produced by `wavecraft-nih_plug` or by user plugin crates.

---

## Plugin-Template Impact

### Current Template

```toml
# plugin-template/engine/Cargo.toml (current)
[dependencies]
wavecraft-core = { git = "...", tag = "{{sdk_version}}" }
wavecraft-protocol = { git = "...", tag = "{{sdk_version}}" }
wavecraft-dsp = { git = "...", tag = "{{sdk_version}}" }
wavecraft-bridge = { git = "...", tag = "{{sdk_version}}" }
wavecraft-metering = { git = "...", tag = "{{sdk_version}}" }
nih_plug = { git = "...", rev = "28b149ec..." }
```

```rust
// plugin-template/engine/src/lib.rs (current)
use wavecraft_core::prelude::*;
```

### After Template

```toml
# plugin-template/engine/Cargo.toml (new)
[dependencies]
# Single SDK dependency — Cargo rename gives us `use wavecraft::prelude::*`
wavecraft = { package = "wavecraft-nih_plug", git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }

# Build dependencies for nih-plug bundling
[build-dependencies]
nih_plug_xtask = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "28b149ec..." }
```

```rust
// plugin-template/engine/src/lib.rs (new)
use wavecraft::prelude::*;

wavecraft_processor!({{plugin_name_pascal}}Gain => Gain);

wavecraft_plugin! {
    name: "{{plugin_name_title}}",
    vendor: "{{vendor}}",
    signal: {{plugin_name_pascal}}Gain,
    crate: wavecraft,
}
```

**Key improvements:**
- Single `wavecraft` dependency instead of 6 separate crates + nih_plug
- `use wavecraft::prelude::*` — clean, SDK-branded import
- No `nih_plug` in user dependencies (only in build-dependencies for bundling)
- `crate: wavecraft` in macro tells the proc-macro to use `::wavecraft::__nih::` paths

---

## Standalone Crate Impact

The `standalone` crate currently depends on `wavecraft-bridge` and `wavecraft-protocol` only — NOT on `wavecraft-core` or `nih_plug`. **No changes needed.**

---

## Workspace Configuration Changes

### `engine/Cargo.toml` Updates

```toml
[workspace]
members = ["crates/*", "xtask"]
resolver = "2"

[workspace.dependencies]
# nih-plug framework
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "28b149ec..." }

# Internal crates (all with version for crates.io publishing)
wavecraft-core = { path = "crates/wavecraft-core", version = "0.7.1" }
wavecraft-nih_plug = { path = "crates/wavecraft-nih_plug" }   # no version (publish = false)
wavecraft-protocol = { path = "crates/wavecraft-protocol", version = "0.7.1" }
wavecraft-macros = { path = "crates/wavecraft-macros", version = "0.7.1" }
wavecraft-metering = { path = "crates/wavecraft-metering", version = "0.7.1" }
wavecraft-bridge = { path = "crates/wavecraft-bridge", version = "0.7.1" }
wavecraft-dsp = { path = "crates/wavecraft-dsp", version = "0.7.1" }
```

---

## Migration Checklist

### Phase 1: Create wavecraft-nih_plug

1. Create `engine/crates/wavecraft-nih_plug/` directory
2. Create `Cargo.toml` with `publish = false` and all current wavecraft-core deps + nih_plug
3. Move the following from wavecraft-core to wavecraft-nih_plug:
   - `src/lib.rs` → `WavecraftPlugin`, `Plugin`/`Vst3Plugin`/`ClapPlugin` impls, `nih_export_*!()` calls
   - `src/params.rs` → `WavecraftParams`
   - `src/util.rs` → `calculate_stereo_meters`
   - `src/editor/` → entire directory
   - `src/macros.rs` → only the `wavecraft_plugin!` declarative macro (becomes `wavecraft_plugin_decl!`)
4. Create `__nih` module re-exporting nih_plug types
5. Create `src/prelude.rs` that re-exports wavecraft-core prelude + nih_plug + wavecraft_plugin! proc-macro

### Phase 2: Strip wavecraft-core

6. Remove `nih_plug` from wavecraft-core's dependencies
7. Remove `cdylib` from wavecraft-core's `crate-type` (becomes `rlib` only)
8. Remove moved modules (editor, params, util, wavecraft_plugin! declarative macro)
9. Remove platform-specific dependencies (wry, objc2, windows, etc.)
10. Update wavecraft-core prelude to only export publishable types
11. Remove `wavecraft_plugin!` proc-macro re-export from wavecraft-core prelude
12. Keep `wavecraft_processor!` declarative macro and `ProcessorParams` derive re-export

### Phase 3: Update Proc-Macro

13. Add optional `crate` field to `PluginDef` parser
14. Replace all `::nih_plug::` in generated code with `#krate::__nih::`
15. Replace `::wavecraft_core::editor::` with `#krate::editor::`
16. Default `krate` to `::wavecraft_nih_plug`

### Phase 4: Update Template and Workspace

17. Update `engine/Cargo.toml` workspace members and dependencies
18. Update plugin-template to single `wavecraft = { package = "wavecraft-nih_plug" }` dep
19. Update plugin-template lib.rs to `use wavecraft::prelude::*` + add `crate: wavecraft`
20. Update standalone crate if needed (currently no changes required)

### Phase 5: Verify

21. `cargo build --workspace` — all crates compile
22. `cargo test --workspace` — all tests pass
23. `cargo publish --dry-run -p wavecraft-core` — publishes without error
24. `cargo publish --dry-run -p wavecraft-protocol` — publishes without error  
25. Verify plugin loads in DAW (manual test)

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Circular dependency between core and nih_plug crate | None | — | Architecture enforces one-way: nih_plug → core |
| Proc-macro `crate` attribute adds boilerplate | Low | Low | Template auto-generates it; one extra line |
| Proc-macro path resolution with Cargo rename | Medium | High | `crate: wavecraft` field handles rename; default works for internal tests |
| User confusion about which crate to depend on | Low | Medium | Template only shows `wavecraft`; docs explain the split |
| Breakage of existing WavecraftPlugin reference impl | Low | Medium | Move as-is; no logic changes, just crate relocation |
| Missing re-exports in `__nih` module | Medium | Low | Compile errors are immediate; full test suite catches these |

---

## Open Questions (Resolved)

1. **Should `wavecraft-core` keep `wry`/`include_dir`/platform deps?**
   → **No.** These are exclusively used by the editor, which moves to wavecraft-nih_plug.

2. **Should the crate be named `wavecraft-nih` or `wavecraft-nih_plug`?**
   → **`wavecraft-nih_plug`.** Explicitly names the wrapped library; mirrors nih_plug's naming convention.

3. **Can `::nih_plug::` references be removed from the macro?**
   → **Yes.** The proc-macro generates `#krate::__nih::Plugin` instead. The declarative macro uses `$crate::__nih::Plugin`. Users never see `nih_plug`.

4. **How do users get `use wavecraft::prelude::*`?**
   → **Cargo package rename.** Template has `wavecraft = { package = "wavecraft-nih_plug" }`.

5. **Will `cargo ws publish` handle the split?**
   → **Yes.** `wavecraft-nih_plug` has `publish = false`, so `cargo-workspaces` skips it.

---

## Summary

| Aspect | wavecraft-core (After) | wavecraft-nih_plug (New) |
|--------|------------------------|--------------------------|
| **Purpose** | Publishable SDK library | nih_plug integration layer |
| **Published** | ✅ crates.io | ❌ Git-only |
| **crate-type** | `rlib` | `cdylib` + `rlib` |
| **Depends on nih_plug** | ❌ | ✅ |
| **Contains DSP types** | ✅ (re-exports) | ✅ (re-exports via core) |
| **Contains Editor** | ❌ | ✅ |
| **Contains Plugin impl** | ❌ | ✅ |
| **Contains `wavecraft_processor!`** | ✅ | ❌ |
| **Contains `wavecraft_plugin!` (decl)** | ❌ | ✅ |
| **User Cargo.toml** | Not directly | `wavecraft = { package = "wavecraft-nih_plug" }` |
| **User import** | — | `use wavecraft::prelude::*;` |

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Overall architecture
- [Coding Standards](../../architecture/coding-standards.md) — Implementation conventions
- [crates.io Publishing LLD](./low-level-design-crates-io-publishing.md) — Publishing workflow (depends on this split)
