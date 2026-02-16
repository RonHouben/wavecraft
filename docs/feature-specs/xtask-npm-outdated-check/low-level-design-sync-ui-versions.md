# Low-Level Design — Sync UI Versions Automation

- **Feature:** `xtask-npm-outdated-check`
- **Date:** 2026-02-16

## 1) Summary

Introduce a new xtask command, `sync-ui-versions`, to enforce deterministic npm version alignment across exactly three manifests:

- `ui/packages/core/package.json`
- `ui/packages/components/package.json`
- `sdk-template/ui/package.json`

The command supports two modes:

- `--check` for non-mutating drift detection
- `--apply` for controlled updates

Safety principles:

- **Patch-only by default**
- **Strict file/key scope**
- **Idempotent behavior**

## 2) Goals

- Prevent accidental version drift between UI packages and SDK template dependencies.
- Provide deterministic, reviewable version synchronization in local workflows and CI.
- Enforce explicit policy gates for minor/major updates.
- Ensure repeated runs produce stable, no-op outcomes when already aligned.

## 3) Non-Goals

- Full workspace dependency hygiene for unrelated npm packages.
- Automatic lockfile regeneration as part of default command behavior.
- Auto-updating manifests outside the three scoped files.
- Semantic changes to package dependency architecture.

## 4) Related Documentation

- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards](../../architecture/coding-standards.md)
- [Development Workflows](../../architecture/development-workflows.md)
- [Testing & Quality](../../architecture/coding-standards-testing.md)
- [CI Pipeline Guide](../../guides/ci-pipeline.md)
- [Roadmap](../../roadmap.md)

## 5) Current State

Example drift scenario motivating this command:

- `@wavecraft/core` package version: `0.7.29`
- `@wavecraft/components` package version: `0.7.4`
- `sdk-template/ui/package.json` dependencies still pinned to `^0.7.1`

This mismatch causes preventable inconsistency in generated projects and CI expectations.

## 6) Command Surface

- **Command:** `cargo xtask sync-ui-versions`

## 7) CLI Contract

- `--check` and `--apply` are mutually exclusive.
- Default mode is `--check` when no mode flag is provided.
- Optional policy flags:
  - `--allow-minor`
  - `--allow-major`

Policy precedence and gating:

1. Default policy: patch-only.
2. `--allow-minor` allows patch + minor updates.
3. `--allow-major` allows patch + minor + major updates.
4. Major/minor changes are blocked unless explicitly enabled.
5. Invalid flag combinations fail fast with clear diagnostics.

## 8) Scope & Invariants

Strict file scope (only these files may be read/written):

1. `ui/packages/core/package.json`
2. `ui/packages/components/package.json`
3. `sdk-template/ui/package.json`

Strict key scope (only these fields may be mutated):

- `core.version`
- `components.version`
- `components.peerDependencies["@wavecraft/core"]`
- `sdk-template/ui.dependencies["@wavecraft/core"]`
- `sdk-template/ui.dependencies["@wavecraft/components"]`

Invariants:

- No other manifest fields are changed.
- No file outside strict scope is modified.
- Multiple consecutive runs with no upstream version change are no-op.

## 9) Version Policy

- Lockstep invariant: `core.version == components.version`.
- Template dependencies use caret baseline ranges (e.g., `^0.7.29`).
- Default synchronization mode permits patch-only movement.
- Minor/major transitions require explicit policy flags.

## 10) Detailed Behavior

### `--check`

1. Load and parse the three manifests.
2. Compute expected aligned state from policy and lockstep rules.
3. Compare expected vs current scoped keys.
4. Print drift report (human-readable + deterministic ordering).
5. Exit codes:
   - `0`: aligned (no drift)
   - `1`: drift detected
   - `2`: operational/configuration error (parse failure, missing key, invalid semver, invalid flags)

### `--apply`

1. Load manifests and validate policy gates.
2. Compute expected aligned scoped fields.
3. Write only scoped fields that differ.
4. Post-write verify by re-reading scoped fields and re-validating invariants.
5. If already aligned, return success with explicit no-op message.

