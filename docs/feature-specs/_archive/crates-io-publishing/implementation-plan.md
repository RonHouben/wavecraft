# Implementation Plan: crates.io Publishing with cargo-workspaces

## Overview

Migrate the continuous deploy workflow from broken manual `cargo publish` to cargo-workspaces, enabling automated crates.io publishing with independent versioning, crate-specific git tags, and automatic dependency cascade handling.

## Requirements

- Switch from unified workspace version to independent crate versions
- Install and use cargo-workspaces for publishing
- Add crate-specific git tags for all published packages (Rust and npm)
- Add missing metadata (repository, description) to Cargo.toml files
- Exclude `standalone` crate from publishing

## Architecture Changes

| File | Change |
|------|--------|
| `engine/Cargo.toml` | Remove `version` from workspace, add `repository` |
| `engine/crates/*/Cargo.toml` | Replace `version.workspace` with explicit version |
| `engine/crates/standalone/Cargo.toml` | Add `publish = false` |
| `.github/workflows/continuous-deploy.yml` | Replace manual publish loop with cargo-workspaces |

---

## Implementation Steps

### Phase 1: Cargo.toml Metadata Updates

#### Step 1.1: Update Workspace Root
**File:** `engine/Cargo.toml`

**Action:** Remove `version` from `[workspace.package]` and add `repository` field.

```toml
# BEFORE
[workspace.package]
version = "0.7.1"
edition = "2024"
license = "MIT"
authors = ["Wavecraft Team"]

# AFTER
[workspace.package]
edition = "2024"
license = "MIT"
authors = ["Wavecraft Team"]
repository = "https://github.com/RonHouben/wavecraft"
```

**Why:** Independent versioning requires each crate to own its version. Repository field improves crates.io discoverability.

**Dependencies:** None
**Risk:** Low

---

#### Step 1.2: Update wavecraft-protocol
**File:** `engine/crates/wavecraft-protocol/Cargo.toml`

**Action:** Replace `version.workspace = true` with explicit `version = "0.7.1"`. Add `repository.workspace = true`.

**Dependencies:** Step 1.1
**Risk:** Low

---

#### Step 1.3: Update wavecraft-macros
**File:** `engine/crates/wavecraft-macros/Cargo.toml`

**Action:** 
1. Replace `version.workspace = true` with `version = "0.7.1"`
2. Add `authors.workspace = true`
3. Add `description = "Procedural macros for Wavecraft plugin DSL"`
4. Add `repository.workspace = true`

**Why:** This crate is missing required metadata for crates.io.

**Dependencies:** Step 1.1
**Risk:** Low

---

#### Step 1.4: Update wavecraft-metering
**File:** `engine/crates/wavecraft-metering/Cargo.toml`

**Action:**
1. Replace `version.workspace = true` with `version = "0.7.1"`
2. Add `description = "Real-time safe SPSC metering for Wavecraft audio plugins"`
3. Add `repository.workspace = true`

**Dependencies:** Step 1.1
**Risk:** Low

---

#### Step 1.5: Update wavecraft-dsp
**File:** `engine/crates/wavecraft-dsp/Cargo.toml`

**Action:**
1. Replace `version.workspace = true` with `version = "0.7.1"`
2. Add `repository.workspace = true`

**Dependencies:** Step 1.1
**Risk:** Low

---

#### Step 1.6: Update wavecraft-bridge
**File:** `engine/crates/wavecraft-bridge/Cargo.toml`

**Action:**
1. Replace `version.workspace = true` with `version = "0.7.1"`
2. Add `repository.workspace = true`

**Dependencies:** Step 1.1
**Risk:** Low

---

#### Step 1.7: Update wavecraft-core
**File:** `engine/crates/wavecraft-core/Cargo.toml`

**Action:**
1. Replace `version.workspace = true` with `version = "0.7.1"`
2. Add `repository.workspace = true`

**Dependencies:** Step 1.1
**Risk:** Low

---

#### Step 1.8: Exclude standalone from Publishing
**File:** `engine/crates/standalone/Cargo.toml`

**Action:** Add `publish = false` to `[package]` section.

```toml
[package]
name = "standalone"
version.workspace = true  # Can keep workspace version since not published
edition.workspace = true
license.workspace = true
authors.workspace = true
description = "Standalone app for plugin development and testing"
publish = false  # ← ADD THIS
```

