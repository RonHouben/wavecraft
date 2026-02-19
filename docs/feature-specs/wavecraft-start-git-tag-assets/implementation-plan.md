# Implementation Plan: Durable fix for `wavecraft start` git-tag asset failures

## Overview

Generated projects can fail during `wavecraft start` metadata discovery because `wavecraft-nih_plug` compiles `include_dir!("$CARGO_MANIFEST_DIR/assets/ui-dist")`, and git checkout consumers may not have that directory in source artifacts.  
This plan fixes the issue durably in two layers:

1. **Decouple `_param-discovery` builds from UI/editor asset embedding paths** so metadata discovery does not depend on `ui-dist`.
2. **Enforce a distribution contract for git-tag consumers** so non-discovery builds remain self-contained and always ship valid embedded UI fallback assets.

It also adds CI/template coverage for the end-user path: **installed CLI + generated project (git-source dependency semantics) + `wavecraft start` startup path**.

## Problem statement (confirmed touchpoints)

- Compile panic source: `engine/crates/wavecraft-nih_plug/src/editor/assets.rs` (`include_dir!("$CARGO_MANIFEST_DIR/assets/ui-dist")`)
- `_param-discovery` build path: `cli/src/commands/start.rs` (`cargo build --lib --features _param-discovery`)
- Generated project feature declaration: `sdk-template/engine/Cargo.toml.template` (`_param-discovery = []`)
- Current template validation path does **not** cover git-source `wavecraft start` startup behavior:
  - `engine/xtask/src/commands/validate_template.rs`
  - `.github/workflows/template-validation.yml`

---

## Architecture/file touchpoints

### Core decoupling touchpoints

- `engine/crates/wavecraft-nih_plug/Cargo.toml`
- `engine/crates/wavecraft-nih_plug/src/lib.rs`
- `engine/crates/wavecraft-nih_plug/src/editor/mod.rs`
- `engine/crates/wavecraft-nih_plug/src/editor/assets.rs`
- `sdk-template/engine/Cargo.toml.template`

### Distribution contract touchpoints

- `.gitignore` (currently ignores `engine/crates/wavecraft-nih_plug/assets/ui-dist/`)
- `engine/crates/wavecraft-nih_plug/assets/ui-dist/**` (must become distributable contract assets)
- Optional guard point if needed: `engine/crates/wavecraft-nih_plug/build.rs` (or equivalent verification in tests/xtask)

### Validation/CI touchpoints

- `engine/xtask/src/commands/validate_template.rs`
- `.github/workflows/template-validation.yml`
- Optional focused regression tests in:
  - `cli/tests/` (integration smoke for git-source startup behavior)
  - `engine/crates/wavecraft-nih_plug/src/editor/assets.rs` tests

---

## Phased implementation steps

## Phase 1 — Decouple metadata discovery from editor/assets compilation

1. **Introduce/propagate dependency-level discovery feature in `wavecraft-nih_plug`**
   - File: `engine/crates/wavecraft-nih_plug/Cargo.toml`
   - Action: Add a crate feature (reuse `_param-discovery` naming for consistency) that disables editor/UI embedding compile paths for discovery-only builds.
   - Why: `_param-discovery` currently gates macro exports but not `wavecraft-nih_plug` editor asset compilation.
   - Risk: Medium (feature wiring across generated manifest and dependency rename).

2. **Gate editor module and editor exports behind non-discovery config**
   - Files:
     - `engine/crates/wavecraft-nih_plug/src/lib.rs`
     - `engine/crates/wavecraft-nih_plug/src/editor/mod.rs`
   - Action: Ensure discovery builds do not compile editor/webview/assets module tree at all.
   - Why: Removes compile-time dependence on embedded UI assets during metadata discovery.
   - Risk: Medium (must preserve plugin/editor API behavior for non-discovery builds).

3. **Forward generated project `_param-discovery` to dependency feature**
   - File: `sdk-template/engine/Cargo.toml.template`
   - Action: Change feature declaration from empty to forwarding form (e.g., `_param-discovery = ["wavecraft/_param-discovery"]` for renamed dependency).
   - Why: Ensures `cargo build --features _param-discovery` in generated projects affects dependency compilation.
   - Risk: Low/Medium (manifest syntax correctness + rename target).

4. **Regression compile checks**
   - Add tests/checks ensuring:
     - `_param-discovery` build no longer touches editor assets.
     - non-discovery build still compiles with editor available.
   - Risk: Low.

---

## Phase 2 — Enforce git-tag distribution contract for fallback UI assets

1. **Make fallback asset directory a tracked distribution artifact**
   - Files:
     - `.gitignore`
     - `engine/crates/wavecraft-nih_plug/assets/ui-dist/**`
   - Action:
     - Stop ignoring `assets/ui-dist`.
     - Commit minimal valid fallback payload (`index.html` + required static assets).
   - Why: Git-source consumers must receive self-contained fallback assets.
   - Risk: Medium (asset drift, repo size concerns).

2. **Define explicit contract checks**
   - Candidate files:
     - `engine/crates/wavecraft-nih_plug/src/editor/assets.rs` tests
     - or a new validation test in `cli/tests/` / xtask validation
   - Action: Add deterministic checks that required fallback files exist and are loadable.
   - Why: Prevent future regressions where assets vanish from published/tagged source.
   - Risk: Low.

