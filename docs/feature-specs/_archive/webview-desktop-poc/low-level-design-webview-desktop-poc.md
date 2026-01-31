# Low-Level Design — Milestone 2: WebView Desktop POC

**Scope:** Week 2–4  
**Objective:** Create a standalone desktop application that embeds a React UI via wry WebView, demonstrates bidirectional IPC communication, and validates the architecture for plugin integration.

---

## 1. Goals and Success Criteria

### Must Have
- Minimal React app (Vite + TypeScript) with modern tooling setup
- React app embedded in a Rust desktop window via `wry`
- Bidirectional IPC bridge (JSON-RPC style) with clear message contracts
- Successful `setParameter` / `getParameter` roundtrip verified
- Message latency measured and documented (target: < 5ms for UI responsiveness)
- Static assets bundled into Rust binary (no external file dependencies)
- macOS and Windows build verification

### Nice to Have
- Parameter change visualization with smooth animations
- Multiple parameter types (float, bool, enum) demonstrated
- Error handling in IPC layer with clear error messages
- Hot-reload development workflow

### Out of Scope
- Plugin window integration (Milestone 3)
- SPSC ring buffers for metering (Milestone 3)
- Real DSP processing
- Code signing and notarization

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Desktop Application (Rust)                        │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                     Application State                           │ │
│  │  ┌─────────────────────────────────────────────────────────┐   │ │
│  │  │  Parameters (simulated plugin state)                     │   │ │
│  │  │  - gain: AtomicF32                                       │   │ │
│  │  │  - bypass: AtomicBool                                    │   │ │
│  │  │  - mix: AtomicF32                                        │   │ │
│  │  └─────────────────────────────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                              │                                       │
│                              │                                       │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                    IPC Bridge (Layered)                        │ │
│  │                                                                 │ │
│  │  Rust Handler          ←→    Minimal JS Primitives (injected) │ │
│  │  - parse JSON request        window.__VSTKIT_IPC__ (internal)  │ │
│  │  - dispatch to handlers      - postMessage()                   │ │
│  │  - serialize response        - setReceiveCallback()            │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                              │                                       │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │                wry WebView (Platform Native)                   │ │
│  │                                                                 │ │
│  │  macOS: WKWebView    Windows: WebView2    Linux: WebKitGTK    │ │
│  │                                                                 │ │
│  │  ┌──────────────────────────────────────────────────────────┐  │ │
│  │  │              React SPA (Embedded Static Assets)          │  │ │
│  │  │                                                          │  │ │
│  │  │  - TypeScript + React 18                                │  │ │
│  │  │  - Vite build → bundled into Rust binary               │  │ │
│  │  │  - @vstkit/ipc library (typed, tree-shakeable)          │  │ │
│  │  │  - React hooks: import { useParameter } from '@vstkit/ipc' │ │
│  │  └──────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. Project Structure

```
vstkit/
├── engine/
│   ├── Cargo.toml                    # Workspace (add desktop crate)
│   └── crates/
│       ├── dsp/                      # (existing)
│       ├── plugin/                   # (existing)
│       ├── protocol/                 # (existing, extended)
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── params.rs
│       │       └── ipc.rs            # NEW: IPC message contracts
│       │
│       ├── bridge/                   # NEW: IPC implementation
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── handler.rs        # Message dispatch logic
│       │       ├── messages.rs       # Re-export protocol types
│       │       └── error.rs          # Bridge-specific errors
│       │
│       └── desktop/                  # NEW: Standalone desktop app (POC)
│           ├── Cargo.toml
│           ├── build.rs              # Embed UI assets at compile time
│           └── src/
│               ├── main.rs           # Application entry point
│               ├── app.rs            # Application state
│               ├── webview.rs        # wry setup and IPC wiring
│               └── assets.rs         # Embedded asset serving
│
└── ui/                               # NEW: React SPA
    ├── package.json
    ├── tsconfig.json
    ├── vite.config.ts
    ├── index.html
    └── src/
        ├── main.tsx                  # React entry point
        ├── App.tsx                   # Main component
        ├── vite-env.d.ts
        ├── lib/
        │   └── vstkit-ipc/           # @vstkit/ipc library
        │       ├── index.ts          # Public exports
        │       ├── IpcBridge.ts      # Low-level bridge class (wraps injected primitives)
        │       ├── ParameterClient.ts # High-level typed client class
        │       ├── types.ts          # TypeScript message types
        │       └── hooks.ts          # React hooks (useParameter, etc.)
        ├── components/
        │   ├── ParameterSlider.tsx
        │   ├── ParameterToggle.tsx
        │   └── LatencyMonitor.tsx    # Dev tool for measuring roundtrip
        └── styles/
            └── main.css
```

### Crate Responsibilities

| Crate | Responsibility | Dependencies |
|-------|----------------|--------------|
| `protocol` | IPC message contracts, param definitions | `serde` |
| `bridge` | IPC message handling and dispatch | `protocol`, `serde_json` |
| `desktop` | wry window, asset embedding, app state | `bridge`, `wry`, `tao`, `include_dir` |

---

## 4. Module-Level Design

### 4.1 `protocol/ipc` — Message Contracts

```rust
// engine/crates/protocol/src/ipc.rs

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 style request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    /// Unique request ID for correlation
    pub id: u64,
    /// Method name
    pub method: String,
    /// Method parameters (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 style response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    /// Correlates with request ID
    pub id: u64,
    /// Result (mutually exclusive with error)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error (mutually exclusive with result)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<IpcError>,
}

/// JSON-RPC 2.0 style error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Push notification from Rust to JS (no request ID)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcNotification {
    /// Event name
    pub event: String,
    /// Event data
    pub data: serde_json::Value,
}

// ─────────────────────────────────────────────────────────────────────
// Method-specific types
// ─────────────────────────────────────────────────────────────────────

/// Parameters for `getParameter` method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParameterParams {
    pub id: String,
}

/// Result for `getParameter` method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetParameterResult {
    pub id: String,
    pub value: f64,
    pub normalized: f64,
    pub display: String,
}

/// Parameters for `setParameter` method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParameterParams {
    pub id: String,
    pub value: f64,
    /// If true, treat `value` as normalized (0.0-1.0)
    #[serde(default)]
    pub normalized: bool,
}

/// Parameters for `getAllParameters` method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAllParametersResult {
    pub parameters: Vec<ParameterInfo>,
}

/// Full parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    pub id: String,
    pub name: String,
    pub value: f64,
    pub normalized: f64,
    pub default_value: f64,
    pub min: f64,
    pub max: f64,
    pub step: Option<f64>,
    pub unit: String,
}

/// Notification data for `parameterChanged` event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterChangedData {
    pub id: String,
    pub value: f64,
    pub normalized: f64,
    pub display: String,
}

// ─────────────────────────────────────────────────────────────────────
// Error codes (JSON-RPC 2.0 compatible)
// ─────────────────────────────────────────────────────────────────────

pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    
    // Application-specific codes (start at -32000)
    pub const PARAMETER_NOT_FOUND: i32 = -32000;
    pub const VALUE_OUT_OF_RANGE: i32 = -32001;
}
```

