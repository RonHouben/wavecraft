# Low-Level Design: Plugin UI Integration

**Feature Name:** `plugin-ui-integration`  
**Milestone:** 3 (Plugin UI Integration)  
**Status:** Draft  
**Created:** 2026-01-30  
**Author:** Architect Agent

---

## 1. Overview

### 1.1 Purpose

This document specifies the architecture for integrating the WebView-based React UI into the nih-plug plugin as a native editor. It bridges the standalone desktop POC (Milestone 2) with the actual plugin environment running inside a DAW host.

### 1.2 Scope

- WebView integration with nih-plug's `Editor` trait
- Parameter synchronization (UI ↔ host, bidirectional)
- Audio → UI metering via SPSC ring buffer
- Platform-specific native window handling (macOS primary, Windows secondary)

### 1.3 Out of Scope

- Linux support (deferred)
- Complex DSP visualizations (waveforms, FFT)
- Preset management
- MIDI handling

### 1.4 Success Criteria

| Criterion | Target |
|-----------|--------|
| Editor opens in Ableton Live (VST3) | Pass |
| Parameter changes from UI reflect in host automation | ≤16ms latency |
| Host automation changes reflect in UI | ≤16ms latency |
| Peak meter updates at UI refresh rate | 30-60 Hz |
| No audio dropouts at 64-sample buffer | 0 xruns in 5-minute stress test |

---

## 2. Architecture Overview

### 2.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Plugin Process (single binary)                                             │
│                                                                             │
│  ┌──────────────────┐     ┌──────────────────────────────────────────────┐ │
│  │  Audio Thread    │     │  UI Thread (main thread)                     │ │
│  │  ───────────────│     │  ────────────────────────────────────────── │ │
│  │                  │     │                                              │ │
│  │  ┌────────────┐  │     │  ┌────────────┐    ┌───────────────────────┐│ │
│  │  │ Processor  │  │     │  │ WebView    │    │ React UI              ││ │
│  │  │ (DSP)      │  │     │  │ (wry)      │◄──►│ (embedded assets)     ││ │
│  │  └─────┬──────┘  │     │  └─────┬──────┘    └───────────────────────┘│ │
│  │        │         │     │        │                                     │ │
│  │        │ reads   │     │        │ IPC (JSON-RPC)                      │ │
│  │        ▼         │     │        ▼                                     │ │
│  │  ┌────────────┐  │     │  ┌────────────────────────────┐             │ │
│  │  │ Params     │◄─┼─────┼──┤ PluginEditorBridge         │             │ │
│  │  │ (atomics)  │  │     │  │ - implements ParameterHost │             │ │
│  │  └────────────┘  │     │  │ - wraps ParamSetter        │             │ │
│  │        │         │     │  └────────────────────────────┘             │ │
│  │        │ writes  │     │                                              │ │
│  │        ▼         │     │                                              │ │
│  │  ┌────────────┐  │     │  ┌────────────────┐                         │ │
│  │  │ MeterRing  │──┼─────┼─►│ MeterConsumer  │                         │ │
│  │  │ (SPSC)     │  │     │  │ (polls buffer) │                         │ │
│  │  └────────────┘  │     │  └────────────────┘                         │ │
│  │                  │     │                                              │ │
│  └──────────────────┘     └──────────────────────────────────────────────┘ │
│                                                                             │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  Host Interface (nih-plug)                                            │  │
│  │  - VstKitParams: parameter definitions (Params trait)                 │  │
│  │  - VstKitPlugin: Plugin trait impl, owns Processor + MeterRing        │  │
│  │  - VstKitEditor: Editor trait impl, owns WebView + Bridge             │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Thread Model

| Thread | Responsibilities | Constraints |
|--------|------------------|-------------|
| **Audio** | DSP processing, param reads (atomic), meter writes | Real-time safe: no alloc, no locks, no syscalls |
| **UI (main)** | WebView, IPC, param writes via host API, meter reads | May block briefly; no audio work |
| **Host callback** | Parameter automation, editor lifecycle | Called from host's thread; keep fast |

