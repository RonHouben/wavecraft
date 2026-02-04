# CI Local Development Dependencies — Low-Level Design

**Status:** ✅ Implemented  
**Created:** 2026-02-04  
**Author:** Architect Agent

---

## Problem Statement

The CI pipeline for template validation fails because:

1. Generated plugins reference SDK crates via git tags (`tag = "v0.7.0"`)
2. Git tags don't exist until **after** PR merge
3. Cargo's `[patch]` mechanism requires the original source to be resolvable **before** applying patches
4. The bootstrap problem: validating code that depends on versions that don't yet exist

### Current Failure

```
error: failed to get `wavecraft-bridge` as a dependency
Caused by: failed to find tag `v0.7.0`
  reference 'refs/remotes/origin/tags/v0.7.0' not found
```

---

## Goals

| Goal | Priority |
|------|----------|
| Fix CI pipeline with first-class CLI feature | Critical |
| Provide `--local-dev` flag for SDK developers | High |
| Maintain clean separation between CI paths and end-user paths | High |
| Zero impact on end-user experience | High |

---

## Non-Goals

- Changing how end-users consume the SDK (git tags remain the distribution model)
- Publishing SDK crates to crates.io (separate feature)
- Modifying the template structure

---

## Solution Architecture

### Dependency Resolution Modes

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    DEPENDENCY RESOLUTION MODES                          │
└─────────────────────────────────────────────────────────────────────────┘

                      ┌─────────────────────┐
                      │  wavecraft new      │
                      │  (CLI invocation)   │
                      └──────────┬──────────┘
                                 │
              ┌──────────────────┴──────────────────┐
              ▼                                     ▼
     ┌────────────────────┐              ┌────────────────────┐
     │  Standard Mode     │              │  Local Dev Mode    │
     │  (End Users)       │              │  (SDK Developers)  │
     └────────┬───────────┘              └────────┬───────────┘
              │                                   │
              ▼                                   ▼
     ┌────────────────────┐              ┌────────────────────┐
     │  Git tag deps:     │              │  Path deps:        │
     │  tag = "v0.7.0"    │              │  path = "../../.." │
     └────────────────────┘              └────────────────────┘
              │                                   │
              ▼                                   ▼
     ┌────────────────────┐              ┌────────────────────────────┐
     │  Fetch from GitHub │              │  Resolve from local        │
     │  (requires tag)    │              │  filesystem (no network)   │
     └────────────────────┘              └────────────────────────────┘
```

### Why Not `[patch]`?

Cargo's patch mechanism:
1. Parses the original `Cargo.toml`
2. Attempts to fetch all dependency sources
3. **Only then** applies patches

This means the fetch must succeed before patches apply — a chicken-and-egg problem for unreleased versions.

---

## Implementation: CLI `--local-dev` Flag

### Motivation

SDK developers and CI pipelines need to validate generated plugins against unreleased SDK code. A `--local-dev` flag provides a first-class solution that:
- Generates correct path dependencies from the start
- Requires no post-processing or sed hacks
- Documents intent via a self-explanatory CLI flag

### CLI Interface Changes

**File:** `cli/src/main.rs`

Add new argument to the `Commands::New` variant:

```rust
#[derive(Subcommand)]
enum Commands {
    /// Create a new plugin project from the template
    New {
        /// Plugin name (lowercase, alphanumeric + underscore/hyphen)
        name: String,
        
        // ... existing arguments ...
        
        /// Wavecraft SDK version to use (git tag)
        #[arg(long, default_value = "v0.7.0")]
        sdk_version: String,
        
        /// Use local SDK path for development (path to engine/crates)
        /// Mutually exclusive with --sdk-version
        #[arg(long, conflicts_with = "sdk_version")]
        local_dev: Option<PathBuf>,
    },
}
```

### NewCommand Changes

**File:** `cli/src/commands/new.rs`

```rust
/// Options for the `new` command.
#[derive(Debug)]
pub struct NewCommand {
    pub name: String,
    pub vendor: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub output: Option<PathBuf>,
    pub no_git: bool,
    pub sdk_version: String,
    pub local_dev: Option<PathBuf>,  // NEW
}
```

Update the `execute` method to pass `local_dev` to `TemplateVariables`:

```rust
// Create template variables
let vars = TemplateVariables::new(
    self.name.clone(),
    vendor,
    email,
    url,
    self.sdk_version.clone(),
    self.local_dev.clone(),  // NEW
);
```

### TemplateVariables Changes

**File:** `cli/src/template/variables.rs`

```rust
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TemplateVariables {
    pub plugin_name: String,
    pub plugin_name_snake: String,
    pub plugin_name_pascal: String,
    pub plugin_name_title: String,
    pub vendor: String,
    pub email: Option<String>,
    pub url: Option<String>,
    pub sdk_version: String,
    pub local_dev: Option<PathBuf>,  // NEW
    pub year: String,
}