**Design Decisions:**
- JSON-RPC 2.0-inspired format for familiarity and tooling support
- Request/response correlation via numeric IDs
- Separate notification type for push events (no request ID)
- Strongly typed method parameters to catch errors at compile time

---

### 4.2 `bridge` — IPC Handler

```rust
// engine/crates/bridge/src/handler.rs

use protocol::ipc::*;
use serde_json::Value;
use std::sync::Arc;

/// Trait for handling IPC methods
pub trait ParameterHost: Send + Sync {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo>;
    fn set_parameter(&self, id: &str, value: f64, normalized: bool) -> Result<(), String>;
    fn get_all_parameters(&self) -> Vec<ParameterInfo>;
}

/// IPC message handler
pub struct IpcHandler<H: ParameterHost> {
    host: Arc<H>,
}

impl<H: ParameterHost> IpcHandler<H> {
    pub fn new(host: Arc<H>) -> Self {
        Self { host }
    }

    /// Process an incoming request and return a response
    pub fn handle_request(&self, request: IpcRequest) -> IpcResponse {
        let result = match request.method.as_str() {
            "getParameter" => self.handle_get_parameter(request.params),
            "setParameter" => self.handle_set_parameter(request.params),
            "getAllParameters" => self.handle_get_all_parameters(),
            "ping" => Ok(serde_json::json!({ "pong": true })),
            _ => Err(IpcError {
                code: error_codes::METHOD_NOT_FOUND,
                message: format!("Unknown method: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(value) => IpcResponse {
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(error) => IpcResponse {
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    fn handle_get_parameter(&self, params: Option<Value>) -> Result<Value, IpcError> {
        let params: GetParameterParams = params
            .ok_or_else(|| IpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing params".into(),
                data: None,
            })
            .and_then(|v| serde_json::from_value(v).map_err(|e| IpcError {
                code: error_codes::INVALID_PARAMS,
                message: e.to_string(),
                data: None,
            }))?;

        let info = self.host.get_parameter(&params.id).ok_or_else(|| IpcError {
            code: error_codes::PARAMETER_NOT_FOUND,
            message: format!("Parameter not found: {}", params.id),
            data: None,
        })?;

        let result = GetParameterResult {
            id: info.id,
            value: info.value,
            normalized: info.normalized,
            display: format!("{:.1} {}", info.value, info.unit),
        };

        serde_json::to_value(result).map_err(|e| IpcError {
            code: error_codes::INTERNAL_ERROR,
            message: e.to_string(),
            data: None,
        })
    }

    fn handle_set_parameter(&self, params: Option<Value>) -> Result<Value, IpcError> {
        let params: SetParameterParams = params
            .ok_or_else(|| IpcError {
                code: error_codes::INVALID_PARAMS,
                message: "Missing params".into(),
                data: None,
            })
            .and_then(|v| serde_json::from_value(v).map_err(|e| IpcError {
                code: error_codes::INVALID_PARAMS,
                message: e.to_string(),
                data: None,
            }))?;

        self.host
            .set_parameter(&params.id, params.value, params.normalized)
            .map_err(|msg| IpcError {
                code: error_codes::VALUE_OUT_OF_RANGE,
                message: msg,
                data: None,
            })?;

        Ok(serde_json::json!({ "success": true }))
    }

    fn handle_get_all_parameters(&self) -> Result<Value, IpcError> {
        let parameters = self.host.get_all_parameters();
        let result = GetAllParametersResult { parameters };
        
        serde_json::to_value(result).map_err(|e| IpcError {
            code: error_codes::INTERNAL_ERROR,
            message: e.to_string(),
            data: None,
        })
    }
}
```

**Design Decisions:**
- `ParameterHost` trait abstracts parameter storage (reusable for plugin integration)
- All methods return `Result<Value, IpcError>` for consistent error handling
- Synchronous handling (async not needed for parameter operations)
- `ping` method for latency testing

---

### 4.3 `desktop` — Application State

```rust
// engine/crates/desktop/src/app.rs

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use bridge::ParameterHost;
use protocol::ipc::ParameterInfo;

/// Simulated plugin state for desktop POC
pub struct AppState {
    /// Gain in dB, stored as fixed-point (value * 100)
    gain: AtomicU32,
    /// Bypass flag (0 = off, 1 = on)
    bypass: AtomicU32,
    /// Mix percentage (0-100)
    mix: AtomicU32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            gain: AtomicU32::new(float_to_fixed(0.0)),      // 0 dB default
            bypass: AtomicU32::new(0),                      // Off
            mix: AtomicU32::new(float_to_fixed(100.0)),    // 100% default
        }
    }
}

// Fixed-point conversion for atomic storage
fn float_to_fixed(v: f64) -> u32 {
    ((v + 1000.0) * 100.0) as u32  // Offset to handle negative values
}

fn fixed_to_float(v: u32) -> f64 {
    (v as f64 / 100.0) - 1000.0
}

impl ParameterHost for AppState {
    fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
        match id {
            "gain" => Some(ParameterInfo {
                id: "gain".into(),
                name: "Output Gain".into(),
                value: fixed_to_float(self.gain.load(Ordering::Relaxed)),
                normalized: self.normalize_gain(fixed_to_float(self.gain.load(Ordering::Relaxed))),
                default_value: 0.0,
                min: -24.0,
                max: 24.0,
                step: Some(0.1),
                unit: "dB".into(),
            }),
            "bypass" => Some(ParameterInfo {
                id: "bypass".into(),
                name: "Bypass".into(),
                value: self.bypass.load(Ordering::Relaxed) as f64,
                normalized: self.bypass.load(Ordering::Relaxed) as f64,
                default_value: 0.0,
                min: 0.0,
                max: 1.0,
                step: Some(1.0),
                unit: "".into(),
            }),
            "mix" => Some(ParameterInfo {
                id: "mix".into(),
                name: "Dry/Wet Mix".into(),
                value: fixed_to_float(self.mix.load(Ordering::Relaxed)),
                normalized: fixed_to_float(self.mix.load(Ordering::Relaxed)) / 100.0,
                default_value: 100.0,
                min: 0.0,
                max: 100.0,
                step: Some(1.0),
                unit: "%".into(),
            }),
            _ => None,
        }
    }

    fn set_parameter(&self, id: &str, value: f64, normalized: bool) -> Result<(), String> {
        match id {
            "gain" => {
                let v = if normalized { self.denormalize_gain(value) } else { value };
                if v < -24.0 || v > 24.0 {
                    return Err(format!("Gain out of range: {}", v));
                }
                self.gain.store(float_to_fixed(v), Ordering::Relaxed);
                Ok(())
            }
            "bypass" => {
                let v = if value >= 0.5 { 1 } else { 0 };
                self.bypass.store(v, Ordering::Relaxed);
                Ok(())
            }
            "mix" => {
                let v = if normalized { value * 100.0 } else { value };
                if v < 0.0 || v > 100.0 {
                    return Err(format!("Mix out of range: {}", v));
                }
                self.mix.store(float_to_fixed(v), Ordering::Relaxed);
                Ok(())
            }
            _ => Err(format!("Unknown parameter: {}", id)),
        }
    }

    fn get_all_parameters(&self) -> Vec<ParameterInfo> {
        vec![
            self.get_parameter("gain").unwrap(),
            self.get_parameter("bypass").unwrap(),
            self.get_parameter("mix").unwrap(),
        ]
    }
}

impl AppState {
    fn normalize_gain(&self, db: f64) -> f64 {
        (db + 24.0) / 48.0  // -24..+24 → 0..1
    }

    fn denormalize_gain(&self, normalized: f64) -> f64 {
        (normalized * 48.0) - 24.0  // 0..1 → -24..+24
    }
}
```

