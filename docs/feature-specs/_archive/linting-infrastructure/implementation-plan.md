# Implementation Plan: Linting Infrastructure

## Overview

This plan implements comprehensive linting for the VstKit project: ESLint + Prettier for the React UI (TypeScript) and Clippy + rustfmt for the Rust engine, unified under `cargo xtask lint` commands with CI integration and QA agent workflow updates.

## Requirements

- ESLint with strict TypeScript/React rules for `ui/` codebase
- Prettier for consistent formatting in `ui/`
- Unified `cargo xtask lint` command with `--ui`, `--engine`, `--fix` flags
- CI workflow that fails PRs on lint errors
- QA agent includes linting in automated checks

## Success Criteria

- [ ] `cargo xtask lint` runs all checks and exits non-zero on failures
- [ ] `cargo xtask lint --fix` auto-fixes issues where possible
- [ ] All existing code passes linting (or issues documented)
- [ ] CI blocks PRs with lint errors
- [ ] QA agent reports include linting results

## Architecture Changes

| File | Change Type | Description |
|------|-------------|-------------|
| `ui/eslint.config.js` | New | ESLint 9 flat config with TS/React rules |
| `ui/.prettierrc` | New | Prettier formatting configuration |
| `ui/.prettierignore` | New | Prettier ignore patterns |
| `ui/package.json` | Modify | Add lint/format scripts and devDependencies |
| `engine/xtask/src/commands/lint.rs` | New | Lint command implementation |
| `engine/xtask/src/commands/mod.rs` | Modify | Export lint module |
| `engine/xtask/src/main.rs` | Modify | Add Lint subcommand to CLI |
| `engine/xtask/src/lib.rs` | Modify | Add `ui_dir()` path helper |
| `.github/workflows/lint.yml` | New | CI workflow for linting |
| `.github/agents/QA.agent.md` | Modify | Add linting to automated checks |

---

## Implementation Steps

### Phase 1: UI Tooling Setup (~30 min)

#### Step 1.1: Install npm dependencies

**File:** `ui/package.json` (run in terminal)

- **Action:** Install ESLint, Prettier, and related plugins
- **Command:**
  ```bash
  cd ui && npm install -D eslint @eslint/js @typescript-eslint/eslint-plugin @typescript-eslint/parser eslint-plugin-react eslint-plugin-react-hooks eslint-plugin-react-refresh eslint-config-prettier prettier
  ```
- **Why:** Required tooling for TypeScript/React linting and formatting
- **Dependencies:** None
- **Risk:** Low — standard npm packages

#### Step 1.2: Create ESLint configuration

**File:** `ui/eslint.config.js`

- **Action:** Create ESLint 9 flat config with strict TypeScript/React rules
- **Why:** ESLint 9 uses new flat config format; includes rules for:
  - `@typescript-eslint/no-explicit-any`: error
  - `react-hooks/exhaustive-deps`: error (catches stale closures)
  - `react-refresh/only-export-components`: warn (Vite HMR)
- **Dependencies:** Step 1.1
- **Risk:** Low

**Content (from LLD):**
```javascript
import js from '@eslint/js';
import typescript from '@typescript-eslint/eslint-plugin';
import typescriptParser from '@typescript-eslint/parser';
import react from 'eslint-plugin-react';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';
import eslintConfigPrettier from 'eslint-config-prettier';

export default [
  js.configs.recommended,
  {
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      parser: typescriptParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module',
        ecmaFeatures: { jsx: true },
        project: './tsconfig.json',
      },
    },
    plugins: {
      '@typescript-eslint': typescript,
      'react': react,
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...typescript.configs['recommended'].rules,
      ...typescript.configs['strict'].rules,
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/explicit-function-return-type': 'warn',
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      ...react.configs.recommended.rules,
      'react/react-in-jsx-scope': 'off',
      'react/prop-types': 'off',
      ...reactHooks.configs.recommended.rules,
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'error',
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
    },
    settings: {
      react: { version: 'detect' },
    },
  },
  {
    ignores: ['dist/**', 'node_modules/**', '*.config.js'],
  },
  eslintConfigPrettier,
];
```

#### Step 1.3: Create Prettier configuration

**File:** `ui/.prettierrc`

