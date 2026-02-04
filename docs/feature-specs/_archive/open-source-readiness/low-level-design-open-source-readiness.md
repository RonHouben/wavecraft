# Low-Level Design: Open Source Readiness (Milestone 12)

## Overview

This document describes the technical design for making Wavecraft ready for open source release. The primary deliverables are:

1. **`wavecraft-cli`** — A CLI tool for project scaffolding
2. **Template independence** — Remove monorepo path dependencies
3. **Version-locked dependencies** — Git tags for reproducible builds
4. **Documentation fixes** — Broken link resolution (excluding `_archive/`)
5. **CI validation** — Automated template build testing

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     OPEN SOURCE DELIVERY ARCHITECTURE                   │
└─────────────────────────────────────────────────────────────────────────┘

  Developer Machine                              GitHub
  ─────────────────                              ──────
  
  ┌──────────────────┐                    ┌─────────────────────┐
  │ cargo install    │────────────────────│ crates.io           │
  │ wavecraft-cli    │                    │ wavecraft-cli 0.7.0 │
  └────────┬─────────┘                    └─────────────────────┘
           │
           │ wavecraft new my-plugin
           ▼
  ┌──────────────────┐                    ┌─────────────────────┐
  │ CLI scaffolds    │◄───────────────────│ GitHub Releases     │
  │ project from     │  fetch template    │ v0.7.0.tar.gz       │
  │ embedded template│  (embedded/fetch)  │ (template snapshot) │
  └────────┬─────────┘                    └─────────────────────┘
           │
           │ Replace placeholders
           ▼
  ┌──────────────────┐
  │ my-plugin/       │
  │ ├── engine/      │
  │ │   └── Cargo.toml ──► git deps with tag = "v0.7.0"
  │ └── ui/          │
  │     └── package.json
  └────────┬─────────┘
           │
           │ cargo xtask bundle
           ▼
  ┌──────────────────┐                    ┌─────────────────────┐
  │ Plugin Build     │◄───────────────────│ GitHub (main repo)  │
  │ fetches SDK via  │  git clone/fetch   │ wavecraft SDK crates│
  │ git dependencies │                    │ @ tag v0.7.0        │
  └──────────────────┘                    └─────────────────────┘
```

---

## Component 1: `wavecraft` CLI Crate

### 1.1 Crate Location & Naming

The CLI will be a **new standalone crate** at the repository root (not inside `engine/`):

```
wavecraft/                   # Repository root (already named wavecraft)
├── cli/                     # NEW: CLI tool (simple folder name)
│   ├── Cargo.toml          # name = "wavecraft" (crates.io name for user UX)
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   └── new.rs       # `wavecraft new` command
│   │   ├── template/
│   │   │   ├── mod.rs       # Template extraction & variable replacement
│   │   │   └── variables.rs # Variable definitions
│   │   └── validation.rs    # Crate name validation
│   └── template/            # Embedded template files (build-time)
├── engine/                  # Existing SDK crates
├── ui/                      # Existing UI library
└── wavecraft-plugin-template/  # Source template (copied into CLI)
```

**Naming Strategy:**
| Aspect | Value | Rationale |
|--------|-------|-----------|
| Folder name | `cli/` | Simple — repo is already `wavecraft` |
| Crate name | `wavecraft` | User UX — `cargo install wavecraft` is clean |
| Binary name | `wavecraft` | User UX — `wavecraft new my-plugin` |

**Rationale:**
- Standalone crate avoids polluting SDK workspace
- Can be published to crates.io independently
- Clear separation: SDK development vs. CLI tooling
- No conflict with SDK crates (all prefixed with `wavecraft-*`)

### 1.2 CLI Interface

```bash
# Installation
cargo install wavecraft

# Usage
wavecraft new <NAME> [OPTIONS]

# Arguments
<NAME>    Plugin name (must be valid Rust crate name: lowercase, underscores)

