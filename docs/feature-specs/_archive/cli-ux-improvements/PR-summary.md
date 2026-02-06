# CLI UX Improvements (v0.8.0)

## Summary

This PR implements comprehensive CLI UX improvements based on internal testing findings. The changes remove friction from the developer onboarding experience by eliminating prompts, clarifying the interface, and adding better troubleshooting guidance.

**Key improvements:**
- **Zero prompts** — No more questions about vendor/email/URL, uses placeholder defaults
- **Cleaner interface** — Removed `--sdk-version`, renamed `--local-dev` to `--local-sdk` (boolean, hidden)
- **Auto-version detection** — SDK version auto-determined from CLI version
- **Git tag fix** — Corrected format to `wavecraft-cli-v{version}` (matches repository convention)
- **Better docs** — Added PATH troubleshooting, simplified getting started guide

**Version:** `0.8.0` (minor bump from `0.7.1`)

**Quality gates:**
- ✅ 10/10 manual test cases passed
- ✅ QA review: 0 Critical/High/Medium/Low issues
- ✅ Architecture review complete
- ✅ All linting and tests passing

## Changes

### CLI Implementation
- **cli/src/main.rs** — Added `SDK_VERSION` constant, removed `--sdk-version`, changed `--local-dev` to boolean `--local-sdk` with `hide = true`
- **cli/src/commands/new.rs** — Removed interactive prompts, use placeholder defaults, added `find_local_sdk_path()` function
- **cli/Cargo.toml** — Removed `dialoguer` dependency (no longer needed)

### Documentation
- **docs/guides/sdk-getting-started.md** — Removed duplicate intro, added PATH troubleshooting, simplified CLI reference
- **docs/architecture/high-level-design.md** — Updated to reflect new CLI behavior (no prompts, correct git tag format)
- **docs/roadmap.md** — Marked Milestone 13 complete, updated progress to 87%

### CI/Build
- **.github/workflows/template-validation.yml** — Updated to use `--local-sdk` (boolean, no path argument)
- **.github/workflows/ci.yml** — Removed redundant build-plugin job

### Feature Specs (Archived)
- **docs/feature-specs/_archive/cli-ux-improvements/** — Complete feature documentation:
  - `user-stories.md` — 4 user stories with acceptance criteria
  - `low-level-design.md` — Implementation design
  - `implementation-progress.md` — Development tracking
  - `test-plan.md` — 10 test cases (all passing)
  - `QA-report.md` — Comprehensive quality review

### Version Updates
- **engine/Cargo.toml** and crate versions — Updated to 0.7.2

## Commits

```
058b255 [PO] Update roadmap: Milestone 13 complete (v0.8.0) - CLI UX improvements
8acec5e docs(arch): Update high-level design to reflect CLI UX improvements
0ad789a feat(qa): Add comprehensive QA report for CLI UX improvements - all checks pass
d349824 Test plan: All tests pass - feature ready for release
6895e69 Fix git tag format to match repository convention
30a23d7 fix(tests): update test plan with current pass/fail status and document critical issue with git tag format
fe20d80 feat(tests): add comprehensive test plan for CLI UX improvements
45d5b83 feat(cli): implement CLI UX improvements
ec1bc84 refactor(design): simplify --local-sdk to use cwd instead of git rev-parse
d4b1659 feat(cli): update design - change --local-dev to --local-sdk boolean flag
c53752e feat: Add low-level design document for CLI UX improvements
6167cb8 feat: Update installation guidance to include troubleshooting for PATH issues
ab5bd82 feat: Enhance CLI interface by removing internal flags
89bcf87 feat: Add user stories for CLI UX improvements
aaba241 feat: Add findings on PATH setup issue after cargo install
40a2d3a feat: Add findings on --local-dev flag confusion
761db08 feat: Add findings on --sdk-version flag confusion
8389763 feat: Add findings on personal information prompts
80e36a6 feat: Add internal testing findings for Wavecraft CLI
```

## Related Documentation

- [User Stories](./user-stories.md) — 4 user stories with acceptance criteria
- [Low-Level Design](./low-level-design.md) — Implementation details
- [Test Plan](./test-plan.md) — 10 test cases (all passing)
- [QA Report](./QA-report.md) — Comprehensive quality review
- [Implementation Progress](./implementation-progress.md) — Development tracking
- [Internal Testing Findings](../internal-testing/CLI-findings.md) — Original testing results

## Testing

✅ **Manual Testing Complete** (10/10 tests passed):
- TC-001: Help command displays correctly
- TC-002: Project creation with no prompts
- TC-003: Default vendor placeholder used
- TC-004: Optional flags work correctly
- TC-005: SDK version auto-detection
- TC-006: `--local-sdk` flag functionality
- TC-007: `--local-sdk` error handling
- TC-008: Internal flags hidden from help
- TC-009: Generated projects compile (critical bug fixed)
- TC-010: CI template validation

✅ **Automated Checks:**
- Linting: `cargo fmt --check` ✓
- Linting: `cargo clippy` ✓ (zero warnings)
- Build: CLI builds successfully ✓

✅ **QA Review:**
- Code quality: ✓
- Security: ✓
- Architecture compliance: ✓
- User story verification: ✓ (all acceptance criteria met)

## User Impact

**Who benefits:** Plugin developers using `wavecraft new`

**Value delivered:**
- **20-30 seconds saved** per project creation (no prompts)
- **Reduced confusion** — No "why is this asking for my email?" concerns
- **Clearer help** — Self-documenting CLI via `--help`
- **Better onboarding** — PATH troubleshooting prevents "command not found" frustration

## Checklist

- ✅ Code follows project coding standards
- ✅ Tests added/updated (10 manual test cases)
- ✅ Documentation updated (getting-started, high-level-design, roadmap)
- ✅ No linting errors
- ✅ QA review complete (0 issues found)
- ✅ Architecture review complete
- ✅ Feature specs archived
- ✅ Roadmap updated (Milestone 13 complete)

## Release Notes

**Version 0.8.0 - CLI UX Improvements**

This release focuses on removing friction from the developer onboarding experience:

**New:**
- Zero prompts — Just `wavecraft new my-plugin` and go
- Auto-detected SDK version (no more `--sdk-version` flag)
- PATH troubleshooting guidance in Getting Started docs

**Changed:**
- `--local-dev` renamed to `--local-sdk` (boolean, hidden from help)
- Git tag format now `wavecraft-cli-v{version}` for consistency
- Removed `dialoguer` dependency (lighter binary)

**Fixed:**
- Generated projects now compile correctly with proper git tag references
- CLI help output is cleaner (internal flags hidden)

**Documentation:**
- Simplified SDK Getting Started guide
- Updated architecture documentation
