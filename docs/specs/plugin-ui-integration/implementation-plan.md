# Implementation Plan: Plugin UI Integration

## Overview

This plan details the implementation of WebView-based React UI integration into the nih-plug plugin as a native editor. It bridges the standalone desktop POC (Milestone 2) with the actual plugin environment running inside a DAW host. The implementation covers WebView integration with nih-plug's `Editor` trait, bidirectional parameter synchronization, and real-time metering via SPSC ring buffers.

## Requirements

- WebView editor opens in DAW hosts (Ableton Live VST3, Logic Pro AU)
- Parameter changes from UI reflect in host automation with ≤16ms latency
- Host automation changes reflect in UI with ≤16ms latency
- Peak meter updates at UI refresh rate (30-60 Hz)
- No audio dropouts at 64-sample buffer
- macOS primary support, Windows secondary

## Architecture Changes

- **New crate**: [engine/crates/metering](engine/crates/metering) - SPSC ring buffer for audio→UI meter data
- **Major refactor**: [engine/crates/plugin/src/editor.rs](engine/crates/plugin/src/editor.rs) - Replace egui with WebView + modular structure
- **New module**: [engine/crates/plugin/src/editor/](engine/crates/plugin/src/editor/) - Platform-specific WebView implementations
- **Extend**: [engine/crates/bridge/src/handler.rs](engine/crates/bridge/src/handler.rs) - Add meter frame IPC method
- **New UI files**: [ui/src/lib/vstkit-ipc/meters.ts](ui/src/lib/vstkit-ipc/meters.ts) - Meter polling API
- **New UI component**: [ui/src/components/Meter.tsx](ui/src/components/Meter.tsx) - Peak/RMS meter visualization

---

## Implementation Steps

### Phase 1: Metering Crate (Est. 1-2 days)

#### 1.1 **Create metering crate structure** (File: engine/crates/metering/Cargo.toml)
- Action: Create new crate with `rtrb` dependency for SPSC ring buffer
- Why: Provides lock-free, allocation-free metering data transfer from audio to UI thread
- Dependencies: None
- Risk: Low

```toml
[package]
name = "metering"
version = "0.1.0"
edition = "2021"

[dependencies]
rtrb = "0.3"
```

#### 1.2 **Implement MeterFrame and channel creation** (File: engine/crates/metering/src/lib.rs)
- Action: Define `MeterFrame` struct, `MeterProducer`, `MeterConsumer`, and `create_meter_channel()` function
- Why: Core data structures for audio→UI metering communication
- Dependencies: Step 1.1
- Risk: Low

**Key types:**
```rust
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MeterFrame {
    pub peak_l: f32,
    pub peak_r: f32,
    pub rms_l: f32,
    pub rms_r: f32,
    pub timestamp: u64,
}

pub struct MeterProducer { producer: rtrb::Producer<MeterFrame> }
pub struct MeterConsumer { consumer: rtrb::Consumer<MeterFrame> }
```

#### 1.3 **Add metering crate to workspace** (File: engine/Cargo.toml)
- Action: Add metering crate to workspace members and dependencies
- Why: Make crate available to plugin crate
- Dependencies: Step 1.2
- Risk: Low

#### 1.4 **Unit test meter ring buffer** (File: engine/crates/metering/src/lib.rs)
- Action: Add tests for push/pop, buffer overflow behavior, read_latest semantics
- Why: Ensure correctness of lock-free data structure
- Dependencies: Step 1.2
- Risk: Low

---

### Phase 2: Plugin-Level Metering Integration (Est. 1 day)

#### 2.1 **Add metering dependency to plugin crate** (File: engine/crates/plugin/Cargo.toml)
- Action: Add `metering.workspace = true` to dependencies
- Why: Enable plugin to use meter producer
- Dependencies: Phase 1 complete
- Risk: Low

#### 2.2 **Integrate MeterProducer into VstKitPlugin** (File: engine/crates/plugin/src/lib.rs)
- Action: Add `meter_producer: MeterProducer` field to `VstKitPlugin`, create channel in `default()`
- Why: Audio thread needs producer to write meter data
- Dependencies: Step 2.1
- Risk: Low

