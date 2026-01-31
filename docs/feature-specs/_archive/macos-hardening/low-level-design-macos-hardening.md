# Low-Level Design: macOS Hardening & Packaging

> **Milestone:** 4  
> **Status:** Draft  
> **Last Updated:** 2026-01-31

---

## Table of Contents

1. [Overview](#1-overview)
2. [Architecture](#2-architecture)
3. [Entitlements Configuration](#3-entitlements-configuration)
4. [Code Signing Implementation](#4-code-signing-implementation)
5. [Notarization Implementation](#5-notarization-implementation)
6. [Build System Integration](#6-build-system-integration)
7. [CI/CD Pipeline](#7-cicd-pipeline)
8. [Testing Strategy](#8-testing-strategy)
9. [Error Handling](#9-error-handling)
10. [File Structure](#10-file-structure)

---

## 1. Overview

### 1.1 Purpose

This document describes the implementation of macOS code signing and notarization for VstKit plugins. The goal is to produce distributable plugin bundles that:

- Load without security warnings on macOS Catalina+
- Pass Gatekeeper verification
- Work with WKWebView's JavaScript JIT engine
- Support automated CI/CD pipelines

### 1.2 Scope

| In Scope | Out of Scope |
|----------|--------------|
| VST3 bundle signing | Windows code signing |
| CLAP bundle signing | Linux packaging |
| AU component signing | Installer creation (DMG/PKG) |
| Notarization workflow | App Store distribution |
| xtask CLI commands | GUI signing tool |

### 1.3 Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| Xcode Command Line Tools | 14.0+ | `codesign`, `notarytool`, `stapler` |
| Apple Developer Account | - | Developer ID certificate |
| App-specific password | - | Notarization API authentication |

---

## 2. Architecture

### 2.1 Signing Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        Build Pipeline                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  cargo xtask bundle                                              │
│         │                                                        │
│         ▼                                                        │
│  ┌─────────────┐                                                 │
│  │ Unsigned    │                                                 │
│  │ Bundles     │  target/bundled/                                │
│  │ .vst3       │  ├── vstkit.vst3/                               │
│  │ .clap       │  ├── vstkit.clap/                               │
│  │ .component  │  └── vstkit.component/                          │
│  └──────┬──────┘                                                 │
│         │                                                        │
│         ▼                                                        │
│  cargo xtask sign                                                │
│         │                                                        │
│         ├──► Read entitlements.plist                             │
│         │                                                        │
│         ├──► codesign --deep --force --options runtime           │
│         │    --entitlements entitlements.plist                   │
│         │    --sign "Developer ID Application: ..."              │
│         │                                                        │
│         ▼                                                        │
│  ┌─────────────┐                                                 │
│  │ Signed      │                                                 │
│  │ Bundles     │                                                 │
│  └──────┬──────┘                                                 │
│         │                                                        │
│         ▼                                                        │
│  cargo xtask notarize --submit                                   │
│         │                                                        │
│         ├──► Create ZIP archive                                  │
│         ├──► xcrun notarytool submit                             │
│         ├──► Write .notarization-request                         │
│         │                                                        │
│         ▼                                                        │
│  ┌─────────────┐     (5-30 min)                                  │
│  │ Apple       │◄────────────────────┐                           │
│  │ Notary      │                     │                           │
│  │ Service     │─────────────────────┘                           │
│  └──────┬──────┘                                                 │
│         │                                                        │
│         ▼                                                        │
│  cargo xtask notarize --staple                                   │
│         │                                                        │
│         ├──► xcrun notarytool info (check status)                │
│         ├──► xcrun stapler staple (attach ticket)                │
│         │                                                        │
│         ▼                                                        │
│  ┌─────────────┐                                                 │
│  │ Notarized   │  Ready for distribution                         │
│  │ Bundles     │                                                 │
│  └─────────────┘                                                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Responsibilities

| Component | Responsibility |
|-----------|----------------|
| `xtask::commands::sign` | Orchestrate code signing for all bundles |
| `xtask::commands::notarize` | Submit, poll, and staple notarization |
| `xtask::signing::codesign` | Wrapper around `codesign` CLI |
| `xtask::signing::notarytool` | Wrapper around `notarytool` CLI |
| `entitlements.plist` | Runtime entitlements for hardened runtime |

---

## 3. Entitlements Configuration

### 3.1 Why Entitlements Are Required

VstKit embeds WKWebView for the React UI. WebKit's JavaScript engine requires JIT compilation, which is restricted under Apple's hardened runtime. Without proper entitlements, the plugin will crash immediately on load.

### 3.2 Production Entitlements

**File:** `engine/signing/entitlements.plist`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Required for WKWebView JavaScript JIT -->
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    
    <!-- Required for WebKit's unsigned executable memory -->
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    
    <!-- Allow loading plugin dylibs (may be needed for AU wrapper) -->
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
</dict>
</plist>
```

### 3.3 Development Entitlements

**File:** `engine/signing/entitlements-debug.plist`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- All production entitlements -->
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
    
    <!-- Development only: Allow debugging -->
    <key>com.apple.security.cs.debugger</key>
    <true/>
    <key>com.apple.security.get-task-allow</key>
    <true/>
</dict>
</plist>
```

### 3.4 Entitlement Justification

| Entitlement | Required By | Risk Level |
|-------------|-------------|------------|
| `allow-jit` | WKWebView JS engine | Low (standard for WebView apps) |
| `allow-unsigned-executable-memory` | WebKit internals | Low (standard for WebView apps) |
| `disable-library-validation` | AU wrapper loading CLAP | Medium (review if avoidable) |
| `debugger` | LLDB attachment | Dev only |
| `get-task-allow` | Instruments profiling | Dev only |

---

## 4. Code Signing Implementation

### 4.1 Module Structure

**File:** `engine/xtask/src/commands/sign.rs`

```rust
//! Code signing command for macOS bundles.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

use crate::output::*;
use crate::paths;
use crate::Platform;

/// Configuration for code signing.
pub struct SigningConfig {
    /// Developer ID identity (e.g., "Developer ID Application: Name (TEAM_ID)")
    pub identity: String,
    /// Path to entitlements file
    pub entitlements: Option<String>,
    /// Enable verbose output
    pub verbose: bool,
}

impl SigningConfig {
    /// Load signing configuration from environment.
    pub fn from_env() -> Result<Self> {
        let identity = std::env::var("APPLE_SIGNING_IDENTITY")
            .context("APPLE_SIGNING_IDENTITY environment variable not set")?;
        
        let entitlements = std::env::var("APPLE_ENTITLEMENTS").ok();
        
        Ok(Self {
            identity,
            entitlements,
            verbose: false,
        })
    }
}

/// Run the sign command.
pub fn run(config: SigningConfig) -> Result<()> {
    // Verify we're on macOS
    if Platform::current() != Platform::MacOS {
        anyhow::bail!("Code signing is only supported on macOS");
    }
    
    let bundled_dir = paths::bundled_dir()?;
    
    // Sign each bundle type
    let bundles = [
        ("vstkit.vst3", "VST3"),
        ("vstkit.clap", "CLAP"),
        ("vstkit.component", "AU"),
    ];
    
    for (bundle_name, format) in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            print_status(&format!("Signing {} bundle...", format));
            sign_bundle(&bundle_path, &config)?;
            print_success(&format!("{} bundle signed", format));
        } else if config.verbose {
            print_warning(&format!("{} bundle not found, skipping", format));
        }
    }
    
    // Verify signatures
    print_status("Verifying signatures...");
    for (bundle_name, format) in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            verify_signature(&bundle_path)?;
            print_success(&format!("{} signature valid", format));
        }
    }
    
    Ok(())
}

/// Sign a single bundle.
fn sign_bundle(bundle_path: &Path, config: &SigningConfig) -> Result<()> {
    let mut cmd = Command::new("codesign");
    
    cmd.arg("--deep")
       .arg("--force")
       .arg("--options").arg("runtime")  // Enable hardened runtime
       .arg("--timestamp");              // Include secure timestamp
    
    // Add entitlements if specified
    if let Some(ref entitlements) = config.entitlements {
        cmd.arg("--entitlements").arg(entitlements);
    } else {
        // Use default entitlements
        let default_entitlements = paths::engine_dir()?
            .join("signing")
            .join("entitlements.plist");
        if default_entitlements.exists() {
            cmd.arg("--entitlements").arg(&default_entitlements);
        }
    }
    
    cmd.arg("--sign").arg(&config.identity);
    cmd.arg(bundle_path);
    
    let status = cmd.status().context("Failed to run codesign")?;
    
    if !status.success() {
        anyhow::bail!("codesign failed for {}", bundle_path.display());
    }
    
    Ok(())
}

/// Verify a bundle's signature.
fn verify_signature(bundle_path: &Path) -> Result<()> {
    let status = Command::new("codesign")
        .arg("--verify")
        .arg("--deep")
        .arg("--strict")
        .arg(bundle_path)
        .status()
        .context("Failed to run codesign --verify")?;
    
    if !status.success() {
        anyhow::bail!("Signature verification failed for {}", bundle_path.display());
    }
    
    Ok(())
}

/// Check if ad-hoc signing is requested (for local development).
pub fn run_adhoc() -> Result<()> {
    let bundled_dir = paths::bundled_dir()?;
    
    let bundles = ["vstkit.vst3", "vstkit.clap", "vstkit.component"];
    
    for bundle_name in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            print_status(&format!("Ad-hoc signing {}...", bundle_name));
            
            let status = Command::new("codesign")
                .arg("--deep")
                .arg("--force")
                .arg("--sign").arg("-")  // Ad-hoc signature
                .arg(&bundle_path)
                .status()
                .context("Failed to run codesign")?;
            
            if !status.success() {
                anyhow::bail!("Ad-hoc signing failed for {}", bundle_path.display());
            }
        }
    }
    
    print_success("Ad-hoc signing complete");
    Ok(())
}
```

### 4.2 CLI Interface

```
cargo xtask sign [OPTIONS]

OPTIONS:
    --identity <ID>       Signing identity (overrides APPLE_SIGNING_IDENTITY)
    --entitlements <PATH> Path to entitlements.plist
    --adhoc               Use ad-hoc signing (for local development)
    --verbose             Show detailed output
```

### 4.3 Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `APPLE_SIGNING_IDENTITY` | Yes* | Full signing identity string |
| `APPLE_ENTITLEMENTS` | No | Override path to entitlements file |

*Not required if `--adhoc` is used.

---

## 5. Notarization Implementation

### 5.1 Module Structure

**File:** `engine/xtask/src/commands/notarize.rs`

```rust
//! Notarization command for macOS bundles.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::output::*;
use crate::paths;
use crate::Platform;

/// Notarization request state (persisted to disk).
#[derive(Debug, Serialize, Deserialize)]
struct NotarizationRequest {
    request_id: String,
    submitted_at: String,
    bundles: Vec<String>,
}

const REQUEST_FILE: &str = ".notarization-request";

/// Configuration for notarization.
pub struct NotarizeConfig {
    /// Apple ID for notarization
    pub apple_id: String,
    /// Team ID
    pub team_id: String,
    /// App-specific password (or keychain reference)
    pub password: String,
    /// Enable verbose output
    pub verbose: bool,
}

impl NotarizeConfig {
    /// Load configuration from environment.
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            apple_id: std::env::var("APPLE_ID")
                .context("APPLE_ID environment variable not set")?,
            team_id: std::env::var("APPLE_TEAM_ID")
                .context("APPLE_TEAM_ID environment variable not set")?,
            password: std::env::var("APPLE_APP_PASSWORD")
                .or_else(|_| Ok("@keychain:AC_PASSWORD".to_string()))?,
            verbose: false,
        })
    }
}

/// Notarization subcommand.
#[derive(Debug, Clone, Copy)]
pub enum NotarizeAction {
    Submit,
    Status,
    Staple,
    Full,  // Submit, wait, staple (blocking)
}

/// Run the notarize command.
pub fn run(action: NotarizeAction, config: NotarizeConfig) -> Result<()> {
    if Platform::current() != Platform::MacOS {
        anyhow::bail!("Notarization is only supported on macOS");
    }
    
    match action {
        NotarizeAction::Submit => submit(&config),
        NotarizeAction::Status => status(&config),
        NotarizeAction::Staple => staple(&config),
        NotarizeAction::Full => full(&config),
    }
}

/// Submit bundles for notarization.
fn submit(config: &NotarizeConfig) -> Result<()> {
    let bundled_dir = paths::bundled_dir()?;
    let engine_dir = paths::engine_dir()?;
    
    // Create a ZIP of all bundles
    let zip_path = engine_dir.join("target").join("vstkit-notarize.zip");
    
    print_status("Creating submission archive...");
    create_submission_zip(&bundled_dir, &zip_path)?;
    
    print_status("Submitting to Apple notary service...");
    
    let output = Command::new("xcrun")
        .arg("notarytool")
        .arg("submit")
        .arg(&zip_path)
        .arg("--apple-id").arg(&config.apple_id)
        .arg("--team-id").arg(&config.team_id)
        .arg("--password").arg(&config.password)
        .arg("--output-format").arg("json")
        .output()
        .context("Failed to run notarytool")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Notarization submission failed: {}", stderr);
    }
    
    // Parse response to get request ID
    let response: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse notarytool response")?;
    
    let request_id = response["id"]
        .as_str()
        .context("No request ID in response")?
        .to_string();
    
    // Save request state
    let request = NotarizationRequest {
        request_id: request_id.clone(),
        submitted_at: chrono::Utc::now().to_rfc3339(),
        bundles: vec!["vstkit.vst3", "vstkit.clap", "vstkit.component"]
            .into_iter()
            .map(String::from)
            .collect(),
    };
    
    let request_file = engine_dir.join(REQUEST_FILE);
    fs::write(&request_file, serde_json::to_string_pretty(&request)?)
        .context("Failed to write request file")?;
    
    print_success(&format!("Submission successful! Request ID: {}", request_id));
    print_info(&format!("Request saved to: {}", request_file.display()));
    print_info("Run 'cargo xtask notarize --status' to check progress");
    print_info("Run 'cargo xtask notarize --staple' when complete");
    
    // Clean up ZIP
    fs::remove_file(&zip_path).ok();
    
    Ok(())
}

/// Check notarization status.
fn status(config: &NotarizeConfig) -> Result<()> {
    let request = load_request()?;
    
    print_status(&format!("Checking status for request {}...", request.request_id));
    
    let output = Command::new("xcrun")
        .arg("notarytool")
        .arg("info")
        .arg(&request.request_id)
        .arg("--apple-id").arg(&config.apple_id)
        .arg("--team-id").arg(&config.team_id)
        .arg("--password").arg(&config.password)
        .arg("--output-format").arg("json")
        .output()
        .context("Failed to run notarytool info")?;
    
    let response: serde_json::Value = serde_json::from_slice(&output.stdout)
        .context("Failed to parse notarytool response")?;
    
    let status = response["status"].as_str().unwrap_or("unknown");
    
    match status {
        "Accepted" => {
            print_success("Notarization complete! Status: Accepted");
            print_info("Run 'cargo xtask notarize --staple' to staple the ticket");
        }
        "In Progress" => {
            print_info("Notarization in progress... Check again in a few minutes");
        }
        "Invalid" | "Rejected" => {
            print_error(&format!("Notarization failed! Status: {}", status));
            // Fetch log for details
            print_info("Fetching detailed log...");
            fetch_notarization_log(&request.request_id, config)?;
        }
        _ => {
            print_warning(&format!("Unknown status: {}", status));
        }
    }
    
    Ok(())
}

/// Staple notarization ticket to bundles.
fn staple(_config: &NotarizeConfig) -> Result<()> {
    let bundled_dir = paths::bundled_dir()?;
    
    let bundles = ["vstkit.vst3", "vstkit.clap", "vstkit.component"];
    
    for bundle_name in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            print_status(&format!("Stapling ticket to {}...", bundle_name));
            
            let status = Command::new("xcrun")
                .arg("stapler")
                .arg("staple")
                .arg(&bundle_path)
                .status()
                .context("Failed to run stapler")?;
            
            if !status.success() {
                anyhow::bail!("Stapling failed for {}", bundle_name);
            }
            
            print_success(&format!("{} stapled", bundle_name));
        }
    }
    
    // Verify with spctl
    print_status("Verifying notarization...");
    for bundle_name in bundles {
        let bundle_path = bundled_dir.join(bundle_name);
        if bundle_path.exists() {
            let status = Command::new("spctl")
                .arg("--assess")
                .arg("--type").arg("install")
                .arg("--verbose")
                .arg(&bundle_path)
                .status()
                .context("Failed to run spctl")?;
            
            if status.success() {
                print_success(&format!("{} passed Gatekeeper check", bundle_name));
            } else {
                print_warning(&format!("{} may have Gatekeeper issues", bundle_name));
            }
        }
    }
    
    // Clean up request file
    let engine_dir = paths::engine_dir()?;
    fs::remove_file(engine_dir.join(REQUEST_FILE)).ok();
    
    print_success("Notarization complete! Bundles are ready for distribution.");
    
    Ok(())
}

/// Full notarization workflow (blocking).
fn full(config: &NotarizeConfig) -> Result<()> {
    submit(config)?;
    
    print_status("Waiting for notarization to complete...");
    
    // Poll for completion
    let max_attempts = 60;  // 30 minutes at 30-second intervals
    for attempt in 1..=max_attempts {
        std::thread::sleep(std::time::Duration::from_secs(30));
        
        let request = load_request()?;
        
        let output = Command::new("xcrun")
            .arg("notarytool")
            .arg("info")
            .arg(&request.request_id)
            .arg("--apple-id").arg(&config.apple_id)
            .arg("--team-id").arg(&config.team_id)
            .arg("--password").arg(&config.password)
            .arg("--output-format").arg("json")
            .output()
            .context("Failed to run notarytool info")?;
        
        let response: serde_json::Value = serde_json::from_slice(&output.stdout)
            .context("Failed to parse notarytool response")?;
        
        let status = response["status"].as_str().unwrap_or("unknown");
        
        match status {
            "Accepted" => {
                print_success("Notarization accepted!");
                return staple(config);
            }
            "Invalid" | "Rejected" => {
                print_error(&format!("Notarization failed: {}", status));
                fetch_notarization_log(&request.request_id, config)?;
                anyhow::bail!("Notarization rejected");
            }
            _ => {
                print_info(&format!("Attempt {}/{}: Still processing...", attempt, max_attempts));
            }
        }
    }
    
    anyhow::bail!("Notarization timed out after 30 minutes");
}

/// Load the saved notarization request.
fn load_request() -> Result<NotarizationRequest> {
    let engine_dir = paths::engine_dir()?;
    let request_file = engine_dir.join(REQUEST_FILE);
    
    let content = fs::read_to_string(&request_file)
        .context("No pending notarization request. Run 'cargo xtask notarize --submit' first")?;
    
    serde_json::from_str(&content).context("Failed to parse request file")
}

/// Create a ZIP archive for notarization submission.
fn create_submission_zip(bundled_dir: &Path, zip_path: &Path) -> Result<()> {
    // Remove existing ZIP if present
    fs::remove_file(zip_path).ok();
    
    let status = Command::new("ditto")
        .arg("-c")
        .arg("-k")
        .arg("--keepParent")
        .arg(bundled_dir)
        .arg(zip_path)
        .status()
        .context("Failed to create ZIP archive")?;
    
    if !status.success() {
        anyhow::bail!("Failed to create submission ZIP");
    }
    
    Ok(())
}

/// Fetch and display notarization log for failed submissions.
fn fetch_notarization_log(request_id: &str, config: &NotarizeConfig) -> Result<()> {
    let output = Command::new("xcrun")
        .arg("notarytool")
        .arg("log")
        .arg(request_id)
        .arg("--apple-id").arg(&config.apple_id)
        .arg("--team-id").arg(&config.team_id)
        .arg("--password").arg(&config.password)
        .output()
        .context("Failed to fetch notarization log")?;
    
    let log = String::from_utf8_lossy(&output.stdout);
    print_error("Notarization log:");
    println!("{}", log);
    
    Ok(())
}
```

### 5.2 CLI Interface

```
cargo xtask notarize [OPTIONS]

OPTIONS:
    --submit    Submit bundles for notarization
    --status    Check notarization status
    --staple    Staple ticket to bundles (after approval)
    --full      Full workflow: submit, wait, staple (blocking)
    --verbose   Show detailed output
```

### 5.3 Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `APPLE_ID` | Yes | Apple ID email |
| `APPLE_TEAM_ID` | Yes | Developer Team ID (10 characters) |
| `APPLE_APP_PASSWORD` | Yes* | App-specific password or `@keychain:AC_PASSWORD` |

*Can use keychain reference for local development.

---

## 6. Build System Integration

### 6.1 Command Module Registration

**File:** `engine/xtask/src/commands/mod.rs` (additions)

```rust
pub mod sign;
pub mod notarize;
```

### 6.2 Main CLI Updates

**File:** `engine/xtask/src/main.rs` (additions)

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...
    
    /// Sign plugin bundles for macOS distribution
    Sign {
        /// Signing identity (overrides APPLE_SIGNING_IDENTITY)
        #[arg(long)]
        identity: Option<String>,
        
        /// Path to entitlements.plist
        #[arg(long)]
        entitlements: Option<String>,
        
        /// Use ad-hoc signing (for local development)
        #[arg(long)]
        adhoc: bool,
        
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Notarize plugin bundles with Apple
    Notarize {
        /// Submit bundles for notarization
        #[arg(long)]
        submit: bool,
        
        /// Check notarization status
        #[arg(long)]
        status: bool,
        
        /// Staple ticket to bundles
        #[arg(long)]
        staple: bool,
        
        /// Full workflow (submit, wait, staple)
        #[arg(long)]
        full: bool,
        
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Build, sign, and notarize for release
    Release {
        /// Skip notarization (sign only)
        #[arg(long)]
        skip_notarize: bool,
        
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}
```

### 6.3 Release Command

**File:** `engine/xtask/src/commands/release.rs`

```rust
//! Release command - full build, sign, and notarize workflow.

use anyhow::Result;

use super::{bundle, sign, notarize};
use crate::output::*;
use crate::BuildMode;

pub fn run(skip_notarize: bool, verbose: bool) -> Result<()> {
    print_header("VstKit Release Build");
    
    // Step 1: Build with webview feature
    print_status("Building release bundles...");
    bundle::run_with_features(
        BuildMode::Release,
        None,
        &["webview_editor"],
        verbose,
    )?;
    
    // Step 2: Sign
    print_status("Signing bundles...");
    let sign_config = sign::SigningConfig::from_env()?;
    sign::run(sign_config)?;
    
    // Step 3: Notarize (optional)
    if !skip_notarize {
        print_status("Notarizing bundles...");
        let notarize_config = notarize::NotarizeConfig::from_env()?;
        notarize::run(notarize::NotarizeAction::Full, notarize_config)?;
    } else {
        print_warning("Skipping notarization (--skip-notarize)");
    }
    
    print_success("Release build complete!");
    
    Ok(())
}
```

---

## 7. CI/CD Pipeline

### 7.1 GitHub Actions Workflow

**File:** `.github/workflows/release.yml`

```yaml
name: Release Build

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build-macos:
    runs-on: macos-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: ui/package-lock.json
      
      - name: Install UI dependencies
        run: cd ui && npm ci
      
      - name: Import signing certificate
        env:
          CERTIFICATE_P12: ${{ secrets.APPLE_CERTIFICATE_P12 }}
          CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
        run: |
          echo "$CERTIFICATE_P12" | base64 --decode > certificate.p12
          security create-keychain -p "" build.keychain
          security default-keychain -s build.keychain
          security unlock-keychain -p "" build.keychain
          security import certificate.p12 -k build.keychain -P "$CERTIFICATE_PASSWORD" -T /usr/bin/codesign
          security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "" build.keychain
          rm certificate.p12
      
      - name: Build and sign
        env:
          APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
        run: |
          cd engine
          cargo xtask bundle --release --features webview_editor
          cargo xtask sign
      
      - name: Submit for notarization
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
          APPLE_APP_PASSWORD: ${{ secrets.APPLE_APP_PASSWORD }}
        run: |
          cd engine
          cargo xtask notarize --submit
      
      - name: Wait for notarization
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
          APPLE_APP_PASSWORD: ${{ secrets.APPLE_APP_PASSWORD }}
        run: |
          cd engine
          # Poll every 30 seconds for up to 30 minutes
          for i in {1..60}; do
            sleep 30
            if cargo xtask notarize --status 2>&1 | grep -q "Accepted"; then
              echo "Notarization accepted!"
              break
            fi
            echo "Attempt $i: Still processing..."
          done
      
      - name: Staple notarization ticket
        run: |
          cd engine
          cargo xtask notarize --staple
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: vstkit-macos
          path: |
            engine/target/bundled/vstkit.vst3
            engine/target/bundled/vstkit.clap
            engine/target/bundled/vstkit.component
```

### 7.2 Required Secrets

| Secret | Description |
|--------|-------------|
| `APPLE_CERTIFICATE_P12` | Base64-encoded Developer ID certificate |
| `APPLE_CERTIFICATE_PASSWORD` | Certificate password |
| `APPLE_SIGNING_IDENTITY` | Full identity string |
| `APPLE_ID` | Apple ID email |
| `APPLE_TEAM_ID` | Developer Team ID |
| `APPLE_APP_PASSWORD` | App-specific password |

---

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_signing_config_from_env() {
        std::env::set_var("APPLE_SIGNING_IDENTITY", "Test Identity");
        let config = SigningConfig::from_env().unwrap();
        assert_eq!(config.identity, "Test Identity");
    }
    
    #[test]
    fn test_notarization_request_serialization() {
        let request = NotarizationRequest {
            request_id: "abc-123".to_string(),
            submitted_at: "2026-01-31T00:00:00Z".to_string(),
            bundles: vec!["vstkit.vst3".to_string()],
        };
        
        let json = serde_json::to_string(&request).unwrap();
        let parsed: NotarizationRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.request_id, "abc-123");
    }
}
```

### 8.2 Integration Tests

| Test | Description | Validation |
|------|-------------|------------|
| Ad-hoc signing | Sign with `-` identity | `codesign --verify` passes |
| Full signing | Sign with Developer ID | `codesign --verify --deep --strict` passes |
| Notarization flow | Submit mock bundle | Request ID returned |
| Stapling | Staple to signed bundle | `spctl --assess` passes |

### 8.3 Manual Verification Checklist

- [ ] Plugin loads in Ableton Live without security warning
- [ ] Plugin loads on fresh macOS install (no prior approval)
- [ ] `codesign --verify --deep --strict vstkit.vst3` passes
- [ ] `spctl --assess --type install vstkit.vst3` passes
- [ ] Plugin functions correctly after signing (no entitlement issues)

---

## 9. Error Handling

### 9.1 Common Errors and Solutions

| Error | Cause | Solution |
|-------|-------|----------|
| `errSecInternalComponent` | Missing entitlements | Add JIT entitlement |
| `The signature is invalid` | Modified after signing | Re-sign after all modifications |
| `No identity found` | Certificate not in keychain | Import certificate |
| `Notarization rejected` | Unsigned nested code | Sign all nested binaries first |
| `Unable to verify timestamp` | Network issue | Retry with `--timestamp` |

### 9.2 Error Messages

```rust
/// User-friendly error messages for common signing issues.
pub fn diagnose_signing_error(code: i32) -> &'static str {
    match code {
        1 => "Signing failed: Invalid identity or certificate not found. \
              Run 'security find-identity -v -p codesigning' to list available identities.",
        3 => "Signing failed: Bundle is malformed or has invalid structure.",
        _ => "Signing failed: Unknown error. Run with --verbose for details.",
    }
}
```

---

## 10. File Structure

### 10.1 New Files

```
engine/
├── signing/
│   ├── entitlements.plist           # Production entitlements
│   └── entitlements-debug.plist     # Development entitlements
├── xtask/
│   └── src/
│       └── commands/
│           ├── sign.rs              # Signing command
│           ├── notarize.rs          # Notarization command
│           └── release.rs           # Combined release workflow
.github/
└── workflows/
    └── release.yml                  # CI/CD pipeline
docs/
└── guides/
    └── macos-signing.md             # Developer documentation
```

### 10.2 Modified Files

| File | Changes |
|------|---------|
| `engine/xtask/src/commands/mod.rs` | Add `sign`, `notarize`, `release` modules |
| `engine/xtask/src/main.rs` | Add CLI commands |
| `engine/xtask/Cargo.toml` | Add `chrono`, `serde_json` dependencies |

---

## Appendix A: Finding Your Signing Identity

```bash
# List available signing identities
security find-identity -v -p codesigning

# Output example:
# 1) ABC123... "Developer ID Application: Your Name (TEAM_ID)"

# Use the full string in quotes as APPLE_SIGNING_IDENTITY
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
```

## Appendix B: Creating App-Specific Password

1. Go to https://appleid.apple.com/
2. Sign in with your Apple ID
3. Navigate to "App-Specific Passwords"
4. Generate a new password for "VstKit Notarization"
5. Store in keychain or CI secrets

## Appendix C: Local Development Workflow

```bash
# Build without signing (fast iteration)
cargo xtask bundle --features webview_editor

# Ad-hoc sign for local testing (no Apple account needed)
cargo xtask sign --adhoc

# Full signed build (requires Apple Developer account)
export APPLE_SIGNING_IDENTITY="Developer ID Application: ..."
cargo xtask sign
```
