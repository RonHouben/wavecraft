# Implementation Plan: wavecraft-core Crate Split

## Overview

Split `wavecraft-core` into two crates to enable crates.io publishing:
- **`wavecraft-core`** — Publishable SDK library (no nih_plug dependency)
- **`wavecraft-nih_plug`** — Git-only nih_plug integration layer

This split is a prerequisite for the [crates.io Publishing](./low-level-design-crates-io-publishing.md) feature.

## Requirements

- Create new `wavecraft-nih_plug` crate containing all nih_plug-dependent code
- Strip `wavecraft-core` of nih_plug dependencies, making it publishable
- Update proc-macro to generate `#krate::__nih::` paths instead of `::nih_plug::`
- Update plugin-template to use Cargo rename: `wavecraft = { package = "wavecraft-nih_plug" }`
- Maintain `use wavecraft::prelude::*` user experience

## Architecture Changes

| Component | Change |
|-----------|--------|
| `engine/crates/wavecraft-nih_plug/` | **NEW:** Contains editor, params, util, Plugin impl |
| `engine/crates/wavecraft-core/` | **STRIPPED:** Becomes thin re-export + `wavecraft_processor!` macro |
| `engine/crates/wavecraft-macros/src/plugin.rs` | Add `crate` field, change paths to `#krate::__nih::` |
| `plugin-template/engine/Cargo.toml` | Single dep: `wavecraft = { package = "wavecraft-nih_plug" }` |
| `plugin-template/engine/src/lib.rs` | `use wavecraft::prelude::*` + `crate: wavecraft` in macro |
| `engine/Cargo.toml` | Add `wavecraft-nih_plug` to workspace, add versions to path deps |

---

## Implementation Steps

### Phase 1: Create wavecraft-nih_plug Crate (11 steps)

#### Step 1.1: Create Directory Structure
**Action:** Create the `wavecraft-nih_plug` crate directory and subdirectories.

```bash
mkdir -p engine/crates/wavecraft-nih_plug/src/editor/js
```

**Dependencies:** None  
**Risk:** Low

---

#### Step 1.2: Create Cargo.toml for wavecraft-nih_plug
**File:** `engine/crates/wavecraft-nih_plug/Cargo.toml`

**Action:** Create new Cargo.toml with `publish = false` and all dependencies from current wavecraft-core.

```toml
[package]
name = "wavecraft-nih_plug"
version = "0.7.1"
edition.workspace = true
license.workspace = true
authors.workspace = true
repository.workspace = true
description = "Wavecraft nih-plug integration — Plugin, Editor, and host glue"
publish = false

[lib]
name = "wavecraft_nih_plug"
crate-type = ["cdylib", "rlib"]

[features]
default = []
assert_process_allocs = ["nih_plug/assert_process_allocs"]

[dependencies]
wavecraft-core.workspace = true
wavecraft-protocol.workspace = true
wavecraft-dsp.workspace = true
wavecraft-metering.workspace = true
wavecraft-bridge.workspace = true
wavecraft-macros.workspace = true
nih_plug.workspace = true
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

**Dependencies:** Step 1.1  
**Risk:** Low

---

#### Step 1.3: Move editor/ Directory
**From:** `engine/crates/wavecraft-core/src/editor/`  
**To:** `engine/crates/wavecraft-nih_plug/src/editor/`

**Action:** Move entire directory (includes mod.rs, assets.rs, bridge.rs, webview.rs, macos.rs, windows.rs, js/).

**Dependencies:** Step 1.2  
**Risk:** Medium — ensure all relative paths in mod.rs are correct

---

#### Step 1.4: Move params.rs
**From:** `engine/crates/wavecraft-core/src/params.rs`  
**To:** `engine/crates/wavecraft-nih_plug/src/params.rs`

**Action:** Move file as-is. No content changes needed.

**Dependencies:** Step 1.2  
**Risk:** Low

---

#### Step 1.5: Move util.rs
**From:** `engine/crates/wavecraft-core/src/util.rs`  
**To:** `engine/crates/wavecraft-nih_plug/src/util.rs`

**Action:** Move file as-is. Contains `calculate_stereo_meters(buffer: &Buffer)`.

**Dependencies:** Step 1.2  
**Risk:** Low

---

#### Step 1.6: Extract wavecraft_plugin! Declarative Macro
**From:** `engine/crates/wavecraft-core/src/macros.rs`  
**To:** `engine/crates/wavecraft-nih_plug/src/macros.rs`

**Action:** 
1. Copy the `wavecraft_plugin!` declarative macro (lines ~85-280) to new file
2. Modify to use `$crate::__nih::` instead of `nih_plug::prelude::`
3. The original file in wavecraft-core keeps only `wavecraft_processor!`

**Content for wavecraft-nih_plug/src/macros.rs:**

```rust
//! Declarative macros for nih-plug plugin generation.

