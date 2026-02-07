# Implementation Plan: crates.io Publishing Fix

## Overview

Fix crates.io publishing for the Wavecraft CLI by adding explicit version requirements to all CLI path dependencies, making `wavecraft-dev-server` publishable, and tightening the Continuous Deploy workflow with preflight checks. This plan follows the approved low‑level design and prioritizes fast feedback and minimal disruption.

## Requirements

- CLI `cargo publish` must succeed on crates.io.
- All CLI path dependencies must include explicit `version` fields.
- `wavecraft-dev-server` must be publishable (or feature‑gated if later decided).
- Continuous Deploy must fail fast if publishability conditions are not met.

## Architecture Changes

- `cli/Cargo.toml`: add `version` fields to path dependencies.
- `engine/crates/wavecraft-dev-server/Cargo.toml`: set `publish = true` (confirm version alignment).
- `.github/workflows/continuous-deploy.yml`: add CLI publish preflight (dry‑run and guardrails).

## Implementation Steps

### Phase 1: Dependency publishability alignment
1. **Add versioned path dependencies** (File: `cli/Cargo.toml`)
   - Action: Add `version = "<current engine version>"` to `wavecraft-protocol`, `wavecraft-bridge`, `wavecraft-metering`, and `wavecraft-dev-server`.
   - Why: crates.io requires explicit version requirements even when `path` is specified.
   - Dependencies: None
   - Risk: Low

2. **Make dev server publishable** (File: `engine/crates/wavecraft-dev-server/Cargo.toml`)
   - Action: Set `publish = true` and verify the crate version matches the engine workspace version.
   - Why: CLI depends on this crate at runtime; it must be available on crates.io.
   - Dependencies: Step 1
   - Risk: Medium (publishability constraints like missing metadata may surface)

### Phase 2: Workflow guardrails
3. **Add CLI publish preflight** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Add a `cargo publish --dry-run` step for CLI before the real publish step.
   - Why: Fail fast on versioning or publishability issues.
   - Dependencies: Phase 1
   - Risk: Low

4. **Add explicit guardrails** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Add a validation step that checks for `publish = false` in CLI dependency graph (or a simple manifest check for known deps).
   - Why: Prevent hidden publish failures on CI.
   - Dependencies: Phase 2, Step 3
   - Risk: Low/Medium (script maintenance)

### Phase 3: Validation & documentation
5. **Local dry‑run validation** (File: `cli/Cargo.toml` and CLI workspace)
   - Action: Run `cargo publish --manifest-path cli/Cargo.toml --dry-run` locally.
   - Why: Validate publishability before CI.
   - Dependencies: Phase 1
   - Risk: Low

6. **Document outcome** (File: `docs/feature-specs/crates-io-publishing-fix/implementation-progress.md`)
   - Action: Update progress checklist with results and any caveats.
   - Why: Keep feature artifacts complete for handoff.
   - Dependencies: Steps 1–5
   - Risk: Low

## Testing Strategy

- **CLI publishability**: `cargo publish --manifest-path cli/Cargo.toml --dry-run`
- **CI verification**: confirm `publish-cli` job success on `main` after merge.

## Risks & Mitigations

- **Risk**: `wavecraft-dev-server` missing required metadata for publish (e.g., description/license).
  - **Mitigation**: Add missing `license`, `repository`, `description`, and ensure version alignment with workspace.
- **Risk**: Workflow scripts drift with future dependencies.
  - **Mitigation**: Keep guardrails minimal and focused on the known CLI deps.

## Success Criteria

- [ ] CLI `cargo publish` succeeds on crates.io.
- [ ] All CLI path dependencies include explicit `version` fields.
- [ ] `wavecraft-dev-server` is published and available on crates.io.
- [ ] Continuous Deploy `publish-cli` job passes end‑to‑end on `main`.