**Why:** standalone is for local development only, never intended for crates.io.

**Dependencies:** None
**Risk:** Low

---

#### Step 1.9: Verify Workspace Compiles
**Action:** Run verification command

```bash
cd engine && cargo check --workspace
```

**Why:** Ensure version migration didn't break anything.

**Dependencies:** Steps 1.1–1.8
**Risk:** Low

---

### Phase 2: Engine Workflow Update

#### Step 2.1: Rewrite publish-engine Job
**File:** `.github/workflows/continuous-deploy.yml`

**Action:** Replace the entire `publish-engine` job (lines ~107-190) with cargo-workspaces version.

**Current code to replace:**
```yaml
  publish-engine:
    needs: detect-changes
    if: needs.detect-changes.outputs.engine == 'true'
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
      # ... rest of manual publish loop
```

**New code:**
```yaml
  # ============================================================================
  # Job 3: Publish engine crates to crates.io
  # ============================================================================
  publish-engine:
    needs: detect-changes
    if: needs.detect-changes.outputs.engine == 'true'
    runs-on: ubuntu-latest
    permissions:
      contents: write  # For version bump commits and git tags
    steps:
      - uses: actions/checkout@v4
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          fetch-depth: 0  # Full history for cargo-workspaces version detection

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: engine

      - name: Install cargo-workspaces
        run: cargo install cargo-workspaces

      - name: Configure git
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "github-actions[bot]@users.noreply.github.com"

      - name: Verify publishability (dry-run)
        working-directory: engine
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          cargo ws publish \
            --from-git \
            --dry-run \
            --yes \
            --allow-branch main

      - name: Publish to crates.io (with version bump and tags)
        working-directory: engine
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          cargo ws publish \
            --from-git \
            --yes \
            --allow-branch main
          # cargo-workspaces automatically:
          # - Bumps patch version if current version already published
          # - Creates git tags: wavecraft-protocol-v0.7.2, wavecraft-core-v0.7.2, etc.
          # - Commits version changes
          # - Pushes commits and tags
```

**Key changes:**
- Added `fetch-depth: 0` for full git history
- Added `cargo install cargo-workspaces` step
- Added `Configure git` step for commits/tags
- Added dry-run verification step
- Replaced manual publish loop with single `cargo ws publish` command
- Removed hardcoded crate list and 30s sleep loops

**Dependencies:** Phase 1 complete
**Risk:** Medium — workflow change requires careful testing

---

### Phase 3: npm Workflow Update

#### Step 3.1: Add Git Tag to publish-cli Job
**File:** `.github/workflows/continuous-deploy.yml`

**Action:** Add git tag creation step after successful publish in `publish-cli` job.

**Insert after the "Commit version bump" step:**
```yaml
      - name: Create and push git tag
        run: |
          NEW_VERSION=$(cargo metadata --manifest-path cli/Cargo.toml --no-deps --format-version 1 | jq -r '.packages[0].version')
          git tag "wavecraft-cli-v$NEW_VERSION"
          git push origin "wavecraft-cli-v$NEW_VERSION"
```

**Dependencies:** None (can be done in parallel with Phase 2)
**Risk:** Low

---

#### Step 3.2: Add Git Tag to publish-npm-core Job
**File:** `.github/workflows/continuous-deploy.yml`

**Action:** Add git tag creation step after successful publish in `publish-npm-core` job.

**Insert after the "Commit version bump" step:**
```yaml
      - name: Create and push git tag
        run: |
          NEW_VERSION=$(node -p "require('./ui/packages/core/package.json').version")
          git tag "@wavecraft/core-v$NEW_VERSION"
          git push origin "@wavecraft/core-v$NEW_VERSION"
```

**Dependencies:** None
**Risk:** Low

---

#### Step 3.3: Add Git Tag to publish-npm-components Job
**File:** `.github/workflows/continuous-deploy.yml`

**Action:** Add git tag creation step after successful publish in `publish-npm-components` job.

**Insert after the "Commit version bump" step:**
```yaml
      - name: Create and push git tag
        run: |
          NEW_VERSION=$(node -p "require('./ui/packages/components/package.json').version")
          git tag "@wavecraft/components-v$NEW_VERSION"
          git push origin "@wavecraft/components-v$NEW_VERSION"
```

