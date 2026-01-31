# Low-Level Design — Milestone 1: Rust Plugin Skeleton with VST3 Exports

**Scope:** Week 0–2  
**Objective:** Rust plugin skeleton with VST3 exports (nih-plug), native placeholder UI, and verified Ableton host load.

---

## 1. Goals and Success Criteria

### Must Have
- Plugin loads in Ableton Live (macOS & Windows) without crash
- Plugin appears in DAW plugin list with correct name/vendor metadata
- Plugin exposes at least one automatable parameter (e.g., gain)
- Audio passthrough works (input → output) with no dropouts at 64-sample buffer
- Native placeholder UI opens and closes cleanly
- Host automation writes to parameter and plugin reflects change

### Nice to Have
- Parameter state persists across session save/load
- Plugin loads in GarageBand/Logic Pro (AU) and Reaper (VST3/CLAP)

### Out of Scope
- WebView integration (Milestone 2+)
- Real DSP processing beyond passthrough
- IPC bridge
- Metering / ring buffers

---

## 2. Crate Structure

```
vstkit/
├── engine/                       # Rust audio engine & plugin
│   ├── Cargo.toml                # Workspace root
│   └── crates/
│       ├── dsp/                  # Pure DSP (no plugin deps)
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── processor.rs  # Passthrough processor (trivial for now)
│       │       └── gain.rs       # Gain utility functions
│       │
│       ├── protocol/             # Shared contracts (param IDs, ranges)
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs
│       │       └── params.rs     # Parameter IDs, ranges, defaults
│       │
│       ├── plugin/               # nih-plug host integration
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs        # Plugin struct, nih-plug macros
│       │       ├── editor.rs     # Placeholder native GUI
│       │       └── params.rs     # nih-plug Params derive, wraps protocol
│       │
│       └── bridge/               # UI ↔ Audio IPC (Milestone 2+)
│           └── ...               # Out of scope for M1
│
├── ui/                           # React SPA (Milestone 2+)
├── docs/                         # Architecture & specs
├── scripts/                      # Build & CI helpers
├── packaging/                    # Platform installers
│   ├── macos/
│   ├── windows/
│   └── linux/
│
└── tests/
    ├── integration/              # Host-in-the-loop tests
    └── dsp/                      # Offline DSP correctness tests
```

### Rationale

| Crate | Responsibility | Why Separate? |
|-------|----------------|---------------|
| `dsp` | Pure audio processing algorithms | Testable without DAW, no framework coupling |
| `protocol` | Parameter definitions, IDs, ranges | Shared contract between all layers (DSP, plugin, future UI) |
| `plugin` | nih-plug integration, host glue, UI | Framework-specific, links against SDK |
| `bridge` | UI ↔ Audio IPC (future) | Isolated communication layer for WebView integration |

This separation ensures:
1. DSP code can be unit-tested in isolation (no host needed)
2. Parameter definitions in `protocol` are shared across all layers (DSP, plugin, future React UI)
3. Future formats (AU, LV2) can wrap `dsp` without duplicating logic
4. `bridge` crate (Milestone 2+) will encapsulate all IPC complexity

---

## 3. Module-Level Design

### 3.1 `protocol` — Parameter Definitions

```rust
// engine/crates/protocol/src/params.rs

/// Canonical parameter identifiers (used across all layers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ParamId {
    Gain = 0,
    // Future: Drive, Tone, Mix, etc.
}

/// Parameter metadata (host-agnostic)
pub struct ParamSpec {
    pub id: ParamId,
    pub name: &'static str,
    pub short_name: &'static str,  // For narrow host displays
    pub unit: &'static str,        // "dB", "%", etc.
    pub default: f32,
    pub min: f32,
    pub max: f32,
    pub step: Option<f32>,         // None = continuous
}

pub const PARAM_SPECS: &[ParamSpec] = &[
    ParamSpec {
        id: ParamId::Gain,
        name: "Output Gain",
        short_name: "Gain",
        unit: "dB",
        default: 0.0,
        min: -24.0,
        max: 24.0,
        step: Some(0.1),
    },
];

/// Convert dB to linear gain (audio-thread safe, no allocation)
#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}
```

**Design Decisions:**
- `ParamId` is `repr(u32)` for stable ABI and host parameter indexing
- Specs are `const` to avoid runtime allocation
- Conversion functions are `#[inline]` for audio-thread performance

---

### 3.2 `dsp` — Processor

