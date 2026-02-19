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

---

## Slice 2 — Content + IPC script extraction (editor/windows.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`
- Slice goal: Extract content-loading and injected IPC script concerns into a dedicated module while preserving Windows WebView initialization behavior exactly.

### Files Touched

- `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`
  - Added `content` submodule wiring (`#[path = "windows/content.rs"] mod content;`).
  - Rewired WebView controller initialization to call:
    - `content::inject_ipc_script(&wv)`
    - `content::load_ui_content(&wv)`
  - Removed in-file ownership of moved content/IPC helper functions.

- `engine/crates/wavecraft-nih_plug/src/editor/windows/content.rs`
  - New module owning extracted content/IPC script concerns:
    - `inject_ipc_script(...)`
    - `load_ui_content(...)`
    - `get_windows_ipc_primitives()`
    - `get_fallback_html()`

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 2 progress record.

### Invariants (Behavior Preservation)

- [x] Public entrypoint preserved (`create_windows_webview(...)` unchanged).
- [x] WebView initialization semantics preserved exactly (same call order: settings → message handler → IPC script injection → content load).
- [x] IPC primitive JavaScript payload preserved exactly (same global API/queue/listener behavior and diagnostics).
- [x] Embedded `index.html` load and fallback HTML behavior preserved exactly.
- [x] Refactor-only scope preserved (no feature or contract changes).

### Validation

- Required cadence executed:
  - `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
  - `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (fallback per requirement)
  - `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: Extraction boundary was explicit and cohesive (content-loading + injected IPC script concerns only), and no ownership ambiguity or architectural tradeoff emerged.

---

## Slice 1 — Template tsconfig-path helper decomposition (`template/mod.rs`)

### Scope Tracking

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/template/mod.rs`
- Slice goal: Extract tsconfig path-injection helpers into a dedicated module with zero behavior changes.

### Files Touched

- `cli/src/template/mod.rs`
  - Added `mod tsconfig_paths;`.
  - Routed local-dev tsconfig path injection through extracted helper:
    - `tsconfig_paths::inject_tsconfig_paths_if_needed(...)`
  - Removed in-file tsconfig injection constants/helpers now owned by extracted module.

- `cli/src/template/tsconfig_paths.rs`
  - New module containing extracted tsconfig JSONC path-injection concern:
    - `inject_tsconfig_paths_if_needed(...)`
    - `has_jsonc_property_after_anchor(...)`
    - `find_object_bounds_after_key(...)`
    - tsconfig injection constants/snippet data

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 1 progress details for `template/mod.rs` hotspot.

### Invariant Checks (Behavior Preservation)

- [x] Template extraction output semantics preserved.
- [x] Local-dev dependency rewrite flow preserved.
- [x] tsconfig JSONC injection behavior preserved (idempotency, baseUrl handling, anchor/fallback insertion, trailing comma handling).
- [x] Diagnostics and error-context strings preserved.
- [x] SDK dev-mode semantics preserved.
- [x] Refactor-only scope preserved (no feature or contract changes).

### Baseline / Checkpoint Confirmation

- Hotspot baseline re-confirmed before edits:
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
    - Unit tests: **105 passed, 0 failed**
    - Integration tests: **18 passed, 0 failed**

### CI/Validation Checkpoints (Slice 1)

- Requested CLI validation cadence:
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
    - Unit tests: **105 passed, 0 failed**
    - Integration tests: **18 passed, 0 failed**

- Requested full repository gate:
  - `cargo xtask ci-check` — **PASSED**

- Note:
  - One intermediate `cargo test` run reported a single failure in unrelated
    `commands::bundle_command::tests::install_reports_replace_failure_with_diagnostics`; immediate single-test rerun passed and subsequent full `cargo test` run passed cleanly.

### Escalation Log (Mandatory)

- **No Architect escalation required for Slice 1.**
  - Decomposition boundary was clear and low-risk: isolate tsconfig-path helper cluster only.
  - No ambiguous ownership split or architectural decision point was encountered.

---

## Slice 2 — Dependency override cluster decomposition (`template/mod.rs`)

### Scope Tracking

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/template/mod.rs`
- Slice goal: Extract local-dev dependency rewrite concerns from `apply_local_dev_overrides` into a focused module with zero behavior changes.

### Files Touched

- `cli/src/template/mod.rs`
  - Added `mod dependency_overrides;`.
  - Kept `apply_local_dev_overrides` as orchestration point and routed dependency rewrites through extracted helper:
    - `dependency_overrides::apply_dependency_overrides(...)`
  - Preserved canonicalization/error context and tsconfig injection invocation flow.

- `cli/src/template/dependency_overrides.rs`
  - New module containing extracted dependency rewrite cluster:
    - main `wavecraft` rename-dependency replacement (`wavecraft-nih_plug`)
    - SDK crates loop replacement
    - `wavecraft-dev-server` special path replacement
    - npm local file dependency replacements (`@wavecraft/core`, `@wavecraft/components`)

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 2 progress details for `template/mod.rs` hotspot.

### Invariant Checks (Behavior Preservation)

- [x] Main `wavecraft` rename dependency rewrite preserved.
- [x] SDK crates loop rewrite semantics preserved.
- [x] `wavecraft-dev-server` special path rewrite preserved (`{sdk_root}/dev-server`).
- [x] npm local file dependency rewrites preserved for `@wavecraft/core` and `@wavecraft/components`.
- [x] Diagnostics/error-context strings preserved.
- [x] tsconfig injection invocation flow preserved (`tsconfig_paths::inject_tsconfig_paths_if_needed(...)` still called from `apply_local_dev_overrides`).
- [x] Refactor-only scope preserved (no feature or contract changes).

### CI/Validation Checkpoints (Slice 2)

- CLI-focused validation (requested cadence):
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**

- Full repository gate:
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **No Architect escalation required for Slice 2.**
  - Extraction boundary was explicit (dependency rewrite cluster only), and no ambiguous architecture/ownership decision was encountered.

---

## Slice 3 — Template extraction traversal/file handling decomposition (`template/mod.rs`)

### Scope Tracking

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/template/mod.rs`
- Slice goal: Extract template traversal/file handling concern into a dedicated module while preserving extraction behavior and local-dev orchestration.

### Files Touched

- `cli/src/template/mod.rs`
  - Added `mod extract;` and routed extraction entrypoint through extracted module:
    - `extract::extract_dir(&TEMPLATE_DIR, target_dir, vars, &apply_post_processing)`
  - Removed in-file traversal/file-writing implementation.
  - Kept local-dev override + tsconfig injection orchestration behavior unchanged via `apply_post_processing` + existing `apply_local_dev_overrides(...)`.

- `cli/src/template/extract.rs`
  - New module containing extracted template extraction traversal/file handling concerns:
    - recursive directory traversal
    - directory/file skip filters
    - `.template` filename restoration
    - UTF-8 template processing + write-path orchestration

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 3 progress details for `template/mod.rs` hotspot.

### Invariant Checks (Behavior Preservation)

- [x] Public entrypoint preserved: `extract_template(...)` behavior unchanged.
- [x] Directory and file skip filters preserved.
- [x] `.template` filename restoration behavior preserved.
- [x] UTF-8-only processing and non-UTF8 skip behavior preserved.
- [x] Local-dev override + tsconfig injection flow preserved.
- [x] Diagnostics/context strings preserved (`Failed to create directory`, `Invalid directory path`, `Invalid file path`, `Failed to process template`, `Failed to write file`).
- [x] Refactor-only scope preserved (no feature or contract changes).

### CI/Validation Checkpoints (Slice 3)

- CLI-focused validation (requested cadence):
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**

- Full repository gate:
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **No Architect escalation required for Slice 3.**
  - Extraction boundary and responsibility split were explicit in the slice requirements (traversal/file-handling concern only).
  - No ambiguous architecture/ownership decision point was encountered.

---

## Slice 4 — Template local-dev override orchestration decomposition (`template/mod.rs`)

### Scope Tracking

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/template/mod.rs`
- Slice goal: Extract local-dev override orchestration into a focused module while preserving canonicalization semantics, error context, and rewrite/injection sequencing.

### Files Touched

- `cli/src/template/mod.rs`
  - Added `mod overrides;`.
  - Normalized dependency helper module naming in wiring to `mod dependency_rewrites;`.
  - Reduced file to entrypoint/wiring by routing post-processing through extracted helper:
    - `extract::extract_dir(..., &overrides::apply_post_processing)`
  - Removed in-file local-dev override orchestration and related tests now owned by extracted module.

- `cli/src/template/overrides.rs`
  - New focused module containing local-dev override orchestration:
    - `apply_post_processing(...)`
    - `apply_local_dev_overrides(...)`
  - Preserved canonicalization + context message:
    - `Invalid local-dev path: {path}`
  - Preserved sequencing exactly:
    - dependency rewrites → npm rewrites → tsconfig injection
  - Moved local-dev override tests from `mod.rs` into module-local tests.

- `cli/src/template/dependency_rewrites.rs`
  - Introduced plan-aligned naming for dependency rewrite helper module.
  - Split rewrite responsibilities into explicit helpers consumed by orchestration:
    - `apply_dependency_rewrites(...)`
    - `apply_npm_dependency_rewrites(...)`
  - Preserved existing rewrite behavior and regex/error messages.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 4 progress details for `template/mod.rs` hotspot.

### Invariant Checks (Behavior Preservation)

- [x] Canonicalization semantics preserved (`fs::canonicalize(...)`).
- [x] Canonicalization error context preserved exactly (`Invalid local-dev path: ...`).
- [x] Local-dev sequencing preserved exactly:
  - dependency rewrites → npm rewrites → tsconfig injection.
- [x] Existing regex rewrite behavior and diagnostics preserved.
- [x] `extract_template(...)` entrypoint behavior preserved.
- [x] `mod.rs` reduced to entrypoint/wiring/constants focus.
- [x] Refactor-only scope preserved (no feature or contract changes).

### CI/Validation Checkpoints (Slice 4)

- CLI-focused validation (requested cadence):
  - `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path cli/Cargo.toml` — **PASSED**

- Full repository gate:
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **No Architect escalation required for Slice 4.**
  - Module boundary was explicit in the slice request (`overrides.rs` orchestration extraction + behavior preservation constraints).
  - Naming normalization to `dependency_rewrites` was low-risk and implemented without behavioral/contract changes.

---

## Slice 1 — Project root concern extraction (`bundle_command.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/bundle_command.rs`
- Slice goal: Extract project-root detection/validation concern into a dedicated module under `cli/src/commands/bundle/` while keeping `bundle_command.rs` as orchestration/wiring.

### Files Touched

- `cli/src/commands/bundle_command.rs`
  - Added module wiring: `#[path = "bundle/project_root.rs"] mod project_root;`
  - Routed bundle command root-resolution call through extracted module.
  - Removed in-file project-root helper implementations and rewired related tests to module path.

