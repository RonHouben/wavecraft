# Low-Level Design: WebSocket IPC Bridge

## Overview

This document describes the technical design for adding WebSocket transport to VstKit's IPC system, enabling browser-based UI development with real engine communication.

---

## Architecture

### Current State

```
┌─────────────────────────────────────────────────────────────────┐
│  Plugin / Standalone App                                        │
│                                                                 │
│  ┌───────────────┐     ┌─────────────┐     ┌─────────────────┐ │
│  │  AppState /   │────▶│ IpcHandler  │────▶│ WKWebView       │ │
│  │  PluginParams │     │             │     │ (wry)           │ │
│  │  (ParameterHost)    │ handle_json()    │ postMessage()   │ │
│  └───────────────┘     └─────────────┘     └────────┬────────┘ │
│                                                      │          │
└──────────────────────────────────────────────────────│──────────┘
                                                       │
                                            ┌──────────▼──────────┐
                                            │   React UI          │
                                            │   IpcBridge         │
                                            │   (native only)     │
                                            └─────────────────────┘
```

### Target State

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  Standalone App                                                             │
│                                                                             │
│  ┌───────────────┐     ┌─────────────┐                                     │
│  │  AppState     │────▶│ IpcHandler  │◀──────────────────────────┐         │
│  │  (ParameterHost)    │             │                           │         │
│  └───────────────┘     │ handle_json()                          │         │
│                        └──────┬──────┘                           │         │
│                               │                                  │         │
│              ┌────────────────┼────────────────┐                 │         │
│              │                │                │                 │         │
│              ▼                ▼                ▼                 │         │
│  ┌───────────────────┐  ┌─────────────┐  ┌──────────────────┐   │         │
│  │ WKWebView         │  │ WebSocket   │  │ Meter Pusher     │   │         │
│  │ Transport         │  │ Transport   │  │ (60fps to WS)    │───┘         │
│  │ (existing)        │  │ (new)       │  │                  │             │
│  └─────────┬─────────┘  └──────┬──────┘  └──────────────────┘             │
│            │                   │                                           │
└────────────│───────────────────│───────────────────────────────────────────┘
             │                   │
             ▼                   ▼
   ┌─────────────────┐   ┌─────────────────┐
   │ React UI        │   │ React UI        │
   │ (WKWebView)     │   │ (Browser)       │
   │ NativeTransport │   │ WebSocketTransport
   └─────────────────┘   └─────────────────┘
```

---

## Component Design

### 1. Rust: WebSocket Server Module

**New file:** `engine/crates/standalone/src/ws_server.rs`

```rust
/// WebSocket server for browser-based UI development
pub struct WsServer {
    /// Port the server listens on
    port: u16,
    /// Shared IPC handler
    handler: Arc<IpcHandler<AppState>>,
    /// Active client connections (for meter broadcasting)
    clients: Arc<Mutex<Vec<WsClient>>>,
    /// Shutdown signal
    shutdown: broadcast::Sender<()>,
}

impl WsServer {
    /// Create a new WebSocket server
    pub fn new(port: u16, handler: Arc<IpcHandler<AppState>>) -> Self;
    
    /// Start the server (non-blocking, spawns tokio task)
    pub async fn start(&self) -> Result<(), WsError>;
    
    /// Shutdown the server gracefully
    pub fn shutdown(&self);
    
    /// Broadcast meter frame to all connected clients
    pub fn broadcast_meters(&self, frame: &MeterFrame);
}
```

**Dependencies to add to `standalone/Cargo.toml`:**
```toml
tokio = { version = "1", features = ["rt-multi-thread", "net", "sync", "macros"] }
tokio-tungstenite = "0.24"
futures-util = "0.3"
```

**Message handling:**

```rust
async fn handle_connection(
    handler: Arc<IpcHandler<AppState>>,
    ws_stream: WebSocketStream<TcpStream>,
) {
    let (mut write, mut read) = ws_stream.split();
    
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(json)) => {
                // Route through existing IpcHandler
                let response = handler.handle_json(&json);
                write.send(Message::Text(response)).await?;
            }
            Ok(Message::Close(_)) => break,
            _ => {} // Ignore binary, ping, pong
        }
    }
}
```

### 2. Rust: CLI Arguments

**Modified:** `engine/crates/standalone/src/main.rs`

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "standalone", about = "VstKit Standalone App")]
struct Args {
    /// Run in headless dev-server mode (WebSocket only, no UI window)
    #[arg(long)]
    dev_server: bool,
    
    /// WebSocket server port
    #[arg(long, default_value = "9000")]
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    let state = Arc::new(AppState::new());
    let handler = Arc::new(IpcHandler::new((*state).clone()));
    
    if args.dev_server {
        // Headless mode: WebSocket server only
        run_dev_server(handler, args.port)
    } else {
        // Normal mode: WebSocket + WKWebView
        run_app_with_websocket(state, handler, args.port)
    }
}
```