**Changes needed:**
- Add `meter_producer` and `meter_consumer_holder` fields
- Create channel in `Default::default()`
- Store consumer for later transfer to editor

#### 2.3 **Push meter frames in process()** (File: engine/crates/plugin/src/lib.rs)
- Action: Calculate peak/RMS per buffer and push to ring buffer at end of `process()`
- Why: Generate metering data for UI consumption
- Dependencies: Step 2.2
- Risk: Low

**Implementation:**
```rust
fn process(&mut self, buffer: &mut Buffer, ...) -> ProcessStatus {
    // ... existing DSP ...
    
    // Calculate meters (after processing)
    let (peak_l, peak_r, rms_l, rms_r) = calculate_meters(buffer);
    self.meter_producer.push(MeterFrame { peak_l, peak_r, rms_l, rms_r, timestamp: 0 });
    
    ProcessStatus::Normal
}
```

---

### Phase 3: Editor Module Refactoring (Est. 2-3 days)

#### 3.1 **Convert editor.rs to module directory** (File: engine/crates/plugin/src/editor/)
- Action: Create `editor/` directory with `mod.rs` (move existing code), prepare for new submodules
- Why: Need modular structure for bridge, webview, platform-specific code
- Dependencies: None
- Risk: Medium (must maintain existing egui editor functionality during transition)

**New structure:**
```
engine/crates/plugin/src/
├── lib.rs
├── params.rs
└── editor/
    ├── mod.rs        # Re-exports, EditorState enum
    ├── egui.rs       # Existing egui editor (temporary)
    ├── bridge.rs     # PluginEditorBridge
    ├── webview.rs    # Platform abstraction
    ├── macos.rs      # WKWebView implementation
    └── windows.rs    # WebView2 implementation (stub initially)
```

