# Implementation Plan: `sync-ui-versions`

## Overview

This plan implements `cargo xtask sync-ui-versions` in **one PR** (no phased rollout). The command enforces deterministic UI package version alignment across:

- `ui/packages/core/package.json`
- `ui/packages/components/package.json`
- `sdk-template/ui/package.json`

The command contract follows the finalized LLD in `low-level-design-sync-ui-versions.md`: default non-mutating check mode, strict scope, policy-gated semver movement, idempotent apply behavior, and exit codes `0/1/2`.

---

## Feature Scope Summary (`sync-ui-versions`)

The feature adds a single xtask command that:

- Detects drift in scoped version keys (`--check`, default)
- Applies scoped alignment updates when explicitly requested (`--apply`)
- Enforces lockstep package versions (`core.version == components.version`)
- Updates only allowed keys in only allowed files
- Enforces semver policy gates:
  - patch-only by default
  - `--allow-minor` enables minor updates
  - `--allow-major` enables major updates
- Returns deterministic exit codes:
  - `0` = aligned/success
  - `1` = drift found in check mode
  - `2` = operational/configuration/policy error

**Delivery model:** all implementation + tests + CI wiring are shipped in a **single PR**.

---

## Ordered Implementation Tasks (One PR)

### 1) Add new command module

**File:** `engine/xtask/src/commands/sync_ui_versions.rs` (new)

**Likely symbols to introduce:**

- `SyncUiVersionsArgs` (CLI args: `check`, `apply`, `allow_minor`, `allow_major`)
- `run_sync_ui_versions(args: &SyncUiVersionsArgs) -> Result<i32>` (or project-standard result type)
- Internal manifest model/helpers:
  - read/parse helpers for scoped manifests
  - scoped-diff computation
  - policy gate evaluator (patch/minor/major)
  - scoped writer + post-write verification

**Behavior in this step:**

- Implement mutually exclusive mode validation: `--check` and `--apply` cannot both be set.
- Implement default mode: if neither mode flag is provided, treat as check mode.
- Implement strict file scope and strict key scope enforcement.
- Implement idempotent apply logic (no-op when already aligned).
- Implement deterministic drift reporting and return code mapping `0/1/2`.

### 2) Export and wire command module

**File:** `engine/xtask/src/commands/mod.rs`

**Likely updates:**

- `pub mod sync_ui_versions;`
- Re-exports/imports needed by main command dispatcher.

### 3) Add CLI entry point and dispatch

**File:** `engine/xtask/src/main.rs`

**Likely updates:**

- Add subcommand variant for `sync-ui-versions`.
- Parse and pass flags (`--check`, `--apply`, `--allow-minor`, `--allow-major`) into `SyncUiVersionsArgs`.
- Ensure process exit behavior uses command return contract:
  - `0` success/aligned
  - `1` drift in check mode
  - `2` invalid usage/policy/parse/missing-key/IO errors

### 4) Add focused unit tests in xtask test module

**File:** `engine/xtask/src/tests.rs`

**Likely test groupings:**

- CLI mode validation tests:
  - mutual exclusion (`--check` + `--apply` => error code `2`)
  - default check mode when no mode flag set
- Policy gating tests:
  - patch-only default
  - minor blocked unless `--allow-minor`/`--allow-major`
  - major blocked unless `--allow-major`
- Scope and idempotency tests:
  - only scoped keys mutate
  - unrelated keys unchanged
  - apply twice produces no further changes

### 5) Add integration fixture tests (if used)

**Files/locations (planned):**

- `engine/xtask/tests/sync_ui_versions.rs`
- Fixtures under `engine/xtask/tests/fixtures/sync_ui_versions/...`

**Fixture sets to include:**

- `aligned/` → already aligned manifests
- `drift_patch/` → patch drift fixable in default policy
- `drift_minor_blocked/` → requires `--allow-minor` or `--allow-major`
- `drift_major_blocked/` → requires `--allow-major`
- `missing_key_error/` → required scoped key missing
- `invalid_semver_error/` → invalid semver in scoped key

**Validation in this step:**

- Assert output/exit-code contract per scenario.
- Assert on-disk mutations only touch allowed keys/files in apply mode.
- Assert check mode is non-mutating.

### 6) Wire CI required check

**File:** `.github/workflows/ci.yml`

**Update:**

- Add step in PR validation pipeline:
  - `cargo xtask sync-ui-versions --check`

**Required-check note:**

- This should be configured/treated as a required PR check so drift is caught before merge.

---

## CLI Contract Tasks (Explicit)

1. Enforce `--check` / `--apply` as mutually exclusive.
2. Use check mode by default when neither is provided.
3. Honor policy flags:
   - `--allow-minor`
   - `--allow-major`
4. Enforce strict file/key scope from LLD; no out-of-scope writes.
5. Guarantee idempotency for repeated `--apply` on aligned state.
6. Return exact exit-code contract:
   - `0` aligned/success
   - `1` drift in check mode
   - `2` operational/configuration/policy error

---

## Testing Plan

### Parser tests

- Parse all three scoped manifests successfully.
- Detect malformed JSON.
- Detect missing required keys and invalid semver values with clear failure mapping to exit code `2`.

### Unit tests (gating/scope/idempotency)

- Semver policy gate behavior for patch/minor/major transitions.
- Strict scope enforcement (only allowed files/keys may change).
- Idempotency: second `--apply` run is a no-op with success.

### Integration fixture tests

- End-to-end check/apply behavior over fixture directories.
- Exit codes `0/1/2` asserted for each scenario.
- Non-mutating check mode assertions and scoped mutation assertions in apply mode.

---

## CI Tasks

- Update `.github/workflows/ci.yml` to run:
  - `cargo xtask sync-ui-versions --check`
- Ensure this is marked/treated as a required check for PRs.
- Keep CI execution non-mutating (`--check` only).

---

## Risks and Rollback Notes

### Risks

- **False drift from parsing/range normalization:** could create noisy failures if caret/range handling is inconsistent.
- **Accidental out-of-scope mutation:** could modify non-target manifest fields.
- **Policy-gate confusion:** unclear diagnostics could lead to developer friction.
- **CI disruption:** newly required check may fail existing open branches until rebased/aligned.

### Mitigations

- Add strict scoped write tests and fixture snapshots.
- Keep deterministic diff ordering and explicit diagnostics for blocked minor/major changes.
- Validate idempotency and post-write invariants in tests.

### Rollback

- Revert the single PR if critical CI or workflow regressions appear.
- As a softer rollback, keep command code but remove required CI gating step temporarily while preserving tests.

---

## Definition of Done ✅

- [ ] `sync-ui-versions` command exists and is wired in xtask CLI.
- [ ] `--check`/`--apply` mutual exclusion enforced.
- [ ] Default mode is check when mode flags are omitted.
- [ ] `--allow-minor` and `--allow-major` policy gates implemented.
- [ ] Strict file scope and strict key scope enforced.
- [ ] Idempotent apply behavior verified.
- [ ] Exit codes `0/1/2` implemented and tested.
- [ ] Parser tests added.
- [ ] Unit tests for gating/scope/idempotency added.
- [ ] Integration fixture tests added under `engine/xtask/tests/...` (if fixture approach is used).
- [ ] CI includes `cargo xtask sync-ui-versions --check`.
- [ ] CI check is marked/treated as required.
- [ ] Entire feature delivered in **one PR** (no phased rollout).

---

## Implementation Constraint

This feature is explicitly implemented in **one PR only**. There is **no phased rollout** and no split across multiple PRs.