- `cli/src/commands/bundle/project_root.rs`
  - New module owning extracted project-root concern:
    - `resolve_project_root(...)`
    - `find_wavecraft_project_root(...)`
    - internal marker check helper
  - Preserved all existing diagnostics/recovery text and install-flag-aware command suffix behavior.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 1 checkpoint entry for bundle hotspot.

### Invariants (Behavior Preservation)

- [x] Runtime behavior preserved for project-root discovery from nested directories.
- [x] Error diagnostics and recovery guidance text preserved for invalid project context.
- [x] `--install` command-suffix semantics in diagnostics preserved.
- [x] Bundle/install orchestration order and install semantics unchanged.
- [x] Refactor-only scope preserved (no feature or contract changes).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
  - Unit tests: **105 passed, 0 failed**
  - Integration tests: **18 passed, 0 failed**
- `cargo xtask ci-check` — **PASSED**
  - Documentation: passed
  - Lint/type-check: passed
  - Engine + UI tests: passed

### Escalation Log (Mandatory)

- **No Architect escalation required for Slice 1 (bundle hotspot).**
  - Boundary ownership was clear: project-root detection/validation forms a cohesive concern with no cross-module ambiguity.
  - Extraction required no behavior reinterpretation and preserved existing diagnostics verbatim.

---

## Slice 2 — Metadata refresh concern extraction (`bundle_command.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/bundle_command.rs`
- Slice goal: Extract metadata sidecar/discovery + generated TypeScript refresh concern into a dedicated module under `cli/src/commands/bundle/` while preserving behavior and diagnostics.

### Files Touched

- `cli/src/commands/bundle_command.rs`
  - Added module wiring: `#[path = "bundle/metadata_refresh.rs"] mod metadata_refresh;`
  - Kept command orchestration in place and routed metadata refresh via:
    - `metadata_refresh::refresh_generated_types(&project, &package_name)`
  - Removed in-file metadata refresh implementation block and sidecar-focused tests now owned by extracted module.

- `cli/src/commands/bundle/metadata_refresh.rs`
  - New module owning extracted metadata refresh concern:
    - sidecar load and stale checks
    - fallback discovery build + subprocess extraction
    - generated TypeScript parameter/processor type refresh writes
    - concern-local helper functions used only by this cluster
  - Co-located sidecar staleness/freshness tests with the extracted concern.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 2 checkpoint entry for bundle hotspot.

### Invariants (Behavior Preservation)

- [x] Runtime behavior preserved for sidecar-first metadata load and stale fallback to discovery build.
- [x] Existing diagnostics preserved for stale sidecars, discovery build, parse/read failures, and generated type sync logs.
- [x] Generated TypeScript refresh semantics preserved (`write_parameter_types`, `write_processor_types`).
- [x] Bundle/install orchestration and build semantics unchanged.
- [x] Refactor-only scope preserved (no feature or contract changes).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
  - Unit tests: **105 passed, 0 failed**
  - Integration tests: **18 passed, 0 failed**
- `cargo xtask ci-check` — **PASSED**
  - Documentation: passed
  - Lint/type-check: passed
  - Engine + UI tests: passed

### Escalation Log (Mandatory)

- **No Architect escalation required for Slice 2 (bundle hotspot).**
  - Ownership boundary was explicit and cohesive (metadata sidecar/discovery + generated-type refresh concern).
  - No ambiguous cross-module ownership or architectural decision point was encountered during extraction.

---

## Slice 3 — UI assets build/staging/sync concern extraction (`bundle_command.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/bundle_command.rs`
- Slice goal: Extract UI asset build/staging/sync concern into a dedicated module under `cli/src/commands/bundle/` while preserving all bundle/install orchestration semantics and diagnostics.

### Files Touched

- `cli/src/commands/bundle_command.rs`
  - Added module wiring: `#[path = "bundle/ui_assets.rs"] mod ui_assets;`
  - Kept command orchestration in place and routed UI concern calls through extracted module:
    - `ui_assets::build_ui_assets(...)`
    - `ui_assets::sync_ui_dist_into_wavecraft_nih_plug(...)`
  - Removed in-file UI concern implementations and rewired affected tests to module-qualified symbols.

- `cli/src/commands/bundle/ui_assets.rs`
  - New module owning extracted UI asset concern:
    - UI build preconditions/build command (`build_ui_assets`)
    - UI dist staging/sync orchestration (`sync_ui_dist_into_wavecraft_nih_plug`)
    - dependency mode detection (`detect_wavecraft_nih_plug_dependency_mode`)
    - UI dist staging helper (`stage_ui_dist`)
    - local `wavecraft-nih_plug` clean helper tightly coupled to staging flow

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 3 checkpoint entry for bundle hotspot.

### Invariants (Behavior Preservation)

- [x] Bundle/install execution semantics preserved (ordering and control flow unchanged).
- [x] UI build dependency/install/build behavior preserved (`npm install` precondition + `npm run build`).
- [x] UI dist staging behavior preserved for local path dependency mode (replace assets dir + clean `wavecraft-nih_plug`).
- [x] External dependency mode behavior preserved (skip local staging with same user-facing diagnostics).
- [x] Error contexts and recovery diagnostics preserved for UI build/staging/dependency-mode failures.
- [x] Refactor-only scope preserved (no feature or contract changes).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
  - Unit tests: **105 passed, 0 failed**
  - Integration tests: **18 passed, 0 failed**
- `cargo xtask ci-check` — **PASSED**
  - Documentation: passed
  - Lint/type-check: passed
  - Engine + UI tests: passed

### Escalation Log (Mandatory)

- **No Architect escalation required for Slice 3 (bundle hotspot).**
  - Module ownership boundary was clear and cohesive (UI build/staging/sync cluster).
  - No ambiguous cross-module ownership or architectural decision point was encountered during extraction.

---

## Slice 4 — Install/filesystem seam extraction (`bundle_command.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/bundle_command.rs`
- Slice goal: Extract install-target resolution, bundled artifact resolution, install flow, and filesystem seam (`FileSystemOps` + real impl + recursive copy helper) into a dedicated module under `cli/src/commands/bundle/` while preserving orchestration and diagnostics.

### Files Touched

- `cli/src/commands/bundle_command.rs`
  - Added module wiring: `#[path = "bundle/install.rs"] mod install;`
  - Kept bundle command orchestration in place and routed install call through extracted module:
    - `install::install_vst3_bundle(...)`
  - Removed in-file install/filesystem seam implementation and install-specific test block now owned by extracted module.

- `cli/src/commands/bundle/install.rs`
  - New module owning extracted install/filesystem seam concern:
    - bundled artifact path candidate resolution
    - macOS VST3 install target resolution
    - install flow + diagnostics
    - `FileSystemOps` trait, `RealFileSystem`, recursive copy helper (`copy_dir_recursive_impl`)
  - Co-located install diagnostics tests with the extracted ownership boundary.

- `cli/src/commands/bundle/ui_assets.rs`
  - Updated shared recursive copy helper import to new ownership location:
    - `use super::install::copy_dir_recursive_impl;`

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 4 checkpoint entry for bundle hotspot.

### Invariants (Behavior Preservation)

- [x] Bundle command orchestration order preserved.
- [x] Install diagnostics/error/recovery text preserved.
- [x] macOS-only install guard and HOME-based destination resolution preserved.
- [x] Bundled VST3 artifact candidate lookup behavior preserved.
- [x] Filesystem seam behavior preserved (`create_dir_all` / replace existing bundle / recursive copy semantics).
- [x] Existing install-focused tests preserved and moved with ownership.
- [x] Refactor-only scope preserved (no feature or contract changes).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
  - Unit tests: **105 passed, 0 failed**
  - Integration tests: **18 passed, 0 failed**
- `cargo xtask ci-check` — **PASSED**
  - Documentation: passed
  - Lint/type-check: passed
  - Engine + UI tests: passed

### Escalation Log (Mandatory)

- **No Architect escalation required for Slice 4 (bundle hotspot).**
  - Ownership boundary was clear and cohesive (install flow + filesystem seam).
  - No ambiguous module ownership split or architectural decision point was encountered.

---

## Slice 5 — Bundle helper runner extraction (`bundle_command.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/bundle_command.rs`
- Slice goal: Extract bundle helper execution concern into a dedicated module under `cli/src/commands/bundle/` while preserving command orchestration and diagnostics.

### Files Touched

- `cli/src/commands/bundle_command.rs`
  - Added module wiring: `#[path = "bundle/bundle_runner.rs"] mod bundle_runner;`
  - Kept `BundleCommand::execute` orchestration unchanged and routed bundle helper invocation through:
    - `bundle_runner::run_nih_plug_bundle(...)`
  - Removed in-file bundle helper execution functions now owned by extracted module.

- `cli/src/commands/bundle/bundle_runner.rs`
  - New module owning extracted bundle helper execution concern:
    - `run_nih_plug_bundle(...)`
    - `ensure_nih_plug_bundle_helper_manifest(...)`
  - Preserved helper manifest/source materialization and cargo helper invocation behavior.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 5 checkpoint entry for bundle hotspot.

### Invariants (Behavior Preservation)

- [x] `BundleCommand::execute` orchestration order preserved (build → bundle helper → optional install).
- [x] Bundle helper manifest/source generation behavior preserved.
- [x] Bundle helper invocation and exit-code diagnostics preserved.
- [x] Error context strings for helper setup/invocation preserved.
- [x] Refactor-only scope preserved (no feature or contract changes).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **No Architect escalation required for Slice 5 (bundle hotspot).**
  - The extraction boundary was explicit in the slice request (`run_nih_plug_bundle` + helper-manifest generation concern).
  - No ambiguous ownership split or architectural tradeoff was encountered.

---

## Slice 6 — Orchestrator finalization + release-build extraction (`bundle_command.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/bundle_command.rs`
- Slice goal: Perform finalization pass by extracting the last cohesive concern (release package build execution) and co-locating remaining tests into owning modules so `bundle_command.rs` is orchestration/wiring-focused.

### Files Touched

- `cli/src/commands/bundle_command.rs`
  - Added module wiring: `#[path = "bundle/engine_build.rs"] mod engine_build;`
  - Routed release build execution through extracted module:
    - `engine_build::build_release_package(...)`
  - Removed in-file release build command implementation.
  - Removed mixed concern `#[cfg(test)]` block from orchestrator file.

- `cli/src/commands/bundle/engine_build.rs`
  - New module owning extracted release build concern:
    - `build_release_package(...)`
  - Preserved release build invocation behavior and diagnostics (`Failed to run \`cargo build --release\``, `Build failed (exit: ...)`).

- `cli/src/commands/bundle/project_root.rs`
  - Co-located project-root ownership tests moved from `bundle_command.rs`:
    - nested root detection
    - install/non-install invalid-context diagnostics

- `cli/src/commands/bundle/ui_assets.rs`
  - Co-located UI-assets ownership tests moved from `bundle_command.rs`:
    - UI build failure exit handling
    - `wavecraft-nih_plug` dependency mode detection (local path/external)
    - staged UI dist replacement behavior

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 6 completion entry for bundle hotspot.