/// `wavecraft_plugin!` — generates a minimal plugin skeleton using nih-plug.
///
/// This macro creates a plugin struct, Default impl, Plugin trait impl,
/// and VST3/CLAP export registrations.
#[macro_export]
macro_rules! wavecraft_plugin {
    (
        ident: $ident:ident,
        name: $name:expr,
        vendor: $vendor:expr,
        url: $url:expr,
        email: $email:expr,
        version: $version:expr,
        audio: { inputs: $inputs:expr, outputs: $outputs:expr },
        params: [$param:ty],
        processor: $processor:ty $(,)?
    ) => {
        $crate::paste::paste! {
            /// Generated plugin type by `wavecraft_plugin!` macro
            pub struct $ident {
                params: std::sync::Arc<$param>,
                processor: $processor,
                meter_producer: ::wavecraft_metering::MeterProducer,
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                meter_consumer: std::sync::Arc<std::sync::Mutex<::wavecraft_metering::MeterConsumer>>,
            }

            impl std::default::Default for $ident {
                fn default() -> Self {
                    let (meter_producer, _meter_consumer) = ::wavecraft_metering::create_meter_channel(64);
                    Self {
                        params: std::sync::Arc::new(<$param>::default()),
                        processor: <$processor>::default(),
                        meter_producer,
                        #[cfg(any(target_os = "macos", target_os = "windows"))]
                        meter_consumer: std::sync::Arc::new(std::sync::Mutex::new(_meter_consumer)),
                    }
                }
            }

            impl $crate::__nih::Plugin for $ident {
                const NAME: &'static str = $name;
                const VENDOR: &'static str = $vendor;
                const URL: &'static str = $url;
                const EMAIL: &'static str = $email;
                const VERSION: &'static str = $version;

                const AUDIO_IO_LAYOUTS: &'static [$crate::__nih::AudioIOLayout] = &[$crate::__nih::AudioIOLayout {
                    main_input_channels: std::num::NonZeroU32::new($inputs),
                    main_output_channels: std::num::NonZeroU32::new($outputs),
                    ..$crate::__nih::AudioIOLayout::const_default()
                }];

                const MIDI_INPUT: $crate::__nih::MidiConfig = $crate::__nih::MidiConfig::None;
                const MIDI_OUTPUT: $crate::__nih::MidiConfig = $crate::__nih::MidiConfig::None;

                type SysExMessage = ();
                type BackgroundTask = ();

                fn params(&self) -> std::sync::Arc<dyn $crate::__nih::Params> {
                    self.params.clone()
                }

                fn editor(&mut self, _async_executor: $crate::__nih::AsyncExecutor<Self>) -> Option<Box<dyn $crate::__nih::Editor>> {
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    {
                        $crate::editor::create_webview_editor(self.params.clone(), self.meter_consumer.clone())
                    }

                    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                    {
                        None
                    }
                }

                fn initialize(
                    &mut self,
                    _audio_io_layout: &$crate::__nih::AudioIOLayout,
                    buffer_config: &$crate::__nih::BufferConfig,
                    _context: &mut impl $crate::__nih::InitContext<Self>,
                ) -> bool {
                    self.processor.set_sample_rate(buffer_config.sample_rate);
                    true
                }

                fn process(
                    &mut self,
                    _buffer: &mut $crate::__nih::Buffer,
                    _aux: &mut $crate::__nih::AuxiliaryBuffers,
                    _context: &mut impl $crate::__nih::ProcessContext<Self>,
                ) -> $crate::__nih::ProcessStatus {
                    let _ = &self.meter_producer;
                    $crate::__nih::ProcessStatus::Normal
                }
            }

            impl $crate::__nih::Vst3Plugin for $ident {
                const VST3_CLASS_ID: [u8; 16] = *b"MacroPlug0000001";
                const VST3_SUBCATEGORIES: &'static [$crate::__nih::Vst3SubCategory] = &[$crate::__nih::Vst3SubCategory::Fx];
            }

            impl $crate::__nih::ClapPlugin for $ident {
                const CLAP_ID: &'static str = "dev.wavecraft.macro";
                const CLAP_DESCRIPTION: Option<&'static str> = Some("Generated plugin from wavecraft_plugin!");
                const CLAP_MANUAL_URL: Option<&'static str> = Some($url);
                const CLAP_SUPPORT_URL: Option<&'static str> = Some($url);
                const CLAP_FEATURES: &'static [$crate::__nih::ClapFeature] = &[$crate::__nih::ClapFeature::AudioEffect];
            }

            #[cfg(not(test))]
            mod [<__wavecraft_exports_ $ident>] {
                $crate::__nih::nih_export_vst3!(crate::$ident);
                $crate::__nih::nih_export_clap!(crate::$ident);
            }
        }
    };
}
```

**Dependencies:** Step 1.2  
**Risk:** High — macro path changes are critical; test thoroughly

---

#### Step 1.7: Create __nih Re-export Module
**File:** `engine/crates/wavecraft-nih_plug/src/lib.rs` (partial)

**Action:** Create the `__nih` module that re-exports nih_plug types for macro use.

```rust
/// Hidden re-exports used by macro-generated code.
/// This module is an implementation detail — do not depend on it directly.
#[doc(hidden)]
pub mod __nih {
    // Traits
    pub use nih_plug::prelude::{
        ClapPlugin, Editor, InitContext, Param, Params, Plugin, ProcessContext, Vst3Plugin,
    };
    // Types
    pub use nih_plug::prelude::{
        AsyncExecutor, AudioIOLayout, AuxiliaryBuffers, Buffer, BufferConfig, ClapFeature,
        FloatParam, FloatRange, MidiConfig, ParamPtr, ProcessStatus, SmoothingStyle,
        Vst3SubCategory,
    };
    // Export macros
    pub use nih_plug::{nih_export_clap, nih_export_vst3};
}
```

**Dependencies:** Step 1.2  
**Risk:** Medium — must include all types used by macros

---

#### Step 1.8: Create wavecraft-nih_plug lib.rs
**File:** `engine/crates/wavecraft-nih_plug/src/lib.rs`

**Action:** Create main lib.rs with WavecraftPlugin struct and all impls from current wavecraft-core.

```rust
//! Wavecraft nih-plug integration.
//!
//! This crate bridges the publishable Wavecraft SDK (wavecraft-core)
//! with nih-plug for actual VST3/CLAP plugin compilation.

