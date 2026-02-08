# Low-Level Design — CD Auto-Bump vs Branch Protection

**Status:** Draft  
**Created:** 2026-02-08  
**Author:** Architect Agent

---

## Problem Statement

The Continuous Deploy pipeline (`continuous-deploy.yml`) fails in the `publish-cli` job at the "Commit and push auto-bump" step. The job auto-bumps the CLI patch version in `cli/Cargo.toml`, commits as `github-actions[bot]`, and attempts to push directly to `main`. This push is rejected with:

```
remote: error: GH013: Repository rule violations found for refs/heads/main.
remote: - Changes must be made through a pull request.
remote: - 7 of 7 required status checks are expected.
remote: - Required status check "Validate Generated Template" is expected.
remote: - Cannot update this protected ref.
```

**Root cause:** The repository has branch protection rules (rulesets) on `main` that:
1. Require all changes go through a pull request
2. Require 7 status checks to pass (including "Validate Generated Template")

The `GITHUB_TOKEN` used by the workflow cannot bypass these rules. Unlike the legacy branch protection API where admins could select "Include administrators" or not, GitHub's newer **repository rulesets** enforce rules on all actors including `github-actions[bot]` unless the bot is explicitly added to a bypass list.

This same issue affects **all four publish jobs** that auto-bump:
- `publish-cli` — `cli/Cargo.toml`
- `publish-npm-core` — `ui/packages/core/package.json`
- `publish-npm-components` — `ui/packages/components/package.json`
- `publish-engine` — uses `cargo-workspaces` which also pushes to `main`

---

## Constraints

1. Branch protection rulesets **must stay enabled** — they protect `main` from broken code.
2. Auto-bump is needed because distribution packages need independent patch versions (see coding standards: "SDK Distribution Versioning").
3. The CD pipeline triggers on push to `main` — it runs post-merge, not as a PR.
4. Tags must correspond to the published version for CLI dependency resolution (`git = "...", tag = "wavecraft-cli-v0.x.y"`).

---

## Options Analysis

### Option A: Add `github-actions[bot]` to Ruleset Bypass List

**How:** In GitHub Repository Settings → Rules → Rulesets, add `github-actions[bot]` (or a GitHub App) to the bypass actor list.

| Aspect | Assessment |
|--------|-----------|
| Effort | Minimal (settings change, no code) |
| Risk | Medium — any workflow using `GITHUB_TOKEN` can push to `main` without PR review or status checks |
| Security | Weakens protection by exempting all GitHub Actions from rules |
| Maintenance | None — one-time configuration |

**Verdict:** Quick fix, but architecturally unsound. It opens a loophole where any workflow — including compromised or buggy ones — can push directly to main. Not recommended for a project with 7 required status checks.

---

### Option B: Use a Personal Access Token (PAT) or GitHub App Token with Bypass Permissions

**How:** Create a fine-grained PAT or GitHub App that is added to the ruleset bypass list. Use this token instead of `GITHUB_TOKEN` for the push step only.

| Aspect | Assessment |
|--------|-----------|
| Effort | Low (create token/app, add secret, update workflow) |
| Risk | Low — only the specific token has bypass, scoped to contents:write |
| Security | Better than Option A — bypass is limited to a specific identity |
| Maintenance | PAT requires renewal; GitHub App tokens are auto-managed |

**Verdict:** Viable but adds secrets management overhead. A GitHub App is preferred over PAT (auto-rotating, no user-bound expiry).

---

### Option C: Eliminate Direct Pushes — Publish from Detached HEAD (Recommended)

**How:** Restructure the CD workflow so auto-bump commits are **never pushed to `main`**. Instead:
1. Bump the version in the working tree
2. Commit locally (for cargo/npm publish to see the correct version)
3. Publish the crate/package
4. Push only the **git tag** (tags are not subject to branch protection)
5. The bumped version in `main` is "virtual" — it exists in the published artifact and the tag, but `main` stays at the pre-bump version