### Invariants (Behavior Preservation)

- [x] `BundleCommand::execute` orchestration order preserved.
- [x] Release build execution semantics and diagnostics preserved exactly.
- [x] Bundle helper/install call sequence and success messaging preserved.
- [x] Test coverage intent preserved; ownership moved to concern modules only.
- [x] Refactor-only scope preserved (no feature or contract changes).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
  - Unit tests: **105 passed, 0 failed**
  - Integration tests: **18 passed, 0 failed**
- `cargo xtask ci-check` — **PASSED**
  - Documentation: passed
  - Lint/type-check: passed
  - Engine + UI tests: passed

### Escalation Log (Mandatory)

- **No Architect escalation required for Slice 6 (bundle hotspot).**
  - Final boundary was clear: release build execution remained as a cohesive concern, and residual tests had unambiguous owning modules.
  - No ambiguous architectural ownership decision was encountered.

### Hotspot Completion Status

- Bundle hotspot (`cli/src/commands/bundle_command.rs`) status: **COMPLETE** (Slices 1–6 complete and validation green).

---

## Slice 1 — Self-update flow extraction (`update.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/update.rs`
- Slice goal: Extract CLI self-update concern into a dedicated module while keeping `update.rs` as orchestration/wiring for phase coordination, project dependency updates, and summary outcome handling.

### Files Touched

- `cli/src/commands/update.rs`
  - Added module wiring: `mod self_update;`
  - Kept orchestration in `run(...)` and routed self-update phase through:
    - `self_update::update_cli()`
    - `self_update::reexec_with_new_binary()`
  - Retained project dependency update flow and summary decision logic in orchestrator file.
  - Removed self-update helper implementation/test cluster now owned by extracted module.

- `cli/src/commands/update/self_update.rs`
  - New module owning extracted self-update concern:
    - `SelfUpdateResult`
    - `update_cli(...)`
    - `reexec_with_new_binary(...)`
    - install progress parsing helpers and version parsing helpers
  - Co-located self-update parsing/progress tests with ownership boundary.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 1 checkpoint entry for update hotspot.

### Invariants (Behavior Preservation)

- [x] Runtime self-update behavior preserved (`cargo install wavecraft` spawn/wait + stderr capture/parsing).
- [x] Re-exec semantics preserved (`wavecraft update --skip-self` after successful binary update).
- [x] Progress diagnostics preserved (download/compile progress output and failure guidance text).
- [x] Installed-version parsing and "already installed" detection behavior preserved.
- [x] Project dependency update flow and summary/exit semantics preserved in orchestrator.
- [x] Refactor-only scope preserved (no feature or contract changes).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **Architect escalation: NO**
  - Reason: The extraction boundary was explicit and cohesive (self-update flow cluster), with no ambiguous cross-module ownership or behavioral decision point encountered.

---

## Slice 2 — Project dependency update flow extraction (`update.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/update.rs`
- Slice goal: Extract project dependency update flow concern into a dedicated module while keeping `update.rs` as orchestration/wiring for phase coordination and summary/exit handling.

### Files Touched

- `cli/src/commands/update.rs`
  - Added module wiring: `mod project_update;`
  - Kept orchestration in `run(...)` and summary/exit logic in place.
  - Routed project dependency update phase through:
    - `project_update::update_project_deps()`
  - Removed in-file project update implementation block and rewired summary logic type references to extracted module ownership.

- `cli/src/commands/update/project_update.rs`
  - New module owning extracted project dependency update concern:
    - `ProjectUpdateResult`
    - `update_project_deps(...)`
    - `detect_project(...)`
    - `update_rust_deps(...)`
    - `update_npm_deps(...)`
  - Co-located project marker detection tests with concern ownership.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 2 checkpoint entry for update hotspot.

### Invariants (Behavior Preservation)

- [x] `update.rs` remains orchestration/wiring (phase order preserved).
- [x] Project detection semantics preserved (`engine/Cargo.toml`, `ui/package.json`).
- [x] Project update diagnostics preserved for:
  - non-project skip message
  - Rust update success/failure
  - npm update success/failure
- [x] Summary/exit semantics preserved (project error aggregation still controls failure path).
- [x] Refactor-only scope preserved (no feature or contract changes).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **Architect escalation: NO**
  - Reason: The extraction boundary was clear and cohesive (`project_update` concern only), and no ambiguous ownership split or architectural decision point was encountered.

---

## Slice 3 — Summary/exit finalization extraction (`update.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `cli/src/commands/update.rs`
- Slice goal: Finalization pass for remaining cohesive summary/exit concern; extract summary decision + summary print/exit behavior into dedicated module and keep `update.rs` lean orchestration/wiring.

### Files Touched

- `cli/src/commands/update.rs`
  - Added module wiring: `mod summary;`
  - Kept phase orchestration in `run(...)` and routed final summary/exit handling through:
    - `summary::print_summary(&self_update_result, &project_result)`
  - Removed in-file summary outcome enum, summary decision logic, summary print/exit logic, and summary tests (moved to owning module).

- `cli/src/commands/update/summary.rs`
  - New module owning extracted summary/exit concern:
    - `SummaryOutcome`
    - `determine_summary(...)`
    - `print_summary(...)`
  - Co-located summary decision tests (QA-L-003) with ownership boundary.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 3 finalization entry for update hotspot.

### Invariants (Behavior Preservation)

- [x] Update phase order preserved exactly (self-update/re-exec gate → project deps → summary).
- [x] Summary diagnostics/log text preserved exactly:
  - `✨ All updates complete`
  - `✨ Project dependencies updated (CLI self-update skipped)`
- [x] Error aggregation + failure semantics preserved exactly:
  - `Failed to update some dependencies:\n  ...` via `bail!`
- [x] `NoAction` semantics preserved (no extra summary output when CLI self-update failed outside project context).
- [x] Refactor-only scope preserved (no feature/contract changes).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **Architect escalation: NO**
  - Reason: Remaining boundary was clear and cohesive (summary/exit concern only); no ambiguous ownership split or architectural decision point was encountered.

### Hotspot Completion Status

- Update hotspot (`cli/src/commands/update.rs`) status: **COMPLETE** (Slices 1–3 complete and validation green).

---

## Slice 1 — Parse concern extraction (`plugin.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-macros/src/plugin.rs`
- Slice goal: Extract parse-related structures/helpers into `engine/crates/wavecraft-macros/src/plugin/parse.rs` while preserving macro expansion output and contract behavior.

### Files Touched

- `engine/crates/wavecraft-macros/src/plugin.rs`
  - Added parse submodule wiring: `#[path = "plugin/parse.rs"] mod parse;`
  - Kept orchestration/expansion flow in place.
  - Routed parsing of signal processors through extracted helper:
    - `parse::parse_signal_chain_processors(...)`
  - Updated local tests to reference extracted parse ownership boundary.

- `engine/crates/wavecraft-macros/src/plugin/parse.rs`
  - New module owning parse concern:
    - `PluginDef` input parse structure + `Parse` implementation
    - `parse_signal_chain_processors(...)`
  - Preserved all existing parse diagnostics and validation semantics (`name`, `signal`, optional `crate`, `SignalChain![]` constraints).

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 1 checkpoint entry for macros hotspot.

### Invariants (Behavior Preservation)

- [x] Macro DSL parse contract preserved (`name`, `signal`, optional `crate`).
- [x] Parse error diagnostics preserved verbatim for unknown/missing fields and invalid `signal` syntax.
- [x] `SignalChain![]` parsing/validation behavior preserved.
- [x] Macro output behavior preserved (no expansion/contract changes).
- [x] `plugin.rs` remains orchestration/wiring for macro expansion.
- [x] Refactor-only scope preserved (no feature additions).

### Validation

- Required cadence executed:
  - `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED**
    - Note: initial run with exact command form (`cargo fmt --manifest-path engine/Cargo.toml`) reported `Failed to find targets`; reran as workspace format with `--all`.
  - `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **Architect escalation: NO**
  - Reason: boundary was clear and cohesive (`parse` concern extraction only), with no ambiguous ownership split or behavior tradeoff requiring architectural guidance.

---

## Slice 2 — Naming/identifier normalization extraction (`plugin.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-macros/src/plugin.rs`
- Slice goal: Extract naming/identifier normalization helpers into `engine/crates/wavecraft-macros/src/plugin/naming.rs` while preserving macro expansion output exactly and keeping `plugin.rs` as orchestration/wiring.

### Files Touched

- `engine/crates/wavecraft-macros/src/plugin.rs`
  - Added naming submodule wiring: `#[path = "plugin/naming.rs"] mod naming;`
  - Routed naming-dependent calls through extracted ownership:
    - `naming::type_prefix(...)`
    - `naming::processor_id_from_type(...)`
  - Removed inlined naming helper implementations from orchestrator file.
  - Updated local tests to target extracted naming module ownership.

- `engine/crates/wavecraft-macros/src/plugin/naming.rs`
  - New module owning naming/identifier normalization concern:
    - `to_snake_case_identifier(...)`
    - `type_prefix(...)`
    - `processor_id_from_type(...)`
  - Preserved existing snake-case conversion and type-path terminal-segment behavior.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 2 checkpoint entry for macros hotspot.

### Invariants (Behavior Preservation)

- [x] Macro output compatibility preserved (no changes to emitted plugin code shape/contracts).
- [x] `plugin.rs` remains orchestration/wiring for expansion flow.
- [x] Identifier normalization semantics preserved:
  - same snake_case conversion behavior
  - same path terminal-segment selection for processor IDs/prefixes
  - same non-path fallback token-to-string normalization behavior
- [x] Refactor-only scope preserved (no feature additions, no contract changes).

### Validation

- Required cadence executed:
  - `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
  - `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (workspace-correct variant)
  - `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **Architect escalation: NO**
  - Reason: extraction boundary/ownership was explicit (naming/identifier normalization helpers only), and no ambiguous ownership split or architecture decision point was encountered.

---

## Slice 3 — Metadata/codegen-adjacent helper extraction (`plugin.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-macros/src/plugin.rs`
- Slice goal: Extract metadata derivation + metadata-oriented codegen helper cluster into a dedicated module while preserving macro expansion output and generated contract behavior exactly.

### Files Touched

- `engine/crates/wavecraft-macros/src/plugin.rs`
  - Added metadata submodule wiring: `#[path = "plugin/metadata.rs"] mod metadata;`
  - Kept orchestration in place and routed metadata/codegen-adjacent calls through extracted helpers:
    - `metadata::derive_vendor()`
    - `metadata::derive_url()`
    - `metadata::derive_clap_id()`
    - `metadata::generate_vst3_id(...)`
    - `metadata::processor_param_mappings(...)`
    - `metadata::processor_info_entries(...)`
  - Removed in-file helper implementations now owned by metadata module.

