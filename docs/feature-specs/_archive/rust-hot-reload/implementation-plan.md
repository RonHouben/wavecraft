# Implementation Plan: Rust Hot-Reload for Dev Mode

## Related Documents

- [Low-Level Design](./low-level-design-rust-hot-reload.md) — Architecture and design decisions
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards — Rust](../../architecture/coding-standards-rust.md) — Rust conventions
- [Coding Standards — TypeScript](../../architecture/coding-standards-typescript.md) — TypeScript conventions
- [Roadmap](../../roadmap.md) — Milestone 18.9

## 1. Overview

This plan details the steps required to implement hot-reloading for Rust code changes during development with `wavecraft start`. The goal is to automatically rebuild the plugin's engine and update the development server's state without requiring a manual restart, providing a seamless experience similar to frontend hot-module replacement (HMR).

## 2. Requirements

- Detect file changes in the `engine/src` directory and `engine/Cargo.toml`.
- Automatically trigger a rebuild of the plugin library.
- On successful rebuild, update the parameter definitions in the running `DevServerHost`.
- Preserve parameter values for existing parameters across reloads.
- Notify the connected browser UI that the parameters have changed.
- The UI must automatically re-fetch the new parameter list and update the view.
- If a build fails, the server should report the error and continue running with the old plugin version.
- The system must handle rapid file changes and concurrent build requests gracefully.
- If the `audio-dev` feature is enabled, the audio processing thread must also be reloaded.

## 3. Implementation Phases

The implementation is broken down into three phases:
1. **Backend:** File watching, rebuild pipeline, and parameter swapping logic.
2. **Frontend:** Client-side logic to react to server notifications.
3. **Integration:** Tying everything together in the `wavecraft start` command.

---

### Phase 1: Backend Infrastructure (Rust)

**Estimated effort: 3–4 days**

#### Task 1.1: Add Dependencies

- **File:** `cli/Cargo.toml`
- **Action:** Add the `notify` and `notify-debouncer-full` crates.
  ```toml
  notify = "7"
  notify-debouncer-full = "0.4"
  ```
- **Acceptance:** `cargo check -p wavecraft` succeeds with new dependencies.

#### Task 1.2: Implement `BuildGuard` for Concurrency Control

- **File:** `cli/src/dev_server/rebuild.rs` (new file)
- **Action:** Create a `BuildGuard` struct with two `AtomicBool` fields (`building` and `pending`):
  - `try_start()` — Attempts to acquire the build lock. Returns `true` if acquired.
  - `mark_pending()` — Flags that a new build is requested while one is running.
  - `complete()` — Releases the lock and returns whether a pending build should start.
- **Acceptance:** Unit tests verify:
  - `try_start` returns `true` once, then `false` on subsequent calls until `complete()`.
  - `mark_pending` + `complete` correctly returns `true`.
  - `complete` without `mark_pending` returns `false`.

#### Task 1.3: Implement `FileWatcher`

- **File:** `cli/src/dev_server/watcher.rs` (new file)
- **Action:** Create the `FileWatcher` struct.
  - Use `notify_debouncer_full::new_debouncer` with a 500ms timeout.
  - The debouncer's callback sends `WatchEvent::RustFilesChanged` via `tokio::sync::mpsc::Sender`.
  - The `new()` function sets up watches on `engine/src/` (recursive) and `engine/Cargo.toml`.
  - Filter events to only `.rs` files and `Cargo.toml`. Ignore `target/`, hidden files, editor temp files (`.swp`, `.swo`).
- **Acceptance:** Saving a `.rs` file produces exactly one event after the debounce period. Saving a non-Rust file produces no event.

#### Task 1.4: Implement `RebuildPipeline`

- **File:** `cli/src/dev_server/rebuild.rs`
- **Action:** Create the `RebuildPipeline` struct with `Arc` references to `BuildGuard`, `DevServerHost`, and `WsServer`.
  - `handle_change(&self)` — Main entry point. Uses `BuildGuard` to manage concurrency. Loops if pending builds exist after completion.
  - `do_build(&self)` — Runs `cargo build --lib --features _param-discovery --message-format=json`. Parses JSON output for errors. On success, loads new parameters from the rebuilt dylib. On failure, parses and formats compiler errors.
  - On successful build: calls `host.replace_parameters(new_params)` then `ws_server.broadcast_parameters_changed()`.
  - On failed build: prints formatted errors to terminal, preserves old parameters.
- **Acceptance:**
  - Successful builds update parameters and send notification.
  - Failed builds print errors and leave old parameters intact.
  - Concurrent changes are handled (one active + one pending).

#### Task 1.5: Update `DevServerHost` for In-Place Parameter Swapping