```rust
// engine/crates/dsp/src/processor.rs

use protocol::params::db_to_linear;

/// Minimal processor state (will grow with real DSP)
pub struct Processor {
    sample_rate: f32,
}

impl Processor {
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    /// Process audio in-place. `gain_db` is the current parameter value.
    /// 
    /// # Real-time safety
    /// - No allocations
    /// - No locks
    /// - No syscalls
    #[inline]
    pub fn process(&self, left: &mut [f32], right: &mut [f32], gain_db: f32) {
        let gain = db_to_linear(gain_db);
        
        for sample in left.iter_mut() {
            *sample *= gain;
        }
        for sample in right.iter_mut() {
            *sample *= gain;
        }
    }
}
```

**Design Decisions:**
- `process()` takes mutable slices (in-place processing, zero allocation)
- Parameter value passed explicitly (no internal state mutation from UI thread)
- `#[inline]` hint for hot path
- Stereo assumed for simplicity; future: channel-agnostic API

---

### 3.3 `plugin` — nih-plug Integration

```rust
// engine/crates/plugin/src/lib.rs

use nih_plug::prelude::*;
use dsp::processor::Processor;

mod editor;
mod params;

use params::VstKitParams;

pub struct VstKitPlugin {
    params: Arc<VstKitParams>,
    processor: Processor,
}

impl Default for VstKitPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(VstKitParams::default()),
            processor: Processor::new(44100.0), // Will be overwritten in initialize()
        }
    }
}

impl Plugin for VstKitPlugin {
    const NAME: &'static str = "VstKit";
    const VENDOR: &'static str = "Your Name";
    const URL: &'static str = "https://example.com";
    const EMAIL: &'static str = "dev@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        Some(Box::new(editor::PlaceholderEditor::new(self.params.clone())))
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
        // Read parameter atomically (lock-free)
        let gain_db = self.params.gain.value();

        // Process in-place
        for mut channel_samples in buffer.iter_samples() {
            // For simplicity, process sample-by-sample (will optimize later)
            let gain = protocol::params::db_to_linear(gain_db);
            for sample in channel_samples.iter_mut() {
                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }
}

impl Vst3Plugin for VstKitPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"VstKitPluginRHxx"; // Unique 16-byte ID
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Distortion,
    ];
}

impl ClapPlugin for VstKitPlugin {
    const CLAP_ID: &'static str = "com.yourname.vstkit";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Guitar effects plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Distortion,
    ];
}

// nih-plug export macros (VST3 and CLAP only)
// Note: AU is NOT supported by nih-plug. Use clap-wrapper to convert CLAP → AU.
// See: https://github.com/free-audio/clap-wrapper/
nih_export_vst3!(VstKitPlugin);
nih_export_clap!(VstKitPlugin);
```

**Critical Design Points:**
- `params` is `Arc<VstKitParams>` — shared between audio thread and editor
- nih-plug's `FloatParam` uses atomics internally (real-time safe reads)
- `process()` reads param value once per buffer (not per sample) for efficiency
- VST3_CLASS_ID must be globally unique (change before release!)

---

### 3.4 `plugin` — Parameter Wrapper

```rust
// engine/crates/plugin/src/params.rs

use nih_plug::prelude::*;
use protocol::params::{ParamId, PARAM_SPECS};

/// nih-plug parameter struct (derives Params trait)
#[derive(Params)]
pub struct VstKitParams {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for VstKitParams {
    fn default() -> Self {
        let gain_spec = PARAM_SPECS
            .iter()
            .find(|s| s.id == ParamId::Gain)
            .expect("Gain spec must exist");

        Self {
            gain: FloatParam::new(
                gain_spec.name,
                gain_spec.default,
                FloatRange::Linear {
                    min: gain_spec.min,
                    max: gain_spec.max,
                },
            )
            .with_unit(gain_spec.unit)
            .with_step_size(gain_spec.step.unwrap_or(0.01))
            .with_value_to_string(formatters::v2s_f32_rounded(1))
            .with_string_to_value(formatters::s2v_f32_rounded(1)),
        }
    }
}
```

**Design Decisions:**
- String ID `"gain"` used for state persistence (stable across versions)
- Parameter metadata sourced from `protocol` crate (single source of truth)
- Value formatters for clean host display

---

### 3.5 `plugin` — Placeholder Editor