## 11) CI Integration Plan

- Add required PR check:
  - `cargo xtask sync-ui-versions --check`
- Keep this check non-mutating in CI.
- Optional follow-up automation:
  - Scheduled/triggered workflow runs `--apply` on a branch and opens a PR.
  - No direct push to `main`.

## 12) Edge Cases & Failure Modes

| Scenario                                         | Expected behavior                                  |                                  Exit code | Notes                                              |
| ------------------------------------------------ | -------------------------------------------------- | -----------------------------------------: | -------------------------------------------------- |
| Workspace outdated noise from unrelated packages | Ignore unrelated dependencies                      |                                        0/1 | Drift signal must be limited to strict scoped keys |
| Semver/range mismatch (`0.7.29` vs `^0.7.4`)     | Normalize by policy in expected state              | 1 in `--check`, 0 in `--apply` after write | Caret handling remains explicit                    |
| Minor/major required but not allowed             | Fail with policy-gating diagnostic                 |                                          2 | User must pass `--allow-minor` / `--allow-major`   |
| Invalid semver in scoped key                     | Fail fast with key/path detail                     |                                          2 | No writes performed                                |
| Missing required scoped key                      | Fail fast with manifest path + key                 |                                          2 | Protects deterministic behavior                    |
| Lockfile drift/churn expectations                | Do not auto-regenerate lockfiles by default        |                                        0/1 | See lockfile behavior section                      |
| Non-idempotent formatting risk                   | Use stable JSON write strategy for scoped mutation |                    2 on write/verify error | Post-write verify catches unexpected deltas        |

## 13) Lockfile Behavior

Default behavior is **manifest-scope only**:

- No implicit `npm install`/`npm update`
- No implicit lockfile mutation

Future enhancement candidate:

- Optional `--sync-lockfiles` mode to intentionally refresh lockfiles after manifest alignment.

## 14) Testing & Validation Plan

Unit tests:

- Manifest parser/serializer tests for scoped key extraction and writes.
- Policy gating tests (patch-only default, minor/major enablement).
- Scope enforcement tests (no mutation beyond allowed keys/files).
- Idempotency tests (two consecutive apply runs produce no further diffs).

Integration fixture tests:

- Already aligned manifests.
- Drifted patch scenario.
- Minor/major mismatch requiring flags.
- Missing-key and invalid-semver failure fixtures.

CI checks:

- Add `sync-ui-versions --check` to required PR validation.
- Preserve deterministic output ordering for reliable CI logs.

## 15) Single-PR Execution

Implementation is delivered in one PR:

1. Add `sync-ui-versions` command behavior and policy enforcement.
2. Add/adjust tests for check/apply behavior, scope invariants, and idempotency.
3. Add required CI validation step: `cargo xtask sync-ui-versions --check`.

## 16) Open Questions

- Should `--allow-major` imply `--allow-minor`, or should both be independently validated for explicitness?
- Should drift reports also emit machine-readable JSON for CI annotation tooling?
- Do we need a strict mode that fails on extra unexpected Wavecraft dependency keys in scoped manifests?
- Should `--sync-lockfiles` run in one workspace root, or per-package for deterministic lockfile ownership?

## 17) Acceptance Criteria

- [ ] `sync-ui-versions` command exists and is documented.
- [ ] `--check` is default and non-mutating.
- [ ] `--apply` updates only scoped fields in only three scoped files.
- [ ] Patch-only default policy is enforced.
- [ ] `--allow-minor` and `--allow-major` gates are enforced correctly.
- [ ] `core.version` and `components.version` are lockstep after apply.
- [ ] Template dependency ranges are updated to caret baseline.
- [ ] Implementation is completed in one PR.
- [ ] Exit codes `0/1/2` follow the defined contract.
- [ ] Re-running `--apply` on aligned manifests is idempotent no-op.
- [ ] CI can enforce `cargo xtask sync-ui-versions --check` as a required PR gate.
