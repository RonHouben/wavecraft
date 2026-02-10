# Low-Level Design — CD Engine Auto-Bump

**Status:** Draft  
**Created:** 2025-07-24  
**Author:** Architect Agent

---

## Problem Statement

The Continuous Deploy pipeline (`continuous-deploy.yml`) fails in the `publish-cli` job at step 12 "Verify publishability (dry-run)" (`cargo publish --dry-run`, exit code 101). This has broken CD runs #56 and #57, both on the `feat/build-time-param-discovery` branch merge.

**Root cause:** Engine crate source code changed (new `load_params_from_file()` API in `wavecraft-bridge`, new `FileRead` error variant) but the engine workspace version was NOT bumped — it stayed at `0.11.0`, which was already published to crates.io. The publish-engine job used `cargo ws publish --from-git`, which detected that version `0.11.0` was already published and no-oped. Then publish-cli ran `cargo publish --dry-run`, which strips path dependencies and resolves `wavecraft-bridge ^0.11.0` from crates.io — getting the OLD code that lacks the new API — causing a compile failure.

**Secondary bug:** The `detect-changes` paths-filter is missing `engine/crates/wavecraft-dev-server/**` and `engine/Cargo.toml`, causing changes to these paths to not trigger engine publishing.

**Fundamental issue:** Engine crates are the ONLY component in the CD pipeline that requires manual version bumping. CLI, `@wavecraft/core`, and `@wavecraft/components` all auto-bump. Manual versioning is the single point of failure.

---

## Constraints

1. **Branch protection stays** — `main` has rulesets requiring PR merges and 7 status checks. No direct pushes (already solved by option C in the prior cd-git-workflow-order design — bumps are local-only, tag-only).
2. **Infinite loop prevention** — CD must skip runs triggered by `github-actions[bot]` commits. Since bumps aren't pushed to main, this is already satisfied.
3. **crates.io OIDC publishing** — Must continue using `rust-lang/crates-io-auth-action@v1` for trusted publishing.
4. **Tag contract** — CLI-generated plugins reference engine crates via git tags (`tag = "wavecraft-cli-v0.x.y"`). Tags must continue to be pushed.
5. **cargo-workspaces** — Engine uses `cargo-workspaces` for coordinated multi-crate publishing. Must work with the existing workspace structure (mix of `version.workspace = true` and explicit versions).
6. **Semver compatibility** — CLI deps use `^` ranges (Cargo default). Patch bumps within the same minor are always compatible.

---

## Current State

### Version Locations (16 total)

| Location | Current | Mechanism |
|----------|---------|-----------|
| `engine/Cargo.toml` → `[workspace.package] version` | `0.11.0` | Workspace-level |
| `engine/Cargo.toml` → 7 workspace dependency versions | `0.11.0` each | Workspace deps |
| `engine/crates/wavecraft-core/Cargo.toml` | `version.workspace = true` | Inherits workspace |
| `engine/crates/wavecraft-nih_plug/Cargo.toml` | `version.workspace = true` | Inherits workspace (not published) |
| `engine/crates/wavecraft-bridge/Cargo.toml` | `0.11.0` | Explicit |
| `engine/crates/wavecraft-protocol/Cargo.toml` | `0.11.0` | Explicit |
| `engine/crates/wavecraft-macros/Cargo.toml` | `0.11.0` | Explicit |
| `engine/crates/wavecraft-dsp/Cargo.toml` | `0.11.0` | Explicit |
| `engine/crates/wavecraft-metering/Cargo.toml` | `0.11.0` | Explicit |
| `engine/crates/wavecraft-dev-server/Cargo.toml` | `0.11.0` | Explicit |
| `engine/crates/wavecraft-dev-server/Cargo.toml` → inter-crate deps | `0.11.0` × 2 | Explicit (bridge, protocol) |

### Auto-Bump Status by Component

| Component | Auto-Bumps? | Mechanism |
|-----------|-------------|-----------|
| `wavecraft` CLI | ✅ Yes | `npx semver` + `sed` on `cli/Cargo.toml` |
| `@wavecraft/core` | ✅ Yes | `npx semver` + `npm version` |
| `@wavecraft/components` | ✅ Yes | `npx semver` + `npm version` |
| Engine crates (7) | ❌ **No** | Manual only — `cargo ws publish --from-git` |

### Current `publish-engine` Flow (Broken)

