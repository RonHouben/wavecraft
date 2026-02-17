# Implementation Plan: New Project VST3 Build + Install

**Feature Name:** `new-project-vst3-build-install`  
**Date:** 2026-02-17  
**Owner:** Coder  
**Source Design:** `docs/feature-specs/new-project-vst3-build-install/low-level-design-new-project-vst3-build-install.md`

---

## 1) Overview

This plan implements a deterministic, macOS-first, user-facing install flow for newly generated Wavecraft projects.

The canonical contract is:

- `wavecraft bundle --install`

Delegated implementation contract in generated projects remains:

- `cargo xtask bundle`
- `cargo xtask bundle --install`

The implementation will make `wavecraft bundle --install` the only documented install path for end users, while ensuring robust diagnostics for missing artifacts, invalid working-directory context, and install permission failures.

---

## 2) Contract constraints

The following constraints are authoritative and must be preserved through implementation and validation:

1. **Canonical/only user-facing install flow:** `wavecraft bundle --install`
2. **Delegated generated-project mechanism:** top-level CLI delegates to `cargo xtask bundle --install`
3. **Standalone `install` and `install --dry-run`:** not required as public contract for generated projects
4. **Working-directory behavior:** top-level command must validate/resolve project-root context and fail clearly outside valid root
5. **Platform focus:** macOS-first VST3 workflow
6. **Artifact source:** install stage reads built artifacts from `target/bundled` (relative to generated project root)
7. **Install destination (macOS user-level):** `~/Library/Audio/Plug-Ins/VST3`
8. **Execution ordering:** install stage must not run unless bundle stage succeeds
9. **Diagnostics quality:** failures must be actionable, include relevant path/operation context, and suggest recovery steps

---

## 3) Affected files with purpose and sequencing

| Seq | File                                                           | Purpose                                                                                                                                  |
| --: | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
|   1 | `cli/src/main.rs`                                              | Register top-level `bundle` command and route execution for canonical user-facing entrypoint.                                            |
|   2 | `cli/src/commands/mod.rs`                                      | Wire/export bundle command module as part of CLI command surface.                                                                        |
|   3 | `cli/src/commands/bundle.rs` (new or existing)                 | Implement `wavecraft bundle --install`, project-context validation, delegation to xtask, and diagnostic propagation.                     |
|   4 | CLI project-root/context detection utilities (new or existing) | Provide reusable working-directory detection and invalid-context diagnostics.                                                            |
|   5 | `sdk-template/engine/xtask/src/main.rs`                        | Keep delegated `bundle` + `bundle --install` behavior, staged execution, artifact checks, and install diagnostics in generated projects. |
|   6 | `cli/src/commands/create.rs`                                   | Align post-create guidance so users are directed to canonical `wavecraft bundle --install`.                                              |
|   7 | `engine/xtask/src/commands/validate_template.rs`               | Harden template validation to assert delegated xtask contract and expected behavior.                                                     |
|   8 | `.github/workflows/template-validation.yml`                    | Enforce delegated contract checks in CI so drift is caught before merge.                                                                 |

**Sequencing rationale:** Implement top-level CLI command surface and context behavior first, keep delegated xtask behavior aligned second, then lock behavior through local + CI template validation.

---

## 4) Detailed implementation phases and steps

### Phase A — Top-level CLI contract (`cli/src/main.rs`, `cli/src/commands/mod.rs`, `cli/src/commands/bundle.rs`)

1. **Add top-level CLI entrypoint**
   - Register `bundle` command in `cli/src/main.rs`.
   - Wire module exports in `cli/src/commands/mod.rs`.

2. **Implement canonical user command behavior**
   - Support `wavecraft bundle --install` as canonical install workflow.
   - Ensure user-facing help and examples prioritize this command.

3. **Working-directory/project-context validation**
   - Validate current directory (or resolve parent) as a Wavecraft project root via context utilities.
   - Fail fast outside valid context with actionable diagnostics and rerun guidance.

4. **Delegation behavior**
   - Delegate execution to generated-project `cargo xtask bundle --install`.
   - Preserve delegated stdout/stderr and exit code for transparent diagnostics.

5. **Failure propagation requirements**
   - If delegated command fails, return non-zero from top-level CLI.
   - Include delegated command context without masking underlying xtask error details.