**Design Decisions:**
- Atomic storage for thread-safe access (mirrors real plugin state)
- Fixed-point conversion handles negative floats in unsigned atomics
- `Relaxed` ordering sufficient for UI polling (not real-time critical)
- Three parameter types: continuous (gain, mix), boolean (bypass)

---

### 4.4 `desktop` — WebView Setup

```rust
// engine/crates/desktop/src/webview.rs

use std::sync::Arc;
use tao::event_loop::EventLoop;
use tao::window::WindowBuilder;
use wry::WebViewBuilder;
use bridge::IpcHandler;
use crate::app::AppState;
use crate::assets::get_asset;

const WINDOW_WIDTH: u32 = 600;
const WINDOW_HEIGHT: u32 = 500;
const WINDOW_TITLE: &str = "VstKit — WebView POC";

pub fn run_app(state: Arc<AppState>) -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(WINDOW_TITLE)
        .with_inner_size(tao::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .with_resizable(true)
        .build(&event_loop)?;

    let handler = Arc::new(IpcHandler::new(state.clone()));

    // Custom protocol handler for serving embedded assets
    let webview = WebViewBuilder::new(&window)
        .with_custom_protocol("vstkit".into(), move |request| {
            let path = request.uri().path();
            let path = if path == "/" { "/index.html" } else { path };
            
            match get_asset(path) {
                Some((content, mime_type)) => {
                    wry::http::Response::builder()
                        .header("Content-Type", mime_type)
                        .header("Access-Control-Allow-Origin", "*")
                        .body(content.to_vec())
                        .unwrap()
                }
                None => {
                    wry::http::Response::builder()
                        .status(404)
                        .body(b"Not Found".to_vec())
                        .unwrap()
                }
            }
        })
        // IPC handler for messages from JavaScript
        .with_ipc_handler({
            let handler = handler.clone();
            move |message: String| {
                // Parse incoming JSON-RPC request
                match serde_json::from_str::<protocol::ipc::IpcRequest>(&message) {
                    Ok(request) => {
                        let response = handler.handle_request(request);
                        // Response sent back via evaluate_script (see below)
                        if let Ok(json) = serde_json::to_string(&response) {
                            // Queue response to be sent back to JS
                            // Note: In practice, we'd use a channel or callback
                            println!("[IPC] Response: {}", json);
                        }
                    }
                    Err(e) => {
                        eprintln!("[IPC] Parse error: {}", e);
                    }
                }
            }
        })
        // Inject minimal IPC primitives before page loads
        .with_initialization_script(include_str!("../js/ipc-primitives.js"))
        .with_url("vstkit://localhost/")?
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Wait;

        match event {
            tao::event::Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = tao::event_loop::ControlFlow::Exit,
            _ => {}
        }
    });
}
```

