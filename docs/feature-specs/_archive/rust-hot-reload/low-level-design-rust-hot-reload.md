# Low-Level Design: Rust Hot-Reload for Dev Mode

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
- [Development Workflows](../../architecture/development-workflows.md) — Dev workflow documentation
- [Roadmap](../../roadmap.md) — Milestone 18.9

## 1. Overview

### Problem
When a developer runs `wavecraft start` and modifies Rust source files (e.g., adding a processor to the signal chain), changes are not reflected in the browser UI. The developer must manually stop and restart `wavecraft start`. React/TypeScript changes hot-reload via Vite HMR, but Rust changes require a full restart.

### Solution
Add a file watcher to the `wavecraft start` command that detects Rust source file changes, triggers an automatic rebuild, and reloads parameter metadata into the running WebSocket server — all without dropping the WebSocket connection. The browser UI is notified via a `parametersChanged` push notification and re-fetches parameters automatically.

### Key Design Decisions
1. **WebSocket server stays running** — no disconnection during hot-reload
2. **Parameters swapped in-place** via `RwLock` on the parameter store
3. **Push notification** (`parametersChanged`) triggers UI re-fetch
4. **Failed builds preserve old state** — developer fixes errors and saves again
5. **One build at a time** with at most one pending follow-up

## 2. Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      wavecraft start                            │
│                                                                 │
│  ┌─────────────┐   file events   ┌──────────────┐              │
│  │ FileWatcher  │ ──────────────► │ RebuildPipe- │              │
│  │ (notify +    │   (debounced)   │ line         │              │
│  │  debouncer)  │                 │              │              │
│  └─────────────┘                 │ cargo build  │              │
│        │                          │ --lib        │              │
│   watches:                        │ --features   │              │
│   engine/src/**/*.rs              │ _param-disc. │              │
│   engine/Cargo.toml               └──────┬───────┘              │
│                                          │                      │
│                              success     │     failure          │
│                         ┌────────────────┼────────────────┐     │
│                         ▼                │                ▼     │
│                 ┌───────────────┐        │    ┌──────────────┐  │
│                 │ ParameterHost │        │    │ Terminal      │  │
│                 │ .replace_     │        │    │ Error Output  │  │
│                 │  parameters() │        │    │ (structured)  │  │
│                 └───────┬───────┘        │    └──────────────┘  │
│                         │                │                      │
│                         ▼                │                      │
│                 ┌───────────────┐        │                      │
│                 │ WsServer      │        │                      │
│                 │ broadcast:    │        │                      │
│                 │ parameters-   │        │                      │
│                 │ Changed       │        │                      │
│                 └───────┬───────┘        │                      │
│                         │                │                      │
└─────────────────────────┼────────────────┼──────────────────────┘
                          │ WebSocket      │
                          ▼                │
                 ┌───────────────┐         │
                 │ Browser UI    │         │
                 │ (Vite HMR)   │         │
                 │               │         │
                 │ onMessage:    │         │
                 │ → re-fetch    │         │
                 │   parameters  │         │
                 └───────────────┘         │
```

### Data Flow: Successful Hot-Reload

```
1. Developer saves lib.rs
2. notify fires DebouncedEvent::Modify (after 500ms quiet period)
3. FileWatcher sends RebuildRequest to channel
4. RebuildPipeline checks BuildGuard (no build running → acquire)
5. Spawns: cargo build --lib --features _param-discovery --message-format=json
6. Parses JSON output for errors/warnings
7. On success:
   a. Loads new parameter metadata from built dylib (existing FFI pattern)
   b. Calls DevServerHost::replace_parameters(new_params)
   c. Broadcasts "parametersChanged" notification to all WebSocket clients
8. UI receives notification → useAllParameters re-fetches → UI updates
9. Build takes ~3-8s depending on change scope
```

### Data Flow: Failed Build

```
1. Developer saves lib.rs with syntax error
2. Watcher triggers rebuild (same as above)
3. cargo build fails with non-zero exit
4. RebuildPipeline:
   a. Parses JSON output for compiler errors
   b. Prints formatted errors to terminal (with colors)
   c. Does NOT update parameters (old state preserved)
   d. Optionally broadcasts "buildFailed" notification to UI
