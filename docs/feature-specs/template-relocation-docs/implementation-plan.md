# Implementation Plan: Template Reorganization

## Overview

Reorganize the CLI template structure from `cli/plugin-template/` to `cli/sdk-templates/new-project/react/` to improve clarity, support future template variants, and update all documentation references.

## Requirements

- Rename directory structure to `cli/sdk-templates/new-project/react/`
- Update CLI code to reference new path
- Update CI workflow path filters
- Update all documentation references
- Preserve archived files (no modifications)
- Verify CLI builds and generates working projects

## Architecture Changes

| Component | File/Directory | Change |
|-----------|----------------|--------|
| Template | `cli/plugin-template/` | Rename to `cli/sdk-templates/new-project/react/` |
| CLI Code | `cli/src/template/mod.rs` | Update `include_dir!` path |
| CI | `.github/workflows/continuous-deploy.yml` | Update path filter |
| Docs | Multiple files | Update path references |

## Implementation Steps

### Phase 1: Directory Restructure

#### Step 1.1: Create new directory structure
**File:** `cli/sdk-templates/new-project/react/`

- **Action:** Create the new directory hierarchy and move template files
- **Why:** Establishes the extensible structure for future template variants
- **Dependencies:** None
- **Risk:** Low — simple file system operation

**Commands:**
```bash
cd /Users/ronhouben/code/private/wavecraft
mkdir -p cli/sdk-templates/new-project
mv cli/plugin-template cli/sdk-templates/new-project/react
```

---

### Phase 2: Code Updates

#### Step 2.1: Update template extraction path
**File:** `cli/src/template/mod.rs`

- **Action:** Change `include_dir!` path from `plugin-template` to `sdk-templates/new-project/react`
- **Why:** CLI must reference the new template location
- **Dependencies:** Step 1.1
- **Risk:** Low — single line change, easy to verify

**Change:**
```rust
// Before
static TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/plugin-template");

// After
static TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/sdk-templates/new-project/react");
```

---

### Phase 3: CI/CD Updates

#### Step 3.1: Update continuous deploy path filter
**File:** `.github/workflows/continuous-deploy.yml`

- **Action:** Update CLI change detection filter from `cli/plugin-template/**` to `cli/sdk-templates/**`
- **Why:** CI must detect changes in the new location to trigger CLI publishing
- **Dependencies:** Step 1.1
- **Risk:** Low — workflow syntax is straightforward

**Change:**
```yaml
# Before
cli:
  - 'cli/src/**'
  - 'cli/Cargo.toml'
  - 'cli/plugin-template/**'

# After
cli:
  - 'cli/src/**'
  - 'cli/Cargo.toml'
  - 'cli/sdk-templates/**'
```

---

### Phase 4: Documentation Updates (High-Priority)

#### Step 4.1: Update README.md repository structure
**File:** `README.md`

- **Action:** Update the repository structure diagram to show new CLI structure
- **Why:** Primary user-facing documentation must be accurate
- **Dependencies:** Step 1.1
- **Risk:** Low

#### Step 4.2: Update high-level-design.md monorepo structure
**File:** `docs/architecture/high-level-design.md`

- **Action:** Update monorepo structure diagram (~line 41-71) to show `cli/sdk-templates/new-project/react/`
- **Why:** Architecture documentation must reflect actual structure
- **Dependencies:** Step 1.1
- **Risk:** Low

#### Step 4.3: Update ci-pipeline.md path filter table
**File:** `docs/guides/ci-pipeline.md`

- **Action:** Update the path filter table (~line 348) to reference new structure
- **Why:** CI documentation must match actual workflow configuration
- **Dependencies:** Step 3.1
- **Risk:** Low

---

### Phase 5: Documentation Updates (Medium-Priority)

#### Step 5.1: Update backlog.md xtask clean entry
**File:** `docs/backlog.md`

- **Action:** Remove reference to `plugin-template/target` in xtask clean backlog item
- **Why:** Path no longer exists; target is inside cli/target when building
- **Dependencies:** Step 1.1
- **Risk:** Low

#### Step 5.2: Update internal-testing test-plan.md
**File:** `docs/feature-specs/internal-testing/test-plan.md`

- **Action:** Update template location path reference (~line 206)
- **Why:** Test documentation should reflect correct paths
- **Dependencies:** Step 1.1
- **Risk:** Low

#### Step 5.3: Update cli-publish-fix test-plan.md
**File:** `docs/feature-specs/cli-publish-fix/test-plan.md`

- **Action:** Update all references from `cli/plugin-template/` to `cli/sdk-templates/new-project/react/`
- **Why:** Test documentation should reflect correct paths
- **Dependencies:** Step 1.1
- **Risk:** Low

---

### Phase 6: Validation

#### Step 6.1: Verify CLI builds
**Action:** Run `cd cli && cargo build`

- **Why:** Ensure the `include_dir!` path change compiles correctly
- **Dependencies:** Steps 1.1, 2.1
- **Risk:** Low

#### Step 6.2: Verify template extraction
**Action:** Run `./target/debug/wavecraft new test-plugin --vendor Test --no-git`

- **Why:** Ensure templates are correctly extracted from new location
- **Dependencies:** Step 6.1
- **Risk:** Low

#### Step 6.3: Verify generated project builds
**Action:** Run `cd test-plugin && cargo xtask bundle`

- **Why:** Ensure generated project is valid and builds correctly
- **Dependencies:** Step 6.2
- **Risk:** Low

#### Step 6.4: Clean up test project
**Action:** Run `rm -rf test-plugin`

- **Why:** Remove test artifacts
- **Dependencies:** Step 6.3
- **Risk:** Low

---

## Testing Strategy

### Automated Tests
- **CLI build**: `cargo build` in `cli/` directory
- **Rust tests**: `cargo test` (if any exist for template module)

### Manual Tests
- Template extraction: `wavecraft new test-plugin --vendor Test --no-git`
- Generated project build: `cd test-plugin && cargo xtask bundle`
- Verify file structure matches expected layout

### CI Verification
- Push branch and verify no workflow errors
- Check that path filters trigger correctly on PR

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `include_dir!` path typo | Low | High (build fails) | Verify with `cargo build` immediately after change |
| CI filter doesn't trigger | Low | Medium (publishing delayed) | Test with a dummy change to template |
| Documentation inconsistency | Low | Low | Grep search for remaining `plugin-template` references |

---

## Success Criteria

- [ ] Directory structure is `cli/sdk-templates/new-project/react/`
- [ ] CLI builds successfully with `cargo build`
- [ ] `wavecraft new` generates working project
- [ ] Generated project builds with `cargo xtask bundle`
- [ ] CI workflow `continuous-deploy.yml` has correct path filter
- [ ] README.md shows updated structure
- [ ] high-level-design.md shows updated structure
- [ ] ci-pipeline.md shows correct path filter
- [ ] backlog.md updated
- [ ] Test plans updated
- [ ] No references to `plugin-template/` in non-archived docs (verify with grep)

---

## Estimated Effort

| Phase | Effort |
|-------|--------|
| Phase 1: Directory Restructure | 2 min |
| Phase 2: Code Updates | 2 min |
| Phase 3: CI/CD Updates | 2 min |
| Phase 4: High-Priority Docs | 10 min |
| Phase 5: Medium-Priority Docs | 5 min |
| Phase 6: Validation | 10 min |
| **Total** | **~30 min** |
