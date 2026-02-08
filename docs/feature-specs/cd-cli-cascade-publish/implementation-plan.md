# Implementation Plan — CD CLI Cascade Publish

**Feature:** `cd-cli-cascade-publish`  
**Based on:** [Low-Level Design](./low-level-design-cd-cli-cascade-publish.md)  
**Created:** 2026-02-08

---

## Overview

Modify the Continuous Deploy workflow so that (1) all SDK distribution packages auto-bump their patch version when the local version is not ahead of the published version, and (2) the CLI always re-publishes when any SDK package changes, ensuring the git tag stays current for scaffolded projects.

Two files are modified: `.github/workflows/continuous-deploy.yml` and `docs/architecture/coding-standards.md`. No Rust or TypeScript code changes.

---

## Requirements

- Auto-bump patch versions for CLI, `@wavecraft/core`, and `@wavecraft/components` in CI when files change but version is not ahead
- CLI cascade: publish CLI when any SDK package changes (engine, npm-core, npm-components, or CLI itself)
- Prevent infinite loops from auto-bump commits re-triggering the pipeline
- Preserve intentional minor/major version bumps by developers
- No change to the plugin product version workflow (`engine/Cargo.toml` workspace version)

---

## Architecture Changes

| File | Change Summary |
|------|----------------|
| `.github/workflows/continuous-deploy.yml` | 4 changes: loop guard, npm auto-bump, CLI auto-bump, CLI cascade trigger |
| `docs/architecture/coding-standards.md` | Add SDK distribution versioning rule to the Versioning section |

---

## Implementation Steps

### Phase 1: Infinite Loop Guard + Aggregate Output

> **Goal:** Add the `[auto-bump]` guard and `any_sdk_changed` aggregate output to `detect-changes`. This is the foundation all other changes depend on.

#### Step 1.1 — Add `[auto-bump]` skip guard to `detect-changes` job

**File:** `.github/workflows/continuous-deploy.yml`  
**Location:** `detect-changes` job definition (line ~18)

**Action:** Add an `if` condition to the `detect-changes` job that skips if the triggering commit message contains `[auto-bump]`.

```yaml
detect-changes:
  runs-on: ubuntu-latest
  if: "!contains(github.event.head_commit.message, '[auto-bump]')"
```

**Why:** Prevents infinite loops: auto-bump commits push to `main`, which triggers the CD workflow again. This guard terminates the loop at the first job.

**Dependencies:** None  
**Risk:** Low — the `if` condition is standard GitHub Actions functionality

#### Step 1.2 — Add `any_sdk_changed` aggregate output

**File:** `.github/workflows/continuous-deploy.yml`  
**Location:** Inside `detect-changes` job, after the `paths-filter` step

**Action:**
1. Add `any_sdk_changed: ${{ steps.aggregate.outputs.any_sdk_changed }}` to the job `outputs`
2. Add a new step `Compute aggregate flag` that ORs all four filter outputs

```yaml
- name: Compute aggregate flag
  id: aggregate
  run: |
    if [[ "${{ steps.filter.outputs.cli }}" == "true" || \
          "${{ steps.filter.outputs.engine }}" == "true" || \
          "${{ steps.filter.outputs.npm-core }}" == "true" || \
          "${{ steps.filter.outputs.npm-components }}" == "true" ]]; then
      echo "any_sdk_changed=true" >> $GITHUB_OUTPUT
    else
      echo "any_sdk_changed=false" >> $GITHUB_OUTPUT
    fi
```

**Why:** The CLI cascade trigger needs a single flag to determine if any SDK package changed.

**Dependencies:** None  
**Risk:** Low

---

### Phase 2: npm Package Auto-Bump

> **Goal:** Replace the "skip if not ahead" guard in both npm publish jobs with auto-bump-then-publish logic.

#### Step 2.1 — Rewrite `publish-npm-core` version logic

**File:** `.github/workflows/continuous-deploy.yml`  
**Location:** `publish-npm-core` job, "Determine publish version" step (line ~275)

**Action:** Replace the current "Determine publish version" step with the three-step auto-bump logic:

1. **"Determine publish version"** — Compare local vs published. If local > published, set `needs_bump=false`. Otherwise, compute next patch version and set `needs_bump=true`.
2. **"Auto-bump patch version"** (conditional) — Run `npm version <new> --no-git-tag-version` to update `package.json`.
3. **"Commit and push auto-bump"** (conditional) — Configure git, `git add`, commit with `[auto-bump]` marker, `git pull --rebase origin main`, `git push origin main`.