- `engine/crates/wavecraft-macros/src/plugin/metadata.rs`
  - New module owning metadata/codegen-adjacent helper cluster:
    - Cargo-derived plugin metadata helpers (vendor/url/CLAP ID)
    - deterministic VST3 ID generation helper
    - FFI metadata token generation helpers for parameter mappings and processor info entries

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 3 checkpoint entry for macros hotspot.

### Invariants (Behavior Preservation)

- [x] Macro output compatibility preserved (no changes to emitted plugin code shape/contracts).
- [x] Generated FFI contract behavior preserved (`wavecraft_get_params_json`, `wavecraft_get_processors_json` generation paths unchanged semantically).
- [x] Cargo metadata derivation semantics preserved exactly (vendor/url/CLAP ID source logic unchanged).
- [x] Deterministic VST3 ID generation semantics preserved exactly (same hash input + byte layout).
- [x] `plugin.rs` remains orchestration/wiring for macro expansion.
- [x] Refactor-only scope preserved (no feature additions, no contract changes).

### Validation

- Required cadence executed:
  - `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
  - `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (workspace-correct variant)
  - `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **Architect escalation: NO**
  - Reason: extraction boundary was clear and low-risk (metadata derivation + metadata-oriented codegen helpers), with no ambiguous ownership split or architectural tradeoff requiring escalation.

---

## Slice 4 — Runtime parameter codegen block extraction (`plugin.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-macros/src/plugin.rs`
- Slice goal: Extract runtime parameter codegen block generation into a dedicated module while preserving macro expansion output exactly and keeping `plugin.rs` as orchestration/wiring.

### Files Touched

- `engine/crates/wavecraft-macros/src/plugin.rs`
  - Added runtime-params submodule wiring: `#[path = "plugin/runtime_params.rs"] mod runtime_params;`
  - Replaced inlined runtime parameter block token construction with module-owned call:
    - `runtime_params::runtime_param_blocks(&signal_processors, &krate)`
  - Kept expansion/orchestration flow in `expand_wavecraft_plugin(...)` unchanged.

- `engine/crates/wavecraft-macros/src/plugin/runtime_params.rs`
  - New module owning extracted runtime parameter codegen block generation:
    - `runtime_param_blocks(...)`
  - Preserved all emitted token semantics for `ParamRange` variants (`Linear`, `Skewed`, `Stepped`, `Enum`), runtime ID construction, group handling, and enum label parse/display behavior.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 4 checkpoint entry for macros hotspot.

### Invariants (Behavior Preservation)

- [x] Macro output compatibility preserved exactly (token generation behavior unchanged).
- [x] `plugin.rs` remains orchestration/wiring for macro expansion.
- [x] Runtime parameter generation semantics preserved:
  - same `ParamRange` → nih-plug param mapping (`FloatParam`/`IntParam`)
  - same enum label display/parse closures
  - same runtime ID format (`"{processor_prefix}_{id_suffix}"`)
  - same group defaulting behavior
- [x] Refactor-only scope preserved (no feature additions, no contract changes).

### Validation

- Required cadence executed:
  - `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
  - `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (workspace-correct variant)
  - `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **Architect escalation: NO**
  - Reason: boundary was clear and low-risk (runtime parameter codegen block cluster), with no ambiguity in ownership split or behavior semantics.

---

## Slice 5 — Finalization/codegen cluster extraction (`plugin.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-macros/src/plugin.rs`
- Slice goal: Finalization pass for remaining cohesive concern by extracting the monolithic expansion/codegen cluster into a dedicated module and leaving `plugin.rs` as lean orchestration/wiring.

### Files Touched

- `engine/crates/wavecraft-macros/src/plugin.rs`
  - Added codegen submodule wiring: `#[path = "plugin/codegen.rs"] mod codegen;`
  - Removed in-file monolithic `quote!` expansion block.
  - Routed expansion generation through:
    - `codegen::generate_plugin_code(codegen::CodegenInput { ... })`
  - Retained orchestration responsibilities (parse → metadata/runtime prep → codegen invocation → error wrapping).

- `engine/crates/wavecraft-macros/src/plugin/codegen.rs`
  - New module owning extracted expansion/codegen concern:
    - `CodegenInput<'a>`
    - `generate_plugin_code(...)`
  - Contains moved plugin expansion `quote!` body verbatim in behavior, including:
    - generated plugin/params/runtime-param structs and trait impls
    - FFI exports (`wavecraft_get_params_json`, `wavecraft_get_processors_json`, `wavecraft_free_string`, `wavecraft_dev_create_processor`)
    - CLAP/VST3 exports and compile-time trait assertions.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 5 finalization entry for macros hotspot.

### Invariants (Behavior Preservation)

- [x] Macro output compatibility preserved exactly (no semantic/codegen contract changes).
- [x] Generated FFI/export surface preserved exactly.
- [x] `plugin.rs` ownership finalized to lean orchestration/wiring.
- [x] Existing macro tests preserved and still passing.
- [x] Refactor-only scope preserved (no feature additions).

### Validation

- Required cadence executed:
  - `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
  - `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (workspace-correct variant)
  - `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log (Mandatory)

- **Architect escalation: NO**
  - Reason: The remaining boundary was unambiguous and cohesive (single codegen cluster), and extraction required no architectural tradeoff or contract reinterpretation.

### Hotspot Completion Status

- Macros hotspot (`engine/crates/wavecraft-macros/src/plugin.rs`) status: **COMPLETE** (Slices 1–5 complete and validation green).

---

## Slice 1 — Envelope extraction boundary (`ipc.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-protocol/src/ipc.rs`
- Slice goal: Extract JSON-RPC envelope-level types/helpers into a dedicated module while preserving wire format compatibility exactly and keeping `ipc.rs` as orchestration/wiring.

### Files Touched

- `engine/crates/wavecraft-protocol/src/ipc.rs`
  - Added submodule wiring:
    - `#[path = "ipc/envelope.rs"] mod envelope;`
    - `pub use envelope::{IpcNotification, IpcRequest, IpcResponse, RequestId};`
  - Removed in-file envelope type/helper implementations and retained non-envelope orchestration/contracts in `ipc.rs`.

- `engine/crates/wavecraft-protocol/src/ipc/envelope.rs`
  - New module owning extracted envelope concern:
    - `IpcRequest`, `IpcResponse`, `IpcNotification`, `RequestId`
    - constructor helpers `IpcRequest::new`, `IpcResponse::{success,error}`, `IpcNotification::new`
  - Preserved all serde attributes/field names/optionality and constructor behavior.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 1 checkpoint entry for `ipc.rs` hotspot.

### Invariants (Behavior Preservation)

- [x] JSON-RPC wire format compatibility preserved exactly:
  - envelope fields unchanged (`jsonrpc`, `id`, `method`, `params`, `result`, `error`)
  - `RequestId` remains `#[serde(untagged)]` (string/number compatibility preserved)
  - optional fields keep same `skip_serializing_if = "Option::is_none"` behavior.
- [x] Public API behavior preserved via `pub use` re-exports from `ipc.rs`.
- [x] `ipc.rs` retained as orchestration/wiring module for broader IPC contracts.
- [x] Refactor-only scope preserved (no feature additions, no contract changes).

### Validation

- Required cadence executed:
  - `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
  - `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (workspace-correct variant)
  - `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: The envelope boundary is explicit, cohesive, and low-risk (pure extraction of JSON-RPC envelope definitions/helpers), with no ambiguous ownership split requiring architectural input.

---

## Slice 2 — Error contract extraction (`ipc.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-protocol/src/ipc.rs`
- Slice goal: Extract JSON-RPC/application error contract types/constants/helpers into `ipc/errors.rs` while preserving wire compatibility and keeping `ipc.rs` as orchestration/wiring with stable public exports.

### Files Touched

- `engine/crates/wavecraft-protocol/src/ipc.rs`
  - Added submodule wiring:
    - `#[path = "ipc/errors.rs"] mod errors;`
    - `pub use errors::{ ... , IpcError };`
  - Removed in-file error contract block now owned by extracted module:
    - `IpcError` type
    - JSON-RPC/application error code constants
    - `impl IpcError` helper constructors/factories
  - Retained `ipc.rs` orchestration/wiring role and existing envelope + method/type contract definitions.

- `engine/crates/wavecraft-protocol/src/ipc/errors.rs`
  - New module owning extracted error contract concern:
    - `IpcError`
    - `ERROR_PARSE`, `ERROR_INVALID_REQUEST`, `ERROR_METHOD_NOT_FOUND`, `ERROR_INVALID_PARAMS`, `ERROR_INTERNAL`, `ERROR_PARAM_NOT_FOUND`, `ERROR_PARAM_OUT_OF_RANGE`
    - `IpcError::{new, with_data, parse_error, invalid_request, method_not_found, invalid_params, internal_error, param_not_found, param_out_of_range}`

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 2 checkpoint entry for `ipc.rs` hotspot.

### Invariants (Behavior Preservation)

- [x] JSON-RPC wire compatibility preserved exactly:
  - `IpcError` serde field names/shape unchanged (`code`, `message`, optional `data`).
  - `#[serde(skip_serializing_if = "Option::is_none")]` on `data` preserved.
- [x] Error code values and helper output strings preserved exactly.
- [x] `ipc.rs` public API stability preserved via re-exports (`pub use errors::{...}`), so downstream imports from `wavecraft_protocol::ipc` remain stable.
- [x] `ipc.rs` retained orchestration/wiring responsibility; no feature additions.
- [x] Refactor-only scope preserved (no contract changes, no behavior changes).

### Validation

- Required cadence executed:
  - `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
  - `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (workspace-correct variant)
  - `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: the requested boundary was explicit (`ipc/errors.rs` for error contract extraction), and no ambiguous ownership split or architecture tradeoff was encountered.

---

## Slice 3 — Method contracts + method/notification constants extraction (`ipc.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-protocol/src/ipc.rs`
- Slice goal: Extract method-specific IPC contract types and method/notification constants into a dedicated module while preserving JSON-RPC wire compatibility exactly and keeping `ipc.rs` as orchestration + stable re-export surface.

### Files Touched

- `engine/crates/wavecraft-protocol/src/ipc.rs`
  - Added submodule wiring:
    - `#[path = "ipc/methods.rs"] mod methods;`
    - `pub use methods::{ ... };` for all moved method-specific contracts and method/notification constants.
  - Removed in-file method-specific contract definitions and method/notification constants.
  - Kept envelope/errors ownership and test orchestration in `ipc.rs`.

- `engine/crates/wavecraft-protocol/src/ipc/methods.rs`
  - New module owning extracted method-contract concern:
    - request/response params/results (`getParameter`, `setParameter`, `getAllParameters`, `getMeterFrame`, `getOscilloscopeFrame`, `getAudioStatus`, `requestResize`, `registerAudio`)
    - method-related data contracts (`ParameterInfo`, `ParameterType`, `ProcessorInfo`, metering/oscilloscope/audio status structs/enums)
    - notification payloads (`ParameterChangedNotification`, `MeterUpdateNotification`)
    - method/notification constants (`METHOD_*`, `NOTIFICATION_*`)

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added Slice 3 checkpoint entry for `ipc.rs` hotspot.

