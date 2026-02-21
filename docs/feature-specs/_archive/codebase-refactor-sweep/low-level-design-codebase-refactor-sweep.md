# Low-Level Design: Codebase Refactor Sweep (Milestone 19)

## Related Documents

- [User Stories](./user-stories.md)
- [Roadmap — Milestone 19](../../roadmap.md)
- [Backlog](../../backlog.md)
- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards (Hub)](../../architecture/coding-standards.md)
- [Rust Standards](../../architecture/coding-standards-rust.md)
- [TypeScript/React Standards](../../architecture/coding-standards-typescript.md)
- [Testing & Quality Standards](../../architecture/coding-standards-testing.md)
- [Development Workflows](../../architecture/development-workflows.md)

---

## 1. Purpose and Design Intent

Milestone 19 is a **behavior-preserving refactor sweep** over the existing Wavecraft codebase, delivered in a **single mega-PR**, with two explicit outcomes:

1. Reduce structural complexity and file size in Tier 1 hotspots (target: no Tier 1 file > 400 lines excluding tests).
2. Convert recurring refactor findings into enforceable coding standards updates (lessons-learned pipeline).

This design defines execution architecture, file-level decomposition strategy, invariants, risk controls, and handoff constraints for Planner/Coder.

---

## 2. Scope Boundaries and Non-Goals

### In-scope

- Tier 1 deep decomposition of 8 hotspot files listed in M19.
- Tier 2 quick-scan cleanup across medium files (naming, dead code, obvious extraction, error handling consistency).
- Tier 3 automated lint/format pass for remaining files (and final stabilization).
- QoL improvements only when non-breaking (error clarity, naming clarity, public API comments).
- Lessons-learned capture and mapping to standards updates proposals.

### Explicit non-goals

- New product features/capabilities.
- New public IPC methods or protocol surface changes.
- New crates/packages or macro DSL expansion.
- Runtime behavior changes in DSP/IPC/CLI semantics (except approved QoL wording improvements).
- Performance optimization projects beyond incidental cleanup.
- Roadmap edits (PO-owned), archive edits under `docs/feature-specs/_archive/**`.

---

## 3. Tiered Execution Architecture

## 3.1 Execution phases

1. **Baseline stabilization gate (pre-tier check)**  
   Run `cargo xtask ci-check` (no functional edits yet) to establish a clean baseline snapshot before structural churn.

2. **Tier 1 — Deep Refactor (hotspots only)**  
   Decompose each hotspot file into cohesive modules with explicit responsibilities and doc-commented module purpose.

3. **Tier 2 — Quick Scan**  
   Bounded cleanup on medium files; no deep architecture rewrites discovered here. If deep issue appears, record for future milestone.

4. **QoL pass (bounded)**  
   Apply non-breaking ergonomics improvements discovered during Tier 1/2 (error guidance, naming clarity, doc comments).

5. **Tier 3 — Automated lint/format sweep (final step)**  
   Run formatting/lint autofixes at end to avoid review noise during structural edits.

6. **Lessons Learned distillation**  
   Produce `lessons-learned.md` + standards update proposals with rationale and enforcement route.

## 3.2 Architecture constraints across all tiers

- Tier transitions are gated by green validation checkpoints.
- Tier 3 is terminal cleanup; avoid early full autofix churn that obscures Tier 1 review intent.
- Single PR, but commit topology must remain tier-structured and reviewable.

---

## 4. Tier 1 Module Decomposition Strategy (file-by-file)

For each hotspot, decomposition stays within existing crate/package boundaries and preserves public API behavior.

## 4.1 `cli/src/commands/start.rs` (1,640 lines)

### Responsibility clusters identified

- command orchestration (`StartCommand::execute`)
- tsconfig path injection and JSONC manipulation
- audio startup classification + status mapping
- metadata sidecar cache/staleness logic
- process orchestration (dev servers, shutdown, child lifecycle)
- test block

### Target decomposition

- `cli/src/commands/start/mod.rs` — orchestration entrypoint (`StartCommand`, `execute`)
- `cli/src/commands/start/tsconfig_paths.rs` — JSONC anchor detection/injection helpers
- `cli/src/commands/start/audio_status.rs` — audio startup classification/status conversion
- `cli/src/commands/start/metadata_cache.rs` — sidecar paths/read/write/staleness checks
- `cli/src/commands/start/process_control.rs` — port checks, server spawn, shutdown/kill handling
- `cli/src/commands/start/tests/*.rs` or retained inline tests split by concern

### Notes

- Keep `run_dev_servers` flow deterministic; avoid semantic reorder in startup contract.
- Preserve fail-fast diagnostics for runtime loader/audio startup failures.

---