**Design Decisions:**
- Custom protocol (`vstkit://`) — see [Section 4.4.1](#441-custom-protocol-rationale-and-security) below
- Assets embedded at compile time via `include_dir` crate
- IPC messages parsed synchronously (async not needed for this POC)
- Initialization script injected to set up JS-side IPC bridge

---

#### 4.4.1 Custom Protocol Rationale and Security

**Why not use `file://` URLs?**

When loading local HTML/JS from `file://` URLs in WebViews, browsers enforce strict security restrictions that break modern web development patterns:

| Issue | Impact |
|-------|--------|
| **No defined origin** | `file://` URLs have no origin, breaking the web security model |
| **CORS blocking** | `fetch()` and `XMLHttpRequest` are blocked, even to local files |
| **ES modules fail** | `import` statements don't work from `file://` |
| **Service workers disabled** | Cannot be registered from `file://` origins |
| **localStorage issues** | May not persist or be shared properly |

**What the custom protocol does**

Using `vstkit://localhost/` creates a proper origin that the browser can reason about:

```
vstkit://localhost/index.html  →  Origin: vstkit://localhost
```

This provides:
- A well-defined origin for CORS checks
- Working ES modules and `fetch()` 
- Proper localStorage isolation per origin
- Standard web security model applies

**Security analysis**

The custom protocol implementation is **secure by design**:

| Concern | Why It's Not a Risk |
|---------|---------------------|
| **Network access** | Custom protocol is local-only — no actual network requests leave the machine |
| **Cross-origin attacks** | The `vstkit://` origin is isolated; external websites cannot access it |
| **File system access** | The protocol handler **only serves embedded assets** — no arbitrary file reads |
| **Injection attacks** | Assets are compiled into the binary; cannot be modified at runtime |

**The critical security guarantee** is in the protocol handler implementation:

```rust
.with_custom_protocol("vstkit".into(), move |request| {
    let path = request.uri().path();
    match get_asset(path) {  // ← Only reads from compiled-in bytes
        Some((content, mime_type)) => { /* serve it */ }
        None => { /* 404 */ }
    }
})
```

The `get_asset()` function reads from `include_dir!()` embedded assets — **never from the file system**. This means:

1. ✅ No path traversal attacks possible (e.g., `../../etc/passwd`)
2. ✅ No runtime file modification possible
3. ✅ Assets are integrity-verified at compile time

**What WOULD be insecure (we don't do this)**

```rust
// ⚠️ DANGEROUS — DO NOT DO THIS
.with_custom_protocol("vstkit".into(), move |request| {
    let path = request.uri().path();
    std::fs::read(path)  // ← Arbitrary file system access!
})
```

**Alternative approaches considered**

| Approach | Security | Tradeoffs |
|----------|----------|-----------|
| **Custom protocol** (chosen) | ✅ Best | Clean origin, no network, assets embedded |
| **Local HTTP server** | ⚠️ Risky | Opens network listener, port conflicts, firewall prompts |
| **Data URLs** | ✅ Safe | Painful for multiple files, no relative imports |
| **`file://` with flags** | ⚠️ Fragile | Platform-inconsistent, some features still broken |

**Conclusion**: The custom protocol is the **most secure** option because it provides a proper web origin without exposing any network interfaces or file system access.

---

### 4.5 `desktop` — Asset Embedding

```rust
// engine/crates/desktop/src/assets.rs

use include_dir::{include_dir, Dir};

/// Embedded UI assets (populated at compile time)
static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../../ui/dist");

/// Get an embedded asset by path
pub fn get_asset(path: &str) -> Option<(&'static [u8], &'static str)> {
    let path = path.trim_start_matches('/');
    
    ASSETS.get_file(path).map(|file| {
        let content = file.contents();
        let mime_type = guess_mime_type(path);
        (content, mime_type)
    })
}

fn guess_mime_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("woff2") => "font/woff2",
        Some("woff") => "font/woff",
        _ => "application/octet-stream",
    }
}
```

**Design Decisions:**
- `include_dir` embeds entire `ui/dist` directory at compile time
- Zero runtime file I/O (important for plugin security restrictions)
- MIME type inference from file extension

---

### 4.6 IPC Bridge Architecture (Layered Approach)

The IPC bridge uses a **layered architecture** that separates the injected runtime primitives from the application-level TypeScript library. This provides type safety, testability, and idiomatic module imports.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Architecture Layers                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Layer 3: React Hooks                                          │ │
│  │  import { useParameter } from '@vstkit/ipc';                   │ │
│  │  - Manages React state                                         │ │
│  │  - Handles subscriptions and cleanup                           │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                              │                                       │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Layer 2: Typed Client API                                     │ │
│  │  import { getParameter, setParameter } from '@vstkit/ipc';     │ │
│  │  - Type-safe method wrappers                                   │ │
│  │  - Business logic (validation, formatting)                     │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                              │                                       │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Layer 1: Bridge Module (TypeScript)                           │ │
│  │  import { invoke, on } from './bridge';                        │ │
│  │  - Request/response correlation                                │ │
│  │  - Event subscription management                               │ │
│  │  - Connects to injected primitives                             │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                              │                                       │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │  Layer 0: Injected Primitives (Rust → JS)                      │ │
│  │  window.__VSTKIT_IPC__ = { postMessage, setReceiveCallback }   │ │
│  │  - Minimal surface area                                        │ │
│  │  - Internal implementation detail                              │ │
│  │  - Never accessed directly by application code                 │ │
│  └────────────────────────────────────────────────────────────────┘ │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

#### 4.6.1 Layer 0: Injected Primitives (Rust)

Rust injects **only minimal internal primitives** — the application code never touches these directly.

```javascript
// engine/crates/desktop/src/js/ipc-primitives.js
// Injected by Rust via .with_initialization_script()

(function() {
  'use strict';
  
  // Internal namespace — NOT for application use
  const __VSTKIT_IPC__ = {
    // Callback for receiving messages from Rust
    _receiveCallback: null,
    
    /**
     * Send a message to Rust
     * @param {string} message - JSON-encoded message
     * @internal
     */
    postMessage(message) {
      // wry provides window.ipc.postMessage
      window.ipc.postMessage(message);
    },
    
    /**
     * Register callback for messages from Rust
     * @param {function} callback - Receives JSON string
     * @internal
     */
    setReceiveCallback(callback) {
      this._receiveCallback = callback;
    },
    
    /**
     * Called by Rust to deliver messages
     * @param {string} messageJson - JSON-encoded response/notification
     * @internal
     */
    _receive(messageJson) {
      if (this._receiveCallback) {
        this._receiveCallback(messageJson);
      }
    }
  };
  
  // Expose on window (internal use only)
  Object.defineProperty(window, '__VSTKIT_IPC__', {
    value: Object.freeze(__VSTKIT_IPC__),
    writable: false,
    configurable: false
  });
  
  console.debug('[VstKit] IPC primitives initialized');
})();
```

**Design Decisions:**
- Uses `Object.freeze()` to prevent tampering
- Uses `Object.defineProperty()` with `writable: false` for security
- Minimal API surface: only `postMessage` and `setReceiveCallback`
- Naming convention `__VSTKIT_IPC__` signals "internal, do not use"

---

### 4.7 @vstkit/ipc — TypeScript Library

The `@vstkit/ipc` library provides the clean, typed API that application code imports. It wraps the injected primitives and handles all the complexity of request/response correlation and event subscriptions.

> **Note:** Following our [coding standards](../../docs/architecture/coding-standards.md), all non-React TypeScript uses class syntax. React components and hooks remain functional.

#### 4.7.1 Types

```typescript
// ui/src/lib/vstkit-ipc/types.ts

/**
 * Parameter metadata returned from the plugin
 */
export interface ParameterInfo {
  id: string;
  name: string;
  value: number;
  normalized: number;
  defaultValue: number;
  min: number;
  max: number;
  step?: number;
  unit: string;
}

/**
 * IPC error returned from Rust
 */
export interface IpcError {
  code: number;
  message: string;
  data?: unknown;
}

/**
 * Parameter change notification data
 */
export interface ParameterChangedEvent {
  id: string;
  value: number;
  normalized: number;
  display: string;
}

/**
 * Pending request tracking
 * @internal
 */
export interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (error: Error) => void;
  timeoutId: ReturnType<typeof setTimeout>;
}

/**
 * Internal primitives injected by Rust (not for application use)
 * @internal
 */
export interface VstKitIpcPrimitives {
  postMessage: (message: string) => void;
  setReceiveCallback: (callback: (message: string) => void) => void;
}

declare global {
  interface Window {
    __VSTKIT_IPC__?: VstKitIpcPrimitives;
  }
}
```

#### 4.7.2 IpcBridge Class (Layer 1)

```typescript
// ui/src/lib/vstkit-ipc/IpcBridge.ts

import type { PendingRequest, VstKitIpcPrimitives } from './types';

const DEFAULT_TIMEOUT_MS = 5000;

/**
 * Low-level IPC bridge that wraps the injected primitives.
 * Handles request/response correlation and event subscriptions.
 * 
 * @example
 * ```typescript
 * const bridge = IpcBridge.getInstance();
 * const result = await bridge.invoke<{ value: number }>('getParameter', { id: 'gain' });
 * ```
 */
export class IpcBridge {
  private static instance: IpcBridge | null = null;

  private requestId = 0;
  private initialized = false;
  private readonly pendingRequests = new Map<number, PendingRequest>();
  private readonly eventListeners = new Map<string, Set<(data: unknown) => void>>();

  /**
   * Get the singleton instance of IpcBridge
   */
  static getInstance(): IpcBridge {
    if (!IpcBridge.instance) {
      IpcBridge.instance = new IpcBridge();
    }
    return IpcBridge.instance;
  }

  /**
   * Reset the singleton (for testing)
   * @internal
   */
  static resetInstance(): void {
    IpcBridge.instance = null;
  }

  private constructor() {
    // Private constructor for singleton pattern
  }

  /**
   * Check if IPC primitives are available
   */
  isAvailable(): boolean {
    return typeof window !== 'undefined' && !!window.__VSTKIT_IPC__;
  }

  /**
   * Invoke an IPC method and wait for the response
   * 
   * @param method - Method name (e.g., 'getParameter')
   * @param params - Method parameters
   * @returns Promise resolving to the result
   * @throws Error if the method fails or times out
   */
  async invoke<T>(method: string, params?: unknown): Promise<T> {
    this.ensureInitialized();

    const id = ++this.requestId;
    const primitives = this.getPrimitives();

    return new Promise<T>((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error(`[VstKit] IPC timeout: ${method}`));
        }
      }, DEFAULT_TIMEOUT_MS);

      this.pendingRequests.set(id, {
        resolve: (value) => {
          clearTimeout(timeoutId);
          resolve(value as T);
        },
        reject: (error) => {
          clearTimeout(timeoutId);
          reject(error);
        },
        timeoutId,
      });

      const request = JSON.stringify({ id, method, params });
      primitives.postMessage(request);
    });
  }

  /**
   * Subscribe to push notifications from Rust
   * 
   * @param event - Event name (e.g., 'parameterChanged')
   * @param callback - Handler function
   * @returns Unsubscribe function
   */
  on<T = unknown>(event: string, callback: (data: T) => void): () => void {
    this.ensureInitialized();

    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }

    const listeners = this.eventListeners.get(event)!;
    listeners.add(callback as (data: unknown) => void);

    return () => {
      listeners.delete(callback as (data: unknown) => void);
      if (listeners.size === 0) {
        this.eventListeners.delete(event);
      }
    };
  }

  // ─────────────────────────────────────────────────────────────────
  // Private methods
  // ─────────────────────────────────────────────────────────────────

  private getPrimitives(): VstKitIpcPrimitives {
    const primitives = window.__VSTKIT_IPC__;
    if (!primitives) {
      throw new Error(
        '[VstKit] IPC not available. Are you running outside the VstKit desktop app?'
      );
    }
    return primitives;
  }

  private ensureInitialized(): void {
    if (this.initialized) return;

    const primitives = this.getPrimitives();
    primitives.setReceiveCallback((msg) => this.handleMessage(msg));
    this.initialized = true;

    console.debug('[VstKit] IPC bridge initialized');
  }

  private handleMessage(messageJson: string): void {
    try {
      const message = JSON.parse(messageJson);

      // Response (has id)
      if (typeof message.id === 'number') {
        const pending = this.pendingRequests.get(message.id);
        if (pending) {
          this.pendingRequests.delete(message.id);

          if (message.error) {
            const err = new Error(message.error.message);
            (err as any).code = message.error.code;
            (err as any).data = message.error.data;
            pending.reject(err);
          } else {
            pending.resolve(message.result);
          }
        }
      }
      // Notification (has event)
      else if (typeof message.event === 'string') {
        const listeners = this.eventListeners.get(message.event);
        if (listeners) {
          listeners.forEach((callback) => {
            try {
              callback(message.data);
            } catch (e) {
              console.error('[VstKit] Event handler error:', e);
            }
          });
        }
      }
    } catch (e) {
      console.error('[VstKit] Failed to parse message:', e);
    }
  }
}
```

#### 4.7.3 ParameterClient Class (Layer 2)

```typescript
// ui/src/lib/vstkit-ipc/ParameterClient.ts

import { IpcBridge } from './IpcBridge';
import type { ParameterInfo, ParameterChangedEvent } from './types';

/**
 * High-level client for parameter operations.
 * Provides typed methods for interacting with plugin parameters.
 * 
 * @example
 * ```typescript
 * const client = ParameterClient.getInstance();
 * const gain = await client.getParameter('gain');
 * await client.setParameter('gain', -6.0);
 * ```
 */
export class ParameterClient {
  private static instance: ParameterClient | null = null;
  private readonly bridge: IpcBridge;

  /**
   * Get the singleton instance of ParameterClient
   */
  static getInstance(): ParameterClient {
    if (!ParameterClient.instance) {
      ParameterClient.instance = new ParameterClient(IpcBridge.getInstance());
    }
    return ParameterClient.instance;
  }

  /**
   * Reset the singleton (for testing)
   * @internal
   */
  static resetInstance(): void {
    ParameterClient.instance = null;
  }

  private constructor(bridge: IpcBridge) {
    this.bridge = bridge;
  }

  /**
   * Get a single parameter's current state
   */
  async getParameter(id: string): Promise<ParameterInfo> {
    return this.bridge.invoke<ParameterInfo>('getParameter', { id });
  }

  /**
   * Set a parameter's value
   * 
   * @param id - Parameter ID
   * @param value - New value (in parameter's native range, or 0-1 if normalized)
   * @param normalized - If true, value is treated as normalized (0.0 - 1.0)
   */
  async setParameter(id: string, value: number, normalized = false): Promise<void> {
    await this.bridge.invoke<{ success: boolean }>('setParameter', {
      id,
      value,
      normalized,
    });
  }

  /**
   * Get all parameters
   */
  async getAllParameters(): Promise<ParameterInfo[]> {
    const result = await this.bridge.invoke<{ parameters: ParameterInfo[] }>(
      'getAllParameters'
    );
    return result.parameters;
  }

  /**
   * Ping the Rust backend (for latency testing)
   * @returns Roundtrip time in milliseconds
   */
  async ping(): Promise<number> {
    const start = performance.now();
    await this.bridge.invoke<{ pong: boolean }>('ping');
    return performance.now() - start;
  }

  /**
   * Subscribe to parameter change notifications
   * 
   * @param callback - Called when any parameter changes
   * @returns Unsubscribe function
   */
  onParameterChanged(callback: (data: ParameterChangedEvent) => void): () => void {
    return this.bridge.on<ParameterChangedEvent>('parameterChanged', callback);
  }
}
```

#### 4.7.4 React Hooks (Layer 3)

React hooks remain functional, bridging between class-based clients and React components:

```typescript
// ui/src/lib/vstkit-ipc/hooks.ts

import { useState, useEffect, useCallback } from 'react';
import { ParameterClient } from './ParameterClient';
import type { ParameterInfo } from './types';

/**
 * Hook for managing a single parameter
 * 
 * @example
 * ```tsx
 * function GainControl() {
 *   const { param, setValue, isLoading } = useParameter('gain');
 *   
 *   if (isLoading) return <Spinner />;
 *   
 *   return (
 *     <Slider 
 *       value={param.value} 
 *       onChange={setValue}
 *       min={param.min}
 *       max={param.max}
 *     />
 *   );
 * }
 * ```
 */
export function useParameter(id: string) {
  const [param, setParam] = useState<ParameterInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const client = ParameterClient.getInstance();

  // Fetch initial value
  useEffect(() => {
    let cancelled = false;

    client
      .getParameter(id)
      .then((info) => {
        if (!cancelled) {
          setParam(info);
          setIsLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err);
          setIsLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [id, client]);

  // Subscribe to changes from Rust
  useEffect(() => {
    return client.onParameterChanged((data) => {
      if (data.id === id) {
        setParam((prev) =>
          prev ? { ...prev, value: data.value, normalized: data.normalized } : null
        );
      }
    });
  }, [id, client]);

  // Update value (optimistic update + sync)
  const setValue = useCallback(
    async (value: number) => {
      if (!param) return;

      // Optimistic update for responsive UI
      setParam((prev) => (prev ? { ...prev, value } : null));

      try {
        await client.setParameter(id, value);
      } catch (err) {
        // Revert on error
        const fresh = await client.getParameter(id);
        setParam(fresh);
        throw err;
      }
    },
    [id, param, client]
  );

  return { param, isLoading, error, setValue };
}

/**
 * Hook for all parameters
 */
export function useAllParameters() {
  const [params, setParams] = useState<ParameterInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const client = ParameterClient.getInstance();

  useEffect(() => {
    client
      .getAllParameters()
      .then(setParams)
      .finally(() => setIsLoading(false));
  }, [client]);

  // Subscribe to all parameter changes
  useEffect(() => {
    return client.onParameterChanged((data) => {
      setParams((prev) =>
        prev.map((p) =>
          p.id === data.id
            ? { ...p, value: data.value, normalized: data.normalized }
            : p
        )
      );
    });
  }, [client]);

  return { params, isLoading };
}

/**
 * Hook for measuring IPC latency
 */
export function useLatencyMonitor(intervalMs = 1000) {
  const [latency, setLatency] = useState<number | null>(null);
  const [samples, setSamples] = useState<number[]>([]);

  const client = ParameterClient.getInstance();

  useEffect(() => {
    const measure = async () => {
      const ms = await client.ping();
      setLatency(ms);
      setSamples((prev) => [...prev.slice(-99), ms]); // Keep last 100 samples
    };

    measure();
    const interval = setInterval(measure, intervalMs);
    return () => clearInterval(interval);
  }, [intervalMs, client]);

  const avg =
    samples.length > 0
      ? samples.reduce((a, b) => a + b, 0) / samples.length
      : null;

  const max = samples.length > 0 ? Math.max(...samples) : null;

  return { latency, avg, max, samples };
}
```

#### 4.7.5 Public Exports

```typescript
// ui/src/lib/vstkit-ipc/index.ts

// Types
export type {
  ParameterInfo,
  IpcError,
  ParameterChangedEvent,
} from './types';

// Classes (prefer using singleton instances via getInstance())
export { IpcBridge } from './IpcBridge';
export { ParameterClient } from './ParameterClient';

// React hooks (recommended for components)
export {
  useParameter, 
  useAllParameters, 
  useLatencyMonitor 
} from './hooks';
```

#### 4.7.6 Path Alias Configuration

```typescript
// ui/vite.config.ts

import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  base: './',
  resolve: {
    alias: {
      '@vstkit/ipc': path.resolve(__dirname, 'src/lib/vstkit-ipc'),
    },
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    assetsInlineLimit: 4096,
    rollupOptions: {
      output: {
        manualChunks: undefined,
      },
    },
  },
  server: {
    port: 3000,
  },
});
```

```json
// ui/tsconfig.json (paths section)
{
  "compilerOptions": {
    "paths": {
      "@vstkit/ipc": ["./src/lib/vstkit-ipc"],
      "@vstkit/ipc/*": ["./src/lib/vstkit-ipc/*"]
    }
  }
}
```

**Design Decisions:**
- **Layered architecture**: Each layer has a single responsibility
- **No globals in application code**: Only the bridge module touches `window.__VSTKIT_IPC__`
- **Tree-shakeable**: Unused exports can be eliminated by bundler
- **Testable**: Mock `@vstkit/ipc` module, not global `window` object
- **Type-safe**: Full TypeScript types from primitives to hooks
- **Path alias**: Clean imports like `import { useParameter } from '@vstkit/ipc'`

---

### 4.8 React UI — Components

```tsx
// ui/src/App.tsx

import { useAllParameters, useLatencyMonitor } from '@vstkit/ipc';
import { ParameterSlider } from './components/ParameterSlider';
import { ParameterToggle } from './components/ParameterToggle';
import { LatencyMonitor } from './components/LatencyMonitor';
import './styles/main.css';

export function App() {
  const { params, isLoading } = useAllParameters();
  const latencyInfo = useLatencyMonitor(500);
  
  if (isLoading) {
    return <div className="loading">Loading parameters...</div>;
  }
  
  const gain = params.find(p => p.id === 'gain');
  const bypass = params.find(p => p.id === 'bypass');
  const mix = params.find(p => p.id === 'mix');
  
  return (
    <div className="app">
      <header className="header">
        <h1>VstKit — WebView POC</h1>
        <LatencyMonitor {...latencyInfo} />
      </header>
      
      <main className="controls">
        {gain && (
          <ParameterSlider
            id={gain.id}
            name={gain.name}
            min={gain.min}
            max={gain.max}
            step={gain.step ?? 0.1}
            unit={gain.unit}
          />
        )}
        
        {bypass && (
          <ParameterToggle
            id={bypass.id}
            name={bypass.name}
          />
        )}
        
        {mix && (
          <ParameterSlider
            id={mix.id}
            name={mix.name}
            min={mix.min}
            max={mix.max}
            step={mix.step ?? 1}
            unit={mix.unit}
          />
        )}
      </main>
      
      <footer className="footer">
        <p>Milestone 2: WebView Desktop POC</p>
      </footer>
    </div>
  );
}
```

```tsx
// ui/src/components/ParameterSlider.tsx

import { useParameter } from '@vstkit/ipc';

interface Props {
  id: string;
  name: string;
  min: number;
  max: number;
  step: number;
  unit: string;
}

export function ParameterSlider({ id, name, min, max, step, unit }: Props) {
  const { param, setValue, isLoading } = useParameter(id);
  
  if (isLoading || !param) {
    return <div className="param-slider loading" />;
  }
  
  return (
    <div className="param-slider">
      <label className="param-label">
        <span className="param-name">{name}</span>
        <span className="param-value">
          {param.value.toFixed(1)} {unit}
        </span>
      </label>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={param.value}
        onChange={(e) => setValue(parseFloat(e.target.value))}
        className="param-range"
      />
    </div>
  );
}
```

```tsx
// ui/src/components/LatencyMonitor.tsx

interface Props {
  latency: number | null;
  avg: number | null;
  max: number | null;
}

export function LatencyMonitor({ latency, avg, max }: Props) {
  const getLatencyClass = (ms: number) => {
    if (ms < 2) return 'excellent';
    if (ms < 5) return 'good';
    if (ms < 10) return 'acceptable';
    return 'slow';
  };
  
  return (
    <div className="latency-monitor">
      <span className={`latency-value ${latency ? getLatencyClass(latency) : ''}`}>
        {latency !== null ? `${latency.toFixed(1)}ms` : '--'}
      </span>
      {avg !== null && (
        <span className="latency-stats">
          avg: {avg.toFixed(1)}ms / max: {max?.toFixed(1)}ms
        </span>
      )}
    </div>
  );
}
```

---

## 5. Build Configuration

### 5.1 Cargo Configuration

```toml
# engine/crates/protocol/Cargo.toml

[package]
name = "protocol"
version.workspace = true
edition.workspace = true

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

```toml
# engine/crates/bridge/Cargo.toml

[package]
name = "bridge"
version.workspace = true
edition.workspace = true

[dependencies]
protocol = { path = "../protocol" }
serde_json = "1.0"
```

```toml
# engine/crates/desktop/Cargo.toml

[package]
name = "desktop"
version.workspace = true
edition.workspace = true

[[bin]]
name = "vstkit-desktop"
path = "src/main.rs"

[dependencies]
bridge = { path = "../bridge" }
protocol = { path = "../protocol" }
wry = "0.47"
tao = "0.30"
include_dir = "0.7"
serde_json = "1.0"

[build-dependencies]
# Build script ensures UI is built before embedding
```

### 5.2 Vite Configuration

```typescript
// ui/vite.config.ts

import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: './',  // Relative paths for embedded serving
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    // Inline small assets to reduce HTTP requests
    assetsInlineLimit: 4096,
    // Single bundle for simplicity
    rollupOptions: {
      output: {
        manualChunks: undefined,
      },
    },
  },
  server: {
    // Dev server for hot reload during development
    port: 3000,
  },
});
```

```json
// ui/package.json