#![allow(clippy::crate_in_macro_def)]

pub mod editor;
pub mod macros;
pub mod prelude;
pub mod util;

mod params;

pub use paste;
pub use wavecraft_core;

// Hidden re-exports for macro-generated code
#[doc(hidden)]
pub mod __nih {
    pub use nih_plug::prelude::{
        AsyncExecutor, AudioIOLayout, AuxiliaryBuffers, Buffer, BufferConfig, ClapFeature,
        ClapPlugin, Editor, FloatParam, FloatRange, InitContext, MidiConfig, Param, ParamPtr,
        Params, Plugin, ProcessContext, ProcessStatus, SmoothingStyle, Vst3Plugin,
        Vst3SubCategory,
    };
    pub use nih_plug::{nih_export_clap, nih_export_vst3};
}

use std::sync::Arc;

use nih_plug::prelude::*;
use wavecraft_dsp::GainProcessor;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use wavecraft_metering::MeterConsumer;
use wavecraft_metering::{MeterFrame, MeterProducer, create_meter_channel};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::editor::create_webview_editor;
use crate::params::WavecraftParams;
use crate::util::calculate_stereo_meters;

/// Main plugin struct for Wavecraft (reference implementation).
pub struct WavecraftPlugin {
    params: Arc<WavecraftParams>,
    processor: GainProcessor,
    meter_producer: MeterProducer,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    meter_consumer: Arc<std::sync::Mutex<MeterConsumer>>,
}

