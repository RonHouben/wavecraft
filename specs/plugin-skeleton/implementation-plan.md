# Implementation Plan: Milestone 1 — Rust Plugin Skeleton with VST3/AU Exports

## Overview

This plan implements the foundational Rust audio plugin skeleton using nih-plug, establishing the crate architecture, basic gain parameter, audio passthrough, and placeholder native UI. Upon completion, the plugin will load in Ableton Live (macOS & Windows) as VST3 and GarageBand (macOS) as AU with verified host automation support. The AU format also supports Logic Pro.

## Requirements

- Plugin loads in Ableton Live (macOS & Windows) as VST3 without crash
- Plugin loads in GarageBand (macOS) as AU without crash (Logic Pro also supported)
- Plugin appears in DAW plugin list with correct name/vendor metadata
- Plugin exposes at least one automatable parameter (gain in dB)
- Audio passthrough works with no dropouts at 64-sample buffer
- Native placeholder UI (egui) opens and closes cleanly
- Host automation writes to parameter and plugin reflects change
- Real-time safe audio processing (no allocations on audio thread)
- Plugin passes `auval` validation on macOS

## Architecture Changes

- Create `engine/` workspace with Cargo workspace configuration
- Create `engine/crates/protocol/` — Parameter definitions and shared contracts
- Create `engine/crates/dsp/` — Pure audio processing (passthrough + gain)
- Create `engine/crates/plugin/` — nih-plug integration, host glue, placeholder UI
- Add `tests/dsp/` for offline DSP correctness tests
- Configure AU export with 4-character codes (manufacturer: `'VstK'`, subtype: `'vsk1'`, type: `'aufx'`)
- Create AU bundle (`.component`) alongside VST3 and CLAP bundles

---

## Implementation Steps

### Phase 1: Workspace & Build Infrastructure

**Goal:** Establish the Rust workspace structure and build configuration.

#### 1. **Create engine workspace root** (File: [engine/Cargo.toml](engine/Cargo.toml))
   - Action: Create workspace Cargo.toml with members, resolver, and workspace-level dependencies
   - Why: Central configuration for all crates; ensures consistent dependency versions
   - Dependencies: None
   - Risk: Low
   - Details:
     ```
     - Define workspace members: ["crates/*"]
     - Set resolver = "2"
     - Add workspace.package (version, edition, license, authors)
     - Add workspace.dependencies for nih_plug and nih_plug_egui (git deps)
     - Configure release profile with lto = "thin" and strip = true
     - Configure release-debug profile for debugging release builds
     ```

#### 2. **Create protocol crate structure** (File: [engine/crates/protocol/Cargo.toml](engine/crates/protocol/Cargo.toml))
   - Action: Create minimal Cargo.toml with no external dependencies
   - Why: Protocol crate must be pure Rust with no framework coupling
   - Dependencies: Step 1 (workspace exists)
   - Risk: Low

#### 3. **Create dsp crate structure** (File: [engine/crates/dsp/Cargo.toml](engine/crates/dsp/Cargo.toml))
   - Action: Create Cargo.toml depending only on protocol crate
   - Why: DSP must be testable in isolation without nih-plug
   - Dependencies: Step 2 (protocol crate exists)
   - Risk: Low

#### 4. **Create plugin crate structure** (File: [engine/crates/plugin/Cargo.toml](engine/crates/plugin/Cargo.toml))
   - Action: Create Cargo.toml with cdylib crate-type and nih-plug dependencies
   - Why: Plugin crate compiles to dynamic library for VST3/AU/CLAP export
   - Dependencies: Steps 2-3 (protocol and dsp crates exist)
   - Risk: Low
   - Details:
     ```
     - Set crate-type = ["cdylib"]
     - Add dependencies: dsp, protocol, nih_plug, nih_plug_egui
     - Add assert_process_allocs feature for RT safety checking
     - Enable nih_plug "au" feature for Audio Unit support (macOS)
     ```