# Options
-v, --vendor <VENDOR>    Plugin vendor name [default: interactive prompt]
-e, --email <EMAIL>      Contact email [default: interactive prompt]
-u, --url <URL>          Plugin/vendor URL [default: interactive prompt]
-o, --output <DIR>       Output directory [default: ./<NAME>]
    --no-git             Skip git repository initialization
    --sdk-version <VER>  SDK version tag [default: current CLI version]
-q, --quiet              Suppress non-error output
    --verbose            Show detailed progress

# Examples
wavecraft new my-reverb
wavecraft new my-reverb --vendor "Audio Labs" --email "dev@audiolabs.com"
wavecraft new my-reverb -o ~/projects/plugins/
```

### 1.3 Dependencies

```toml
# cli/Cargo.toml
[package]
name = "wavecraft"
version = "0.7.0"
edition = "2021"
description = "CLI tool for creating Wavecraft audio plugins"
license = "MIT"
repository = "https://github.com/RonHouben/wavecraft"
keywords = ["audio", "vst", "clap", "plugin", "dsp"]
categories = ["development-tools", "multimedia::audio"]

[[bin]]
name = "wavecraft"
path = "src/main.rs"

[dependencies]
clap = { version = "4.4", features = ["derive", "color"] }
dialoguer = "0.11"           # Interactive prompts
console = "0.15"             # Colored output, spinners
indicatif = "0.17"           # Progress bars
anyhow = "1.0"               # Error handling
walkdir = "2.4"              # Directory traversal
include_dir = "0.7"          # Embed template at compile time
regex = "1.10"               # Template variable replacement
heck = "0.4"                 # Case conversion (snake_case, PascalCase)
```

### 1.4 Template Embedding Strategy

**Decision:** Embed template files at compile time using `include_dir!`.

```rust
// src/template/mod.rs
use include_dir::{include_dir, Dir};

/// Template embedded at compile time from wavecraft-plugin-template/
static TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/template");

pub fn extract_template(dest: &Path, variables: &TemplateVariables) -> Result<()> {
    for entry in TEMPLATE.entries() {
        extract_entry(entry, dest, variables)?;
    }
    Ok(())
}
```

**Why embed vs. fetch:**
| Approach | Pros | Cons |
|----------|------|------|
| **Embed (chosen)** | Offline support, instant extraction, version-locked | CLI binary larger (~500KB) |
| **Fetch from GitHub** | Smaller binary, can update template without new CLI | Network dependency, auth issues, slower |

**Build process:**
1. CI copies `wavecraft-plugin-template/` → `wavecraft-cli/template/` (with path deps converted)
2. `include_dir!` embeds the converted template
3. CLI extracts and applies variables at runtime

### 1.5 Template Variable System

**Variables defined in template files:**

| Variable | Example Value | Used In |
|----------|---------------|---------|
| `{{plugin_name}}` | `my-reverb` | Cargo.toml, README, directory names |
| `{{plugin_name_snake}}` | `my_reverb` | Rust module names, crate lib name |
| `{{plugin_name_pascal}}` | `MyReverb` | Rust struct names, display name |
| `{{plugin_name_title}}` | `My Reverb` | UI display, plugin name in DAW |
| `{{vendor}}` | `Audio Labs` | Plugin metadata |
| `{{email}}` | `dev@audiolabs.com` | Plugin metadata |
| `{{url}}` | `https://audiolabs.com` | Plugin metadata |
| `{{sdk_version}}` | `v0.7.0` | Git dependency tags |
| `{{year}}` | `2026` | LICENSE file |

**Variable replacement implementation:**

```rust
// src/template/variables.rs
use heck::{ToSnakeCase, ToPascalCase, ToTitleCase};

pub struct TemplateVariables {
    pub plugin_name: String,      // my-reverb
    pub vendor: String,
    pub email: String,
    pub url: String,
    pub sdk_version: String,
}

impl TemplateVariables {
    pub fn replacements(&self) -> Vec<(&str, String)> {
        vec![
            ("{{plugin_name}}", self.plugin_name.clone()),
            ("{{plugin_name_snake}}", self.plugin_name.to_snake_case()),
            ("{{plugin_name_pascal}}", self.plugin_name.to_pascal_case()),
            ("{{plugin_name_title}}", self.plugin_name.to_title_case()),
            ("{{vendor}}", self.vendor.clone()),
            ("{{email}}", self.email.clone()),
            ("{{url}}", self.url.clone()),
            ("{{sdk_version}}", self.sdk_version.clone()),
            ("{{year}}", chrono::Utc::now().year().to_string()),
        ]
    }
}
```