impl Default for WavecraftPlugin {
    fn default() -> Self {
        let (meter_producer, _meter_consumer) = create_meter_channel(64);
        Self {
            params: Arc::new(WavecraftParams::default()),
            processor: GainProcessor::new(44100.0),
            meter_producer,
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            meter_consumer: Arc::new(std::sync::Mutex::new(_meter_consumer)),
        }
    }
}

impl Plugin for WavecraftPlugin {
    const NAME: &'static str = "Wavecraft";
    const VENDOR: &'static str = "Wavecraft";
    const URL: &'static str = "https://github.com/RonHouben/wavecraft";
    const EMAIL: &'static str = "contact@wavecraft.dev";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            create_webview_editor(self.params.clone(), self.meter_consumer.clone())
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            None
        }
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.processor.set_sample_rate(buffer_config.sample_rate);
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for mut channel_samples in buffer.iter_samples() {
            let gain_db = self.params.gain.smoothed.next();
            let gain_linear = wavecraft_protocol::db_to_linear(gain_db);

            for sample in channel_samples.iter_mut() {
                *sample *= gain_linear;
            }
        }

        if buffer.channels() >= 2 {
            let (peak_l, peak_r, rms_l, rms_r) = calculate_stereo_meters(buffer);
            self.meter_producer.push(MeterFrame {
                peak_l,
                peak_r,
                rms_l,
                rms_r,
                timestamp: 0,
            });
        }

        ProcessStatus::Normal
    }
}

impl Vst3Plugin for WavecraftPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"WavecraftPlug001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

impl ClapPlugin for WavecraftPlugin {
    const CLAP_ID: &'static str = "dev.wavecraft.wavecraft";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Wavecraft audio plugin framework");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Utility];
}

nih_export_vst3!(WavecraftPlugin);
nih_export_clap!(WavecraftPlugin);
```

**Dependencies:** Steps 1.3-1.7  
**Risk:** Medium — ensure all imports resolve correctly

---

#### Step 1.9: Create wavecraft-nih_plug prelude.rs
**File:** `engine/crates/wavecraft-nih_plug/src/prelude.rs`

**Action:** Create prelude that re-exports wavecraft-core prelude + nih_plug types.

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
```

**Dependencies:** Step 1.8  
**Risk:** Low

---

#### Step 1.10: Update Workspace Configuration
**File:** `engine/Cargo.toml`

**Action:** Add `wavecraft-nih_plug` to workspace dependencies.

```toml
[workspace.dependencies]
# ... existing deps ...
wavecraft-nih_plug = { path = "crates/wavecraft-nih_plug" }
```

**Dependencies:** Step 1.2  
**Risk:** Low

---

#### Step 1.11: Verify wavecraft-nih_plug Builds
**Command:** `cargo build -p wavecraft-nih_plug`

**Action:** Ensure the new crate compiles successfully.

**Dependencies:** Steps 1.1-1.10  
**Risk:** Low

---

### Phase 2: Strip wavecraft-core (7 steps)

