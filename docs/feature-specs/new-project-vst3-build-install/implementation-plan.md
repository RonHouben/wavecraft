# Implementation Plan: Remove `sdk-template/engine/xtask`

## Overview

This plan removes `sdk-template/engine/xtask` entirely from newly generated projects and moves bundle/install orchestration responsibility to the Wavecraft CLI. The change is delivered as **one cohesive implementation** (no split PRs), with **no backward-compatibility guarantees** for older generated templates or workflows.

Target end state:

- Generated projects do **not** contain `engine/xtask`.
- `wavecraft start --install` and related CLI flows own build/bundle/install behavior for generated plugins.
- CI and docs are updated to reflect the new CLI-owned workflow.

## Scope and Non-Goals

### In scope

- Remove `sdk-template/engine/xtask` from template source.
- Update template workspace/manifests/scripts that currently depend on template-local xtask behavior.
- Ensure CLI commands (`create`, `start`, and any install/bundle path touched by generated projects) perform required orchestration.
- Update tests and CI checks for the new flow.
- Update affected docs in `docs/` and template docs.

### Out of scope

- Backward compatibility for already-generated projects that still contain `engine/xtask`.
- Migration tooling for old projects.
- Any roadmap edits.

## Target Architecture

### Desired ownership model

- **CLI-owned orchestration**
  - Build/bundle/install logic is owned by the main Wavecraft CLI (`cli/`).
  - Generated projects rely on CLI entrypoints instead of a template-local xtask binary.

- **Template as consumer, not orchestrator**
  - `sdk-template/engine` contains only plugin/runtime code and minimal build metadata.
  - No `sdk-template/engine/xtask` crate, scripts, or references.

- **Single operational path**
  - Developer workflow for generated projects runs through `wavecraft start`/CLI commands.
  - CI validates the same path used by developers.

## Impact Map

### 1) CLI (`cli/`)

- Commands likely impacted:
  - `cli/src/commands/start.rs`
  - `cli/src/commands/create.rs`
  - Any helper modules used for install/bundle orchestration.
- Required outcome:
  - CLI performs all actions previously expected from template-local xtask in generated projects.

### 2) Template (`sdk-template/`)

- Remove directory:
  - `sdk-template/engine/xtask/`
- Update template manifests and references:
  - `sdk-template/engine/Cargo.toml`
  - `sdk-template/Cargo.toml` and/or workspace members if xtask is currently listed.
  - Any template scripts/readme instructions mentioning `engine/xtask`.
- Required outcome:
  - Freshly generated project has no `engine/xtask` crate and no dangling references.

### 3) CI / Validation

- Areas likely impacted:
  - Template validation paths and generated-project checks.
  - `cargo xtask ci-check` phases that validate template-generated builds.
  - Workflow files/scripts that invoke generated `engine/xtask` directly.
- Required outcome:
  - CI verifies generated projects using CLI-owned build/install path only.

### 4) Documentation

- Update docs that mention generated-project xtask usage to CLI usage.
- Priority docs:
  - `docs/architecture/development-workflows.md`
  - `docs/guides/sdk-getting-started.md`
  - `sdk-template/README.md` (template consumer-facing instructions)
  - Any feature spec docs tied to this flow (excluding `_archive`).
- Required outcome:
  - No active docs instruct users to run template-local `engine/xtask`.

## Ordered Execution Steps (Single Delivery)

### Step 1 — Baseline inventory and change list freeze

**Actions**

- Locate every reference to `sdk-template/engine/xtask` and generated-project `engine/xtask` invocation paths.
- Confirm exact CLI entrypoints that must absorb orchestration behavior.
- Freeze a definitive affected-file list across CLI/template/CI/docs.

**Checkpoint**

- ✅ Complete impact list approved for one-pass implementation.

### Step 2 — Move orchestration responsibility into CLI

**Actions**

- Implement/adjust CLI internals so bundle/install behavior required by generated projects is fully handled by CLI commands.
- Remove assumptions in CLI code that generated project provides its own `engine/xtask`.
- Keep command UX coherent (same top-level command surface where possible, but no compatibility shim required).

**Checkpoint**

- ✅ CLI can build/bundle/install generated plugin without relying on template-local xtask.

### Step 3 — Remove `sdk-template/engine/xtask` and clean template references