```
checkout → install rust → install cargo-workspaces → configure git
    → cargo ws publish --from-git --dry-run    ← no-ops if version exists on crates.io
    → cargo ws publish --from-git               ← also no-ops
    → git push origin --tags                    ← pushes nothing
```

### Current `detect-changes` Paths-Filter (Incomplete)

```yaml
engine:
  - 'engine/crates/wavecraft-core/**'
  - 'engine/crates/wavecraft-dsp/**'
  - 'engine/crates/wavecraft-protocol/**'
  - 'engine/crates/wavecraft-bridge/**'
  - 'engine/crates/wavecraft-metering/**'
  - 'engine/crates/wavecraft-macros/**'
  # MISSING: engine/crates/wavecraft-dev-server/**
  # MISSING: engine/Cargo.toml
```

---

## Design

### Overview

Bring engine crates in line with CLI and NPM packages: auto-bump patch version when the source version isn't ahead of the registry. This is a publishing concern, not a development concern — developers never need to think about version numbers.

### Phase 1: Fix `detect-changes` Paths-Filter

Add the two missing paths:

```yaml
engine:
  - 'engine/crates/wavecraft-core/**'
  - 'engine/crates/wavecraft-dsp/**'
  - 'engine/crates/wavecraft-protocol/**'
  - 'engine/crates/wavecraft-bridge/**'
  - 'engine/crates/wavecraft-metering/**'
  - 'engine/crates/wavecraft-macros/**'
  - 'engine/crates/wavecraft-dev-server/**'    # NEW
  - 'engine/Cargo.toml'                        # NEW (workspace-level changes)
```

**Rationale:** Any change to `wavecraft-dev-server` or the workspace `Cargo.toml` (dependency versions, workspace members, features) must trigger engine publishing.

### Phase 2: Engine Auto-Bump

Replace the `--from-git` publish approach with an explicit version-check-and-bump pattern.

#### New `publish-engine` Flow

```
checkout → install rust → install cargo-workspaces → install node → configure git
    ↓
┌─ Determine publish version ──────────────────────────────────┐
│  CURRENT = workspace version from engine/Cargo.toml          │
│  PUBLISHED = latest version from crates.io sparse index      │
│  if CURRENT > PUBLISHED → publish CURRENT                    │
│  else → auto-bump needed                                     │
└──────────────────────────────────────────────────────────────┘
    ↓ (if auto-bump needed)
┌─ Auto-bump patch version ────────────────────────────────────┐
│  NEW = npx semver "$PUBLISHED" -i patch                      │
│  cargo ws version custom $NEW --yes --no-git-push            │
│      --allow-branch main --no-git-commit                     │
│  (updates ALL 16 version locations atomically)               │
└──────────────────────────────────────────────────────────────┘
    ↓
┌─ Commit locally ─────────────────────────────────────────────┐
│  git add -A                                                  │
│  git commit -m "chore: bump engine crates to $NEW"           │
│  NOTE: NOT pushed to main                                    │
└──────────────────────────────────────────────────────────────┘
    ↓
┌─ Dry-run ────────────────────────────────────────────────────┐
│  cargo ws publish --yes --no-git-push --allow-branch main    │
│      --dry-run                                               │
└──────────────────────────────────────────────────────────────┘
    ↓
┌─ Authenticate + Publish ─────────────────────────────────────┐
│  OIDC auth via crates-io-auth-action                         │
│  cargo ws publish --yes --no-git-push --allow-branch main    │
└──────────────────────────────────────────────────────────────┘
    ↓
┌─ Tag + push tags ────────────────────────────────────────────┐
│  git push origin --tags                                      │
│  (cargo-workspaces creates tags like wavecraft-core-v0.11.1) │
└──────────────────────────────────────────────────────────────┘
```

#### Concrete Workflow Steps