```rust
// engine/crates/plugin/src/editor.rs

use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::Arc;

use crate::params::VstKitParams;

const WINDOW_WIDTH: u32 = 400;
const WINDOW_HEIGHT: u32 = 300;

pub struct PlaceholderEditor {
    params: Arc<VstKitParams>,
    state: Arc<EguiState>,
}

impl PlaceholderEditor {
    pub fn new(params: Arc<VstKitParams>) -> Self {
        Self {
            params,
            state: EguiState::from_size(WINDOW_WIDTH, WINDOW_HEIGHT),
        }
    }
}

impl Editor for PlaceholderEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn std::any::Any + Send> {
        let params = self.params.clone();
        let state = self.state.clone();

        create_egui_editor(
            state,
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.heading("VstKit — Placeholder UI");
                    ui.separator();
                    
                    ui.label("This will be replaced with React WebView in Milestone 2");
                    ui.add_space(20.0);

                    // Simple gain slider (demonstrates param binding)
                    ui.horizontal(|ui| {
                        ui.label("Gain:");
                        let mut gain = params.gain.value();
                        if ui.add(egui::Slider::new(&mut gain, -24.0..=24.0).suffix(" dB")).changed() {
                            setter.begin_set_parameter(&params.gain);
                            setter.set_parameter(&params.gain, gain);
                            setter.end_set_parameter(&params.gain);
                        }
                    });

                    ui.add_space(20.0);
                    ui.label(format!("Current gain: {:.1} dB", params.gain.value()));
                });
            },
        )
    }

    fn size(&self) -> (u32, u32) {
        (WINDOW_WIDTH, WINDOW_HEIGHT)
    }

    fn set_scale_factor(&self, _factor: f32) -> bool {
        // egui handles scaling internally
        true
    }

    fn param_value_changed(&self, _id: &str, _normalized_value: f32) {
        // egui redraws automatically
    }

    fn param_modulation_changed(&self, _id: &str, _modulation_offset: f32) {
        // Not used in placeholder
    }

    fn param_values_changed(&self) {
        // egui redraws automatically
    }
}
```

**Design Decisions:**
- Using `nih_plug_egui` for quick native UI (will be replaced by WebView)
- `setter.begin_set_parameter` / `end_set_parameter` required for proper host automation recording
- Editor holds `Arc<VstKitParams>` for read access; writes go through `GuiContext` setter

---

## 4. Threading Model

```
┌─────────────────────────────────────────────────────────────┐
│                         HOST (DAW)                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────┐          ┌─────────────────────────┐   │
│  │   Audio Thread  │          │      Main/UI Thread     │   │
│  │   (real-time)   │          │     (non-real-time)     │   │
│  └────────┬────────┘          └───────────┬─────────────┘   │
│           │                               │                 │
│           │                               │                 │
│           ▼                               ▼                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Arc<VstKitParams>                       │   │
│  │  ┌─────────────────────────────────────────────┐    │   │
│  │  │  FloatParam (gain)                          │    │   │
│  │  │  ├── value: AtomicF32       ◄── read ───────┼────┼───┤ Audio Thread
│  │  │  └── normalized: AtomicF32  ◄── write ──────┼────┼───┤ UI Thread (via setter)
│  │  └─────────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Invariants:**
1. Audio thread **only reads** parameter values (via `.value()`)
2. UI thread **only writes** through `GuiContext` setter (triggers host automation)
3. Host automation writes go through nih-plug's internal atomic update path
4. No locks, no `Mutex`, no `RwLock` on shared state

---

## 5. Build Configuration

### `engine/Cargo.toml` (Workspace Root)

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
authors = ["Your Name <dev@example.com>"]

[workspace.dependencies]
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git", features = ["assert_process_allocs"] }
nih_plug_egui = { git = "https://github.com/robbert-vdh/nih-plug.git" }

[profile.release]
lto = "thin"
strip = true

[profile.release-debug]
inherits = "release"
debug = true
strip = false
```

### `engine/crates/protocol/Cargo.toml`

```toml
[package]
name = "protocol"
version.workspace = true
edition.workspace = true

[dependencies]
# No dependencies — pure Rust definitions
```

### `engine/crates/dsp/Cargo.toml`

```toml
[package]
name = "dsp"
version.workspace = true
edition.workspace = true

[dependencies]
protocol = { path = "../protocol" }
```

### `engine/crates/plugin/Cargo.toml`

```toml
[package]
name = "plugin"
version.workspace = true
edition.workspace = true

[lib]
crate-type = ["cdylib"]

[dependencies]
dsp = { path = "../dsp" }
protocol = { path = "../protocol" }
nih_plug.workspace = true
nih_plug_egui.workspace = true

[features]
default = []
assert_process_allocs = ["nih_plug/assert_process_allocs"]
```