**Actions**

- Delete `sdk-template/engine/xtask/`.
- Update template Cargo workspace members/dependencies/scripts/readme files to remove xtask references.
- Ensure generated project scaffolding remains consistent and compilable.

**Checkpoint**

- ✅ Newly scaffolded project contains no `engine/xtask` path and no broken manifest references.

### Step 4 — Update CI and template validation flow

**Actions**

- Replace CI/template-validation commands that invoke generated `engine/xtask` with CLI-owned equivalents.
- Keep checks aligned with real user flow (`wavecraft create` + CLI-driven start/install/bundle path).

**Checkpoint**

- ✅ CI validates new projects exclusively via CLI-owned orchestration.

### Step 5 — Update documentation

**Actions**

- Revise architecture/guides/template docs to remove xtask instructions and document CLI-owned flow.
- Ensure cross-references remain valid and consistent with coding/documentation standards.

**Checkpoint**

- ✅ Documentation reflects new architecture with no stale xtask guidance.

### Step 6 — End-to-end verification and finalization

**Actions**

- Generate a fresh plugin project from current CLI.
- Run full validation sequence (see commands below).
- Confirm no references to `sdk-template/engine/xtask` remain in active paths/docs (except historical archive material).

**Checkpoint**

- ✅ All validation passes; implementation ready as one cohesive change set.

## Risks and Mitigations

| Risk                                                          | Impact                                      | Mitigation                                                                                 |
| ------------------------------------------------------------- | ------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Hidden coupling to generated `engine/xtask` in CLI or scripts | Build/install regressions                   | Perform repo-wide reference sweep first; add focused integration tests for CLI-owned path  |
| Template manifest drift after xtask removal                   | `wavecraft create` scaffolds broken project | Validate generated project compile + clippy + bundle on fresh scaffold                     |
| CI still invoking removed paths                               | Pipeline failures after merge               | Update CI commands in same delivery and run full local CI check before finalizing          |
| Documentation lagging behind implementation                   | Developer confusion                         | Treat docs updates as mandatory in same delivery; include grep-based stale-reference check |

## Validation Commands

Run from repository root unless noted.

```bash
# 1) Fast/standard validation
cargo xtask ci-check

# 2) Full validation path (template + CD dry-run)
cargo xtask ci-check --full

# 3) Generate a fresh project with current CLI
cargo run --manifest-path cli/Cargo.toml -- create TestPlugin --output target/tmp/test-plugin

# 4) Validate generated engine compiles cleanly
cd target/tmp/test-plugin/engine
cargo clippy --all-targets -- -D warnings

# 5) Validate generated project bundle/install flow via CLI-owned path
cd ..
cargo run --manifest-path /Users/ronhouben/code/private/wavecraft/cli/Cargo.toml -- start --install

# 6) Cleanup
cd /Users/ronhouben/code/private/wavecraft
rm -rf target/tmp/test-plugin
```

Optional safety checks for stale references:

```bash
# Ensure active docs/code no longer reference removed template xtask path
rg "sdk-template/engine/xtask|engine/xtask" docs cli sdk-template .github scripts
```

## Definition of Done

- [ ] `sdk-template/engine/xtask/` is removed from source.
- [ ] Generated projects no longer include/use `engine/xtask`.
- [ ] CLI fully owns bundle/install behavior used by generated projects.
- [ ] CLI/template tests and validation pass (`cargo xtask ci-check` and `--full`).
- [ ] CI/template-validation paths no longer call generated `engine/xtask`.
- [ ] Active documentation is updated to CLI-owned workflow.
- [ ] No roadmap changes were made.
- [ ] Delivered as one cohesive implementation (no split PR plan).

## Assumptions / Open Questions

Under the explicit no-backward-compatibility constraint, only one operational question remains:

- **Install semantics parity:** confirm whether CLI-owned install should remain behaviorally identical to current generated-project flow or can be simplified if not user-visible. Default recommendation: preserve user-visible behavior while simplifying internals.

All other compatibility-related migration questions are intentionally out of scope.

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Documentation and code conventions
- [Development Workflows](../../architecture/development-workflows.md) — CI and development flow context
- [Agent Development Flow](../../architecture/agent-development-flow.md) — Workflow and handoff model