---

### Phase 2: Protocol Layer — Parameter Definitions

**Goal:** Establish the single source of truth for parameter metadata.

#### 5. **Implement ParamId enum** (File: [engine/crates/protocol/src/lib.rs](engine/crates/protocol/src/lib.rs))
   - Action: Create lib.rs with module declarations
   - Why: Entry point for protocol crate
   - Dependencies: Step 2
   - Risk: Low

#### 6. **Implement parameter specifications** (File: [engine/crates/protocol/src/params.rs](engine/crates/protocol/src/params.rs))
   - Action: Create params.rs with ParamId enum, ParamSpec struct, PARAM_SPECS const, and db_to_linear function
   - Why: Centralized parameter definitions shared across all layers
   - Dependencies: Step 5
   - Risk: Low
   - Details:
     ```
     - ParamId enum with #[repr(u32)] for stable ABI
     - ParamSpec struct with id, name, short_name, unit, default, min, max, step
     - PARAM_SPECS const array with Gain parameter (-24.0 to +24.0 dB, default 0.0)
     - db_to_linear() function marked #[inline] for audio-thread performance
     ```

---

### Phase 3: DSP Layer — Audio Processing

**Goal:** Implement pure, testable audio processing logic.

#### 7. **Create dsp crate entry point** (File: [engine/crates/dsp/src/lib.rs](engine/crates/dsp/src/lib.rs))
   - Action: Create lib.rs with module declarations for processor and gain
   - Why: Entry point for dsp crate
   - Dependencies: Step 6 (protocol params available)
   - Risk: Low

#### 8. **Implement gain utility functions** (File: [engine/crates/dsp/src/gain.rs](engine/crates/dsp/src/gain.rs))
   - Action: Create gain.rs with any additional gain-related utilities (re-export db_to_linear from protocol)
   - Why: Consolidate gain-related DSP utilities
   - Dependencies: Step 7
   - Risk: Low

#### 9. **Implement Processor struct** (File: [engine/crates/dsp/src/processor.rs](engine/crates/dsp/src/processor.rs))
   - Action: Create processor.rs with Processor struct and process() method
   - Why: Encapsulates audio processing state; processes audio in-place
   - Dependencies: Step 7
   - Risk: Low
   - Details:
     ```
     - Processor struct with sample_rate field
     - new(sample_rate: f32) constructor
     - set_sample_rate() method
     - process(&self, left: &mut [f32], right: &mut [f32], gain_db: f32) marked #[inline]
     - No allocations, no locks, no syscalls in process()
     ```

#### 10. **Add unit tests for Processor** (File: [engine/crates/dsp/src/processor.rs](engine/crates/dsp/src/processor.rs))
   - Action: Add #[cfg(test)] module with tests for passthrough and gain application
   - Why: Verify DSP correctness without DAW dependency
   - Dependencies: Step 9
   - Risk: Low
   - Details:
     ```
     - test_passthrough_at_0db: verify unity gain
     - test_gain_applied: verify -6dB produces ~0.5 linear gain
     - test_negative_gain: verify attenuation works correctly
     - test_positive_gain: verify boost works correctly
     ```

---

### Phase 4: Plugin Layer — nih-plug Integration

**Goal:** Create the plugin struct with nih-plug traits and VST3/AU/CLAP exports.