```yaml
publish-engine:
  needs: detect-changes
  if: needs.detect-changes.outputs.engine == 'true' || needs.detect-changes.outputs.cli == 'true'
  runs-on: ubuntu-latest
  permissions:
    id-token: write
    contents: write
  outputs:
    version: ${{ steps.final.outputs.version }}
  steps:
    - uses: actions/checkout@v4
      with:
        token: ${{ secrets.GITHUB_TOKEN }}
        fetch-depth: 0

    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable

    - uses: actions/setup-node@v4
      with:
        node-version: '20'

    - name: Install Linux system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y \
          pkg-config libglib2.0-dev libgtk-3-dev \
          libsoup-3.0-dev libjavascriptcoregtk-4.1-dev \
          libwebkit2gtk-4.1-dev

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

    - name: Determine publish version
      id: version
      run: |
        # Read current workspace version
        CURRENT=$(grep -m1 'version = "' engine/Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
        echo "current=$CURRENT" >> $GITHUB_OUTPUT

        # Read latest published version from crates.io sparse index
        # Use wavecraft-core as the reference crate
        PUBLISHED=$(curl -s "https://index.crates.io/wa/ve/wavecraft-core" \
          | tail -1 | jq -r '.vers' 2>/dev/null || echo "0.0.0")
        echo "published=$PUBLISHED" >> $GITHUB_OUTPUT

        # Check if current is ahead of published
        if [ "$CURRENT" != "$PUBLISHED" ] && \
           [ "$(printf '%s\n' "$PUBLISHED" "$CURRENT" | sort -V | tail -1)" = "$CURRENT" ]; then
          echo "version=$CURRENT" >> $GITHUB_OUTPUT
          echo "Publishing current version: $CURRENT (latest on crates.io: $PUBLISHED)"
        else
          echo "Auto-bumping: workspace $CURRENT is not ahead of crates.io $PUBLISHED"
        fi

    - name: Auto-bump patch version (if needed)
      id: bump
      if: steps.version.outputs.version == ''
      working-directory: engine
      run: |
        PUBLISHED=${{ steps.version.outputs.published }}
        NEW=$(npx --yes semver "$PUBLISHED" -i patch)
        echo "Bumping engine workspace from ${{ steps.version.outputs.current }} to $NEW"
        cargo ws version custom "$NEW" --yes --no-git-push --allow-branch main --no-git-commit
        echo "version=$NEW" >> $GITHUB_OUTPUT
        echo "bumped=true" >> $GITHUB_OUTPUT
        echo "Auto-bumped engine crates to $NEW"

    - name: Commit auto-bump locally
      if: steps.bump.outputs.bumped == 'true'
      run: |
        git add -A
        git commit -m "chore: bump engine crates to ${{ steps.bump.outputs.version }} [auto-bump]"
        # NOTE: No push to main — branch protection prevents direct pushes.
        # The version bump exists locally for cargo publish and tagging only.

    - name: Set final version
      id: final
      run: |
        VERSION="${{ steps.version.outputs.version }}${{ steps.bump.outputs.version }}"
        echo "version=$VERSION" >> $GITHUB_OUTPUT

    - name: Verify publishability (dry-run)
      working-directory: engine
      run: |
        cargo ws publish \
          --yes \
          --no-git-push \
          --allow-branch main \
          --dry-run

    - name: Authenticate with crates.io (OIDC)
      id: crates-auth
      uses: rust-lang/crates-io-auth-action@v1

    - name: Publish to crates.io
      working-directory: engine
      env:
        CARGO_REGISTRY_TOKEN: ${{ steps.crates-auth.outputs.token }}
      run: |
        cargo ws publish \
          --yes \
          --no-git-push \
          --allow-branch main

    - name: Push tags only
      run: |
        git push origin --tags
```

#### Key Differences from Current Workflow

| Aspect | Before | After |
|--------|--------|-------|
| Version source | `--from-git` (compares git tags) | Compare workspace vs crates.io |
| Auto-bump | None | Patch via `cargo ws version custom` |
| Node.js | Not needed | Required for `npx semver` |
| Publish flags | `--from-git` | No `--from-git` |
| Version output | None | `outputs.version` (for CLI consumption) |
| Dry-run | Before auto-bump (useless) | After auto-bump (validates patchable) |

### Phase 3: CLI Compatibility (No Changes Needed)

When engine auto-bumps from `0.11.0` to `0.11.1`, the CLI's `Cargo.toml` still says:

```toml
[dependencies.wavecraft-bridge]
path = "../engine/crates/wavecraft-bridge"
version = "0.11.0"
```

This works because Cargo interprets `version = "0.11.0"` as `^0.11.0` (i.e., `>=0.11.0, <0.12.0`). When `cargo publish --dry-run` strips the `path` and resolves from crates.io:

- `wavecraft-bridge ^0.11.0` → resolves to `0.11.1` (just published by `publish-engine`)
- Version `0.11.1` contains the new API → compiles successfully

