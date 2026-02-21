# Summary

This PR completes the `codebase-refactor-sweep` stream with broad modularization across CLI, dev-server, protocol, macros, and Windows WebView integration, followed by QA hardening fixes and documentation closeout/archival updates.

# Changes grouped by area

## CLI and template modularization

- Decomposed large command modules into focused submodules:
  - `bundle` split into `bundle_runner`, `engine_build`, `install`, `metadata_refresh`, `project_root`, `ui_assets`
  - `start` split into `audio_runtime`, `metadata_cache`, `preflight`, `reload_extractors`, `shutdown`, `startup_pipeline`, `tsconfig_paths`
  - `update` split into `project_update`, `self_update`, `summary`
- Refined template orchestration and extraction paths:
  - `cli/src/template/overrides.rs`, `extract.rs`, `dependency_rewrites.rs`, `tsconfig_paths.rs`
- Preserved behavior while reducing monolithic file size and improving maintainability.

## Engine / protocol / macro refactors

- `wavecraft-protocol` IPC decomposition:
  - Split `ipc.rs` into `ipc/envelope.rs`, `ipc/errors.rs`, `ipc/methods.rs`
- `wavecraft-macros` plugin decomposition:
  - Split plugin generation/parsing/runtime logic into `plugin/codegen.rs`, `metadata.rs`, `naming.rs`, `parse.rs`, `runtime_params.rs`
- `wavecraft-nih_plug` Windows editor modularization:
  - Split `windows.rs` into `windows/mod.rs`, `content.rs`, `ipc_bridge.rs`, `runtime_checks.rs`, `webview2_init.rs`
- Additional focused cleanups in bridge, DSP, processors, and dev-server audio/reload/session paths.

## Dev-server restructuring

- Refactored `dev-server/src/audio/server.rs` into dedicated modules:
  - `device_setup`, `input_pipeline`, `metering`, `output_modifiers`, `output_routing`, `startup_wiring`
- Kept runtime/audio-status behavior aligned while improving separation of concerns.

## QA hardening and closeout documentation

- Applied QA follow-up fix commit (`fix: apply QA hardening follow-ups`).
- Finalized and archived feature spec artifacts under `docs/feature-specs/_archive/codebase-refactor-sweep/`.
- Marked milestone completion in roadmap and synchronized closeout docs.

# Commits list

- `0f33391` docs: mark milestone 19 complete in roadmap
- `6a08948` docs: archive codebase refactor sweep spec
- `cd677a6` docs: finalize post-hardening test and QA reports
- `bc1b196` fix: apply QA hardening follow-ups
- `e4d49e8` docs: update validation outcomes for full CI check failure reason
- `ec8f6e5` docs: record tier 3 full-gate rerun result
- `ec4253b` refactor: Tier 2 batch 15 quick-scan cleanup
- `1a60f6b` refactor: Tier 2 batch 14 quick-scan cleanup
- `c022894` refactor: Tier 2 batch 13 quick-scan cleanup
- `82e7a5a` refactor: Tier 2 batch 12 quick-scan cleanup
- `b8bf719` refactor: Tier 2 batch 11 quick-scan cleanup
- `43c0aef` refactor: Tier 2 batch 10 quick-scan cleanup
- `64ecd9f` refactor: Tier 2 batch 9 quick-scan cleanup
- `50a540d` refactor: Tier 2 batch 8 quick-scan cleanup
- `d233b48` refactor: Tier 2 batch 7 quick-scan cleanup
- `351d49a` refactor: Tier 2 batch 6 quick-scan cleanup
- `1d57e1d` refactor: Tier 2 batch 5 quick-scan cleanup
- `27e5228` refactor: Tier 2 batch 4 quick-scan cleanup
- `b41d963` refactor(m19): apply tier2 batch3 bounded cleanup
- `fc22591` refactor(m19): complete windows modularization and tier2 batch cleanup
- `4e3d16a` feat: Implement Tier 2 Batch 1 quick-scan cleanup across multiple modules
- `973e146` Refactor Windows WebView2 integration
- `da2944c` feat: Decompose output callback routing into dedicated module for clarity and behavior preservation
- `6884211` Refactor IPC module: Extract method contracts and constants into dedicated module
- `f8b6fae` Add wavecraft-macros plugin metadata and naming utilities
- `e028c9a` feat: Decompose template handling into focused modules
- `1731b0b` feat: Update implementation progress for Codebase Refactor Sweep with detailed slice tracking and validation results
- `4e59c72` feat: Refactor startup command by extracting modules for shutdown, startup pipeline, and tsconfig path injection
- `29a8fcb` feat: add implementation plan for Milestone 19 - Codebase Refactor Sweep
- `3e8a931` feat: add low-level design document for codebase refactor sweep and update user stories and roadmap with escalation rules
- `be5facb` feat: add user story for coding guidelines updates and document lessons learned during codebase refactor
- `c2465bb` archive: remove obsolete oscillator passthrough mix documentation

# Related docs links

- [Implementation progress](./implementation-progress.md)
- [Archived implementation plan](../_archive/codebase-refactor-sweep/implementation-plan.md)
- [Archived low-level design](../_archive/codebase-refactor-sweep/low-level-design-codebase-refactor-sweep.md)
- [Archived test plan](../_archive/codebase-refactor-sweep/test-plan.md)
- [Archived QA report](../_archive/codebase-refactor-sweep/QA-report.md)
- [Roadmap](../../roadmap.md)

# Testing summary

- Full project validation was executed successfully prior to PR creation (`cargo xtask ci-check --full` exit code `0` in the active terminal context).
- This PR is primarily a refactor + hardening + documentation closeout aggregation with behavior-preserving intent.

# Checklist

- [x] Aggregate PR title selected for multi-commit sweep + hardening + closeout
- [x] Commit range reviewed from merge-base to `HEAD`
- [x] File-level impact reviewed (`git diff --stat` and `--name-only`)
- [x] PR summary created at requested path
- [ ] Await PR review and merge