**Dependencies to add:**
```toml
clap = { version = "4", features = ["derive"] }
```

### 3. Rust: Meter Broadcasting

The current `get_meter_frame` is poll-based. For WebSocket, we need push-based updates.

**Modified:** `engine/crates/standalone/src/ws_server.rs`

```rust
/// Meter broadcaster that polls at 60fps and pushes to clients
struct MeterBroadcaster {
    host: Arc<dyn ParameterHost>,
    clients: Arc<Mutex<Vec<WsClient>>>,
}

impl MeterBroadcaster {
    fn start(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(16)); // ~60fps
            
            loop {
                interval.tick().await;
                
                if let Some(frame) = self.host.get_meter_frame() {
                    let notification = IpcNotification {
                        jsonrpc: "2.0".to_string(),
                        method: "meterFrame".to_string(),
                        params: Some(serde_json::to_value(&frame).unwrap()),
                    };
                    let json = serde_json::to_string(&notification).unwrap();
                    
                    // Broadcast to all clients
                    let clients = self.clients.lock().unwrap();
                    for client in clients.iter() {
                        let _ = client.send(Message::Text(json.clone()));
                    }
                }
            }
        });
    }
}
```

### 4. TypeScript: Transport Interface

**New file:** `ui/src/lib/vstkit-ipc/transports/Transport.ts`

```typescript
/**
 * Transport interface - abstracts communication layer
 */
export interface Transport {
  /** Send a JSON-RPC request and wait for response */
  send(request: string): Promise<string>;
  
  /** Subscribe to server-pushed notifications */
  onNotification(callback: (notification: string) => void): () => void;
  
  /** Check if transport is connected */
  isConnected(): boolean;
  
  /** Clean up resources */
  dispose(): void;
}
```

### 5. TypeScript: Native Transport

**New file:** `ui/src/lib/vstkit-ipc/transports/NativeTransport.ts`

```typescript
/**
 * Native transport - wraps globalThis.__VSTKIT_IPC__
 * Used when running inside WKWebView
 */
export class NativeTransport implements Transport {
  private primitives: typeof globalThis.__VSTKIT_IPC__;
  private notificationCallbacks = new Set<(msg: string) => void>();
  
  constructor() {
    if (!globalThis.__VSTKIT_IPC__) {
      throw new Error('Native IPC primitives not available');
    }
    this.primitives = globalThis.__VSTKIT_IPC__;
    
    // Wire up receive callback
    this.primitives.setReceiveCallback((message: string) => {
      // Notifications are handled separately from responses
      this.notificationCallbacks.forEach(cb => cb(message));
    });
  }
  
  async send(request: string): Promise<string> {
    return new Promise((resolve) => {
      // Store resolver for this request
      // Response comes via setReceiveCallback
      this.primitives.postMessage(request);
      // ... response handling via callback
    });
  }
  
  onNotification(callback: (notification: string) => void): () => void {
    this.notificationCallbacks.add(callback);
    return () => this.notificationCallbacks.delete(callback);
  }
  
  isConnected(): boolean {
    return true; // Native is always "connected"
  }
  
  dispose(): void {
    this.notificationCallbacks.clear();
  }
}
```

### 6. TypeScript: WebSocket Transport

**New file:** `ui/src/lib/vstkit-ipc/transports/WebSocketTransport.ts`

