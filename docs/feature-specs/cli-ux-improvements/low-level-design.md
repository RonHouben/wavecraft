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

## Story 3: Remove `--sdk-version`, Rename `--local-dev` to `--sdk-path`

### Files Changed

| File | Change |
|------|--------|
| `cli/src/main.rs` | Remove `--sdk-version`, rename `--local-dev`, add version constant |
| `cli/src/commands/new.rs` | Update struct field name, use constant |

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
        
        /// Path to local SDK crates directory (for SDK development only)
        #[arg(long = "sdk-path", hide = true)]
        sdk_path: Option<PathBuf>,
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
            sdk_path,
        } => {
            let cmd = NewCommand {
                name,
                vendor,
                email,
                url,
                output,
                no_git,
                sdk_version: SDK_VERSION.to_string(),
                sdk_path,
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
    pub sdk_version: String,          // Now always set from CLI constant
    pub sdk_path: Option<PathBuf>,    // Renamed from local_dev
}
```

**Update `TemplateVariables::new()` call:**
```rust
let vars = TemplateVariables::new(
    self.name.clone(),
    vendor,
    email,
    url,
    self.sdk_version.clone(),
    self.sdk_path.clone(),  // renamed from local_dev
);
```

### Template Variables

Check `cli/src/template/variables.rs` — the `local_dev` field name may need updating to `sdk_path` for consistency. The actual behavior (generating path deps vs git deps) remains unchanged.

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

The workflow at `.github/workflows/template-validation.yml` uses `--local-dev` (to be renamed `--sdk-path`).

**Required update:**
```yaml
# Before:
--local-dev ${{ github.workspace }}/engine/crates

# After:
--sdk-path ${{ github.workspace }}/engine/crates
```

Also remove the now-unnecessary flags that will use defaults:
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
  --sdk-path ${{ github.workspace }}/engine/crates
```

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

4. **SDK path mode (hidden flag):**
   ```bash
   wavecraft new my-test-plugin --sdk-path /path/to/wavecraft/engine/crates
   ```
   Verify: Generates path dependencies, not git tag dependencies.

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