Also: remove all `if: steps.version.outputs.skip != 'true'` guards from subsequent steps (build, dry-run, publish, tag), since the job-level `if` already gates on files changing, and the version is now guaranteed to be ahead after auto-bump.

**Why:** Eliminates the manual version bump requirement for `@wavecraft/core`.

**Dependencies:** Step 1.1 (loop guard must be in place before auto-bumps push to main)  
**Risk:** Medium — git push from CI, tested by Step 4.1

#### Step 2.2 — Rewrite `publish-npm-components` version logic

**File:** `.github/workflows/continuous-deploy.yml`  
**Location:** `publish-npm-components` job, "Determine publish version" step (line ~355)

**Action:** Same three-step pattern as Step 2.1, but for `@wavecraft/components`:
- Working directory: `ui/packages/components`
- npm view target: `@wavecraft/components`
- Git add target: `ui/packages/components/package.json`
- Commit message: `chore(npm): auto-bump @wavecraft/components to <version> [auto-bump]`

Also: ensure `git pull --rebase origin main` before push, since `publish-npm-core` may have pushed a commit in the same pipeline run.

**Why:** Same rationale as Step 2.1, for the components package.

**Dependencies:** Step 1.1  
**Risk:** Medium — potential rebase conflict if npm-core also pushed (mitigated by different files)

---

### Phase 3: CLI Auto-Bump + Cascade Trigger

> **Goal:** Make the CLI auto-bump and trigger on any SDK change.

#### Step 3.1 — Change `publish-cli` trigger condition and dependencies

**File:** `.github/workflows/continuous-deploy.yml`  
**Location:** `publish-cli` job definition (line ~53)

**Action:** Replace the current `needs` and `if`:

Current:
```yaml
publish-cli:
  needs: [detect-changes, publish-engine]
  if: needs.detect-changes.outputs.cli == 'true' && always()
```

New:
```yaml
publish-cli:
  needs: [detect-changes, publish-engine, publish-npm-core, publish-npm-components]
  if: |
    always() &&
    needs.detect-changes.outputs.any_sdk_changed == 'true' &&
    !cancelled() &&
    (needs.publish-engine.result == 'success' || needs.publish-engine.result == 'skipped') &&
    (needs.publish-npm-core.result == 'success' || needs.publish-npm-core.result == 'skipped') &&
    (needs.publish-npm-components.result == 'success' || needs.publish-npm-components.result == 'skipped')
```

**Why:** CLI becomes the final publish job that runs whenever any SDK package changed and all upstream jobs succeeded or were skipped.

**Dependencies:** Step 1.2 (needs `any_sdk_changed` output)  
**Risk:** Low — standard GitHub Actions `needs`/`if` pattern

#### Step 3.2 — Rewrite `publish-cli` version logic with auto-bump

**File:** `.github/workflows/continuous-deploy.yml`  
**Location:** `publish-cli` job, "Determine publish version" step (line ~126)

**Action:** Replace the current "Determine publish version" step with auto-bump logic:

1. **"Determine publish version"** — Compare local vs crates.io. If local > published, set `needs_bump=false`. Otherwise, compute next patch (`MAJOR.MINOR.PATCH+1`) and set `needs_bump=true`.
2. **"Auto-bump CLI patch version"** (conditional) — Use `sed` to update version in `cli/Cargo.toml`. Configure git, commit with `[auto-bump]` marker, `git pull --rebase origin main`, push.

Also: remove `if: steps.version.outputs.skip != 'true'` guards from subsequent steps (dry-run, auth, publish, tag). After auto-bump, the version is guaranteed to be ahead.

**Note:** The CLI job must `git pull --rebase origin main` before its own push because upstream npm jobs may have pushed auto-bump commits.

**Why:** Eliminates manual CLI version bump; enables cascade publishing.

**Dependencies:** Steps 1.1, 1.2, 3.1  
**Risk:** Medium — `sed -i` on `cli/Cargo.toml`. Mitigated by matching `^version = "` pattern in the `[package]` section.

#### Step 3.3 — Ensure CLI re-reads Cargo.toml after auto-bump

**File:** `.github/workflows/continuous-deploy.yml`  
**Location:** `publish-cli` job, between the auto-bump commit and the `cargo publish --dry-run` step

**Action:** After the auto-bump `git push`, the local `cli/Cargo.toml` is already updated (the `sed` happened locally). However, subsequent steps like `cargo publish` will re-read `Cargo.toml`, so no special action is needed. But the "Create and push git tag" step should read the version from the already-computed `publish_version` output instead of re-querying `cargo metadata`:

