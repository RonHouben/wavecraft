# Low-Level Design: Continuous Deployment

## Overview

**Objective:** Automatically publish packages to npm and crates.io when changes are merged to `main`, without manual version bumping or tag creation.

**Trigger:** Push to `main` branch (PR merge)

**Scope:** 

### npm Packages
- `@wavecraft/core` — IPC bridge, hooks, utilities
- `@wavecraft/components` — Pre-built React components

### Rust Crates (crates.io)
- `wavecraft` — CLI tool for scaffolding plugins
- `wavecraft-core` — nih-plug VST3/CLAP integration
- `wavecraft-dsp` — Pure DSP algorithms
- `wavecraft-protocol` — Shared parameter definitions
- `wavecraft-bridge` — IPC handling
- `wavecraft-metering` — SPSC ring buffer for audio → UI
- `wavecraft-macros` — Procedural macros

### Not Published
- `standalone` — Internal dev tool, not for public consumption
- `xtask` — Build system, not for public consumption

---

## Design Principles

1. **Change-driven publishing** — Only publish packages whose source files changed
2. **Independent versioning** — Each package has its own version, can drift
3. **Automatic patch bumps** — Patch version incremented automatically
4. **Manual major/minor** — Breaking changes require manual version bump in PR
5. **Idempotent** — Re-running workflow doesn't double-publish

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         continuous-deploy.yml                        │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Job 1: detect-changes                                              │
│  ────────────────────                                               │
│  Uses: dorny/paths-filter@v3                                        │
│  Outputs:                                                           │
│    - cli: true/false                                                │
│    - engine: true/false  (all engine crates share version)          │
│    - core: true/false                                               │
│    - components: true/false                                         │
└─────────────────────────────────────────────────────────────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              ▼                   ▼                   ▼
       ┌──────────┐        ┌──────────┐        ┌──────────────┐
       │ cli      │        │ engine   │        │ npm packages │
       │ changed? │        │ changed? │        │ changed?     │
       └────┬─────┘        └────┬─────┘        └──────┬───────┘
            │                   │                     │
            ▼                   ▼                     ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Job 2: publish-cli (if cli changed)                                │
│  ───────────────────────────────────                                │
│  1. Check if current version already published                      │
│  2. If not published: bump patch, commit, publish                   │
│  3. If already published: skip (idempotent)                         │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Job 3: publish-engine (if engine changed)                          │
│  ─────────────────────────────────────────                          │
│  Publishes ALL engine crates in dependency order:                   │
│  1. wavecraft-protocol (no deps)                                    │
│  2. wavecraft-macros (no deps)                                      │
│  3. wavecraft-dsp (depends on protocol, macros)                     │
│  4. wavecraft-metering (no deps)                                    │
│  5. wavecraft-bridge (depends on protocol)                          │
│  6. wavecraft-core (depends on all above)                           │
│                                                                     │
│  All share workspace version — bump once, publish all together      │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Job 4: publish-core (if core changed)                              │
│  ─────────────────────────────────────                              │
│  1. Check if current version already published                      │
│  2. If not published: bump patch, commit, publish                   │
│  3. If already published: skip                                      │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Job 5: publish-components (if components changed)                  │
│  ─────────────────────────────────────────────────                  │
│  Depends on: publish-core (components imports from core)            │
│  1. Check if current version already published                      │
│  2. If not published: bump patch, commit, publish                   │
│  3. If already published: skip                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Re-Trigger Prevention

The `[skip ci]` marker in commit messages **prevents re-triggering**:

```
chore(cli): release v0.7.2 [skip ci]
```

GitHub Actions natively respects `[skip ci]`, `[ci skip]`, and `[no ci]` — workflows are skipped entirely for that push.

**Additional safety:** The idempotency check (version already published?) means even if a workflow somehow ran, it would detect the version exists and skip publishing.

---

## Path Filters

### CLI (`wavecraft` crate)

```yaml
cli:
  - 'cli/src/**'
  - 'cli/Cargo.toml'
  - 'wavecraft-plugin-template/**'  # Template is embedded in CLI
```

### Engine Crates (all share workspace version)

```yaml
engine:
  - 'engine/crates/wavecraft-core/**'
  - 'engine/crates/wavecraft-dsp/**'
  - 'engine/crates/wavecraft-protocol/**'
  - 'engine/crates/wavecraft-bridge/**'
  - 'engine/crates/wavecraft-metering/**'
  - 'engine/crates/wavecraft-macros/**'
  - 'engine/Cargo.toml'  # Workspace version
```

