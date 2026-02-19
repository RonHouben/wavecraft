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
