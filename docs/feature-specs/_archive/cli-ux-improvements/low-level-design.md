# Low-Level Design: CLI UX Improvements

## Overview

This document details the implementation for improving the Wavecraft CLI user experience based on internal testing findings.

**Related:** [User Stories](./user-stories.md) | [Testing Findings](../internal-testing/CLI-findings.md)

---

## Story 1: Help Command

**Status:** Already implemented ✅

Clap provides `--help` automatically. No code changes required.

**Documentation update:** Add reference to `wavecraft --help` in `sdk-getting-started.md`.

---

## Story 2: Remove Personal Information Prompts

### Files Changed

| File | Change |
|------|--------|
| `cli/src/commands/new.rs` | Remove interactive prompts, use defaults |
| `cli/Cargo.toml` | Remove `dialoguer` dependency (if unused elsewhere) |

### Implementation

**Before:**
```rust
fn execute(&self) -> Result<()> {
    // ...
    let vendor = self.get_vendor()?;   // prompts if None
    let email = self.get_email()?;     // prompts if None  
    let url = self.get_url()?;         // prompts if None
    // ...
}
```

**After:**
```rust
fn execute(&self) -> Result<()> {
    // ...
    let vendor = self.vendor.clone().unwrap_or_else(|| "Your Company".to_string());
    let email = self.email.clone();
    let url = self.url.clone();
    // ...
}
```

### Methods to Remove

Delete these methods from `NewCommand` impl (lines 87-124):
- `get_vendor()`
- `get_email()`
- `get_url()`

### Imports to Remove

```rust
// Remove:
use dialoguer::{theme::ColorfulTheme, Input};
```

### Dependency Cleanup

Check if `dialoguer` is used elsewhere. If not, remove from `cli/Cargo.toml`:
```toml
# Remove if unused:
dialoguer = "0.11"
```

---

## Story 3: Remove `--sdk-version`, Rename `--local-dev` to `--local-sdk` (boolean)

### Files Changed

| File | Change |
|------|--------|
| `cli/src/main.rs` | Remove `--sdk-version`, change `--local-dev` to boolean `--local-sdk`, add version constant |
| `cli/src/commands/new.rs` | Update struct field, add repo root detection logic |

### Implementation: `cli/src/main.rs`

**Add version constant (top of file):**
```rust
/// SDK version derived from CLI package version at compile time.
/// Used for git tag dependencies in generated projects.
const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");
```

**Update `Commands::New` enum:**

```rust
#[derive(Subcommand)]
enum Commands {
    /// Create a new plugin project from the template
    New {
        /// Plugin name (lowercase, alphanumeric + underscore/hyphen)
        name: String,
        
        /// Vendor name (company or developer name)
        #[arg(short, long)]
        vendor: Option<String>,
        
        /// Contact email (optional)
        #[arg(short, long)]
        email: Option<String>,
        
        /// Website URL (optional)
        #[arg(short, long)]
        url: Option<String>,
        
        /// Output directory (defaults to plugin name)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Skip git initialization
        #[arg(long)]
        no_git: bool,

        // REMOVED: --sdk-version (now automatic via SDK_VERSION constant)
        
        /// Use local SDK from repository (for SDK development only)
        #[arg(long, hide = true)]
        local_sdk: bool,
    },
}
```

**Update match arm in `main()`:**

```rust
fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New {
            name,
            vendor,
            email,
            url,
            output,
            no_git,
            local_sdk,
        } => {
            let cmd = NewCommand {
                name,
                vendor,
                email,
                url,
                output,
                no_git,
                sdk_version: SDK_VERSION.to_string(),
                local_sdk,
            };
            cmd.execute()?;
        }
    }
    
    Ok(())
}
```

### Implementation: `cli/src/commands/new.rs`

**Update struct:**
```rust
pub struct NewCommand {
    pub name: String,
    pub vendor: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub output: Option<PathBuf>,
    pub no_git: bool,
    pub sdk_version: String,
    pub local_sdk: bool,  // Changed from Option<PathBuf>
}
```