{
  "name": "vstkit-ui",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "typecheck": "tsc --noEmit"
  },
  "dependencies": {
    "react": "^18.3.0",
    "react-dom": "^18.3.0"
  },
  "devDependencies": {
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "typescript": "^5.5.0",
    "vite": "^5.4.0"
  }
}
```

---

## 6. Build and Development Workflow

### 6.1 Development Commands

```bash
# Terminal 1: React dev server (hot reload)
cd ui
npm install
npm run dev
# → Serves at http://localhost:3000

# Terminal 2: Rust desktop app (pointing to dev server)
cd engine
cargo run -p desktop -- --dev-url http://localhost:3000

# Or: Build everything and run with embedded assets
cd ui && npm run build
cd ../engine && cargo run -p desktop --release
```

### 6.2 Full Build

```bash
# Build UI
cd ui
npm ci
npm run build

# Build desktop app (embeds ui/dist at compile time)
cd ../engine
cargo build -p desktop --release

# Run
./target/release/vstkit-desktop
```

### 6.3 xtask Integration

```bash
# Add convenience commands to xtask
cd engine

# Build everything
cargo xtask desktop --release

# Build with dev server URL (for development)
cargo xtask desktop --dev
```

---

## 7. IPC Message Flow

### 7.1 Request/Response Sequence

```
┌──────────┐         ┌────────────┐         ┌──────────────┐
│  React   │         │  wry/IPC   │         │ Rust Handler │
│    UI    │         │  Bridge    │         │              │
└────┬─────┘         └─────┬──────┘         └──────┬───────┘
     │                     │                        │
     │ invoke('getParameter', {id: 'gain'})        │
     │─────────────────────>│                      │
     │                     │                        │
     │                     │ postMessage(JSON)      │
     │                     │───────────────────────>│
     │                     │                        │
     │                     │                        │ handle_request()
     │                     │                        │───────┐
     │                     │                        │       │
     │                     │                        │<──────┘
     │                     │                        │
     │                     │ evaluate_script(response)
     │                     │<───────────────────────│
     │                     │                        │
     │ ipc._receive(JSON)  │                        │
     │<─────────────────────│                       │
     │                     │                        │
     │ Promise resolves    │                        │
     │ {value: 0.0, ...}   │                        │
     │                     │                        │
