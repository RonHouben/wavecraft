# Test Plan — `xtask-npm-outdated-check`

- **Date:** 2026-02-16
- **Branch:** `feature/xtask-npm-outdated-check`
- **Tester Verdict:** ✅ Ready for QA

## Scope and Acceptance References

This test plan validates the `sync-ui-versions` command behavior and CI integration for the `xtask-npm-outdated-check` feature.

Acceptance references:

- [Low-Level Design — sync-ui-versions](./low-level-design-sync-ui-versions.md)
- [Implementation Plan](./implementation-plan.md)

## Environment

- **OS:** macOS
- **Repository:** `wavecraft`
- **Validation basis:** Verified command outputs + integration test evidence

## Test Cases

| Command                                            | Expected                                                     | Observed                                                              | Result  |
| -------------------------------------------------- | ------------------------------------------------------------ | --------------------------------------------------------------------- | ------- |
| `cargo xtask ci-check`                             | Overall PASS; Documentation/Linting/Automated Tests all pass | PASS; Documentation: PASSED, Linting: PASSED, Automated Tests: PASSED | ✅ PASS |
| `cargo xtask sync-ui-versions --help`              | Exit `0`; usage/flags displayed                              | Exit `0`; flags shown                                                 | ✅ PASS |
| `cargo xtask sync-ui-versions`                     | Aligned state; exit `0`                                      | Aligned; exit `0`                                                     | ✅ PASS |
| `cargo xtask sync-ui-versions --check`             | Aligned state; exit `0`                                      | Aligned; exit `0`                                                     | ✅ PASS |
| `cargo xtask sync-ui-versions --apply`             | No changes required; exit `0`                                | No changes required; exit `0`                                         | ✅ PASS |
| `cargo xtask sync-ui-versions --check --apply`     | CLI argument conflict; exit `2`                              | clap conflict; exit `2`                                               | ✅ PASS |
| `cargo xtask sync-ui-versions --apply` (run again) | Idempotent no-op; exit `0`                                   | No-op; exit `0`                                                       | ✅ PASS |

## Exit Code Verification

Validated exit-code contract:

- **Exit `0`**: confirmed via default, `--check`, and `--apply` successful/aligned runs.
- **Exit `2`**: confirmed via conflict invocation `--check --apply` (clap conflict).
- **Exit `1`**: confirmed via integration test evidence in:
  - `engine/xtask/tests/sync_ui_versions.rs`
  - test: `check_mode_returns_one_when_scoped_drift_exists`

## Risks / Issues

- **Non-blocking caveat:** local unstaged manifest changes were present during verification.
- **Impact assessment:** non-blocking; behavior contract validated through command outputs and integration tests.

## Final Verdict

Feature validation is complete for documented acceptance scope.

**Recommendation:** ✅ Ready for QA.