#### 11. **Create plugin crate entry point** (File: [engine/crates/plugin/src/lib.rs](engine/crates/plugin/src/lib.rs))
   - Action: Create lib.rs with VstKitPlugin struct and Plugin trait implementation
   - Why: Main plugin struct that nih-plug exports to hosts
   - Dependencies: Steps 6, 9 (protocol and dsp available)
   - Risk: Medium
   - Details:
     ```
     - VstKitPlugin struct with params: Arc<VstKitParams> and processor: Processor
     - Implement Plugin trait with:
       - NAME, VENDOR, URL, EMAIL, VERSION constants
       - AUDIO_IO_LAYOUTS (stereo in/out)
       - MIDI_INPUT/OUTPUT = None
       - params() returns Arc<dyn Params>
       - editor() returns placeholder editor
       - initialize() sets sample rate
       - process() reads gain atomically, applies gain in-place
     - Implement Vst3Plugin trait with unique VST3_CLASS_ID
     - Implement ClapPlugin trait with CLAP_ID
     - Implement AuPlugin trait (macOS only) with:
       - AU_MANUFACTURER_CODE = *b"VstK" (4-char manufacturer code)
       - AU_SUBTYPE_CODE = *b"vsk1" (4-char plugin identifier)
       - AU_TYPE_CODE = aufx (effect type)
     - Add nih_export_vst3! and nih_export_clap! macros
     - Add nih_export_au! macro (macOS only, conditionally compiled)
     ```

#### 12. **Implement VstKitParams wrapper** (File: [engine/crates/plugin/src/params.rs](engine/crates/plugin/src/params.rs))
   - Action: Create params.rs with VstKitParams struct deriving nih-plug Params
   - Why: Bridges protocol parameter specs to nih-plug's parameter system
   - Dependencies: Step 6 (protocol params)
   - Risk: Low
   - Details:
     ```
     - VstKitParams struct with #[derive(Params)]
     - gain: FloatParam field with #[id = "gain"] attribute
     - Default impl sources metadata from PARAM_SPECS
     - Configure FloatRange::Linear with min/max from spec
     - Add unit suffix, step size, value formatters
     ```

---

### Phase 5: Placeholder UI — egui Editor

**Goal:** Create a minimal native UI for parameter visualization and manipulation.

#### 13. **Implement PlaceholderEditor** (File: [engine/crates/plugin/src/editor.rs](engine/crates/plugin/src/editor.rs))
   - Action: Create editor.rs with PlaceholderEditor implementing nih-plug Editor trait
   - Why: Provides visual feedback and parameter control until WebView UI is ready
   - Dependencies: Step 12 (params available)
   - Risk: Medium (egui rendering varies by platform)
   - Details:
     ```
     - PlaceholderEditor struct with params: Arc<VstKitParams> and state: Arc<EguiState>
     - new() constructor with 400x300 default size
     - Implement Editor trait:
       - spawn() creates egui editor with:
         - "VstKit — Placeholder UI" heading
         - Gain slider (-24.0 to +24.0 dB)
         - Current gain value display
         - Proper setter.begin/set/end_parameter flow for host automation
       - size() returns (400, 300)
       - set_scale_factor() returns true (egui handles internally)
       - param_value_changed(), param_modulation_changed(), param_values_changed() stubs
     ```

---

### Phase 6: Build Verification & Testing

**Goal:** Verify the plugin builds and tests pass on all target platforms.

#### 14. **Verify workspace compilation** (Local)
   - Action: Run `cd engine && cargo build -p plugin` and fix any compilation errors
   - Why: Ensure all crates compile and link correctly
   - Dependencies: Steps 1-13
   - Risk: Medium (potential dependency issues)

#### 15. **Run unit tests** (Local)
   - Action: Run `cd engine && cargo test -p dsp -p protocol`
   - Why: Verify DSP and protocol logic before host testing
   - Dependencies: Step 14
   - Risk: Low

#### 16. **Build release with RT safety checks** (Local)
   - Action: Run `cargo build --release --features assert_process_allocs -p plugin`
   - Why: Detect any allocations on audio thread during development
   - Dependencies: Step 14
   - Risk: Low

#### 17. **Bundle plugin for distribution** (Local)
   - Action: Run `cargo xtask bundle plugin --release` to create .vst3, .component (AU), and .clap bundles
   - Why: Produces installable plugin packages for host testing
   - Dependencies: Step 16
   - Risk: Medium (bundler configuration)
   - Details:
     ```
     - VST3 bundle: VstKit.vst3
     - AU bundle: VstKit.component (macOS only)
     - CLAP bundle: VstKit.clap
     - AU bundle must contain valid Info.plist with AudioComponents array
     ```

