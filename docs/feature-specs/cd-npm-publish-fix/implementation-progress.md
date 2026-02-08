# Implementation Progress: Fix CD npm Publish Version Drift

## Status: Not Started

---

## Phase 1: Immediate Fix — Sync Local Versions Past npm

- [ ] **Step 1.1:** Bump `@wavecraft/core` to `0.7.4` in `ui/packages/core/package.json`
- [ ] **Step 1.2:** Bump `@wavecraft/components` to `0.7.4` in `ui/packages/components/package.json`

## Phase 2: Robust Version Resolution — Registry-Aware Strategy

- [ ] **Step 2.1:** Replace version check + bump in `publish-npm-core` with registry-aware "Determine publish version" step
- [ ] **Step 2.2:** Replace version check + bump in `publish-npm-components` with same pattern
- [ ] **Step 2.3:** Update `Commit version bump` conditions in both npm jobs (`steps.version.outputs.bumped`)
- [ ] **Step 2.4:** Verify git tag version references still work (no code change expected)

## Phase 3: Structural Hardening

- [ ] **Step 3.1:** Add `concurrency` group at workflow top level
- [ ] **Step 3.2:** Fix `publish-npm-components` conditional to allow components when core wasn't needed

## Phase 4: CLI Version Resolution (same pattern)

- [ ] **Step 4.1:** Replace CLI version check with registry-aware strategy
- [ ] **Step 4.2:** Update `Commit version bump` condition for CLI (`steps.version.outputs.bumped`)

## Testing

- [ ] Trigger `workflow_dispatch` → all jobs pass or correctly skip
- [ ] Verify npm versions are >= `0.7.4`
- [ ] Re-run `workflow_dispatch` → publish jobs skip (idempotent)
- [ ] Verify git tags created correctly
