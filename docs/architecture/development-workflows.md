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

- Explicit disconnected state with actionable diagnostics

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
```

`cargo xtask dev` now runs a **preflight refresh** before launching `wavecraft start`.
The preflight is fail-fast and logs explicit status lines for each rule (refreshed vs skipped):

- **UI package artifacts** (`ui/packages/core/dist`, `ui/packages/components/dist`)
  - Runs `npm run build:lib` in `ui/` when package source/config files are newer than dist outputs or outputs are missing.
  - Skips when artifacts are up-to-date.
- **Metadata/typegen startup caches** (`wavecraft-params.json`, `wavecraft-processors.json`, `sdk-template/ui/src/generated/parameters.ts`, `sdk-template/ui/src/generated/processors.ts`)
  - Invalidates stale metadata sidecars when SDK engine sources or relevant dev tooling files changed.
  - Preflight now tracks both parameter and processor generated artifacts in the cache/typegen scope.
  - Generated TypeScript artifacts are treated as regeneration targets for startup/hot-reload (codegen-first); stale metadata is corrected by regenerating from fresh extraction.

**SDK Mode Detection:**

When `cargo xtask dev` (or `wavecraft start`) is run from the SDK repository root, the CLI detects the `[workspace]` in `engine/Cargo.toml` and automatically enters "SDK mode." In this mode, the CLI redirects all engine/UI operations (build, parameter extraction, FFI loading, file watching, UI dev server) to the canonical scaffold at `sdk-template/engine` and `sdk-template/ui`.

Before first SDK-mode use, run:

```bash
./scripts/setup-dev-template.sh
```

This materializes `.template` manifests into concrete files, applies development defaults, rewrites Wavecraft git dependencies to local path dependencies, and installs `sdk-template/ui` dependencies.

**TypeScript Path Resolution in SDK Mode:**

In SDK mode, `wavecraft start` also injects `tsconfig.json` path overrides so that `@wavecraft/core` and `@wavecraft/components` resolve to the local monorepo source (`ui/packages/*/src/`) rather than published npm packages. This uses `baseUrl: "."` and `paths` entries, enabling SDK contributors to iterate on both the npm packages and generated types simultaneously without publishing.

**Dev server startup behavior (CLI `wavecraft start`):**

- Performs preflight checks to ensure the WebSocket and UI ports are free before starting any servers.
- Starts the UI dev server with strict port binding (no auto-switching). If the UI port is in use, startup fails fast with a clear error and no servers are left running.

### TypeScript Metadata Codegen (codegen-first v1)

When `wavecraft start` launches, it:

1. **Extracts plugin metadata via FFI** from the compiled Rust engine using `wavecraft_get_params_json` and `wavecraft_get_processors_json`
2. **Generates `ui/src/generated/parameters.ts`** — a module augmentation file that narrows `@wavecraft/core`'s `ParameterId` type to a literal union of the plugin's actual parameter IDs
3. **Generates `ui/src/generated/processors.ts`** — a generated registry/typing artifact used for processor presence (`WavecraftProcessorIdMap` augmentation + startup registration)
4. **Regenerates on hot-reload** — when Rust source files change, the rebuild pipeline re-extracts metadata and regenerates both TypeScript artifacts via the `TsTypesWriterFn` callback in `RebuildCallbacks`
5. **Enforces codegen-first scope** — both generated files are canonical startup/hot-reload outputs in SDK mode; stale metadata is corrected by regeneration, not by retaining stale sidecars
6. **`wavecraft bundle` refresh** — before the UI build and embedding step, `wavecraft bundle` also refreshes `ui/src/generated/parameters.ts` and `ui/src/generated/processors.ts` using fresh sidecars when available, or falling back to a discovery build+extraction if sidecars are missing or stale

> **Sidecar freshness:** Freshness checks consider newer files in `engine/src/`, plugin build artifacts, and a newer CLI binary. Any of these conditions marks the sidecars as stale and triggers re-extraction.

This enables IDE autocompletion and compile-time type safety for `useParameter('oscillator_enabled')` calls without any developer configuration.

The generated files are build artifacts (gitignored) and should not be checked into source control. In SDK mode, this codegen output is a required contract: if generation fails during startup or hot-reload, the update fails fast and prints actionable diagnostics (root cause + remediation) instead of silently continuing with stale types.

### Why Module-Level Detection?

The environment constant is evaluated at module scope (not inside hooks) to comply with React's Rules of Hooks. This ensures consistent hook call order across renders.

### Dev Audio via FFI

When running `wavecraft start`, the CLI uses a **two-phase approach** for plugin metadata loading:

1. **Cached Metadata Discovery** — First checks for cached `wavecraft-params.json` and `wavecraft-processors.json` sidecar files
2. **Feature-Gated Build** — If not cached, builds with `--features _param-discovery` which skips nih-plug's VST3/CLAP static initializers (preventing macOS `AudioComponentRegistrar` hangs during `dlopen`)
3. **FFI Extraction** — Extracts metadata from this safe dylib via `wavecraft_get_params_json` and `wavecraft_get_processors_json` and caches it for subsequent runs
4. **Contract Enforcement** — Current SDK metadata-discovery contract is required in SDK mode. Missing/incompatible discovery symbols or feature expectations fail fast with actionable diagnostics.

After parameter discovery completes, the CLI also attempts to load an FFI vtable symbol (`wavecraft_dev_create_processor`). This symbol is **not** gated by the `_param-discovery` feature and remains available regardless of which build was used. If found, audio processing runs **in-process** via cpal — no separate binary or subprocess is needed. Users never see or write any audio capture code.

#### FFI Audio Architecture (Full-Duplex)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  CLI Process (`wavecraft start`)                                             │
│                                                                              │
│  ┌──────────────────────────────────────────────────────────────────┐         │
│  │  Phase 1: Parameter Discovery (two-phase)                       │         │
│  │  1. Check for cached wavecraft-params.json                      │         │
│  │  2. If not cached: cargo build --features _param-discovery      │         │
│  │     (skips nih-plug static initializers, prevents dlopen hang)  │         │
│  │  3. dlopen(cdylib) → wavecraft_get_params_json() → cache JSON   │         │
│  │  4. Contract check: mismatch/missing symbols → fail fast         │         │
│  └──────────────────────────────────────────────────────────────────┘         │
│                                                                              │
│  ┌─────────────────────┐     ┌──────────────────────────────┐                │
│  │  Phase 2: Audio FFI  │     │  AtomicParameterBridge        │                │
│  │  dlopen(cdylib)      │     │  (Arc<AtomicF32> per param)   │                │
│  │  → vtable            │     │  WS writes ──► audio reads    │                │
│  │    (not gated by     │     └──────────────┬───────────────┘                │
│  │     _param-discovery)│                    │                                │
│  └──────────┬──────────┘                    │                                │
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

| Function          | Purpose                                                        |
| ----------------- | -------------------------------------------------------------- |
| `create`          | Heap-allocate a new processor instance (returns `*mut c_void`) |
| `process`         | Process deinterleaved audio buffers in-place                   |
| `set_sample_rate` | Update the processor's sample rate                             |
| `reset`           | Clear processor state (delay lines, filters, etc.)             |
| `drop`            | Free the processor instance                                    |

The vtable includes a `version` field (`DEV_PROCESSOR_VTABLE_VERSION`) so the CLI can detect incompatible changes and fail fast with actionable diagnostics instead of invoking undefined behavior.

#### Key Components

> **Note:** `wavecraft-dev-server` is a standalone crate at `dev-server/` (repository root), not under `engine/crates/`. It bridges CLI and engine concerns and is never distributed to end users.

- **`DevProcessorVTable`** (`wavecraft-protocol`): Versioned C-ABI vtable defining the FFI contract between user cdylib and CLI.
- **`wavecraft_dev_create_processor`** (`wavecraft-macros` generated): FFI symbol exported by `wavecraft_plugin!` that returns the vtable. Every `extern "C"` function is wrapped in `catch_unwind` for panic safety.
- **`PluginParamLoader`** (`wavecraft-bridge`): Loads the vtable alongside parameters via `try_load_processor_vtable()`. Missing/incompatible vtable in SDK mode is treated as a contract violation and fails fast with remediation guidance.
- **`DevAudioProcessor`** trait (`wavecraft-dev-server`): Simplified audio processing trait without associated types — compatible with both FFI and direct Rust usage.
- **`FfiProcessor`** (`wavecraft-dev-server`): Safe wrapper around the vtable's opaque `*mut c_void` instance. Implements `DevAudioProcessor` and calls through vtable function pointers. Uses stack-allocated `[*mut f32; 2]` for channel pointers (no heap allocation). `Drop` implementation calls the vtable's `drop` function.
- **`AudioServer`** (`wavecraft-dev-server`): Full-duplex audio I/O server. Opens paired cpal input + output streams connected by an `rtrb` SPSC ring buffer. Input callback: deinterleave → `FfiProcessor::process()` → compute meters → write to audio ring buffer. Output callback: read from ring buffer → speakers (silence on underflow). Meter data is delivered via a separate `rtrb` SPSC ring buffer to a tokio drain task for WebSocket broadcast. All buffers are pre-allocated before stream creation. Missing required devices/configuration in SDK mode fails fast with explicit diagnostics.
- **`AtomicParameterBridge`** (`wavecraft-dev-server`): Lock-free bridge for passing parameter values from the WebSocket thread to the audio thread. Contains a `HashMap<String, Arc<AtomicF32>>` built once at startup (immutable structure, mutable atomic values). WS thread writes via `store(Relaxed)`, audio thread reads via `load(Relaxed)`. Zero allocations, zero locks on the audio thread.

#### Memory and Lifetime Safety

All processor memory is allocated and freed **inside the dylib** via vtable functions (`create` → `Box::into_raw`, `drop` → `Box::from_raw`). The CLI never allocates or frees processor memory, avoiding cross-allocator issues.

**Drop ordering invariant:** The `FfiProcessor` (which holds vtable function pointers into the loaded library) must be dropped **before** the `PluginParamLoader` (which holds the `Library`). This is enforced by:

1. **Struct field order** in `PluginParamLoader`: `_library` is the last field, so it's dropped last.
2. **Local variable order** in the CLI: `_audio_handle` is declared after `loader`, so it's dropped first (reverse declaration order for locals).

#### Contract Policy (Pre-1.0)

Pre-1.0 SDK development prioritizes fast iteration and strict contracts over backward compatibility in dev mode. Plugins that do not export `wavecraft_dev_create_processor`, have incompatible vtable versions, or otherwise violate current SDK contracts fail fast with actionable diagnostics (what failed, expected contract, and how to fix).

If a temporary compatibility path is introduced for migration, it must be explicit and opt-in (for example, a dedicated CLI flag or environment variable) and must never be the default behavior.

#### Parameter Sync in Dev Mode

The `AtomicParameterBridge` enables lock-free parameter flow from the browser UI to the audio thread:

1. **UI** sends `setParameter("gain", 0.75)` via WebSocket
2. **WS thread** calls `DevServerHost::set_parameter()` which writes to both `InMemoryParameterHost` (for IPC queries) and `AtomicParameterBridge` (for audio thread)
3. **Audio thread** reads `bridge.read("gain")` via `AtomicF32::load(Relaxed)` at block boundaries

The FFI vtable v1 `process()` does not accept parameters directly. The bridge infrastructure is in place for future vtable v2 which will add a `set_parameter` function pointer. Currently, parameter values are available to the audio callback but not injected into the processor (documented known limitation of the `wavecraft_plugin!` macro).

### Audio Runtime Status Contract (Browser Dev)

Browser-dev audio startup in `wavecraft start` is deterministic and **independent** of parameter sidecar cache hit/miss paths. Audio runtime initialization is always attempted from the current runtime state after startup preconditions pass.

Pre-1.0 behavior is strict fail-fast: runtime loader failures, FFI vtable load/version failures, and audio initialization failures are treated as startup-fatal with actionable diagnostics.

The audio readiness contract is exposed over dedicated IPC APIs:

- Method: `getAudioStatus`
- Notification: `audioStatusChanged`

These status APIs are separate from transport health. `useConnectionStatus()` reports transport connectivity (WebSocket/native bridge), not audio readiness.

Current behavior: if no usable default output device is available, startup transitions to `failed` with diagnostic reason `noOutputDevice`. This remains startup-fatal until explicit device selection is implemented.

### Benefits

- **Real engine communication**: Browser UI talks to real Rust backend
- **Hot module reload**: Vite HMR for instant UI updates
- **Same IPC layer**: Identical JSON-RPC protocol as production
- **Robust reconnection**: Automatic recovery from connection drops; `useAllParameters` re-fetches parameters on reconnection
- **Explicit failure states**: UI shows clear connection/error states with actionable diagnostics
- **In-process audio**: Full-duplex audio I/O (input → process → output) via FFI with zero user boilerplate
- **Lock-free parameter sync**: UI parameter changes reach the audio thread via `AtomicParameterBridge` with zero allocations

---

## Build System & Tooling

Wavecraft uses a Rust-based build system (`xtask`) that provides a unified interface for building, testing, signing, and distributing plugins.

### Available Commands

| Command                                | Description                                                                                                                                      |
| -------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `cargo xtask ci-check`                 | **Pre-push validation** — 6-phase local CI simulation (docs, UI build, lint+typecheck, tests; add `--full` for template validation + CD dry-run) |
| `cargo xtask sync-ui-versions --check` | Verify UI dependency/version synchronization across workspaces (non-mutating CI/pre-push check)                                                  |
| `cargo xtask ci-check -F`              | **Full validation** — All 6 phases including template validation and CD dry-run                                                                  |
| `cargo xtask validate-template`        | Validate CLI template generation (replicates CI `template-validation.yml`)                                                                       |
| `cargo xtask dev`                      | Run preflight refresh (UI package artifacts + param/typegen cache invalidation) then start WebSocket + Vite dev servers                          |
| `cargo xtask bundle`                   | Build and bundle VST3/CLAP plugins (**internal/advanced SDK workflow**; generated projects should use `wavecraft bundle`)                        |
| `cargo xtask test`                     | Run all tests (Engine + UI)                                                                                                                      |
| `cargo xtask test --ui`                | Run UI tests only (Vitest)                                                                                                                       |
| `cargo xtask test --engine`            | Run Engine tests only (cargo test)                                                                                                               |
| `cargo xtask lint`                     | Run linters for UI and/or engine code                                                                                                            |
| `cargo xtask desktop`                  | Build and run the desktop POC                                                                                                                    |
| `cargo xtask au`                       | Build AU wrapper (macOS only)                                                                                                                    |
| `cargo xtask install`                  | Install plugins to system directories (**internal/advanced SDK workflow**; generated projects should use `wavecraft bundle --install`)           |
| `cargo xtask clean`                    | Clean all build artifacts across workspace (engine/target, cli/target, ui/dist, ui/coverage, target/tmp) with disk space reporting               |
| `cargo xtask all`                      | Run full build pipeline (test → bundle → au → install)                                                                                           |
| `cargo xtask sign`                     | Sign plugin bundles for macOS distribution                                                                                                       |
| `cargo xtask notarize`                 | Notarize plugin bundles with Apple                                                                                                               |
| `cargo xtask release`                  | Complete release workflow (build → sign → notarize)                                                                                              |

### Development Workflow

```bash
# Pre-push validation (recommended before every push)
cargo xtask ci-check            # 6-phase local CI: docs, UI build, lint+typecheck, tests (~1 min)
cargo xtask sync-ui-versions --check  # Verify UI version/dependency sync (required in CI validation)
cargo xtask ci-check --fix      # Auto-fix linting issues
cargo xtask ci-check -F         # Full: adds template validation + CD dry-run
cargo xtask ci-check --skip-docs    # Skip doc link checking
cargo xtask ci-check --skip-lint    # Skip linting phase
cargo xtask ci-check --skip-tests   # Skip test phase
cargo xtask ci-check -F --skip-template  # Full minus template validation
cargo xtask ci-check -F --skip-cd        # Full minus CD dry-run
```

```bash
# Browser-based UI development (recommended for UI work)
cargo xtask dev              # Starts WebSocket server + Vite
cargo xtask dev --verbose    # With detailed IPC logging

# Plugin project development (embedded server + FFI parameter/audio discovery)
wavecraft start

# Fast iteration (debug build, no signing) — SDK internal
cargo xtask bundle --debug

# Full build with React UI — SDK internal
cargo xtask bundle

# Build and install for DAW testing (canonical generated-project workflow)
# wavecraft bundle / wavecraft bundle --install are CLI-owned:
# - path dependency (SDK dev mode): stages UI locally, clean rebuilds engine
# - git/tag dependency (normal install): skips local staging, bundles directly
wavecraft bundle --install

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
- Validates code quality: linting (ESLint, Prettier, TypeScript type-check via `tsc --noEmit`, cargo fmt, clippy), documentation links
- Includes `cargo xtask sync-ui-versions --check` as part of CI validation to enforce UI workspace dependency/version synchronization
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

| Component  | Test ID Pattern                       | Example             |
| ---------- | ------------------------------------- | ------------------- |
| App Root   | `app-root`                            | Full page container |
| Meter      | `meter-{L\|R}[-{peak\|rms\|db}]`      | `meter-L-peak`      |
| Parameter  | `param-{id}[-{label\|slider\|value}]` | `param-gain-slider` |
| Version    | `version-badge`                       | Version display     |
| Connection | `connection-status`                   | WebSocket status    |

### Baseline Storage

Visual baselines are stored externally (not in git) at `~/.wavecraft/visual-baselines/`:

- Keeps repository lean
- Independent versioning from code
- Shareable across development machines

### Key Design Decisions

| Decision            | Choice                     | Rationale                       |
| ------------------- | -------------------------- | ------------------------------- |
| Automation tool     | Playwright MCP             | Agent-native, no custom scripts |
| Baseline location   | External (`~/.wavecraft/`) | Repository stays lean           |
| Test orchestration  | Agent-driven               | On-demand, not CI               |
| Component targeting | `data-testid` attributes   | Stable selectors                |

See [Visual Testing Guide](../guides/visual-testing.md) for complete setup and usage instructions.