#### 17a. **Validate AU plugin with auval** (Local, macOS only)
   - Action: Run `auval -v aufx vsk1 VstK` to validate the Audio Unit
   - Why: AU plugins must pass validation before testing in GarageBand/Logic Pro
   - Dependencies: Step 17
   - Risk: Medium (AU validation may expose metadata or behavioral issues)
   - Details:
     ```
     - Validates plugin loads correctly
     - Checks parameter metadata consistency
     - Verifies audio processing callbacks
     - Tests state save/restore
     - Must pass with no errors before GarageBand testing
     ```

---

### Phase 7: Host Compatibility Testing

**Goal:** Verify plugin loads and functions correctly in target DAWs (Ableton Live for VST3, Logic Pro for AU).

#### 18. **Install plugin on macOS** (Manual)
   - Action: Copy VstKit.vst3 to ~/Library/Audio/Plug-Ins/VST3/ and VstKit.component to ~/Library/Audio/Plug-Ins/Components/
   - Why: Make plugin discoverable by DAWs (VST3 for Ableton, AU for GarageBand/Logic Pro)
   - Dependencies: Step 17
   - Risk: Low
   - Notes: After installing AU, run `killall -9 AudioComponentRegistrar` to refresh macOS plugin cache

#### 19. **Test in Ableton Live (macOS)** (Manual)
   - Action: Open Ableton Live, scan plugins, verify VstKit appears, load on track
   - Why: Primary host compatibility requirement
   - Dependencies: Step 18
   - Risk: Medium (host-specific issues possible)
   - Checklist:
     ```
     - [ ] Plugin appears in plugin list with name "VstKit"
     - [ ] Plugin loads on audio track without crash
     - [ ] Audio passes through (connect audio source)
     - [ ] Gain parameter visible in device view
     - [ ] Gain slider moves without artifacts
     - [ ] Automation lane shows gain parameter
     - [ ] Recording automation captures slider movements
     - [ ] Playing back automation moves plugin slider
     - [ ] UI opens without crash
     - [ ] UI closes without crash
     - [ ] No dropouts at 64-sample buffer size
     ```

#### 20. **Test session save/load** (Manual)
   - Action: Set gain to non-default value, save Ableton project, close, reopen, verify gain restored
   - Why: Validates parameter state persistence
   - Dependencies: Step 19
   - Risk: Low

#### 20a. **Test in GarageBand (macOS, AU)** (Manual)
   - Action: Open GarageBand, scan plugins, verify VstKit appears as AU, load on track
   - Why: AU format required for Apple DAW compatibility (GarageBand used for testing; Logic Pro also supported)
   - Dependencies: Step 18, Step 17a (auval must pass)
   - Risk: Medium (GarageBand has stricter AU validation than some hosts)
   - Checklist:
     ```
     - [ ] Plugin appears in GarageBand's plugin list with name "VstKit"
     - [ ] Plugin loads on audio track without crash
     - [ ] Audio passes through (connect audio source)
     - [ ] Gain parameter visible in plugin interface
     - [ ] Gain slider moves without artifacts
     - [ ] UI opens without crash
     - [ ] UI closes without crash
     - [ ] No dropouts at 64-sample buffer size
     ```
   - Notes: 
     - GarageBand is used for testing as Logic Pro is not available to the developer
     - AU plugin supports both Logic Pro and GarageBand
     - Restart GarageBand after plugin updates to refresh plugin cache

#### 20b. **Test AU state persistence in GarageBand** (Manual)
   - Action: Set gain to non-default value, save GarageBand project, close, reopen, verify gain restored
   - Why: Validates AU state persistence via project save/restore
   - Dependencies: Step 20a
   - Risk: Low