- **File:** `cli/src/dev_server/host.rs`
- **Action:** Modify `InMemoryParameterHost`:
  - Change `parameters` field from `Vec<StoredParameter>` to `RwLock<Vec<StoredParameter>>`.
  - Update all existing reads of `parameters` to acquire a read lock.
  - Add `replace_parameters(&self, new_params: Vec<ParameterInfo>)` method:
    - Acquires write lock.
    - For each new parameter, if an old parameter with the same ID exists, preserve its value.
    - New parameters get their default value.
    - Parameters removed from the new set are dropped.
- **Acceptance:**
  - All existing parameter reads still work.
  - `replace_parameters` correctly preserves values for matching IDs.
  - New parameters get defaults; removed parameters disappear.

#### Task 1.6: Add `broadcast_parameters_changed()` to `WsServer`

- **File:** `engine/crates/wavecraft-dev-server/src/ws_server.rs`
- **Action:** Add a public async method `broadcast_parameters_changed(&self)`:
  - Constructs a JSON-RPC 2.0 notification: `{"jsonrpc": "2.0", "method": "parametersChanged", "params": {}}`
  - Calls the existing `broadcast` method to send to all connected clients.
- **Acceptance:** All connected WebSocket clients receive the notification when called.

#### Task 1.7: Create `DevSession` for Lifecycle Management

- **File:** `cli/src/dev_server/session.rs` (new file)
- **Action:** Create a `DevSession` struct that holds:
  - `host: Arc<DevServerHost>`
  - `ws_server: Arc<WsServer>`
  - `watcher: FileWatcher`
  - `pipeline: RebuildPipeline`
  - `audio_processor: Option<FfiAudioProcessor>` (only when `audio-dev` active)
- **Why:** Encapsulates entire dev server state with correct ownership and drop ordering.
- **Acceptance:** All components are properly owned and dropped in correct order on shutdown.

#### Task 1.8: Update `dev_server` Module Exports

- **File:** `cli/src/dev_server/mod.rs`
- **Action:** Add module declarations and re-exports:
  ```rust
  pub mod watcher;
  pub mod rebuild;
  pub mod session;
  ```
- **Acceptance:** All new types are accessible from `cli::dev_server::*`.

**Phase 1 Checkpoint:** At this point, each component should have passing unit tests. The `BuildGuard`, `FileWatcher`, and `RebuildPipeline` can be tested individually.

---

### Phase 2: Frontend Integration (TypeScript)

**Estimated effort: 0.5–1 day**

#### Task 2.1: Add Notification Handler to `IpcBridge`

- **File:** `ui/packages/core/src/ipc/IpcBridge.ts`
- **Action:** Check if the existing `IpcBridge` already supports server-push notification subscriptions. If not, add:
  ```typescript
  public onNotification(method: string, callback: () => void): () => void
  ```
  This should register a callback for JSON-RPC notifications (messages without an `id` field) matching the given method name. Return an unsubscribe function.
- **Acceptance:** A component can subscribe to `parametersChanged` notifications and receive callbacks when the server sends them.

#### Task 2.2: Update `useAllParameters` to Listen for `parametersChanged`

- **File:** `ui/packages/core/src/hooks/useAllParameters.ts`
- **Action:** Add a `useEffect` hook that subscribes to the `parametersChanged` notification:
  ```typescript
  useEffect(() => {
    const bridge = IpcBridge.getInstance();
    const unsubscribe = bridge.onNotification('parametersChanged', () => {
      logger.info('Parameters changed on server, re-fetching...');
      reload(); // Use the existing reload/fetchParameters function
    });
    return unsubscribe;
  }, [reload]);
  ```
- **Acceptance:** When the server sends a `parametersChanged` notification, the hook automatically re-fetches parameters and the UI updates to show the new parameter list.

**Phase 2 Checkpoint:** Frontend changes can be tested against a mock WebSocket server that sends `parametersChanged` notifications, or by manually sending the notification via browser dev tools.

---

### Phase 3: Integration and Command Flow

**Estimated effort: 1–2 days**

#### Task 3.1: Integrate Hot-Reload into `start` Command

- **File:** `cli/src/commands/start.rs`
- **Action:** Modify the `run_dev_servers` function:
  1. After the initial build and `WsServer` startup, create an `mpsc` channel.
  2. Initialize `FileWatcher` with the channel's sender, watching the project's `engine/` directory.
  3. Initialize `RebuildPipeline` with `Arc` references to host, server, and build guard.
  4. Spawn a Tokio task that receives events from the channel and calls `pipeline.handle_change().await`.
  5. Wrap all components in `DevSession`.
  6. Print watcher startup message: `👀 Watching engine/src/ for changes...`
- **Terminal output format:**
  ```
  [14:32:15] File changed: engine/src/lib.rs
  🔄 Rebuilding plugin...
  ✓ Build succeeded in 3.2s — found 6 parameters (+2 new)
  ✓ Hot-reload complete
  ```
  Or on failure:
  ```
  [14:32:45] File changed: engine/src/lib.rs
  ✗ Build failed:
    error[E0433]: failed to resolve: use of undeclared type `Foo`
     --> src/lib.rs:35:14
  ```