**No changes to `publish-cli` or `cli/Cargo.toml` are required for patch-level engine auto-bumps.**

The only scenario requiring manual version sync is a **minor or major engine version bump** (e.g., `0.12.0`), which would be a deliberate breaking change committed via a regular PR — just like today.

### Phase 4: Enhanced `validate-cli-deps` (Guardrail)

Extend the existing `cargo xtask validate-cli-deps` with a `--check-registry` mode that verifies engine crate availability on crates.io. This runs in the CD pipeline before `cargo publish --dry-run` to catch version mismatches early with actionable error messages.

#### New Check: Registry Availability

```rust
// In validate_cli_deps.rs — new function
fn check_registry_availability(dep: &CliDependency) -> Result<Vec<ValidationError>> {
    let version_req = dep.version.as_deref()
        .ok_or_else(|| anyhow::anyhow!("no version for {}", dep.name))?;

    // Query crates.io sparse index
    let url = format!(
        "https://index.crates.io/{}/{}",
        crate_index_prefix(&dep.name),
        dep.name
    );

    let response = ureq::get(&url).call()?;
    let body = response.into_string()?;
    let latest_version = body.lines().last()
        .and_then(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .and_then(|v| v["vers"].as_str().map(String::from));

    match latest_version {
        Some(ver) if semver::VersionReq::parse(&format!("^{}", version_req))?.matches(&ver.parse()?) => {
            Ok(vec![]) // Compatible version exists
        }
        Some(ver) => {
            Ok(vec![ValidationError {
                name: dep.name.clone(),
                message: format!(
                    "crates.io has {} v{} but CLI requires ^{} — engine may need version bump",
                    dep.name, ver, version_req
                ),
            }])
        }
        None => {
            Ok(vec![ValidationError {
                name: dep.name.clone(),
                message: format!("{} not found on crates.io", dep.name),
            }])
        }
    }
}
```

#### CLI Integration

```
cargo xtask validate-cli-deps                  # existing checks only
cargo xtask validate-cli-deps --check-registry # + crates.io compatibility
```

The `--check-registry` flag is used in `publish-cli` only (not in local dev):

```yaml
- name: Validate CLI dependencies
  working-directory: engine
  run: cargo xtask validate-cli-deps --check-registry
```

---

## Revised CD Flow (Full Picture)

```
  Push to main (PR merge)
         │
         ▼
  ┌──────────────────┐
  │ detect-changes   │  paths-filter (now complete)
  └────────┬─────────┘
           │
     ┌─────┴─────┬──────────────┬───────────────┐
     ▼           ▼              ▼               ▼
  ┌────────┐ ┌──────────┐ ┌───────────┐  ┌───────────────┐
  │publish-│ │publish-  │ │publish-   │  │publish-       │
  │engine  │ │npm-core  │ │npm-comps  │  │cli            │
  │(NEW:   │ │(existing │ │(existing  │  │(existing      │
  │auto-   │ │auto-bump)│ │auto-bump) │  │auto-bump)     │
  │bump)   │ └────┬─────┘ └─────┬─────┘  └───────┬───────┘
  └────┬───┘      │              │                │
       │     All jobs: local-only bump            │
       │     + registry publish + tag push        │
       │                                          │
       └──────────────────────────────────────────┘
              All versions managed by CD
              Developers never bump versions
```

### Version Flow Example

Starting state: all engine crates at `0.11.0` on main and crates.io.

**Merge 1:** PR adds new API to `wavecraft-bridge`
1. `detect-changes`: engine=true
2. `publish-engine`: workspace 0.11.0 = crates.io 0.11.0 → auto-bump to 0.11.1 → publish → tag
3. `publish-cli`: `^0.11.0` resolves to 0.11.1 → dry-run succeeds → publish

**Merge 2:** PR changes only CLI code
1. `detect-changes`: cli=true, engine=false
2. `publish-engine`: skipped (no engine changes)
3. `publish-cli`: `^0.11.0` resolves to 0.11.1 (from merge 1) → works

**Merge 3:** PR changes engine AND CLI
1. `detect-changes`: cli=true, engine=true
2. `publish-engine`: workspace 0.11.0 = crates.io 0.11.1 → ⚠️ 0.11.0 < 0.11.1 → auto-bump to 0.11.2 → publish
3. `publish-cli`: `^0.11.0` resolves to 0.11.2 → works