### 1.6 Crate Name Validation

```rust
// src/validation.rs
use regex::Regex;

/// Validates that the name is a valid Rust crate name.
/// Rules: lowercase, alphanumeric + underscore/hyphen, starts with letter, not reserved.
pub fn validate_crate_name(name: &str) -> Result<(), ValidationError> {
    // Check length
    if name.is_empty() || name.len() > 64 {
        return Err(ValidationError::InvalidLength);
    }
    
    // Check pattern: starts with letter, contains only a-z, 0-9, -, _
    let pattern = Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();
    if !pattern.is_match(name) {
        return Err(ValidationError::InvalidFormat(
            "Must start with a letter and contain only lowercase letters, numbers, hyphens, or underscores"
        ));
    }
    
    // Check reserved names
    const RESERVED: &[&str] = &["std", "core", "alloc", "test", "proc_macro"];
    if RESERVED.contains(&name) {
        return Err(ValidationError::ReservedName);
    }
    
    Ok(())
}
```

---

## Component 2: Template Independence

### 2.1 Dependency Conversion

**Current (monorepo path deps):**
```toml
# wavecraft-plugin-template/engine/Cargo.toml
wavecraft-core = { path = "../../engine/crates/wavecraft-core" }
wavecraft-protocol = { path = "../../engine/crates/wavecraft-protocol" }
# ... etc
```

**Target (git deps with version tag):**
```toml
# Generated project Cargo.toml
wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-protocol = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-dsp = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-bridge = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-metering = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
```

### 2.2 Template Source Files (Updated for Variables)

**`engine/Cargo.toml` template:**
```toml
[package]
name = "{{plugin_name}}"
version = "0.1.0"
edition = "2021"
license = "MIT"

[lib]
name = "{{plugin_name_snake}}"
crate-type = ["cdylib"]

[dependencies]
# Wavecraft SDK (version-locked to {{sdk_version}})
wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-protocol = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-dsp = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-bridge = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-metering = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }

# nih-plug (same version as Wavecraft SDK)
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "28b149ec4d62757d0b448809148a0c3ca6e09a95" }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"

[build-dependencies]
nih_plug_xtask = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "28b149ec4d62757d0b448809148a0c3ca6e09a95" }
```

**`engine/src/lib.rs` template:**
```rust
use wavecraft_core::prelude::*;

// Define the processor chain
wavecraft_processor!({{plugin_name_pascal}}Gain => Gain);

// Generate the complete plugin
wavecraft_plugin! {
    name: "{{plugin_name_title}}",
    vendor: "{{vendor}}",
    url: "{{url}}",
    email: "{{email}}",
    signal: {{plugin_name_pascal}}Gain,
}
```

**`ui/package.json` template:**
```json
{
  "name": "{{plugin_name}}-ui",
  "private": true,
  "version": "0.1.0",
  ...
}
```

### 2.3 nih-plug Version Pinning

**Problem:** nih-plug is also a git dependency that must match exactly between Wavecraft SDK and user projects.

**Solution:** Pin to the same git revision in both places:
- Wavecraft SDK's `Cargo.toml`
- Generated template's `Cargo.toml`

