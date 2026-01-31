# Low-Level Design: Linting Infrastructure

> **Feature:** Comprehensive linting for UI (TypeScript/React) and Engine (Rust)  
> **Author:** Architect Agent  
> **Date:** 2026-01-31  
> **Status:** Draft  
> **User Stories:** [user-stories.md](./user-stories.md)

---

## Table of Contents

1. [Overview](#1-overview)
2. [UI Linting (ESLint + Prettier)](#2-ui-linting-eslint--prettier)
3. [Engine Linting (Clippy + fmt)](#3-engine-linting-clippy--fmt)
4. [xtask Commands](#4-xtask-commands)
5. [QA Agent Integration](#5-qa-agent-integration)
6. [CI Integration](#6-ci-integration)
7. [File Changes Summary](#7-file-changes-summary)

---

## 1. Overview

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         cargo xtask lint                                     │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────┐          ┌─────────────────────────┐           │
│  │    --engine             │          │      --ui               │           │
│  │                         │          │                         │           │
│  │  cargo fmt --check      │          │  npm run lint           │           │
│  │  cargo clippy           │          │  npm run format:check   │           │
│  │                         │          │                         │           │
│  └─────────────────────────┘          └─────────────────────────┘           │
│              │                                     │                        │
│              ▼                                     ▼                        │
│  ┌─────────────────────────┐          ┌─────────────────────────┐           │
│  │   Rust Codebase         │          │   TypeScript Codebase   │           │
│  │   engine/crates/*       │          │   ui/src/*              │           │
│  └─────────────────────────┘          └─────────────────────────┘           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Consistency with existing tooling** — Follow established patterns in xtask
2. **Fast feedback** — Linting should be quick (<30s for full workspace)
3. **CI-first design** — All commands have non-zero exit codes on failure
4. **Developer ergonomics** — Clear output, helpful error messages

---

## 2. UI Linting (ESLint + Prettier)

### 2.1 ESLint Configuration

**File:** `ui/eslint.config.js` (ESLint 9 flat config format)

```javascript
import js from '@eslint/js';
import typescript from '@typescript-eslint/eslint-plugin';
import typescriptParser from '@typescript-eslint/parser';
import react from 'eslint-plugin-react';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';

export default [
  // Base JavaScript rules
  js.configs.recommended,
  
  // TypeScript files
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
      // TypeScript strict rules
      ...typescript.configs['recommended'].rules,
      ...typescript.configs['strict'].rules,
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/explicit-function-return-type': 'warn',
      '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      
      // React rules
      ...react.configs.recommended.rules,
      'react/react-in-jsx-scope': 'off', // Not needed with React 17+
      'react/prop-types': 'off', // Using TypeScript
      
      // React Hooks rules (exhaustive deps)
      ...reactHooks.configs.recommended.rules,
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'error',
      
      // React Refresh (Vite HMR)
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
    },
    settings: {
      react: { version: 'detect' },
    },
  },
  
  // Ignore patterns
  {
    ignores: ['dist/**', 'node_modules/**', '*.config.js'],
  },
];
```

### 2.2 Prettier Configuration

**File:** `ui/.prettierrc`

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

**File:** `ui/.prettierignore`

```
dist/
node_modules/
*.min.js
```

### 2.3 ESLint-Prettier Integration

To prevent ESLint and Prettier from conflicting:

**Install:** `eslint-config-prettier` (disables ESLint formatting rules)

Add to `eslint.config.js`:
```javascript
import eslintConfigPrettier from 'eslint-config-prettier';

export default [
  // ... other configs
  eslintConfigPrettier, // Must be last
];
```

### 2.4 Package.json Scripts

**File:** `ui/package.json` (additions)

```json
{
  "scripts": {
    "lint": "eslint src --ext .ts,.tsx --max-warnings 0",
    "lint:fix": "eslint src --ext .ts,.tsx --fix",
    "format": "prettier --write \"src/**/*.{ts,tsx,css}\"",
    "format:check": "prettier --check \"src/**/*.{ts,tsx,css}\""
  },
  "devDependencies": {
    "@eslint/js": "^9.0.0",
    "@typescript-eslint/eslint-plugin": "^7.0.0",
    "@typescript-eslint/parser": "^7.0.0",
    "eslint": "^9.0.0",
    "eslint-config-prettier": "^9.0.0",
    "eslint-plugin-react": "^7.34.0",
    "eslint-plugin-react-hooks": "^4.6.0",
    "eslint-plugin-react-refresh": "^0.4.0",
    "prettier": "^3.2.0"
  }
}
```

### 2.5 Key ESLint Rules Rationale

| Rule | Setting | Rationale |
|------|---------|-----------|
| `@typescript-eslint/no-explicit-any` | `error` | Strict typing requested |
| `react-hooks/exhaustive-deps` | `error` | Catches stale closure bugs |
| `@typescript-eslint/explicit-function-return-type` | `warn` | Encourages explicitness (warn allows gradual adoption) |
| `react-refresh/only-export-components` | `warn` | Vite HMR compatibility |

---

## 3. Engine Linting (Clippy + fmt)

### 3.1 Existing Tools

The Rust side already has mature tooling:

| Tool | Purpose | Command |
|------|---------|---------|
| `cargo fmt` | Code formatting | `cargo fmt --check` |
| `cargo clippy` | Linting & static analysis | `cargo clippy --workspace -- -D warnings` |

### 3.2 Clippy Configuration (Optional Enhancement)

For stricter rules, add `engine/.clippy.toml`:

```toml
# Warn on cognitive complexity
cognitive-complexity-threshold = 25

# Enforce documentation on public items
missing-docs-in-crate-items = true
```

And in `engine/Cargo.toml` (workspace):

```toml
[workspace.lints.clippy]
# Real-time safety violations
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"

# Code quality
pedantic = { level = "warn", priority = -1 }
nursery = { level = "warn", priority = -1 }

# Allow some pedantic rules that are too noisy
module_name_repetitions = "allow"
must_use_candidate = "allow"
```

> **Note:** This enhancement is optional for initial implementation. The basic `clippy -D warnings` is sufficient.

---

## 4. xtask Commands

### 4.1 Command Structure

```
cargo xtask lint [OPTIONS]

OPTIONS:
    --ui        Run UI linting (ESLint + Prettier)
    --engine    Run engine linting (clippy + fmt)
    --fix       Auto-fix where possible

If no options specified, runs both --ui and --engine.
```

### 4.2 New Files

**File:** `engine/xtask/src/commands/lint.rs`

```rust
//! Lint command - Run linters for UI and/or engine code.

use anyhow::{Context, Result};
use std::process::Command;

use xtask::output::*;
use xtask::paths;

/// Lint target selection.
#[derive(Debug, Clone, Copy)]
pub struct LintTargets {
    pub ui: bool,
    pub engine: bool,
    pub fix: bool,
}

impl Default for LintTargets {
    fn default() -> Self {
        Self {
            ui: true,
            engine: true,
            fix: false,
        }
    }
}

/// Run the lint command.
pub fn run(targets: LintTargets, verbose: bool) -> Result<()> {
    print_header("VstKit Linting");

    let mut ui_result = Ok(());
    let mut engine_result = Ok(());

    // Run engine linting
    if targets.engine {
        engine_result = run_engine_lint(targets.fix, verbose);
    }

    // Run UI linting
    if targets.ui {
        ui_result = run_ui_lint(targets.fix, verbose);
    }

    // Print summary
    println!();
    print_status("Summary:");
    
    if targets.engine {
        match &engine_result {
            Ok(()) => print_success_item("Engine (Rust): PASSED"),
            Err(e) => print_error(&format!("  ✗ Engine (Rust): FAILED - {}", e)),
        }
    }
    
    if targets.ui {
        match &ui_result {
            Ok(()) => print_success_item("UI (TypeScript): PASSED"),
            Err(e) => print_error(&format!("  ✗ UI (TypeScript): FAILED - {}", e)),
        }
    }

    // Return error if any failed
    engine_result?;
    ui_result?;

    println!();
    print_success("All linting checks passed!");
    Ok(())
}

/// Run engine linting (cargo fmt + clippy).
fn run_engine_lint(fix: bool, verbose: bool) -> Result<()> {
    let engine_dir = paths::engine_dir()?;

    println!();
    print_status("Checking Rust formatting...");

    // Step 1: cargo fmt
    let mut fmt_cmd = Command::new("cargo");
    fmt_cmd.current_dir(&engine_dir);
    
    if fix {
        fmt_cmd.args(["fmt"]);
        if verbose {
            println!("Running: cargo fmt");
        }
    } else {
        fmt_cmd.args(["fmt", "--check"]);
        if verbose {
            println!("Running: cargo fmt --check");
        }
    }

    let fmt_status = fmt_cmd.status().context("Failed to run cargo fmt")?;
    if !fmt_status.success() {
        if fix {
            anyhow::bail!("cargo fmt failed");
        } else {
            anyhow::bail!("Formatting issues found. Run 'cargo xtask lint --engine --fix' or 'cargo fmt' to fix.");
        }
    }
    print_success_item("Formatting OK");

    // Step 2: cargo clippy
    print_status("Running Clippy...");

    let mut clippy_cmd = Command::new("cargo");
    clippy_cmd.current_dir(&engine_dir);
    
    if fix {
        clippy_cmd.args(["clippy", "--workspace", "--fix", "--allow-dirty", "--allow-staged", "--", "-D", "warnings"]);
        if verbose {
            println!("Running: cargo clippy --workspace --fix --allow-dirty --allow-staged -- -D warnings");
        }
    } else {
        clippy_cmd.args(["clippy", "--workspace", "--", "-D", "warnings"]);
        if verbose {
            println!("Running: cargo clippy --workspace -- -D warnings");
        }
    }

    let clippy_status = clippy_cmd.status().context("Failed to run cargo clippy")?;
    if !clippy_status.success() {
        anyhow::bail!("Clippy found issues. Run 'cargo xtask lint --engine --fix' or 'cargo clippy --fix' to auto-fix.");
    }
    print_success_item("Clippy OK");

    Ok(())
}

/// Run UI linting (ESLint + Prettier).
fn run_ui_lint(fix: bool, verbose: bool) -> Result<()> {
    let ui_dir = paths::project_root()?.join("ui");

    // Check if node_modules exists
    if !ui_dir.join("node_modules").exists() {
        anyhow::bail!(
            "node_modules not found in ui/. Run 'npm install' in the ui/ directory first."
        );
    }

    println!();
    print_status("Running ESLint...");

    // Step 1: ESLint
    let mut eslint_cmd = Command::new("npm");
    eslint_cmd.current_dir(&ui_dir);
    
    if fix {
        eslint_cmd.args(["run", "lint:fix"]);
        if verbose {
            println!("Running: npm run lint:fix");
        }
    } else {
        eslint_cmd.args(["run", "lint"]);
        if verbose {
            println!("Running: npm run lint");
        }
    }

    let eslint_status = eslint_cmd.status().context("Failed to run ESLint")?;
    if !eslint_status.success() {
        anyhow::bail!("ESLint found issues. Run 'cargo xtask lint --ui --fix' or 'npm run lint:fix' to auto-fix.");
    }
    print_success_item("ESLint OK");

    // Step 2: Prettier
    print_status("Checking Prettier formatting...");

    let mut prettier_cmd = Command::new("npm");
    prettier_cmd.current_dir(&ui_dir);
    
    if fix {
        prettier_cmd.args(["run", "format"]);
        if verbose {
            println!("Running: npm run format");
        }
    } else {
        prettier_cmd.args(["run", "format:check"]);
        if verbose {
            println!("Running: npm run format:check");
        }
    }

    let prettier_status = prettier_cmd.status().context("Failed to run Prettier")?;
    if !prettier_status.success() {
        if fix {
            anyhow::bail!("Prettier formatting failed");
        } else {
            anyhow::bail!("Formatting issues found. Run 'cargo xtask lint --ui --fix' or 'npm run format' to fix.");
        }
    }
    print_success_item("Prettier OK");

    Ok(())
}
```

### 4.3 Main.rs Integration

Add to `engine/xtask/src/main.rs`:

```rust
// In Commands enum:
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

// In match statement:
Some(Commands::Lint { ui, engine, fix }) => {
    let targets = if !ui && !engine {
        // Neither specified = run both
        commands::lint::LintTargets { ui: true, engine: true, fix }
    } else {
        commands::lint::LintTargets { ui, engine, fix }
    };
    commands::lint::run(targets, cli.verbose)
}
```

### 4.4 Commands Module Registration

Update `engine/xtask/src/commands/mod.rs`:

```rust
pub mod lint;
```

### 4.5 UI Directory Path Helper

Add to `engine/xtask/src/lib.rs` in the `paths` module:

```rust
/// Returns the UI directory.
pub fn ui_dir() -> Result<PathBuf> {
    Ok(project_root()?.join("ui"))
}
```

---

## 5. QA Agent Integration

### 5.1 Updated Automated Checks

Update `.github/agents/QA.agent.md`:

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

**Run conditionally for affected crate(s)**:

```bash
# Only for the crate(s) being reviewed
cargo test -p {crate_name}
```

Document all command outputs in the QA report, including:
- Exit codes
- Warning/error counts
- Specific issues found
```

### 5.2 QA Report Template Addition

Add to the QA report template:

```markdown
## Automated Checks

### cargo xtask lint
✅ **PASSED** / ❌ **FAILED**

#### Engine Linting
- `cargo fmt --check`: ✅ PASSED / ❌ FAILED
- `cargo clippy --workspace -- -D warnings`: ✅ PASSED / ❌ FAILED
  - Warnings: 0
  - Errors: 0

#### UI Linting  
- `npm run lint` (ESLint): ✅ PASSED / ❌ FAILED
  - Errors: 0
  - Warnings: 0
- `npm run format:check` (Prettier): ✅ PASSED / ❌ FAILED
```

---

## 6. CI Integration

### 6.1 GitHub Actions Workflow

**File:** `.github/workflows/lint.yml`

```yaml
name: Lint

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  lint-engine:
    name: Lint Engine (Rust)
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        with:
          components: rustfmt, clippy
      
      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: engine
      
      - name: Run engine linting
        run: cargo xtask lint --engine
        working-directory: engine

  lint-ui:
    name: Lint UI (TypeScript)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: ui/package-lock.json
      
      - name: Install dependencies
        run: npm ci
        working-directory: ui
      
      - name: Run UI linting
        run: npm run lint && npm run format:check
        working-directory: ui
```

### 6.2 Integration with Main CI

When the main CI workflow exists, linting should run **first** (fast feedback):

```yaml
jobs:
  lint:
    # ... lint job
  
  build:
    needs: lint  # Build only runs if lint passes
    # ... build job
  
  test:
    needs: lint
    # ... test job
```

---

## 7. File Changes Summary

### New Files

| File | Purpose |
|------|---------|
| `ui/eslint.config.js` | ESLint configuration (flat config) |
| `ui/.prettierrc` | Prettier configuration |
| `ui/.prettierignore` | Prettier ignore patterns |
| `engine/xtask/src/commands/lint.rs` | xtask lint command implementation |
| `.github/workflows/lint.yml` | CI workflow for linting |

### Modified Files

| File | Changes |
|------|---------|
| `ui/package.json` | Add lint/format scripts and dev dependencies |
| `engine/xtask/src/main.rs` | Add `Lint` command to CLI |
| `engine/xtask/src/commands/mod.rs` | Export `lint` module |
| `engine/xtask/src/lib.rs` | Add `ui_dir()` path helper |
| `.github/agents/QA.agent.md` | Add linting to automated checks |

### Dependencies to Install

**UI (npm):**
```bash
cd ui
npm install -D eslint @eslint/js @typescript-eslint/eslint-plugin @typescript-eslint/parser \
  eslint-plugin-react eslint-plugin-react-hooks eslint-plugin-react-refresh \
  eslint-config-prettier prettier
```

**Engine (Rust):**
- No new dependencies (clippy and rustfmt are rustup components)

---

## 8. Implementation Order

1. **Phase 1: UI Tooling** (~30 min)
   - Install npm packages
   - Create `eslint.config.js`
   - Create `.prettierrc` and `.prettierignore`
   - Add scripts to `package.json`
   - Fix any existing lint errors

2. **Phase 2: xtask Commands** (~45 min)
   - Create `lint.rs` command
   - Integrate into `main.rs`
   - Add `ui_dir()` helper
   - Test all flag combinations

3. **Phase 3: QA Agent Update** (~15 min)
   - Update automated checks section
   - Update report template

4. **Phase 4: CI Workflow** (~15 min)
   - Create GitHub Actions workflow
   - Test in PR

---

## 9. Example Usage

```bash
# Run all linting (UI + Engine)
cargo xtask lint

# Run only engine linting
cargo xtask lint --engine

# Run only UI linting  
cargo xtask lint --ui

# Auto-fix all issues
cargo xtask lint --fix

# Auto-fix engine only
cargo xtask lint --engine --fix

# Verbose output
cargo xtask lint -v
```

---

## 10. Open Questions

1. **Should `cargo xtask lint` be part of `cargo xtask all`?**
   - Recommendation: Yes, as a first step before tests

2. **Should we add a pre-commit hook?**
   - Recommendation: Optional future enhancement (husky for UI, cargo-husky for Rust)

3. **Should TypeScript strict mode settings move from tsconfig to ESLint?**
   - Recommendation: Keep in tsconfig; ESLint handles code patterns, TS handles types

---

## Appendix: ESLint Flat Config Migration

ESLint 9 uses "flat config" (`eslint.config.js`) instead of `.eslintrc.*`. Key differences:

| Old (`.eslintrc`) | New (`eslint.config.js`) |
|-------------------|--------------------------|
| `extends: [...]` | Import and spread configs |
| `plugins: [...]` | `plugins: { name: plugin }` |
| `env: { browser: true }` | `languageOptions: { globals: globals.browser }` |
| `overrides: [...]` | Multiple config objects with `files` |

The design above uses flat config for future compatibility.
