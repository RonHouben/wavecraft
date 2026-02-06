# CI Build Stage Removal — Low-Level Design

**Status:** Draft  
**Created:** 2026-02-06  
**Author:** Architect Agent  
**Feature Name:** ci-build-stage-removal

---

## Problem Statement

The CI workflow (`.github/workflows/ci.yml`) contains a "Build Plugin" stage that is **permanently skipped** due to conflicting trigger conditions:

```yaml
# Workflow triggers
on:
  pull_request:
    branches: [main]
  workflow_dispatch:

# Build stage condition
build-plugin:
  if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')
```

**The contradiction:**
- Workflow runs on **PRs to main** (feature branches)
- Build stage requires **main branch or version tags**
- Result: Build stage **never executes**

This represents architectural debt: dead code in critical infrastructure that:
1. Misleads maintainers about CI behavior
2. Adds complexity without value
3. Obscures the actual validation pipeline

---

## Current Architecture

### CI Workflow Structure

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           CI WORKFLOW (PRs)                             │
└─────────────────────────────────────────────────────────────────────────┘

Stage 1: Validation (parallel)
├─ check-docs (ubuntu)
├─ check-ui (ubuntu)
│  └─ test-ui (ubuntu) ← depends on check-ui
├─ prepare-engine (ubuntu)
│  ├─ Build UI dist
│  ├─ Build with clippy (compiles all code)
│  └─ Pre-compile test binaries
└─ check-engine (ubuntu) ← depends on prepare-engine

Stage 2: Tests
└─ test-engine (ubuntu) ← depends on check-engine

Stage 3: Build (NEVER RUNS ON PRS)
└─ build-plugin (macos) ← condition: github.ref == 'refs/heads/main'
   ├─ cargo xtask bundle --release
   ├─ Ad-hoc sign
   └─ Upload artifacts (7/90 days retention)
      ├─ wavecraft-vst3-adhoc-signed
      └─ wavecraft-clap-adhoc-signed
```

### Why This Stage Exists

Historical reasons:
1. **Legacy design**: CI originally ran on push to main
2. **Artifact generation**: Provides downloadable test builds
3. **Build verification**: Confirms release bundling works

### Why It's Redundant

1. **prepare-engine already compiles everything**:
   ```yaml
   - name: Build with clippy (compile for lint compatibility)
     run: cargo clippy --workspace --all-targets --no-deps
   ```
   This compiles all production and test code on Linux.

2. **Release workflow handles production builds**:
   - `.github/workflows/release.yml` builds, signs, notarizes
   - Runs on version tags
   - Produces distributable artifacts

3. **Never executes in practice**:
   - PRs run on feature branches (condition fails)
   - Main branch doesn't trigger workflow
   - Version tags don't trigger CI workflow

---

## Proposed Solution

**Remove the `build-plugin` job entirely.**

### Rationale

| Concern | Current (with build-plugin) | Proposed (without) |
|---------|----------------------------|---------------------|
| Code compiles | Verified by prepare-engine (clippy) | ✅ Same |
| Tests pass | Verified by test-engine | ✅ Same |
| Bundles build | Verified by build-plugin (if it ran) | ✅ Verified by Release workflow |
| Ad-hoc signing | Tested in CI (if it ran) | ✅ Tested in Release workflow |
| macOS-specific issues | Caught by build-plugin (if it ran) | ⚠️ Caught later (see Risks) |
| Artifacts available | 7-day retention | ❌ Only on releases |

### Design Principles Satisfied

1. **Single Responsibility**: CI validates correctness; Release workflow produces artifacts
2. **No Dead Code**: Remove permanently skipped logic
3. **Fail Fast**: Tests run before any distribution concerns
4. **DRY**: Release workflow is source of truth for bundling

---

## Impact Analysis

### What Changes

**File: `.github/workflows/ci.yml`**
- Remove entire `build-plugin` job (lines 227-272)
- Update job dependency graph (if other jobs depend on it — currently none do)

**Documentation:**
- Update `docs/guides/ci-pipeline.md` to remove build stage references
- Update this design doc to track rationale

### What Stays the Same

- All lint checks (UI + Engine)
- All tests (UI + Engine)
- Compile verification via clippy
- PR-based validation workflow
- Release workflow (unchanged)

### Breaking Changes

**None.** The job never executes, so removing it cannot break existing behavior.

---

## Risks & Mitigations

### Risk 1: Delayed macOS-Specific Build Failures

**Risk:**  
The current `build-plugin` runs on macOS (vs prepare-engine on Linux). Removing it means macOS-specific build issues are only caught during release.

**Likelihood:** Low  
**Impact:** Medium (delays error detection)

**Analysis:**
- nih-plug is cross-platform
- Most build issues are Rust-level (caught by Linux clippy)
- macOS-specific issues are typically:
  - Code signing (tested in Release workflow)
  - WKWebView API changes (rare, caught in testing)

**Mitigation:**
1. **Template Validation workflow runs on macOS** (validates scaffolding)
2. **Release workflow catches issues before distribution**
3. **Developer testing on macOS** (primary platform per docs)
4. **Future enhancement**: Add macOS clippy check if needed

**Decision:** Accept risk. The job currently never runs anyway.

---

### Risk 2: Loss of PR Artifacts

**Risk:**  
Currently (in theory), build-plugin provides downloadable VST3/CLAP for PR testing.

**Likelihood:** N/A (job never runs)  
**Impact:** None (no one relies on these artifacts)

**Analysis:**
- PR artifacts would have 7-day retention
- Never generated in practice (job condition fails)
- Developers test via `cargo xtask bundle` locally

**Mitigation:**
- Document local testing workflow in CONTRIBUTING.md
- Release artifacts available for version tags (90-day retention)

**Decision:** No mitigation needed (artifact generation was theoretical).

---

## Implementation Plan

### Phase 1: Remove Job Definition

**Changes:**
1. Delete `build-plugin` job from `.github/workflows/ci.yml` (lines 227-272)
2. Verify no other jobs depend on it (confirmed: none do)
3. Test workflow YAML syntax (optional: use `act` to validate)

**Validation:**
- Workflow still runs successfully on PR
- All validation checks complete
- No orphaned artifact references

---

### Phase 2: Update Documentation

**Changes:**
1. Update `docs/guides/ci-pipeline.md`:
   - Remove "Build Plugin" stage description
   - Update workflow diagram
   - Clarify artifact availability (Release workflow only)

2. Update comments in `.github/workflows/ci.yml`:
   - Remove "STAGE 3: Build" header (if stage 3 is empty)
   - Adjust stage numbering if needed

---

### Phase 3: Verify Release Workflow

**Validation checklist:**
1. `.github/workflows/release.yml` still contains:
   - Full plugin build (`cargo xtask bundle --release`)
   - Developer ID signing (production)
   - Notarization
   - Artifact uploads (90-day retention)

2. Release workflow triggers correctly:
   - On version tags (`v*`)
   - Via manual dispatch

**Status:** ✅ Verified (no changes needed to Release workflow)

---

## Alternative Considered: Fix the Condition

**Option:** Make build-plugin run on main branch.

**How:**
```yaml
on:
  pull_request:
    branches: [main]
  push:
    branches: [main]
  workflow_dispatch:
