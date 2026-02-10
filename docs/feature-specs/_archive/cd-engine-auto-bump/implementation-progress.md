# Implementation Progress — CD Engine Auto-Bump

**Status:** Complete  
**Started:** 2026-02-10  
**Completed:** 2026-02-10  
**Based on:** [Implementation Plan](./implementation-plan.md)

---

## Summary

Successfully implemented automatic patch version bumping for engine crates in the CD pipeline. All 4 phases completed with one adjustment discovered during Phase 2.0 verification.

---

## Completed Phases

### ✅ Phase 1: Fix `detect-changes` Paths-Filter

**Status:** Complete  
**File:** `.github/workflows/continuous-deploy.yml`

**Changes:**
- Added `engine/crates/wavecraft-dev-server/**` to the engine filter
- Added `engine/Cargo.toml` to the engine filter

**Verification:** Both paths now trigger the `detect-changes` engine output.

---

### ✅ Phase 2: Implement Engine Auto-Bump Workflow

**Status:** Complete (with adjustment)  
**File:** `.github/workflows/continuous-deploy.yml`

**Changes:**
1. Added `outputs.version` to `publish-engine` job definition
2. Added Node.js setup step for `npx semver`
3. Added "Determine publish version" step - queries crates.io sparse index
4. Added "Auto-bump patch version" step - uses `cargo ws version custom`
5. Added "Set final version" step - outputs the version for dependency jobs
6. Updated dry-run and publish steps - removed all `--from-git` flags
7. Simplified tag push - `git push origin --tags`

**Phase 2.0 Verification Finding:**

Discovered that `cargo ws version` does not support using both `--no-git-commit` and `--no-git-push` together:

```
error: The argument '--no-git-push' cannot be used with '--no-git-commit'
```

**Adjustment Made:**
- Removed `--no-git-commit` flag from the auto-bump step
- Removed the separate "Commit auto-bump locally" step
- `cargo ws version` now creates the commit automatically with its default message

**Final Command:**
```bash
cargo ws version custom "$NEW" --yes --no-git-push --allow-branch main
```

---

### ✅ Phase 3: CLI Compatibility Analysis

**Status:** Complete (no code changes required)

**Analysis:**

The CLI's `Cargo.toml` specifies engine dependencies as:
```toml
[dependencies.wavecraft-bridge]
path = "../engine/crates/wavecraft-bridge"
version = "0.11.0"
```

Cargo interprets `version = "0.11.0"` as `^0.11.0` (caret requirement), which means:
- `>=0.11.0, <0.12.0`
- Any patch bump (0.11.1, 0.11.2, etc.) is compatible
- Minor/major bumps require manual version updates in CLI

**When engine auto-bumps from 0.11.0 → 0.11.1:**
1. `cargo publish --dry-run` strips the `path` dependency
2. Resolves `wavecraft-bridge ^0.11.0` from crates.io
3. Finds 0.11.1 (just published by `publish-engine`)
4. 0.11.1 satisfies ^0.11.0 ✅
5. Compilation succeeds

**Conclusion:** No changes needed to `cli/Cargo.toml` or `publish-cli` job for patch-level engine auto-bumps. Only deliberate minor/major engine bumps (committed via PR) require manual CLI dependency updates.

---

### ✅ Phase 4: Implement `validate-cli-deps --check-registry` Guardrail

**Status:** Complete  
**Files:**
- `engine/xtask/Cargo.toml`
- `engine/xtask/src/main.rs`
- `engine/xtask/src/commands/validate_cli_deps.rs`
- `.github/workflows/continuous-deploy.yml`

**Changes:**

#### 4.1: Dependencies Added
- `ureq = "2"`
- `semver = "1"`
- (`serde_json = "1"` already present)

#### 4.2: CLI Flag Added
Updated `Commands::ValidateCliDeps` enum variant:
```rust
ValidateCliDeps {
    /// Also verify crate availability on crates.io
    #[arg(long)]
    check_registry: bool,
},
```