### Invariants (Behavior Preservation)

- [x] JSON-RPC wire format preserved exactly (no serde attribute, field name, enum casing, or optional-field behavior changes).
- [x] Method and notification string constants preserved exactly (same literal values).
- [x] Public API compatibility preserved through `wavecraft_protocol::ipc` re-exports in `ipc.rs`.
- [x] Existing serialization/tests for IPC envelopes and method payloads remained green.
- [x] Refactor-only scope preserved (no feature additions, no semantic changes).

### Validation

- Required cadence executed:
  - `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
  - `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (fallback rerun)
  - `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: extraction boundary was explicit and cohesive (method-specific contract types + method/notification constants), with no ambiguous ownership split requiring architectural guidance.

---

## Slice 1 — Device/config negotiation + stream construction extraction (audio/server.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `dev-server/src/audio/server.rs`
- Slice goal: Extract device/config negotiation and stream construction concerns into a dedicated internal module while preserving runtime behavior and keeping `server.rs` focused on orchestration/wiring.

### Files Touched

- `dev-server/src/audio/server.rs`
  - Added `mod device_setup;` and rewired:
    - `AudioServer::new(...)` device/config setup via `device_setup::negotiate_default_devices_and_configs(...)`
    - input stream construction via `device_setup::build_input_stream(...)`
    - output stream construction via `device_setup::build_output_stream(...)`
  - Kept public API/signatures and orchestration flow intact.

- `dev-server/src/audio/server/device_setup.rs`
  - New module owning extracted cohesive boundary:
    - default input/output device + config negotiation
    - input stream construction (including pre-callback buffer setup and callback wiring)
    - output stream construction (ring-consumer playback callback wiring)

- `dev-server/src/host.rs`
  - Minimal lint-only correction required by strict validation gate (`clone_on_copy`) with no functional behavior change.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this slice progress record.

### Invariants (Behavior Preservation)

- [x] No functional changes introduced (refactor-only extraction).
- [x] Public API/signatures remain stable (`AudioServer`, `AudioConfig`, `AudioHandle`, `start/new` signatures unchanged).
- [x] Device negotiation behavior/logging preserved (same required-device semantics, same mismatch warning behavior).
- [x] Stream callback behavior preserved (same deinterleave/process/modifier/meter/interleave flow and output routing).
- [x] Real-time safety unchanged in callback paths:
  - pre-allocated callback buffers remain allocated before callback execution;
  - lock-free ring-buffer usage unchanged;
  - no new locks/allocations introduced in RT callback hot paths.

### Validation

- Required cadence:
  - `cargo fmt --manifest-path dev-server/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
    - unit/integration summary: **42 passed, 0 failed** (`37` unit + `3` audio status integration + `2` reload integration)
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Boundary ownership was cohesive and unambiguous: device/config negotiation and stream construction are tightly coupled setup concerns that can be isolated from server orchestration without crossing ownership boundaries.

---

## Slice 2 — Output modifiers + gain read/apply helpers extraction (audio/server.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `dev-server/src/audio/server.rs`
- Slice goal: Extract runtime output-modifier flow and parameter-read/gain-apply helpers into a dedicated module while preserving behavior and keeping `server.rs` orchestration-focused.

### Files Touched

- `dev-server/src/audio/server.rs`
  - Added `mod output_modifiers;` wiring and removed in-file output modifier/gain helper cluster.
  - Kept orchestration responsibilities (`AudioServer` setup/start + meter computation) in place.

- `dev-server/src/audio/server/device_setup.rs`
  - Rewired callback modifier invocation to extracted module:
    - `super::output_modifiers::apply_output_modifiers(...)`

- `dev-server/src/audio/server/output_modifiers.rs`
  - New module owning extracted boundary:
    - runtime oscillator/output-modifier flow
    - canonical gain multiplier reads (`input_gain_level`, `output_gain_level`)
    - gain application helper tied to modifier path
  - Co-located output-modifier behavior tests moved from `server.rs`.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 2 progress record.

### Invariants (Behavior Preservation)

- [x] Runtime oscillator/output-modifier behavior preserved.
- [x] Canonical parameter-ID policy preserved (no legacy alias fallback introduced).
- [x] Gain multiplier read/validation/clamp behavior preserved.
- [x] Gain application path preserved and remains tied to output modifier flow.
- [x] RT-safety characteristics unchanged (no new locks/allocations introduced in callback hot path).
- [x] Refactor-only scope preserved (no feature or contract changes).

### Validation

- Required cadence executed:
  - `cargo fmt --manifest-path dev-server/Cargo.toml` — **PASSED**
  - `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
    - Unit/integration summary: **42 passed, 0 failed**
    - Output-modifier tests now under `audio::server::output_modifiers::tests`: **10 passed**
  - `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Boundary remained narrow and cohesive (output modifier + gain helper cluster) with no cross-cutting ownership ambiguity.

---

## Slice 3 — Metering helper extraction (audio/server.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `dev-server/src/audio/server.rs`
- Slice goal: Extract metering helper concern (peak/RMS computation + meter notification packaging/cadence helper) into a dedicated module while preserving callback behavior and keeping `server.rs` orchestration-focused.

### Files Touched

- `dev-server/src/audio/server.rs`
  - Added `mod metering;`.
  - Removed in-file metering helper ownership from orchestrator file.

- `dev-server/src/audio/server/metering.rs`
  - New module owning extracted metering concern:
    - `maybe_build_meter_update(...)` (notification packaging + cadence gate)
    - `compute_peak_and_rms(...)` (peak/RMS helper)

- `dev-server/src/audio/server/device_setup.rs`
  - Rewired input callback metering path to call:
    - `super::metering::maybe_build_meter_update(frame_counter, left, right)`
  - Preserved existing lock-free meter push flow (`meter_producer.push(...)`).

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 3 progress record.

### Invariants (Behavior Preservation)

- [x] Meter cadence preserved exactly (every other callback via `frame_counter.is_multiple_of(2)`).
- [x] Meter fields preserved exactly (`timestamp_us`, `left_peak`, `left_rms`, `right_peak`, `right_rms`).
- [x] Peak/RMS computation logic preserved exactly (same absolute-peak fold + RMS formula).
- [x] Callback flow preserved (capture → metering helper → lock-free push → interleave/ring write).
- [x] RT-safety unchanged (no new locks/blocking/allocations in callback hot path).
- [x] Refactor-only scope preserved (no feature/contract changes).

### Validation

- `cargo fmt --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
  - Unit/integration summary: **42 passed, 0 failed** (`37` unit + `3` audio status integration + `2` reload integration)
- `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Boundary was narrow and cohesive (meter math + meter packaging/cadence helper) and did not crosscut broader ownership beyond callback-local metering responsibilities.

---

## Slice 4 — Input callback pipeline decomposition (audio/server.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `dev-server/src/audio/server.rs`
- Slice goal: Extract the cohesive input callback processing pipeline (deinterleave/process/modifier/oscilloscope/meter/interleave/ring-write) from stream setup wiring into a dedicated helper module while preserving behavior and keeping `server.rs` orchestration-focused.

### Files Touched

- `dev-server/src/audio/server.rs`
  - Added `mod input_pipeline;` wiring for extracted callback-pipeline ownership.

- `dev-server/src/audio/server/device_setup.rs`
  - Kept input stream build wiring responsibility.
  - Replaced in-closure callback processing body with delegated call to extracted helper:
    - `input_pipeline.process_callback(data)`

- `dev-server/src/audio/server/input_pipeline.rs`
  - New module owning extracted input callback pipeline concern:
    - callback-local state (frame counter, oscillator phase, preallocated channel/interleave buffers)
    - `InputCallbackPipeline::new(...)` pre-callback allocation/setup
    - `InputCallbackPipeline::process_callback(...)` with unchanged processing sequence.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 4 progress record.

### Invariants (Behavior Preservation)

- [x] Input callback sequence preserved exactly:
  - deinterleave → DSP process → output modifiers → oscilloscope capture → optional meter update/push → interleave → ring write.
- [x] Callback-local state semantics preserved (`frame_counter`, `oscillator_phase`).
- [x] Buffer sizing/reuse semantics preserved (same preallocated `left/right/interleave` buffers, no per-callback allocation).
- [x] Lock-free ring interactions preserved (`meter_producer.push`, `ring_producer.push` with drop-on-full behavior unchanged).
- [x] Public API/signatures preserved (`AudioServer`/`AudioHandle`/`AudioConfig`/`start/new` unchanged).
- [x] Refactor-only scope preserved (no feature/contract changes).

### Validation

- `cargo fmt --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: ownership boundary was cohesive and explicit (input callback processing pipeline as a single concern), with no ambiguous architecture split or contract decision required.

---

## Slice 5 — Output callback routing decomposition (audio/server.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `dev-server/src/audio/server.rs`
- Slice goal: Extract output callback routing concern (ring-consumer pop, mono downmix, stereo-to-device-channel mapping, and underflow/multichannel silence fill) into a dedicated helper module while preserving behavior and keeping `server.rs` orchestration-focused and `device_setup.rs` stream-wiring-focused.

### Files Touched

- `dev-server/src/audio/server.rs`
  - Added `mod output_routing;` wiring for extracted output callback routing ownership.

- `dev-server/src/audio/server/device_setup.rs`
  - Kept output stream build wiring responsibility.
  - Replaced in-closure output routing body with delegated call:
    - `super::output_routing::route_output_callback(data, output_channels, &mut ring_consumer)`

- `dev-server/src/audio/server/output_routing.rs`
  - New module owning extracted output callback routing concern:
    - `route_output_callback(...)`
  - Preserves existing callback behavior:
    - `output_channels == 0` → full silence
    - ring underflow → per-sample silence fallback
    - mono output → `(L + R) * 0.5` downmix
    - multi-channel output → `L/R` on channels `0/1`, channels `2+` zeroed

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 5 progress record.

### Invariants (Behavior Preservation)

- [x] Output callback routing semantics preserved exactly (same ring pop order, same downmix/mapping/silence behavior).
- [x] Stream wiring ownership preserved in `device_setup.rs`; no contract/public API changes.
- [x] `server.rs` remains orchestration/wiring focused.
- [x] RT-safety unchanged in output callback hot path (no blocking, no locks, no allocations introduced).
- [x] Refactor-only scope preserved (no feature/contract changes).

### Validation

- `cargo fmt --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
  - Unit/integration summary: **42 passed, 0 failed** (`37` unit + `3` audio status integration + `2` reload integration)
