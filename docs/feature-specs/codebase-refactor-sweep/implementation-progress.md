# Implementation Progress — Codebase Refactor Sweep (Milestone 19)

## Scope Tracking

### Current slice
- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/start.rs`
- Slice goal: Extract tsconfig path injection concern into dedicated module without behavior changes.

## Files Touched

- `cli/src/commands/start.rs`
  - Added submodule declaration and routed tsconfig-path setup call through extracted module.
  - Removed inlined tsconfig path injection implementation from this file.
- `cli/src/commands/start/tsconfig_paths.rs`
  - New module containing tsconfig JSONC path injection helpers and `ensure_sdk_ui_paths_for_typescript`.
- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Created progress ledger for Milestone 19 slices.

## Invariant Checks

### Behavior-preserving constraints
- [x] No feature expansion introduced.
- [x] `StartCommand::execute` flow preserved; only helper location changed.
- [x] CLI diagnostics/messages for tsconfig path injection preserved.

### Tier 1 decomposition constraints
- [x] First responsibility cluster extracted (`tsconfig_paths`).
- [x] Decomposition remains within existing crate boundaries.
- [x] No roadmap/backlog/archive files edited.

### RT-safety constraints
- [x] No audio callback code touched in this slice.

## CI/Validation Checkpoints

- Baseline gate (Phase 0): `cargo xtask ci-check` — **PASSED**
  - Documentation: passed
  - Lint/type-check: passed
  - Engine + UI tests: passed

- Post-slice checkpoint:
  - `cargo xtask ci-check` — **PASSED**
    - Documentation: passed
    - Lint/type-check: passed
    - Engine + UI tests: passed

- CLI-focused validation (touched `cli/`):
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**

- Final checkpoint (after fixing escaped marker literal):
  - `cargo xtask ci-check` — **PASSED**

## Architect Escalations

- None in this slice.
  - Rationale: extraction boundary (`tsconfig_paths`) is explicitly prescribed by the low-level design and implementation plan.

---

## Slice 2 — Metadata cache decomposition (`start.rs`)

### Scope Tracking
- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/start.rs`
- Slice goal: Extract metadata sidecar/cache responsibilities into dedicated module with zero behavior change.

### Files Touched
- `cli/src/commands/start.rs`
  - Added `mod metadata_cache;` and routed metadata calls through the extracted module.
  - Replaced in-file metadata/cache usage with module-qualified calls:
    - metadata load at startup
    - sidecar write callback for rebuild pipeline
    - sidecar-focused test hooks
  - Removed inlined metadata cache implementation block (path helpers, staleness checks, sidecar read/write, metadata discovery build/load orchestration).

- `cli/src/commands/start/metadata_cache.rs`
  - New module containing metadata cache concerns extracted from `start.rs`:
    - sidecar path helpers (`params` + `processors`)
    - sidecar cache read/write helpers
    - staleness guards (dylib/source/CLI binary mtimes)
    - metadata loading orchestration (cached load fallback to `_param-discovery` build + extraction)

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 2 checkpoint details.

### Invariant Checks (Behavior Preservation)
- [x] Cache staleness rules preserved exactly:
  - Dylib newer than sidecar invalidates cache.
  - Newest file under `engine/src` newer than sidecar invalidates cache.
  - Current CLI binary newer than sidecar invalidates cache.
- [x] Sidecar semantics preserved:
  - same filenames (`wavecraft-params.json`, `wavecraft-processors.json`)
  - same read/write JSON shape and warning behavior on write failures.
- [x] Metadata discovery behavior preserved:
  - same `_param-discovery` build path and diagnostics.
  - same `audio-dev` / subprocess branching for metadata extraction.
- [x] Existing test intent preserved (frequency range assertion still exercises cached sidecar path).
- [x] Refactor-only scope respected (no feature or contract changes).

### CI/Validation Checkpoints (Slice 2)
- CLI-focused validation (requested cadence):
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED** (105 unit tests + integration suites green)