#### Step 2.1: Remove nih_plug from Dependencies
**File:** `engine/crates/wavecraft-core/Cargo.toml`

**Action:** Remove `nih_plug.workspace = true` from dependencies.

**Dependencies:** Phase 1 complete  
**Risk:** Medium — will cause compile errors until Phase 2 complete

---

#### Step 2.2: Change crate-type to rlib Only
**File:** `engine/crates/wavecraft-core/Cargo.toml`

**Action:** Change from `crate-type = ["cdylib", "rlib"]` to `crate-type = ["rlib"]`.

```toml
[lib]
name = "wavecraft_core"
crate-type = ["rlib"]
```

**Dependencies:** Step 2.1  
**Risk:** Low

---

#### Step 2.3: Remove Platform-Specific Dependencies
**File:** `engine/crates/wavecraft-core/Cargo.toml`

**Action:** Remove all of:
- `wry`
- `raw-window-handle`
- `include_dir`
- All `[target.'cfg(target_os = "macos")'.dependencies]`
- All `[target.'cfg(target_os = "windows")'.dependencies]`

Keep: `paste`, `serde_json`, and wavecraft-* workspace deps.

**Dependencies:** Step 2.1  
**Risk:** Low

---

#### Step 2.4: Delete Moved Modules
**Files to delete from `engine/crates/wavecraft-core/src/`:**
- `editor/` (entire directory)
- `params.rs`
- `util.rs`

**Action:** `rm -rf editor/ params.rs util.rs`

**Dependencies:** Phase 1 complete (files already copied)  
**Risk:** Low

---

#### Step 2.5: Update macros.rs (Keep Only wavecraft_processor!)
**File:** `engine/crates/wavecraft-core/src/macros.rs`

**Action:** Remove the `wavecraft_plugin!` declarative macro (keep only `wavecraft_processor!` and its tests).

The file should contain only:
1. `wavecraft_processor!` macro (lines 1-84 approximately)
2. Its unit tests

**Dependencies:** Step 1.6 (macro already copied to nih_plug crate)  
**Risk:** Medium — ensure correct lines are removed

---

#### Step 2.6: Update wavecraft-core lib.rs
**File:** `engine/crates/wavecraft-core/src/lib.rs`

**Action:** Strip to minimal content:

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

**Dependencies:** Steps 2.4-2.5  
**Risk:** Medium

---

#### Step 2.7: Update wavecraft-core prelude.rs
**File:** `engine/crates/wavecraft-core/src/prelude.rs`

**Action:** Remove nih_plug re-exports, keep only publishable types:

```rust
//! Prelude module - Common imports for Wavecraft SDK users.
//!
//! This module re-exports the most commonly used types and traits for building
//! plugins with Wavecraft.

// Re-export Wavecraft DSP traits and types
pub use wavecraft_dsp::{Chain, ParamRange, ParamSpec, Processor, ProcessorParams, Transport};

// Re-export built-in processors
pub use wavecraft_dsp::builtins::{GainDsp, PassthroughDsp};

// Re-export Wavecraft protocol types
pub use wavecraft_protocol::{ParamId, ParameterInfo, ParameterType, db_to_linear};

// Re-export metering
pub use wavecraft_metering::{MeterConsumer, MeterFrame, MeterProducer, create_meter_channel};

// Re-export derive macro (proc-macro — publishable)
pub use wavecraft_macros::ProcessorParams as DeriveProcessorParams;

// Re-export wavecraft_processor! declarative macro
pub use crate::wavecraft_processor;
```

**Dependencies:** Step 2.6  
**Risk:** Medium — ensure all removed items are in wavecraft-nih_plug prelude

---

### Phase 3: Update Proc-Macro (4 steps)

#### Step 3.1: Add `crate` Field to Parser
**File:** `engine/crates/wavecraft-macros/src/plugin.rs`

**Action:** Add optional `krate` field to `PluginDef` struct and parser.