**Note:** `standalone/` and `xtask/` are excluded — they're internal tools.

### Core (`@wavecraft/core`)

```yaml
core:
  - 'ui/packages/core/src/**'
  - 'ui/packages/core/vite.lib.config.ts'
  - 'ui/packages/core/tsconfig.json'
```

### Components (`@wavecraft/components`)

```yaml
components:
  - 'ui/packages/components/src/**'
  - 'ui/packages/components/vite.lib.config.ts'
  - 'ui/packages/components/tsconfig.json'
```

**Note:** `package.json` is excluded from path filters to prevent re-trigger on version bump commits.

---

## Version Bump Strategy

### Automatic (Patch)

When source files change but version wasn't manually bumped:

```bash
# npm packages
npm version patch --no-git-tag-version

# Rust crates
cargo set-version --bump patch
```

### Manual (Major/Minor)

For breaking changes or new features, developer bumps version in PR:

```bash
# In PR branch, before merge
cd ui/packages/core
npm version minor --no-git-tag-version
git add package.json
git commit -m "chore(core): bump to 0.8.0 for new hooks API"
```

The workflow detects the version is already higher and skips auto-bump.

---

## Publish Check (Idempotency)

Before publishing, check if version exists:

### npm

```bash
# Returns 404 if not published
npm view @wavecraft/core@0.7.2 version 2>/dev/null || echo "not-published"
```

### crates.io

```bash
# Returns empty if not published
cargo search wavecraft --limit 1 | grep "0.7.2" || echo "not-published"
```

---

## Job Details

### Job 1: detect-changes

```yaml
detect-changes:
  runs-on: ubuntu-latest
  outputs:
    cli: ${{ steps.filter.outputs.cli }}
    engine: ${{ steps.filter.outputs.engine }}
    core: ${{ steps.filter.outputs.core }}
    components: ${{ steps.filter.outputs.components }}
  steps:
    - uses: actions/checkout@v4
    - uses: dorny/paths-filter@v3
      id: filter
      with:
        filters: |
          cli:
            - 'cli/src/**'
            - 'cli/Cargo.toml'
            - 'wavecraft-plugin-template/**'
          engine:
            - 'engine/crates/wavecraft-core/**'
            - 'engine/crates/wavecraft-dsp/**'
            - 'engine/crates/wavecraft-protocol/**'
            - 'engine/crates/wavecraft-bridge/**'
            - 'engine/crates/wavecraft-metering/**'
            - 'engine/crates/wavecraft-macros/**'
          core:
            - 'ui/packages/core/src/**'
          components:
            - 'ui/packages/components/src/**'
```

### Job 2: publish-cli

```yaml
publish-cli:
  needs: detect-changes
  if: needs.detect-changes.outputs.cli == 'true'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Install cargo-edit
      run: cargo install cargo-edit
    
    - name: Get current version
      id: version
      run: |
        VERSION=$(cargo metadata --manifest-path cli/Cargo.toml --no-deps --format-version 1 | jq -r '.packages[0].version')
        echo "current=$VERSION" >> $GITHUB_OUTPUT
    
    - name: Check if already published
      id: check
      run: |
        if cargo search wavecraft --limit 1 | grep -q "${{ steps.version.outputs.current }}"; then
          echo "published=true" >> $GITHUB_OUTPUT
        else
          echo "published=false" >> $GITHUB_OUTPUT
        fi
    
    - name: Bump patch version
      if: steps.check.outputs.published == 'true'
      working-directory: cli
      run: cargo set-version --bump patch
    
    - name: Publish to crates.io
      env:
        CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
      run: |
        cd cli
        cargo publish --token "$CARGO_REGISTRY_TOKEN"
    
    - name: Commit version bump
      if: steps.check.outputs.published == 'true'
      run: |
        git config user.name "github-actions[bot]"
        git config user.email "github-actions[bot]@users.noreply.github.com"
        git add cli/Cargo.toml
        NEW_VERSION=$(cargo metadata --manifest-path cli/Cargo.toml --no-deps --format-version 1 | jq -r '.packages[0].version')
        git commit -m "chore(cli): release v$NEW_VERSION [skip ci]"
        git push
```