5. Developer sees errors in terminal, fixes code, saves again
6. Watcher triggers new rebuild cycle
```

## 3. File Watcher Design

### Crate Dependencies

```toml
# cli/Cargo.toml
[dependencies]
notify = "7"
notify-debouncer-full = "0.4"
```

### Watch Configuration

| Setting | Value | Rationale |
|---------|-------|-----------|
| Debounce timeout | 500ms | Handles multi-file saves (e.g., cargo fmt) |
| Watch paths | `engine/src/`, `engine/Cargo.toml` | Rust source and dependency changes |
| Ignore patterns | `**/target/`, `.*`, `*.swp`, `*.swo` | Build artifacts, hidden files, editor temp files |
| Recursive | Yes | Watch all subdirectories of `engine/src/` |
| Event types | Create, Modify, Remove | File renames trigger remove + create |

### FileWatcher Module

```rust
// cli/src/dev_server/watcher.rs (new file)

use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct FileWatcher {
    debouncer: Debouncer<notify::RecommendedWatcher>,
    _rx: mpsc::Receiver<WatchEvent>,
}

pub enum WatchEvent {
    RustFilesChanged(Vec<PathBuf>),
}

impl FileWatcher {
    pub fn new(
        engine_dir: &Path,
        tx: mpsc::Sender<WatchEvent>,
    ) -> Result<Self> {
        let debouncer = new_debouncer(
            Duration::from_millis(500),
            None, // no tick rate override
            move |events: Result<Vec<DebouncedEvent>, _>| {
                // Filter to .rs files and Cargo.toml
                // Send WatchEvent::RustFilesChanged
            },
        )?;
        
        debouncer.watch(engine_dir.join("src"), RecursiveMode::Recursive)?;
        debouncer.watch(engine_dir.join("Cargo.toml"), RecursiveMode::NonRecursive)?;
        
        Ok(Self { debouncer, _rx })
    }
}
```

## 4. Rebuild Pipeline

### Build Command

```rust
cargo build --lib --features _param-discovery --message-format=json
```

- `--lib`: Only rebuild the library (no binaries)
- `--features _param-discovery`: Enable parameter discovery FFI exports
- `--message-format=json`: Structured output for error parsing

### BuildGuard (Concurrency Control)

```rust
// cli/src/dev_server/rebuild.rs (new file)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct BuildGuard {
    building: AtomicBool,
    pending: AtomicBool,
}

impl BuildGuard {
    /// Try to start a build. Returns true if acquired.
    pub fn try_start(&self) -> bool {
        self.building
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }
    
    /// Mark a pending rebuild request (received during active build).
    pub fn mark_pending(&self) {
        self.pending.store(true, Ordering::SeqCst);
    }
    
    /// Complete current build. Returns true if a pending build should start.
    pub fn complete(&self) -> bool {
        self.building.store(false, Ordering::SeqCst);
        self.pending.swap(false, Ordering::SeqCst)
    }
}
```

### RebuildPipeline

```rust
// cli/src/dev_server/rebuild.rs

pub struct RebuildPipeline {
    guard: Arc<BuildGuard>,
    engine_dir: PathBuf,
    host: Arc<DevServerHost>,
    ws_server: Arc<WsServer>,
}

impl RebuildPipeline {
    pub async fn handle_change(&self) -> Result<()> {
        if !self.guard.try_start() {
            self.guard.mark_pending();
            println!("  Build already in progress, queuing rebuild...");
            return Ok(());
        }
        
        loop {
            let result = self.do_build().await;
            
            match result {
                Ok(params) => {
                    self.host.replace_parameters(params);
                    self.ws_server.broadcast_parameters_changed().await;
                    println!("  ✓ Hot-reload complete");
                }
                Err(e) => {
                    eprintln!("  ✗ Build failed:\n{}", e);
                    // Preserve old state, don't update parameters
                }
            }
            
            if !self.guard.complete() {
                break; // No pending rebuild
            }
            println!("  Pending changes detected, rebuilding...");
        }
        
        Ok(())
    }
    
    async fn do_build(&self) -> Result<Vec<ParameterInfo>> {
        println!("  🔄 Rebuilding plugin...");
        let start = std::time::Instant::now();
        
        let output = Command::new("cargo")
            .args(["build", "--lib", "--features", "_param-discovery", "--message-format=json"])
            .current_dir(&self.engine_dir)
            .output()
            .await?;
        
        let elapsed = start.elapsed();
        
        if !output.status.success() {
            // Parse JSON lines for compiler-error messages
            // Format and return error
            return Err(BuildError::CompileFailed(parse_errors(&output.stdout)));
        }
        
        println!("  Build succeeded in {:.1}s", elapsed.as_secs_f64());
        
        // Load parameters from rebuilt dylib (existing FFI pattern)
        let params = load_parameters_from_dylib(&self.engine_dir)?;
        Ok(params)
    }
}
```

## 5. Server Reload Strategy (In-Place Parameter Swap)

### Key Decision: No WebSocket Restart

The WebSocket server stays running throughout hot-reload. Only the parameter data is swapped.

### DevServerHost Changes

```rust
// cli/src/dev_server/host.rs — modifications

