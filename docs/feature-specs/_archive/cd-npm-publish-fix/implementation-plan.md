# Implementation Plan: Fix CD npm Publish Version Drift

## Overview

The Continuous Deploy workflow fails when publishing `@wavecraft/core` and `@wavecraft/components` to npm because the in-repo `package.json` versions have drifted behind the npm registry. The current "check-then-bump-once" strategy can only handle a 1-patch gap; any larger drift (caused by version bump commits not landing in git) results in permanent failure. This plan fixes the immediate version drift, replaces the fragile version resolution with a registry-aware strategy, and adds structural protections.

## Requirements

- CD workflow must publish `@wavecraft/core` and `@wavecraft/components` successfully
- Version resolution must be idempotent — safe to re-run any number of times
- Concurrent workflow runs must not collide
- The npm registry is the source of truth for "what's the latest published patch"
- `package.json` version represents the *minimum* version (intentional minor/major bumps)
- Components publish job must not be blocked by core when only components changed

## Root Cause Analysis

### Current State (broken)

| Package | `package.json` (git) | npm latest | Gap |
|---------|---------------------|------------|-----|
| `@wavecraft/core` | `0.7.2` | `0.7.3` | 1 patch behind |
| `@wavecraft/components` | `0.7.2` | `0.7.3` | 1 patch behind |

### Failure Sequence

1. Workflow reads `package.json` → `0.7.2`
2. Checks `npm view @wavecraft/core@0.7.2` → exists → `published=true`
3. `npm version patch` → bumps to `0.7.3`
4. `npm publish --dry-run` → **fails**: `0.7.3` already published

The gap occurred because a previous run successfully published `0.7.3` but the version bump commit never landed in git (push failed or was superseded by a concurrent merge).

### Structural Problems

1. **Non-idempotent version bump**: Single `npm version patch` assumes the repo is exactly 1 patch behind. If n>=2 behind, it fails permanently.
2. **Publish-before-commit order**: The workflow publishes first, then tries to commit the version bump. If the commit/push fails, the registry drifts ahead of git permanently.
3. **No concurrency control**: Two simultaneous runs can read the same version, both try to publish, one fails.
4. **Components blocked by core unnecessarily**: If only components changed and core ran (and failed), components is skipped even though it has no dependency on core's publish step.

## Architecture Changes

- **File:** `.github/workflows/continuous-deploy.yml` — Replace npm version resolution logic in Jobs 4 and 5
- **File:** `.github/workflows/continuous-deploy.yml` — Add top-level concurrency group
- **File:** `.github/workflows/continuous-deploy.yml` — Fix `publish-npm-components` conditional to handle core failures when core wasn't needed
- **File:** `ui/packages/core/package.json` — Bump version past npm drift (immediate fix)
- **File:** `ui/packages/components/package.json` — Bump version past npm drift (immediate fix)

## Implementation Steps

### Phase 1: Immediate Fix — Sync Local Versions Past npm

> Goal: Unblock the CD pipeline by aligning `package.json` versions ahead of npm.

#### Step 1.1: Bump `@wavecraft/core` to `0.7.4` 
**File:** `ui/packages/core/package.json`
- Action: Change `"version": "0.7.2"` to `"version": "0.7.4"`
- Why: npm latest is `0.7.3`, so `0.7.4` will be treated as "not yet published" and the workflow will publish it directly without needing the bump logic
- Dependencies: None
- Risk: Low — this is the new minimum version going forward

#### Step 1.2: Bump `@wavecraft/components` to `0.7.4`
**File:** `ui/packages/components/package.json`
- Action: Change `"version": "0.7.2"` to `"version": "0.7.4"`
- Why: Same reason as core — npm latest is `0.7.3`
- Dependencies: None
- Risk: Low

---

### Phase 2: Robust Version Resolution — Registry-Aware Strategy

> Goal: Replace the fragile "check + single patch bump" with a strategy that queries the npm registry and computes the correct next version regardless of drift.

#### Step 2.1: Replace version check + bump in `publish-npm-core`
**File:** `.github/workflows/continuous-deploy.yml` (Job 4: publish-npm-core)