```yaml
- name: Create and push git tag
  run: |
    git pull --rebase origin main
    git tag "wavecraft-cli-v${{ steps.version.outputs.publish_version }}"
    git push origin "wavecraft-cli-v${{ steps.version.outputs.publish_version }}"
```

**Why:** Avoid git tag referencing a stale version if `cargo metadata` doesn't pick up the `sed` edit.

**Dependencies:** Step 3.2  
**Risk:** Low

---

### Phase 4: Update Coding Standards

> **Goal:** Document the new versioning policy.

#### Step 4.1 — Add SDK Distribution Versioning section

**File:** `docs/architecture/coding-standards.md`  
**Location:** After the existing "Versioning" subsection (after line ~982, before "Comments and Documentation")

**Action:** Add a new subsection `### SDK Distribution Versioning` that explains:
- SDK distribution versions (CLI, npm packages) are auto-bumped by CI at patch level
- Developers may still bump minor/major manually in PRs; CI respects versions that are already ahead
- Engine crates continue using `cargo-workspaces --from-git` (no change)
- The existing "PO decides, Coder bumps" rule applies only to the plugin product version

**Why:** Documents the new behavior and prevents confusion about "who bumps what."

**Dependencies:** None (can be done in parallel with workflow changes)  
**Risk:** Low — documentation only

---

## Testing Strategy

### Automated Verification

No unit or integration tests can validate GitHub Actions workflows locally. Testing is done via:

1. **YAML Syntax Validation:** Use `actionlint` or a YAML linter to verify the modified workflow parses correctly.
2. **Manual `workflow_dispatch` Trigger:** After merging, trigger the CD workflow manually to test the auto-bump logic path.

### Manual Scenario Testing

After merging this feature, verify the following scenarios by making targeted commits:

| # | Scenario | Test Method | Expected Result |
|---|----------|-------------|-----------------|
| 1 | Only engine crate changed | Merge PR touching `engine/crates/wavecraft-protocol/` | Engine publishes → CLI auto-bumps + publishes + tags |
| 2 | Only npm-core changed | Merge PR touching `ui/packages/core/` | npm-core auto-bumps + publishes → CLI auto-bumps + publishes |
| 3 | Only npm-components changed | Merge PR touching `ui/packages/components/` | npm-components auto-bumps + publishes → CLI auto-bumps + publishes |
| 4 | Only CLI source changed | Merge PR touching `cli/src/` | CLI auto-bumps + publishes (no upstream jobs) |
| 5 | Developer bumped CLI to 0.9.0 | Merge PR with CLI version pre-bumped | CLI publishes 0.9.0 as-is (no auto-bump) |
| 6 | Auto-bump commit is pushed | Observe CD after auto-bump commit lands | `detect-changes` skips → pipeline terminates |
| 7 | No SDK files changed | Merge docs-only PR | All jobs skipped |

### Risk Validation

| Risk | Verification |
|------|-------------|
| Infinite loop | Scenario 6 — verify `detect-changes` skips on `[auto-bump]` commits |
| Git push from CI | Scenario 1 — verify `github-actions[bot]` can push to `main` |
| Git rebase conflict | Scenario where both npm-core and npm-components change — verify both auto-bump + push succeed |

---

## Success Criteria

- [ ] `detect-changes` job skips when commit message contains `[auto-bump]`
- [ ] `any_sdk_changed` output is `true` when any SDK package changes
- [ ] `publish-npm-core` auto-bumps patch when local version == published
- [ ] `publish-npm-components` auto-bumps patch when local version == published
- [ ] `publish-cli` triggers on any SDK package change (not just CLI source)
- [ ] `publish-cli` auto-bumps patch when local version == published
- [ ] CLI git tag is created after every successful publish
- [ ] Developer-bumped versions are respected (no auto-bump if already ahead)
- [ ] No infinite loop: auto-bump commit does not re-trigger full pipeline
- [ ] Coding standards document updated with SDK distribution versioning rule

---

## Implementation Order Summary

```
Phase 1 (Foundation)
  ├── Step 1.1: [auto-bump] skip guard
  └── Step 1.2: any_sdk_changed aggregate output
          │
Phase 2 (npm auto-bump) — depends on Phase 1
  ├── Step 2.1: publish-npm-core auto-bump
  └── Step 2.2: publish-npm-components auto-bump
          │
Phase 3 (CLI cascade) — depends on Phase 1 + Phase 2
  ├── Step 3.1: CLI cascade trigger + dependencies
  ├── Step 3.2: CLI auto-bump logic
  └── Step 3.3: CLI git tag from computed version
          │
Phase 4 (Documentation) — independent, can parallel
  └── Step 4.1: Coding standards update
```

All phases target a **single PR** since the workflow changes are tightly coupled.