### Phase B — Delegated template command contract (`sdk-template/engine/xtask/src/main.rs`)

1. **Extend `bundle` command flags**
   - Add `--install` boolean flag to the template `Bundle` subcommand.
   - Keep existing build/check semantics only as implementation details where still useful.

2. **Implement staged flow for `bundle --install`**
   - Stage 1: execute bundle build path.
   - Stage 2: if and only if Stage 1 succeeds, execute install routine.
   - Return non-zero immediately when Stage 1 fails.

3. **Enforce artifact discovery from canonical path**
   - Resolve source root as `target/bundled` (relative to generated project root).
   - Assert VST3 artifact exists before install copy starts.

4. **Implement macOS-first install destination behavior**
   - Resolve destination to `~/Library/Audio/Plug-Ins/VST3`.
   - Ensure destination directory exists (create if missing).
   - Use deterministic replacement semantics when plugin with same name already exists.

5. **Diagnostics and user output requirements**
   - On success: print built artifact location and installed destination path.
   - On missing artifact: print missing source path and recovery command `cargo xtask bundle --install`.
   - On permission/filesystem error: include failing operation, destination path, and OS error string; suggest closing DAW and checking permissions.

6. **Keep non-contractual internals internal**
   - Do not expose standalone `install` command as required user-facing path in template docs/messages.
   - If helper routines exist (e.g., dry-run internals), keep them implementation-private and undocumented as public contract.

---

### Phase C — CLI post-create message alignment (`cli/src/commands/create.rs`)

1. Replace next-step guidance that currently emphasizes `wavecraft start` as the only immediate action.
2. Add explicit post-create build/install guidance for macOS-first DAW testing:
   - `wavecraft bundle --install`
3. Keep development guidance (`wavecraft start`) but ensure canonical install flow is present and unambiguous.
4. Ensure message tone reflects contract constraints (no recommendation of standalone install command).

---

### Phase D — Docs alignment (`docs/guides/sdk-getting-started.md`, optional `README.md`)

1. Update getting-started “Build” and “Test in DAW” sections to canonical install flow:
   - Prefer single-step `wavecraft bundle --install`
2. Remove/replace references that imply standalone `cargo xtask install` is the required public workflow for generated projects.
3. Add troubleshooting entries aligned to expected diagnostics:
   - Missing artifact under `target/bundled`
   - Permission/locked-file failure in `~/Library/Audio/Plug-Ins/VST3`
4. Review `README.md` and update only if it contains conflicting user-facing install guidance.
5. Confirm wording stays consistent with macOS-first VST3 positioning.

---

### Phase E — Template validation hardening (`engine/xtask/src/commands/validate_template.rs`)

1. Expand generated-project xtask validation from `bundle --check` only to contract-relevant checks.
2. Add command-contract assertions for generated project:
   - `cargo xtask bundle --help` includes `--install`
3. Add behavior-level validation cases:
   - `cargo xtask bundle --install` succeeds in happy path and emits expected output markers.
   - Missing artifact scenario triggers expected failure shape and recovery guidance.
4. Add robust assertion helpers (stdout/stderr capture and matchers) to avoid brittle string checks.
5. Keep test execution deterministic and CI-friendly (no host DAW dependency).

---

### Phase F — CI enforcement (`.github/workflows/template-validation.yml`)

1. Update template-validation workflow steps to mirror local validation contract checks.
2. Add/replace xtask check step(s) so CI verifies canonical command surface and install pathway behavior.
3. Ensure CI failure messages clearly indicate contract drift (command surface vs docs vs behavior).
4. Keep runtime constraints practical for CI environment (use validation-safe checks; avoid requiring actual DAW integration).

---

### Phase G — Negative scenarios (required)

Implement and validate at least these negative cases end-to-end:

1. **Invalid project context (outside project root)**
   - Condition: run `wavecraft bundle --install` outside a valid Wavecraft project root.
   - Expected: non-zero exit; actionable guidance describing expected project root and rerun steps.

2. **Missing artifact**
   - Condition: install stage runs with no expected VST3 artifact in `target/bundled`.
   - Expected: non-zero exit; explicit missing-path diagnostic; recovery command shown.