### 2.3 Ownership Model

```rust
// Plugin owns shared state
struct VstKitPlugin {
    params: Arc<VstKitParams>,           // Shared with editor
    processor: Processor,
    meter_producer: MeterProducer,       // Writes on audio thread
}

// Editor is created/destroyed by host, receives shared refs
struct VstKitEditor {
    params: Arc<VstKitParams>,           // Shared with plugin
    param_setter: Arc<ParamSetter>,      // Host communication (from editor())
    meter_consumer: MeterConsumer,       // Reads on UI thread
    webview: Option<WebView>,            // Created in open(), dropped in close()
    bridge: Arc<PluginEditorBridge>,     // IPC handler
}
```

---

## 3. Editor Lifecycle

### 3.1 nih-plug Editor Trait

```rust
pub trait Editor: Send {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send>;

    fn size(&self) -> (u32, u32);
    fn set_scale_factor(&self, factor: f32) -> bool;
    fn param_value_changed(&self, id: &str, normalized_value: f32);
    fn param_modulation_changed(&self, id: &str, modulation_offset: f32);
}
```

### 3.2 Lifecycle State Machine

```
                    ┌───────────────────┐
                    │   Plugin::editor()│
                    │   called by host  │
                    └─────────┬─────────┘
                              │
                              ▼
                    ┌───────────────────┐
                    │ Editor Created    │
                    │ (no window yet)   │
                    └─────────┬─────────┘
                              │
              host calls spawn(parent, context)
                              │
                              ▼
         ┌─────────────────────────────────────────┐
         │              Editor Open                 │
         │  - WebView created with parent handle   │
         │  - IPC bridge active                    │
         │  - Meter polling started               │
         │  - Parameter sync active               │
         └─────────────────┬───────────────────────┘
                           │
           returned handle dropped / host closes window
                           │
                           ▼
                 ┌───────────────────┐
                 │  Editor Closed    │
                 │  - WebView dropped│
                 │  - IPC stopped    │
                 └───────────────────┘
```

### 3.3 Window Handle Integration

The host provides a `ParentWindowHandle` (from `raw-window-handle` crate) that the WebView must attach to:

```rust
impl Editor for VstKitEditor {
    fn spawn(
        &self,
        parent: ParentWindowHandle,
        context: Arc<dyn GuiContext>,
    ) -> Box<dyn Any + Send> {
        // Extract native handle based on platform
        let webview = match parent {
            ParentWindowHandle::AppKitNsView(ns_view) => {
                // macOS: attach to NSView
                create_macos_webview(ns_view, self.bridge.clone())
            }
            ParentWindowHandle::Win32Hwnd { hwnd, .. } => {
                // Windows: attach to HWND
                create_windows_webview(hwnd, self.bridge.clone())
            }
            _ => panic!("Unsupported platform"),
        };
        
        self.webview = Some(webview);
        
        // Return handle that will close editor when dropped
        Box::new(EditorHandle { /* ... */ })
    }
}
```

**Platform Considerations:**

| Platform | Parent Type | WebView Engine | Notes |
|----------|-------------|----------------|-------|
| macOS | `NSView*` | WKWebView | Must use `addSubview:` on parent |
| Windows | `HWND` | WebView2 | Requires WebView2 runtime installed |

---

## 4. Parameter Synchronization

### 4.1 Data Flow: UI → Audio (User Changes Parameter)