```typescript
/**
 * WebSocket transport - connects to Rust dev server
 * Used when running in browser with `npm run dev`
 */
export class WebSocketTransport implements Transport {
  private ws: WebSocket | null = null;
  private pendingRequests = new Map<number, {
    resolve: (response: string) => void;
    reject: (error: Error) => void;
  }>();
  private notificationCallbacks = new Set<(msg: string) => void>();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;
  
  constructor(
    private url: string = 'ws://localhost:9000',
    private onStatusChange?: (connected: boolean) => void
  ) {}
  
  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(this.url);
      
      this.ws.onopen = () => {
        console.log('[WebSocket] Connected to', this.url);
        this.reconnectAttempts = 0;
        this.onStatusChange?.(true);
        resolve();
      };
      
      this.ws.onclose = () => {
        console.log('[WebSocket] Disconnected');
        this.onStatusChange?.(false);
        this.attemptReconnect();
      };
      
      this.ws.onerror = (error) => {
        console.error('[WebSocket] Error:', error);
        reject(error);
      };
      
      this.ws.onmessage = (event) => {
        this.handleMessage(event.data);
      };
    });
  }
  
  async send(request: string): Promise<string> {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket not connected');
    }
    
    // Extract ID from request for response matching
    const parsed = JSON.parse(request);
    const id = parsed.id;
    
    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });
      this.ws!.send(request);
      
      // Timeout
      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error('Request timeout'));
        }
      }, 5000);
    });
  }
  
  private handleMessage(data: string): void {
    const parsed = JSON.parse(data);
    
    // Check if it's a response (has id) or notification (no id)
    if (parsed.id !== undefined) {
      const pending = this.pendingRequests.get(parsed.id);
      if (pending) {
        this.pendingRequests.delete(parsed.id);
        pending.resolve(data);
      }
    } else {
      // Notification
      this.notificationCallbacks.forEach(cb => cb(data));
    }
  }
  
  private attemptReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.log('[WebSocket] Max reconnect attempts reached');
      return;
    }
    
    this.reconnectAttempts++;
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);
    
    console.log(`[WebSocket] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);
    
    setTimeout(() => this.connect().catch(() => {}), delay);
  }
  
  onNotification(callback: (notification: string) => void): () => void {
    this.notificationCallbacks.add(callback);
    return () => this.notificationCallbacks.delete(callback);
  }
  
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }
  
  dispose(): void {
    this.ws?.close();
    this.pendingRequests.clear();
    this.notificationCallbacks.clear();
  }
}
```

### 7. TypeScript: Transport Factory

**New file:** `ui/src/lib/vstkit-ipc/transports/index.ts`

```typescript
import { Transport } from './Transport';
import { NativeTransport } from './NativeTransport';
import { WebSocketTransport } from './WebSocketTransport';
import { isWebViewEnvironment } from '../environment';

// Module-level detection (stable across renders)
const IS_WEBVIEW = isWebViewEnvironment();

let transportInstance: Transport | null = null;

/**
 * Get the appropriate transport for the current environment
 * - WKWebView: NativeTransport
 * - Browser: WebSocketTransport
 */
export async function getTransport(): Promise<Transport> {
  if (transportInstance) {
    return transportInstance;
  }
  
  if (IS_WEBVIEW) {
    console.log('[Transport] Using NativeTransport (WKWebView)');
    transportInstance = new NativeTransport();
  } else {
    console.log('[Transport] Using WebSocketTransport (Browser)');
    const wsTransport = new WebSocketTransport();
    await wsTransport.connect();
    transportInstance = wsTransport;
  }
  
  return transportInstance;
}

/**
 * Check if transport is available without creating it
 */
export function hasTransport(): boolean {
  return transportInstance !== null && transportInstance.isConnected();
}

export { Transport, NativeTransport, WebSocketTransport };
```

### 8. TypeScript: Refactored IpcBridge

**Modified:** `ui/src/lib/vstkit-ipc/IpcBridge.ts`

```typescript
import type { IpcRequest, IpcResponse, IpcNotification } from './types';
import { isIpcResponse, isIpcNotification } from './types';
import { getTransport, hasTransport, type Transport } from './transports';

export class IpcBridge {
  private static instance: IpcBridge | null = null;
  private nextId = 1;
  private transport: Transport | null = null;
  private initPromise: Promise<void> | null = null;
  private eventListeners = new Map<string, Set<(data: unknown) => void>>();

  private constructor() {}

  public static getInstance(): IpcBridge {
    IpcBridge.instance ??= new IpcBridge();
    return IpcBridge.instance;
  }

  /**
   * Initialize the bridge (lazy, returns cached promise)
   */
  private async initialize(): Promise<void> {
    if (this.transport) return;
    
    if (!this.initPromise) {
      this.initPromise = this.doInitialize();
    }
    
    return this.initPromise;
  }

  private async doInitialize(): Promise<void> {
    this.transport = await getTransport();
    
    // Subscribe to notifications
    this.transport.onNotification((message: string) => {
      this.handleNotification(message);
    });
  }

  /**
   * Invoke a method and wait for response
   */
  public async invoke<TResult>(
    method: string,
    params?: unknown,
    timeoutMs = 5000
  ): Promise<TResult> {
    await this.initialize();
    
    if (!this.transport) {
      throw new Error('Transport not available');
    }

    const id = this.nextId++;
    const request: IpcRequest = {
      jsonrpc: '2.0',
      id,
      method,
      params,
    };

    const responseJson = await this.transport.send(JSON.stringify(request));
    const response: IpcResponse = JSON.parse(responseJson);

    if (response.error) {
      throw new Error(`IPC Error ${response.error.code}: ${response.error.message}`);
    }

    return response.result as TResult;
  }