## 4.2 `cli/src/commands/bundle_command.rs` (1,140 lines)

### Responsibility clusters identified

- bundle orchestration
- metadata/type refresh and sidecar staleness detection
- UI build/staging/sync
- dependency mode detection (`wavecraft-nih_plug` mode)
- install/copy filesystem ops abstraction
- project root resolution
- tests

### Target decomposition

- `cli/src/commands/bundle/mod.rs` — `BundleCommand` orchestration
- `cli/src/commands/bundle/metadata_refresh.rs` — sidecar freshness/discovery/type refresh
- `cli/src/commands/bundle/ui_assets.rs` — UI build, staging, sync into nih-plug assets
- `cli/src/commands/bundle/dependency_mode.rs` — dependency mode detection
- `cli/src/commands/bundle/install.rs` — bundle path resolution + install routines
- `cli/src/commands/bundle/fs_ops.rs` — `FileSystemOps`, real + mock implementations
- `cli/src/commands/bundle/project_root.rs` — root detection and errors

### Notes

- Keep install semantics unchanged (especially diagnostics and fallback path behavior).
- Preserve test doubles around filesystem boundaries.

---

## 4.3 `engine/crates/wavecraft-macros/src/plugin.rs` (1,016 lines)

### Responsibility clusters identified

- macro input parsing (`PluginDef`, signal chain parsing)
- naming/id generation utilities
- expansion/token generation
- generated runtime parameter mapping and plugin impl blocks
- FFI/vtable related generated sections
- tests

### Target decomposition

- `.../plugin/mod.rs` — public macro entrypoint (`wavecraft_plugin_impl`)
- `.../plugin/parse.rs` — `PluginDef` parse + signal chain parse
- `.../plugin/naming.rs` — snake_case/id/type prefix utilities
- `.../plugin/metadata.rs` — processor metadata derivation helpers
- `.../plugin/codegen.rs` — `expand_wavecraft_plugin` token assembly
- `.../plugin/runtime_params.rs` — generated runtime param map/token helpers
- `.../plugin/tests.rs` — parse/id/codegen expectations

### Notes

- **Critical promoted backlog item** lives here; keep generated contracts stable.
- Preserve RT-safe generated process path constraints and current ABI expectations.

---

## 4.4 `cli/src/template/mod.rs` (992 lines)

### Responsibility clusters identified

- template extraction traversal
- variable substitution + local dev overrides
- tsconfig path injection utilities (duplicated family with start command)
- path/dependency rewrite helpers
- tests

### Target decomposition

- `cli/src/template/mod.rs` — public entry + high-level flow
- `cli/src/template/extract.rs` — directory/file extraction traversal
- `cli/src/template/overrides.rs` — local-dev override logic (Cargo/npm/path rewrites)
- `cli/src/template/tsconfig_paths.rs` — JSONC path injection helpers
- `cli/src/template/dependency_rewrites.rs` — focused dependency rewrite helpers
- tests split by extraction/override/tsconfig concerns

### Notes

- Prefer shared helper extraction with `start` JSONC logic only if it does not create coupling confusion; otherwise duplicate small helper intentionally with clear docs.

---

## 4.5 `dev-server/src/audio/server.rs` (923 lines)

### Responsibility clusters identified

- audio server object lifecycle/start flow
- device/config selection and stream setup
- callback processing path (including output modifiers)
- param bridge reads and metering utilities
- oscillator/gain modifier logic
- tests

### Target decomposition

- `dev-server/src/audio/server/mod.rs` — `AudioServer` orchestration API
- `dev-server/src/audio/server/device_setup.rs` — device/config negotiation and stream construction
- `dev-server/src/audio/server/callbacks.rs` — input/output callback wiring
- `dev-server/src/audio/server/output_modifiers.rs` — oscillator/gain transform logic
- `dev-server/src/audio/server/metering.rs` — peak/rms helpers and meter packaging
- `dev-server/src/audio/server/params.rs` — parameter bridge read helpers
- tests split by modifier behavior and startup behavior

### Notes

- RT-safety invariant is highest priority: no allocations/locks/syscalls in callback hot path.
- Any abstraction must not introduce heap allocs in per-buffer processing.

---

## 4.6 `engine/crates/wavecraft-protocol/src/ipc.rs` (746 lines)

### Responsibility clusters identified

- core JSON-RPC envelope types and constructors
- method-specific params/results/notifications
- audio status and diagnostic contract types
- meter/oscilloscope payload types
- error helpers and tests

### Target decomposition