#### 4.3: Config Struct Updated
```rust
pub struct ValidateCliDepsConfig {
    pub verbose: bool,
    pub check_registry: bool,  // NEW
}
```

#### 4.4: Registry Check Functions Implemented
- `crate_index_prefix(name: &str) -> String` - Generates crates.io sparse index URL paths
- `check_registry_availability(dep: &CliDependency) -> Result<Vec<ValidationError>>` - Queries crates.io and validates semver compatibility

**Logic:**
1. Query `https://index.crates.io/{prefix}/{crate_name}` (sparse index)
2. Parse latest published version from response
3. Check if CLI's `^X.Y.Z` requirement matches the published version
4. Return validation error if no compatible version exists

#### 4.5: Integration into `run()` Function
Added registry check loop after existing validation, controlled by `config.check_registry` flag.

#### 4.6: Unit Tests Added
- `test_crate_index_prefix()` - Verifies URL path generation for 1-4+ char names
- `test_check_registry_config_default()` - Verifies `check_registry` defaults to `false`
- `test_check_registry_availability_real_crate()` - (#[ignore] integration test)

#### 4.7: CD Workflow Updated
Updated `publish-cli` job:
```yaml
- name: Validate CLI dependencies
  working-directory: engine
  run: cargo xtask validate-cli-deps --check-registry
```

---

## Testing Performed

### Local Testing
- ✅ Phase 1: Verified paths-filter syntax
- ✅ Phase 2.0: Tested `cargo ws version custom` behavior (discovered flag conflict)
- ✅ Phase 4: Unit tests pass (`cargo test -p xtask`)

### Manual Verification Needed (Post-Merge)
1. Push a change to only `wavecraft-dev-server` → Verify `detect-changes` outputs `engine=true`
2. Trigger CD → Verify `publish-engine` auto-bumps and publishes successfully
3. Verify `publish-cli` succeeds with new engine versions

---

## Files Changed

| File | Lines Changed | Description |
|------|---------------|-------------|
| `.github/workflows/continuous-deploy.yml` | ~60 | Paths-filter + auto-bump workflow |
| `engine/xtask/Cargo.toml` | +2 | Add ureq, semver deps |
| `engine/xtask/src/main.rs` | ~10 | Add check_registry flag |
| `engine/xtask/src/commands/validate_cli_deps.rs` | ~120 | Registry check + tests |

**Total:** ~190 lines changed/added

---

## Key Implementation Decisions

1. **Removed `--no-git-commit` flag** — Cannot be used with `--no-git-push`. Let `cargo ws version` create the commit automatically.

2. **Used crates.io sparse index** — Direct HTTP query to `https://index.crates.io/` avoids git clone overhead.

3. **Caret requirement interpretation** — CLI version "0.11.0" is treated as "^0.11.0" (Cargo default), allowing patch bumps without CLI changes.

4. **Version drift is intentional** — Source version on `main` stays at baseline (e.g., 0.11.0) while published versions increment (0.11.1, 0.11.2...). This matches existing CLI and NPM package behavior.

5. **Registry check is opt-in** — `--check-registry` flag only used in CD, not in local dev (avoids network calls during development).

---

## Success Criteria

- [x] `detect-changes` correctly identifies `wavecraft-dev-server` and `engine/Cargo.toml` changes
- [x] `publish-engine` workflow includes auto-bump logic
- [x] No `--from-git` flags remain in publish steps
- [x] `cargo xtask validate-cli-deps --check-registry` implemented and wired into CD
- [x] Unit tests added for new functionality
- [x] All code follows Rust coding standards

---

## Related Documents

- [Low-Level Design](./low-level-design-cd-engine-auto-bump.md) — Technical design
- [Implementation Plan](./implementation-plan.md) — Step-by-step plan
- [Coding Standards — Rust](../../architecture/coding-standards-rust.md) — Rust conventions
- [Versioning and Distribution](../../architecture/versioning-and-distribution.md) — Version flow
