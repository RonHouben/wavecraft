# Test Plan & Report: New Project VST3 Build + Install

**Feature:** `new-project-vst3-build-install`  
**Date:** 2026-02-17  
**Owner:** Tester  
**Validation Status:** ✅ PASS

---

## Objective

Validate that the canonical new-project build/install flow is present, testable, and contract-enforced for macOS-first VST3 workflows, including positive and negative execution paths.

---

## Scope

This test pass covers:

- Canonical command surface and usability from the top-level CLI
- Delegated generated-project execution and diagnostics behavior
- Template xtask validation behavior
- CI/template contract enforcement points

Out of scope:

- DAW runtime behavior beyond command/contract validation
- Cross-platform install-path behavior (non-macOS)

---

## Commands Executed

| #   | Command                                                                          | Expected                                                    | Result        |
| --- | -------------------------------------------------------------------------------- | ----------------------------------------------------------- | ------------- |
| 1   | `cargo xtask ci-check`                                                           | Full standard checks pass                                   | ✅ PASS       |
| 2   | `cargo xtask ci-check --full`                                                    | Extended checks pass (including template/CD dry-run phases) | ✅ PASS       |
| 3   | `cargo test --manifest-path cli/Cargo.toml --test bundle_command -- --nocapture` | CLI bundle command test suite passes                        | ✅ PASS (5/5) |
| 4   | `cargo run --manifest-path cli/Cargo.toml -- --help`                             | `bundle` command appears in top-level help                  | ✅ PASS       |
| 5   | `cargo run --manifest-path cli/Cargo.toml -- bundle --install` (invalid context) | Exits non-zero with actionable guidance                     | ✅ PASS       |
| 6   | `cargo test --manifest-path sdk-template/engine/xtask/Cargo.toml -- --nocapture` | Template xtask test suite passes                            | ✅ PASS (3/3) |

---

## Contract Checks Verified

The following contract verification points were confirmed:

- `.github/workflows/template-validation.yml` includes `bundle --install` checks
- `engine/xtask/src/commands/validate_template.rs` contains assert/install-check logic

These checks ensure that command-contract drift is caught by both local validation and CI template validation pathways.

---

## Results Summary

All executed validations passed.

- No functional failures were observed in this test pass.
- Positive-path command surface and behavior validated.
- Negative-path invalid-context behavior validated.
- Delegated xtask contract and diagnostics coverage validated.
- Template/CI contract checks validated.

---

## Acceptance Criteria Verification

| Acceptance Item                                   | Verdict |
| ------------------------------------------------- | ------- |
| Canonical command presence/use                    | ✅ PASS |
| Invalid context behavior coverage                 | ✅ PASS |
| Delegated xtask failure-path diagnostics coverage | ✅ PASS |
| Template validation/CI contract checks            | ✅ PASS |

---

## Retest Addendum (Post-QA Fixes)

**Date:** 2026-02-17  
**Retest Verdict:** ✅ PASS

### Retest Focus

1. `wavecraft bundle` build-only and `wavecraft bundle --install` build+install contract
2. Artifact path alignment to `target/bundled` semantics
3. Positive-path delegation test for `wavecraft bundle --install`

### Commands and Outcomes

| Command                                                                               | Outcome                                |
| ------------------------------------------------------------------------------------- | -------------------------------------- |
| `cargo test --manifest-path cli/Cargo.toml --test bundle_command -- --nocapture`      | ✅ PASS (6/6)                          |
| `cargo test --manifest-path cli/Cargo.toml bundle::tests -- --nocapture`              | ✅ PASS (2/2)                          |
| `cargo run --manifest-path cli/Cargo.toml -- bundle --help`                           | ✅ PASS (help reflects fixed contract) |
| `cargo test --manifest-path engine/xtask/Cargo.toml validate_template -- --nocapture` | ✅ PASS                                |
| `cd engine && cargo xtask validate-template`                                          | ✅ PASS                                |

### Recommendation

✅ Ready for QA re-review (no coder rework required for tested items).

---

## Recommendation

✅ Route to **QA** for quality review. No coder rework is required based on this test pass.

---

## Related Documents

- [Implementation Plan](./implementation-plan.md)
- [Implementation Progress](./implementation-progress.md)
- [Low-Level Design](./low-level-design-new-project-vst3-build-install.md)
- [High-Level Design](../../architecture/high-level-design.md)
- [Coding Standards](../../architecture/coding-standards.md)
- [Agent Development Flow](../../architecture/agent-development-flow.md)
- [Roadmap](../../roadmap.md)
