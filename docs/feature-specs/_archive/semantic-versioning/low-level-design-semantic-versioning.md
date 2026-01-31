# Low-Level Design: Semantic Versioning

## Overview

This document describes the technical design for implementing semantic versioning (SemVer) across VstKit, with a single source of truth in `Cargo.toml` that propagates to plugin metadata and the React UI.

---

## Related Documents

- [User Stories](./user-stories.md) — Feature requirements
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions

---

## Design Goals

1. **Single Source of Truth** — Version defined once in `engine/Cargo.toml`
2. **Automatic Propagation** — Version flows to plugin metadata and UI without manual sync
3. **Build-Time Injection** — UI receives version at build time, not runtime IPC
4. **Zero Runtime Cost** — No additional IPC calls or startup latency

---

## Current State Analysis

### Plugin Metadata (Already Working ✅)

nih-plug already reads the version from Cargo.toml via the `env!` macro:

```rust
// engine/crates/plugin/src/lib.rs (line 55)
impl Plugin for VstKitPlugin {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    // ...
}
```

This works because:
- `engine/Cargo.toml` defines `[workspace.package] version = "0.1.0"`
- `engine/crates/plugin/Cargo.toml` inherits: `version.workspace = true`
- Rust's `env!("CARGO_PKG_VERSION")` reads this at compile time
- nih-plug uses `VERSION` constant for VST3/CLAP metadata

**VST3/CLAP**: Handled automatically by nih-plug.  
**AU (via clap-wrapper)**: Inherits from CLAP metadata.

### UI Version Display (Gap to Fill)

Currently, the UI has no version display. The footer in [App.tsx](../../../ui/src/App.tsx) is static:

```tsx
<footer className="border-t border-plugin-border bg-plugin-surface p-4 text-center text-sm text-gray-400">
  <p>VstKit Audio Plugin | React + WKWebView</p>
</footer>
```

---

## Solution Design

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         BUILD-TIME VERSION FLOW                         │
└─────────────────────────────────────────────────────────────────────────┘

  ┌────────────────────┐
  │ engine/Cargo.toml  │  [workspace.package]
  │ version = "1.0.0"  │  version = "1.0.0"
  └─────────┬──────────┘
            │
            ├──────────────────────────────────────────┐
            │                                          │
            ▼                                          ▼
  ┌─────────────────────┐                   ┌─────────────────────┐
  │ Plugin Binary       │                   │ cargo xtask build-ui│
  │ (Rust compile)      │                   │ reads Cargo.toml    │
  │                     │                   │ exports to Vite env │
  │ env!("CARGO_PKG_    │                   └──────────┬──────────┘
  │      VERSION")      │                              │
  │         │           │                              ▼
  │         ▼           │                   ┌─────────────────────┐
  │  nih-plug Plugin    │                   │ Vite Build          │
  │  VERSION constant   │                   │ VITE_APP_VERSION    │
  │  → VST3/CLAP        │                   │ → import.meta.env   │
  │    metadata         │                   └──────────┬──────────┘
  └─────────────────────┘                              │
                                                       ▼
                                            ┌─────────────────────┐
                                            │ React UI            │
                                            │ import.meta.env.    │
                                            │   VITE_APP_VERSION  │
                                            │ → Footer display    │
                                            └─────────────────────┘
```

### Component Changes

#### 1. Build System: `cargo xtask build-ui`

Modify [build_ui.rs](../../../engine/xtask/src/commands/build_ui.rs) to:

1. Read version from workspace `Cargo.toml`
2. Pass version to Vite via environment variable

```rust
// engine/xtask/src/commands/build_ui.rs

use std::process::Command;
use toml::Value;

pub fn run(verbose: bool) -> Result<()> {
    // ... existing code ...

    // Read version from workspace Cargo.toml
    let version = read_workspace_version()?;
    
    if verbose {
        println!("  Plugin version: {}", version);
    }

    // Run npm build with VITE_APP_VERSION env var
    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .env("VITE_APP_VERSION", &version)  // <-- Inject version
        .current_dir(&ui_dir)
        .status()
        .context("Failed to run npm build")?;

    // ... rest of function ...
}

/// Read version from workspace Cargo.toml
fn read_workspace_version() -> Result<String> {
    let workspace_toml = paths::engine_dir()?.join("Cargo.toml");
    let content = std::fs::read_to_string(&workspace_toml)
        .context("Failed to read workspace Cargo.toml")?;
    
    let toml: Value = content.parse()
        .context("Failed to parse Cargo.toml")?;
    
    let version = toml
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("workspace.package.version not found in Cargo.toml"))?;
    
    Ok(version.to_string())
}
```

**Dependencies**: Add `toml = "0.8"` to `engine/xtask/Cargo.toml`.

#### 2. Vite Configuration

Update [vite.config.ts](../../../ui/vite.config.ts) to expose the environment variable:

```typescript
// ui/vite.config.ts

