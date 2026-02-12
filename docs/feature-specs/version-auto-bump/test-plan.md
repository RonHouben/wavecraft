# Test Plan — version-auto-bump

## Scope

Validate PR #64 fixes the publish-engine failure by auto-bumping when local versions are behind published versions, and ensure no regressions in lint/tests.

## Environment

- OS: macOS
- Branch: bugfix/version-auto-bump
- PR: https://github.com/RonHouben/wavecraft/pull/64

## Test Cases

### TC-001: Automated Linting Checks

**Steps**

1. Run `cargo xtask ci-check`

**Expected**

- `cargo fmt`, `clippy`, ESLint, and Prettier all pass

**Result**

- ✅ PASS

### TC-002: Automated Tests

**Steps**

1. Run `cargo xtask ci-check`

**Expected**

- Engine and UI tests pass

**Result**

- ✅ PASS

### TC-003: Workflow Logic Review

**Steps**

1. Review `.github/workflows/continuous-deploy.yml` auto-bump logic for publish jobs

**Expected**

- If local < published, job uses auto-bump path instead of failing

**Result**

- ✅ PASS

### TC-004: Version Consistency Check

**Steps**

1. Verify version bumps in `engine/Cargo.toml` and `dev-server/Cargo.toml` align with published 0.12.6

**Expected**

- All related crate versions are updated to 0.12.6

**Result**

- ✅ PASS

## Test Summary

- Linting: ✅ PASS (8.6s)
- Tests: ✅ PASS (8.4s)
- Total: ✅ PASS (16.9s)

## Notes

No issues found. PR #64 is ready to merge. Post-merge, monitor the next continuous-deploy run to confirm the auto-bump path triggers correctly when local versions lag behind crates.io.