impl TemplateVariables {
    pub fn new(
        plugin_name: String,
        vendor: String,
        email: Option<String>,
        url: Option<String>,
        sdk_version: String,
        local_dev: Option<PathBuf>,  // NEW
    ) -> Self {
        // ... existing transformations ...
        
        Self {
            plugin_name,
            plugin_name_snake,
            plugin_name_pascal,
            plugin_name_title,
            vendor,
            email,
            url,
            sdk_version,
            local_dev,  // NEW
            year,
        }
    }
```

### Template Changes

**File:** `plugin-template/engine/Cargo.toml`

The template uses conditional Cargo.toml syntax based on a new marker pattern:

**Option A: Post-processing approach (recommended)**

Keep the template as-is with git tags. Add post-processing in `extract_template()`:

```rust
fn extract_dir(dir: &Dir, target_dir: &Path, vars: &TemplateVariables) -> Result<()> {
    // ... existing code ...
    
    for entry in dir.entries() {
        match entry {
            // ... existing dir handling ...
            
            include_dir::DirEntry::File(file) => {
                // ... existing file handling ...
                
                if let Some(content) = file.contents_utf8() {
                    let mut processed = vars.apply(content)?;
                    
                    // Post-process for local dev mode
                    if vars.local_dev.is_some() {
                        processed = apply_local_dev_overrides(&processed, vars)?;
                    }
                    
                    fs::write(&file_path, processed)?;
                }
            }
        }
    }
    
    Ok(())
}

/// Replaces git dependencies with local path dependencies.
fn apply_local_dev_overrides(content: &str, vars: &TemplateVariables) -> Result<String> {
    let Some(sdk_path) = &vars.local_dev else {
        return Ok(content.to_string());
    };
    
    // Canonicalize the SDK path
    let sdk_path = fs::canonicalize(sdk_path)
        .with_context(|| format!("Invalid local-dev path: {}", sdk_path.display()))?;
    
    let crates = [
        "wavecraft-core",
        "wavecraft-protocol", 
        "wavecraft-dsp",
        "wavecraft-bridge",
        "wavecraft-metering",
    ];
    
    let mut result = content.to_string();
    
    for crate_name in &crates {
        let git_pattern = format!(
            r#"{} = {{ git = "https://github.com/RonHouben/wavecraft", tag = "[^"]*" }}"#,
            crate_name
        );
        let path_replacement = format!(
            r#"{} = {{ path = "{}/{}" }}"#,
            crate_name,
            sdk_path.display(),
            crate_name
        );
        
        let re = Regex::new(&git_pattern)?;
        result = re.replace_all(&result, path_replacement.as_str()).to_string();
    }
    
    Ok(result)
}
```

**Option B: Template markers (alternative)**

Add conditional markers in the template:

```toml
# {{#if local_dev}}
wavecraft-core = { path = "{{local_dev}}/wavecraft-core" }
# {{else}}
wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
# {{/if}}
```

This requires a templating engine (e.g., `handlebars-rs`). **Not recommended** due to added complexity.

---

## Updated CI Workflow

The CI workflow uses `--local-dev` to generate plugins with path dependencies:

```yaml
- name: Generate test plugin (local dev mode)
  run: |
    wavecraft new test-plugin \
      --vendor "Test Vendor" \
      --email "test@example.com" \
      --url "https://example.com" \
      --no-git \
      --local-dev ${{ github.workspace }}/engine/crates
  working-directory: /tmp

# No more override step needed!

- name: Check generated engine code
  run: cargo check --manifest-path engine/Cargo.toml
  working-directory: /tmp/test-plugin
```

---

## Dependency Graph

```
┌──────────────────────────────────────────────────────────────────────┐
│                      AFFECTED FILES                                  │
└──────────────────────────────────────────────────────────────────────┘