- Full repository gate:
  - `cargo xtask ci-check` — **PASSED**
    - Documentation: passed
    - Lint/type-check: passed
    - Engine + UI tests: passed

### Escalation Log (Mandatory)
- **No Architect escalation required for Slice 2.**
  - Decomposition target and boundary were explicit in task direction (metadata cache concern extraction only).

---

## Slice 3 — Hot-reload dylib extractor decomposition (`start.rs`)

### Scope Tracking
- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/start.rs`
- Slice goal: Extract hot-reload dylib extraction helpers into a dedicated module with zero behavior changes.

### Files Touched
- `cli/src/commands/start.rs`
  - Added `mod reload_extractors;`.
  - Rewired rebuild callback loaders to call:
    - `reload_extractors::load_parameters_from_dylib`
    - `reload_extractors::load_processors_from_dylib`
  - Removed in-file helper implementations moved to the dedicated module.

- `cli/src/commands/start/reload_extractors.rs`
  - New module containing extracted helpers:
    - `load_parameters_from_dylib`
    - `load_processors_from_dylib`
    - `create_temp_dylib_copy`

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 3 progress details.

### Invariant Checks (Behavior Preservation)
- [x] Same hot-reload extraction contract preserved (subprocess-based metadata extraction).
- [x] Same diagnostics/log lines preserved for dylib discovery, temp copy, and extraction results.
- [x] Same temporary dylib copy semantics preserved (timestamped temp filename + cleanup behavior).
- [x] Refactor-only scope preserved (no feature changes, no contract changes).

### CI/Validation Checkpoints (Slice 3)
- CLI-focused validation (requested cadence):
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**

- Full repository gate:
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)
- **No Architect escalation required for Slice 3.**
  - Extraction target, destination module, and moved function set were explicitly defined in slice requirements, and no ambiguous design decisions were encountered.
  - No ambiguity in ownership split was encountered during implementation.

---

## Slice 4 — Audio runtime startup/status orchestration decomposition (`start.rs`)

### Scope Tracking
- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/start.rs`
- Slice goal: Extract audio runtime startup/status orchestration (including classification helpers and startup flow) into a dedicated module with zero behavior changes.

### Files Touched
- `cli/src/commands/start.rs`
  - Added `mod audio_runtime;`.
  - Rewired audio runtime startup in `run_dev_servers()` to call extracted orchestration entrypoint:
    - `audio_runtime::start_audio_runtime(...)`
  - Removed in-file audio runtime helper block and startup/status orchestration block now owned by dedicated module.
  - Updated audio-runtime tests to import helpers from extracted module.

- `cli/src/commands/start/audio_runtime.rs`
  - New module containing extracted audio runtime concerns:
    - startup/status orchestration entrypoint (`start_audio_runtime`)
    - runtime loader and startup flow helpers
    - diagnostic classification helpers (`classify_audio_init_error`, `classify_runtime_loader_error`)
    - running-status helper (`status_for_running_audio`)

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 4 progress details.

### Invariant Checks (Behavior Preservation)
- [x] Status transition flow preserved: `Initializing` → `RunningFullDuplex` on success, `Failed` with diagnostics on failures.
- [x] Diagnostics classification and hint mapping preserved.
- [x] Strict/degraded mode behavior preserved, including `WAVECRAFT_ALLOW_NO_AUDIO=1` bypass semantics.
- [x] Broadcast behavior/messages preserved for init, failure, and running status updates.
- [x] User-facing startup/failure log lines preserved.
- [x] Refactor-only scope preserved (no feature or contract changes).

### CI/Validation Checkpoints (Slice 4)
- CLI-focused validation (requested cadence):
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED** (105 unit tests + integration suites green)

- Full repository gate:
  - `cargo xtask ci-check` — **PASSED**
    - Documentation: passed
    - Lint/type-check: passed
    - Engine + UI tests: passed

### Escalation Log (Mandatory)
- **No Architect escalation required for Slice 4.**
  - Extraction target, destination module, and behavior-preserving boundaries were explicit in slice requirements.
  - No ambiguous ownership or behavioral decision point was encountered during implementation.