use std::sync::RwLock;

pub struct InMemoryParameterHost {
    // CHANGE: Vec → RwLock<Vec> to allow parameter replacement
    parameters: RwLock<Vec<StoredParameter>>,
    // ... other fields unchanged
}

impl InMemoryParameterHost {
    /// Replace all parameters with new metadata from a fresh build.
    /// Called by RebuildPipeline after successful compilation.
    pub fn replace_parameters(&self, new_params: Vec<ParameterInfo>) {
        let mut stored = new_params.into_iter().map(|p| StoredParameter {
            info: p,
            value: /* default or preserved if ID matches */,
        }).collect();
        
        let mut params = self.parameters.write().unwrap();
        *params = stored;
    }
}
```

### Parameter Value Preservation

When parameters are replaced, values for parameters with matching IDs are preserved:

```
Old state: [gain=0.5, mix=0.8]
New params: [gain, mix, oscillator_freq, oscillator_waveform]
Result:     [gain=0.5, mix=0.8, oscillator_freq=default, oscillator_waveform=default]
```

This provides a smooth experience — the developer doesn't lose their current knob positions when adding new parameters.

### WebSocket Notification

```rust
// engine/crates/wavecraft-dev-server/src/ws_server.rs — addition

impl WsServer {
    /// Broadcast a parametersChanged notification to all connected clients.
    pub async fn broadcast_parameters_changed(&self) {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "parametersChanged",
            "params": {}
        });
        self.broadcast(notification.to_string()).await;
    }
}
```

## 6. UI Integration

### New IPC Notification Handler

The UI needs to handle the `parametersChanged` notification. This requires a small addition to the IPC layer:

```typescript
// ui/packages/core/src/hooks/useAllParameters.ts — addition

// Listen for parametersChanged notification from server
useEffect(() => {
  const bridge = IpcBridge.getInstance();
  const unsubscribe = bridge.onNotification('parametersChanged', () => {
    logger.info('Parameters changed on server, re-fetching...');
    fetchParameters();
  });
  return unsubscribe;
}, []);
```

### IpcBridge Notification Support

If the IpcBridge doesn't already support server-push notifications, add:

```typescript
// ui/packages/core/src/ipc/IpcBridge.ts — addition

onNotification(method: string, callback: () => void): () => void {
    // Register callback for JSON-RPC notifications (no id field)
    // Return unsubscribe function
}
```

The transport layer already receives all messages — this just adds routing for notifications (messages without an `id` field, per JSON-RPC 2.0).

## 7. Audio Reload (audio-dev feature)

When the `audio-dev` feature is active, audio processing uses FFI into the plugin dylib. Hot-reload must also reload the audio processor.

### Reload Sequence

```
1. Stop audio thread (drain buffers)
2. Unload old dylib (drop FFI references)  
3. Load new dylib
4. Rebuild AtomicParameterBridge with new parameters
5. Restart audio thread
6. ~200-500ms audio gap (acceptable for dev mode)
```

### DevSession Struct

```rust
// cli/src/dev_server/session.rs (new file)