**Add helper function to detect SDK path:**
```rust
fn find_local_sdk_path() -> Result<PathBuf> {
    let sdk_path = PathBuf::from("engine/crates");
    
    if !sdk_path.exists() {
        anyhow::bail!(
            "Error: --local-sdk must be run from the wavecraft repository root.\n\
             Could not find: engine/crates"
        );
    }
    
    sdk_path.canonicalize().context("Failed to resolve SDK path")
}
```

**Update `execute()` to use it:**
```rust
pub fn execute(&self) -> Result<()> {
    // ... validation ...
    
    // Resolve SDK path if --local-sdk is set
    let sdk_path = if self.local_sdk {
        Some(find_local_sdk_path()?)
    } else {
        None
    };
    
    let vars = TemplateVariables::new(
        self.name.clone(),
        vendor,
        email,
        url,
        self.sdk_version.clone(),
        sdk_path,  // Option<PathBuf>, same as before
    );
    
    // ... rest unchanged ...
}
```

### Template Variables

No changes needed to `TemplateVariables` — it still receives `Option<PathBuf>`. The change is only in how that path is determined (auto-detected vs user-provided).

---

## Story 4: Documentation Troubleshooting

### File Changed

| File | Change |
|------|--------|
| `docs/guides/sdk-getting-started.md` | Add troubleshooting note after install step |

### Implementation

After the install step:
```bash
cargo install wavecraft
```

Add this callout:

```markdown
> **Troubleshooting:** If you see `command not found: wavecraft`, your shell PATH 
> may not include Cargo's bin directory. Either restart your terminal, or add it manually:
>
> **zsh:** `echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc`
>
> **bash:** `echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc`
>
> Or run directly: `~/.cargo/bin/wavecraft new my-plugin`
```

---

## CI Impact

### Template Validation Workflow

The workflow at `.github/workflows/template-validation.yml` uses `--local-dev` (to be renamed `--local-sdk`).

**Required update:**
```yaml
# Before:
wavecraft new test-plugin \
  --vendor "Test Vendor" \
  --email "test@example.com" \
  --url "https://example.com" \
  --no-git \
  --local-dev ${{ github.workspace }}/engine/crates

# After:
wavecraft new test-plugin \
  --no-git \
  --local-sdk
```

Much simpler — no path needed, auto-detected from repo root.

---

## Testing Plan

### Manual Testing

1. **Help command:**
   ```bash
   wavecraft --help
   wavecraft new --help
   ```
   Verify `--sdk-path` is NOT shown. Verify `--sdk-version` is gone.

2. **Project generation (no flags):**
   ```bash
   wavecraft new my-test-plugin
   cd my-test-plugin
   ```
   Verify: No prompts, uses defaults, generates valid project.

3. **Project generation (with optional overrides):**
   ```bash
   wavecraft new my-test-plugin --vendor "Acme Audio" --email "dev@acme.com"
   ```
   Verify: Values used in generated files.

4. **Local SDK mode (hidden flag):**
   ```bash
   # From within wavecraft repo:
   wavecraft new my-test-plugin --local-sdk
   ```
   Verify: Auto-detects `engine/crates` path, generates path dependencies.

5. **Local SDK error (outside repo):**
   ```bash
   # From outside wavecraft repo:
   cd /tmp && wavecraft new my-test-plugin --local-sdk
   ```
   Verify: Clear error message about requiring wavecraft repository.

5. **Generated project compiles:**
   ```bash
   cd my-test-plugin
   cd ui && npm install && cd ..
   cargo check --manifest-path engine/Cargo.toml
   ```

### CI Testing

The `template-validation.yml` workflow will validate the full flow on every PR.

---

## Summary of Changes

| Category | Files | Effort |
|----------|-------|--------|
| Code | `cli/src/main.rs`, `cli/src/commands/new.rs` | ~30 lines changed |
| Dependencies | `cli/Cargo.toml` | Remove `dialoguer` (if unused) |
| CI | `.github/workflows/template-validation.yml` | ~3 lines |
| Docs | `docs/guides/sdk-getting-started.md` | ~10 lines |

**Total effort:** Small. Estimated 1-2 hours implementation + testing.
