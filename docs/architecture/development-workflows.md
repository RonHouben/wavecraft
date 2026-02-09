# Development Workflows

This document covers browser development mode, the build system, visual testing, and CI/CD pipelines for the Wavecraft project.

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview, component design, data flows
- [Coding Standards — Testing](./coding-standards-testing.md) — Testing, linting, logging conventions
- [Agent Development Flow](./agent-development-flow.md) — Agent testing workflow
- [Visual Testing Guide](../guides/visual-testing.md) — Playwright-based screenshot comparison
- [CI/CD Pipeline Guide](../guides/ci-pipeline.md) — Pipeline details and troubleshooting
- [macOS Signing Guide](../guides/macos-signing.md) — Code signing and notarization setup

---

## Browser Development Mode

Wavecraft supports running the UI in a standard browser for rapid development iteration, with **real engine communication** via WebSocket.

### Development Mode Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                       TRANSPORT ABSTRACTION                             │
└─────────────────────────────────────────────────────────────────────────┘

  ┌─────────────────────┐                ┌─────────────────────┐
  │  Browser Dev Mode   │                │  Plugin Mode        │
  │  (npm run dev +     │                │  (Plugin in DAW)    │
  │   cargo xtask dev)  │                │                     │
  └──────────┬──────────┘                └──────────┬──────────┘
             │                                      │
             ▼                                      ▼
  ┌─────────────────────┐                ┌─────────────────────┐
  │ WebSocketTransport  │                │ NativeTransport     │
  │                     │                │                     │
  │ • ws://127.0.0.1:9000               │ • postMessage IPC   │
  │ • Auto-reconnect    │                │ • WKWebView native  │
  │ • Real engine data  │                │ • Synchronous calls │
  └──────────┬──────────┘                └──────────┬──────────┘
             │                                      │
             ▼                                      ▼
  ┌─────────────────────┐                ┌─────────────────────┐
  │ Dev Server          │                │ Plugin Binary       │
  │ Server (Rust)       │                │ (Embedded WebView)  │
  │ IpcHandler          │                │ IpcHandler          │
  └─────────────────────┘                └─────────────────────┘
```

### Transport Factory Pattern

The IPC system uses a factory pattern to automatically select the appropriate transport:

```
┌────────────────────────────────────────────────────────────────┐
│                      getTransport()                            │
│                                                                │
│   isWebViewEnvironment() ─────┬───────────────┐                │
│                               │               │                │
│                          (true)          (false)               │
│                               │               │                │
│                               ▼               ▼                │
│                    ┌──────────────┐  ┌───────────────────┐     │
│                    │NativeTransport│  │WebSocketTransport │     │
│                    │              │  │                   │     │
│                    │ postMessage  │  │ ws://127.0.0.1:   │     │
│                    │ __WAVECRAFT_IPC__│  │ 9000              │     │
│                    └──────────────┘  └───────────────────┘     │
└────────────────────────────────────────────────────────────────┘
```

### How It Works

1. **Environment Detection** (`environment.ts`):
   ```typescript
   export function isWebViewEnvironment(): boolean {
     return globalThis.__WAVECRAFT_IPC__ !== undefined;
   }
   ```

2. **Transport Selection** (`transports/index.ts`):
   ```typescript
   // Module-level constant (evaluated once)
   const IS_WEBVIEW = isWebViewEnvironment();
   
   export function getTransport(): Transport {
     if (IS_WEBVIEW) {
       return new NativeTransport();
     } else {
       return new WebSocketTransport({ url: 'ws://127.0.0.1:9000' });
     }
   }
   ```

3. **IpcBridge Abstraction** (`IpcBridge.ts`):
   - Lazy initialization of transport
   - Same API regardless of transport type
   - Rate-limited disconnect warnings (1 per 5s)
   - Graceful degradation when disconnected

4. **WebSocket Features**:
   - Automatic reconnection with exponential backoff (1s, 2s, 4s, 8s, 16s)
   - Maximum 5 reconnection attempts
   - Request timeout (5s) with proper cleanup
   - Connection status monitoring via `useConnectionStatus()` hook

### Development Workflow

```bash
# Start both servers (recommended for SDK development)
cargo xtask dev