The CLI embeds the correct revision at build time (read from SDK's Cargo.toml).

---

## Component 3: Version Management

### 3.1 Git Tag Strategy

```
Release Workflow:
                                                
  1. Bump version in engine/Cargo.toml          
     └─► version = "0.7.0"                      
                                                
  2. Commit with message                        
     └─► "release: v0.7.0"                      
                                                
  3. Create annotated git tag                   
     └─► git tag -a v0.7.0 -m "Release 0.7.0"  
                                                
  4. Push tag                                   
     └─► git push origin v0.7.0                 
                                                
  5. GitHub Release (automated via CI)          
     └─► Attaches CLI binary, changelog         
```

### 3.2 SDK Version Detection

The CLI determines its matching SDK version from its own `Cargo.toml`:

```rust
// src/main.rs
const CLI_VERSION: &str = env!("CARGO_PKG_VERSION");

fn default_sdk_version() -> String {
    format!("v{}", CLI_VERSION)
}
```

**Assumption:** CLI version always matches SDK version (released together).

### 3.3 Future: crates.io Migration Path

When ready for crates.io, dependencies change minimally:

```toml
# Git dependencies (M12)
wavecraft-core = { git = "...", tag = "v0.7.0" }

# crates.io dependencies (future)
wavecraft-core = "0.7"
```

The CLI can support both via a `--registry` flag (future enhancement).

---

## Component 4: Documentation Link Fixes

### 4.1 Scope Definition

| Directory | Action | Rationale |
|-----------|--------|-----------|
| `docs/roadmap.md` | ✅ Fix | Primary navigation document |
| `docs/architecture/*.md` | ✅ Fix | Developer reference |
| `docs/guides/*.md` | ✅ Fix | User-facing tutorials |
| `docs/feature-specs/_archive/**` | ❌ Skip | Historical records |
| `README.md` | ✅ Fix | First impression |
| `CONTRIBUTING.md` | ✅ Fix | Contributor guidance |
| `wavecraft-plugin-template/**` | ✅ Fix | External user docs |

### 4.2 Link Validation Script

Add a simple link checker that respects the exclusion:

```bash
#!/bin/bash
# scripts/check-links.sh

# Find all markdown files, excluding _archive
find docs -name "*.md" -not -path "*/\_archive/*" | while read file; do
  # Extract relative links and check they exist
  grep -oE '\]\([^)]+\)' "$file" | while read link; do
    # Parse and validate...
  done
done
```

**CI Integration:** Run in `lint.yml` workflow on PR.

### 4.3 Common Link Fixes Needed

Based on Issue #5, the main fixes are:

| File | Broken Pattern | Fix |
|------|---------------|-----|
| `roadmap.md` | `feature-specs/declarative-plugin-dsl/` | `feature-specs/_archive/declarative-plugin-dsl/` |
| `roadmap.md` | `feature-specs/code-quality-polish/` | `feature-specs/_archive/code-quality-polish/` |
| `coding-standards.md` | Relative path issues | Adjust `../` prefixes |

---

## Component 5: CI Template Validation

### 5.1 New CI Workflow

```yaml
# .github/workflows/template-validation.yml
name: Template Validation

on:
  push:
    branches: [main]
    paths:
      - 'wavecraft-plugin-template/**'
      - 'engine/crates/**'
  pull_request:
    paths:
      - 'wavecraft-plugin-template/**'
      - 'engine/crates/**'

jobs:
  validate-template:
    name: Build Template as External User
    runs-on: macos-latest
    
    steps:
      - name: Checkout
        uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      # Simulate external user: copy template to isolated location
      - name: Copy template to temp location
        run: |
          cp -r wavecraft-plugin-template /tmp/test-plugin
          cd /tmp/test-plugin
          
          # Convert path deps to git deps pointing to THIS commit
          sed -i '' 's|path = "../../engine/crates/wavecraft-core"|git = "https://github.com/${{ github.repository }}", rev = "${{ github.sha }}"|g' engine/Cargo.toml
          # ... similar for other crates
      
      - name: Build UI
        run: |
          cd /tmp/test-plugin/ui
          npm install
          npm run build
      
      - name: Build Plugin
        run: |
          cd /tmp/test-plugin/engine
          cargo xtask bundle --release
      
      - name: Verify artifacts
        run: |
          ls -la /tmp/test-plugin/engine/target/bundled/
          test -d /tmp/test-plugin/engine/target/bundled/*.vst3
          test -f /tmp/test-plugin/engine/target/bundled/*.clap
```

### 5.2 CI Considerations

**Fork-friendliness:** The workflow must work for forks:
- No secrets required for basic validation
- Uses `${{ github.repository }}` for repo reference
- Uses `${{ github.sha }}` for revision pinning in tests

---

## Implementation Sequence

### Phase 1: Template Conversion (Days 1-2)

1. Create `wavecraft-plugin-template-src/` with template variables
2. Convert all hardcoded values to `{{variables}}`
3. Update `Cargo.toml` to use git dependencies
4. Verify template builds manually (outside monorepo)

### Phase 2: CLI Scaffolding (Days 3-6)

1. Create `wavecraft-cli/` crate structure
2. Implement `clap` argument parsing
3. Implement `dialoguer` interactive prompts
4. Implement template extraction with `include_dir!`
5. Implement variable replacement
6. Add crate name validation
7. Add progress output and error handling
8. Write unit tests

### Phase 3: Documentation (Days 7-8)

1. Run link checker on active docs
2. Fix broken links in `roadmap.md`
3. Fix broken links in `architecture/*.md`
4. Fix broken links in `guides/*.md`
5. Update SDK Getting Started for external users
6. Update template README

### Phase 4: CI & Release (Days 9-10)

1. Add `template-validation.yml` workflow
2. Add link checker to lint workflow
3. Create release workflow with git tagging
4. Test full flow: tag → release → `cargo install` → `wavecraft new`
5. Version bump to 0.7.0

---

## Testing Strategy

### Unit Tests (CLI)

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_name_validation() {
        assert!(validate_crate_name("my-plugin").is_ok());
        assert!(validate_crate_name("my_plugin").is_ok());
        assert!(validate_crate_name("MyPlugin").is_err()); // uppercase
        assert!(validate_crate_name("123plugin").is_err()); // starts with number
        assert!(validate_crate_name("std").is_err()); // reserved
    }
    
    #[test]
    fn test_variable_replacement() {
        let vars = TemplateVariables {
            plugin_name: "my-reverb".into(),
            vendor: "Audio Labs".into(),
            // ...
        };
        let input = "name = \"{{plugin_name}}\"";
        let output = apply_variables(input, &vars);
        assert_eq!(output, "name = \"my-reverb\"");
    }
}
```

### Integration Tests

| Test | Validates |
|------|-----------|
| `wavecraft new test-plugin` | Project created, compiles, bundles |
| `wavecraft new test-plugin --vendor "Test"` | Non-interactive mode |
| `wavecraft new INVALID` | Error message for bad crate name |
| Template CI | Template builds on clean machine |

### Manual Tests

1. Fresh macOS machine → `cargo install wavecraft-cli` → `wavecraft new` → `cargo xtask bundle`
2. Verify plugin loads in Ableton Live
3. Verify all documentation links work

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Git rate limiting for deps | Medium | High | Document auth setup, consider crates.io fallback |
| Template drift from SDK | Medium | Medium | CI validation catches early |
| nih-plug version mismatch | Low | High | Pin exact revision in template |
| crates.io name squatting | Low | Medium | Reserve names before announcement |

---

## Open Questions

1. **CLI distribution:** Publish to crates.io now, or wait until SDK is more stable?
   - **Recommendation:** Publish CLI to crates.io immediately (it's the entry point)

2. **Template source of truth:** Keep `wavecraft-plugin-template/` with path deps, or convert it?
   - **Recommendation:** Keep both — monorepo version for development, embedded version with git deps for CLI

3. **Windows/Linux support:** Test CLI on other platforms?
   - **Recommendation:** macOS first (M12 scope), document platform support for M13+

---

## Appendix: File Changes Summary

| File/Directory | Change |
|----------------|--------|
| `wavecraft-cli/` | NEW — CLI crate |
| `wavecraft-plugin-template/` | MODIFY — Add template variables |
| `.github/workflows/template-validation.yml` | NEW — CI workflow |
| `.github/workflows/lint.yml` | MODIFY — Add link checker |
| `docs/roadmap.md` | MODIFY — Fix broken links |
| `docs/architecture/*.md` | MODIFY — Fix broken links |
| `docs/guides/*.md` | MODIFY — Fix broken links |
| `engine/Cargo.toml` | MODIFY — Bump to 0.7.0 |