export default defineConfig({
  // ... existing config ...
  define: {
    // Expose VITE_APP_VERSION as a compile-time constant
    // Falls back to 'dev' for local development without xtask
    '__APP_VERSION__': JSON.stringify(process.env.VITE_APP_VERSION || 'dev'),
  },
});
```

**Why `define` over `.env` files?**
- Environment variables from the shell take precedence over `.env` files
- `define` creates a compile-time replacement (zero runtime cost)
- No need to manage `.env` files that could drift from Cargo.toml

#### 3. TypeScript Type Declaration

Add type declaration for the global constant:

```typescript
// ui/src/vite-env.d.ts (append)

declare const __APP_VERSION__: string;
```

#### 4. React UI: Version Display Component

Create a reusable version display component:

```typescript
// ui/src/components/VersionBadge.tsx

/**
 * Displays the plugin version.
 * Version is injected at build time from Cargo.toml.
 */
export function VersionBadge(): React.JSX.Element {
  return (
    <span className="text-xs text-gray-500">
      v{__APP_VERSION__}
    </span>
  );
}
```

#### 5. App.tsx Footer Update

Update the footer to include the version:

```tsx
// ui/src/App.tsx (footer section)

import { VersionBadge } from './components/VersionBadge';

// ... in render ...

<footer className="border-t border-plugin-border bg-plugin-surface p-4 text-center text-sm text-gray-400">
  <p>
    VstKit Audio Plugin <VersionBadge /> | React + WKWebView
  </p>