  ├── cli/src/main.rs (add --local-dev arg)
  ├── cli/src/commands/new.rs (pass local_dev to variables)
  ├── cli/src/template/mod.rs (add apply_local_dev_overrides)
  ├── cli/src/template/variables.rs (add local_dev field)
  └── .github/workflows/template-validation.yml (use --local-dev flag)
```

---

## Testing Strategy

| Test | Method | Location |
|------|--------|----------|
| `--local-dev` conflicts with custom `--sdk-version` | clap validation | Built-in |
| Invalid path rejected | Unit test | `commands/new.rs` |
| Valid path produces path deps | Unit test | `template/mod.rs` |
| All 5 crates replaced | Integration test | `tests/integration.rs` |
| Generated project compiles | CI validation | GitHub Actions |

**Unit test example:**

```rust
#[test]
fn test_apply_local_dev_overrides() {
    let content = r#"
wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
wavecraft-dsp = { git = "https://github.com/RonHouben/wavecraft", tag = "v0.7.0" }
"#;

    let vars = TemplateVariables {
        local_dev: Some(PathBuf::from("/sdk/crates")),
        // ... other fields ...
    };

    let result = apply_local_dev_overrides(content, &vars).unwrap();
    
    assert!(result.contains(r#"wavecraft-core = { path = "/sdk/crates/wavecraft-core" }"#));
    assert!(result.contains(r#"wavecraft-dsp = { path = "/sdk/crates/wavecraft-dsp" }"#));
    assert!(!result.contains("git ="));
}
```

---

## Migration Path

1. Implement `--local-dev` CLI flag
2. Update CI workflow to use `--local-dev`
3. Remove existing `[patch]` workaround from workflow

---

## Error Handling

| Error | Cause | Handling |
|-------|-------|----------|
| Invalid `--local-dev` path | Path doesn't exist | `anyhow::bail!("Local dev path does not exist: {}")` |
| Path not absolute | Relative path provided | Auto-canonicalize with `fs::canonicalize()` |
| Missing crates | SDK path wrong | cargo check fails with missing dep error |

---

## Architectural Considerations

### Why Not Change the Template to Use Path Dependencies by Default?

1. **End-user experience degrades** — Users would get broken projects
2. **Separation of concerns violated** — Template should represent production configuration
3. **CI concerns leak into product** — Dev workflow shouldn't affect distribution

### Why Not Use Git Branch Instead of Tag?

1. **Reproducibility** — Branches move, tags don't
2. **Version semantics** — Tags communicate version intent
3. **Release workflow** — Tags integrate with `cargo install` and crates.io patterns

### Why Post-processing Instead of Template Conditionals?

1. **Simplicity** — No new templating engine dependency
2. **Auditability** — The template remains human-readable
3. **Minimal surface area** — One function handles the transformation

---

## Summary

| Deliverable | Effort | Impact |
|-------------|--------|--------|
| `--local-dev` CLI flag | 2-4 hours | Fixes CI, improves SDK developer experience |

This is a first-class CLI feature, not a workaround. It makes local SDK development explicit and self-documenting.
