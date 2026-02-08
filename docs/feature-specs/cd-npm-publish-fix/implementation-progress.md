# Implementation Progress: Fix CD npm Publish Version Drift

## Status: Follow-up fix in progress — Publish-only model

---

## Phase 1: Immediate Fix — Sync Local Versions Past npm

- [x] **Step 1.1:** Bump `@wavecraft/core` to `0.7.4` in `ui/packages/core/package.json`
- [x] **Step 1.2:** Bump `@wavecraft/components` to `0.7.4` in `ui/packages/components/package.json`

## Phase 2: Robust Version Resolution — Registry-Aware Strategy

- [x] **Step 2.1:** Replace version check + bump in `publish-npm-core` with registry-aware "Determine publish version" step
- [x] **Step 2.2:** Replace version check + bump in `publish-npm-components` with same pattern
- [x] **Step 2.3:** Update `Commit version bump` conditions in both npm jobs (`steps.version.outputs.bumped`)
- [x] **Step 2.4:** Verify git tag version references still work (no code change needed — tags read from package.json)

## Phase 3: Structural Hardening

- [x] **Step 3.1:** Add `concurrency` group at workflow top level
- [x] **Step 3.2:** Fix `publish-npm-components` conditional to allow components when core wasn't needed

## Phase 4: CLI Version Resolution (same pattern)

- [x] **Step 4.1:** Replace CLI version check with registry-aware strategy (crates.io sparse index + sort -V)
- [x] **Step 4.2:** Update `Commit version bump` condition for CLI (`steps.version.outputs.bumped`)

## Phase 5: Follow-up — Publish-Only Model (post-merge findings)

The first CD run after PR #38 merge revealed two additional issues:
1. **Idempotency bug:** When `local == registry`, the version resolution treated it as "needs bump" instead of skipping. This caused unwanted 0.7.5 (core) and 0.8.5 (CLI) publishes.
2. **Branch protection:** `git push` to main is blocked by branch protection rules (requires PRs + status checks). The auto-bump commit model is incompatible.

**Fix: Switch to "publish-only" model:**
- [x] **Step 5.1:** Sync local versions to match registry (core → 0.7.5, CLI → 0.8.5)
- [x] **Step 5.2:** Replace auto-bump logic with skip-when-not-ahead for all 3 publish jobs
- [x] **Step 5.3:** Remove all "Commit version bump" steps (no more direct pushes to main)
- [x] **Step 5.4:** Condition build/publish/tag steps on `skip != 'true'`
- [x] **Step 5.5:** Remove "Pull latest changes" step from components (no longer needed)

## Testing

- [x] Trigger `workflow_dispatch` → push-triggered run published 0.7.4 successfully
- [x] Verify npm versions are >= `0.7.4` (core: 0.7.5, components: 0.7.4)
- [ ] Trigger `workflow_dispatch` → publish jobs skip (idempotent, local matches registry)
- [ ] Verify git tags created correctly
