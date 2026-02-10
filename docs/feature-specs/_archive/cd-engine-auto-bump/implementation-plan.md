# Implementation Plan — CD Engine Auto-Bump

**Status:** Not Started  
**Created:** 2026-02-10  
**Based on:** [Low-Level Design](./low-level-design-cd-engine-auto-bump.md)

---

## Overview

This plan details the steps to implement automatic patch version bumping for the engine crates within the `continuous-deploy.yml` workflow. The goal is to eliminate the manual versioning requirement that is currently causing CD pipeline failures (runs #56, #57). The implementation is broken into four distinct phases, each independently shippable.

---

## Requirements

- The CD pipeline must automatically increment the patch version of engine crates if changes are detected and the current version is not ahead of the published version on crates.io.
- The `detect-changes` job must correctly identify all relevant changes within the `engine/` directory.
- The solution must be compatible with the existing `cargo-workspaces` setup and the mix of explicit and workspace-level versioning.
- The auto-bump mechanism must not push version bump commits back to the `main` branch.
- A new validation step should be added to the `publish-cli` job to verify engine crate availability on crates.io before attempting to publish the CLI.

---

## Architecture Changes

| File | Change |
|------|--------|
| `.github/workflows/continuous-deploy.yml` | Fix paths-filter, rewrite `publish-engine` job, add registry validation to `publish-cli` |
| `engine/xtask/src/commands/validate_cli_deps.rs` | Add `check_registry_availability()` function and `--check-registry` flag handling |
| `engine/xtask/src/main.rs` | Add `check_registry` boolean field to `ValidateCliDeps` command |
| `engine/xtask/Cargo.toml` | Add `ureq` and `semver` dependencies |

---

## Implementation Steps

### Phase 1: Fix `detect-changes` Paths-Filter

> **Effort:** Small (2 lines) · **Risk:** Low · **Ships independently:** Yes

#### Step 1.1: Add Missing Paths

**File:** `.github/workflows/continuous-deploy.yml`

Add two missing path patterns to the `engine` key in the `paths-filter` step of the `detect-changes` job (around line 42):

```yaml
engine:
  - 'engine/crates/wavecraft-core/**'
  - 'engine/crates/wavecraft-dsp/**'
  - 'engine/crates/wavecraft-protocol/**'
  - 'engine/crates/wavecraft-bridge/**'
  - 'engine/crates/wavecraft-metering/**'
  - 'engine/crates/wavecraft-macros/**'
  - 'engine/crates/wavecraft-dev-server/**'    # NEW
  - 'engine/Cargo.toml'                        # NEW
```

**Verification:**
- Review the diff to confirm only the two new lines were added
- The `detect-changes` job logic remains unchanged

---

### Phase 2: Implement Engine Auto-Bump Workflow

> **Effort:** Medium (workflow rewrite) · **Risk:** Medium · **Ships independently:** Yes (requires Phase 1) · **Depends on:** Phase 1

#### Step 2.0: Verify `cargo ws version` Behavior (Pre-Implementation Check)

**Action:** Run the following command locally in the `engine/` directory to confirm behavior:

```bash
cd engine
cargo ws version custom 0.11.99 --yes --no-git-push --allow-branch main --no-git-commit
```

**Verify:**
- [ ] `engine/Cargo.toml` → `[workspace.package] version` updated to `0.11.99`
- [ ] `engine/Cargo.toml` → all 7 `[workspace.dependencies]` wavecraft-* versions updated
- [ ] All 6 crates with explicit versions updated (bridge, protocol, macros, dsp, metering, dev-server)
- [ ] `wavecraft-core` and `wavecraft-nih_plug` (using `version.workspace = true`) inherit correctly
- [ ] Inter-crate deps in `wavecraft-dev-server/Cargo.toml` (bridge, protocol) updated
- [ ] No git commit was created (check `git status` shows unstaged changes)

**If `--no-git-commit` is not supported:** Remove the flag and let `cargo ws version` create a local commit. Adjust Step 2.2 accordingly (skip the separate commit step).

**Rollback:** `git checkout -- .` to restore all files.

#### Step 2.1: Add Node.js Setup to `publish-engine`

**File:** `.github/workflows/continuous-deploy.yml`

Add a Node.js setup step to the `publish-engine` job (needed for `npx semver`):

```yaml
- uses: actions/setup-node@v4
  with:
    node-version: '20'
```

Insert after the "Install Rust" step.

#### Step 2.2: Add Version Determination Step

**File:** `.github/workflows/continuous-deploy.yml`

Add a new step to compare workspace version against crates.io:

```yaml
- name: Determine publish version
  id: version
  run: |
    CURRENT=$(grep -m1 'version = "' engine/Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
    PUBLISHED=$(curl -s "https://index.crates.io/wa/ve/wavecraft-core" \
      | tail -1 | jq -r '.vers' 2>/dev/null || echo "0.0.0")
    echo "current=$CURRENT" >> $GITHUB_OUTPUT
    echo "published=$PUBLISHED" >> $GITHUB_OUTPUT

    if [ "$CURRENT" != "$PUBLISHED" ] && \
       [ "$(printf '%s\n' "$PUBLISHED" "$CURRENT" | sort -V | tail -1)" = "$CURRENT" ]; then
      echo "version=$CURRENT" >> $GITHUB_OUTPUT
      echo "Publishing current version: $CURRENT (latest on crates.io: $PUBLISHED)"
    else
      echo "Auto-bumping: workspace $CURRENT is not ahead of crates.io $PUBLISHED"
    fi
```

Insert after the "Configure git" step.

#### Step 2.3: Add Auto-Bump Step

**File:** `.github/workflows/continuous-deploy.yml`

```yaml
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
```

#### Step 2.4: Add Local Commit Step

**File:** `.github/workflows/continuous-deploy.yml`

```yaml
- name: Commit auto-bump locally
  if: steps.bump.outputs.bumped == 'true'
  run: |
    git add -A
    git commit -m "chore: bump engine crates to ${{ steps.bump.outputs.version }} [auto-bump]"
    # NOTE: No push to main — branch protection prevents direct pushes.
```

#### Step 2.5: Add Final Version Output

**File:** `.github/workflows/continuous-deploy.yml`

```yaml
- name: Set final version
  id: final
  run: |
    VERSION="${{ steps.version.outputs.version }}${{ steps.bump.outputs.version }}"
    echo "version=$VERSION" >> $GITHUB_OUTPUT
```

Add `outputs: version: ${{ steps.final.outputs.version }}` to the job definition.

#### Step 2.6: Update Dry-Run and Publish Steps

**File:** `.github/workflows/continuous-deploy.yml`

Replace the existing dry-run and publish steps. Remove all `--from-git` flags:

```yaml
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
  run: git push origin --tags
```

**Verification:**
- The complete `publish-engine` job should follow the flow in the LLD
- No `--from-git` flags remain

---

### Phase 3: CLI Compatibility Analysis (No Code Changes)

> **Effort:** None · **Risk:** None · **Ships independently:** N/A

#### Step 3.1: Verify Semver Resolution

**Analysis task only.** Confirm:

- `cli/Cargo.toml` has `version = "0.11.0"` for engine deps → Cargo interprets this as `^0.11.0`
- When engine bumps to `0.11.1`, `^0.11.0` resolves to `0.11.1` ✅
- No changes needed to `cli/Cargo.toml` or `publish-cli` job for patch-level engine bumps
- Manual CLI dep version updates are only needed for **minor/major** engine bumps

**Document findings** in the implementation progress file.

---

### Phase 4: Implement `validate-cli-deps --check-registry` Guardrail

> **Effort:** Medium · **Risk:** Medium · **Ships independently:** Yes · **Depends on:** None (can be done in parallel)

#### Step 4.1: Add Dependencies to xtask

**File:** `engine/xtask/Cargo.toml`

Add:
```toml
ureq = "2"
semver = "1"
serde_json = "1"
```

Note: `serde_json` may already be a transitive dependency — check before adding.

#### Step 4.2: Add `--check-registry` CLI Flag

**File:** `engine/xtask/src/main.rs`

Update the `ValidateCliDeps` variant in the `Commands` enum:

```rust
ValidateCliDeps {
    /// Also verify crate availability on crates.io
    #[arg(long)]
    check_registry: bool,
},
```

Update the match arm to pass the flag:

```rust
Some(Commands::ValidateCliDeps { check_registry }) => {
    let config = commands::validate_cli_deps::ValidateCliDepsConfig {
        verbose: cli.verbose,
        check_registry,
    };
    commands::validate_cli_deps::run(config)
}
```

#### Step 4.3: Update Config Struct

**File:** `engine/xtask/src/commands/validate_cli_deps.rs`

Add field to `ValidateCliDepsConfig`:

```rust
pub struct ValidateCliDepsConfig {
    pub verbose: bool,
    pub check_registry: bool,
}
```

Update the `Default` impl accordingly.

#### Step 4.4: Implement Registry Check Function

**File:** `engine/xtask/src/commands/validate_cli_deps.rs`

Add the `check_registry_availability` function and helper `crate_index_prefix` function as specified in the LLD. The function should:

1. Query the crates.io sparse index URL for the crate
2. Parse the latest version from the response
3. Check if the CLI's required version range (`^X.Y.Z`) has a matching published version
4. Return `ValidationError` if no compatible version exists

Follow the coding standards in `docs/architecture/coding-standards-rust.md`:
- Use `snake_case` for functions
- Use `///` doc comments
- Handle errors with `anyhow`

#### Step 4.5: Integrate into `run()` Function

**File:** `engine/xtask/src/commands/validate_cli_deps.rs`

In the `run()` function, after the existing validation loop, add a conditional block:

```rust
if config.check_registry {
    print_header("Registry Availability Check");
    for dep in &deps {
        match check_registry_availability(dep) {
            Ok(reg_errors) => {
                errors.extend(reg_errors);
            }
            Err(e) => {
                print_error_item(&format!("{} — registry check failed: {}", dep.name, e));
            }
        }
    }
}
```

#### Step 4.6: Add Unit Tests

**File:** `engine/xtask/src/commands/validate_cli_deps.rs`

Add tests in the existing `#[cfg(test)] mod tests` block:

- `test_crate_index_prefix` — verify URL path generation for different crate name lengths
- `test_check_registry_config_default` — verify `check_registry` defaults to `false`

Note: Testing the actual HTTP call requires either mocking or an integration test. Consider a `#[ignore]` test that hits crates.io for a known crate.

#### Step 4.7: Update CD Workflow

**File:** `.github/workflows/continuous-deploy.yml`

In the `publish-cli` job, update the existing "Validate CLI dependencies" step:

```yaml
- name: Validate CLI dependencies
  working-directory: engine
  run: cargo xtask validate-cli-deps --check-registry
```

---

## Testing Strategy

### Unit Tests
- New `check_registry_availability` function needs tests with mocked HTTP responses
- Test `crate_index_prefix` helper for different crate name lengths (1, 2, 3, 4+ chars)

### Integration Tests (Manual)
1. **Phase 1 verification:** Push a change to only `wavecraft-dev-server` and confirm `detect-changes` outputs `engine=true`
2. **Phase 2 verification:** After Phase 2 merge, trigger CD and verify:
   - `publish-engine` bumps version correctly
   - All 7 crates published to crates.io with new version
   - Tags pushed (e.g., `wavecraft-core-v0.11.1`)
   - `publish-cli` succeeds with the new engine versions

### Rollback Strategy

| Phase | Rollback |
|-------|----------|
| Phase 1 | Revert the 2-line change in `continuous-deploy.yml` |
| Phase 2 | Restore previous `publish-engine` job from git history. No repo state cleanup needed (bumps are local-only). |
| Phase 4 | Remove `--check-registry` from the CD workflow step. The xtask code is inert without the flag. |

---

## Success Criteria

- [ ] `detect-changes` correctly identifies changes in `engine/crates/wavecraft-dev-server/**` and `engine/Cargo.toml`
- [ ] Engine crate changes trigger automatic patch version bump in `publish-engine`
- [ ] `publish-engine` successfully publishes all 7 engine crates to crates.io
- [ ] `publish-cli` succeeds, resolving new engine crate versions via `^` semver range
- [ ] `cargo xtask validate-cli-deps --check-registry` correctly identifies compatible/incompatible versions
- [ ] CD pipeline remains stable across multiple consecutive merges

---

## Estimated Effort

| Phase | Effort | Time |
|-------|--------|------|
| Phase 1 | Small | 15 min |
| Phase 2 | Medium | 2–3 hours (including local verification) |
| Phase 3 | None | 30 min (analysis only) |
| Phase 4 | Medium | 2–3 hours |
| **Total** | | **~5–6 hours** |

---

## Related Documents

- [Low-Level Design](./low-level-design-cd-engine-auto-bump.md) — Technical design
- [Coding Standards — Rust](../../architecture/coding-standards-rust.md) — Rust conventions
- [Coding Standards](../../architecture/coding-standards.md) — General conventions
- [CI Pipeline Guide](../../guides/ci-pipeline.md) — Pipeline architecture
- [Versioning and Distribution](../../architecture/versioning-and-distribution.md) — Version flow