pub struct DevSession {
    host: Arc<DevServerHost>,
    ws_server: Arc<WsServer>,
    audio_processor: Option<FfiAudioProcessor>,  // Only when audio-dev active
    watcher: FileWatcher,
    pipeline: RebuildPipeline,
}
```

This struct manages the lifecycle and correct drop ordering of all dev mode components.

## 8. Integration into start.rs

### Current Flow (simplified)

```rust
// cli/src/commands/start.rs — current
fn run_dev_servers(...) {
    build_plugin();           // One-time build
    let host = create_host(); // Load params
    let server = start_ws();  // Start WebSocket
    start_vite();             // Start UI
    wait_for_shutdown();      // Block until Ctrl+C
}
```

### New Flow

```rust
// cli/src/commands/start.rs — with hot-reload
fn run_dev_servers(...) {
    build_plugin();                    // Initial build
    let host = create_host();          // Load params
    let server = start_ws();           // Start WebSocket
    
    // NEW: Start file watcher + rebuild pipeline
    let (tx, rx) = mpsc::channel(16);
    let watcher = FileWatcher::new(&engine_dir, tx)?;
    let pipeline = RebuildPipeline::new(engine_dir, host.clone(), server.clone());
    
    // Spawn rebuild handler on Tokio runtime
    runtime.spawn(async move {
        while let Some(event) = rx.recv().await {
            pipeline.handle_change().await;
        }
    });
    
    start_vite();                      // Start UI
    wait_for_shutdown();               // Block until Ctrl+C
}
```

### Terminal Output

```
🔧 wavecraft start v0.12.0
  Building plugin for parameter discovery...
  ✓ Found 4 parameters
  Starting WebSocket server on ws://localhost:9876
  👀 Watching engine/src/ for changes...
  Starting UI dev server...
  
  VITE v6.4.1  ready in 178 ms
  ➜ Local: http://localhost:5173/

  [14:32:15] File changed: engine/src/lib.rs
  🔄 Rebuilding plugin...
  ✓ Build succeeded in 3.2s — found 6 parameters (+2 new)
  ✓ Hot-reload complete

  [14:32:45] File changed: engine/src/lib.rs
  ✗ Build failed:
    error[E0433]: failed to resolve: use of undeclared type `Foo`
     --> src/lib.rs:35:14
```

## 9. Edge Cases

| Scenario | Handling |
|----------|----------|
| Rapid saves (< 500ms apart) | Debouncer coalesces into single event |
| Save during active build | BuildGuard queues one pending rebuild |
| Build failure | Preserve old parameters, print errors, wait for next save |
| Cargo.toml changes | Watched — triggers rebuild (dependency changes) |
| New .rs files created | Watched — `notify` recursive mode catches new files |
| File deleted | Watched — triggers rebuild (may fail if referenced) |
| Editor temp files (.swp) | Filtered out by extension check |
| Symlinked engine dir | `notify` follows symlinks by default |
| First build already running | Watcher starts after initial build completes |

## 10. Testing Strategy

| Test | Type | Description |
|------|------|-------------|
| Debounce coalescing | Unit | Multiple events within 500ms produce one rebuild |
| BuildGuard concurrency | Unit | Only one build runs at a time, pending flag works |
| Parameter preservation | Unit | Matching param IDs keep values after replace |
| JSON error parsing | Unit | Compiler errors extracted from `--message-format=json` |
| End-to-end hot-reload | Integration | Save file → params appear in UI (manual test) |
| Build failure recovery | Integration | Bad code → error shown → fix → hot-reload works |
| Audio reload | Integration | Audio stops briefly, resumes with new params |

## 11. Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| File change detection | < 1s | 500ms debounce + notify latency |
| Incremental Rust build | 3–8s | Depends on change scope |
| Parameter swap | < 10ms | In-memory operation |
| UI update after swap | < 100ms | WebSocket notification + React re-render |
| Total end-to-end | < 10s | From save to UI update |
| Audio gap (if active) | < 500ms | Dylib unload/reload + buffer drain |

## 12. New Files

| File | Purpose |
|------|---------|
| `cli/src/dev_server/watcher.rs` | FileWatcher with notify + debouncer |
| `cli/src/dev_server/rebuild.rs` | RebuildPipeline + BuildGuard |
| `cli/src/dev_server/session.rs` | DevSession lifecycle management |

## 13. Modified Files

| File | Changes |
|------|---------|
| `cli/Cargo.toml` | Add `notify`, `notify-debouncer-full` dependencies |
| `cli/src/commands/start.rs` | Integrate watcher + pipeline into dev server flow |
| `cli/src/dev_server/host.rs` | `parameters: Vec` → `RwLock<Vec>`, add `replace_parameters()` |
| `cli/src/dev_server/mod.rs` | Export new modules |
| `engine/crates/wavecraft-dev-server/src/ws_server.rs` | Add `broadcast_parameters_changed()` |
| `ui/packages/core/src/ipc/IpcBridge.ts` | Add `onNotification()` method |
| `ui/packages/core/src/hooks/useAllParameters.ts` | Listen for `parametersChanged` notification |

## 14. Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `notify` | 7.x | Cross-platform file system notifications |
| `notify-debouncer-full` | 0.4.x | Event debouncing for notify |

No new npm dependencies required — the UI changes use existing IpcBridge infrastructure.