```
┌─────────────┐    IPC Request     ┌─────────────────┐
│   React UI  │───────────────────►│ PluginEditor-   │
│   (slider)  │   setParameter     │ Bridge          │
└─────────────┘   {id, value}      └────────┬────────┘
                                            │
                                            │ context.begin_set_parameter()
                                            │ context.set_value_normalized()
                                            │ context.end_set_parameter()
                                            ▼
                                   ┌─────────────────┐
                                   │ GuiContext      │
                                   │ (nih-plug)      │
                                   └────────┬────────┘
                                            │
                                            │ atomic write
                                            ▼
                                   ┌─────────────────┐
                                   │ VstKitParams    │
                                   │ (atomics)       │
                                   └────────┬────────┘
                                            │
                                            │ smoothed.next()
                                            ▼
                                   ┌─────────────────┐
                                   │ Audio Thread    │
                                   │ process()       │
                                   └─────────────────┘
```

### 4.2 Data Flow: Audio → UI (Host Automation)

```
┌─────────────┐    automation      ┌─────────────────┐
│   DAW Host  │───────────────────►│ VstKitParams    │
│   (lane)    │   param write      │ (atomics)       │
└─────────────┘                    └────────┬────────┘
                                            │
                                            │ param_value_changed()
                                            ▼
                                   ┌─────────────────┐    Push Update
                                   │ VstKitEditor    │──────────────────┐
                                   │                 │                  │
                                   └─────────────────┘                  │
                                                                        │
                                                    evaluate_script()   │
                                                                        ▼
                                                              ┌─────────────────┐
                                                              │ React UI        │
                                                              │ state update    │
                                                              └─────────────────┘
```

### 4.3 PluginEditorBridge Implementation

This bridge implements the existing `ParameterHost` trait but wraps nih-plug's `GuiContext`:

```rust
/// Bridge between IPC handler and nih-plug parameter system
pub struct PluginEditorBridge {
    params: Arc<VstKitParams>,
    context: Arc<dyn GuiContext>,
}

impl ParameterHost for PluginEditorBridge {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        // Look up parameter by string ID
        let param = self.params.get_param_by_id(id)?;
        
        Some(ParameterInfo {
            id: id.to_string(),
            name: param.name().to_string(),
            param_type: ParameterType::Float,
            value: param.unmodulated_normalized_value(),
            default: param.default_normalized_value(),
            unit: param.unit().map(|u| u.to_string()),
        })
    }

    fn set_parameter(&self, id: &str, normalized_value: f32) -> Result<(), BridgeError> {
        let param = self.params.get_param_by_id(id)
            .ok_or_else(|| BridgeError::ParameterNotFound(id.to_string()))?;
        
        // Use ParamSetter to notify host (enables undo, automation recording)
        self.context.begin_set_parameter(param);
        self.context.set_value_normalized(param, normalized_value);
        self.context.end_set_parameter(param);
        
        Ok(())
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        // Iterate all params from VstKitParams
        self.params.iter_params()
            .map(|p| self.param_to_info(p))
            .collect()
    }
}
```

### 4.4 Host Automation → UI Updates

nih-plug calls `param_value_changed()` on the editor when the host changes a parameter:

```rust
impl Editor for VstKitEditor {
    fn param_value_changed(&self, id: &str, normalized_value: f32) {
        // Must push update to WebView
        if let Some(webview) = &self.webview {
            let update = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "paramUpdate",
                "params": {
                    "id": id,
                    "value": normalized_value
                }
            });
            
            let js = format!(
                "window.__VSTKIT_IPC__._onParamUpdate({});",
                serde_json::to_string(&update).unwrap()
            );
            
            let _ = webview.evaluate_script(&js);
        }
    }
}
```

### 4.5 Parameter Normalization

nih-plug uses normalized values (0.0–1.0) internally. The UI must work in display values (e.g., -24 to +24 dB):

| Direction | Conversion | Who Does It |
|-----------|------------|-------------|
| UI → Plugin | Display → Normalized | React UI (before IPC call) |
| Plugin → UI | Normalized → Display | React UI (after receiving update) |

```typescript
// ui/src/lib/vstkit-ipc/normalization.ts
export function displayToNormalized(value: number, min: number, max: number): number {
  return (value - min) / (max - min);
}

export function normalizedToDisplay(normalized: number, min: number, max: number): number {
  return min + normalized * (max - min);
}
```