```

### 7.2 Push Notification Sequence (Parameter Changed from Host)

```
┌──────────────┐         ┌────────────┐         ┌──────────┐
│ Rust Handler │         │  wry/IPC   │         │  React   │
│  (simulate   │         │  Bridge    │         │    UI    │
│   host)      │         │            │         │          │
└──────┬───────┘         └─────┬──────┘         └────┬─────┘
       │                       │                      │
       │ Parameter changed     │                      │
       │ (e.g., by automation) │                      │
       │───────────────────────>                      │
       │                       │                      │
       │                       │ evaluate_script(notification)
       │                       │─────────────────────>│
       │                       │                      │
       │                       │                      │ ipc._receive()
       │                       │                      │───────┐
       │                       │                      │       │
       │                       │                      │<──────┘
       │                       │                      │
       │                       │                      │ Event: 'parameterChanged'
       │                       │                      │ Listeners notified
       │                       │                      │
```

---

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
// engine/crates/bridge/src/handler.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    struct MockHost {
        gain: AtomicU32,
    }

    impl ParameterHost for MockHost {
        fn get_parameter(&self, id: &str) -> Option<ParameterInfo> {
            if id == "gain" {
                Some(ParameterInfo {
                    id: "gain".into(),
                    name: "Gain".into(),
                    value: 0.0,
                    normalized: 0.5,
                    default_value: 0.0,
                    min: -24.0,
                    max: 24.0,
                    step: Some(0.1),
                    unit: "dB".into(),
                })
            } else {
                None
            }
        }
        
        fn set_parameter(&self, _id: &str, _value: f64, _normalized: bool) -> Result<(), String> {
            Ok(())
        }
        
        fn get_all_parameters(&self) -> Vec<ParameterInfo> {
            vec![self.get_parameter("gain").unwrap()]
        }
    }

    #[test]
    fn test_get_parameter_success() {
        let host = Arc::new(MockHost { gain: AtomicU32::new(0) });
        let handler = IpcHandler::new(host);
        
        let request = IpcRequest {
            id: 1,
            method: "getParameter".into(),
            params: Some(serde_json::json!({ "id": "gain" })),
        };
        
        let response = handler.handle_request(request);
        
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    fn test_unknown_method() {
        let host = Arc::new(MockHost { gain: AtomicU32::new(0) });
        let handler = IpcHandler::new(host);
        
        let request = IpcRequest {
            id: 1,
            method: "unknownMethod".into(),
            params: None,
        };
        
        let response = handler.handle_request(request);
        
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, error_codes::METHOD_NOT_FOUND);
    }
}
```