#### 21. **Cross-platform build verification** (Optional - Windows)
   - Action: Build on Windows with MSVC toolchain, test in Ableton Live Windows
   - Why: Validates Windows compatibility
   - Dependencies: Step 17
   - Risk: Medium (platform-specific issues)
   - Notes: AU is macOS-only; Windows testing is VST3/CLAP only

---

## Testing Strategy

### Unit Tests
- [engine/crates/dsp/src/processor.rs](engine/crates/dsp/src/processor.rs) — Gain application correctness
- [engine/crates/protocol/src/params.rs](engine/crates/protocol/src/params.rs) — db_to_linear conversion accuracy

### Integration Tests
- Plugin loads in Ableton Live as VST3 (manual)
- Plugin loads in Logic Pro as AU (manual)
- Host automation roundtrip (manual)
- Session save/load persistence — VST3 and AU formats (manual)

### AU Validation Tests
- Run `auval -v aufx vsk1 VstK` before any Logic Pro testing
- Validate state save/restore via `.aupreset` format
- Test bypass state handling

### Real-Time Safety Tests
- Build with `assert_process_allocs` feature and run in host to detect audio thread allocations

### Host Compatibility Matrix

| Test | Ableton Live (macOS) | Ableton Live (Windows) | GarageBand (macOS) | Reaper |
|------|---------------------|------------------------|-------------------|--------|
| Format | VST3 | VST3 | AU | VST3/AU/CLAP |
| Plugin loads | Required | Required | Required | Nice to have |
| Audio passthrough | Required | Required | Required | Nice to have |
| Automation read | Required | Required | N/A* | Nice to have |
| Automation write | Required | Required | N/A* | Nice to have |
| UI opens/closes | Required | Required | Required | Nice to have |
| Session save/load | Required | Required | Required | Nice to have |
| 64-sample buffer | Required | Required | Required | Nice to have |
| auval validation | N/A | N/A | Required | N/A |

*GarageBand has limited automation support compared to Logic Pro. Basic parameter control is tested.

---

## Risks & Mitigations

- **Risk:** egui rendering issues on HiDPI displays
  - Likelihood: Medium
  - Impact: Low (placeholder UI will be replaced)
  - Mitigation: Test on Retina Mac early; egui handles scaling but may need DPI awareness tweaks

- **Risk:** Ableton rejects plugin due to incorrect metadata
  - Likelihood: Low
  - Impact: High (blocks milestone completion)
  - Mitigation: Follow VST3 spec strictly; use unique VST3_CLASS_ID; test plugin scan early

- **Risk:** Audio dropouts at low buffer sizes
  - Likelihood: Medium
  - Impact: High (fails success criteria)
  - Mitigation: Profile with `assert_process_allocs`; ensure no per-sample allocations; read param once per buffer

- **Risk:** nih-plug API changes between versions
  - Likelihood: Low
  - Impact: Medium (requires code updates)
  - Mitigation: Pin to specific git commit in Cargo.toml

- **Risk:** Cross-platform build failures
  - Likelihood: Medium
  - Impact: Medium (delays Windows support)
  - Mitigation: Set up CI early with both macOS and Windows runners

- **Risk:** AU validation fails (`auval` errors)
  - Likelihood: Medium
  - Impact: High (blocks Logic Pro compatibility)
  - Mitigation: Run `auval` in CI pipeline; ensure parameter ranges and metadata are consistent between VST3 and AU; test state save/restore early

- **Risk:** AU cache invalidation issues during development
  - Likelihood: High
  - Impact: Low (development friction only)
  - Mitigation: Use `killall -9 AudioComponentRegistrar` after rebuilds; restart GarageBand after plugin updates; consider incrementing version during development