### Job 3: publish-engine

Engine crates share a workspace version and must be published in dependency order.

```yaml
publish-engine:
  needs: detect-changes
  if: needs.detect-changes.outputs.engine == 'true'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Install cargo-edit
      run: cargo install cargo-edit
    
    - name: Get current version
      id: version
      working-directory: engine
      run: |
        VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "wavecraft-protocol") | .version')
        echo "current=$VERSION" >> $GITHUB_OUTPUT
    
    - name: Check if already published
      id: check
      run: |
        if cargo search wavecraft-protocol --limit 1 | grep -q "${{ steps.version.outputs.current }}"; then
          echo "published=true" >> $GITHUB_OUTPUT
        else
          echo "published=false" >> $GITHUB_OUTPUT
        fi
    
    - name: Bump workspace version
      if: steps.check.outputs.published == 'true'
      working-directory: engine
      run: |
        # Bump version in workspace Cargo.toml
        CURRENT=$(grep -E '^version = "' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
        # Increment patch version
        NEW=$(echo $CURRENT | awk -F. '{print $1"."$2"."$3+1}')
        sed -i "s/^version = \"$CURRENT\"/version = \"$NEW\"/" Cargo.toml
        echo "Bumped workspace version: $CURRENT -> $NEW"
    
    - name: Publish crates in dependency order
      env:
        CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
      working-directory: engine
      run: |
        # Order matters! Publish dependencies first.
        CRATES=(
          "wavecraft-protocol"
          "wavecraft-macros"
          "wavecraft-metering"
          "wavecraft-dsp"
          "wavecraft-bridge"
          "wavecraft-core"
        )
        
        for crate in "${CRATES[@]}"; do
          echo "Publishing $crate..."
          cargo publish -p $crate --token "$CARGO_REGISTRY_TOKEN"
          # Wait for crates.io to index (required for dependent crates)
          sleep 30
        done
    
    - name: Commit version bump
      if: steps.check.outputs.published == 'true'
      run: |
        git config user.name "github-actions[bot]"
        git config user.email "github-actions[bot]@users.noreply.github.com"
        git add engine/Cargo.toml
        NEW_VERSION=$(grep -E '^version = "' engine/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
        git commit -m "chore(engine): release v$NEW_VERSION [skip ci]"
        git push
```

### Job 4: publish-core

```yaml
publish-core:
  needs: detect-changes
  if: needs.detect-changes.outputs.core == 'true'
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
    
    - uses: actions/setup-node@v4
      with:
        node-version: '20'
        registry-url: 'https://registry.npmjs.org'
    
    - name: Install dependencies
      working-directory: ui
      run: npm ci
    
    - name: Get current version
      id: version
      run: |
        VERSION=$(node -p "require('./ui/packages/core/package.json').version")
        echo "current=$VERSION" >> $GITHUB_OUTPUT
    
    - name: Check if already published
      id: check
      run: |
        if npm view @wavecraft/core@${{ steps.version.outputs.current }} version 2>/dev/null; then
          echo "published=true" >> $GITHUB_OUTPUT
        else
          echo "published=false" >> $GITHUB_OUTPUT
        fi
    
    - name: Bump patch version
      if: steps.check.outputs.published == 'true'
      working-directory: ui/packages/core
      run: npm version patch --no-git-tag-version
    
    - name: Build package
      working-directory: ui/packages/core
      run: npm run build:lib
    
    - name: Publish to npm
      working-directory: ui/packages/core
      env:
        NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
      run: npm publish --access public
    
    - name: Commit version bump
      if: steps.check.outputs.published == 'true'
      run: |
        git config user.name "github-actions[bot]"
        git config user.email "github-actions[bot]@users.noreply.github.com"
        git add ui/packages/core/package.json
        NEW_VERSION=$(node -p "require('./ui/packages/core/package.json').version")
        git commit -m "chore(core): release v$NEW_VERSION [skip ci]"
        git push
```

### Job 5: publish-components