# Start embedded dev server from a plugin project (recommended for plugin authors)
wavecraft start

# Or manually:
# Terminal 1: Start WebSocket server
cargo run -p wavecraft-dev-server -- --dev-server --port 9000

# Terminal 2: Start Vite dev server
cd ui && npm run dev
```

**Dev server startup behavior (CLI `wavecraft start`):**
- Performs preflight checks to ensure the WebSocket and UI ports are free before starting any servers.
- Starts the UI dev server with strict port binding (no auto-switching). If the UI port is in use, startup fails fast with a clear error and no servers are left running.

### Why Module-Level Detection?

The environment constant is evaluated at module scope (not inside hooks) to comply with React's Rules of Hooks. This ensures consistent hook call order across renders.

### Dev Audio via FFI

When running `wavecraft start`, the CLI loads the user's compiled cdylib (already built for parameter discovery) and also attempts to load an FFI vtable symbol (`wavecraft_dev_create_processor`). If found, audio processing runs **in-process** via cpal — no separate binary or subprocess is needed. Users never see or write any audio capture code.

#### FFI Audio Architecture (Full-Duplex)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  CLI Process (`wavecraft start`)                                             │
│                                                                              │
│  ┌─────────────────────┐     ┌──────────────────────────────┐                │
│  │  dlopen(cdylib)      │     │  AtomicParameterBridge        │                │
│  │  → params (existing) │     │  (Arc<AtomicF32> per param)   │                │
│  │  → vtable (new)      │     │  WS writes ──► audio reads    │                │
│  └──────────┬──────────┘     └──────────────┬───────────────┘                │
│             │                               │                                │
│             ▼                               ▼                                │
│  ┌─────────────────────────────────────────────────────────────────┐          │
│  │  cpal Input Callback                                           │          │
│  │  OS Mic → deinterleave → FfiProcessor::process() → meters      │          │
│  │                              (wraps vtable)           │        │          │
│  │                                                       ▼        │          │
│  │                                              rtrb SPSC (meters)│          │
│  │  interleave processed audio → rtrb SPSC (audio) ──────┐        │          │
│  └───────────────────────────────────────────────────────┼────────┘          │
│                                                          │                   │
│                                                          ▼                   │
│  ┌─────────────────────────────────────────────────────────────────┐          │
│  │  cpal Output Callback                                          │          │
│  │  rtrb SPSC (audio) → speakers (silence on underflow)           │          │
│  └─────────────────────────────────────────────────────────────────┘          │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────┐          │
│  │  Tokio Meter Drain Task (16ms interval)                        │          │
│  │  rtrb SPSC (meters) → WebSocket broadcast → Browser UI         │          │
│  └─────────────────────────────────────────────────────────────────┘          │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

#### FFI VTable Contract

The `DevProcessorVTable` (`wavecraft-protocol`) is a `#[repr(C)]` struct with `extern "C"` function pointers:

| Function | Purpose |
|----------|---------|
| `create` | Heap-allocate a new processor instance (returns `*mut c_void`) |
| `process` | Process deinterleaved audio buffers in-place |
| `set_sample_rate` | Update the processor's sample rate |
| `reset` | Clear processor state (delay lines, filters, etc.) |
| `drop` | Free the processor instance |

The vtable includes a `version` field (`DEV_PROCESSOR_VTABLE_VERSION`) so the CLI can detect incompatible changes and fall back gracefully instead of invoking undefined behavior.

#### Key Components