- `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: output callback routing is a narrow, cohesive concern with a clear ownership boundary separate from stream wiring/orchestration, and extraction required no architectural tradeoff.

---

## Tier 2 — Batch 7 quick-scan cleanup

### Scope

- Tier: **Tier 2 (quick-scan cleanup)**
- Batch: **7**
- Goal: Apply bounded, behavior-preserving readability/maintainability cleanups only (no architecture shifts, no cross-crate API changes).

### Files

1. `cli/src/commands/create.rs`
   - Extracted small local helpers for:
     - output directory resolution
     - default author metadata resolution
     - git init command execution + success check
   - Preserved command flow and all CLI UX text/output.

2. `cli/src/sdk_detect.rs`
   - Extracted minor path/marker helpers for repeated join/marker checks:
     - `join_repo_path(...)`
     - `has_sdk_marker(...)`
   - Added constant reuse for `engine/crates` relative path.
   - Preserved detection semantics.

3. `dev-server/src/audio/atomic_params.rs`
   - Added local ordering constant reuse (`PARAM_ORDERING`) and lookup helper (`lookup_param(...)`).
   - Preserved lock-free semantics (`AtomicF32`, relaxed ordering, immutable map ownership).

4. `engine/crates/wavecraft-processors/src/oscilloscope.rs`
   - Extracted small internal helpers to flatten capture flow:
     - frame sample capture
     - history update
     - trigger-start resolution
     - aligned window copy
     - frame publication
   - Preserved capture/trigger/publication behavior and test expectations.

### Invariants

- [x] Behavior-preserving cleanup only.
- [x] No architecture or cross-crate API changes.
- [x] Changes bounded to Batch 7 target files + this ledger update.
- [x] User-facing CLI text/flow unchanged for `create` command.
- [x] Lock-free semantics preserved in atomic parameter bridge.
- [x] Oscilloscope output semantics preserved (triggering/history/alignment/publication).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml --all` — **PASSED**
- `cargo fmt --manifest-path dev-server/Cargo.toml --all` — **PASSED**
- `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation

- **Architect escalation: NO**
  - Rationale: all changes are local helper extractions/constant reuse within existing file boundaries, with no ambiguous ownership split or design tradeoffs.

---

## Slice 6 — Start-time stream/ring setup orchestration decomposition (audio/server.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `dev-server/src/audio/server.rs`
- Slice goal: Extract start-time stream/ring setup orchestration from `AudioServer::start` into a dedicated internal helper module (ring buffers + oscilloscope channel + input/output stream build and start invocation), while preserving public API and runtime behavior.

### Files Touched

- `dev-server/src/audio/server.rs`
  - Added `mod startup_wiring;`.
  - Reduced `AudioServer::start` to orchestration by delegating start-time stream/ring setup through:
    - `startup_wiring::start_audio_io(...)`
  - Preserved `AudioServer::start` signature and return shape.

- `dev-server/src/audio/server/startup_wiring.rs`
  - New module owning extracted startup wiring concern:
    - input/output audio ring buffer creation
    - meter ring + oscilloscope channel/tap setup
    - `device_setup::build_input_stream(...)` / `device_setup::build_output_stream(...)` invocation
    - stream `.play()` startup and startup log emission
    - assembly of unchanged `(AudioHandle, meter_consumer, oscilloscope_consumer)` return tuple.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 6 progress record.

### Invariants (Behavior Preservation)

- [x] Public API and contract preserved (`AudioServer::start` signature/return shape unchanged).
- [x] Startup ordering preserved (sample-rate set before stream setup; input stream start before output stream start).
- [x] Ring buffer capacities/semantics preserved (`buffer_size * 2 * 4` audio ring, `64` meter ring).
- [x] Oscilloscope channel/tap setup preserved (same channel capacity and sample-rate assignment).
- [x] Stream build/start error contexts and startup logs preserved.
- [x] RT-safety semantics preserved (no additional locks/blocking/allocations introduced in callback hot paths).
- [x] Refactor-only scope preserved (no feature/contract changes).

### Validation

- `cargo fmt --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
  - Unit/integration summary: **42 passed, 0 failed** (`37` unit + `3` audio status integration + `2` reload integration)
- `cargo xtask ci-check` — **PASSED**
  - Documentation: passed
  - Lint/type-check: passed
  - Engine + UI tests: passed

### Escalation Log

- **Architect escalation: NO**
  - Reason: start-time stream/ring setup forms a narrow, cohesive ownership boundary (startup wiring only), and extraction required no architectural tradeoff or contract reinterpretation.

---

## Slice 7 — Gain-bound constants ownership relocation (audio/server.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `dev-server/src/audio/server.rs`
- Slice goal: Remove remaining non-orchestration coupling by relocating gain-bound constants ownership from `server.rs` into `output_modifiers.rs` (the owning usage module) while preserving exact runtime behavior and public contracts.

### Files Touched

- `dev-server/src/audio/server.rs`
  - Removed gain-bound constants from orchestration shell:
    - `GAIN_MULTIPLIER_MIN`
    - `GAIN_MULTIPLIER_MAX`

- `dev-server/src/audio/server/output_modifiers.rs`
  - Added local ownership of gain-bound constants used by `read_gain_multiplier(...)`:
    - `GAIN_MULTIPLIER_MIN`
    - `GAIN_MULTIPLIER_MAX`
  - Removed coupling import from parent module (`use super::{...}`) and kept all call sites unchanged.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 7 progress record.

### Invariants (Behavior Preservation)

- [x] Gain clamp bounds preserved exactly (`0.0..=2.0`).
- [x] `read_gain_multiplier(...)` logic unchanged (same finite check, same clamp, same fallback to `1.0`).
- [x] Runtime output-modifier behavior unchanged (same canonical gain IDs and gain-apply flow).
- [x] Public API/signatures unchanged (`AudioServer`, `AudioConfig`, `AudioHandle`, `new/start/has_output`).
- [x] `server.rs` remains orchestration/public API shell only.
- [x] Refactor-only scope preserved (no feature/contract changes).

### Validation

- `cargo fmt --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: ownership boundary is explicit and narrow (constants now co-located with the only consuming module), with no architectural tradeoff or contract reinterpretation required.

---

## Slice 1 — Runtime checks + handle helpers extraction (`editor/windows.rs`)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`
- Slice goal: Extract runtime checks and parent/child window handle helper concern into a dedicated module while preserving `create_windows_webview` behavior/signature and Windows init diagnostics exactly.

### Files Touched

- `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`
  - Added narrow submodule wiring:
    - `#[path = "windows/runtime_checks.rs"] mod runtime_checks;`
  - Rewired existing entrypoint flow to extracted helpers only:
    - `runtime_checks::check_webview2_runtime()`
    - `runtime_checks::get_parent_hwnd(...)`
    - `runtime_checks::create_child_window(...)`
  - Removed in-file implementations for:
    - WebView2 runtime presence check
    - parent HWND extraction from `ParentWindowHandle`
    - child host window creation + child window proc

- `engine/crates/wavecraft-nih_plug/src/editor/windows/runtime_checks.rs`
  - New module owning extracted concern:
    - `check_webview2_runtime(...)`
    - `get_parent_hwnd(...)`
    - `create_child_window(...)`
    - private `window_proc(...)`
  - Preserved all existing diagnostics/error strings and Win32 setup logic.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 1 ledger entry for the Windows editor hotspot.

### Invariants (Behavior Preservation)

- [x] `create_windows_webview` signature unchanged.
- [x] `create_windows_webview` initialization order preserved exactly:
  - COM init once → WebView2 runtime check → parent HWND extraction → child window creation → WebView2 init.
- [x] Runtime detection diagnostics preserved exactly (including install URL and detection-failure wording).
- [x] Parent handle and null-HWND validation behavior preserved exactly.
- [x] Child window registration/creation behavior and diagnostics preserved exactly.
- [x] No cross-platform rewrites; Windows-only extraction scope maintained.
- [x] Refactor-only scope preserved (no feature/contract changes).

### Validation

- `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
- `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (required fallback)
- `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: the requested boundary (runtime checks + handle helpers) was explicit and cohesive, and extraction did not introduce ownership ambiguity or behavior interpretation risk.

---

## Slice 3 — WebView2 init wiring extraction (editor/windows.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`
- Slice goal: Extract WebView2 initialization wiring concern into a dedicated module while preserving initialization order, timeout behavior, IPC handler semantics, content-loading behavior, diagnostics, and public API signatures.

### Files Touched

- `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`
  - Added narrow submodule wiring:
    - `#[path = "windows/webview2_init.rs"] mod webview2_init;`
  - Rewired existing init call path from in-file helper to extracted module:
    - `webview2_init::initialize_webview2(...)`
  - Kept `create_windows_webview(...)` signature and flow intact.
  - Exposed IPC trait to sibling submodule boundary only (`pub(super) trait JsonIpcHandler`).
  - Removed moved WebView2 init helper implementations from this file:
    - `initialize_webview2(...)`
    - `configure_webview_settings(...)`
    - `setup_web_message_handler(...)`
    - `pump_messages_until_ready(...)`

- `engine/crates/wavecraft-nih_plug/src/editor/windows/webview2_init.rs`
  - New module owning extracted WebView2 init wiring concern:
    - `initialize_webview2(...)`
    - settings setup helper
    - web-message IPC handler setup helper
    - readiness message-pump/timeout helper
  - Preserved call ordering and side effects, including:
    - controller bounds/visibility setup
    - settings configuration
    - IPC handler registration
    - content script injection and content loading ordering
    - readiness wait loop timeout/logging semantics.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 3 ledger entry.

### Invariants (Behavior Preservation)

- [x] `create_windows_webview(...)` public signature preserved.
- [x] WebView2 initialization order preserved exactly:
  - environment creation → controller creation → bounds/visibility → settings → web-message handler → IPC script injection → content load → controller/webview store → success log → readiness pump.
- [x] IPC message handling semantics preserved exactly (request/response handling + escaping + JS callback dispatch unchanged).
- [x] Content-loading semantics preserved exactly (delegation to existing extracted `content` module unchanged).
- [x] Readiness timeout behavior/logging preserved exactly (`5s` timeout + same timeout error log).
- [x] Refactor-only scope preserved (no feature/contract changes).

### Validation

- `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
- `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (required fallback)
- `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: extraction boundary was explicit and cohesive (WebView2 init wiring + directly related setup helpers), and implementation introduced no ownership ambiguity or contract/behavior tradeoffs.

---

## Slice 4 — Web-message IPC bridge extraction (editor/windows.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`
- Slice goal: Extract WebView2 web-message IPC bridge plumbing from init wiring into a dedicated module so `webview2_init.rs` stays focused on setup sequencing, with zero behavior/contract changes.

### Files Touched

- `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`
  - Added narrow module wiring:
    - `#[path = "windows/ipc_bridge.rs"] mod ipc_bridge;`

- `engine/crates/wavecraft-nih_plug/src/editor/windows/webview2_init.rs`
  - Removed in-file ownership of web-message IPC bridge helper and delegated to extracted module:
    - `ipc_bridge::setup_web_message_handler(&wv, handler_inner.clone())`
  - Kept WebView2 init sequencing responsibility in place.