**Current flow (lines ~272–296):**
```yaml
- name: Get current version
- name: Check if already published
- name: Bump patch version (conditional)
```

**Replace with a single "Determine publish version" step:**

```yaml
- name: Determine publish version
  id: version
  working-directory: ui/packages/core
  run: |
    LOCAL=$(node -p "require('./package.json').version")
    LATEST=$(npm view @wavecraft/core version 2>/dev/null || echo "0.0.0")
    echo "local=$LOCAL" >> $GITHUB_OUTPUT
    echo "latest=$LATEST" >> $GITHUB_OUTPUT
    
    # Compare: if local > latest, publish local (intentional bump)
    # If local <= latest, compute latest + patch
    if npx --yes semver "$LOCAL" -r ">$LATEST" > /dev/null 2>&1; then
      echo "publish=$LOCAL" >> $GITHUB_OUTPUT
      echo "bumped=false" >> $GITHUB_OUTPUT
      echo "Publishing local version: $LOCAL (ahead of npm: $LATEST)"
    else
      NEXT=$(npx --yes semver "$LATEST" -i patch)
      npm version "$NEXT" --no-git-tag-version
      echo "publish=$NEXT" >> $GITHUB_OUTPUT
      echo "bumped=true" >> $GITHUB_OUTPUT
      echo "Bumping to: $NEXT (local: $LOCAL, npm: $LATEST)"
    fi
```

- Action: Remove the three separate steps (`Get current version`, `Check if already published`, `Bump patch version`) and replace with the single step above
- Why: This handles any size of version drift by computing next version from the registry
- Dependencies: None
- Risk: Medium — need to verify `npx semver` is available in the runner (it is, via Node.js setup)

**Also update downstream references:**
- The `Bump patch version` condition `if: steps.check.outputs.published == 'true'` → becomes part of the unified step (the `bumped` output replaces this)
- The `Commit version bump` condition changes from `if: steps.check.outputs.published == 'true'` to `if: steps.version.outputs.bumped == 'true'`
- The `Build package` step stays unchanged (runs after version is resolved)
- The `Verify publishability (dry-run)` step stays unchanged

#### Step 2.2: Replace version check + bump in `publish-npm-components`
**File:** `.github/workflows/continuous-deploy.yml` (Job 5: publish-npm-components)

Same pattern as Step 2.1 but for `@wavecraft/components`:

```yaml
- name: Determine publish version
  id: version
  working-directory: ui/packages/components
  run: |
    LOCAL=$(node -p "require('./package.json').version")
    LATEST=$(npm view @wavecraft/components version 2>/dev/null || echo "0.0.0")
    echo "local=$LOCAL" >> $GITHUB_OUTPUT
    echo "latest=$LATEST" >> $GITHUB_OUTPUT
    
    if npx --yes semver "$LOCAL" -r ">$LATEST" > /dev/null 2>&1; then
      echo "publish=$LOCAL" >> $GITHUB_OUTPUT
      echo "bumped=false" >> $GITHUB_OUTPUT
      echo "Publishing local version: $LOCAL (ahead of npm: $LATEST)"
    else
      NEXT=$(npx --yes semver "$LATEST" -i patch)
      npm version "$NEXT" --no-git-tag-version
      echo "publish=$NEXT" >> $GITHUB_OUTPUT
      echo "bumped=true" >> $GITHUB_OUTPUT
      echo "Bumping to: $NEXT (local: $LOCAL, npm: $LATEST)"
    fi
```

- Action: Same replacement as Step 2.1, targeting the components job
- Why: Same version drift protection
- Dependencies: Step 2.1 (same pattern, apply consistently)
- Risk: Low

#### Step 2.3: Update `Commit version bump` conditions in both jobs
**File:** `.github/workflows/continuous-deploy.yml`

For both `publish-npm-core` (Job 4) and `publish-npm-components` (Job 5):

- Action: Change `if: steps.check.outputs.published == 'true'` on the "Commit version bump" step to `if: steps.version.outputs.bumped == 'true'`
- Why: The old `check` step ID no longer exists; the new `version` step outputs `bumped`
- Dependencies: Steps 2.1 and 2.2
- Risk: Low

