# QA Report: Canonical SDK Template Refactor

**Date**: 2026-02-13  
**Reviewer**: QA Agent  
**Branch**: `refactor/canonical-sdk-template`  
**Status**: **PASS with findings**

## Summary

| Severity | Count |
| -------- | ----- |
| Critical | 0     |
| High     | 0     |
| Medium   | 1     |
| Low      | 2     |
| Info     | 5     |

**Overall verdict**: **PASS** (no Critical/High issues), with recommended follow-up cleanup.

## Automated Check Results

From `docs/feature-specs/canonical-sdk-template/test-plan.md`:

- Linting/tests via `cargo xtask ci-check`: ✅ PASSED
- Template generation/clippy/build smoke tests: ✅ PASSED
- SDK-mode dev flow + HMR + alias behavior: ✅ PASSED

## Findings

| ID | Severity | Category | Description | Location | Recommendation |
|---|---|---|---|---|---|
| 1 | Medium | Robustness / Script maintainability | Dependency rewrite in setup script relies on exact `Cargo.toml` formatting/order via brittle `sed` patterns. Small formatting changes can silently skip rewrite and leave git deps in SDK mode. | `scripts/setup-dev-template.sh:55-59` | Use a structured transform (e.g. `tomlq`, small Rust helper, or explicit post-check that expected `path =` entries exist). Fail fast if rewrite did not occur. |
| 2 | Low | Documentation consistency | Testing standards doc still references old root-level `ui/src/test/...` paths inconsistent with current `ui/test/...` workspace layout. | `docs/architecture/coding-standards-testing.md:109`, `docs/architecture/coding-standards-testing.md:137` | Update examples/paths to current test structure (`ui/test/...`) and ensure all snippets match repo layout. |
| 3 | Low | Agent/docs consistency | Live agent instruction docs still mention old `ui/src/...` locations, which can mislead future codebase audits/search requests. | `.github/copilot-instructions.md:69`, `.github/copilot-instructions.md:75`, `.github/agents/tester.agent.md:88` | Refresh those prompts to package-workspace paths (e.g., `ui/packages/**`, `ui/test/**`) to reflect canonical structure. |

## Risk Assessment Notes

### include_dir path safety
- `include_dir!("$CARGO_MANIFEST_DIR/../sdk-template")` is implemented at `cli/src/template/mod.rs:13`.
- **Assessment**: acceptable in current project constraints (git-centric monorepo workflow), matches design intent.
- Residual risk is mostly around packaging context assumptions, not runtime behavior.

### SDK-mode detection and errors
- Detection and guidance messages are clear/actionable:
  - missing generated manifest + setup hint: `cli/src/project/detection.rs:81-83`
  - missing template UI package: `cli/src/project/detection.rs:90`
- **Assessment**: good UX for failure recovery.

### setup script sed robustness
- See finding #1 (medium).
- `set -euo pipefail` present (`scripts/setup-dev-template.sh:2`), good baseline.
- Current implementation is functional but fragile to template formatting drift.

### Vite alias SDK-mode detection
- SDK-mode check: `sdk-template/ui/vite.config.ts:8`
- Alias gating: `sdk-template/ui/vite.config.ts:14-17`
- **Assessment**: practical and low risk; false positives are unlikely in typical generated plugin repos.

## Consistency & Completeness Checks

- No stale **live source/workflow** references to `wavecraft-example` / `cli/sdk-templates` found in active code paths; matches refactor intent.
- Remaining matches observed were primarily:
  - archived specs/history (`docs/feature-specs/_archive/`) — expected
  - build artifacts under `target/` — expected noise
- `.gitignore` covers new template artifacts and generated files:
  - `sdk-template` artifacts: `.gitignore:17-24` ✅
- CI path filter updated:
  - `continuous-deploy.yml` includes `sdk-template/**` at `.github/workflows/continuous-deploy.yml:59` ✅

## Architecture Alignment

Compared with:
- `docs/feature-specs/canonical-sdk-template/low-level-design-canonical-sdk-template.md`
- `docs/feature-specs/canonical-sdk-template/implementation-plan.md`

**Result**: implementation aligns with approved architecture:
- canonical `sdk-template/` established,
- CLI embedding switched,
- SDK-mode redirected,
- `ui/` treated as package workspace,
- workflow/docs updated.

No architectural blocker identified.

## Handoff Decision

**Target agent**: `coder`  
**Reason**: fix medium/low maintainability + documentation consistency findings (no architecture redesign required).

---

## Related Documents

- [Low-Level Design](./low-level-design-canonical-sdk-template.md) — Architecture rationale
- [Implementation Plan](./implementation-plan.md) — Step-by-step plan
- [Test Plan](./test-plan.md) — Test cases and results