```yaml
publish-npm-components:
  needs: [detect-changes, publish-npm-core]
  if: |
    always() &&
    needs.detect-changes.outputs.components == 'true' &&
    (needs.publish-npm-core.result == 'success' || needs.publish-npm-core.result == 'skipped')
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
        # Fetch latest to get any version bumps from publish-npm-core
        fetch-depth: 0
    
    - name: Pull latest changes
      run: git pull origin main
    
    - uses: actions/setup-node@v4
      with:
        node-version: '20'
        registry-url: 'https://registry.npmjs.org'
    
    - name: Install dependencies
      working-directory: ui
      run: npm ci
    
    - name: Get current version
      id: version
      run: |
        VERSION=$(node -p "require('./ui/packages/components/package.json').version")
        echo "current=$VERSION" >> $GITHUB_OUTPUT
    
    - name: Check if already published
      id: check
      run: |
        if npm view @wavecraft/components@${{ steps.version.outputs.current }} version 2>/dev/null; then
          echo "published=true" >> $GITHUB_OUTPUT
        else
          echo "published=false" >> $GITHUB_OUTPUT
        fi
    
    - name: Bump patch version
      if: steps.check.outputs.published == 'true'
      working-directory: ui/packages/components
      run: npm version patch --no-git-tag-version
    
    - name: Build package
      working-directory: ui/packages/components
      run: npm run build:lib
    
    - name: Publish to npm
      working-directory: ui/packages/components
      env:
        NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
      run: npm publish --access public
    
    - name: Commit version bump
      if: steps.check.outputs.published == 'true'
      run: |
        git config user.name "github-actions[bot]"
        git config user.email "github-actions[bot]@users.noreply.github.com"
        git add ui/packages/components/package.json
        NEW_VERSION=$(node -p "require('./ui/packages/components/package.json').version")
        git commit -m "chore(npm-components): release v$NEW_VERSION [skip ci]"
        git push
```

---

## Commit Message Convention

Version bump commits use `[skip ci]` to prevent infinite loops:

```
chore(cli): release v0.7.2 [skip ci]
chore(engine): release v0.7.2 [skip ci]
chore(npm-core): release v0.8.0 [skip ci]
chore(npm-components): release v0.7.3 [skip ci]
```

---

## Secrets Required

| Secret | Purpose | Where to get |
|--------|---------|--------------|
| `CARGO_REGISTRY_TOKEN` | crates.io publishing | https://crates.io/settings/tokens |
| `NPM_TOKEN` | npm publishing | https://www.npmjs.com/settings/tokens |
| `GITHUB_TOKEN` | Commit version bumps | Built-in (automatic) |

---

## Edge Cases

### 1. Multiple packages change in same PR

Each job runs independently. Commits happen sequentially due to job dependencies.

### 2. Version already bumped manually in PR

The "Check if already published" step returns `false`, so we publish without bumping again.

### 3. Publish fails mid-workflow

- Version bump commit may or may not have happened
- Next run will detect unpublished version and retry
- Idempotent: won't double-publish

### 4. Components depends on new core feature

Developer should:
1. Bump core version manually in PR
2. Update components' peer dependency range
3. Both get published in correct order (core → components)

### 5. Template changes but CLI code doesn't

Template is embedded in CLI binary via `include_dir!`. Path filter includes `wavecraft-plugin-template/**`, so CLI gets republished with new template.

---

## Alternative: Tag-Based (Keep Existing)

The existing `cli-release.yml` and `npm-release.yml` remain as **manual override**:

```bash
# Force publish specific version
git tag cli-v0.8.0
git push origin cli-v0.8.0
```

Useful for:
- Emergency hotfixes
- Major version releases
- Skipping auto-bump logic

---

## Implementation Tasks

| # | Task | Estimate |
|---|------|----------|
| 1 | Create `.github/workflows/continuous-deploy.yml` | 45 min |
| 2 | Add engine crates metadata for crates.io | 15 min |
| 3 | Test with dry-run (workflow_dispatch + echo) | 15 min |
| 4 | Add `CARGO_REGISTRY_TOKEN` secret if not present | 5 min |
| 5 | Update existing release workflows to note "manual override" | 10 min |
| 6 | Document in `docs/guides/ci-pipeline.md` | 15 min |

**Total:** ~1.75 hours

---

## Decisions Made

1. ✅ **Auto-bump patch versions** — Automatic patch bumps on merge
2. ✅ **Publish engine crates** — All 6 engine crates to crates.io
3. ✅ **Bot commits to main** — Using `[skip ci]` to prevent re-triggers

---

## Approval

- [x] Architecture reviewed
- [x] Secrets confirmed available (NPM_TOKEN, need CARGO_REGISTRY_TOKEN)
- [x] Ready for implementation