- **`DevProcessorVTable`** (`wavecraft-protocol`): Versioned C-ABI vtable defining the FFI contract between user cdylib and CLI.
- **`wavecraft_dev_create_processor`** (`wavecraft-macros` generated): FFI symbol exported by `wavecraft_plugin!` that returns the vtable. Every `extern "C"` function is wrapped in `catch_unwind` for panic safety.
- **`PluginParamLoader`** (`wavecraft-bridge`): Extended to optionally load the vtable alongside parameters via `try_load_processor_vtable()`. Missing vtable → graceful fallback to metering-only mode.
- **`DevAudioProcessor`** trait (`wavecraft-dev-server`): Simplified audio processing trait without associated types — compatible with both FFI and direct Rust usage.
- **`FfiProcessor`** (`wavecraft-dev-server`): Safe wrapper around the vtable's opaque `*mut c_void` instance. Implements `DevAudioProcessor` and calls through vtable function pointers. Uses stack-allocated `[*mut f32; 2]` for channel pointers (no heap allocation). `Drop` implementation calls the vtable's `drop` function.
- **`AudioServer`** (`wavecraft-dev-server`): Full-duplex audio I/O server. Opens paired cpal input + output streams connected by an `rtrb` SPSC ring buffer. Input callback: deinterleave → `FfiProcessor::process()` → compute meters → write to audio ring buffer. Output callback: read from ring buffer → speakers (silence on underflow). Meter data is delivered via a separate `rtrb` SPSC ring buffer to a tokio drain task for WebSocket broadcast. All buffers are pre-allocated before stream creation. Gracefully falls back to input-only (metering) mode if no output device is available.
- **`AtomicParameterBridge`** (`wavecraft-dev-server`): Lock-free bridge for passing parameter values from the WebSocket thread to the audio thread. Contains a `HashMap<String, Arc<AtomicF32>>` built once at startup (immutable structure, mutable atomic values). WS thread writes via `store(Relaxed)`, audio thread reads via `load(Relaxed)`. Zero allocations, zero locks on the audio thread.

#### Memory and Lifetime Safety

All processor memory is allocated and freed **inside the dylib** via vtable functions (`create` → `Box::into_raw`, `drop` → `Box::from_raw`). The CLI never allocates or frees processor memory, avoiding cross-allocator issues.

**Drop ordering invariant:** The `FfiProcessor` (which holds vtable function pointers into the loaded library) must be dropped **before** the `PluginParamLoader` (which holds the `Library`). This is enforced by:
1. **Struct field order** in `PluginParamLoader`: `_library` is the last field, so it's dropped last.
2. **Local variable order** in the CLI: `_audio_handle` is declared after `loader`, so it's dropped first (reverse declaration order for locals).

#### Backward Compatibility

Plugins compiled with older SDK versions that don't export `wavecraft_dev_create_processor` continue to work — the CLI falls back to silent meters (zeros) with a clear informational message. No synthetic/animated meter data is generated; the honest representation is "no audio processing is happening." Version mismatches in the vtable also trigger a graceful fallback with upgrade guidance.

#### Parameter Sync in Dev Mode

The `AtomicParameterBridge` enables lock-free parameter flow from the browser UI to the audio thread:

1. **UI** sends `setParameter("gain", 0.75)` via WebSocket
2. **WS thread** calls `DevServerHost::set_parameter()` which writes to both `InMemoryParameterHost` (for IPC queries) and `AtomicParameterBridge` (for audio thread)
3. **Audio thread** reads `bridge.read("gain")` via `AtomicF32::load(Relaxed)` at block boundaries

The FFI vtable v1 `process()` does not accept parameters directly. The bridge infrastructure is in place for future vtable v2 which will add a `set_parameter` function pointer. Currently, parameter values are available to the audio callback but not injected into the processor (documented known limitation of the `wavecraft_plugin!` macro).

### Benefits

- **Real engine communication**: Browser UI talks to real Rust backend
- **Hot module reload**: Vite HMR for instant UI updates
- **Same IPC layer**: Identical JSON-RPC protocol as production
- **Robust reconnection**: Automatic recovery from connection drops; `useAllParameters` re-fetches parameters on reconnection
- **Graceful degradation**: UI shows "Connecting..." when disconnected
- **In-process audio**: Full-duplex audio I/O (input → process → output) via FFI with zero user boilerplate
- **Lock-free parameter sync**: UI parameter changes reach the audio thread via `AtomicParameterBridge` with zero allocations

---

## Build System & Tooling

Wavecraft uses a Rust-based build system (`xtask`) that provides a unified interface for building, testing, signing, and distributing plugins.


### Available Commands