**Key insight:** The only reason version bumps are committed to `main` is to keep the source file in sync with the published version. But this sync is not actually needed:
- The published artifact on crates.io/npm has the correct version.
- The git tag records the exact version.
- The next CD run compares against the **registry** version, not the source file — so it will auto-bump again correctly.

| Aspect | Assessment |
|--------|-----------|
| Effort | Medium (workflow restructure, no token changes) |
| Risk | Low — main remains protected; published versions are traceable via tags |
| Security | Best — no bypasses, no extra tokens |
| Maintenance | Minimal — removes complexity of push-to-main |
| Traceability | Tags link published versions to exact commits |

**Tradeoff:** The version in source files on `main` won't always match the published version. This is acceptable because:
- The coding standards explicitly state: "Do not manually bump CLI or npm package versions unless making a deliberate breaking change."
- The published version is authoritative (crates.io/npm), not the source file.
- CI already compares against registry, not source.

**Verdict:** Recommended. Removes the conflict entirely without weakening protections.

---

### Option D: Version Bumps via Auto-Created PR

**How:** Instead of pushing directly, create a PR with the version bump, auto-approve it, and merge it.

| Aspect | Assessment |
|--------|-----------|
| Effort | High (needs PR creation, auto-approval, merge logic, re-trigger handling) |
| Risk | Medium — complex orchestration, potential for stuck PRs |
| Security | Good — changes still go through PR flow |
| Maintenance | High — PR automation is brittle |

**Verdict:** Over-engineered for a patch version bump. Introduces fragile automation that adds failure modes without proportional benefit.

---

## Recommended Solution: Option C — Tag-Only Publishing

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        REVISED CD FLOW (Option C)                           │
└─────────────────────────────────────────────────────────────────────────────┘

  Push to main (PR merge)
         │
         ▼
  ┌──────────────────┐
  │ detect-changes   │  Identify which packages changed
  └────────┬─────────┘
           │
     ┌─────┴─────┬──────────────┬───────────────┐
     ▼           ▼              ▼               ▼
  ┌────────┐ ┌──────────┐ ┌───────────┐  ┌───────────────┐
  │publish-│ │publish-  │ │publish-   │  │publish-       │
  │engine  │ │npm-core  │ │npm-comps  │  │cli            │
  └────┬───┘ └────┬─────┘ └─────┬─────┘  └───────┬───────┘
       │          │              │                │
       │    For each job:                         │
       │    ┌─────────────────────────────────┐   │
       │    │ 1. Determine version from       │   │
       │    │    registry (crates.io / npm)    │   │
       │    │ 2. Bump version in working tree  │   │
       │    │    (local only, NOT pushed)      │   │
       │    │ 3. Publish to registry           │   │
       │    │ 4. Tag commit with version       │   │
       │    │ 5. Push TAG only (not branch)    │   │
       │    └─────────────────────────────────┘   │
       │                                          │
       └──────────────────────────────────────────┘
                         │
                         ▼
               Tags pushed (not protected):
               wavecraft-cli-v0.8.6
               @wavecraft/core-v0.8.6
               etc.
```

### Changes Per Job

#### `publish-cli`

**Before (broken):**
```yaml
- name: Commit and push auto-bump
  if: steps.bump.outputs.bumped == 'true'
  run: |
    git config user.name "github-actions[bot]"
    git config user.email "github-actions[bot]@users.noreply.github.com"
    git add cli/Cargo.toml
    git commit -m "chore: bump wavecraft CLI to ${{ steps.bump.outputs.version }} [auto-bump]"
    git pull --rebase origin main
    git push origin main
```

**After (fixed):**
```yaml
- name: Commit auto-bump locally
  if: steps.bump.outputs.bumped == 'true'
  run: |
    git config user.name "github-actions[bot]"
    git config user.email "github-actions[bot]@users.noreply.github.com"
    git add cli/Cargo.toml
    git commit -m "chore: bump wavecraft CLI to ${{ steps.bump.outputs.version }} [auto-bump]"
    # NOTE: No push to main — branch protection prevents direct pushes.
    # The version bump exists locally for cargo publish and tagging only.