**Dependencies:** None
**Risk:** Low

---

### Phase 4: Verification

#### Step 4.1: Install cargo-workspaces Locally
**Action:** Install the tool for local testing.

```bash
cargo install cargo-workspaces
```

**Dependencies:** None
**Risk:** None

---

#### Step 4.2: Verify Crate Metadata Locally
**Action:** List workspace crates and verify metadata.

```bash
cd engine
cargo ws list
cargo ws list --json  # Detailed view
```

**Expected output:** All 6 publishable crates listed with correct versions.

**Dependencies:** Phase 1 complete
**Risk:** None

---

#### Step 4.3: Dry-Run Publish Locally
**Action:** Test publishing without actually publishing.

```bash
cd engine
export CARGO_REGISTRY_TOKEN="your-token"
cargo ws publish --from-git --dry-run --yes --no-git-push
```

**Expected:** All crates pass validation, no errors about missing metadata or path dependencies.

**Dependencies:** Phase 1 complete, crates.io token available
**Risk:** None (dry-run only)

---

#### Step 4.4: Check Crate Name Availability
**Action:** Verify crate names are available on crates.io.

```bash
for crate in wavecraft-protocol wavecraft-macros wavecraft-metering wavecraft-dsp wavecraft-bridge wavecraft-core; do
  echo -n "$crate: "
  cargo search "$crate" --limit 1 | grep -q "$crate" && echo "EXISTS" || echo "AVAILABLE"
done
```

**Why:** Prevent first-publish failures due to name squatting.

**Dependencies:** None
**Risk:** High if names are taken (requires rename)

---

#### Step 4.5: Test Workflow on Feature Branch (Manual Trigger)
**Action:** Use `workflow_dispatch` to test the workflow before merging.

1. Push all changes to feature branch
2. Temporarily modify workflow to allow feature branch:
   ```yaml
   on:
     push:
       branches:
         - main
         - feat/test-changes-for-publishing  # TEMPORARY
   ```
3. Trigger workflow via GitHub Actions UI
4. Verify dry-run succeeds
5. Remove temporary branch allowance before merge

**Dependencies:** Phases 1–3 complete
**Risk:** Medium — tests real CI environment

---

## Testing Strategy

### Unit Tests
- `cargo check --workspace` (Step 1.9)
- `cargo ws list` (Step 4.2)

### Integration Tests
- Dry-run publish locally (Step 4.3)
- CI workflow dry-run (Step 4.5)

### E2E Tests (Post-Merge)
- Verify crates appear on crates.io after merge to main
- Verify git tags are created
- Verify version numbers in crates.io match git tags

---

## Risks & Mitigations

| Risk | Likelihood | Mitigation | Step |
|------|------------|------------|------|
| Crate names already taken | Low | Check availability early (Step 4.4) | 4.4 |
| Workflow fails on first real publish | Medium | Extensive dry-run testing (Steps 4.3, 4.5) | 4.3, 4.5 |
| Path deps not converted properly | Low | cargo-workspaces handles this; dry-run verifies | 4.3 |
| Git tag conflicts | Low | Fresh tags, cargo-workspaces handles atomically | - |

---

## Success Criteria

- [ ] All crate `Cargo.toml` files have explicit `version` field
- [ ] `standalone` crate has `publish = false`
- [ ] `cargo check --workspace` passes
- [ ] `cargo ws publish --dry-run` succeeds locally
- [ ] CI workflow dry-run succeeds
- [ ] After merge: crates appear on crates.io with correct metadata
- [ ] After merge: git tags created for each published crate

---

## Rollback Plan

If publishing fails after merge:

1. **Partial publish:** cargo-workspaces is idempotent; re-run workflow after fixing issues
2. **Bad version published:** Use `cargo yank` to remove broken version
3. **Workflow broken:** Revert workflow changes, fix, re-deploy

---

## Estimated Effort

| Phase | Steps | Estimated Time |
|-------|-------|----------------|
| Phase 1: Cargo.toml Updates | 9 | 30 min |
| Phase 2: Engine Workflow | 1 | 20 min |
| Phase 3: npm Workflow | 3 | 15 min |
| Phase 4: Verification | 5 | 45 min |
| **Total** | **18** | **~2 hours** |

