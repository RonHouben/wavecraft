# Implementation Plan: Milestone 19 — Codebase Refactor Sweep

## Overview

Milestone 19 is a behavior-preserving, single-PR refactor sweep focused on reducing structural complexity, improving maintainability, and codifying recurring quality findings into coding standards updates.  
Execution follows tiered delivery: Tier 1 deep refactors on 8 hotspot files, Tier 2 bounded cleanup on medium files, Tier 3 terminal lint/format sweep, plus lessons-learned outputs.

## Scope, Constraints, and Invariants

### In scope

- Tier 1 decomposition of the 8 hotspot files listed in roadmap M19.
- Tier 2 quick-scan cleanup on medium files (~200–500 lines).
- Tier 3 automated lint/format pass.
- Minor QoL improvements only (diagnostic clarity, naming clarity, doc comments).
- Lessons-learned artifact + standards-update mapping proposals.

### Out of scope

- New features/capabilities.
- New IPC methods / protocol surface expansion.
- New crates/packages / major architecture changes.
- Runtime behavior changes beyond approved QoL wording-level improvements.
- Roadmap/backlog edits.
- Any edits under `docs/feature-specs/_archive/**`.

### Non-negotiable invariants

- Behavior-preserving refactor scope.
- No Tier 1 file >400 lines (excluding tests) at completion.
- IPC wire format compatibility preserved.
- Real-time safety preserved in audio callback paths.
- Public API compatibility preserved (Rust + TypeScript exports).

### Escalation policy (mandatory)

**If implementation/refactor decisions are unclear, the Coder must ask Architect before proceeding.**

---

## Phase-by-phase execution (by tier)

## Phase 0 — Baseline & guardrails gate

1. Run baseline validation:
   - `cargo xtask ci-check`
2. Capture baseline state in implementation progress notes:
   - current failures (if any),
   - known flaky areas,
   - starting file sizes for Tier 1 files.
3. Create/update sweep ledger (in feature folder progress notes):
   - file touched,
   - invariant checks performed,
   - ci-check status after each gate.

**Exit gate:** baseline green or documented pre-existing failures with explicit non-regression tracking.

---

## Phase 1 — Tier 1 deep refactor (hotspots, sequenced)

### Sequencing order (dependency-aware, conflict-minimizing)

1. `cli/src/commands/start.rs`
2. `cli/src/template/mod.rs`
3. `cli/src/commands/bundle_command.rs`
4. `cli/src/commands/update.rs`
5. `engine/crates/wavecraft-macros/src/plugin.rs`
6. `engine/crates/wavecraft-protocol/src/ipc.rs`
7. `dev-server/src/audio/server.rs`
8. `engine/crates/wavecraft-nih_plug/src/editor/windows.rs`

### Tier 1 per-file task breakdown

#### 1) `cli/src/commands/start.rs` (target: module decomposition + deterministic startup behavior)

- Create `cli/src/commands/start/` module family:
  - `mod.rs` (orchestration entry),
  - `tsconfig_paths.rs`,
  - `audio_status.rs`,
  - `metadata_cache.rs`,
  - `process_control.rs`,
  - tests split by concern.
- Preserve startup ordering and fail-fast diagnostics.
- Keep sidecar cache behavior unchanged.

**Validation checkpoint:** `cargo xtask ci-check` immediately after this file family is stable.

---

#### 2) `cli/src/template/mod.rs` (target: extraction + clear override boundaries)

- Decompose into:
  - `extract.rs`,
  - `overrides.rs`,
  - `tsconfig_paths.rs`,
  - `dependency_rewrites.rs`,
  - lean `mod.rs`.
- Decide JSONC helper sharing with `start` only if coupling remains clear; otherwise prefer intentional local helper duplication with docs.
- Preserve template output behavior exactly.

**Validation checkpoint:** `cargo xtask ci-check`.

---

#### 3) `cli/src/commands/bundle_command.rs` (target: orchestration vs filesystem/test seams)

- Decompose into:
  - `bundle/mod.rs`,
  - `metadata_refresh.rs`,
  - `ui_assets.rs`,
  - `dependency_mode.rs`,
  - `install.rs`,
  - `fs_ops.rs`,
  - `project_root.rs`.
- Preserve install semantics, diagnostics, and FS test doubles behavior.

**Validation checkpoint:** `cargo xtask ci-check`.

---

#### 4) `cli/src/commands/update.rs` (target: self-update flow clarity + summary logic separation)

- Decompose into:
  - `update/mod.rs`,
  - `self_update.rs`,
  - `progress.rs`,
  - `project_update.rs`,
  - `summary.rs`.