---

## Slice 5 — Shutdown/lifecycle decomposition (`start.rs`)

### Scope Tracking
- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/start.rs`
- Slice goal: Extract shutdown/lifecycle helpers into a dedicated module with zero behavior changes.

### Files Touched
- `cli/src/commands/start.rs`
  - Added `mod shutdown;`.
  - Rewired shutdown wait flow to call `shutdown::wait_for_shutdown(...)`.
  - Rewired return-status mapping to use `shutdown::ShutdownReason`.
  - Removed in-file shutdown/lifecycle helper block.

- `cli/src/commands/start/shutdown.rs`
  - New module containing extracted shutdown/lifecycle concerns:
    - `ShutdownReason`
    - `wait_for_shutdown(...)`
    - `send_shutdown_signal(...)`
    - `kill_process(...)`

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 5 progress details.

### Invariant Checks (Behavior Preservation)
- [x] Ctrl+C handling preserved (`ctrlc::set_handler` semantics unchanged).
- [x] Shutdown signaling preserved (`watch::Sender<bool>` broadcast semantics unchanged).
- [x] UI process cleanup preserved (`kill()` + `wait()` on shutdown/disconnect paths).
- [x] Return-status semantics preserved (`CtrlC`/`ChannelClosed` => success; UI exit => error).
- [x] User-facing shutdown logs preserved.
- [x] Refactor-only scope preserved (no feature or contract changes).

### CI/Validation Checkpoints (Slice 5)
- CLI-focused validation (requested cadence):
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**

- Full repository gate:
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)
- **No Architect escalation required for Slice 5.**
  - Decomposition boundary and extracted symbol set were explicitly specified, and no ambiguous ownership split or behavior tradeoff was encountered.

---

## Slice 6 — Startup preflight/orchestration decomposition (`start.rs`)

### Scope Tracking
- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/start.rs`
- Slice goal: Extract startup preflight/orchestration helpers (dependency checks/install prompt flow + port/startup preconditions) into a dedicated module with zero behavior changes.

### Files Touched
- `cli/src/commands/start.rs`
  - Added `mod preflight;`.
  - Rewired dependency preflight in `StartCommand::execute()` to `preflight::ensure_dependencies(...)`.
  - Rewired startup precondition port checks in `run_dev_servers()` to `preflight::ensure_ports_available(...)`.
  - Removed in-file helpers moved to dedicated preflight module:
    - dependency prompt/install flow helpers
    - port availability helper

- `cli/src/commands/start/preflight.rs`
  - New module containing extracted startup preflight/orchestration helpers:
    - `ensure_dependencies(...)`
    - `ensure_ports_available(...)`
    - `prompt_install(...)`
    - `install_dependencies(...)`
    - `ensure_port_available(...)`

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 6 progress details.

### Invariant Checks (Behavior Preservation)
- [x] Dependency gating semantics preserved (`--install`, `--no-install`, prompt accept/decline paths).
- [x] Prompt text, diagnostics, and dependency install failure messaging preserved.
- [x] Port precondition checks preserved (same bind strategy, same error text/flags).
- [x] Startup order preserved (`detect project` → `dependency preflight` → `tsconfig path ensure` → `server startup preconditions`).
- [x] Refactor-only scope preserved (no feature or contract changes).

### CI/Validation Checkpoints (Slice 6)
- CLI-focused validation (requested cadence):
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**

- Full repository gate:
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)
- **No Architect escalation required for Slice 6.**
  - Extraction boundaries and candidate scope were explicitly provided by slice instructions.
  - No ambiguous ownership split or behavior tradeoff was encountered during implementation.

---

## Slice 7 — Dev-server startup pipeline decomposition (`start.rs`)