#### Step 2.4: Update git tag version references
**File:** `.github/workflows/continuous-deploy.yml`

For both npm jobs, the "Create and push git tag" step currently reads the version from `package.json`. This still works, but verify the step uses the same version that was published.

- Action: No code change needed — the `package.json` was already modified by `npm version` in the determine step, so reading it at tag time gives the correct version
- Why: Confirm correctness, no drift
- Dependencies: Steps 2.1, 2.2
- Risk: None

---

### Phase 3: Structural Hardening

> Goal: Prevent concurrent runs from colliding and ensure components isn't unnecessarily blocked.

#### Step 3.1: Add workflow concurrency group
**File:** `.github/workflows/continuous-deploy.yml` (top level, after `on:`)

```yaml
concurrency:
  group: continuous-deploy
  cancel-in-progress: false
```

- Action: Add this block at the top level of the workflow, between the `on:` trigger and `jobs:`
- Why: Prevents two CD runs (e.g., a push + manual dispatch) from running simultaneously. `cancel-in-progress: false` ensures the first run completes rather than being cancelled by a later one
- Dependencies: None
- Risk: Low — only queues subsequent runs, doesn't cancel them

#### Step 3.2: Fix `publish-npm-components` conditional
**File:** `.github/workflows/continuous-deploy.yml` (Job 5 `if:`)

**Current:**
```yaml
if: |
  always() &&
  needs.detect-changes.outputs.npm-components == 'true' &&
  (needs.publish-npm-core.result == 'success' || needs.publish-npm-core.result == 'skipped')
```

**Replace with:**
```yaml
if: |
  always() &&
  needs.detect-changes.outputs.npm-components == 'true' &&
  needs.publish-npm-core.result != 'cancelled'
```

- Action: Change the condition to only block on cancellation, not on core's failure
- Why: `@wavecraft/components` has a *peer dependency* on `@wavecraft/core`, not a build-time dependency. If core fails to publish (e.g., version issue), that doesn't affect whether components can publish. The only real blocker is if core changes were meant to be consumed first (which is already handled by the `git pull` step)
- Dependencies: None
- Risk: Medium — need to verify that a failed core publish doesn't leave the npm registry in a state that breaks components. Since they're independent packages with only a peer dependency, this is safe.

**Alternative (more conservative):**
If we want to keep the safety guarantee that components only publishes when core is healthy:
```yaml
if: |
  always() &&
  needs.detect-changes.outputs.npm-components == 'true' &&
  (needs.publish-npm-core.result == 'success' || needs.publish-npm-core.result == 'skipped') ||
  (needs.publish-npm-core.result == 'failure' && needs.detect-changes.outputs.npm-core != 'true')
```
This allows components to proceed when core failed BUT core wasn't supposed to run (i.e., no core changes detected).

**Recommended:** Use the conservative option. It covers the cascading-skip scenario while preserving safety when core actually had changes.

---

### Phase 4: CLI Version Resolution (same pattern)

> Goal: Apply the same registry-aware pattern to the CLI publish job to prevent similar drift.

#### Step 4.1: Replace CLI version check with registry-aware strategy
**File:** `.github/workflows/continuous-deploy.yml` (Job 2: publish-cli)

**Current flow (lines ~125–145):**
```yaml
- name: Get current version
- name: Check if already published
- name: Bump patch version (conditional)
```