#### 3.2 **Create PluginEditorBridge** (File: engine/crates/plugin/src/editor/bridge.rs)
- Action: Implement `ParameterHost` trait that wraps nih-plug's `GuiContext`
- Why: Bridge between IPC handler and nih-plug parameter system
- Dependencies: Step 3.1
- Risk: Medium (must correctly use nih-plug's parameter setter APIs)

**Key implementation:**
```rust
pub struct PluginEditorBridge {
    params: Arc<VstKitParams>,
    context: Arc<dyn GuiContext>,
    meter_consumer: Mutex<MeterConsumer>,
}

impl ParameterHost for PluginEditorBridge {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> { ... }
    fn set_parameter(&self, id: &str, value: f32) -> Result<(), BridgeError> {
        // Use context.begin_set_parameter(), set_value_normalized(), end_set_parameter()
    }
    fn get_all_parameters(&self) -> Vec<ParameterInfo> { ... }
}
```

#### 3.3 **Add get_param_by_id helper to VstKitParams** (File: engine/crates/plugin/src/params.rs)
- Action: Add method to look up parameter by string ID for the bridge
- Why: Bridge needs to find parameters by their IPC string identifiers
- Dependencies: None
- Risk: Low

#### 3.4 **Add getMeterFrame IPC method** (File: engine/crates/bridge/src/handler.rs)
- Action: Extend `IpcHandler` to handle `getMeterFrame` method, add trait method to `ParameterHost`
- Why: UI needs to poll meter data via IPC
- Dependencies: Step 3.2
- Risk: Low

**Protocol extension:**
```rust
const METHOD_GET_METER_FRAME: &str = "getMeterFrame";

// In ParameterHost trait (or new MeterHost trait)
fn get_meter_frame(&self) -> Option<MeterFrame>;
```

---

### Phase 4: WebView Editor Implementation - macOS (Est. 3-4 days)

#### 4.1 **Add WebView dependencies to plugin crate** (File: engine/crates/plugin/Cargo.toml)
- Action: Add wry, raw-window-handle, and platform-specific dependencies
- Why: Enable WebView creation in plugin editor
- Dependencies: None
- Risk: Medium (dependency version compatibility with nih-plug)

```toml
[dependencies]
wry = { version = "0.47", default-features = false }
raw-window-handle = "0.6"

[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-foundation = "0.2"
objc2-app-kit = "0.2"
objc2-webkit = "0.2"
```

#### 4.2 **Implement VstKitEditor struct** (File: engine/crates/plugin/src/editor/mod.rs)
- Action: Create `VstKitEditor` implementing nih-plug's `Editor` trait
- Why: Core editor type that hosts WebView and handles lifecycle
- Dependencies: Steps 3.2, 4.1
- Risk: High (must correctly implement nih-plug Editor trait)

**Key structure:**
```rust
pub struct VstKitEditor {
    params: Arc<VstKitParams>,
    meter_consumer: Arc<Mutex<Option<MeterConsumer>>>,
    size: (u32, u32),
}

impl Editor for VstKitEditor {
    fn spawn(&self, parent: ParentWindowHandle, context: Arc<dyn GuiContext>) -> Box<dyn Any + Send>;
    fn size(&self) -> (u32, u32);
    fn set_scale_factor(&self, factor: f32) -> bool;
    fn param_value_changed(&self, id: &str, normalized_value: f32);
    fn param_modulation_changed(&self, id: &str, modulation_offset: f32);
}
```

#### 4.3 **Create platform abstraction layer** (File: engine/crates/plugin/src/editor/webview.rs)
- Action: Define `WebViewHandle` with platform-agnostic interface for script evaluation, resize
- Why: Abstract platform differences behind common interface
- Dependencies: Step 4.1
- Risk: Low

#### 4.4 **Implement macOS WebView integration** (File: engine/crates/plugin/src/editor/macos.rs)
- Action: Create WKWebView, attach to parent NSView, configure IPC and custom protocol
- Why: macOS is primary target platform
- Dependencies: Steps 4.2, 4.3
- Risk: High (platform-specific code, parent window attachment)

**Key challenges:**
- Attach WebView to host-provided NSView (must use `addSubview:`)
- Configure custom URL scheme handler for embedded assets
- Set up JavaScript message handler for IPC

#### 4.5 **Embed UI assets in plugin binary** (File: engine/crates/plugin/src/editor/assets.rs)
- Action: Create asset embedding module (similar to desktop crate's assets.rs)
- Why: Plugin needs embedded React UI assets without external files
- Dependencies: None
- Risk: Low (can reuse pattern from desktop crate)

#### 4.6 **Update plugin's editor() function** (File: engine/crates/plugin/src/lib.rs)
- Action: Replace egui editor creation with WebView editor creation
- Why: Switch to new editor implementation
- Dependencies: Steps 4.2, 4.4, 4.5
- Risk: Medium

**Feature flag approach (optional):**
```rust
#[cfg(feature = "egui_editor")]
fn editor(&mut self, _: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    editor::create_egui_editor(self.params.clone())
}

#[cfg(not(feature = "egui_editor"))]
fn editor(&mut self, _: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    editor::create_webview_editor(self.params.clone(), self.meter_consumer.take())
}
```

#### 4.7 **Test WebView editor in nih-plug standalone** (Manual test)
- Action: Run plugin in standalone mode, verify WebView opens with React UI
- Why: Validate basic functionality before DAW testing
- Dependencies: Steps 4.1-4.6
- Risk: Medium

---

### Phase 5: Bidirectional Parameter Synchronization (Est. 2-3 days)

#### 5.1 **Implement UI→Plugin parameter flow** (Files: editor/bridge.rs, editor/mod.rs)
- Action: Wire IPC handler to PluginEditorBridge, ensure `setParameter` calls go through GuiContext
- Why: User slider changes must notify host for automation recording
- Dependencies: Phase 4 complete
- Risk: Medium

**Verification:** Move slider in UI → check host shows parameter change

#### 5.2 **Implement param_value_changed push** (File: engine/crates/plugin/src/editor/mod.rs)
- Action: In `param_value_changed()`, push update to WebView via `evaluate_script()`
- Why: Host automation must reflect in UI
- Dependencies: Step 5.1
- Risk: Medium

**Implementation:**
```rust
fn param_value_changed(&self, id: &str, normalized_value: f32) {
    if let Some(webview) = &self.webview {
        let js = format!(
            "window.__VSTKIT_IPC__._onParamUpdate({});",
            serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "method": "paramUpdate",
                "params": { "id": id, "value": normalized_value }
            })).unwrap()
        );
        let _ = webview.evaluate_script(&js);
    }
}
```

#### 5.3 **Add paramUpdate handler to IPC primitives** (File: engine/crates/desktop/src/js/ipc-primitives.js)
- Action: Add `_onParamUpdate` method to IPC primitives object
- Why: Receive pushed parameter updates from plugin
- Dependencies: None
- Risk: Low

#### 5.4 **Create onParamChange subscription API** (File: ui/src/lib/vstkit-ipc/index.ts)
- Action: Add `onParamChange(id, callback)` function for subscribing to host automation
- Why: React components need to react to external parameter changes
- Dependencies: Step 5.3
- Risk: Low

#### 5.5 **Update React hooks for bidirectional sync** (File: ui/src/lib/vstkit-ipc/hooks.ts)
- Action: Modify `useParameter` hook to subscribe to host automation changes
- Why: Complete bidirectional sync in React components
- Dependencies: Step 5.4
- Risk: Medium

---

### Phase 6: Metering UI Integration (Est. 1-2 days)

#### 6.1 **Add getMeterFrame to IPC protocol** (File: engine/crates/protocol/src/lib.rs)
- Action: Add method constant, request/result types for meter frame
- Why: Define protocol types for meter IPC
- Dependencies: None
- Risk: Low

#### 6.2 **Create meters TypeScript module** (File: ui/src/lib/vstkit-ipc/meters.ts)
- Action: Define `MeterFrame` interface and `getMeterFrame()` async function
- Why: TypeScript API for meter polling
- Dependencies: Step 6.1
- Risk: Low

#### 6.3 **Create Meter React component** (File: ui/src/components/Meter.tsx)
- Action: Build polling-based meter component using `requestAnimationFrame`
- Why: Visual feedback for audio levels
- Dependencies: Step 6.2
- Risk: Low

#### 6.4 **Add linear-to-dB conversion utilities** (File: ui/src/lib/vstkit-ipc/normalization.ts)
- Action: Create `linearToDb()`, `displayToNormalized()`, `normalizedToDisplay()` helpers
- Why: Convert between linear meter values and dB display
- Dependencies: None
- Risk: Low

#### 6.5 **Integrate Meter into App** (File: ui/src/App.tsx)
- Action: Add Meter component to main UI layout
- Why: Show meters in plugin UI
- Dependencies: Steps 6.3, 6.4
- Risk: Low

---

### Phase 7: Windows Support (Est. 2-3 days - can be parallel/deferred)

#### 7.1 **Add Windows dependencies** (File: engine/crates/plugin/Cargo.toml)
- Action: Add WebView2 and Windows crate dependencies under `[target.'cfg(target_os = "windows")'.dependencies]`
- Why: Enable Windows WebView support
- Dependencies: None
- Risk: Low

#### 7.2 **Implement Windows WebView integration** (File: engine/crates/plugin/src/editor/windows.rs)
- Action: Create WebView2 controller, attach to parent HWND, configure IPC
- Why: Windows DAW support
- Dependencies: Step 7.1
- Risk: High (not actively tested, may need runtime detection)

#### 7.3 **WebView2 runtime detection** (File: engine/crates/plugin/src/editor/windows.rs)
- Action: Detect WebView2 runtime presence, show error message if missing
- Why: Graceful degradation on systems without WebView2
- Dependencies: Step 7.2
- Risk: Medium

---

### Phase 8: DAW Integration Testing (Est. 2-3 days)

#### 8.1 **Test in Ableton Live (macOS VST3)**
- Action: Load plugin, test editor open/close, parameter automation, meters
- Why: Primary host validation
- Dependencies: Phases 1-6 complete
- Risk: Medium

**Test cases:**
- [ ] Editor opens without crash
- [ ] UI renders correctly
- [ ] Slider changes reflect in host
- [ ] Host automation reflects in UI
- [ ] Meters animate with audio
- [ ] No audio dropouts during UI interaction

#### 8.2 **Test in Logic Pro (AU)**
- Action: Build AU via xtask, load in Logic Pro, repeat test cases
- Why: AU host validation for macOS
- Dependencies: Step 8.1
- Risk: Medium

#### 8.3 **Test editor lifecycle edge cases**
- Action: Test rapid open/close, resize (if supported), multiple project load
- Why: Ensure stability across DAW interactions
- Dependencies: Step 8.1
- Risk: Low

#### 8.4 **Performance profiling**
- Action: Measure IPC latency, meter update rate, CPU usage
- Why: Verify ≤16ms latency requirement
- Dependencies: Step 8.1
- Risk: Low

---

## Testing Strategy

### Unit Tests
| Test | Location | Description |
|------|----------|-------------|
| `meter_ring_push_pop` | metering/src/lib.rs | SPSC correctness |
| `meter_ring_overflow` | metering/src/lib.rs | Buffer overflow handling |
| `bridge_set_get_roundtrip` | plugin/src/editor/bridge.rs | Set param via bridge, read back |
| `normalization_roundtrip` | UI tests | Display ↔ normalized conversion |

### Integration Tests
| Test | Description | Pass Criteria |
|------|-------------|---------------|
| Editor opens in standalone | Use nih-plug standalone mode | Window visible, UI renders |
| UI → host param flow | Move slider, check host value | Host sees normalized value |
| Host → UI param flow | Automate param, check UI | UI reflects value within 16ms |
| Meter data flows | Play audio, check UI meters | Meters animate correctly |

### Host Compatibility Matrix
| Host | Platform | Format | Priority | Status |
|------|----------|--------|----------|--------|
| Ableton Live | macOS | VST3 | P0 | ⬜ |
| Logic Pro | macOS | AU | P1 | ⬜ |
| Reaper | macOS | VST3/CLAP | P2 | ⬜ |
| Ableton Live | Windows | VST3 | P1 | ⬜ |
| FL Studio | Windows | VST3 | P2 | ⬜ |

---

## Risks & Mitigations

### High Risk
- **Risk**: wry doesn't work with plugin window handles (ParentWindowHandle)
  - Likelihood: Medium
  - Impact: High
  - Mitigation: Fallback to direct WKWebView/WebView2 APIs without wry. The LLD already accounts for this with platform-specific modules.

- **Risk**: Threading issues in param sync (race conditions, deadlocks)
  - Likelihood: Medium
  - Impact: High
  - Mitigation: Use nih-plug's setter APIs correctly (begin/set/end pattern); test with automation recording; stress test rapid parameter changes.

### Medium Risk
- **Risk**: Editor resize handling varies by host
  - Likelihood: Medium
  - Impact: Medium
  - Mitigation: Start with fixed size; add resize support as follow-up.

- **Risk**: WebView2 not installed on Windows
  - Likelihood: Medium
  - Impact: High (Windows-only)
  - Mitigation: Detect runtime presence; show graceful error; document requirement.

- **Risk**: IPC latency spikes under load
  - Likelihood: Low
  - Impact: Medium
  - Mitigation: Profile; implement update batching if needed.

### Low Risk
- **Risk**: Meter ring buffer overflow
  - Likelihood: Low
  - Impact: Low
  - Mitigation: Size buffer for worst case (60Hz UI / 32 sample buffers); drop oldest on overflow.

---

## Success Criteria

- [ ] Editor opens in Ableton Live (VST3) without crash
- [ ] Parameter changes from UI reflect in host automation with ≤16ms latency
- [ ] Host automation changes reflect in UI with ≤16ms latency
- [ ] Peak meter updates at 30-60 Hz
- [ ] No audio dropouts at 64-sample buffer in 5-minute stress test
- [ ] Editor opens in Logic Pro (AU)
- [ ] All unit tests pass
- [ ] All integration tests pass

---

## Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2026-01-30 | 1.0 | Planner Agent | Initial implementation plan |