---

## 5. Metering (Audio → UI)

### 5.1 Ring Buffer Design

Use an SPSC ring buffer for lock-free, allocation-free metering data transfer:

```rust
// New crate: engine/crates/metering/

/// Meter frame written by audio thread
#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct MeterFrame {
    pub peak_l: f32,
    pub peak_r: f32,
    pub rms_l: f32,
    pub rms_r: f32,
    pub timestamp: u64,  // Sample position
}

/// Producer side (audio thread)
pub struct MeterProducer {
    producer: rtrb::Producer<MeterFrame>,
}

impl MeterProducer {
    /// Write meter frame. Called once per audio buffer (real-time safe).
    #[inline]
    pub fn push(&mut self, frame: MeterFrame) {
        // Non-blocking: if buffer full, drop oldest (consumer too slow)
        let _ = self.producer.push(frame);
    }
}

/// Consumer side (UI thread)
pub struct MeterConsumer {
    consumer: rtrb::Consumer<MeterFrame>,
}

impl MeterConsumer {
    /// Read all available frames. Returns the most recent.
    pub fn read_latest(&mut self) -> Option<MeterFrame> {
        let mut latest = None;
        while let Ok(frame) = self.consumer.pop() {
            latest = Some(frame);
        }
        latest
    }
}

/// Create producer/consumer pair
pub fn create_meter_channel(capacity: usize) -> (MeterProducer, MeterConsumer) {
    let (producer, consumer) = rtrb::RingBuffer::new(capacity);
    (
        MeterProducer { producer },
        MeterConsumer { consumer },
    )
}
```

### 5.2 Audio Thread Integration

```rust
impl Plugin for VstKitPlugin {
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Calculate meters for this buffer
        let (peak_l, peak_r, rms_l, rms_r) = calculate_meters(buffer);
        
        // Push to ring buffer (non-blocking, never fails)
        self.meter_producer.push(MeterFrame {
            peak_l,
            peak_r,
            rms_l,
            rms_r,
            timestamp: context.transport().pos_samples().unwrap_or(0) as u64,
        });
        
        // ... rest of DSP processing ...
        
        ProcessStatus::Normal
    }
}

#[inline]
fn calculate_meters(buffer: &Buffer) -> (f32, f32, f32, f32) {
    let mut peak_l = 0.0f32;
    let mut peak_r = 0.0f32;
    let mut sum_sq_l = 0.0f32;
    let mut sum_sq_r = 0.0f32;
    
    let samples = buffer.samples();
    for i in 0..samples {
        let l = buffer.as_slice()[0][i];
        let r = buffer.as_slice()[1][i];
        
        peak_l = peak_l.max(l.abs());
        peak_r = peak_r.max(r.abs());
        sum_sq_l += l * l;
        sum_sq_r += r * r;
    }
    
    let rms_l = (sum_sq_l / samples as f32).sqrt();
    let rms_r = (sum_sq_r / samples as f32).sqrt();
    
    (peak_l, peak_r, rms_l, rms_r)
}
```

### 5.3 UI Thread Polling

The editor polls the meter buffer and pushes updates to the WebView:

```rust
impl VstKitEditor {
    /// Called periodically from UI thread (e.g., via timer or idle callback)
    fn poll_meters(&mut self) {
        if let Some(frame) = self.meter_consumer.read_latest() {
            if let Some(webview) = &self.webview {
                let update = serde_json::json!({
                    "type": "meterFrame",
                    "peakL": frame.peak_l,
                    "peakR": frame.peak_r,
                    "rmsL": frame.rms_l,
                    "rmsR": frame.rms_r,
                });
                
                let js = format!(
                    "window.__VSTKIT_IPC__._onMeterFrame({});",
                    serde_json::to_string(&update).unwrap()
                );
                
                let _ = webview.evaluate_script(&js);
            }
        }
    }
}
```

### 5.4 React Meter Component

