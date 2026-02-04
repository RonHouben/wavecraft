# Implementation Plan: Open Source Readiness (Milestone 12)

## Overview

**Feature:** Prepare Wavecraft for open source release with CLI scaffolding tool
**Target Version:** `0.7.0`
**Estimated Effort:** 8-10 days
**Branch:** `feature/open-source-readiness`

## Requirements Summary

1. CLI tool: `cargo install wavecraft` → `wavecraft new my-plugin`
2. Template independence: Git dependencies, no monorepo paths
3. Version-locked builds: Git tags for reproducibility
4. Documentation: Fix broken links (excluding `_archive/`)
5. CI validation: Automated template build testing

## Architecture Changes

| Component | File/Directory | Change Type |
|-----------|----------------|-------------|
| CLI crate | `cli/` | NEW |
| CLI template | `cli/template/` | NEW (embedded from wavecraft-plugin-template) |
| Template source | `wavecraft-plugin-template/` | MODIFY (add variables) |
| CI workflow | `.github/workflows/template-validation.yml` | NEW |
| Link checker | `.github/workflows/lint.yml` | MODIFY |
| Documentation | `docs/**/*.md` | MODIFY (fix links) |

---

## Implementation Steps

### Phase 1: Template Variable Conversion (Days 1-2)

#### Step 1.1: Create Template Variable Schema

**File:** `cli/src/template/variables.rs` (design reference, implemented in Phase 2)

- Action: Document all template variables and their transformations
- Why: Establishes contract between template files and CLI
- Dependencies: None
- Risk: Low

**Variables to support:**
| Variable | Transformation | Example |
|----------|---------------|---------|
| `{{plugin_name}}` | Raw input | `my-reverb` |
| `{{plugin_name_snake}}` | to_snake_case | `my_reverb` |
| `{{plugin_name_pascal}}` | to_pascal_case | `MyReverb` |
| `{{plugin_name_title}}` | to_title_case | `My Reverb` |
| `{{vendor}}` | Raw input | `Audio Labs` |
| `{{email}}` | Raw input | `dev@example.com` |
| `{{url}}` | Raw input | `https://example.com` |
| `{{sdk_version}}` | From CLI version | `v0.7.0` |
| `{{year}}` | Current year | `2026` |

---

#### Step 1.2: Convert Template engine/Cargo.toml

**File:** `wavecraft-plugin-template/engine/Cargo.toml`

- Action: Replace hardcoded values with `{{variables}}`
- Why: Enables CLI to customize project metadata
- Dependencies: Step 1.1
- Risk: Medium (must maintain valid TOML when variables replaced)

**Before:**
```toml
[package]
name = "my-plugin"
version.workspace = true

[lib]
name = "my_plugin"

[dependencies]
wavecraft-core = { path = "../../engine/crates/wavecraft-core" }
```

**After:**
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
wavecraft-core = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-protocol = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-dsp = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-bridge = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
wavecraft-metering = { git = "https://github.com/RonHouben/wavecraft", tag = "{{sdk_version}}" }
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "28b149ec4d62757d0b448809148a0c3ca6e09a95" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"

[build-dependencies]
nih_plug_xtask = { git = "https://github.com/robbert-vdh/nih-plug.git", rev = "28b149ec4d62757d0b448809148a0c3ca6e09a95" }
```

---

#### Step 1.3: Convert Template engine/src/lib.rs

**File:** `wavecraft-plugin-template/engine/src/lib.rs`

- Action: Replace hardcoded plugin names with variables
- Why: Generated plugins need unique names/types
- Dependencies: Step 1.1
- Risk: Low

**Before:**
```rust
use wavecraft_core::prelude::*;

wavecraft_processor!(MyGain => Gain);

wavecraft_plugin! {
    name: "My Plugin",
    vendor: "My Company",
    url: "https://example.com",
    email: "contact@example.com",
    signal: MyGain,
}
```

**After:**
```rust
use wavecraft_core::prelude::*;

wavecraft_processor!({{plugin_name_pascal}}Gain => Gain);