3. **Document contract in code comments near embedding point**
   - File: `engine/crates/wavecraft-nih_plug/src/editor/assets.rs`
   - Action: Add concise contract note that non-discovery builds require distributable fallback assets.
   - Why: Makes ownership/expectation explicit for future maintainers.
   - Risk: Low.

---

## Phase 3 — Add CI/template validation for installed-CLI + git-source startup scenario

1. **Extend `validate-template` with git-source simulation path**
   - File: `engine/xtask/src/commands/validate_template.rs`
   - Action: Add a validation phase that exercises generated project with git-source dependency semantics (not only local-sdk/path mode).
   - Recommended approach:
     - Continue existing local-sdk validation for unreleased changes.
     - Add git-source simulation fixture and run metadata/startup smoke.
   - Why: Reproduces end-user failure class before release.
   - Risk: Medium (CI determinism, networking/tag availability).

2. **Add non-interactive `wavecraft start` smoke coverage**
   - File: `engine/xtask/src/commands/validate_template.rs` (and workflow if needed)
   - Action: Execute a bounded startup smoke (timeout-based) validating discovery build path succeeds and server startup reaches ready state.
   - Why: Directly covers the user-visible failing command, not just cargo build.
   - Risk: Medium (process lifecycle in CI).

3. **Wire the new phase into workflow**
   - File: `.github/workflows/template-validation.yml`
   - Action: Ensure workflow runs the new validate-template scenario(s) and fails on startup/compile panic signatures.
   - Why: Prevent regression merging.
   - Risk: Low/Medium (runtime increase).

---

## Testing & validation matrix

| Scenario              | Dependency mode      | Command                                         | Expected                                     |
| --------------------- | -------------------- | ----------------------------------------------- | -------------------------------------------- |
| Discovery compile     | git-source semantics | `cargo build --lib --features _param-discovery` | ✅ No `include_dir` panic; builds clean      |
| Start smoke           | git-source semantics | `wavecraft start` (bounded smoke)               | ✅ Reaches startup readiness; no asset panic |
| Normal build          | git-source semantics | `cargo build --lib` or bundle path              | ✅ Editor/assets embed path works            |
| Bundle path           | path dependency      | `wavecraft bundle`                              | ✅ Local UI staging still works              |
| Full local validation | SDK repo             | `cargo xtask ci-check --full`                   | ✅ includes updated template validation      |

---

## Risks & mitigations

- **Risk:** Feature propagation mistakes break discovery or normal builds.  
  **Mitigation:** Add explicit dual-mode compile checks (`_param-discovery` vs default).

- **Risk:** Asset directory drift (stale/outdated fallback files).  
  **Mitigation:** Add contract test + optional sync/verification workflow step.

- **Risk:** CI flakiness in startup smoke tests.  
  **Mitigation:** Use deterministic ports, bounded timeout, and explicit success log markers.

- **Risk:** Over-coupling template validation to unreleased git tags.  
  **Mitigation:** Keep local-sdk validation; add git-source simulation without external tag dependency.

---

## Acceptance criteria

- [ ] `_param-discovery` builds no longer depend on `wavecraft-nih_plug` editor/UI asset embedding paths.
- [ ] `wavecraft start` no longer fails with compile-time `include_dir` panic in generated projects using git-source dependency semantics.
- [ ] Non-discovery builds remain self-contained (fallback UI assets are present and embedded for git-tag consumers).
- [ ] CI/template validation explicitly covers installed-CLI-equivalent generated project startup path and fails on regression.
- [ ] `cargo xtask ci-check --full` passes with new validation included.

---

## Rollback strategy

If regressions appear after merge:

1. **Fast rollback:** revert feature-forwarding changes in `sdk-template/engine/Cargo.toml.template` and related `wavecraft-nih_plug` gating commits as one unit.
2. **Safety preserve:** keep asset distribution contract commits (tracked fallback assets + checks) if they are independently safe.
3. **CI rollback:** temporarily gate new startup smoke phase behind a flag in `validate-template` if instability is CI-only, while maintaining compile-level regression checks.
4. **Post-rollback triage:** capture failing matrix row(s), reintroduce fix in smaller increments (feature wiring first, then CI smoke).

---

## Coder handoff (explicit task list)

1. Implement Phase 1 feature propagation and compile gating across:
   - `wavecraft-nih_plug` feature config + module gating
   - template `_param-discovery` forwarding
2. Implement Phase 2 asset distribution contract:
   - unignore + track fallback assets
   - add asset contract tests/checks
3. Implement Phase 3 validation expansion:
   - update `validate-template` command
   - update `template-validation.yml`
4. Run and record:
   - targeted crate tests
   - template validation
   - `cargo xtask ci-check --full`
5. Update feature progress doc with evidence/log excerpts for each matrix row.

---

## Tester handoff (explicit task list)

1. Execute validation matrix end-to-end on the implementation branch.
2. Reproduce previous failure signature and confirm it is absent:
   - no `include_dir` panic during discovery/start path.
3. Validate both dependency modes:
   - local-sdk/path mode
   - git-source simulation mode
4. Run regression suite:
   - `cargo xtask ci-check --full`
   - template workflow-equivalent checks
5. Produce `test-plan.md` with:
   - commands run
   - observed outputs
   - pass/fail per matrix row
   - any flaky behavior notes and rerun data.

---

## Out-of-scope (for this fix)

- Roadmap updates (PO-owned).
- Architectural redesign of editor/webview subsystem.
- Changes to archived feature spec files.