```typescript
// ui/src/components/Meter.tsx
import { useEffect, useState } from 'react';
import { onMeterFrame, MeterFrame } from '../lib/vstkit-ipc';

export function Meter() {
  const [meters, setMeters] = useState<MeterFrame | null>(null);
  
  useEffect(() => {
    const unsubscribe = onMeterFrame(setMeters);
    return unsubscribe;
  }, []);
  
  if (!meters) return null;
  
  const peakLDb = linearToDb(meters.peakL);
  const peakRDb = linearToDb(meters.peakR);
  
  return (
    <div className="meter">
      <MeterBar value={peakLDb} min={-60} max={6} />
      <MeterBar value={peakRDb} min={-60} max={6} />
    </div>
  );
}

function linearToDb(linear: number): number {
  if (linear <= 0) return -Infinity;
  return 20 * Math.log10(linear);
}
```

---

## 6. Platform-Specific WebView Implementation

### 6.1 macOS (WKWebView)

```rust
// engine/crates/plugin/src/editor/macos.rs

use objc2::rc::Id;
use objc2_foundation::NSRect;
use objc2_webkit::{WKWebView, WKWebViewConfiguration};

pub fn create_macos_webview(
    parent_ns_view: *mut c_void,
    bridge: Arc<PluginEditorBridge>,
    assets: &'static EmbeddedAssets,
) -> WebViewHandle {
    // 1. Create WKWebViewConfiguration
    let config = WKWebViewConfiguration::new();
    
    // 2. Configure custom URL scheme handler for embedded assets
    config.set_url_scheme_handler(/* vstkit:// handler */);
    
    // 3. Configure user content controller for IPC
    let user_controller = config.user_content_controller();
    user_controller.add_script_message_handler(/* IPC handler */);
    
    // 4. Create WKWebView
    let frame = get_parent_frame(parent_ns_view);
    let webview = WKWebView::initWithFrame_configuration(frame, &config);
    
    // 5. Add as subview of parent
    let parent: Id<NSView> = Id::from_raw(parent_ns_view as *mut NSView);
    parent.addSubview(&webview);
    
    // 6. Load embedded content
    webview.loadRequest(/* vstkit://localhost/index.html */);
    
    WebViewHandle { inner: webview }
}
```

### 6.2 Windows (WebView2)

```rust
// engine/crates/plugin/src/editor/windows.rs

use webview2::Controller;
use windows::Win32::Foundation::HWND;

pub fn create_windows_webview(
    parent_hwnd: isize,
    bridge: Arc<PluginEditorBridge>,
    assets: &'static EmbeddedAssets,
) -> WebViewHandle {
    let hwnd = HWND(parent_hwnd);
    
    // 1. Create WebView2 environment (async, but we block briefly)
    let controller = block_on(async {
        let env = webview2::Environment::builder().build().await?;
        env.create_controller(hwnd).await
    })?;
    
    // 2. Configure IPC handler
    controller.add_web_message_received(|msg| {
        // Handle JSON-RPC messages
    });
    
    // 3. Navigate to embedded content
    controller.navigate("vstkit://localhost/index.html");
    
    WebViewHandle { inner: controller }
}
```

### 6.3 Abstraction Layer

```rust
// engine/crates/plugin/src/editor/webview.rs

/// Platform-agnostic WebView handle
pub struct WebViewHandle {
    #[cfg(target_os = "macos")]
    inner: MacOSWebView,
    
    #[cfg(target_os = "windows")]
    inner: WindowsWebView,
}

impl WebViewHandle {
    pub fn evaluate_script(&self, js: &str) -> Result<(), WebViewError> {
        #[cfg(target_os = "macos")]
        return self.inner.evaluate_javascript(js);
        
        #[cfg(target_os = "windows")]
        return self.inner.execute_script(js);
    }
    
    pub fn resize(&self, width: u32, height: u32) {
        // Platform-specific resize
    }
}
```

---

## 7. File Structure

### 7.1 New/Modified Crates

