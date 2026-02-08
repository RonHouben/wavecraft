# Implementation Plan: CD Auto-Bump Tag-Only Publishing

## Overview

The Continuous Deploy pipeline fails because auto-bump commits try to push directly to `main`, which is blocked by branch protection rulesets. The fix restructures all 4 publish jobs to commit version bumps locally only (for publish tooling) and push **tags only** — tags are not subject to branch protection rules. Documentation is updated to reflect the new behavior.

## Requirements

- All 4 publish jobs (`publish-cli`, `publish-engine`, `publish-npm-core`, `publish-npm-components`) must not push commits to `main`
- Git tags must still be created and pushed for each published version
- Published packages must contain the correct bumped version
- No changes to branch protection rules or additional secrets
- Documentation must reflect the new tag-only auto-bump behavior

## Architecture Changes

- [.github/workflows/continuous-deploy.yml](../../../.github/workflows/continuous-deploy.yml) — Remove `git push origin main` from all 4 publish jobs; add `--no-git-push` to `cargo ws publish`; manually tag engine crates
- [docs/guides/ci-pipeline.md](../../guides/ci-pipeline.md) — Update "Auto-Bump Pattern", "Infinite Loop Prevention", and "Git Conflict Prevention" sections
- [docs/architecture/coding-standards.md](../../architecture/coding-standards.md) — Update "SDK Distribution Versioning" section

## Implementation Steps

### Phase 1: Fix `publish-cli` Job

1. **Rename and update "Commit and push auto-bump" step** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Rename to "Commit auto-bump locally". Remove `git pull --rebase origin main` and `git push origin main`. Add comment explaining why.
   - Why: This is the step that fails. The local commit is still needed so `cargo publish` sees the correct version in `Cargo.toml`.
   - Dependencies: None
   - Risk: Low — `cargo publish` reads from the working tree, not the remote

2. **Update "Create and push git tag" step for CLI** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Remove `git pull --rebase origin main` (no longer needed since no commits were pushed that could conflict). Keep `git tag` and `git push origin <tag>`.
   - Why: Tags are not subject to branch protection. The rebase was only needed when commits were pushed first.
   - Dependencies: Step 1
   - Risk: Low

### Phase 2: Fix `publish-engine` Job

3. **Add `--no-git-push` to `cargo ws publish` command** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Add `--no-git-push` flag to both the dry-run and actual `cargo ws publish` invocations. This prevents `cargo-workspaces` from pushing commits to `main` while still allowing it to create local commits and tags.
   - Why: `cargo-workspaces` pushes commits and tags by default. We need to disable the push but keep local versioning.
   - Dependencies: None (independent of Phase 1)
   - Risk: Low — `--no-git-push` is a documented flag of `cargo-workspaces`

4. **Add manual tag push step for engine crates** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: After `cargo ws publish`, add a step that pushes only the tags created by `cargo-workspaces` using `git push origin --tags`. This ensures engine crate tags reach the remote.
   - Why: With `--no-git-push`, neither commits nor tags are pushed. We need tags pushed for traceability.
   - Dependencies: Step 3
   - Risk: Low — `git push --tags` pushes tags that exist locally but not on the remote

### Phase 3: Fix `publish-npm-core` Job

5. **Rename and update "Commit and push auto-bump" step for npm-core** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Same as Step 1 — rename to "Commit auto-bump locally", remove `git pull --rebase` and `git push origin main`.
   - Why: Same root cause — branch protection blocks direct push.
   - Dependencies: None (independent)
   - Risk: Low

6. **Update "Create and push git tag" step for npm-core** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Remove `git pull --rebase origin main`. Keep tag creation and push.
   - Dependencies: Step 5
   - Risk: Low

### Phase 4: Fix `publish-npm-components` Job

7. **Rename and update "Commit and push auto-bump" step for npm-components** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Same as Step 1 — rename to "Commit auto-bump locally", remove `git pull --rebase` and `git push origin main`.
   - Dependencies: None (independent)
   - Risk: Low

8. **Update "Create and push git tag" step for npm-components** (File: `.github/workflows/continuous-deploy.yml`)
   - Action: Remove `git pull --rebase origin main`. Keep tag creation and push.
   - Dependencies: Step 7
   - Risk: Low

### Phase 5: Update Documentation

9. **Update CI Pipeline Guide — Auto-Bump Pattern section** (File: `docs/guides/ci-pipeline.md`)
   - Action: Change step 3 from "Commit + push — Commit the version bump as `github-actions[bot]`, then push to `main`" to "Commit locally — Commit the version bump locally for publish tooling; version is **not** pushed to `main`". Add step about pushing tags only.
   - Dependencies: Steps 1–8 (ensures docs match implementation)
   - Risk: Low

10. **Update CI Pipeline Guide — Infinite Loop Prevention section** (File: `docs/guides/ci-pipeline.md`)
    - Action: Update to reflect that auto-bump commits are no longer pushed to `main`, so the infinite loop scenario no longer applies. Keep the `github-actions[bot]` author check as defense-in-depth.
    - Dependencies: Step 9
    - Risk: Low

11. **Update CI Pipeline Guide — Git Conflict Prevention section** (File: `docs/guides/ci-pipeline.md`)
    - Action: Simplify to note that since no commits are pushed to `main`, parallel job conflicts are no longer possible for version bumps. Only tag pushes remain (which use unique prefixes per package).
    - Dependencies: Step 9
    - Risk: Low

12. **Update Coding Standards — SDK Distribution Versioning section** (File: `docs/architecture/coding-standards.md`)
    - Action: Update "What CI does" and "Infinite loop prevention" bullets to reflect tag-only publishing. Clarify that the version in source files on `main` is the "product baseline" — the registry holds the authoritative published version.
    - Dependencies: Steps 1–8
    - Risk: Low

### Phase 6: Verify

13. **Trigger CD pipeline to verify fix** 
    - Action: After merging, push a change to `main` that touches an SDK component and verify the CD pipeline completes without branch protection errors. Verify tags are created on the remote.
    - Dependencies: All previous steps
    - Risk: Low — this is verification only

## Testing Strategy

- **CI validation**: Run `cargo xtask ci-check` to ensure no regressions in existing tests
- **YAML linting**: Validate workflow YAML syntax (GitHub Actions validates on push)
- **Manual verification**: After merge, trigger CD pipeline and confirm:
  1. All 4 publish jobs complete (or skip) without `GH013` errors
  2. Git tags are created and pushed for each published package
  3. Packages are published to their respective registries
  4. No commits are pushed to `main` by the pipeline

## Risks & Mitigations

- **Risk**: `cargo publish` doesn't see bumped version if not committed
  - Mitigation: The local commit step is preserved — `cargo publish` reads from the committed working tree
- **Risk**: `cargo ws publish --no-git-push` also prevents tag push
  - Mitigation: Step 4 adds an explicit `git push origin --tags` after `cargo ws publish`
- **Risk**: Version drift confusion (source vs published)
  - Mitigation: Steps 9–12 update all documentation to clarify the two-domain versioning model
- **Risk**: `cargo ws publish --from-git --no-git-push` flag combination not working as expected
  - Mitigation: The `--from-git` flag means "publish versions as they appear in git" — combining with `--no-git-push` should work since `--from-git` only affects version detection, not git operations. Verify in dry-run step.

## Success Criteria

- [ ] CD pipeline runs without `GH013` branch protection errors
- [ ] Git tags are created and pushed for all published packages
- [ ] Packages are published to crates.io and npm with correct versions
- [ ] No commits are pushed to `main` by the CD pipeline
- [ ] Documentation accurately describes the tag-only auto-bump behavior