| Command | Description |
|---------|-------------|
| `cargo xtask ci-check` | **Pre-push validation** — Run lint + tests locally (~52s, 26x faster than Docker CI) |
| `cargo xtask dev` | Start WebSocket + Vite dev servers for browser development |
| `cargo xtask bundle` | Build and bundle VST3/CLAP plugins |
| `cargo xtask test` | Run all tests (Engine + UI) |
| `cargo xtask test --ui` | Run UI tests only (Vitest) |
| `cargo xtask test --engine` | Run Engine tests only (cargo test) |
| `cargo xtask lint` | Run linters for UI and/or engine code |
| `cargo xtask desktop` | Build and run the desktop POC |
| `cargo xtask au` | Build AU wrapper (macOS only) |
| `cargo xtask install` | Install plugins to system directories |
| `cargo xtask clean` | Clean all build artifacts across workspace (engine/target, cli/target, ui/dist, ui/coverage, target/tmp) with disk space reporting |
| `cargo xtask all` | Run full build pipeline (test → bundle → au → install) |
| `cargo xtask sign` | Sign plugin bundles for macOS distribution |
| `cargo xtask notarize` | Notarize plugin bundles with Apple |
| `cargo xtask release` | Complete release workflow (build → sign → notarize) |

### Development Workflow

```bash
# Pre-push validation (recommended before every push)
cargo xtask ci-check            # Run lint + tests (~52s)
cargo xtask ci-check --fix      # Auto-fix linting issues

# Browser-based UI development (recommended for UI work)
cargo xtask dev              # Starts WebSocket server + Vite
cargo xtask dev --verbose    # With detailed IPC logging

# Plugin project development (embedded server + FFI parameter/audio discovery)
wavecraft start

# Fast iteration (debug build, no signing)
cargo xtask bundle --debug

# Full build with React UI
cargo xtask bundle

# Build and install for DAW testing
cargo xtask bundle && cargo xtask install

# Build with AU wrapper (macOS)
cargo xtask all
```

### Visual Testing Workflow

Visual testing is done separately from automated checks using the Playwright MCP skill:

```bash
# 1. Start dev servers
cargo xtask dev

# 2. Use Playwright MCP skill for browser-based testing
#    (handled by Tester agent via "playwright-mcp-ui-testing" skill)

# 3. Stop servers when done
pkill -f "cargo xtask dev"
```

### Release Workflow

```bash
# Full release build (requires Apple Developer certificate)
cargo xtask release

# Or step-by-step:
cargo xtask bundle --release
cargo xtask sign
cargo xtask notarize --full
```

### Code Signing (macOS)

Wavecraft plugins require code signing for distribution. The build system provides two signing modes:

**Ad-Hoc Signing** (local development):
```bash
cargo xtask sign --adhoc
```
- No Apple Developer account required
- Plugins work on your local machine only
- Cannot be notarized or distributed

**Developer ID Signing** (distribution):
```bash
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
cargo xtask sign
```
- Requires Apple Developer Program membership ($99/year)
- Enables notarization and distribution
- Required for plugins to load without Gatekeeper warnings

### Notarization (macOS)

Apple notarization is required for distributed plugins to load on macOS Catalina+.

**Two-Step Workflow** (CI/CD friendly):
```bash
cargo xtask notarize --submit   # Submit and get request ID
# ... wait 5-30 minutes ...
cargo xtask notarize --status   # Check progress
cargo xtask notarize --staple   # Attach ticket when approved
```

**Blocking Workflow** (local development):
```bash
cargo xtask notarize --full     # Submit, wait, and staple
```

### Entitlements

Wavecraft plugins require specific entitlements for the hardened runtime due to WKWebView's JavaScript JIT:

```xml
<!-- engine/signing/entitlements.plist -->
<key>com.apple.security.cs.allow-jit</key>
<true/>
<key>com.apple.security.cs.allow-unsigned-executable-memory</key>
<true/>
<key>com.apple.security.cs.disable-library-validation</key>
<true/>
```

See [macOS Signing Guide](../guides/macos-signing.md) for complete setup instructions.

### CI/CD Pipelines