### Scope Tracking
- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/start.rs`
- Slice goal: Extract dev-server bootstrap/startup orchestration concerns into a focused module while preserving startup order, diagnostics, and failure semantics.

### Files Touched
- `cli/src/commands/start.rs`
  - Added `mod startup_pipeline;`.
  - Rewired startup execution to `startup_pipeline::run_dev_servers(...)`.
  - Removed in-file startup orchestration helpers moved to dedicated module.
  - Updated test import for env parser helper to use extracted module path.

- `cli/src/commands/start/startup_pipeline.rs`
  - New focused module containing extracted startup orchestration concerns:
    - startup banner + step sequencing (`run_dev_servers`)
    - hot-reload rebuild callback wiring (`build_rebuild_callbacks`)
    - UI dev-server bring-up scaffolding (`start_ui_dev_server`)
    - no-audio fallback env parsing helper (`parse_allow_no_audio_env`)

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 7 progress details.

### Invariant Checks (Behavior Preservation)
- [x] Startup sequencing preserved exactly:
  - port preflight → metadata load/codegen → WebSocket start → hot-reload session init → audio runtime startup → UI server startup → success banner/shutdown wait.
- [x] Startup diagnostics/log text preserved for banner, step outputs, success output, and early UI failure path.
- [x] Failure semantics preserved (including UI early-exit, audio strict/degraded behavior, and shutdown reason mapping).
- [x] Refactor-only scope preserved (no feature or contract changes).

### CI/Validation Checkpoints (Slice 7)
- CLI-focused validation (requested cadence):
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**

- Full repository gate:
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)
- **No Architect escalation required for Slice 7.**
  - Decomposition boundary was clear and explicit (startup orchestration cluster extraction), and no ambiguous ownership split or design decision point was encountered.

---

## Slice 8 — Start command test/module co-location finalization (`start.rs`)

### Scope Tracking
- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/start.rs`
- Slice goal: Finalize decomposition by co-locating remaining `start` command tests into module ownership boundaries and leaving `start.rs` as command wiring + minimal orchestration.

### Files Touched
- `cli/src/commands/start.rs`
  - Removed centralized `#[cfg(test)]` test bucket.
  - Retained only command-level wiring/orchestration symbols (`StartCommand`, module declarations, execute flow).

- `cli/src/commands/start/tsconfig_paths.rs`
  - Added module-local tests for JSONC path injection behavior and diagnostics:
    - injection when missing
    - idempotency when mappings already exist
    - fallback insertion path
    - warning for missing `compilerOptions`
    - trailing-comma correctness before following properties

- `cli/src/commands/start/startup_pipeline.rs`
  - Added module-local tests for `parse_allow_no_audio_env` opt-in parsing semantics.

- `cli/src/commands/start/audio_runtime.rs`
  - Added module-local (`audio-dev`) tests for:
    - audio init error classification
    - runtime loader error classification
    - running-audio status mapping

- `cli/src/commands/start/metadata_cache.rs`
  - Added module-local regression test for cached sidecar path preserving full frequency range values in browser-dev mode.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 8 completion record.

### Invariant Checks (Behavior Preservation)
- [x] No feature expansion introduced.
- [x] No runtime/log/flow semantics changed.
- [x] Test logic preserved exactly; only ownership location changed.
- [x] `start.rs` reduced to command wiring + minimal orchestration symbols.

### CI/Validation Checkpoints (Slice 8)
- Requested CLI validation cadence:
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
    - Unit tests: **105 passed, 0 failed**
    - Integration tests: **18 passed, 0 failed** (`bundle_command`, `update_command`, `version_flag`)

- Requested full repository gate:
  - `cargo xtask ci-check` — **PASSED**
    - Documentation: passed
    - Lint/type-check: passed
    - Engine + UI tests: passed

### Escalation Log (Mandatory)
- **No Architect escalation required for Slice 8.**
  - Module ownership boundaries were unambiguous: each test was moved to the module that owns the exercised symbol/behavior.
  - No cross-module boundary ambiguity or architectural tradeoff emerged while implementing this slice.

### Hotspot Completion Status
- `cli/src/commands/start.rs` Tier 1 hotspot status: **COMPLETE** (Slices 1–8 complete and validation green).