### 8.2 Integration Tests (TypeScript)

```typescript
// ui/src/ipc/__tests__/client.test.ts

import { describe, it, expect, vi } from 'vitest';

describe('IPC Client', () => {
  it('should invoke methods and receive responses', async () => {
    // Mock window.ipc
    const mockInvoke = vi.fn().mockResolvedValue({ value: 0.0 });
    window.ipc = { invoke: mockInvoke } as any;
    
    const { getParameter } = await import('../client');
    const result = await getParameter('gain');
    
    expect(mockInvoke).toHaveBeenCalledWith('getParameter', { id: 'gain' });
    expect(result.value).toBe(0.0);
  });
});
```

### 8.3 Manual Test Checklist

| Test | Expected Result | Status |
|------|-----------------|--------|
| App launches without crash | Window opens with UI | ☐ |
| All parameters display initial values | Gain=0dB, Bypass=off, Mix=100% | ☐ |
| Slider drag updates value in real-time | Value changes smoothly | ☐ |
| Toggle button works | Bypass toggles on/off | ☐ |
| Latency monitor shows < 5ms average | Green "good" indicator | ☐ |
| Rapid slider movement doesn't lag | No perceptible delay | ☐ |
| Window resize doesn't break layout | UI scales appropriately | ☐ |
| App closes cleanly | No orphan processes | ☐ |