```

The "Create and push git tag" step remains unchanged — tags are not protected:
```yaml
- name: Create and push git tag
  run: |
    git tag "wavecraft-cli-v${{ steps.final.outputs.version }}"
    git push origin "wavecraft-cli-v${{ steps.final.outputs.version }}"
```

Remove `git pull --rebase origin main` from the tag step (no longer needed since we're not pulling commits).

#### `publish-npm-core` and `publish-npm-components`

Same pattern — remove `git push origin main`, keep tag push.

#### `publish-engine`

This job uses `cargo-workspaces` which has its own git push behavior. Options:
1. Use `--no-git-push` flag if available in `cargo ws publish`
2. Or restructure to not use `cargo-workspaces` for versioning

Investigate `cargo ws publish` flags to disable git push while keeping version bump and tagging.

---

### Infinite Loop Prevention — Simplified

**Current approach:** The `detect-changes` job skips if `github.event.head_commit.author.name == 'github-actions[bot]'`. This was designed to prevent auto-bump commits from re-triggering the pipeline.

**With Option C:** Since no commits are pushed to `main`, the infinite loop issue disappears entirely. The `detect-changes` guard can be kept as defense-in-depth but is no longer functionally needed.

---

### Impact on Downstream Consumers

CLI-generated plugins use git tags for SDK dependencies:
```toml
wavecraft = { package = "wavecraft-nih_plug", git = "https://github.com/RonHouben/wavecraft", tag = "wavecraft-cli-v0.7.1" }
```

Tags continue to be pushed, so this contract is preserved.

---

### Version Drift Consideration

With Option C, the version in `cli/Cargo.toml` on `main` will always be the "base" version (whatever was manually set), while published versions on crates.io will be auto-bumped patches ahead. Example:

| Source (`main`) | Published (crates.io) | Tag |
|:-:|:-:|:-:|
| 0.8.0 | 0.8.6 | wavecraft-cli-v0.8.6 |
| 0.8.0 | 0.8.7 | wavecraft-cli-v0.8.7 |

This is fine because:
1. The CD pipeline already compares against the **registry** version, not the source file
2. Manual version bumps (minor/major) are committed via PRs as before
3. The source file reflects the "product version" while registry versions reflect "distribution versions" — exactly matching the existing coding standards separation

---

## Migration Steps

1. **Update `continuous-deploy.yml`:**
   - All 4 publish jobs: replace `git push origin main` with a no-op comment
   - Remove `git pull --rebase origin main` from commit steps
   - Keep `git push origin <tag>` for all tag steps
   - For `publish-engine`: verify `cargo ws publish` flags to prevent branch push

2. **Update documentation:**
   - `docs/guides/ci-pipeline.md`: note that auto-bumps are tag-only
   - `docs/architecture/coding-standards.md`: clarify that source version on `main` is the product baseline, not the published version

3. **No changes to branch protection rules** — the whole point is to work within them.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `cargo publish` requires committed version | Low | High | Auto-bump commits locally before publish — already the current behavior, just without pushing |
| `cargo ws publish` pushes to remote by default | Medium | Medium | Verify `cargo ws publish` flags; if no disable flag, restructure engine publishing to not use `cargo ws` for push |
| Tag conflicts from parallel jobs | Low | Low | Each job tags with a unique prefix; `git pull --rebase` before tagging |
| Confusion about source vs published version | Low | Low | Document clearly in coding standards |

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Overall architecture
- [Coding Standards — SDK Distribution Versioning](../../architecture/coding-standards.md#sdk-distribution-versioning-ci-auto-bump) — Version bump policy
- [CI/CD Pipeline Guide](../../guides/ci-pipeline.md) — Pipeline architecture