```
engine/crates/
├── plugin/
│   ├── Cargo.toml           # Add: wry, raw-window-handle deps
│   └── src/
│       ├── lib.rs           # Modified: add meter channel creation
│       ├── params.rs        # Unchanged
│       ├── editor.rs        # REPLACE: placeholder egui → WebView
│       └── editor/
│           ├── mod.rs       # Editor trait impl
│           ├── bridge.rs    # PluginEditorBridge (ParameterHost impl)
│           ├── webview.rs   # Platform abstraction
│           ├── macos.rs     # WKWebView specifics
│           └── windows.rs   # WebView2 specifics
│
├── metering/                # NEW CRATE
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs           # MeterFrame, channel creation
│       └── ring.rs          # SPSC wrapper
│
└── bridge/
    └── src/
        └── handler.rs       # Unchanged (reuse ParameterHost trait)
```

### 7.2 UI Additions

```
ui/src/
├── lib/
│   └── vstkit-ipc/
│       ├── index.ts         # Existing IPC client
│       ├── meters.ts        # NEW: meter subscription
│       └── normalization.ts # NEW: value conversion helpers
│
└── components/
    ├── Meter.tsx            # NEW: peak/RMS meter display
    └── Meter.css
```

---

## 8. IPC Protocol Extensions

### 8.1 New Methods (Plugin → UI Push)

These are not request/response; they are pushed via `evaluate_script()`:

```typescript
// Pushed when host automation changes a parameter
interface ParamUpdateNotification {
  jsonrpc: "2.0";
  method: "paramUpdate";
  params: {
    id: string;
    value: number;  // Normalized 0-1
  };
}

// Pushed periodically with meter data
interface MeterFrameNotification {
  type: "meterFrame";
  peakL: number;  // Linear 0-1+
  peakR: number;
  rmsL: number;
  rmsR: number;
}
```

### 8.2 UI-Side Subscription API

```typescript
// ui/src/lib/vstkit-ipc/meters.ts

type MeterCallback = (frame: MeterFrame) => void;
type ParamCallback = (id: string, value: number) => void;

// Global subscription registry
const meterSubscribers: Set<MeterCallback> = new Set();
const paramSubscribers: Map<string, Set<ParamCallback>> = new Map();

// Called from injected IPC primitives
window.__VSTKIT_IPC__._onMeterFrame = (frame: MeterFrame) => {
  meterSubscribers.forEach(cb => cb(frame));
};

window.__VSTKIT_IPC__._onParamUpdate = (update: { id: string; value: number }) => {
  const subs = paramSubscribers.get(update.id);
  if (subs) {
    subs.forEach(cb => cb(update.id, update.value));
  }
};

// Public API
export function onMeterFrame(callback: MeterCallback): () => void {
  meterSubscribers.add(callback);
  return () => meterSubscribers.delete(callback);
}

export function onParamChange(id: string, callback: ParamCallback): () => void {
  if (!paramSubscribers.has(id)) {
    paramSubscribers.set(id, new Set());
  }
  paramSubscribers.get(id)!.add(callback);
  return () => paramSubscribers.get(id)?.delete(callback);
}
```

---

## 9. Crate Dependencies

### 9.1 plugin crate (Cargo.toml additions)

```toml
[dependencies]
nih_plug = { version = "0.x", features = ["assert_process_allocs"] }

# WebView (replacing nih_plug_egui)
wry = { version = "0.47", default-features = false }
raw-window-handle = "0.6"

# For macOS WebView integration
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.5"
objc2-foundation = "0.2"
objc2-app-kit = "0.2"
objc2-webkit = "0.2"

# For Windows WebView integration
[target.'cfg(target_os = "windows")'.dependencies]
webview2 = "0.1"
windows = { version = "0.52", features = ["Win32_Foundation", "Win32_UI_WindowsAndMessaging"] }

# Shared
metering = { path = "../metering" }
bridge = { path = "../bridge" }
protocol = { path = "../protocol" }
```

