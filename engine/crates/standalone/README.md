# VstKit Desktop POC

**Status:** ✅ Complete

A standalone desktop application demonstrating WebView ↔ Rust IPC communication for the VstKit plugin framework.

## Features

- ✅ React 18 + TypeScript UI embedded in native window
- ✅ Bidirectional IPC with JSON-RPC 2.0 protocol
- ✅ Real-time parameter synchronization
- ✅ Atomic parameter storage (thread-safe)
- ✅ Static asset embedding (single binary)
- ✅ Cross-platform support (macOS, Windows, Linux)

## Architecture

```
┌─────────────────────────────────────┐
│   React UI (WebView)                │
│   - ParameterSlider components      │
│   - ParameterToggle components      │
│   - LatencyMonitor                  │
│   - @vstkit/ipc library             │
└──────────────┬──────────────────────┘
               │ JSON-RPC IPC
┌──────────────▼──────────────────────┐
│   Rust Backend                      │
│   - IpcHandler (bridge layer)       │
│   - AppState (atomic parameters)    │
│   - Asset embedding (include_dir!)  │
└─────────────────────────────────────┘
```

## Build & Run

### Prerequisites

- Rust 1.70+ (2024 edition)
- Node.js 18+ and npm
- macOS, Windows, or Linux

**Platform Support:**
- ✅ **macOS** - Fully tested and supported (WKWebView)
- ⚠️ **Windows** - Theoretically supported via WebView2, **not tested** (no Windows dev machine)
- ⚠️ **Linux** - Theoretically supported via WebKitGTK, **not tested** (no Linux dev machine)

If you encounter platform-specific issues on Windows or Linux, please file an issue with details.

### Build React UI

```bash
cd ui
npm install
npm run build
```

### Build Desktop App

```bash
cd engine
cargo build -p desktop --release
```

### Run

```bash
cargo run -p desktop --release
```

Or run the binary directly:

```bash
./target/release/desktop
```

### CLI Options

```bash
desktop --help              # Show help
desktop --list-assets       # List embedded UI assets
```

## Testing

### Unit Tests

```bash
cargo test -p protocol      # Protocol types
cargo test -p bridge        # IPC handler
cargo test -p desktop       # App state & assets
```

### Integration Tests

```bash
cargo test -p desktop --test integration_test
```

### Latency Benchmarks

```bash
cargo test -p desktop --test latency_bench -- --nocapture
```

**Results (on Apple Silicon):**
- p50: ~0.003 ms (handler only)
- p95: ~0.003 ms (handler only)
- p99: ~0.005 ms (handler only)
- JSON parsing + handler p95: ~0.013 ms

*Note: These are Rust-side only. WebView IPC adds ~1-5ms overhead depending on platform.*

## Project Structure

```
vstkit/
├── engine/
│   └── crates/
│       ├── protocol/       # Shared IPC types
│       ├── bridge/         # IPC handler
│       └── desktop/        # WebView app
│           ├── src/
│           │   ├── app.rs          # AppState
│           │   ├── assets.rs       # Asset embedding
│           │   ├── webview.rs      # WebView setup
│           │   └── js/
│           │       └── ipc-primitives.js
│           └── tests/
│               ├── integration_test.rs
│               └── latency_bench.rs
└── ui/
    ├── src/
    │   ├── lib/vstkit-ipc/     # IPC library
    │   │   ├── types.ts
    │   │   ├── IpcBridge.ts
    │   │   ├── ParameterClient.ts
    │   │   ├── hooks.ts
    │   │   └── index.ts
    │   ├── components/
    │   │   ├── ParameterSlider.tsx
    │   │   ├── ParameterToggle.tsx
    │   │   └── LatencyMonitor.tsx
    │   ├── App.tsx
    │   └── main.tsx
    ├── package.json
    ├── tsconfig.json
    └── vite.config.ts
```

## IPC Protocol

### Methods

| Method | Params | Returns | Description |
|--------|--------|---------|-------------|
| `getParameter` | `{id: string}` | `{id: string, value: number}` | Get parameter value |
| `setParameter` | `{id: string, value: number}` | `{}` | Set parameter value (normalized 0-1) |
| `getAllParameters` | - | `{parameters: ParameterInfo[]}` | Get all parameters |
| `ping` | - | `{pong: true}` | Test connectivity |

### Notifications

| Event | Params | Description |
|-------|--------|-------------|
| `parameterChanged` | `{id: string, value: number}` | Parameter changed (e.g., from host automation) |

### Error Codes

- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error
- `-32000`: Parameter not found
- `-32001`: Parameter out of range

## Parameters

The POC simulates three parameters:

| ID | Name | Type | Default | Unit |
|----|------|------|---------|------|
| `gain` | Gain | Float | 0.7 | dB |
| `bypass` | Bypass | Bool | 0.0 | - |
| `mix` | Mix | Float | 1.0 | % |

## Development

### Hot Reload (UI only)

```bash
cd ui
npm run dev
```

Then update `webview.rs` to load from `http://localhost:5173` instead of `vstkit://localhost/index.html`.

### Type Safety

TypeScript types in `ui/src/lib/vstkit-ipc/types.ts` **must** stay in sync with Rust types in `engine/crates/protocol/src/ipc.rs`.

## Next Steps

This POC validates the architecture. Next milestones:

1. **Milestone 3**: Integrate WebView into nih-plug plugin editor
2. **Milestone 4**: Add SPSC ring buffers for audio metering
3. **Milestone 5**: Build real DSP effects