Wavecraft uses GitHub Actions for continuous integration and release automation.

**CI** (`.github/workflows/ci.yml`):
- Triggers on PRs to `main` (not on merge/push — code already validated via PR)
- Manual trigger available via `workflow_dispatch`
- Validates code quality: linting (ESLint, Prettier, cargo fmt, clippy), documentation links
- Runs automated tests: UI (Vitest) and Engine (cargo test)
- Does NOT build plugin bundles — that's the Release workflow's responsibility

**Template Validation** (`.github/workflows/template-validation.yml`):
- Triggers on PRs to `main` (not on merge/push)
- Manual trigger available via `workflow_dispatch`
- Scaffolds test plugin with CLI and validates compilation

**Continuous Deploy** (`.github/workflows/continuous-deploy.yml`):
- Triggers on push to `main` (after PR merge)
- Detects changed packages via path filters and publishes to crates.io/npm
- **Auto-bump:** If local version ≤ published version, CI auto-bumps the patch version, commits as `github-actions[bot]`, and publishes
- **CLI cascade:** CLI re-publishes whenever _any_ SDK component changes (engine, npm-core, npm-components, or CLI itself), ensuring the git tag always reflects the latest SDK state
- **Infinite loop prevention:** `detect-changes` skips if commit author is `github-actions[bot]`
- See [CI/CD Pipeline Guide](../guides/ci-pipeline.md) for details

**Release Build** (`.github/workflows/release.yml`):
- Triggers on version tags (`v*`) or manual dispatch
- Imports Developer ID certificate from secrets
- Signs with hardened runtime and entitlements
- Submits for Apple notarization and staples tickets
- Uploads production-ready artifacts

**Required Secrets for Release:**
| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE_P12` | Base64-encoded .p12 certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Password for .p12 file |
| `APPLE_SIGNING_IDENTITY` | Full signing identity string |
| `APPLE_ID` | Apple ID email for notarization |
| `APPLE_TEAM_ID` | 10-character team identifier |
| `APPLE_APP_PASSWORD` | App-specific password for notarytool |

---

## Visual Testing

Wavecraft supports browser-based visual testing using Playwright MCP for agent-driven UI validation.

### Architecture

```
  Agent / Developer                 Playwright MCP              Browser
  ┌─────────────┐                  ┌─────────────┐           ┌─────────────┐
  │             │ "take screenshot"│             │   CDP     │             │
  │   Copilot   │─────────────────►│  Playwright │──────────►│  Chromium   │
  │             │                  │  MCP Server │           │  Wavecraft UI  │
  │             │◄─────────────────│             │◄──────────│             │
  └─────────────┘  screenshot.png  └─────────────┘           └──────┬──────┘
        │                                                          │ ws://
        ▼                                                          ▼
  ┌─────────────┐                                           ┌─────────────┐
  │  External   │                                           │    Rust     │
  │  Baselines  │                                           │   Engine    │
  │ ~/.wavecraft/  │                                           │ (xtask dev) │
  └─────────────┘                                           └─────────────┘
```

### Test ID Convention

All UI components have `data-testid` attributes for reliable Playwright selection:

| Component | Test ID Pattern | Example |
|-----------|-----------------|---------|
| App Root | `app-root` | Full page container |
| Meter | `meter-{L\|R}[-{peak\|rms\|db}]` | `meter-L-peak` |
| Parameter | `param-{id}[-{label\|slider\|value}]` | `param-gain-slider` |
| Version | `version-badge` | Version display |
| Connection | `connection-status` | WebSocket status |

### Baseline Storage

Visual baselines are stored externally (not in git) at `~/.wavecraft/visual-baselines/`:
- Keeps repository lean
- Independent versioning from code
- Shareable across development machines

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Automation tool | Playwright MCP | Agent-native, no custom scripts |
| Baseline location | External (`~/.wavecraft/`) | Repository stays lean |
| Test orchestration | Agent-driven | On-demand, not CI |
| Component targeting | `data-testid` attributes | Stable selectors |

See [Visual Testing Guide](../guides/visual-testing.md) for complete setup and usage instructions.