</footer>
```

**Rendered output:** `VstKit Audio Plugin v1.0.0 | React + WKWebView`

---

## UI Placement Decision

**Recommendation: Footer**

The footer is the appropriate location for version display because:

1. **Industry convention** — Most audio plugins show version in the footer or an "about" area
2. **Non-intrusive** — Doesn't distract from the main control interface
3. **Always visible** — Users can find it without digging into menus
4. **Existing location** — Footer already exists in the current UI

**Alternatives considered:**

| Location | Pros | Cons |
|----------|------|------|
| Header | Highly visible | Clutters main branding |
| Settings panel | Clean main UI | Requires navigation |
| Tooltip on logo | Non-intrusive | May not be discovered |
| **Footer** ✅ | Conventional, visible, non-intrusive | None significant |

---

## Build Flow Integration

### Development Mode (`npm run dev`)

When running `npm run dev` directly (without xtask), `VITE_APP_VERSION` is not set.
The fallback `'dev'` is displayed: `VstKit Audio Plugin vdev | React + WKWebView`

This clearly indicates a development build.

### Production Build (`cargo xtask build-ui`)

1. xtask reads `version = "1.0.0"` from `engine/Cargo.toml`
2. xtask sets `VITE_APP_VERSION=1.0.0` environment variable
3. Vite replaces `__APP_VERSION__` with `"1.0.0"` at compile time
4. Built assets contain the hardcoded version string

### Plugin Build (`cargo xtask bundle`)

1. `cargo xtask bundle` triggers `cargo xtask build-ui` first (existing behavior)
2. UI assets are built with version baked in
3. `include_dir!` embeds `ui/dist/` into plugin binary
4. Both plugin metadata and UI show the same version

---

## Version Synchronization Guarantee

The version is guaranteed to match across all surfaces:

| Surface | Source | Mechanism |
|---------|--------|-----------|
| Cargo.toml | **Authoritative** | Manual edit |
| Plugin metadata (VST3/CLAP) | `engine/Cargo.toml` | `env!("CARGO_PKG_VERSION")` |
| Plugin metadata (AU) | CLAP metadata | clap-wrapper inheritance |
| React UI | `engine/Cargo.toml` | xtask → env var → Vite define |

**No manual synchronization required.** Bumping the version in `Cargo.toml` is the only step.

---

## File Changes Summary

| File | Change |
|------|--------|
| `engine/xtask/Cargo.toml` | Add `toml = "0.8"` dependency |
| `engine/xtask/src/commands/build_ui.rs` | Read version, pass to npm build |
| `ui/vite.config.ts` | Add `define` block for `__APP_VERSION__` |
| `ui/src/vite-env.d.ts` | Declare `__APP_VERSION__` type |
| `ui/src/components/VersionBadge.tsx` | New component (display version) |
| `ui/src/App.tsx` | Import and use `VersionBadge` in footer |

---

## Testing Strategy

### Unit Tests

1. **VersionBadge component** — Renders version string correctly
2. **read_workspace_version()** — Parses version from Cargo.toml

### Integration Tests

1. **Build verification** — After `cargo xtask build-ui`, check that `ui/dist/` assets contain the expected version string
2. **Plugin metadata** — Load plugin in a test host, verify reported version matches Cargo.toml

### Manual Verification

1. Build plugin: `cargo xtask bundle --features webview_editor`
2. Open in DAW, verify version in plugin manager
3. Open plugin UI, verify footer shows matching version

---

## Edge Cases

### Missing Version in Cargo.toml

If `workspace.package.version` is missing, `cargo xtask build-ui` fails with a clear error:

```
Error: workspace.package.version not found in Cargo.toml
```

This is intentional — we want to fail loudly rather than deploy with an unknown version.

### Local Development Without xtask

Running `npm run dev` directly shows `vdev` as the version. This is acceptable because:
- It's visually obvious this is a development build
- Production builds always go through xtask

### Pre-release Tags (Future)

The current design supports SemVer pre-release tags (e.g., `1.0.0-beta.1`) without modification:
- Cargo.toml accepts pre-release versions
- `env!()` macro captures the full string
- UI displays whatever is in Cargo.toml

This is out of scope for the current implementation but architecturally supported.

---

## Security Considerations

None. The version string is:
- Read from a trusted source (project's own Cargo.toml)
- Injected at build time (not runtime user input)
- Used only for display (not for authorization or logic)

---

## Performance Impact

**Zero runtime impact.**

- Version is a compile-time constant in both Rust and TypeScript
- No IPC calls required
- No additional bundle size beyond the version string literal (~10 bytes)

---

## Rollout Plan

1. Implement xtask version reading
2. Update Vite config
3. Add VersionBadge component
4. Update App.tsx footer
5. Test full build pipeline
6. Document in README (optional)

---

## Open Questions

None. All design decisions have been made.

---

## Appendix: Alternative Approaches Considered

### A. Runtime IPC Call

```typescript
// ❌ Rejected: Adds latency, requires IPC infrastructure
useEffect(() => {
  ipc.invoke('getVersion').then(setVersion);
}, []);
```

**Rejected because:**
- Adds startup latency
- Requires new IPC method
- Version should be known at build time

### B. JSON File in Assets

```
// ❌ Rejected: Extra file to manage, fetch at runtime
ui/public/version.json → { "version": "1.0.0" }
```

**Rejected because:**
- Still requires runtime fetch
- Another file to keep in sync (or generate)
- More complex than environment variable

### C. Git-Based Version

```bash
# ❌ Rejected: Requires git, not always accurate
VERSION=$(git describe --tags --always)
```

**Rejected because:**
- Requires git in build environment
- Tags may not exist or may not match Cargo.toml
- Cargo.toml is already the source of truth for Rust crates