**Key Build Decisions:**
- `crate-type = ["cdylib"]` required for VST3/CLAP dynamic library export
- `assert_process_allocs` feature enables runtime detection of audio-thread allocations (dev only)
- `lto = "thin"` balances compile time vs binary size
- `strip = true` removes debug symbols from release builds

---

## 6. Platform-Specific Considerations

### macOS

| Concern | Solution |
|---------|----------|
| Code signing | `codesign --deep --force --sign "Developer ID"` on `.vst3` and `.clap` bundles |
| Notarization | `xcrun notarytool submit` + `xcrun stapler staple` |
| AU support | nih-plug does **NOT** export AU; use [clap-wrapper](https://github.com/free-audio/clap-wrapper/) to convert CLAP → AUv2 |
| Bundle structure | `.vst3` and `.clap` are directory bundles; ensure `Info.plist` is correct |

### Windows

| Concern | Solution |
|---------|----------|
| MSVC vs GNU | Use MSVC toolchain for VST3 compatibility (`x86_64-pc-windows-msvc`) |
| Runtime deps | No C++ runtime needed (pure Rust); no installer bootstrapping required |
| Path length | Keep build paths short to avoid MAX_PATH issues |

### Linux (Secondary)

| Concern | Solution |
|---------|----------|
| GUI toolkit | egui uses raw X11/Wayland; no GTK dependency |
| Plugin path | Install to `~/.vst3` or `/usr/lib/vst3` |

---

## 7. Testing Strategy

### 7.1 Unit Tests (dsp crate)

```rust
// engine/crates/dsp/src/processor.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_passthrough_at_0db() {
        let proc = Processor::new(44100.0);
        let mut left = [0.5, -0.5, 0.25];
        let mut right = [0.1, -0.1, 0.05];
        
        proc.process(&mut left, &mut right, 0.0); // 0 dB = unity gain
        
        assert!((left[0] - 0.5).abs() < 1e-6);
        assert!((right[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_gain_applied() {
        let proc = Processor::new(44100.0);
        let mut left = [1.0];
        let mut right = [1.0];
        
        proc.process(&mut left, &mut right, -6.0); // ~0.5 linear
        
        assert!((left[0] - 0.501187).abs() < 1e-4);
    }
}
```

Additional integration tests can be placed in `tests/dsp/` for offline DSP correctness verification.

### 7.2 Host Compatibility Tests (Manual)

| Test | Ableton Live | Logic Pro | Reaper |
|------|--------------|-----------|--------|
| Format | VST3 | AU (via clap-wrapper) | VST3/CLAP |
| Plugin loads | ☐ | ☐ | ☐ |
| Audio passthrough | ☐ | ☐ | ☐ |
| Automation read | ☐ | ☐ | ☐ |
| Automation write | ☐ | ☐ | ☐ |
| UI opens/closes | ☐ | ☐ | ☐ |
| Session save/load | ☐ | ☐ | ☐ |
| 64-sample buffer | ☐ | ☐ | ☐ |

> **Note:** Logic Pro requires AU format, which is built using clap-wrapper from the CLAP plugin. Reaper can test both VST3 and CLAP directly from nih-plug.

### 7.3 Real-Time Safety Verification

Run with `assert_process_allocs` feature enabled:

```bash
cd engine
cargo build --release --features assert_process_allocs -p plugin
```

This will panic if any allocation occurs on the audio thread during `process()`.

---

## 8. File Outputs and Install Paths

After `cd engine && cargo xtask bundle plugin --release`:

| Platform | Output | Install Path |
|----------|--------|--------------|
| macOS | `engine/target/bundled/VstKit.vst3` | `~/Library/Audio/Plug-Ins/VST3/` |
| macOS | `engine/target/bundled/VstKit.clap` | `~/Library/Audio/Plug-Ins/CLAP/` |
| Windows | `engine/target/bundled/VstKit.vst3` | `C:\Program Files\Common Files\VST3\` |
| Linux | `engine/target/bundled/VstKit.vst3` | `~/.vst3/` |

**AU Plugin (macOS only, via clap-wrapper):**

AU plugins are built separately using clap-wrapper (not by nih-plug):

| Platform | Output | Install Path |
|----------|--------|--------------|
| macOS | `packaging/macos/au-wrapper/build/VstKit.component` | `~/Library/Audio/Plug-Ins/Components/` |

See the [high-level design](../../docs/architecture/high-level-design.md) for clap-wrapper build instructions.

Packaging scripts in `packaging/{macos,windows,linux}/` will handle signing, notarization, and installer creation.

---

## 9. Risk Register (Milestone 1 Specific)

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| egui rendering issues on HiDPI | Medium | Low | Test on Retina Mac; egui handles scaling but may need tweaks |
| Ableton rejects plugin due to metadata | Low | High | Follow VST3 spec strictly; test early |
| Audio dropouts at low buffer sizes | Medium | High | Profile with `assert_process_allocs`; avoid per-sample allocations |
| nih-plug API changes | Low | Medium | Pin to specific git commit |

---

## 10. Definition of Done

Milestone 1 is complete when:

1. ✅ `cd engine && cargo build --release -p plugin` succeeds on macOS and Windows
2. ✅ Plugin binary loads in Ableton Live without crash
3. ✅ Plugin appears in Ableton's plugin list with correct name ("VstKit")
4. ✅ Gain parameter visible in Ableton's parameter list and automatable
5. ✅ Audio signal passes through with correct gain applied
6. ✅ Placeholder UI opens, displays current gain, allows slider adjustment
7. ✅ UI changes reflect in Ableton's automation lane
8. ✅ No audio dropouts at 64-sample buffer size
9. ✅ `cd engine && cargo test -p dsp -p protocol` passes
10. ✅ Build with `assert_process_allocs` does not panic during normal use

---

## Appendix A: Quick Start Commands

```bash
# Clone and setup
cd vstkit
rustup target add x86_64-apple-darwin aarch64-apple-darwin  # macOS
rustup target add x86_64-pc-windows-msvc                     # Windows (cross-compile)

# Navigate to engine workspace
cd engine

# Build debug
cargo build -p plugin

# Build release with real-time safety checks
cargo build --release --features assert_process_allocs -p plugin

# Bundle for distribution (requires nih-plug's bundler)
cargo xtask bundle plugin --release

# Run tests
cargo test -p dsp -p protocol

# Run all workspace tests
cargo test --workspace
```

---

## Appendix B: Future Extension Points

The architecture is designed for these planned extensions:

| Extension | Where It Fits |
|-----------|---------------|
| Additional parameters | Add to `ParamId` enum and `PARAM_SPECS` in `engine/crates/protocol` |
| Real DSP (distortion, etc.) | Extend `Processor` in `engine/crates/dsp` |
| WebView UI (Milestone 2) | Replace `PlaceholderEditor` with wry-based editor; implement `bridge` crate |
| Metering (ring buffers) | Add SPSC buffer in `bridge` crate, read in React UI |
| Preset system | Add serialization to `protocol`, expose via nih-plug's state API |
| React SPA | Build in `ui/`, output to `ui/dist/`, embed in plugin binary |

---

## Appendix C: Sequence Diagram — Parameter Change from UI

```
┌────────┐         ┌──────────────┐         ┌──────────────┐         ┌──────────┐
│  User  │         │    egui UI   │         │  nih-plug    │         │   Host   │
└───┬────┘         └──────┬───────┘         └──────┬───────┘         └────┬─────┘
    │                     │                        │                      │
    │  Drag slider        │                        │                      │
    │────────────────────>│                        │                      │
    │                     │                        │                      │
    │                     │ begin_set_parameter()  │                      │
    │                     │───────────────────────>│                      │
    │                     │                        │                      │
    │                     │ set_parameter(0.5)     │                      │
    │                     │───────────────────────>│                      │
    │                     │                        │                      │
    │                     │                        │ Notify host (atomic) │
    │                     │                        │─────────────────────>│
    │                     │                        │                      │
    │                     │                        │    Record automation │
    │                     │                        │<─────────────────────│
    │                     │                        │                      │
    │                     │ end_set_parameter()    │                      │
    │                     │───────────────────────>│                      │
    │                     │                        │                      │
    │                     │                        │                      │
    │                     │     (next audio callback)                     │
    │                     │                        │                      │
    │                     │                        │ process() reads      │
    │                     │                        │ param.value() = 0.5  │
    │                     │                        │                      │
```

---

## Appendix D: References

- [nih-plug documentation](https://github.com/robbert-vdh/nih-plug)
- [VST3 SDK Documentation](https://steinbergmedia.github.io/vst3_doc/)
- [nih-plug egui example](https://github.com/robbert-vdh/nih-plug/tree/master/plugins/examples/gain_gui_egui)
- [Real-time audio programming guidelines](https://www.rossbencina.com/code/real-time-audio-programming-101-time-waits-for-nothing)