- `engine/crates/wavecraft-nih_plug/src/editor/windows/ipc_bridge.rs`
  - New module owning extracted web-message IPC bridge concern:
    - handler registration (`add_WebMessageReceived`)
    - request reception (`TryGetWebMessageAsString`)
    - JSON IPC forwarding (`handler.handle_json(...)`)
    - response escaping + JS callback dispatch (`globalThis.__WAVECRAFT_IPC__._receive(...)`)
  - Preserved existing logging statements and escaping semantics exactly.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 4 ledger entry.

### Invariants (Behavior Preservation)

- [x] `create_windows_webview(...)` public signature unchanged.
- [x] WebView2 initialization order preserved exactly:
  - settings → web-message handler registration → IPC script injection → content load.
- [x] Web-message IPC semantics preserved exactly (same request handling, response generation, escaping, and JS dispatch target).
- [x] IPC logging preserved exactly (`[IPC] Received message`, `[IPC] Response`).
- [x] Refactor-only scope preserved (no feature/contract changes).

### Validation

- `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
- `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (required fallback)
- `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: extraction boundary was explicit and narrow (web-message IPC bridge registration + forwarding glue only), and no ownership ambiguity or behavior tradeoff required architectural guidance.

---

## Slice 5 — Module root migration to windows/mod.rs (editor/windows.rs)

### Scope

- Tier: **Tier 1 (hotspot decomposition)**
- Hotspot: `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`
- Slice goal: Migrate Windows module root from `editor/windows.rs` to `editor/windows/mod.rs` to align module layout with extracted submodule structure, with zero behavior/signature/contract changes.

### Files Touched

- `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`
  - Removed legacy flat module root file.

- `engine/crates/wavecraft-nih_plug/src/editor/windows/mod.rs`
  - New module root containing moved `windows.rs` logic unchanged in behavior.
  - Replaced path-qualified submodule declarations with local declarations:
    - `mod content;`
    - `mod ipc_bridge;`
    - `mod runtime_checks;`
    - `mod webview2_init;`
  - Kept all public and internal signatures unchanged (`create_windows_webview`, `WindowsWebView`, `JsonIpcHandler`, `WebViewHandle` impl behavior).

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Added this Slice 5 migration record.

### Invariants (Behavior Preservation)

- [x] No behavior changes introduced (module root relocation only).
- [x] No public API/signature changes (`editor::windows` exports/entrypoints preserved by Rust module resolution).
- [x] Existing extracted submodules remain functional and path-stable under new root layout.
- [x] WebView2 initialization order and IPC/content wiring remain unchanged.
- [x] Refactor-only scope preserved (no feature/contract changes).

### Validation

- `cargo fmt --manifest-path engine/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation Log

- **Architect escalation: NO**
  - Reason: the requested migration boundary was explicit and mechanical (module root relocation + minimal path cleanup), and no ownership or behavior tradeoff required architectural guidance.

---

## Tier 2 — Batch 1 quick-scan cleanup

### Scope

- Tier: **Tier 2 (quick scan)**
- Batch: **Batch 1**
- Goal: Bounded cleanup only (naming consistency, dead/unused cleanup, obvious local helper extraction, error-message consistency) with **no behavior/API/architecture changes**.

### Files Touched

- `engine/crates/wavecraft-dsp/src/combinators/chain.rs`
  - Corrected doc wording for `ChainParams` to match actual behavior (merged specs, no ID prefixing).
  - Extracted local `extend_specs(...)` helper inside `param_specs()` to remove duplicated merge logic.

- `engine/crates/wavecraft-bridge/src/plugin_loader.rs`
  - Standardized JSON parse display message to generic payload wording (`Failed to parse JSON payload: ...`) for params/processors consistency.
  - Extracted duplicated FFI JSON load/parse/free sequence into local helper `load_json_via_ffi(...)`.

- `engine/crates/wavecraft-bridge/src/in_memory_host.rs`
  - Extracted duplicated `ParameterInfo` materialization into local helper `materialize_parameter(...)`.
  - Reused helper in both `get_parameter(...)` and `get_all_parameters(...)` for local consistency.

- `engine/crates/wavecraft-bridge/src/handler.rs`
  - Added local helper `parse_required_params(...)` to unify repeated required-param parsing path and missing-param error shape.
  - Updated malformed-request comment to match implementation (synthetic request ID rather than `null`).

- `engine/crates/wavecraft-nih_plug/src/editor/mod.rs`
  - Added explicit `#[path = "windows/mod.rs"] mod windows;` to disambiguate module resolution and unblock required validation commands (no runtime behavior change).

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Appended Tier 2 Batch 1 record.

### Bounded-Change Invariants

- [x] No behavior changes introduced.
- [x] No public API surface changes introduced.
- [x] No architecture changes introduced.
- [x] Cleanup remained local to naming/consistency/helper extraction scope.
- [x] No roadmap or archived spec files were modified.

### Validation

- `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
- `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (required fallback)
- `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Deferred Follow-ups

- **Candidate:** clean up duplicate Windows module root artifact (`editor/windows.rs` vs `editor/windows/mod.rs`) repository-wide.
  - Current batch applied a non-behavioral explicit module path in `editor/mod.rs` to keep checks green.
  - Deeper normalization of file layout is deferred to a dedicated cleanup slice to avoid scope drift.

### Escalation Decision

- **Architect escalation: NO**
  - Reason: all changes stayed within bounded Tier 2 quick-scan cleanup scope; no redesign or ownership ambiguity required architectural input.

---

## Tier 2 — Batch 2 quick-scan cleanup

### Scope

- Tier: **Tier 2 (quick scan)**
- Batch: **Batch 2**
- Goal: Bounded cleanup only (naming consistency, dead/unused cleanup, obvious local helper extraction, error-message consistency) with **no behavior/API/architecture changes**.

### Files Touched

- `cli/src/project/detection.rs`
  - Introduced shared `PROJECT_CONTEXT_HINT` constant to remove duplicated guidance text.
  - Standardized missing-marker diagnostics by appending the same guidance hint to missing `ui/package.json` and `engine/Cargo.toml` errors.

- `cli/src/project/dylib.rs`
  - Extracted duplicated debug-directory rendering into local helper `format_debug_dirs(...)`.
  - Reused helper for both "build output directory not found" error paths for message consistency.

- `dev-server/src/host.rs`
  - Extracted duplicated constructor state initialization into local helper `initialize_shared_state()`.
  - Applied small naming consistency cleanup in `request_resize(...)` argument names (`width`, `height`).

- `engine/crates/wavecraft-processors/src/oscillator.rs`
  - Extracted phase-advance/wrap logic into local helper `advance_phase(...)` to remove inline duplication.
  - Aligned `level` parameter doc wording with actual normalized range (`0.0 – 1.0`).

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Appended Tier 2 Batch 2 record.

### Bounded-Change Invariants

- [x] No behavior changes introduced.
- [x] No public API surface changes introduced.
- [x] No architecture changes introduced.
- [x] Cleanup remained local to naming/consistency/helper extraction scope.
- [x] No roadmap or archived spec files were modified.

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo fmt --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
  - Note: one intermediate clippy failure (`clippy::type_complexity`) from newly introduced helper return tuple was fixed by switching to a small private `SharedState` struct; rerun passed.
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo fmt --manifest-path engine/Cargo.toml` — **FAILED** (`Failed to find targets`)
- `cargo fmt --manifest-path engine/Cargo.toml --all` — **PASSED** (required fallback)
- `cargo clippy --manifest-path engine/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path engine/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Deferred Follow-ups

- **Candidate:** consolidate repeated project-structure validation checks in `cli/src/project/detection.rs` into explicit small validators (`require_dir`, `require_file`) if additional detection branches are added.
  - Deferred to avoid broadening quick-scan scope beyond bounded cleanup.

- **Candidate:** consider replacing `f32` waveform index in `OscillatorParams` with an enum-backed parameter abstraction at processor boundary.
  - Deferred because this would be a behavior/contract redesign (outside Tier 2 bounded cleanup).

### Escalation Decision

- **Architect escalation: NO**
  - Reason: changes were strictly local cleanups with no ownership ambiguity and no architectural redesign.

---

## Tier 2 — Batch 3 quick-scan cleanup

### Scope

- Tier: **Tier 2 (quick scan)**
- Batch: **Batch 3**
- Goal: Bounded cleanup only (naming consistency, dead/unused cleanup, obvious local helper extraction, error-message consistency) with **no behavior/API/architecture changes**.

### Files Touched

- `dev-server/src/reload/watcher.rs`
  - Performed naming consistency cleanup for keep-alive fields by using underscore-prefixed names (`_debouncer`, `_shutdown_rx`) and removed now-unnecessary dead-code attributes.
  - Clarified local variable names in `new(...)` (`engine_dir_path`, `watched_root`) to make watcher-root ownership explicit.
  - Standardized watcher error prefixing (`Warning: file watcher error: ...`) to align with existing warning style.
  - Extracted editor temporary-file predicate into local helper `is_editor_temp_file(...)`.

- `dev-server/src/reload/rebuild.rs`
  - Extracted duplicated TypeScript-regeneration failure logging into local helper `log_regeneration_failure(...)`.
  - Unified retry guidance text emission for parameter and processor type-generation failure paths.

- `dev-server/src/ws/mod.rs`
  - Improved naming consistency by replacing hardcoded method/notification strings with protocol constants where available (`METHOD_REGISTER_AUDIO`, `NOTIFICATION_METER_UPDATE`, `NOTIFICATION_PARAMETER_CHANGED`).
  - Introduced local constant `NOTIFICATION_PARAMETERS_CHANGED` for the dev-server-only notification string to avoid inline literal duplication.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Appended Tier 2 Batch 3 quick-scan record.

### Bounded-Change Invariants

- [x] No behavior changes introduced.
- [x] No public API surface changes introduced.
- [x] No architecture changes introduced.
- [x] Cleanup remained local to naming/consistency/helper extraction scope.
- [x] No roadmap or archived spec files were modified.

### Validation

- `cargo fmt --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
  - Unit/integration summary: **42 passed, 0 failed** (`37` unit + `3` audio status integration + `2` reload integration)
- `cargo xtask ci-check` — **PASSED**

### Deferred Follow-ups

- **Candidate:** replace the remaining dev-server-local notification literal `parametersChanged` with a protocol-level exported constant for full cross-transport parity.
  - Deferred because introducing/standardizing protocol constants is cross-crate contract hygiene and outside this bounded quick-scan scope.

### Escalation Decision

- **Architect escalation: NO**
  - Reason: all edits were strictly local cleanup (naming/consistency/helper extraction) with no contract or ownership redesign.

---

## Tier 2 — Batch 4 quick-scan cleanup

### Scope

- Tier: **Tier 2 (quick scan)**
- Batch: **Batch 4**
- Goal: Bounded cleanup only (small helper extraction + duplication reduction + wording consistency) with **no behavior/API/architecture changes**.

### Files Touched

