# Low-Level Design: crates.io Publishing with cargo-workspaces

**Status:** Draft  
**Created:** 2026-02-06  
**Author:** Architect Agent

---

## Problem Statement

The current `continuous-deploy.yml` workflow cannot publish Rust crates to crates.io because:

1. **Path dependencies are not allowed** — crates.io rejects `{ path = "..." }` dependencies
2. **Missing required metadata** — Some crates lack `description` field
3. **No repository field** — crates.io recommends this for discoverability
4. **Manual ordering is fragile** — Hardcoded publish order can drift from actual dependencies

---

## Goals

| Goal | Priority |
|------|----------|
| Automated crates.io publishing on push to main | High |
| Correct dependency order resolution | High |
| Idempotent (skip already-published versions) | High |
| Minimal workflow complexity | Medium |
| Version bump automation | Medium |

---

## Non-Goals

- Publishing `standalone` crate (dev-only, not intended for distribution)
- Semantic versioning automation based on commit messages
- Publishing to alternate registries

---

## Solution: cargo-workspaces

[cargo-workspaces](https://github.com/pksunkara/cargo-workspaces) is the recommended tool for publishing Cargo workspace crates. It handles:

- **Automatic path → version conversion** at publish time
- **Topological sort** of crate dependencies
- **Independent or synchronized versioning** across workspace
- **Automatic cascade** — re-publishes dependents when a dependency changes
- **Dry-run verification** before publishing
- **Idempotent publishing** (skips already-published versions)

### Why cargo-workspaces over Manual Script?

| Aspect | Manual Script | cargo-workspaces |
|--------|---------------|------------------|
| Path dep conversion | Custom sed/awk | Built-in |
| Dependency ordering | Hardcoded list | Automatic from Cargo.toml |
| Error handling | Manual | Robust with rollback |
| Maintenance burden | High | Low (community-maintained) |
| Dry-run support | Must implement | Built-in |

---

## Architecture

### Current Workflow (Broken)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    CURRENT (BROKEN) PUBLISH FLOW                        │
└─────────────────────────────────────────────────────────────────────────┘

  push to main
       │
       ▼
  ┌─────────────────┐
  │ detect-changes  │ paths-filter on engine/crates/**
  └────────┬────────┘
           │
           ▼
  ┌─────────────────┐
  │ publish-engine  │  cargo publish -p $crate
  └────────┬────────┘
           │
           ▼
      ❌ FAILS
      "all dependencies must have a version specified"
      (path deps rejected by crates.io)
```

### Proposed Workflow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    PROPOSED PUBLISH FLOW                                │
└─────────────────────────────────────────────────────────────────────────┘

  push to main
       │
       ▼
  ┌─────────────────┐
  │ detect-changes  │ paths-filter on engine/crates/**
  └────────┬────────┘
           │ engine == 'true'
           ▼
  ┌─────────────────────────────────────────────────────────────────┐
  │                      publish-engine                             │
  │                                                                 │
  │  1. Checkout with full history (for version detection)          │
  │  2. Install cargo-workspaces                                    │
  │  3. Verify all crates can publish (--dry-run)                   │
  │  4. Publish changed crates only                                 │
  │     └─► cargo ws publish --from-git --yes                       │
  │         • Converts path deps to version deps automatically      │
  │         • Publishes in correct dependency order                 │
  │         • Skips already-published versions                      │
  │  5. (Optional) Commit version bumps                             │
  └─────────────────────────────────────────────────────────────────┘
           │
           ▼
      ✅ SUCCESS
      All crates published to crates.io
```

---

## Implementation Details

### 1. Required Cargo.toml Changes

Before the workflow can succeed, these metadata changes are required:

#### Workspace Root (`engine/Cargo.toml`)

Remove `version` from `[workspace.package]` (each crate owns its version).
Keep shared metadata that doesn't vary per crate.

```toml
[workspace.package]
# version REMOVED — each crate defines its own version
edition = "2024"
license = "MIT"
authors = ["Wavecraft Team"]
repository = "https://github.com/RonHouben/wavecraft"  # ← ADD
```

#### All Crate `Cargo.toml` Files

Each crate gets an explicit `version` field instead of `version.workspace = true`.
All crates start at their current shared version (`0.7.1`) but diverge independently from here.

**`wavecraft-protocol/Cargo.toml`:**
```toml
[package]
name = "wavecraft-protocol"
version = "0.7.1"                   # ← OWN VERSION (was version.workspace)
edition.workspace = true
license.workspace = true
authors.workspace = true
description = "Shared parameter definitions and contracts for Wavecraft"
```

**`wavecraft-macros/Cargo.toml`:**
```toml
[package]
name = "wavecraft-macros"
version = "0.7.1"                   # ← OWN VERSION
edition.workspace = true
license.workspace = true
authors.workspace = true            # ← ADD
description = "Procedural macros for Wavecraft plugin DSL"  # ← ADD
```

**`wavecraft-metering/Cargo.toml`:**
```toml
[package]
name = "wavecraft-metering"
version = "0.7.1"                   # ← OWN VERSION
edition.workspace = true
license.workspace = true
authors.workspace = true
description = "Real-time safe SPSC metering for Wavecraft audio plugins"  # ← ADD
```

**`wavecraft-dsp/Cargo.toml`:**
```toml
[package]
name = "wavecraft-dsp"
version = "0.7.1"                   # ← OWN VERSION
edition.workspace = true
license.workspace = true
authors.workspace = true
description = "Pure DSP algorithms for Wavecraft - no plugin framework dependencies"
```

**`wavecraft-bridge/Cargo.toml`:**
```toml
[package]
name = "wavecraft-bridge"
version = "0.7.1"                   # ← OWN VERSION
edition.workspace = true
license.workspace = true
authors.workspace = true
description = "IPC bridge for WebView ↔ Rust communication"
```

**`wavecraft-core/Cargo.toml`:**
```toml
[package]
name = "wavecraft-core"
version = "0.7.1"                   # ← OWN VERSION
edition.workspace = true
license.workspace = true
authors.workspace = true
description = "Wavecraft audio plugin - nih-plug VST3/CLAP integration"
```

### 2. Exclude `standalone` from Publishing

The `standalone` crate is for local development only. Add to `engine/crates/standalone/Cargo.toml`:

```toml
[package]
name = "standalone"
# ... existing fields ...
publish = false  # ← ADD: Never publish to crates.io
```

### 3. Updated GitHub Workflow (Engine Crates)

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

### 4. Updated GitHub Workflow (npm Packages)

Add git tagging to npm publish jobs for consistency:

```yaml
# In publish-npm-core job, after successful publish:
- name: Create and push git tag
  run: |
    NEW_VERSION=$(node -p "require('./ui/packages/core/package.json').version")
    git tag "@wavecraft/core-v$NEW_VERSION"
    git push origin "@wavecraft/core-v$NEW_VERSION"

# In publish-npm-components job, after successful publish:
- name: Create and push git tag
  run: |
    NEW_VERSION=$(node -p "require('./ui/packages/components/package.json').version")
    git tag "@wavecraft/components-v$NEW_VERSION"
    git push origin "@wavecraft/components-v$NEW_VERSION"
```

### 5. cargo-workspaces Command Reference

| Flag | Purpose |
|------|---------||
| `--from-git` | Only publish crates changed since last git tag |
| `--yes` | Skip confirmation prompts (required for CI) |
| `--dry-run` | Verify without actually publishing |
| `--allow-branch main` | Allow publishing from main branch (required for CI) |
| `--token $TOKEN` | Registry token (reads from env `CARGO_REGISTRY_TOKEN`) |

### 6. Dependency Graph

cargo-workspaces will automatically determine this publishing order:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    CRATE DEPENDENCY GRAPH                               │
└─────────────────────────────────────────────────────────────────────────┘

Level 0 (no internal deps):
    wavecraft-protocol  ←──────────────────────────────────────────────┐
    wavecraft-macros  ←─────────────────────────────────────────┐      │
    wavecraft-metering  ←────────────────────────────────┐      │      │
                                                         │      │      │
Level 1 (depends on L0):                                 │      │      │
    wavecraft-dsp ───────────────────────────────────────┼──────┴──────┤
         │                                               │             │
         │ depends on: protocol, macros                  │             │
         │                                               │             │
    wavecraft-bridge ────────────────────────────────────┼─────────────┤
         │                                               │             │
         │ depends on: protocol                          │             │
         │                                               │             │
Level 2 (depends on L0 + L1):                            │             │
    wavecraft-core ──────────────────────────────────────┴─────────────┘
         │
         │ depends on: protocol, dsp, metering, bridge, macros
         │
         ▼
    (Top of dependency tree - published last)


Publishing order: protocol → macros → metering → dsp → bridge → core
```

---

## Version Strategy

### Decision: Independent Versions ✅

Each crate, npm package, and the CLI maintains its **own version**. Only crates that actually changed (or whose dependencies changed) get a version bump.

| Package Type | Version Source | Tagging |
|--------------|----------------|--------|
| Each engine crate | Own `version` in `Cargo.toml` | `wavecraft-{crate}-v{version}` |
| npm packages | Individual `package.json` | `@wavecraft/{pkg}-v{version}` |
| CLI | `cli/Cargo.toml` | `wavecraft-cli-v{version}` |

**Rationale:**
- A doc fix in `wavecraft-protocol` should NOT bump `wavecraft-metering`
- Version bumps carry meaning — phantom releases erode user trust
- Users pull fewer unnecessary updates
- cargo-workspaces automates the dependency cascade (if `protocol` bumps, its dependents are automatically re-published)

### Dependency Cascade Rules

When a crate's version changes, cargo-workspaces automatically re-publishes dependents:

```
protocol bumps  →  dsp, bridge, core also re-publish (they depend on protocol)
macros bumps    →  dsp, core also re-publish (they depend on macros)
metering bumps  →  core also re-publishes (it depends on metering)
dsp bumps       →  core also re-publishes
bridge bumps    →  core also re-publishes
core bumps      →  only core
```

Crates with **no dependents** (like `metering` if bumped alone) only bump themselves.

### Automatic Version Bumping

cargo-workspaces handles version bumping automatically:

1. Detects which crates changed since last git tag
2. Bumps patch version only for changed crates (e.g., `0.7.1` → `0.7.2`)
3. Bumps dependents whose dependencies were bumped
4. Commits the version changes
5. Creates crate-specific git tags
6. Pushes commits and tags

### Git Tag Format

Each package gets its own tag. Versions diverge over time as only changed packages get bumped.

| Package | Tag Format | Example |
|---------|------------|--------|
| wavecraft-protocol | `wavecraft-protocol-v{version}` | `wavecraft-protocol-v0.7.3` |
| wavecraft-core | `wavecraft-core-v{version}` | `wavecraft-core-v0.7.5` |
| wavecraft-metering | `wavecraft-metering-v{version}` | `wavecraft-metering-v0.7.1` |
| @wavecraft/core | `@wavecraft/core-v{version}` | `@wavecraft/core-v0.7.2` |
| @wavecraft/components | `@wavecraft/components-v{version}` | `@wavecraft/components-v0.7.2` |
| CLI | `wavecraft-cli-v{version}` | `wavecraft-cli-v0.8.0` |

Note: versions will naturally diverge — `metering` may stay at `0.7.1` while `core` advances to `0.7.5`.

---

## Error Handling

### Scenario: Partial Publish Failure

If crate N fails to publish after crates 1..N-1 succeeded:

1. **cargo-workspaces behavior:** Stops immediately, logs error
2. **crates.io state:** Crates 1..N-1 are published, N+ are not
3. **Recovery:** Re-run workflow after fixing the issue. cargo-workspaces will skip already-published crates.

### Scenario: Rate Limiting

crates.io has rate limits. cargo-workspaces handles this with automatic retries and delays between publishes.

### Scenario: Yanked Dependency

If a dependency was yanked, publish will fail. Manual intervention required.

---

## Testing Strategy

### Local Testing (Before Merge)

```bash
# Verify metadata is complete
cd engine
cargo ws list  # Shows all workspace crates

# Dry-run publish (requires crates.io token)
export CARGO_REGISTRY_TOKEN="your-token"
cargo ws publish --from-git --dry-run --yes --no-git-push
```

### CI Testing

The `--dry-run` step in the workflow acts as a gate. If it fails, the actual publish step is never reached.

---

## Implementation Checklist

### Phase 1: Cargo.toml Metadata (Coder)

- [ ] Remove `version` from `[workspace.package]` in `engine/Cargo.toml`
- [ ] Add `repository` to `engine/Cargo.toml` workspace package
- [ ] Add explicit `version = "0.7.1"` to each crate's `Cargo.toml` (replacing `version.workspace = true`)
- [ ] Add `description` and `authors.workspace` to `wavecraft-macros/Cargo.toml`
- [ ] Add `description` to `wavecraft-metering/Cargo.toml`
- [ ] Add `publish = false` to `standalone/Cargo.toml`
- [ ] Verify `cargo check --workspace` still passes after version changes

### Phase 2: Engine Workflow Update (Coder)

- [ ] Replace manual `cargo publish` loop with `cargo ws publish`
- [ ] Add `cargo install cargo-workspaces` step
- [ ] Add git config step for commits/tags
- [ ] Add dry-run verification step
- [ ] Use `fetch-depth: 0` for full git history
- [ ] Remove hardcoded crate list and sleep loops
- [ ] Remove `--no-git-tag` flag (we want crate-specific tags)
- [ ] Add `--allow-branch main` flag

### Phase 3: npm Workflow Update (Coder)

- [ ] Add git tag creation step to `publish-npm-core` job
- [ ] Add git tag creation step to `publish-npm-components` job
- [ ] Add git tag creation step to `publish-cli` job
- [ ] Ensure tags are pushed after successful publish

### Phase 4: Verification (Tester)

- [ ] Verify dry-run passes locally
- [ ] Create test crate versions (e.g., 0.7.2-test) if needed
- [ ] Verify actual publish works on merge to main
- [ ] Verify crates appear on crates.io with correct metadata
- [ ] Verify git tags are created for each published package

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| First publish fails due to name squatting | Low | High | Check crate name availability on crates.io first |
| Rate limiting on bulk publish | Low | Medium | cargo-workspaces handles retries; 6 crates is well under limits |
| Git tag conflicts | Low | Low | cargo-workspaces handles tag creation atomically |
| Version divergence confusion | Low | Low | Tags make each crate's version clear; cargo-workspaces handles cascade |

---

## Decisions (Resolved)

| Question | Decision |
|----------|----------|
| Version strategy | ✅ **Independent versions** — each crate owns its version, cargo-workspaces handles dependency cascade |
| Version bump strategy | ✅ **Automatic** — cargo-workspaces bumps patch version only for changed crates + their dependents |
| Git tags | ✅ **Crate-specific tags** — e.g., `wavecraft-core-v0.7.2`, `@wavecraft/core-v0.7.2` |
| CLI version sync | ✅ **Independent** — CLI maintains separate version from engine crates |

---

## References

- [cargo-workspaces documentation](https://github.com/pksunkara/cargo-workspaces)
- [crates.io publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Current continuous-deploy.yml](/.github/workflows/continuous-deploy.yml)

