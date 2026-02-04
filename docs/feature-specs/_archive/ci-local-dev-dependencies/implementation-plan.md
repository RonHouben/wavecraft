# Implementation Plan: CLI `--local-dev` Flag

## Overview

Add a `--local-dev` CLI flag to `wavecraft new` that generates plugins with local path dependencies instead of git tag dependencies. This fixes the CI bootstrap problem and provides a first-class developer experience for SDK contributors.

## Requirements

- `--local-dev <path>` flag generates path dependencies pointing to local SDK crates
- Flag is mutually exclusive with a custom `--sdk-version` (default `--sdk-version` is ignored when `--local-dev` is provided)
- Invalid paths are rejected with clear error messages
- Relative paths are auto-canonicalized to absolute paths
- All 5 SDK crates are replaced: `wavecraft-core`, `wavecraft-protocol`, `wavecraft-dsp`, `wavecraft-bridge`, `wavecraft-metering`

## Architecture Changes

| File | Change |
|------|--------|
| [cli/src/main.rs](cli/src/main.rs) | Add `--local-dev` argument to `Commands::New` |
| [cli/src/commands/new.rs](cli/src/commands/new.rs) | Add `local_dev` field, pass to `TemplateVariables` |
| [cli/src/template/variables.rs](cli/src/template/variables.rs) | Add `local_dev` field to struct and constructor |
| [cli/src/template/mod.rs](cli/src/template/mod.rs) | Add `apply_local_dev_overrides()` function |
| [.github/workflows/template-validation.yml](.github/workflows/template-validation.yml) | Use `--local-dev` flag, remove `[patch]` workaround |

## Implementation Steps

### Phase 1: CLI Argument

#### Step 1.1: Add `--local-dev` argument to main.rs
**File:** `cli/src/main.rs`

- **Action:** Add `local_dev: Option<PathBuf>` to `Commands::New` with `#[arg(long, conflicts_with = "sdk_version")]`
- **Why:** Enables the flag to be passed from command line
- **Dependencies:** None
- **Risk:** Low

#### Step 1.2: Update NewCommand struct
**File:** `cli/src/commands/new.rs`

- **Action:** Add `pub local_dev: Option<PathBuf>` field to `NewCommand` struct
- **Why:** Carries the flag value through the command execution
- **Dependencies:** Step 1.1
- **Risk:** Low

#### Step 1.3: Pass local_dev in main.rs match arm
**File:** `cli/src/main.rs`

- **Action:** Add `local_dev` field when constructing `NewCommand` in the match arm
- **Why:** Connects CLI parsing to command execution
- **Dependencies:** Steps 1.1, 1.2
- **Risk:** Low

### Phase 2: Template Variables

#### Step 2.1: Add local_dev to TemplateVariables struct
**File:** `cli/src/template/variables.rs`

- **Action:** Add `pub local_dev: Option<PathBuf>` field to `TemplateVariables`
- **Why:** Makes the path available during template processing
- **Dependencies:** None (parallel with Phase 1)
- **Risk:** Low

#### Step 2.2: Update TemplateVariables::new() constructor
**File:** `cli/src/template/variables.rs`

- **Action:** Add `local_dev: Option<PathBuf>` parameter and assign to struct
- **Why:** Initializes the field during variable creation
- **Dependencies:** Step 2.1
- **Risk:** Low

#### Step 2.3: Update TemplateVariables call site in new.rs
**File:** `cli/src/commands/new.rs`

- **Action:** Pass `self.local_dev.clone()` to `TemplateVariables::new()`
- **Why:** Connects the CLI argument to template processing
- **Dependencies:** Steps 1.2, 2.2
- **Risk:** Low

### Phase 3: Post-Processing Logic

#### Step 3.1: Add apply_local_dev_overrides() function
**File:** `cli/src/template/mod.rs`

- **Action:** Create function that replaces git dependencies with path dependencies using regex
- **Why:** Core logic for transforming Cargo.toml dependencies
- **Dependencies:** Step 2.1
- **Risk:** Medium (regex patterns must be correct)

#### Step 3.2: Call apply_local_dev_overrides in extract_dir()
**File:** `cli/src/template/mod.rs`

- **Action:** After `vars.apply(content)`, call `apply_local_dev_overrides()` if `local_dev.is_some()`
- **Why:** Applies the transformation during template extraction
- **Dependencies:** Steps 3.1, 2.1
- **Risk:** Low

### Phase 4: Unit Tests

#### Step 4.1: Add test for apply_local_dev_overrides()
**File:** `cli/src/template/mod.rs`

- **Action:** Add unit test verifying all 5 crates are replaced correctly
- **Why:** Validates the core transformation logic
- **Dependencies:** Step 3.1
- **Risk:** Low

#### Step 4.2: Update existing TemplateVariables tests
**File:** `cli/src/template/variables.rs`

- **Action:** Update test fixtures to include `local_dev` field
- **Why:** Existing tests need the new field to compile
- **Dependencies:** Step 2.2
- **Risk:** Low

### Phase 5: CI Workflow

#### Step 5.1: Update template-validation.yml
**File:** `.github/workflows/template-validation.yml`

- **Action:** 
  1. Change `wavecraft new` command to use `--local-dev ${{ github.workspace }}/engine/crates`
  2. Remove the "Override SDK dependencies" step that appends `[patch]` section
- **Why:** Uses the new first-class feature instead of the workaround
- **Dependencies:** All previous steps
- **Risk:** Low

## Testing Strategy

| Test | Method | Location |
|------|--------|----------|
| `--local-dev` conflicts with custom `--sdk-version` | clap validation | Built-in |
| Invalid path rejected | Unit test | `template/mod.rs` |
| Valid path produces path deps | Unit test | `template/mod.rs` |
| All 5 crates replaced | Unit test | `template/mod.rs` |
| Generated project compiles | CI validation | GitHub Actions |

## Risks & Mitigations

- **Risk:** Regex pattern doesn't match template format
  - Mitigation: Unit test with exact template content
  
- **Risk:** Path canonicalization fails on CI
  - Mitigation: Use absolute path in workflow, test with both relative and absolute

- **Risk:** Missing crate in replacement list
  - Mitigation: Centralize crate list as constant, test all 5

## Success Criteria

- [ ] `wavecraft new test --local-dev ./engine/crates` generates path dependencies
- [ ] `wavecraft new test --local-dev ./bad/path` fails with clear error
- [ ] `wavecraft new test --sdk-version v1.0.0 --local-dev ./path` fails (conflicts)
- [ ] CI pipeline passes with `--local-dev` flag
- [ ] Generated project compiles against local SDK crates
- [ ] All unit tests pass