- `.../ipc/mod.rs` — public re-exports and envelope constructors
- `.../ipc/envelope.rs` — `IpcRequest`, `IpcResponse`, `IpcNotification`, `RequestId`
- `.../ipc/errors.rs` — `IpcError` + helper constructors
- `.../ipc/parameters.rs` — parameter request/result/info types
- `.../ipc/metering.rs` — meter + oscilloscope types
- `.../ipc/audio_status.rs` — `AudioRuntimePhase`, diagnostics, status types
- `.../ipc/resize_audio.rs` — request-resize/register-audio contract structs
- tests grouped by contract family

### Notes

- No wire-format changes: serialized field names and optionality must remain backward-compatible for current consumers.

---

## 4.7 `cli/src/commands/update.rs` (669 lines)

### Responsibility clusters identified

- command orchestration
- self-update execution + re-exec logic
- install progress stream parsing
- project detection + dependency updates
- summary determination/output
- tests

### Target decomposition

- `cli/src/commands/update/mod.rs` — `run` orchestration
- `cli/src/commands/update/self_update.rs` — update CLI, reexec, version detection
- `cli/src/commands/update/progress.rs` — stderr stream parsing + phase detection
- `cli/src/commands/update/project_update.rs` — project detection + rust/npm updates
- `cli/src/commands/update/summary.rs` — outcome matrix + user-facing summary
- tests split by progress parsing, summary rules, project detection

### Notes

- Preserve current UX improvements already shipped pre-M19 (single invocation behavior + split phase messaging).

---

## 4.8 `engine/crates/wavecraft-nih_plug/src/editor/windows.rs` (598 lines)

### Responsibility clusters identified

- windows webview handle implementation
- WebView2 initialization/configuration/message plumbing
- JS bridge script + content loading
- runtime checks/message pumping/fallback content

### Target decomposition

- `.../editor/windows/mod.rs` — `WindowsWebView`, creation entrypoint
- `.../editor/windows/webview2_init.rs` — COM/WebView2 environment/controller init
- `.../editor/windows/ipc_bridge.rs` — message handler + script injection + primitives
- `.../editor/windows/content.rs` — UI loading and fallback HTML
- `.../editor/windows/runtime_checks.rs` — runtime availability and readiness pump

### Cross-platform abstraction opportunity (bounded)

Introduce/expand shared editor-internal modules for duplicated concepts, without forcing platform unification:

- `.../editor/ipc_handler.rs` — shared `JsonIpcHandler` trait/adapter
- `.../editor/ipc_primitives.rs` — shared constants/snippets for JS bridge primitives
- keep platform-specific lifecycle and webview APIs isolated in `macos`/`windows`.

---

## 5. Refactor Invariants (must hold)

## 5.1 Behavior preservation invariants

- CLI command semantics (exit codes, core workflow, required side effects) remain unchanged.
- IPC protocol wire contract remains unchanged (field names, result/error shape, notification semantics).
- Macro-generated plugin behavior and exported contract remain unchanged unless explicitly marked QoL text-only.
- No functional regression in startup/build/bundle/update flows.

## 5.2 Real-time safety invariants

- No new locks/allocations/blocking I/O in audio callback paths.
- Preserve lock-free/atomic handoff patterns in audio-sensitive code.
- No additional logging or expensive string formatting in real-time hot loops.

## 5.3 API compatibility invariants

- No breaking changes to public Rust crate APIs from this milestone.
- No breaking changes to TypeScript package exports in this milestone.
- No new IPC methods introduced as part of refactor-only work.

## 5.4 QoL boundary invariant

- QoL changes are additive and non-breaking (diagnostics clarity, naming clarity, doc comment clarity).
- Any behavioral change requires explicit "out-of-scope" escalation, not silent inclusion.

---

## 6. Testing and Validation Strategy

## 6.1 Validation cadence

- **Baseline:** `cargo xtask ci-check` before Tier 1 begins.
- **Tier 1:** run `cargo xtask ci-check` after each hotspot file decomposition completes.
- **Tier 2:** run `cargo xtask ci-check` every 3–5 medium-file batches (or per package boundary).
- **QoL pass:** targeted tests + one full `ci-check`.
- **Tier 3 final:** lint/format sweep + final `cargo xtask ci-check` clean run.
- If template/CLI-sensitive areas are touched significantly, run `cargo xtask ci-check --full` at least once before PR finalization.

## 6.2 Required checks

- Rust: `cargo fmt`, `cargo clippy` (warnings as errors policy).
- TS: ESLint + Prettier + type-check path via `ci-check` lint phase.
- Automated tests: engine + UI tests via `ci-check`.
- No new lint suppressions without justification.

## 6.3 Test structure expectations

- Keep/refactor tests alongside extracted modules to preserve behavior lock-in.
- Where extraction creates pure helpers, add unit tests that mirror existing behavior assertions.
- Do not weaken test coverage as a side effect of decomposition.

---