```

**Why rejected:**
1. **Contradicts stated design**: "Runs on PRs only — main branch validation happens via PR quality gates"
2. **Wastes CI resources**: Rebuilds code already validated in PR
3. **Duplicates Release workflow**: Same build logic exists there
4. **Architectural confusion**: Blurs CI (validation) vs Release (distribution)

---

## Testing Strategy

### Pre-Removal Validation

1. **Confirm job never executes**:
   - Check recent CI runs on PRs
   - Verify `build-plugin` shows as "skipped"

2. **Verify no dependencies**:
   ```bash
   grep -n "needs:.*build-plugin" .github/workflows/*.yml
   ```
   Expected: No results

### Post-Removal Validation

1. **Workflow syntax**:
   ```bash
   # Optional: Test with act
   act pull_request --list
   ```

2. **Create test PR**:
   - Verify all checks still run
   - Confirm no missing job errors

3. **Test Release workflow**:
   ```bash
   # Manual trigger via GitHub UI
   gh workflow run release.yml
   ```

---

## Success Criteria

- [ ] `build-plugin` job removed from `.github/workflows/ci.yml`
- [ ] CI workflow runs successfully on PRs (all checks pass)
- [ ] No job dependency errors in GitHub Actions UI
- [ ] Documentation updated (ci-pipeline.md)
- [ ] Release workflow still produces signed artifacts on version tags
- [ ] Local testing workflow documented (CONTRIBUTING.md)

---

## Future Considerations

### Option: Add macOS Validation (If Needed)

If macOS-specific build failures become a problem:

```yaml
check-engine-macos:
  name: Check Engine (macOS)
  runs-on: macos-latest
  needs: [prepare-engine]
  steps:
    - uses: actions/checkout@v4
    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@stable
    - name: Download UI dist
      uses: actions/download-artifact@v4
      with:
        name: ui-dist
        path: ui/dist
    - name: Clippy (macOS)
      run: cargo clippy --workspace --all-targets -- -D warnings
      working-directory: engine
```

**Trigger:** Only add if we see repeated macOS-specific issues in Release workflow.

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Build system overview
- [CI Pipeline Guide](../../guides/ci-pipeline.md) — CI/CD documentation
- [Agent Development Flow](../../architecture/agent-development-flow.md) — PR merge policy

---

## Appendix: Current Job Definition (For Reference)

```yaml
build-plugin:
  name: Build Plugin
  runs-on: macos-latest
  needs: [test-ui, test-engine]
  if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')
  steps:
    - uses: actions/checkout@v4
    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@stable
    - name: Download UI dist
      uses: actions/download-artifact@v4
      with:
        name: ui-dist
        path: ui/dist
    - name: Cache cargo
      uses: Swatinem/rust-cache@v2
      with:
        workspaces: engine
    - name: Build plugin bundles
      run: cargo xtask bundle --release
      working-directory: engine
    - name: Ad-hoc sign (macOS)
      run: cargo xtask sign --adhoc
      working-directory: engine
    - name: Verify signatures
      run: cargo xtask sign --verify --verbose
      working-directory: engine
    - name: Upload VST3
      uses: actions/upload-artifact@v4
      with:
        name: wavecraft-vst3-adhoc-signed
        path: engine/target/bundled/wavecraft.vst3
        retention-days: ${{ startsWith(github.ref, 'refs/tags/v') && 90 || 7 }}
    - name: Upload CLAP
      uses: actions/upload-artifact@v4
      with:
        name: wavecraft-clap-adhoc-signed
        path: engine/target/bundled/wavecraft.clap
        retention-days: ${{ startsWith(github.ref, 'refs/tags/v') && 90 || 7 }}
```

**Lines to remove:** 227-272 (entire job definition)
