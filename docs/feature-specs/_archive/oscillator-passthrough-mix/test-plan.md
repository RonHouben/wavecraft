# Archived

This feature spec has been archived.

Canonical file: `docs/feature-specs/_archive/oscillator-passthrough-mix/test-plan.md`

Please read/update the archived file only.

- **Date**: 2026-02-18
- **Tester**: Tester Agent
- **Branch Observed During Test Run**: `bugfix/oscillator-passthrough-mix`

## Test Summary

| Status     | Count |
| ---------- | ----- |
| ✅ PASS    | 4     |
| ❌ FAIL    | 0     |
| ⏸️ BLOCKED | 3     |
| ⬜ NOT RUN | 0     |

## Prerequisites

- [x] `cargo xtask ci-check` passes (all lint + tests)
- [x] Build/install artifacts prepared for DAW test (`osc_mix_test.vst3` installed)
- [x] Ableton available on machine (`Ableton Live 12 Standard.app` detected)

## Commands Executed and Outcomes

1. `git -C /Users/ronhouben/code/private/wavecraft status --short --branch`
   - **Outcome**: branch/worktree captured
2. `cargo test --manifest-path engine/Cargo.toml -p wavecraft-processors`
   - **Outcome**: ✅ `18 passed; 0 failed`
3. `cargo xtask ci-check`
   - **Outcome**: ✅ Docs/Lint/Typecheck/Engine/UI all passed
4. `cd engine && cargo xtask bundle --release`
   - **Outcome**: ❌ No plugin bundles exported from selected SDK crate context (non-feature-specific packaging limitation)
5. `cargo run --manifest-path cli/Cargo.toml -- create OscMixTest --output target/tmp/osc-mix-test`
   - **Outcome**: ✅ Generated fresh plugin project
6. `cd target/tmp/osc-mix-test && cargo run --manifest-path /Users/ronhouben/code/private/wavecraft/cli/Cargo.toml -- bundle --install`
   - **Outcome**: ✅ Built and installed `osc_mix_test.vst3`
7. Artifact checks for bundled/install paths
   - **Outcome**: ✅ `.vst3` and `.clap` present; installed VST3 present

---

## Test Cases

### TC-001: Disabled oscillator preserves passthrough (automated)

**Description**: Verify oscillator does not mutate input when disabled.

**Expected Result**: Input samples unchanged.

**Status**: ✅ PASS

**Actual Result**: `oscillator_preserves_passthrough_when_disabled` passed in `wavecraft-processors` tests.

---

### TC-002: Enabled oscillator remains audible on silent input (automated)

**Description**: Verify oscillator outputs audible signal when enabled.

**Expected Result**: Non-zero output signal generated.

**Status**: ✅ PASS

**Actual Result**: `oscillator_generates_signal_when_enabled_on_silent_input` passed.

---

### TC-003: Enabled oscillator mixes additively with non-zero input (automated)

**Description**: Verify oscillator adds to existing signal rather than replacing it.

**Expected Result**: `mixed - input == oscillator_only` sample-wise.

**Status**: ✅ PASS

**Actual Result**: `oscillator_enabled_adds_signal_without_removing_input` passed.

---

### TC-004: Repo regression sweep (automated)

**Description**: Ensure no broader regressions from oscillator behavior change.

**Expected Result**: CI checks all pass.

**Status**: ✅ PASS

**Actual Result**: `cargo xtask ci-check` passed (docs/lint/typecheck/tests).

---

### TC-005: Ableton smoke — DAW input audible with oscillator enabled (manual)

**Description**: In Ableton, verify passthrough remains audible after enabling oscillator.

**Expected Result**: DAW input remains audible.

**Status**: ⏸️ BLOCKED

**Actual Result**: Not executed in-session; requires interactive DAW monitoring.

---

### TC-006: Ableton smoke — oscillator + DAW input audible simultaneously (manual)

**Description**: Verify both sources are audible at once.

**Expected Result**: Mixed output with both oscillator and DAW signal present.

**Status**: ⏸️ BLOCKED

**Actual Result**: Not executed in-session; requires interactive DAW monitoring.

---

### TC-007: Ableton smoke — disabling oscillator does not "restore" passthrough (manual)