- **Acceptance:** The `wavecraft start` command watches for `.rs` file changes and triggers automatic rebuild + parameter reload.

#### Task 3.2: Implement Audio Reload Logic (audio-dev feature)

- **File:** `cli/src/dev_server/rebuild.rs` and `cli/src/commands/start.rs`
- **Action:** When the `audio-dev` feature is active and a rebuild succeeds:
  1. Stop the audio thread (drain buffers).
  2. Drop the old `PluginLoader` / FFI references.
  3. Load the new dylib.
  4. Rebuild `AtomicParameterBridge` with new parameters.
  5. Restart the audio thread.
  6. Expected audio gap: ~200–500ms.
- **Note:** This task should be feature-gated behind `#[cfg(feature = "audio-dev")]`.
- **Acceptance:** With `audio-dev` enabled, Rust changes cause a brief audio dropout followed by resumed processing with new code.

#### Task 3.3: Update Development Workflows Documentation

- **File:** `docs/architecture/development-workflows.md`
- **Action:** Update the dev workflow documentation to reflect that Rust hot-reload is now supported. Update any mentions of "UI only" hot reload.
- **Acceptance:** Documentation accurately describes both UI HMR and Rust hot-reload behavior.

**Phase 3 Checkpoint:** Full end-to-end testing (see Testing Strategy below).

---

## 4. File Summary

### New Files

| File | Purpose |
|------|---------|
| `cli/src/dev_server/watcher.rs` | `FileWatcher` with notify + debouncer |
| `cli/src/dev_server/rebuild.rs` | `RebuildPipeline` + `BuildGuard` |
| `cli/src/dev_server/session.rs` | `DevSession` lifecycle management |

### Modified Files

| File | Changes |
|------|---------|
| `cli/Cargo.toml` | Add `notify`, `notify-debouncer-full` dependencies |
| `cli/src/commands/start.rs` | Integrate watcher + pipeline into dev server flow |
| `cli/src/dev_server/host.rs` | `parameters: Vec` → `RwLock<Vec>`, add `replace_parameters()` |
| `cli/src/dev_server/mod.rs` | Export new modules |
| `engine/crates/wavecraft-dev-server/src/ws_server.rs` | Add `broadcast_parameters_changed()` |
| `ui/packages/core/src/ipc/IpcBridge.ts` | Add `onNotification()` method (if not present) |
| `ui/packages/core/src/hooks/useAllParameters.ts` | Listen for `parametersChanged` notification |
| `docs/architecture/development-workflows.md` | Update hot-reload documentation |

## 5. Testing Strategy

### Unit Tests

| Test | File | Scope |
|------|------|-------|
| `BuildGuard` concurrency | `cli/src/dev_server/rebuild.rs` | `try_start`, `mark_pending`, `complete` |
| Parameter value preservation | `cli/src/dev_server/host.rs` | `replace_parameters` preserves matching IDs |
| JSON error parsing | `cli/src/dev_server/rebuild.rs` | Parse `--message-format=json` output |

### Integration Tests (Manual)

| # | Scenario | Steps | Expected Result |
|---|----------|-------|-----------------|
| 1 | Happy path | Run `wavecraft start`. Add `Oscillator` to signal chain. Save. | New oscillator parameters appear in browser UI within ~10s |
| 2 | Value preservation | Set gain slider to 0.5. Add a new parameter in Rust. Save. | Gain slider stays at 0.5; new parameter appears with default |
| 3 | Build failure | Introduce a syntax error. Save. | Terminal shows formatted error; UI unchanged |
| 4 | Failure recovery | Fix the syntax error. Save. | Hot-reload succeeds; UI updates |
| 5 | Rapid saves | Save 3 times in 1 second. | Only 1–2 builds triggered (debounce + guard) |
| 6 | Audio reload | With `audio-dev`: modify DSP code. Save. | Brief audio dropout, then audio resumes with new processing |

## 6. Estimated Total Effort

| Phase | Effort |
|-------|--------|
| Phase 1: Backend | 3–4 days |
| Phase 2: Frontend | 0.5–1 day |
| Phase 3: Integration | 1–2 days |
| **Total** | **4.5–7 days** |

## 7. Implementation Order

Tasks should be implemented in this order to minimize blocked work:

1. Task 1.1 (dependencies)
2. Task 1.2 (BuildGuard — no external deps)
3. Task 1.5 (DevServerHost RwLock change — foundational)
4. Task 1.6 (WsServer broadcast — foundational)
5. Task 1.3 (FileWatcher)
6. Task 1.4 (RebuildPipeline — depends on 1.2, 1.5, 1.6)
7. Task 1.7 (DevSession)
8. Task 1.8 (module exports)
9. Task 2.1 (IpcBridge notification)
10. Task 2.2 (useAllParameters listener)
11. Task 3.1 (start command integration)
12. Task 3.2 (audio reload — can be deferred)
13. Task 3.3 (documentation update)