**Replace with:**
```yaml
- name: Determine publish version
  id: version
  run: |
    CURRENT=$(cargo metadata --manifest-path cli/Cargo.toml --no-deps --format-version 1 | jq -r '.packages[0].version')
    PUBLISHED=$(curl -s "https://index.crates.io/wa/ve/wavecraft" | tail -1 | jq -r '.vers' 2>/dev/null || echo "0.0.0")
    echo "current=$CURRENT" >> $GITHUB_OUTPUT
    echo "published=$PUBLISHED" >> $GITHUB_OUTPUT
    
    if [ "$(printf '%s\n' "$PUBLISHED" "$CURRENT" | sort -V | tail -1)" = "$CURRENT" ] && [ "$CURRENT" != "$PUBLISHED" ]; then
      echo "publish=$CURRENT" >> $GITHUB_OUTPUT
      echo "bumped=false" >> $GITHUB_OUTPUT
      echo "Publishing current version: $CURRENT (latest on crates.io: $PUBLISHED)"
    else
      NEW=$(echo "$PUBLISHED" | awk -F. '{print $1"."$2"."$3+1}')
      sed -i "s/^version = \"$CURRENT\"/version = \"$NEW\"/" cli/Cargo.toml
      echo "publish=$NEW" >> $GITHUB_OUTPUT
      echo "bumped=true" >> $GITHUB_OUTPUT
      echo "Bumping to: $NEW (current: $CURRENT, crates.io: $PUBLISHED)"
    fi
```

- Action: Replace the three steps with a single registry-aware version determination
- Why: Same drift protection as npm jobs
- Dependencies: None
- Risk: Low — uses the same crates.io index API already in use

#### Step 4.2: Update `Commit version bump` condition for CLI
**File:** `.github/workflows/continuous-deploy.yml` (Job 2)

- Action: Change `if: steps.bump.outputs.bumped == 'true'` to `if: steps.version.outputs.bumped == 'true'`
- Why: Step ID changed from `bump` to `version`
- Dependencies: Step 4.1
- Risk: Low

---

## Testing Strategy

### Automated Testing
- **Dry-run validation:** After changes, trigger `workflow_dispatch` and verify all jobs pass
- **Version math:** Manually verify the `npx semver` comparison works for edge cases:
  - local `0.7.4` > npm `0.7.3` → publish `0.7.4` (no bump)
  - local `0.7.2` < npm `0.7.5` → publish `0.7.6` (bump from registry)
  - local `0.8.0` > npm `0.7.5` → publish `0.8.0` (intentional minor, no bump)
  - local `0.7.2`, npm doesn't exist → publish `0.7.2` (first publish)

### Manual Testing
1. Push the changes to main → verify push-triggered CD succeeds
2. Trigger `workflow_dispatch` when no changes exist → verify all jobs skip correctly
3. Trigger `workflow_dispatch` → verify jobs detect "already published" and skip or bump
4. Verify git tags are created correctly after successful publish
5. Verify npm shows the expected version: `npm view @wavecraft/core version`

### Regression Testing
- Verify `cargo xtask ci-check` still passes (workflow changes don't affect local dev)
- Verify template `package.json` in `cli/sdk-templates/` still references compatible versions

## Risks & Mitigations

- **Risk:** `npx semver` not available in GitHub Actions runner
  - Mitigation: The runner has Node.js installed via `actions/setup-node@v4`. `npx` comes with npm. The `--yes` flag auto-installs `semver` if not cached. We can also pin it: `npx --yes semver@7`
  
- **Risk:** Race condition between determine-version and publish (another run publishes the same version between the two steps)
  - Mitigation: The concurrency group (Phase 3, Step 3.1) prevents concurrent runs entirely

- **Risk:** Version bump commit fails to push (e.g., branch protection, merge conflict)
  - Mitigation: This is benign now — the next run will query the registry and compute the correct version regardless. The commit is a "nice to have" for keeping git in sync, not a requirement for correctness.

- **Risk:** `sort -V` not available on ubuntu runner for CLI version comparison
  - Mitigation: `sort -V` (version sort) is available on Ubuntu 20.04+. The runner uses Ubuntu 24.04. As a fallback, could use `dpkg --compare-versions`.

## Success Criteria

- [ ] `workflow_dispatch` of CD pipeline completes with all 5 jobs either succeeding or correctly skipping
- [ ] `@wavecraft/core` version on npm is >= `0.7.4`
- [ ] `@wavecraft/components` version on npm is >= `0.7.4`
- [ ] Re-running `workflow_dispatch` immediately after a successful run results in all publish jobs skipping (idempotent)
- [ ] Concurrent `workflow_dispatch` triggers don't collide (second queues until first finishes)
- [ ] If only components changes are detected and core has no changes, components publishes even if core had a prior failure