3. **Permission failure / file lock**
   - Condition: write/copy into `~/Library/Audio/Plug-Ins/VST3` fails.
   - Expected: non-zero exit; operation + destination path + OS error; remediation hints (permission check, close DAW, retry).

4. **Delegated xtask failure propagation**
   - Condition: generated-project xtask returns non-zero.
   - Expected: top-level CLI returns non-zero and preserves delegated diagnostics.

---

## 5) Test/verification strategy (explicit pass/fail criteria)

### A. Local template validation (`cargo xtask validate-template`)

- **Pass** if generated project validation verifies command surface and behavior for `bundle` and `bundle --install`, including negative-case assertions.
- **Fail** if generated template lacks `--install` support on `bundle`, or diagnostics/exit behavior diverge from contract.

### B. Generated project command checks

1. `wavecraft bundle --install` from valid project root
   - **Pass:** command succeeds, delegates to xtask, and install result is reported clearly.
   - **Fail:** context resolution/delegation breaks or success output is ambiguous.

2. `wavecraft bundle --install` outside project root
   - **Pass:** command exits non-zero with actionable context guidance.
   - **Fail:** generic failure without clear next step.

3. Delegated failure propagation check
   - **Pass:** top-level CLI exits non-zero and preserves xtask diagnostics.
   - **Fail:** CLI masks underlying cause with non-actionable wrapper message.

4. `cargo xtask bundle`
   - **Pass:** bundle succeeds and artifacts are present under `target/bundled`.
   - **Fail:** missing/incorrect output path or non-actionable error.

5. `cargo xtask bundle --install`
   - **Pass:** build and install execute in order; VST3 appears in `~/Library/Audio/Plug-Ins/VST3`; success output includes installed path.
   - **Fail:** install runs before successful bundle, or target path not populated, or output unclear.

6. Missing artifact negative check
   - **Pass:** command exits non-zero with explicit missing source path and recovery guidance.
   - **Fail:** generic error without actionable path/recovery.

7. Permission failure negative check
   - **Pass:** command exits non-zero and prints operation + destination + OS error + remediation hints.
   - **Fail:** opaque/partial diagnostics.

### C. CI template validation workflow

- **Pass:** `.github/workflows/template-validation.yml` enforces same contract and fails on drift.
- **Fail:** CI still validates outdated command behavior (`bundle --check` only) without contract assertions.

### D. Documentation consistency checks

- **Pass:** `docs/guides/sdk-getting-started.md` (and `README.md` if applicable) consistently describe canonical install flow as `wavecraft bundle --install`.
- **Fail:** mixed guidance remains (canonical + legacy standalone install paths presented as equivalent user contract).

---

## 6) Dependencies and risks + mitigations

### Dependencies

- Existing generated-template xtask command scaffold in `sdk-template/engine/xtask/src/main.rs`
- CLI create output formatting in `cli/src/commands/create.rs`
- Template validation command framework in `engine/xtask/src/commands/validate_template.rs`
- CI enforcement in `.github/workflows/template-validation.yml`
- User-facing docs in `docs/guides/sdk-getting-started.md` and possibly `README.md`

### Risks and mitigations

| Risk                                                            | Impact     | Mitigation                                                                              |
| --------------------------------------------------------------- | ---------- | --------------------------------------------------------------------------------------- |
| Command contract drift between template implementation and docs | High       | Update docs in same change-set and add CI contract assertions.                          |
| Validation false confidence (checks surface only, not behavior) | High       | Add behavior checks (happy + negative paths), not only `--help` assertions.             |
| Platform-path ambiguity for generated projects                  | Medium     | Keep macOS-first VST3 path explicit and deterministic in code + docs.                   |
| Weak diagnostics in failure paths                               | High       | Require structured error output (operation, path, OS cause, remediation).               |
| Existing users relying on standalone install commands           | Low/Medium | Preserve internal flexibility but keep public guidance canonical to `bundle --install`. |

---

## Related Documents

- [Low-Level Design: New Project VST3 Build + Install](./low-level-design-new-project-vst3-build-install.md)
- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards](../../architecture/coding-standards.md)
- [Agent Development Flow](../../architecture/agent-development-flow.md)
- [Roadmap](../../roadmap.md)