- **Risk:** GarageBand/Logic Pro-specific behavioral differences
  - Likelihood: Medium
  - Impact: Medium (may require AU-specific code paths)
  - Mitigation: Test in GarageBand early (Logic Pro not available); ensure Cocoa view lifecycle is handled correctly; verify state persistence via project save/restore

---

## Success Criteria

- [ ] `cd engine && cargo build --release -p plugin` succeeds on macOS
- [ ] `cd engine && cargo build --release -p plugin` succeeds on Windows
- [ ] VST3 plugin binary loads in Ableton Live without crash
- [ ] AU plugin binary loads in GarageBand without crash (Logic Pro also supported)
- [ ] Plugin passes `auval -v aufx vsk1 VstK` validation
- [ ] Plugin appears in Ableton's plugin list with name "VstKit" (VST3)
- [ ] Plugin appears in GarageBand's plugin list with name "VstKit" (AU)
- [ ] Gain parameter visible in Ableton's parameter list
- [ ] Gain parameter visible in GarageBand's plugin interface
- [ ] Gain parameter automatable (record and playback) in both hosts
- [ ] Audio signal passes through with correct gain applied
- [ ] Placeholder UI opens and displays current gain value
- [ ] Placeholder UI slider adjusts gain and reflects in automation lane
- [ ] UI closes cleanly without crash
- [ ] No audio dropouts at 64-sample buffer size
- [ ] `cd engine && cargo test -p dsp -p protocol` passes
- [ ] Build with `assert_process_allocs` does not panic during normal use
- [ ] AU state save/restore works via project save (tested in GarageBand)

---

## Appendix: File Creation Summary

| File | Purpose |
|------|---------|
| [engine/Cargo.toml](engine/Cargo.toml) | Workspace root configuration |
| [engine/crates/protocol/Cargo.toml](engine/crates/protocol/Cargo.toml) | Protocol crate manifest |
| [engine/crates/protocol/src/lib.rs](engine/crates/protocol/src/lib.rs) | Protocol crate entry point |
| [engine/crates/protocol/src/params.rs](engine/crates/protocol/src/params.rs) | Parameter definitions |
| [engine/crates/dsp/Cargo.toml](engine/crates/dsp/Cargo.toml) | DSP crate manifest |
| [engine/crates/dsp/src/lib.rs](engine/crates/dsp/src/lib.rs) | DSP crate entry point |
| [engine/crates/dsp/src/gain.rs](engine/crates/dsp/src/gain.rs) | Gain utilities |
| [engine/crates/dsp/src/processor.rs](engine/crates/dsp/src/processor.rs) | Audio processor |
| [engine/crates/plugin/Cargo.toml](engine/crates/plugin/Cargo.toml) | Plugin crate manifest |
| [engine/crates/plugin/src/lib.rs](engine/crates/plugin/src/lib.rs) | Plugin struct & nih-plug integration |
| [engine/crates/plugin/src/params.rs](engine/crates/plugin/src/params.rs) | nih-plug params wrapper |
| [engine/crates/plugin/src/editor.rs](engine/crates/plugin/src/editor.rs) | Placeholder egui editor |

---

## Appendix: Quick Start Commands

```bash
# Navigate to engine workspace
cd engine

# Build debug
cargo build -p plugin

# Build release
cargo build --release -p plugin

# Build with real-time safety checks
cargo build --release --features assert_process_allocs -p plugin

# Run tests
cargo test -p dsp -p protocol

# Bundle for distribution (creates .vst3, .component, .clap)
cargo xtask bundle plugin --release

# macOS VST3 installation
cp -r target/bundled/VstKit.vst3 ~/Library/Audio/Plug-Ins/VST3/

# macOS AU installation
cp -r target/bundled/VstKit.component ~/Library/Audio/Plug-Ins/Components/

# Refresh macOS AU cache (required after AU updates)
killall -9 AudioComponentRegistrar

# Validate AU plugin (must pass before Logic Pro testing)
auval -v aufx vsk1 VstK
```