- `cli/src/commands/start/metadata_cache.rs`
  - Extracted shared stale-sidecar decision helper (`stale_sidecar_reason(...)`) used by both params/processors cache paths.
  - Extracted sidecar read-if-fresh helper (`try_read_cached_sidecar_json(...)`) to dedupe sidecar existence/mtime/read flow while keeping per-type JSON parsing local.
  - Extracted sidecar write helper (`write_sidecar_json(...)`) to dedupe path/write IO flow while preserving existing serialize/write error contexts.
  - Normalized stale-cache diagnostics formatting between parameter and processor sidecars.

- `cli/src/commands/start/tsconfig_paths.rs`
  - Extracted insertion-string builder (`build_updated_content(...)`) to remove duplicated string assembly paths for anchored and fallback insertion.
  - Added shared warning constants for compiler-options and existing-paths warning paths.
  - Normalized warning wording prefix to consistent `could not inject SDK TypeScript paths: ...`.
  - Applied naming symmetry cleanup for comma-need booleans (`needs_prepend_comma`).

- `cli/src/commands/update/self_update.rs`
  - Extracted cargo-install spawn/wait/progress capture flow into helper (`run_cargo_install_with_progress(...)`).
  - Added shared failure-reporting helper (`report_self_update_failure(...)`) and shared manual-guidance helper to dedupe repeated user guidance text.
  - Preserved phase detection/progress reporting and updated/already-up-to-date decision behavior.

- `dev-server/src/audio/server/output_modifiers.rs`
  - Extracted oscillator parameter normalization helpers:
    - `normalize_oscillator_frequency(...)`
    - `normalize_oscillator_level(...)`
    - `normalize_phase(...)`
  - Extracted phase-advance helper (`advance_phase(...)`) for readability and consistency.
  - Replaced inline normalization/phase math with helper calls only (same ranges/fallbacks/wrap logic).

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Appended Tier 2 Batch 4 progress record.

### Bounded-Change Invariants

- [x] No behavior changes introduced.
- [x] No public API surface changes introduced.
- [x] No architecture changes introduced.
- [x] Cleanup remained local to selected Batch 4 files and helper extraction scope.
- [x] `dev-server/src/ws/mod.rs` was re-read for context and left unmodified.
- [x] No roadmap or archived spec files were modified.

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml --all` — **PASSED**
- `cargo fmt --manifest-path dev-server/Cargo.toml --all` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
  - Unit tests: **105 passed, 0 failed**
  - Integration tests: **18 passed, 0 failed**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
  - Unit/integration summary: **42 passed, 0 failed** (`37` unit + `3` audio status integration + `2` reload integration)
- `cargo xtask ci-check` — **PASSED**
  - Documentation: passed
  - Lint/type-check: passed
  - Engine + UI tests: passed

### Escalation Decision

- **Architect escalation: NO**
  - Reason: changes were strictly local cleanup/refactor actions with explicit scope and no ownership/architecture ambiguity.

---

## Tier 2 — Batch 5 quick-scan cleanup

### Scope

- Tier: **Tier 2 (quick scan)**
- Batch: **Batch 5**
- Goal: Bounded cleanup only (small helper extraction, duplication reduction, readability-only nested-flow simplification) with **no behavior/API/architecture changes**.

### Files Touched

- `cli/src/project/param_extract.rs`
  - Extracted subprocess spawn helper (`spawn_extraction_subprocess`) to dedupe process construction and error-context formatting.
  - Extracted stdout/stderr capture helper (`take_subprocess_pipes`) and buffered pipe-read helper (`read_subprocess_pipes`) to reduce repeated diagnostics plumbing.
  - Extracted timeout diagnostic formatter (`timeout_diagnostics`) to centralize long timeout guidance text while preserving wording and values.

- `cli/src/commands/start/reload_extractors.rs`
  - Extracted shared dylib find/copy logging flow into `prepare_temp_dylib(...)`.
  - Extracted shared temp-dylib extraction + cleanup flow into `load_from_temp_dylib(...)` used by both parameter and processor loaders.
  - Preserved temp cleanup semantics (best-effort `remove_file` after extraction attempt, including failure path).

- `dev-server/src/audio/server/input_pipeline.rs`
  - Extracted callback-guard helper (`callback_sample_count`) for sample-count gating.
  - Extracted pure channel-layout helpers (`deinterleave_input`, `interleave_stereo`) and ring-write helper (`push_samples_to_ring`) for readability.
  - Preserved callback order and preallocated-buffer usage; no new callback allocations/locks introduced.

- `dev-server/src/session.rs`
  - Extracted tiny watch/rebuild path helpers:
    - `log_rust_file_change(...)`
    - `report_rebuild_result(...)`
    - `handle_rust_files_changed(...)`
  - Simplified nested event-handler flow while preserving rebuild/panic reporting behavior and message text.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Appended Tier 2 Batch 5 progress record.

### Bounded-Change Invariants

- [x] No behavior changes introduced.
- [x] No public API surface changes introduced.
- [x] No architecture changes introduced.
- [x] Cleanup remained local to selected Batch 5 files and helper-extraction scope.
- [x] Real-time safety in audio callback paths preserved (no allocations/locks added in callback hot path).
- [x] No roadmap or archived spec files were modified.

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml --all` — **PASSED**
- `cargo fmt --manifest-path dev-server/Cargo.toml --all` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
  - Unit tests: **105 passed, 0 failed**
  - Integration tests: **18 passed, 0 failed**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
  - Unit/integration summary: **42 passed, 0 failed** (`37` unit + `3` audio status integration + `2` reload integration)
- `cargo xtask ci-check` — **PASSED**
  - Documentation: passed
  - Lint/type-check: passed
  - Engine + UI tests: passed

### Escalation Decision

- **Architect escalation: NO**
  - Reason: the cleanup boundaries were explicit and local; no ownership ambiguity or architectural tradeoff appeared while implementing Batch 5.

---

## Tier 2 — Batch 6 quick-scan cleanup

### Scope

- Tier: **Tier 2 (quick scan)**
- Batch: **Batch 6**
- Goal: Bounded cleanup only (small helper extraction, duplication reduction, readability-only guard/path handling cleanup) with **no behavior/API/architecture changes**.

### Files Touched

- `cli/src/project/ts_codegen.rs`
  - Extracted shared generated-directory creation helper (`ensure_generated_types_dir(...)`).
  - Extracted shared TypeScript file header writer (`append_generated_header(...)`).
  - Extracted write-if-changed helper (`write_generated_file_if_changed(...)`) and interface-line emission helper (`append_interface_entry(...)`).
  - Preserved generated output shape/order/escaping and reserved-marker checks.

- `cli/src/commands/bundle/metadata_refresh.rs`
  - Extracted sidecar usage decision helper (`decide_sidecar_usage(...)`) with explicit `SidecarDecision` states.
  - Extracted shared sidecar read helper (`read_sidecar_contents(...)`) to dedupe read-path error contexts.
  - Preserved stale-sidecar diagnostics and discovery-build fallback behavior.

- `cli/src/commands/bundle/ui_assets.rs`
  - Extracted repeated command-exit formatting and success checks (`format_exit_code(...)`, `ensure_command_success(...)`).
  - Extracted dependency path resolution helper (`resolve_dependency_path(...)`) for local/external dependency readability.
  - Preserved npm build/install and `cargo clean -p wavecraft-nih_plug` failure semantics/messages.

- `dev-server/src/audio/ffi_processor.rs`
  - Extracted callback guard helper (`process_dimensions(...)`) and pointer prep helper (`prepare_channel_ptrs(...)`).
  - Added behavior-preserving debug assertions for instance validity and channel-length consistency.
  - Preserved FFI call sequencing, stereo guard behavior, and real-time-safe stack-only pointer preparation.

- `docs/feature-specs/codebase-refactor-sweep/implementation-progress.md`
  - Appended Tier 2 Batch 6 progress record.

### Bounded-Change Invariants

- [x] No behavior changes introduced.
- [x] No public API surface changes introduced.
- [x] No architecture changes introduced.
- [x] Cleanup remained local to the Batch 6 target files (+ this ledger entry).
- [x] FFI behavior preserved (`create/process/set_sample_rate/reset/drop` dispatch unchanged).
- [x] Dev-server audio safety preserved (no new allocations/locks/blocking added in audio callback path).
- [x] No roadmap or archived spec files were modified.

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml --all` — **PASSED**
- `cargo fmt --manifest-path dev-server/Cargo.toml --all` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation

- **Architect escalation: NO**
  - Reason: all changes were bounded helper/readability refactors with explicit local scope and no ownership or architecture ambiguity.

---

## Tier 2 — Batch 8 quick-scan cleanup

### Scope

- Tier: **Tier 2 (quick scan)**
- Batch: **8**
- Goal: Bounded, behavior-preserving cleanup only (helper extraction, duplication reduction, naming clarity, logging helper reuse) with **no architecture moves and no cross-crate API changes**.

### Files

1. `cli/src/template/tsconfig_paths.rs`
   - Extracted shared JSONC scanning helpers (`skip_jsonc_whitespace_and_commas`, `skip_jsonc_comment`) used by both anchor and object-bound scans.
   - Extracted punctuation insertion helper (`comma_if_needed`) to dedupe comma decisions in anchor/fallback insertion paths.

2. `cli/src/template/dependency_rewrites.rs`
   - Added shared constants for Git URL regex and `dev-server` path segment.
   - Deduplicated dependency replacement assembly via helper extraction:
     - `package_attr_for_capture(...)`
     - `format_path_dependency(...)`

3. `cli/src/template/overrides.rs`
   - Extracted test-only fixture/helpers to reduce repeated setup and variable construction:
     - `make_vars(...)`
     - `setup_sdk_fixture(...)`
   - Preserved all test coverage intent and assertions.

4. `dev-server/src/audio/server/device_setup.rs`
   - Extracted repeated device/sample-rate logging helpers and stream-error callback logging helper:
     - `log_required_device_name(...)`
     - `log_device_name_or_fallback(...)`
     - `log_sample_rate(...)`
     - `log_stream_error(...)`
   - Tightened negotiated config naming clarity (`input_supported_config`, `output_supported_config`, `output_sample_rate`).

### Invariants

- [x] Behavior-preserving cleanup only.
- [x] No architecture moves or cross-crate API changes.
- [x] Edits bounded to Batch 8 files + this ledger update.
- [x] JSONC insertion/rewrite semantics preserved for template processing.
- [x] Dev-server audio runtime behavior preserved (device negotiation, stream build/callback wiring unchanged).

### Validation

- `cargo fmt --manifest-path cli/Cargo.toml --all` — **PASSED**
- `cargo fmt --manifest-path dev-server/Cargo.toml --all` — **PASSED**
- `cargo clippy --manifest-path cli/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo clippy --manifest-path dev-server/Cargo.toml --all-targets -- -D warnings` — **PASSED**
- `cargo test --manifest-path cli/Cargo.toml` — **PASSED**
- `cargo test --manifest-path dev-server/Cargo.toml` — **PASSED**
- `cargo xtask ci-check` — **PASSED**

### Escalation

- **Architect escalation: NO**
  - Rationale: changes stayed within explicit quick-scan helper/readability cleanup scope and did not introduce ownership or architecture ambiguity.