  /**
   * Subscribe to notification events (e.g., meterFrame)
   */
  public on<T>(event: string, callback: (data: T) => void): () => void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    
    const listeners = this.eventListeners.get(event)!;
    listeners.add(callback as (data: unknown) => void);
    
    return () => listeners.delete(callback as (data: unknown) => void);
  }

  private handleNotification(message: string): void {
    try {
      const parsed = JSON.parse(message);
      
      if (isIpcNotification(parsed)) {
        const listeners = this.eventListeners.get(parsed.method);
        listeners?.forEach(cb => cb(parsed.params));
      }
    } catch (error) {
      console.error('[IpcBridge] Failed to parse notification:', error);
    }
  }

  /**
   * Check connection status
   */
  public isConnected(): boolean {
    return this.transport?.isConnected() ?? false;
  }
}
```

### 9. TypeScript: Connection Status Hook

**New file:** `ui/src/lib/vstkit-ipc/useConnectionStatus.ts`

```typescript
import { useState, useEffect } from 'react';
import { IpcBridge } from './IpcBridge';

export interface ConnectionStatus {
  connected: boolean;
  transport: 'native' | 'websocket' | 'none';
}

export function useConnectionStatus(): ConnectionStatus {
  const [status, setStatus] = useState<ConnectionStatus>({
    connected: false,
    transport: 'none',
  });

  useEffect(() => {
    const bridge = IpcBridge.getInstance();
    
    // Initial check
    const checkConnection = (): void => {
      setStatus({
        connected: bridge.isConnected(),
        transport: getTransportType(),
      });
    };
    
    checkConnection();
    
    // Poll for status changes (simple approach)
    const interval = setInterval(checkConnection, 1000);
    
    return () => clearInterval(interval);
  }, []);

  return status;
}

function getTransportType(): 'native' | 'websocket' | 'none' {
  if (globalThis.__VSTKIT_IPC__) return 'native';
  // If we got here without native, must be WebSocket
  return IpcBridge.getInstance().isConnected() ? 'websocket' : 'none';
}
```

---

## File Structure

### New Files (Rust)

```
engine/crates/standalone/src/
├── main.rs              # Modified: CLI args, mode selection
├── app.rs               # Unchanged
├── assets.rs            # Unchanged  
├── webview.rs           # Minor: extract handler creation
├── ws_server.rs         # NEW: WebSocket server
└── js/
    └── ipc-primitives.js
```

### New Files (TypeScript)

```
ui/src/lib/vstkit-ipc/
├── transports/
│   ├── index.ts              # NEW: Factory + exports
│   ├── Transport.ts          # NEW: Interface
│   ├── NativeTransport.ts    # NEW: WKWebView wrapper
│   └── WebSocketTransport.ts # NEW: Browser WebSocket
├── IpcBridge.ts              # Modified: Use transport abstraction
├── useConnectionStatus.ts    # NEW: Connection status hook
├── environment.ts            # Unchanged
├── hooks.ts                  # Minor: remove mock data
├── types.ts                  # Unchanged
└── index.ts                  # Modified: export new items
```

---

## Protocol

### Message Format

No protocol changes required. WebSocket uses the exact same JSON-RPC 2.0 messages as the native bridge:

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "getParameter",
  "params": { "id": "gain" }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "id": "gain",
    "name": "Gain",
    "value": 0.7,
    "default": 0.7,
    "type": "float",
    "unit": "dB"
  }
}
```

**Notification (pushed from server):**
```json
{
  "jsonrpc": "2.0",
  "method": "meterFrame",
  "params": {
    "peak_l": 0.85,
    "peak_r": 0.82,
    "rms_l": 0.45,
    "rms_r": 0.43,
    "timestamp": 1706832000000
  }
}
```

---

## Development Workflow

### Two-Terminal Setup

**Terminal 1: Rust Engine**
```bash
# Headless mode (WebSocket only, no window)
cargo run -p standalone -- --dev-server

# Or with custom port
cargo run -p standalone -- --dev-server --port 9001
```

**Terminal 2: React UI**
```bash
cd ui
npm run dev
# Open http://localhost:5173
```

### Combined Mode (Future)

For integrated development (WebSocket + WKWebView):
```bash
cargo run -p standalone
# Opens WKWebView AND starts WebSocket server
# Browser can connect to ws://localhost:9000 while native UI runs
```

---

## Error Handling

### WebSocket Connection Failures

