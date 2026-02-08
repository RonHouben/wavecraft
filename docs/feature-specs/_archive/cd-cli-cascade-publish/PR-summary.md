## Summary

Enhance the Continuous Deployment pipeline with CLI cascade triggers, `[auto-bump]` loop prevention, and a publish-only npm model. This ensures that when **any** SDK component changes (engine crates, npm packages, or CLI itself), the CLI is also re-published with an updated git tag — keeping the SDK distribution consistent.

Key improvements:
- **CLI cascade trigger** — CLI re-publishes whenever any SDK component changes, ensuring the git tag always references the latest SDK state
- **`[auto-bump]` loop prevention** — Replaces `[skip ci]` with `[auto-bump]` commit marker so other workflows (CI, template validation) still run on auto-bump commits
- **Publish-only npm model** — npm jobs no longer build; they publish the pre-built `dist/` already committed in the repo
- **Upstream failure guards** — `!cancelled()` replaces `always()` to prevent cascade publishing when upstream jobs fail

## Changes

- **CI/CD**: Rewrote `continuous-deploy.yml` with auto-bump logic, cascade triggers, loop prevention, and upstream failure guards
- **Documentation**: Updated `coding-standards.md` (SDK Distribution Versioning), `high-level-design.md` (CD section), `ci-pipeline.md` (full CD rewrite with diagrams), `roadmap.md` (changelog + M12 tasks)
- **Build/Config**: Added `target/tmp/` to `.gitignore`
- **Feature Specs**: Full spec lifecycle (LLD, impl plan, impl progress, test plan, QA report) — archived to `_archive/`

## Commits

- `9935b87` feat: CLI cascade publish + CI auto-bump for distribution packages
- `87bf1c6` fix: scope CLI sed to [package] section & add upstream failure guards
- `56f551d` feat(cd): implement publish-only model for npm packages and CLI
- `45acf45` fix: update .gitignore to include target/tmp and revise test plan results
- `735f9a6` docs: update roadmap and archive cd-cli-cascade-publish feature spec

## Related Documentation

- [Implementation Plan](./implementation-plan.md)
- [Low-Level Design](./low-level-design-cd-cli-cascade-publish.md)
- [Test Plan](./test-plan.md) — 12/12 TCs PASS
- [QA Report](./QA-report.md) — PASS (0 Critical/High/Medium)
- [CI Pipeline Guide](../../../guides/ci-pipeline.md) — CD section fully rewritten

## Testing

- [x] Linting passes: `cargo xtask lint`
- [x] Tests pass: `cargo xtask test` (165 Rust + 28 UI tests)
- [x] CI checks pass: `cargo xtask ci-check`
- [x] 12/12 manual test cases pass (YAML validation, trigger logic, auto-bump, cascade, loop prevention)
- [x] QA review approved (0 Critical/High/Medium findings)
- [x] Link checker: 0 broken links

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated (coding-standards, high-level-design, ci-pipeline, roadmap)
- [x] No linting errors (`cargo xtask lint`)
- [x] Feature spec archived to `_archive/`