```rust
struct PluginDef {
    name: LitStr,
    vendor: LitStr,
    url: Option<LitStr>,
    email: Option<LitStr>,
    signal: Expr,
    krate: Option<syn::Path>,  // NEW
}

// In Parse impl, add case:
"crate" => krate = Some(input.parse()?),
```

**Dependencies:** Phase 2 complete  
**Risk:** Low

---

#### Step 3.2: Update Generated Code Paths
**File:** `engine/crates/wavecraft-macros/src/plugin.rs`

**Action:** Replace all `::nih_plug::` with `#krate::__nih::` and `::wavecraft_core::` with `#krate::`.

```rust
// Default krate to ::wavecraft_nih_plug
let krate = plugin_def.krate
    .unwrap_or_else(|| syn::parse_quote!(::wavecraft_nih_plug));

let expanded = quote! {
    impl #krate::__nih::Plugin for __WavecraftPlugin { ... }
    // ... all other ::nih_plug:: references become #krate::__nih::
    // ... all ::wavecraft_core:: references become #krate::
};
```

**Dependencies:** Step 3.1  
**Risk:** High — many paths to update; test thoroughly

---

#### Step 3.3: Update Editor Path
**File:** `engine/crates/wavecraft-macros/src/plugin.rs`

**Action:** Change `::wavecraft_core::editor::create_webview_editor` to `#krate::editor::create_webview_editor`.

**Dependencies:** Step 3.2  
**Risk:** Medium

---

#### Step 3.4: Update Proc-Macro Tests
**File:** `engine/crates/wavecraft-macros/tests/` (if any)

**Action:** Update any tests to use `crate: wavecraft_nih_plug` or rely on default.

**Dependencies:** Step 3.3  
**Risk:** Low

---

### Phase 4: Update Template and Workspace (5 steps)

#### Step 4.1: Add Version Specifiers to Workspace Path Dependencies
**File:** `engine/Cargo.toml`

**Action:** Add `version = "0.7.1"` to all publishable crate workspace dependencies.

```toml
[workspace.dependencies]
wavecraft-protocol = { path = "crates/wavecraft-protocol", version = "0.7.1" }
wavecraft-macros = { path = "crates/wavecraft-macros", version = "0.7.1" }
wavecraft-metering = { path = "crates/wavecraft-metering", version = "0.7.1" }
wavecraft-bridge = { path = "crates/wavecraft-bridge", version = "0.7.1" }
wavecraft-dsp = { path = "crates/wavecraft-dsp", version = "0.7.1" }
wavecraft-core = { path = "crates/wavecraft-core", version = "0.7.1" }
wavecraft-nih_plug = { path = "crates/wavecraft-nih_plug" }  # no version (publish = false)
```

**Dependencies:** Phase 3 complete  
**Risk:** Low

---

#### Step 4.2: Update plugin-template Cargo.toml
**File:** `plugin-template/engine/Cargo.toml`

**Action:** Replace 6 separate crate deps + nih_plug with single renamed dep.

```toml
[dependencies]
# Single SDK dependency — Cargo rename gives us `use wavecraft::prelude::*`
wavecraft = { package = "wavecraft-nih_plug", git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
log = "0.4"

[build-dependencies]
nih_plug_xtask = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "28b149ec4d62757d0b448809148a0c3ca6e09a95" }
```

**Dependencies:** Step 4.1  
**Risk:** Medium — template users need to regenerate

---

#### Step 4.3: Update plugin-template lib.rs
**File:** `plugin-template/engine/src/lib.rs`

**Action:** Change import and add `crate: wavecraft` to macro.

```rust
use wavecraft::prelude::*;

wavecraft_processor!({{plugin_name_pascal}}Gain => Gain);

wavecraft_plugin! {
    name: "{{plugin_name_title}}",
    vendor: "{{vendor}}",
    url: "{{url}}",
    email: "{{email}}",
    signal: {{plugin_name_pascal}}Gain,
    crate: wavecraft,
}
```

