# QA Report: SDK Example Plugin (`cargo xtask dev` from SDK root)

**Date**: 2026-02-12  
**Reviewer**: QA Agent  
**Status**: PASS (with recommendations)

## Summary

| Severity | Count |
| -------- | ----- |
| Critical | 0     |
| High     | 0     |
| Medium   | 3     |
| Low      | 2     |

**Overall**: PASS (no Critical/High findings)

## Scope Reviewed

Goal: Enable `cargo xtask dev` from the Wavecraft SDK repository root by routing the dev-server workflow through a minimal example plugin crate (`engine/crates/wavecraft-example`) and "SDK mode" project detection.

Key areas reviewed:

- `engine/xtask` dev command invocation and path resolution
- CLI project detection and SDK-mode routing
- Dylib discovery behavior when `engine_dir` is a workspace member (`engine/crates/wavecraft-example`)
- Example plugin crate structure and template parity intent
- Test coverage and regression risk

## Automated Check Results

**Note:** Automated checks and manual validation were run by the Tester agent. Evidence is recorded in `docs/feature-specs/sdk-example-plugin/test-plan.md`.

- Linting: ✅ PASSED (ESLint, Prettier, cargo fmt, clippy)
- Tests: ✅ PASSED (Engine + UI)

## Findings

| ID  | Severity | Category                        | Description                                                                                                                                                                                                                                                                                                           | Location                                                              | Recommendation                                                                                                                                                                                                                      |
| --- | -------- | ------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | Medium   | Robustness / Detection          | SDK repo detection uses a string search for `"[workspace]"`, which can mis-detect if the file contains that substring in a comment or if a future supported plugin project uses an engine workspace. Current error message on "workspace-but-not-SDK" cases is also slightly confusing ("missing wavecraft-example"). | `cli/src/project/detection.rs:98` (`content.contains("[workspace]")`) | Parse TOML and check for a real `workspace` table OR make detection more specific (e.g., confirm presence of `cli/Cargo.toml` at repo root and/or `engine/crates/wavecraft-core`). Consider a clearer error for non-SDK workspaces. |
| 2   | Medium   | Error handling                  | Uses `expect()` for child stdio handles; project coding standards generally discourage panic paths in production code, even if it's "should never happen".                                                                                                                                                            | `cli/src/project/param_extract.rs:57-58`                              | Replace `expect()` with `ok_or_else(...).context(...)` and return a proper error instead of panicking.                                                                                                                              |
| 3   | Medium   | Repo hygiene                    | A root-level `package-lock.json` (minimal/empty) was added. The repo's JS workspace appears to be under `ui/`; a root lockfile may be accidental and can confuse contributors/CI in some setups.                                                                                                                      | `package-lock.json`                                                   | Confirm whether root npm usage is intended. If accidental, remove and add guidance to avoid running npm at repo root. If intended, ensure there is a matching root `package.json` and documented workflow.                          |
| 4   | Low      | UX / Logging                    | The log line "Watching engine/src/ for changes" is inaccurate in SDK mode (it watches `engine/crates/wavecraft-example/src`). This can mislead debugging.                                                                                                                                                             | `cli/src/commands/start.rs` (watch message)                           | Print the actual watched path (e.g., `project.engine_dir.join("src")`) or a relative path.                                                                                                                                          |
| 5   | Low      | Documentation / Maintainability | `engine/crates/wavecraft-example/src/lib.rs` is intentionally minimal and mirrors template style, but has no crate-level comment explaining its role as "SDK integration example".                                                                                                                                    | `engine/crates/wavecraft-example/src/lib.rs`                          | Add a short module doc comment clarifying its purpose and the "template parity" expectation.                                                                                                                                        |

## Positive Notes / What Looks Good

- **Pathing is coherent end-to-end:** repo-root `cargo xtask dev` works via `.cargo/config.toml` alias and `engine/xtask` resolves paths relative to `engine/xtask` safely.
- **SDK-mode implementation is low-impact:** normal plugin projects remain supported; SDK mode only activates on workspace detection and then validates the example crate exists.
- **Dylib discovery accounts for workspace targets:** `resolve_debug_dir()` now checks crate-local, parent, and workspace-root target locations—appropriate for `engine/crates/wavecraft-example` builds.
- **Testing is strong for a workflow change:** test-plan includes `cargo xtask ci-check`, SDK startup, hot reload, and a regression test on a generated plugin project.

## Architectural Concerns

None requiring an Architect handoff. This change stays within CLI detection + dev workflow routing and doesn't violate the documented layer boundaries (DSP vs bridge vs UI).

## Handoff Decision

**Target Agent**: `coder`  
**Reasoning**: Only Medium/Low issues remain and they are localized (robust detection, panic removal, repo lockfile cleanup, minor logging/doc polish). No architectural redesign needed.