### 9.2 New metering crate

```toml
[package]
name = "metering"
version = "0.1.0"
edition = "2021"

[dependencies]
rtrb = "0.3"  # SPSC ring buffer
```

---

## 10. Testing Strategy

### 10.1 Unit Tests

| Test | Location | Description |
|------|----------|-------------|
| `bridge_set_get_roundtrip` | `plugin/src/editor/bridge.rs` | Set param via bridge, read back |
| `meter_ring_push_pop` | `metering/src/lib.rs` | SPSC correctness |
| `normalization_roundtrip` | UI tests | Display ↔ normalized conversion |

### 10.2 Integration Tests

| Test | Description | Pass Criteria |
|------|-------------|---------------|
| Editor opens in test host | Use `nih-plug`'s standalone mode | Window visible, UI renders |
| UI → host param flow | Move slider, check host value | Host sees normalized value |
| Host → UI param flow | Automate param, check UI | UI reflects value |
| Meter data flows | Play audio, check UI meters | Meters animate |

### 10.3 Host Compatibility Tests

| Host | Platform | Format | Priority |
|------|----------|--------|----------|
| Ableton Live | macOS | VST3 | P0 |
| Ableton Live | Windows | VST3 | P1 |
| Logic Pro | macOS | AU | P1 |
| Reaper | macOS/Windows | VST3/CLAP | P2 |

---

## 11. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **wry doesn't work with plugin window handles** | Medium | High | Fallback: direct WKWebView/WebView2 APIs without wry |
| **Editor resize handling** | Medium | Medium | Follow host guidelines; test in multiple DAWs |
| **IPC latency spikes** | Low | Medium | Profile; batch updates if needed |
| **WebView2 not installed on Windows** | Medium | High | Graceful fallback message; document requirement |
| **Meter ring overflows** | Low | Low | Size buffer for worst case (60Hz UI / 32 sample buffers) |
| **Threading issues in param sync** | Medium | High | Use nih-plug's setter APIs correctly; test automation |

---

## 12. Open Questions

1. **wry vs raw platform APIs**: Should we use wry for the plugin editor, or go directly to WKWebView/WebView2 APIs? 
   - *Recommendation*: Try wry first; it worked for desktop POC. Fall back if issues arise.

2. **Meter polling mechanism**: How to trigger periodic meter polls in the editor?
   - *Options*: Timer thread, `requestAnimationFrame` in JS polling via IPC, idle callbacks
   - *Recommendation*: Let React poll via `requestAnimationFrame` → IPC request

3. **Editor resize**: How to handle dynamic resize (user drags window)?
   - *Recommendation*: Start with fixed size; add resize support as follow-up

4. **Multiple editor instances**: Can the host open multiple editor windows?
   - *Answer*: nih-plug handles this; each `spawn()` call creates independent editor

---

## 13. Implementation Order

### Phase 1: Core Editor (Est. 3-4 days)
1. Create `metering` crate with SPSC ring buffer
2. Refactor `plugin/src/editor.rs` to module structure
3. Implement `PluginEditorBridge` (wraps `GuiContext`)
4. Implement `VstKitEditor` with WebView creation (macOS first)
5. Test in nih-plug standalone mode

### Phase 2: Parameter Sync (Est. 2-3 days)
1. Wire IPC handler to `PluginEditorBridge`
2. Implement `param_value_changed()` push to UI
3. Update React components to handle host automation
4. Test bidirectional sync in Ableton

### Phase 3: Metering (Est. 2 days)
1. Integrate `MeterProducer` into plugin process
2. Implement meter polling in editor
3. Create React `Meter` component
4. Test visual meters in DAW

### Phase 4: Windows Support (Est. 2-3 days)
1. Implement WebView2 integration
2. Test in Windows DAWs
3. Handle WebView2 runtime detection

---

## 14. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2026-01-30 | 0.1 | Architect Agent | Initial draft |