Note on merge 3: the workspace version on `main` is still `0.11.0` because auto-bumps aren't pushed. The CD correctly bumps from the _published_ version (0.11.1 → 0.11.2), not from the source version.

---

## Version Drift Consideration

With auto-bump, the engine version in source files on `main` will diverge from the published version:

| Source (`main`) | Published (crates.io) | Tags |
|:-:|:-:|:-:|
| 0.11.0 | 0.11.0 | wavecraft-core-v0.11.0 |
| 0.11.0 | 0.11.1 | wavecraft-core-v0.11.1 |
| 0.11.0 | 0.11.2 | wavecraft-core-v0.11.2 |

This is identical to how CLI and NPM packages already work. The version in source is the "product baseline"; the published version is the "distribution version."

When a deliberate minor/major bump is needed, a developer commits it via a normal PR. The CD pipeline then sees the source version is ahead of the registry and publishes it as-is (no auto-bump).

---

## `cargo ws version custom` Compatibility

The engine workspace has a mix of version inheritance patterns:

| Crate | Version Mechanism |
|-------|-------------------|
| `wavecraft-core` | `version.workspace = true` |
| `wavecraft-nih_plug` | `version.workspace = true` (not published) |
| `wavecraft-bridge` | Explicit `version = "0.11.0"` |
| `wavecraft-protocol` | Explicit `version = "0.11.0"` |
| `wavecraft-macros` | Explicit `version = "0.11.0"` |
| `wavecraft-dsp` | Explicit `version = "0.11.0"` |
| `wavecraft-metering` | Explicit `version = "0.11.0"` |
| `wavecraft-dev-server` | Explicit `version = "0.11.0"` |

`cargo ws version custom X.Y.Z` updates:
1. `[workspace.package] version` → affects crates using `version.workspace = true`
2. Each crate's explicit `version` field
3. All inter-crate dependency `version` requirements (e.g., `wavecraft-dev-server`'s deps on `wavecraft-bridge`)
4. Workspace-level `[workspace.dependencies]` version fields

**Verification needed during implementation:** Confirm that `cargo ws version custom 0.11.1 --yes --no-git-push --allow-branch main --no-git-commit` correctly updates all 16 version locations without creating a git commit.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `cargo ws version custom` doesn't support `--no-git-commit` | Medium | Medium | Fall back to `cargo ws version custom $NEW --yes --no-git-push --allow-branch main` (which commits locally) and skip the separate commit step |
| crates.io index propagation delay (publish-engine → publish-cli) | Low | Medium | publish-cli already depends on publish-engine succeeding; add a 30s sleep before dry-run if needed |
| `cargo ws version` doesn't handle mixed `version.workspace` + explicit versions | Low | High | Test locally; if broken, normalize all crates to explicit versions |
| npx semver not available on ubuntu-latest | Very Low | Medium | Already used by CLI/NPM jobs with `npx --yes semver`; node is installed |
| Published version on crates.io for different crates diverges | Very Low | Medium | All 7 crates are published atomically by cargo-workspaces; check `wavecraft-core` as reference |
| CLI version range becomes incompatible after multiple auto-bumps | None | — | `^0.11.0` covers all `0.11.x` patches; only breaks on minor/major which is always manual |

---

## Implementation Phases

| Phase | Description | Effort | Can Ship Independently |
|-------|-------------|--------|----------------------|
| 1 | Fix `detect-changes` paths-filter | Small (2 lines) | ✅ Yes |
| 2 | Engine auto-bump in `publish-engine` | Medium (workflow rewrite) | ✅ Yes (requires Phase 1) |
| 3 | CLI compatibility analysis | None (no code changes) | N/A |
| 4 | Enhanced `validate-cli-deps --check-registry` | Medium (new Rust code + flag) | ✅ Yes |

**Recommended order:** Phase 1 → Phase 2 → Phase 4. Phase 3 is analysis only.

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Conventions and versioning rules
- [CD Git Workflow Order](../_archive/cd-git-workflow-order/low-level-design-cd-git-workflow-order.md) — Prior design for tag-only publishing (Option C)
- [CI Pipeline Guide](../../guides/ci-pipeline.md) — Pipeline architecture
- [Versioning and Distribution](../../architecture/versioning-and-distribution.md) — Version flow and packaging