wavecraft_plugin! {
    name: "{{plugin_name_title}}",
    vendor: "{{vendor}}",
    url: "{{url}}",
    email: "{{email}}",
    signal: {{plugin_name_pascal}}Gain,
}
```

---

#### Step 1.4: Convert Template Workspace Cargo.toml

**File:** `wavecraft-plugin-template/Cargo.toml`

- Action: Update workspace-level Cargo.toml with variables
- Why: Root manifest needs plugin name
- Dependencies: Step 1.2
- Risk: Low

**After:**
```toml
[workspace]
members = ["engine", "engine/xtask"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
```

---

#### Step 1.5: Convert Template ui/package.json

**File:** `wavecraft-plugin-template/ui/package.json`

- Action: Replace package name with variable
- Why: npm package should match plugin name
- Dependencies: Step 1.1
- Risk: Low

**Before:**
```json
{
  "name": "my-plugin-ui",
  ...
}
```

**After:**
```json
{
  "name": "{{plugin_name}}-ui",
  ...
}
```

---

#### Step 1.6: Convert Template README.md

**File:** `wavecraft-plugin-template/README.md`

- Action: Replace all "My Plugin" references with variables
- Why: README is user-facing documentation
- Dependencies: Step 1.1
- Risk: Low

---

#### Step 1.7: Convert Template LICENSE

**File:** `wavecraft-plugin-template/LICENSE`

- Action: Add `{{year}}` variable for copyright year
- Why: LICENSE should have correct year
- Dependencies: Step 1.1
- Risk: Low

---

#### Step 1.8: Remove Template Workspace Dependency

**File:** `wavecraft-plugin-template/engine/Cargo.toml`

- Action: Remove `version.workspace = true` references
- Why: Generated projects are standalone, not part of a workspace
- Dependencies: Step 1.2
- Risk: Low

---

### Phase 2: CLI Crate Implementation (Days 3-6)

#### Step 2.1: Create CLI Crate Structure

**Files:** `cli/Cargo.toml`, `cli/src/main.rs`

- Action: Create new crate with dependencies
- Why: Entry point for `cargo install wavecraft`
- Dependencies: None
- Risk: Low

```
cli/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── commands/
│   │   ├── mod.rs
│   │   └── new.rs
│   ├── template/
│   │   ├── mod.rs
│   │   └── variables.rs
│   └── validation.rs
└── template/           # Populated during build
```

**cli/Cargo.toml:**
```toml
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
dialoguer = "0.11"
console = "0.15"
indicatif = "0.17"
anyhow = "1.0"
walkdir = "2.4"
include_dir = "0.7"
regex = "1.10"
heck = "0.5"
chrono = "0.4"
```

---

#### Step 2.2: Implement CLI Argument Parsing

**File:** `cli/src/main.rs`

- Action: Set up clap argument parsing with subcommands
- Why: Parse `wavecraft new <name>` and options
- Dependencies: Step 2.1
- Risk: Low

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wavecraft")]
#[command(about = "CLI for creating Wavecraft audio plugins")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Wavecraft plugin project
    New {
        /// Plugin name (lowercase, hyphens/underscores allowed)
        name: String,
        
        /// Plugin vendor/company name
        #[arg(short, long)]
        vendor: Option<String>,
        
        /// Contact email
        #[arg(short, long)]
        email: Option<String>,
        
        /// Plugin/vendor URL
        #[arg(short, long)]
        url: Option<String>,
        
        /// Output directory [default: ./<name>]
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Skip git repository initialization
        #[arg(long)]
        no_git: bool,
        
        /// SDK version tag [default: CLI version]
        #[arg(long)]
        sdk_version: Option<String>,
    },
}
```

---

#### Step 2.3: Implement Crate Name Validation

**File:** `cli/src/validation.rs`

- Action: Validate plugin name follows Rust crate naming rules
- Why: Prevent invalid project creation
- Dependencies: Step 2.1
- Risk: Low

```rust
use anyhow::{bail, Result};
use regex::Regex;

pub fn validate_crate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Plugin name cannot be empty");
    }
    
    if name.len() > 64 {
        bail!("Plugin name cannot exceed 64 characters");
    }
    
    let pattern = Regex::new(r"^[a-z][a-z0-9_-]*$").unwrap();
    if !pattern.is_match(name) {
        bail!(
            "Invalid plugin name '{}'. Must start with a lowercase letter \
             and contain only lowercase letters, numbers, hyphens, or underscores.",
            name
        );
    }
    
    const RESERVED: &[&str] = &["std", "core", "alloc", "test", "proc_macro", "self", "super", "crate"];
    if RESERVED.contains(&name) {
        bail!("'{}' is a reserved Rust keyword and cannot be used as a plugin name", name);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn valid_names() {
        assert!(validate_crate_name("my-plugin").is_ok());
        assert!(validate_crate_name("my_plugin").is_ok());
        assert!(validate_crate_name("plugin123").is_ok());
        assert!(validate_crate_name("a").is_ok());
    }
    
    #[test]
    fn invalid_names() {
        assert!(validate_crate_name("").is_err());
        assert!(validate_crate_name("MyPlugin").is_err());
        assert!(validate_crate_name("123plugin").is_err());
        assert!(validate_crate_name("-plugin").is_err());
        assert!(validate_crate_name("std").is_err());
    }
}
```

---

#### Step 2.4: Implement Interactive Prompts

**File:** `cli/src/commands/new.rs`

- Action: Use dialoguer for missing options
- Why: Better UX than requiring all flags
- Dependencies: Step 2.2
- Risk: Low

```rust
use dialoguer::{Input, theme::ColorfulTheme};

fn prompt_for_metadata(name: &str, vendor: Option<String>, email: Option<String>, url: Option<String>) 
    -> Result<(String, String, String)> 
{
    let theme = ColorfulTheme::default();
    
    let vendor = match vendor {
        Some(v) => v,
        None => Input::with_theme(&theme)
            .with_prompt("Vendor name")
            .default("My Company".into())
            .interact_text()?,
    };
    
    let email = match email {
        Some(e) => e,
        None => Input::with_theme(&theme)
            .with_prompt("Contact email")
            .default("contact@example.com".into())
            .interact_text()?,
    };
    
    let url = match url {
        Some(u) => u,
        None => Input::with_theme(&theme)
            .with_prompt("Plugin URL")
            .default(format!("https://github.com/example/{}", name))
            .interact_text()?,
    };
    
    Ok((vendor, email, url))
}
```

---

#### Step 2.5: Implement Template Variables

**File:** `cli/src/template/variables.rs`

- Action: Define TemplateVariables struct and replacement logic
- Why: Core variable system for template processing
- Dependencies: Step 1.1
- Risk: Low

```rust
use heck::{ToSnakeCase, ToPascalCase, ToTitleCase};
use chrono::Datelike;

pub struct TemplateVariables {
    pub plugin_name: String,
    pub vendor: String,
    pub email: String,
    pub url: String,
    pub sdk_version: String,
}

impl TemplateVariables {
    pub fn new(plugin_name: String, vendor: String, email: String, url: String, sdk_version: String) -> Self {
        Self { plugin_name, vendor, email, url, sdk_version }
    }
    
    pub fn apply(&self, content: &str) -> String {
        let mut result = content.to_string();
        
        // Order matters: more specific patterns first
        result = result.replace("{{plugin_name_snake}}", &self.plugin_name.to_snake_case());
        result = result.replace("{{plugin_name_pascal}}", &self.plugin_name.to_pascal_case());
        result = result.replace("{{plugin_name_title}}", &self.plugin_name.to_title_case());
        result = result.replace("{{plugin_name}}", &self.plugin_name);
        result = result.replace("{{vendor}}", &self.vendor);
        result = result.replace("{{email}}", &self.email);
        result = result.replace("{{url}}", &self.url);
        result = result.replace("{{sdk_version}}", &self.sdk_version);
        result = result.replace("{{year}}", &chrono::Utc::now().year().to_string());
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_variable_replacement() {
        let vars = TemplateVariables::new(
            "my-reverb".into(),
            "Audio Labs".into(),
            "dev@audiolabs.com".into(),
            "https://audiolabs.com".into(),
            "v0.7.0".into(),
        );
        
        assert_eq!(vars.apply("{{plugin_name}}"), "my-reverb");
        assert_eq!(vars.apply("{{plugin_name_snake}}"), "my_reverb");
        assert_eq!(vars.apply("{{plugin_name_pascal}}"), "MyReverb");
        assert_eq!(vars.apply("{{plugin_name_title}}"), "My Reverb");
        assert_eq!(vars.apply("{{vendor}}"), "Audio Labs");
    }
}
```

---

#### Step 2.6: Implement Template Extraction

**File:** `cli/src/template/mod.rs`

- Action: Extract embedded template and apply variables
- Why: Core functionality of `wavecraft new`
- Dependencies: Steps 2.4, 2.5
- Risk: Medium (file system operations)

```rust
use include_dir::{include_dir, Dir};
use std::path::Path;
use std::fs;
use anyhow::Result;

use super::variables::TemplateVariables;

static TEMPLATE: Dir = include_dir!("$CARGO_MANIFEST_DIR/template");

pub fn extract_template(dest: &Path, variables: &TemplateVariables) -> Result<()> {
    extract_dir(&TEMPLATE, dest, variables)
}

fn extract_dir(dir: &Dir, dest: &Path, variables: &TemplateVariables) -> Result<()> {
    fs::create_dir_all(dest)?;
    
    for entry in dir.entries() {
        match entry {
            include_dir::DirEntry::Dir(d) => {
                let dir_name = variables.apply(d.path().file_name().unwrap().to_str().unwrap());
                extract_dir(d, &dest.join(dir_name), variables)?;
            }
            include_dir::DirEntry::File(f) => {
                let file_name = variables.apply(f.path().file_name().unwrap().to_str().unwrap());
                let content = f.contents_utf8().unwrap_or("");
                let processed = variables.apply(content);
                fs::write(dest.join(file_name), processed)?;
            }
        }
    }
    
    Ok(())
}
```

---

#### Step 2.7: Implement New Command

**File:** `cli/src/commands/new.rs`

- Action: Wire everything together for `wavecraft new`
- Why: Main user-facing command
- Dependencies: Steps 2.3, 2.4, 2.5, 2.6
- Risk: Low

```rust
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::template::{extract_template, TemplateVariables};
use crate::validation::validate_crate_name;

pub fn run(
    name: String,
    vendor: Option<String>,
    email: Option<String>,
    url: Option<String>,
    output: Option<PathBuf>,
    no_git: bool,
    sdk_version: Option<String>,
) -> Result<()> {
    // Validate name
    validate_crate_name(&name)?;
    
    // Determine output directory
    let output_dir = output.unwrap_or_else(|| PathBuf::from(&name));
    
    if output_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", output_dir.display());
    }
    
    // Get metadata (interactive if not provided)
    let (vendor, email, url) = prompt_for_metadata(&name, vendor, email, url)?;
    
    // Determine SDK version
    let sdk_version = sdk_version.unwrap_or_else(|| format!("v{}", env!("CARGO_PKG_VERSION")));
    
    // Create variables
    let variables = TemplateVariables::new(name.clone(), vendor, email, url, sdk_version);
    
    // Show progress
    println!("{} Creating plugin '{}'...", style("→").cyan(), name);
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}")?);
    
    // Extract template
    pb.set_message("Extracting template...");
    extract_template(&output_dir, &variables)?;
    
    // Initialize git repo
    if !no_git {
        pb.set_message("Initializing git repository...");
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&output_dir)
            .status()
            .context("Failed to initialize git repository")?;
    }
    
    pb.finish_and_clear();
    
    // Success message
    println!("{} Plugin '{}' created successfully!", style("✓").green(), name);
    println!();
    println!("Next steps:");
    println!("  {} {}", style("cd").cyan(), output_dir.display());
    println!("  {} && {} && {}", 
        style("cd ui && npm install").cyan(),
        style("npm run build").cyan(),
        style("cd ..").cyan());
    println!("  {}", style("cd engine && cargo xtask bundle --release").cyan());
    println!();
    println!("Your plugin will be in {}/engine/target/bundled/", output_dir.display());
    
    Ok(())
}
```

---

#### Step 2.8: Implement Main Entry Point

**File:** `cli/src/main.rs`

- Action: Connect CLI parsing to command execution
- Why: Complete the CLI flow
- Dependencies: Steps 2.2, 2.7
- Risk: Low

```rust
use anyhow::Result;
use clap::Parser;

mod commands;
mod template;
mod validation;

// ... Cli and Commands definitions from Step 2.2 ...

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::New { name, vendor, email, url, output, no_git, sdk_version } => {
            commands::new::run(name, vendor, email, url, output, no_git, sdk_version)
        }
    }
}
```

---

#### Step 2.9: Copy Template for Embedding

**Action:** Create build script or manual process to copy template

- Action: Copy converted template to `cli/template/`
- Why: `include_dir!` needs files at compile time
- Dependencies: Phase 1 complete
- Risk: Medium (keeping templates in sync)

**Option A: Manual copy (simpler)**
```bash
# Before building CLI:
rm -rf cli/template
cp -r wavecraft-plugin-template cli/template
```

**Option B: Build script (automated)**
```rust
// cli/build.rs
use std::fs;
use std::path::Path;

fn main() {
    let src = Path::new("../wavecraft-plugin-template");
    let dest = Path::new("template");
    
    if dest.exists() {
        fs::remove_dir_all(dest).unwrap();
    }
    
    copy_dir_all(src, dest).unwrap();
    
    println!("cargo:rerun-if-changed=../wavecraft-plugin-template");
}
```

**Recommendation:** Option A for M12 (simpler), consider Option B for future.

---

#### Step 2.10: Write CLI Unit Tests

**File:** `cli/src/validation.rs`, `cli/src/template/variables.rs`

- Action: Add comprehensive unit tests
- Why: Ensure correctness of core logic
- Dependencies: Steps 2.3, 2.5
- Risk: Low

(Tests already included in Steps 2.3 and 2.5)

---

### Phase 3: Documentation Fixes (Days 7-8)

#### Step 3.1: Identify Broken Links

**Action:** Run link checker on active docs

- Action: Create script to find broken links
- Why: Identify scope of work
- Dependencies: None
- Risk: Low

```bash
#!/bin/bash
# scripts/check-links.sh

echo "Checking documentation links (excluding _archive)..."

find docs -name "*.md" -not -path "*/_archive/*" -print0 | while IFS= read -r -d '' file; do
    # Extract relative markdown links
    grep -oE '\]\([^)#]+\)' "$file" | sed 's/\](\(.*\))/\1/' | while read -r link; do
        # Skip external URLs
        [[ "$link" =~ ^https?:// ]] && continue
        
        # Resolve relative path
        dir=$(dirname "$file")
        target="$dir/$link"
        
        if [ ! -e "$target" ]; then
            echo "BROKEN: $file -> $link"
        fi
    done
done
```

---

#### Step 3.2: Fix Links in roadmap.md

**File:** `docs/roadmap.md`

- Action: Update links to archived feature specs
- Why: Primary navigation document
- Dependencies: Step 3.1
- Risk: Low

**Common fixes:**
```
feature-specs/declarative-plugin-dsl/ → feature-specs/_archive/declarative-plugin-dsl/
feature-specs/code-quality-polish/ → feature-specs/_archive/code-quality-polish/
feature-specs/websocket-ipc-bridge/ → feature-specs/_archive/websocket-ipc-bridge/
```

---

#### Step 3.3: Fix Links in architecture/*.md

**Files:** `docs/architecture/high-level-design.md`, `docs/architecture/coding-standards.md`

- Action: Fix relative path references
- Why: Developer reference documentation
- Dependencies: Step 3.1
- Risk: Low

---

#### Step 3.4: Fix Links in guides/*.md

**Files:** `docs/guides/*.md`

- Action: Update any broken references
- Why: User-facing tutorials
- Dependencies: Step 3.1
- Risk: Low

---

#### Step 3.5: Update SDK Getting Started Guide

**File:** `docs/guides/sdk-getting-started.md`

- Action: Update for external developer workflow
- Why: Primary onboarding document
- Dependencies: Phase 2 complete
- Risk: Medium (must be accurate)

**Key changes:**
1. Replace template clone with `wavecraft new` instructions
2. Update all commands for external usage
3. Remove monorepo-specific references
4. Add troubleshooting section

---

#### Step 3.6: Update Template README

**File:** `wavecraft-plugin-template/README.md`

- Action: Update for standalone usage
- Why: First thing external developers see
- Dependencies: Phase 1 complete
- Risk: Low

---

#### Step 3.7: Add Link Checker to CI

**File:** `.github/workflows/lint.yml`

- Action: Add link validation step
- Why: Prevent future broken links
- Dependencies: Step 3.1
- Risk: Low

```yaml
- name: Check documentation links
  run: |
    chmod +x scripts/check-links.sh
    ./scripts/check-links.sh | tee link-check.log
    if grep -q "BROKEN:" link-check.log; then
      echo "❌ Broken links found!"
      exit 1
    fi
```

---

### Phase 4: CI & Release (Days 9-10)

#### Step 4.1: Create Template Validation Workflow

**File:** `.github/workflows/template-validation.yml`

- Action: Add CI workflow to test template builds
- Why: Catch template breakage early
- Dependencies: Phase 1 complete
- Risk: Medium (CI complexity)

```yaml
name: Template Validation

on:
  push:
    branches: [main]
    paths:
      - 'wavecraft-plugin-template/**'
      - 'engine/crates/**'
      - 'cli/**'
  pull_request:
    paths:
      - 'wavecraft-plugin-template/**'
      - 'engine/crates/**'
      - 'cli/**'

jobs:
  validate-template:
    name: Build Template
    runs-on: macos-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Build CLI
        run: |
          cp -r wavecraft-plugin-template cli/template
          cd cli && cargo build --release
      
      - name: Create test project
        run: |
          ./cli/target/release/wavecraft new test-plugin \
            --vendor "Test" --email "test@test.com" --url "https://test.com" \
            --output /tmp/test-plugin --no-git
      
      - name: Build test project UI
        run: |
          cd /tmp/test-plugin/ui
          npm install
          npm run build
      
      - name: Build test project plugin
        run: |
          cd /tmp/test-plugin/engine
          cargo xtask bundle --release
      
      - name: Verify artifacts
        run: |
          ls -la /tmp/test-plugin/engine/target/bundled/
          test -d /tmp/test-plugin/engine/target/bundled/test-plugin.vst3
          test -f /tmp/test-plugin/engine/target/bundled/test-plugin.clap
```

---

#### Step 4.2: Create Release Workflow

**File:** `.github/workflows/release.yml`

- Action: Automate releases with git tags
- Why: Consistent release process
- Dependencies: All phases complete
- Risk: Medium

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    name: Create Release
    runs-on: macos-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: dtolnay/rust-action@stable
      
      - name: Build CLI
        run: |
          cp -r wavecraft-plugin-template cli/template
          cd cli && cargo build --release
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: cli/target/release/wavecraft
          generate_release_notes: true
```

---

#### Step 4.3: Version Bump

**Files:** `engine/Cargo.toml`, `cli/Cargo.toml`

- Action: Bump version to 0.7.0
- Why: Signals significant release
- Dependencies: All features complete
- Risk: Low

---

#### Step 4.4: Create Git Tag

**Action:** Manual step to create release tag

```bash
git tag -a v0.7.0 -m "Release 0.7.0 - Open Source Readiness"
git push origin v0.7.0
```

---

#### Step 4.5: Publish CLI to crates.io

**Action:** Publish CLI crate

```bash
cd cli
cargo publish
```

**Prerequisites:**
- crates.io account
- API token configured
- `wavecraft` crate name available (verified in M12 planning)

---

#### Step 4.6: End-to-End Testing

**Action:** Manual verification of complete flow

1. `cargo install wavecraft` (from crates.io)
2. `wavecraft new test-plugin`
3. Build and test plugin
4. Load in DAW

---

## Testing Strategy

### Unit Tests

| Component | Test File | Coverage |
|-----------|-----------|----------|
| Crate validation | `cli/src/validation.rs` | Name rules, edge cases |
| Variable replacement | `cli/src/template/variables.rs` | All transformations |

### Integration Tests

| Test | Command | Validates |
|------|---------|-----------|
| Basic creation | `wavecraft new test-plugin --no-git` | Project structure |
| With options | `wavecraft new test --vendor "X" --email "x@x.com" --url "https://x.com"` | Non-interactive |
| Invalid name | `wavecraft new INVALID` | Error handling |
| Build test | Full build pipeline | Template correctness |

### Manual Tests

| Test | Steps | Expected |
|------|-------|----------|
| Fresh install | `cargo install wavecraft` on clean machine | Works |
| Create & build | `wavecraft new` → build → load in DAW | Plugin works |
| Documentation | Follow SDK guide | All steps work |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Git rate limiting | Medium | High | Document GitHub auth setup |
| Template/SDK drift | Medium | Medium | CI validation |
| crates.io name taken | Low | High | Already verified available |
| nih-plug version mismatch | Low | High | Pin exact revision |

---

## Success Criteria

- [ ] `cargo install wavecraft` works
- [ ] `wavecraft new my-plugin` creates working project
- [ ] Generated project builds without errors
- [ ] Plugin loads in Ableton Live
- [ ] All active documentation links work
- [ ] CI validates template on each PR
- [ ] Version 0.7.0 released with git tag

---

## Estimated Timeline

| Phase | Days | Deliverables |
|-------|------|--------------|
| 1. Template Conversion | 1-2 | Template with variables |
| 2. CLI Implementation | 3-6 | Working CLI |
| 3. Documentation | 7-8 | Fixed links, updated guides |
| 4. CI & Release | 9-10 | Workflows, release |

**Total: 8-10 days**