**Description**: Verify passthrough is already present before disabling/removing oscillator.

**Expected Result**: Disabling/removing oscillator changes tonal content only; passthrough remains present.

**Status**: ⏸️ BLOCKED

**Actual Result**: Not executed in-session; requires interactive DAW monitoring.

---

## Issues Found

### Issue #1: SDK root `cargo xtask bundle --release` does not emit plugin bundle in this context

- **Severity**: Low (workflow/context issue, not oscillator behavior bug)
- **Test Case**: Setup step before manual DAW check
- **Description**: Bundling from SDK `engine` workspace selected `wavecraft-nih_plug` and reported no plugin entry points for bundling.
- **Expected**: Bundle command in this context would produce plugin artifact.
- **Actual**: No plugin bundles found in `engine/target/bundled`.
- **Workaround**: Generate a fresh plugin project (`wavecraft create`) and run `wavecraft bundle --install` there.
- **Evidence**: xtask error output captured during test run.

## Testing Notes

- Core oscillator passthrough-mix behavior is strongly covered by unit tests and is currently passing.
- Manual Ableton listening checks are still required for milestone acceptance in primary target flow.
- Test environment now has an installed plugin artifact for manual verification:
  - `~/Library/Audio/Plug-Ins/VST3/osc_mix_test.vst3`

## Sign-off

- [x] All critical automated tests pass
- [x] All high-priority automated regressions pass
- [x] Issues documented
- [ ] Ready for release: **Pending manual Ableton verification**

---

## Retest Cycle (Post Follow-up Coder Fix)

- **Date**: 2026-02-18
- **Scope**: Verify stale generated contract refresh in `wavecraft bundle --install` flow
- **Fix Under Test**:
  - Root cause: stale `ui/src/generated/parameters.ts` / `processors.ts` could be installed
  - Fix location: `cli/src/commands/bundle_command.rs`
  - Added regression test: `cli/tests/bundle_command.rs`

### Additional Commands Executed and Outcomes

8. `cargo test --manifest-path /Users/ronhouben/code/private/wavecraft/cli/Cargo.toml --test bundle_command`
   - **Outcome**: ✅ `9 passed; 0 failed`
   - **Key evidence**: `test_bundle_refreshes_generated_contract_types_from_sidecars ... ok`

9. `cargo xtask ci-check`
   - **Outcome**: ✅ PASSED
   - **Evidence summary**:
     - Documentation links: pass
     - Lint + typecheck: pass
     - Engine tests: pass
     - UI tests: `20 passed` files, `96 passed` tests
     - Final summary: `All checks passed! Ready to push.`

### Retest Test Cases

#### TC-008: CLI bundle refreshes generated contracts before bundling (automated)

**Description**: Validate regression test proving stale generated contracts are overwritten by sidecar-derived contract generation before bundle.

**Expected Result**:

- Bundle flow refreshes `ui/src/generated/parameters.ts` and `ui/src/generated/processors.ts`
- Generated output contains oscillator contract IDs/types (not stale IDs)

**Status**: ✅ PASS

**Actual Result**:

- `test_bundle_refreshes_generated_contract_types_from_sidecars` passed in `cli/tests/bundle_command.rs`
- Assertions validated presence of:
  - `oscillator_enabled`
  - `oscillator_waveform`
  - `oscillator` processor registration

---

#### TC-009: Post-fix repository regression sweep (automated)

**Description**: Verify no cross-repo regressions after bundle command changes.

**Expected Result**: Standard CI checks pass.

**Status**: ✅ PASS

**Actual Result**:

- `cargo xtask ci-check` passed end-to-end (docs/lint/typecheck/tests)

---

## Updated Test Summary

| Status     | Count |
| ---------- | ----- |
| ✅ PASS    | 6     |
| ❌ FAIL    | 0     |
| ⏸️ BLOCKED | 3     |
| ⬜ NOT RUN | 0     |

## Manual Validation Status (unchanged in this terminal session)

Manual Ableton checks remain required for final acceptance:

- TC-005/006/007 are still blocked pending interactive DAW execution.
- No new automated failures introduced by the follow-up fix.