- Preserve one-invocation update behavior and progress UX semantics.

**Validation checkpoint:** `cargo xtask ci-check`.

---

#### 5) `engine/crates/wavecraft-macros/src/plugin.rs` (target: parse/codegen/runtime-params separation)

- Decompose into:
  - `plugin/mod.rs`,
  - `parse.rs`,
  - `naming.rs`,
  - `metadata.rs`,
  - `codegen.rs`,
  - `runtime_params.rs`,
  - `tests.rs`.
- Protect macro output compatibility and generated contract stability.

**Validation checkpoint:** `cargo xtask ci-check`.

---

#### 6) `engine/crates/wavecraft-protocol/src/ipc.rs` (target: contract families split without wire change)

- Decompose into:
  - `ipc/mod.rs`,
  - `envelope.rs`,
  - `errors.rs`,
  - `parameters.rs`,
  - `metering.rs`,
  - `audio_status.rs`,
  - `resize_audio.rs`,
  - grouped tests.
- Preserve serialized field names/optionality and existing JSON-RPC contract behavior.

**Validation checkpoint:** `cargo xtask ci-check`.

---

#### 7) `dev-server/src/audio/server.rs` (target: RT-safe decomposition)

- Decompose into:
  - `server/mod.rs`,
  - `device_setup.rs`,
  - `callbacks.rs`,
  - `output_modifiers.rs`,
  - `metering.rs`,
  - `params.rs`,
  - focused tests.
- Enforce RT-safety: no new allocations/locks/blocking calls in callback hot path.

**Validation checkpoint:** `cargo xtask ci-check` + targeted audio tests as needed.

---

#### 8) `engine/crates/wavecraft-nih_plug/src/editor/windows.rs` (target: platform modularization)

- Decompose into:
  - `windows/mod.rs`,
  - `webview2_init.rs`,
  - `ipc_bridge.rs`,
  - `content.rs`,
  - `runtime_checks.rs`.
- Optionally extract narrow shared editor internals (`ipc_handler.rs`, `ipc_primitives.rs`) only if no platform lifecycle coupling leak.
- Avoid cross-platform rewrite.

**Validation checkpoint:** `cargo xtask ci-check`.

---

### Phase 1 stop conditions

Stop Tier 1 immediately and escalate if any of these occur:

- Proposed change alters runtime behavior/contract (not wording-only QoL).
- Ambiguous module boundary with risk to invariants.
- CI turns red in a way not attributable to touched hotspot.
- Need to add new crate/package/API to continue.
- RT-safety uncertainty in audio callback path.

---

## Phase 2 — Tier 2 quick scan (bounded medium-file cleanup)

### Batch strategy

- Process medium files in batches of 3–5 files, grouped by crate/package.
- Per batch, do only:
  - naming consistency,
  - dead code/unused imports cleanup,
  - obvious helper extraction,
  - error-message consistency improvements.
- Defer any deep redesign discovered to follow-up items (do not absorb into M19).

### Validation cadence (Tier 2)

- Run `cargo xtask ci-check` after each 3–5 file batch or package boundary, whichever comes first.

### Tier 2 stop conditions (scope-creep guardrails)

- Any extraction needing cross-package architecture changes.
- Any change that requires behavior reinterpretation.
- Any medium-file cleanup crossing into >1 day redesign effort.
- Any proposed API surface change.

When triggered: record in sweep ledger as "deferred follow-up candidate."

---

## Phase 3 — QoL bounded pass

- Apply non-breaking improvements found during Tier 1/2:
  - actionable CLI error wording,
  - clearer naming (no semantic change),
  - public API doc comment clarity.
- No workflow or contract changes allowed.

**Validation checkpoint:** `cargo xtask ci-check`.

---

## Phase 4 — Tier 3 terminal lint/format sweep

- Run formatting/lint auto-fixes after structural refactors are done:
  - Rust: `cargo fmt`, clippy path through `cargo xtask ci-check`
  - TS: ESLint + Prettier path through `cargo xtask ci-check`
- Keep this phase separate from Tier 1 logic commits.

**Validation checkpoints:**

1. `cargo xtask ci-check`
2. Final pre-PR hardening: `cargo xtask ci-check --full` (at least once before PR finalization).

---

## Phase 5 — Lessons learned + coding standards mapping

Create `docs/feature-specs/codebase-refactor-sweep/lessons-learned.md` during execution (not post-hoc only), then finalize at end.

### Required checkpoint moments

- After Tier 1 completion.
- Mid Tier 2.
- Post Tier 3 final pass.