**Dependencies:** Step 4.2  
**Risk:** Low

---

#### Step 4.4: Update CLI Template Variables (if needed)
**File:** `cli/src/template/variables.rs`

**Action:** Verify template variable processing handles new structure. No changes expected.

**Dependencies:** Step 4.3  
**Risk:** Low

---

#### Step 4.5: Update Documentation References
**Files:**
- `docs/architecture/high-level-design.md`
- `docs/guides/sdk-getting-started.md`
- `README.md`

**Action:** Update any references to `wavecraft-core` imports to show `wavecraft::prelude::*`.

**Dependencies:** Phase 4 complete  
**Risk:** Low

---

### Phase 5: Verification (5 steps)

#### Step 5.1: Build Full Workspace
**Command:** `cargo build --workspace`

**Expected Result:** All crates compile without errors.

**Dependencies:** Phase 4 complete  
**Risk:** Medium — first integration test

---

#### Step 5.2: Run All Tests
**Command:** `cargo test --workspace`

**Expected Result:** All tests pass.

**Dependencies:** Step 5.1  
**Risk:** Medium

---

#### Step 5.3: Dry-Run Publish wavecraft-core
**Command:** `cargo publish --dry-run -p wavecraft-core`

**Expected Result:** No errors about nih_plug or missing dependencies.

**Dependencies:** Step 5.2  
**Risk:** High — this is the primary goal

---

#### Step 5.4: Dry-Run Publish All Publishable Crates
**Command:** 
```bash
cargo publish --dry-run -p wavecraft-protocol
cargo publish --dry-run -p wavecraft-macros
cargo publish --dry-run -p wavecraft-metering
cargo publish --dry-run -p wavecraft-bridge
cargo publish --dry-run -p wavecraft-dsp
cargo publish --dry-run -p wavecraft-core
```

**Expected Result:** All succeed.

**Dependencies:** Step 5.3  
**Risk:** Medium

---

#### Step 5.5: Test Plugin in DAW (Manual)
**Action:** 
1. Build plugin: `cargo xtask bundle`
2. Install: `cargo xtask install`
3. Open Ableton Live
4. Load Wavecraft plugin
5. Verify UI loads and parameters work

**Expected Result:** Plugin functions identically to before the split.

**Dependencies:** Step 5.4  
**Risk:** Medium — final integration validation

---

## Testing Strategy

### Unit Tests
- Existing tests in each crate move with their code
- New tests for `__nih` module re-exports
- Proc-macro tests with `crate:` field

### Integration Tests
- `cargo build --workspace` — all crates compile
- `cargo test --workspace` — all tests pass
- Template scaffolding test via CLI

### Manual Tests
- Plugin loads in DAW
- UI renders correctly
- Parameters respond to changes
- Meters update in real-time

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Macro path resolution fails | Medium | High | Test with both default (`::wavecraft_nih_plug`) and override (`wavecraft`) |
| Missing types in `__nih` module | Medium | Low | Compile errors are immediate; add as discovered |
| Editor fails to load | Low | High | Move entire directory; no logic changes |
| Template compilation fails | Medium | Medium | Test template after all changes; regenerate test project |
| DAW plugin validation fails | Low | High | No behavioral changes; just code relocation |

---

## Success Criteria

1. ✅ `cargo publish --dry-run -p wavecraft-core` succeeds
2. ✅ `cargo publish --dry-run -p wavecraft-*` succeeds for all publishable crates
3. ✅ `cargo build --workspace` succeeds
4. ✅ `cargo test --workspace` succeeds
5. ✅ Plugin loads and functions in Ableton Live
6. ✅ `use wavecraft::prelude::*` works in plugin-template

---

## Related Documents

- [Low-Level Design: Core Split](./low-level-design-core-split.md)
- [Low-Level Design: crates.io Publishing](./low-level-design-crates-io-publishing.md)
- [Test Plan](./test-plan.md)