- **Action:** Create Prettier config with project standards
- **Why:** Consistent formatting (matches Rust's `cargo fmt` philosophy)
- **Dependencies:** Step 1.1
- **Risk:** Low

**Content:**
```json
{
  "semi": true,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5",
  "printWidth": 100,
  "bracketSpacing": true,
  "arrowParens": "always",
  "endOfLine": "lf"
}
```

#### Step 1.4: Create Prettier ignore file

**File:** `ui/.prettierignore`

- **Action:** Exclude build outputs from formatting
- **Why:** Avoid formatting generated files
- **Dependencies:** None
- **Risk:** Low

**Content:**
```
dist/
node_modules/
*.min.js
```

#### Step 1.5: Add npm scripts to package.json

**File:** `ui/package.json`

- **Action:** Add `lint`, `lint:fix`, `format`, `format:check` scripts
- **Why:** Standard entry points for linting commands
- **Dependencies:** Step 1.1
- **Risk:** Low

**Scripts to add:**
```json
{
  "scripts": {
    "lint": "eslint src --max-warnings 0",
    "lint:fix": "eslint src --fix",
    "format": "prettier --write \"src/**/*.{ts,tsx,css}\"",
    "format:check": "prettier --check \"src/**/*.{ts,tsx,css}\""
  }
}
```

#### Step 1.6: Fix existing UI lint errors

- **Action:** Run `npm run lint:fix` and `npm run format` to auto-fix
- **Why:** Baseline must pass before CI enforcement
- **Dependencies:** Steps 1.2–1.5
- **Risk:** Medium — may require manual fixes for unfixable issues

**Manual verification needed for:**
- `@typescript-eslint/no-explicit-any` violations (may need type annotations)
- `react-hooks/exhaustive-deps` violations (may need memoization)

---

### Phase 2: xtask Lint Command (~45 min)

#### Step 2.1: Add `ui_dir()` path helper

**File:** `engine/xtask/src/lib.rs`

- **Action:** Add `ui_dir()` function to `paths` module
- **Why:** Consistent path resolution for UI directory
- **Dependencies:** None
- **Risk:** Low

**Add after `engine_dir()` function (~line 118):**
```rust
/// Returns the UI directory.
pub fn ui_dir() -> Result<PathBuf> {
    Ok(project_root()?.join("ui"))
}
```

#### Step 2.2: Create lint command module

**File:** `engine/xtask/src/commands/lint.rs` (new file)

- **Action:** Implement full lint command with `--ui`, `--engine`, `--fix` flags
- **Why:** Unified command for all linting operations
- **Dependencies:** Step 2.1
- **Risk:** Low

**Implementation notes:**
- Use `std::process::Command` to invoke external tools
- Check `node_modules` exists before running npm commands
- Run both checks even if one fails (report all issues)
- Clear error messages guide users to fix commands

#### Step 2.3: Register lint module

**File:** `engine/xtask/src/commands/mod.rs`

- **Action:** Add `pub mod lint;` export
- **Why:** Make lint module accessible
- **Dependencies:** Step 2.2
- **Risk:** Low

#### Step 2.4: Add Lint subcommand to CLI

**File:** `engine/xtask/src/main.rs`

- **Action:** Add `Lint` variant to `Commands` enum with args
- **Why:** Expose lint command via `cargo xtask lint`
- **Dependencies:** Steps 2.2, 2.3
- **Risk:** Low

**Add to Commands enum:**
```rust
/// Run linters for UI and/or engine code
#[command(about = "Run linters for UI and/or engine code")]
Lint {
    /// Run UI linting only (ESLint + Prettier)
    #[arg(long)]
    ui: bool,

    /// Run engine linting only (Clippy + fmt)
    #[arg(long)]
    engine: bool,

    /// Auto-fix issues where possible
    #[arg(long)]
    fix: bool,
},
```

**Add to match statement:**
```rust
Some(Commands::Lint { ui, engine, fix }) => {
    let targets = if !ui && !engine {
        commands::lint::LintTargets { ui: true, engine: true, fix }
    } else {
        commands::lint::LintTargets { ui, engine, fix }
    };
    commands::lint::run(targets, cli.verbose)
}
```

#### Step 2.5: Test all command combinations

- **Action:** Manual testing of all flag combinations
- **Why:** Verify correct behavior before CI integration
- **Dependencies:** Step 2.4
- **Risk:** Low

**Test matrix:**
| Command | Expected Behavior |
|---------|-------------------|
| `cargo xtask lint` | Run both UI + Engine |
| `cargo xtask lint --engine` | Run Engine only |
| `cargo xtask lint --ui` | Run UI only |
| `cargo xtask lint --fix` | Auto-fix both |
| `cargo xtask lint --engine --fix` | Auto-fix Engine only |
| `cargo xtask lint --ui --fix` | Auto-fix UI only |
| `cargo xtask lint -v` | Verbose output |

---

### Phase 3: QA Agent Update (~15 min)

#### Step 3.1: Update automated checks section

**File:** `.github/agents/QA.agent.md`

- **Action:** Replace individual `cargo fmt` + `cargo clippy` commands with unified `cargo xtask lint`
- **Why:** Consistent tooling, includes UI checks
- **Dependencies:** Phase 2 complete
- **Risk:** Low

**Replace automated checks section with:**
```markdown
## Automated Checks Workflow

**Always run at the start of every QA review**:

```bash
# Run all linting checks (UI + Engine)
cargo xtask lint
```

This runs:
- **Engine**: `cargo fmt --check` + `cargo clippy --workspace -- -D warnings`
- **UI**: `npm run lint` + `npm run format:check`

Document all command outputs in the QA report, including:
- Exit codes
- Warning/error counts
- Specific issues found
```

#### Step 3.2: Add linting section to QA report template

**File:** `.github/agents/QA.agent.md`

- **Action:** Add "Linting Results" section to report template
- **Why:** Structured reporting of lint status
- **Dependencies:** Step 3.1
- **Risk:** Low

**Add to report template:**
```markdown
## Linting Results

### cargo xtask lint
{✅ PASSED | ❌ FAILED}

#### Engine (Rust)
- `cargo fmt --check`: {✅ | ❌}
- `cargo clippy -- -D warnings`: {✅ | ❌}

#### UI (TypeScript)
- ESLint: {✅ | ❌} (Errors: X, Warnings: X)
- Prettier: {✅ | ❌}
```

---

### Phase 4: CI Integration (~15 min)

#### Step 4.1: Create lint workflow

**File:** `.github/workflows/lint.yml` (new file)

- **Action:** Create GitHub Actions workflow for linting
- **Why:** Enforce linting on all PRs
- **Dependencies:** Phases 1–2 complete
- **Risk:** Low

**Implementation notes:**
- Separate jobs for Engine (macos-latest) and UI (ubuntu-latest)
- Use caching for faster runs (rust-cache, npm cache)
- Run early in pipeline (lint → build → test)

---

## Testing Strategy

### Unit Tests
- No new unit tests required (xtask commands are integration-level)

### Integration Tests
- **Manual:** Run all `cargo xtask lint` flag combinations
- **CI:** Verify workflow runs on PRs

### E2E Tests
- Open PR with lint error → verify CI fails
- Open clean PR → verify CI passes

---

## Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| ESLint rules too strict, many existing errors | Medium | Use `lint:fix` first; document unfixable issues; consider `warn` for strict rules initially |
| npm not installed on dev machine | Low | Clear error message in xtask pointing to npm install |
| CI workflow syntax errors | Low | Test locally with `act` if available; iterate quickly |
| Breaking existing developer workflow | Low | Document in README; `--fix` makes fixing easy |

---

## Dependencies

### External
- Node.js 18+ (for npm/ESLint)
- Rust stable (for Clippy/fmt — already required)

### Internal  
- None (this is foundational infrastructure)

---

## Estimated Effort

| Phase | Time | Complexity |
|-------|------|------------|
| Phase 1: UI Tooling | 30 min | Low |
| Phase 2: xtask Commands | 45 min | Medium |
| Phase 3: QA Agent Update | 15 min | Low |
| Phase 4: CI Integration | 15 min | Low |
| **Total** | **~2 hours** | **Low-Medium** |

---

## File Changes Checklist

### New Files
- [ ] `ui/eslint.config.js`
- [ ] `ui/.prettierrc`
- [ ] `ui/.prettierignore`
- [ ] `engine/xtask/src/commands/lint.rs`
- [ ] `.github/workflows/lint.yml`

### Modified Files
- [ ] `ui/package.json` — add scripts + devDependencies
- [ ] `engine/xtask/src/lib.rs` — add `ui_dir()` helper
- [ ] `engine/xtask/src/commands/mod.rs` — export lint module
- [ ] `engine/xtask/src/main.rs` — add Lint subcommand
- [ ] `.github/agents/QA.agent.md` — update automated checks

---

## Post-Implementation

1. **Update roadmap** — Mark "Linting infrastructure" as complete
2. **Announce to team** — Document new `cargo xtask lint` workflow
3. **Optional enhancements** (future):
   - Pre-commit hooks (husky for UI, cargo-husky for Rust)
   - Stricter Clippy lints in `Cargo.toml`
   - IDE integration guidance (VS Code settings)