### Entry template (each lesson)

- Pattern type (anti-pattern / effective abstraction / naming gap / testing/lint gap / error-handling gap)
- Evidence (file references)
- Why it mattered
- Proposed coding-standard rule text
- Enforcement method (lint, clippy/eslint rule, checklist, test template)

### Required mapping to coding standards docs

- Cross-cutting rules → `docs/architecture/coding-standards.md`
- Rust-specific → `docs/architecture/coding-standards-rust.md`
- TypeScript-specific → `docs/architecture/coding-standards-typescript.md`
- Validation/lint/testing cadence → `docs/architecture/coding-standards-testing.md`

---

## Validation cadence (explicit schedule)

1. Baseline: `cargo xtask ci-check`
2. Tier 1: after each hotspot file decomposition (8 runs)
3. Tier 2: every 3–5 files / per package boundary
4. Post QoL pass: one run
5. Post Tier 3: one run
6. Final pre-PR gate: `cargo xtask ci-check --full` (mandatory once)

If failures occur: resolve before advancing to next tier gate.

---

## Commit topology guidance (single mega-PR)

Use one PR with tightly structured commits to keep reviewable diffs.

### Recommended commit groups

- `C00`: baseline/no-op verification notes (optional, no broad formatting churn)
- `C01–C08`: one commit group per Tier 1 hotspot (structural changes + local tests only)
- `C09`: Tier 2 batch commits (group by package/crate)
- `C10`: QoL-only commit(s)
- `C11`: Tier 3 formatting/lint-only commits
- `C12`: lessons-learned artifact + standards proposals references

### Commit hygiene rules

- Do not mix Tier 1 decomposition with global formatting.
- Keep file moves/renames separate from behavior-adjacent edits where possible.
- Commit messages must state:
  - file family touched,
  - invariant checks performed,
  - ci-check status.

---

## Architect escalation policy (operational)

Escalate to Architect immediately when any of the following is true:

- Module responsibility boundaries are ambiguous.
- Behavior-preserving intent is uncertain.
- Shared abstraction vs duplication tradeoff is unclear.
- Cross-platform abstraction decision might alter lifecycle behavior.
- RT-safety impact is uncertain.

Mandatory recorded statement in progress notes for each escalation:

- question asked,
- Architect guidance received,
- implementation decision taken.

---

## Stop conditions / scope-creep guardrails

Stop and defer to follow-up issue if:

1. Refactor requires new product behavior.
2. Refactor requires new public API/protocol.
3. Refactor requires new crate/package introduction.
4. Refactor introduces significant cross-platform redesign beyond targeted extraction.
5. Any "quick scan" item grows into deep architectural work.

Rule: "When in doubt, defer and document."

---

## Risks and mitigations

- **Risk:** Behavioral regression during decomposition  
  **Mitigation:** ci-check gate after each Tier 1 file; keep test coverage intact.
- **Risk:** Mega-PR review complexity  
  **Mitigation:** strict commit topology and tier-labeled PR sections.
- **Risk:** RT-safety regressions in audio paths  
  **Mitigation:** isolate audio refactor, targeted validation, Architect escalation on uncertainty.
- **Risk:** Scope creep in Tier 2  
  **Mitigation:** explicit stop conditions and defer list.

---

## Definition of Done / exit criteria

- [ ] All 8 Tier 1 files decomposed to ≤400 lines (excluding tests)
- [ ] Each extracted module has clear purpose documentation
- [ ] Tier 2 cleanup complete within bounded scope
- [ ] Tier 3 lint/format complete with no unjustified suppressions
- [ ] QoL improvements are non-breaking and behavior-preserving
- [ ] `cargo xtask ci-check` passes at all required checkpoints
- [ ] `cargo xtask ci-check --full` passes once before PR finalization
- [ ] `lessons-learned.md` created with top 5+ enforceable patterns
- [ ] Standards mapping proposals prepared for `coding-standards*.md`
- [ ] Architect escalations recorded where ambiguity occurred
- [ ] No roadmap/backlog edits; no archive edits

---

## Coder handoff checklist

1. Confirm baseline green.
2. Execute Tier 1 in specified order.
3. Run and record ci-check after each hotspot.
4. Keep a defer list for out-of-scope discoveries.
5. Keep lessons-learned log updated per checkpoint.
6. Apply Tier 2 bounded batches.
7. Run QoL pass.
8. Run terminal Tier 3 lint/format.
9. Run final `cargo xtask ci-check --full`.
10. Prepare PR with tier-structured commit narrative.