### 8.4 Latency Benchmarking

Target metrics for IPC roundtrip:

| Metric | Target | Action if Exceeded |
|--------|--------|---------------------|
| p50 latency | < 2ms | None (excellent) |
| p95 latency | < 5ms | None (acceptable) |
| p99 latency | < 10ms | Investigate |
| Max latency | < 50ms | Critical issue |

---

## 9. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| wry IPC latency too high | Low | High | Measure early; consider alternative message passing if > 10ms |
| WebView2 not installed on Windows | Medium | High | Bundle WebView2 bootstrapper or document requirement |
| Cross-platform rendering differences | Medium | Medium | Test on all platforms; use web-safe CSS only |
| Asset embedding increases binary size | Low | Low | Monitor size; consider compression if > 10MB |
| Memory leaks from JS↔Rust boundary | Medium | Medium | Profile with heap tools; ensure proper cleanup |

---

## 10. Definition of Done

Milestone 2 is complete when:

1. ☐ React app builds successfully with `cd ui && npm run build`
2. ☐ Desktop app builds with embedded assets: `cargo build -p desktop --release`
3. ☐ App launches and displays UI on macOS and Windows
4. ☐ `getAllParameters` returns three parameters with correct metadata
5. ☐ `setParameter` roundtrip completes within 5ms (p95)
6. ☐ Parameter changes from UI persist (verified by refresh)
7. ☐ Latency monitor shows metrics in real-time
8. ☐ Unit tests pass: `cargo test -p bridge -p protocol`
9. ☐ TypeScript compiles without errors: `cd ui && npm run typecheck`
10. ☐ Binary size < 20MB (release, stripped)

---

## 11. Next Steps (Preparing for Milestone 3)

The desktop POC architecture prepares for plugin integration:

| POC Component | Plugin Integration Path |
|---------------|------------------------|
| `ParameterHost` trait | Implement for `Arc<VstKitParams>` |
| `IpcHandler` | Reuse unchanged |
| `AppState` (atomics) | Replace with nih-plug's atomic params |
| wry WebView | Replace with `baseview` + platform-specific webview |
| Custom protocol handler | Adapt for plugin resource loading |
| React hooks | Reuse unchanged |

---

## Appendix A: Alternative Approaches Considered

### A.1 WebSocket Instead of postMessage

**Pros:**
- More familiar API
- Better debugging tools

**Cons:**
- Requires spawning a local server
- Additional network stack overhead
- Security concerns in plugin context

**Decision:** Use wry's native IPC (postMessage) for lower latency and simpler security model.

### A.2 MessagePack Instead of JSON

**Pros:**
- Smaller message size
- Faster serialization

**Cons:**
- Harder to debug
- Additional dependency

**Decision:** Use JSON for POC simplicity; can optimize later if benchmarks show bottleneck.

### A.3 Tauri Instead of Raw wry

**Pros:**
- More batteries-included
- Better tooling (CLI, bundler)

**Cons:**
- Heavier abstraction
- Harder to integrate with nih-plug

**Decision:** Use wry directly for maximum control and minimal overhead.

---

## Appendix B: File Tree Summary

```
vstkit/
├── engine/
│   └── crates/
│       ├── protocol/src/
│       │   ├── lib.rs          # Re-exports
│       │   ├── params.rs       # (existing)
│       │   └── ipc.rs          # NEW: IPC contracts
│       │
│       ├── bridge/src/
│       │   ├── lib.rs          # Re-exports
│       │   ├── handler.rs      # IPC dispatch
│       │   ├── messages.rs     # Re-export protocol types
│       │   └── error.rs        # Bridge errors
│       │
│       └── desktop/
│           ├── Cargo.toml
│           ├── build.rs
│           └── src/
│               ├── main.rs
│               ├── app.rs
│               ├── webview.rs
│               ├── assets.rs
│               └── js/
│                   └── ipc-primitives.js  # Minimal injected primitives
│
└── ui/
    ├── package.json
    ├── vite.config.ts
    ├── tsconfig.json
    ├── index.html
    └── src/
        ├── main.tsx
        ├── App.tsx
        ├── lib/
        │   └── vstkit-ipc/           # @vstkit/ipc library
        │       ├── index.ts          # Public exports
        │       ├── IpcBridge.ts      # Low-level bridge class (wraps primitives)
        │       ├── ParameterClient.ts # High-level typed client class
        │       ├── types.ts          # TypeScript types
        │       └── hooks.ts          # React hooks
        ├── components/
        │   ├── ParameterSlider.tsx
        │   ├── ParameterToggle.tsx
        │   └── LatencyMonitor.tsx
        └── styles/
            └── main.css
```

---

## Appendix C: References

- [wry documentation](https://github.com/nicebusiness/nice-wry)
- [tao (windowing)](https://github.com/nicebusiness/nice-tao)
- [include_dir crate](https://docs.rs/include_dir)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Vite documentation](https://vitejs.dev/)
- [React 18 documentation](https://react.dev/)