## 7. Single Mega-PR Risk Controls

## 7.1 Commit topology (inside one PR)

- Commit groups by tier and by hotspot file (one logical concern per commit).
- Avoid mixing Tier 1 structural moves with Tier 3 auto-format in same commit.
- Keep renames/moves isolated from logic touch where possible to improve diff review.

## 7.2 Merge risk mitigation

- Freeze scope to M19 list; newly discovered deep issues in Tier 2 become deferred follow-ups.
- Require green `ci-check` at every tier gate before proceeding.
- Keep a running "refactor ledger" in PR description mapping commits → files → invariants checked.

## 7.3 Reviewability controls

- PR sections mirror tiers: Tier 1, Tier 2, QoL, Tier 3, Lessons.
- For each Tier 1 hotspot, include:
  - old file responsibilities
  - new module map
  - behavior checks run
  - known no-op rationale (if any extracted module is thin wrapper)

---

## 8. Lessons-Learned Capture Pipeline

## 8.1 Data capture during implementation

Create and maintain:

- `docs/feature-specs/codebase-refactor-sweep/lessons-learned.md`

Track findings continuously during tiers, not as post-hoc memory reconstruction.

Each entry format:

- **Pattern type:** anti-pattern / effective abstraction / naming gap / testing gap / error-handling gap
- **Evidence:** file(s), before/after snippet references
- **Impact:** maintenance/readability/defect-risk
- **Proposed guideline:** concrete rule text
- **Enforcement:** lint rule / clippy lint / ESLint rule / PR checklist item / test template

## 8.2 Mapping findings to standards docs

- Cross-cutting rules → `docs/architecture/coding-standards.md`
- Rust-specific rules → `docs/architecture/coding-standards-rust.md`
- TS/React-specific rules → `docs/architecture/coding-standards-typescript.md`
- Test/lint cadence + quality gates → `docs/architecture/coding-standards-testing.md`

## 8.3 Acceptance for lessons output

- Minimum top 5 high-impact patterns with enforceable updates.
- No vague advice; each update must include "why" and "how enforced".
- DocWriter applies approved standards updates in follow-up doc PR/work.

---

## 9. Planner Handoff Guidance (work breakdown + sequencing constraints)

## 9.1 Required work breakdown order

1. Baseline gate (ci-check green).
2. Tier 1 decomposition (8 hotspots) in dependency-aware order:
   - `start.rs`
   - `template/mod.rs`
   - `bundle_command.rs`
   - `update.rs`
   - `wavecraft-macros/plugin.rs`
   - `wavecraft-protocol/ipc.rs`
   - `dev-server/audio/server.rs`
   - `nih_plug/editor/windows.rs`
3. Tier 2 quick scan batches by package/crate.
4. QoL bounded pass.
5. Tier 3 final lint/format sweep.
6. Lessons-learned distillation and standards proposals.

## 9.2 Sequencing constraints

- Run `ci-check` between Tier 1 files; do not stack multiple unvalidated deep refactors.
- Do not run global autofix/format sweep before Tier 1 review checkpoints.
- Keep protocol/macro refactors behavior-locked with serialization/generation tests.
- Treat audio server refactor as RT-sensitive: isolate and validate separately.
- Windows editor refactor should coordinate with macOS counterpart only via shared narrow abstractions; avoid cross-platform rewrite.
- **Coder must consult the Architect before continuing implementation whenever a decomposition boundary, module responsibility, or behavior-preservation decision is ambiguous.**

## 9.3 Planner deliverable expectations

Implementation plan should include per-file tasks with:

- target module map
- invariants checklist
- validation command cadence
- explicit "stop conditions" for scope creep
- lessons-learned capture checkpoints after each tier
- architect consultation checkpoint: note any decision that was ambiguous and record that Architect guidance was obtained before proceeding

---

## 10. Definition of Done (architecture perspective)

- All Tier 1 hotspot files decomposed to ≤400 lines each (excluding tests).
- Tier 2 cleanup complete without scope creep into new deep refactors.
- Tier 3 lint/format clean and final `cargo xtask ci-check` passing.
- No behavior regressions against existing tests and contracts.
- `lessons-learned.md` produced with enforceable standards proposals.
- Planner-ready execution sequence documented and consumable.

---

## 11. Open Decisions for Planner

1. Confirm preferred Tier 1 execution order among CLI-heavy files (`start` vs `template` first) based on expected conflict surface.
2. Decide whether shared JSONC helper extraction between `start` and `template` is worth coupling risk vs intentional duplication.
3. Define exact Tier 2 batch size (recommended 3–5 files/batch with ci-check gate).
4. Decide whether one mid-sweep `ci-check --full` is mandatory or optional based on touched template/CLI surface.