| Scenario | Behavior |
|----------|----------|
| Engine not running | UI shows "Connecting..." then "Engine disconnected" after max retries |
| Engine crashes | UI auto-reconnects with exponential backoff (1s, 2s, 4s, 8s, 16s) |
| Port in use | Engine logs error and exits with non-zero code |
| Invalid message | Engine logs warning, returns JSON-RPC error response |

### Graceful Degradation

If WebSocket fails to connect after max retries:
- UI shows persistent "Engine disconnected" indicator
- Hooks return `null`/`undefined` for data (no mock data)
- User can click to retry connection

---

## Testing Strategy

### Unit Tests

1. **WebSocketTransport**
   - Connection establishment
   - Request/response matching
   - Reconnection logic
   - Notification handling

2. **Transport Factory**
   - Correct transport selection based on environment
   - Singleton behavior

3. **Refactored IpcBridge**
   - Works with mock transport
   - Handles transport failures

### Integration Tests

1. **Rust WebSocket Server**
   - Accepts connections
   - Routes messages through IpcHandler
   - Handles concurrent clients
   - Clean shutdown

2. **End-to-End**
   - Browser connects to engine
   - Parameters sync correctly
   - Meters update in real-time

---

## Performance Considerations

### Meter Broadcasting

- **Target:** 60fps meter updates
- **Approach:** Server-side timer (16ms interval), push to all clients
- **Optimization:** If no clients connected, skip serialization

### WebSocket Overhead

| Metric | Target | Notes |
|--------|--------|-------|
| Connection latency | <100ms | Initial connect only |
| Message roundtrip | <5ms | Match native IPC performance |
| Meter update jitter | <5ms | Acceptable for visual display |

### Memory

- Single WebSocket connection per browser tab
- Pending request map bounded by timeout (5s)
- Notification callbacks cleaned up on unmount

---

## Security Considerations

### Localhost Only

The WebSocket server binds to `127.0.0.1` only:
```rust
let addr = "127.0.0.1:9000".parse()?;
```

This prevents external network access. The dev server is not intended for production use.

### No Authentication

For M6, no authentication is implemented:
- Single-user development tool
- Localhost-only binding provides basic isolation
- Future: Consider token-based auth if remote debugging needed

---

## Migration Path

### Phase 1: Add WebSocket Server (Rust)
- Add `ws_server.rs` module
- Add CLI arguments
- Implement `--dev-server` mode
- Verify with `websocat` or similar

### Phase 2: Add Transport Abstraction (TypeScript)
- Create `transports/` directory
- Implement `NativeTransport` (extract from IpcBridge)
- Implement `WebSocketTransport`
- Create factory with environment detection

### Phase 3: Refactor IpcBridge
- Switch to transport-based architecture
- Remove mock data
- Add connection status

### Phase 4: Add Meter Streaming
- Implement push-based meter notifications
- Update `useMeterFrame` hook to use notifications

### Phase 5: Polish
- Connection status indicator component
- Reconnection UX
- Documentation

---

## Risks and Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Tokio runtime conflicts with wry | Medium | High | Test early; may need separate thread for tokio |
| WebSocket perf insufficient for meters | Low | Medium | Can switch to binary frames if needed |
| Breaking existing native IPC | Low | High | Keep NativeTransport as thin wrapper |
| Reconnection loops on fast restarts | Medium | Low | Exponential backoff, max attempts |

---

## Dependencies

### New Rust Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "net", "sync", "macros"] }
tokio-tungstenite = "0.24"
futures-util = "0.3"
clap = { version = "4", features = ["derive"] }
```

### No New TypeScript Dependencies

WebSocket is built into browsers. No additional npm packages required.

---

## Success Criteria

1. ✅ `cargo run -p standalone -- --dev-server` starts WebSocket server
2. ✅ `npm run dev` connects to WebSocket automatically
3. ✅ Parameters read/write work over WebSocket
4. ✅ Meters update at ~60fps in browser
5. ✅ Reconnection works after engine restart
6. ✅ Native (WKWebView) mode still works unchanged
7. ✅ Mock data removed from production code

---

## Open Questions

1. **Should WebSocket be always-on in normal mode?**
   - Current design: Only in `--dev-server` mode
   - Alternative: Always start WebSocket, even with window
   - Recommendation: Start with dev-server only, add always-on later if needed

2. **Binary protocol for meters?**
   - JSON is ~100 bytes per frame, 60fps = 6KB/s
   - Binary could reduce to ~32 bytes per frame
   - Recommendation: Start with JSON, optimize only if needed

3. **Multiple simultaneous browser connections?**
   - Current design: Supports multiple clients
   - Question: Should we limit to 1 for simplicity?
   - Recommendation: Allow multiple (useful for dev tools, debugging)

