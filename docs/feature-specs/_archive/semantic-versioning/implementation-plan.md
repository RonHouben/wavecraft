# Implementation Plan: Semantic Versioning

## Overview

Implement semantic versioning (SemVer) for VstKit plugins with a single source of truth in `engine/Cargo.toml` that propagates to plugin metadata (already working via nih-plug) and the React UI (to be implemented). The version will be injected at build time via Vite's `define` constant, eliminating runtime IPC calls.

## Requirements

From [user-stories.md](./user-stories.md):

- US-1: Version visible in plugin UI
- US-2: Single source of truth in Cargo.toml
- US-3: Version in plugin metadata (VST3/CLAP/AU) — **Already working ✅**
- US-4: Build-time version injection

## Architecture Changes

Based on [low-level-design-semantic-versioning.md](./low-level-design-semantic-versioning.md):

| File | Change |
|------|--------|
| `engine/xtask/Cargo.toml` | Add `toml = "0.8"` dependency |
| `engine/xtask/src/commands/bundle.rs` | Read version from Cargo.toml, pass to npm build via env var |
| `ui/vite.config.ts` | Add `define` block for `__APP_VERSION__` compile-time constant |
| `ui/src/vite-env.d.ts` | Add TypeScript declaration for `__APP_VERSION__` |
| `ui/src/components/VersionBadge.tsx` | **New file** — Component to display version |
| `ui/src/components/VersionBadge.test.tsx` | **New file** — Unit test for VersionBadge |
| `ui/src/App.tsx` | Import and use `VersionBadge` in footer |

---

## Implementation Steps

### Phase 1: Build System — Version Extraction

#### Step 1.1: Add `toml` dependency to xtask

**File:** [engine/xtask/Cargo.toml](../../../engine/xtask/Cargo.toml)

- **Action:** Add `toml = "0.8"` to `[dependencies]` section
- **Why:** Required to parse `engine/Cargo.toml` and extract the workspace version
- **Dependencies:** None
- **Risk:** Low — standard crate with no breaking changes expected
- **Verification:** Run `cargo check -p xtask` — should compile successfully

#### Step 1.2: Create version extraction helper function

**File:** [engine/xtask/src/commands/bundle.rs](../../../engine/xtask/src/commands/bundle.rs)

- **Action:** Add a `read_workspace_version()` function that:
  1. Reads `engine/Cargo.toml`
  2. Parses TOML content
  3. Extracts `workspace.package.version`
  4. Returns `Result<String>` with clear error message if not found
- **Why:** Encapsulates version extraction logic for reuse and testability
- **Dependencies:** Step 1.1 (toml crate)
- **Risk:** Low — straightforward file parsing
- **Verification:** Add unit test that parses a sample Cargo.toml string

#### Step 1.3: Pass version to npm build command

**File:** [engine/xtask/src/commands/bundle.rs](../../../engine/xtask/src/commands/bundle.rs)

- **Action:** Modify the `npm run build` command to include `VITE_APP_VERSION` environment variable:
  ```rust
  Command::new("npm")
      .env("VITE_APP_VERSION", &version)
      .args(["run", "build"])
      // ...
  ```
- **Why:** Vite reads environment variables prefixed with `VITE_` and makes them available during build
- **Dependencies:** Step 1.2 (version extraction)
- **Risk:** Low — environment variable injection is standard practice
- **Verification:** Run `cargo xtask bundle --features webview_editor` with `--verbose`, verify version is printed

---

### Phase 2: UI Build Configuration

#### Step 2.1: Add `define` block to Vite config

**File:** [ui/vite.config.ts](../../../ui/vite.config.ts)

- **Action:** Add `define` property to Vite config:
  ```typescript
  define: {
    '__APP_VERSION__': JSON.stringify(process.env.VITE_APP_VERSION || 'dev'),
  },
  ```
- **Why:** Creates a compile-time constant replacement, meaning the version string is inlined during build (zero runtime cost)
- **Dependencies:** None (can be done in parallel with Phase 1)
- **Risk:** Low — standard Vite feature
- **Verification:** Run `npm run build` and check that built JS contains `"dev"` string (fallback when env var not set)

#### Step 2.2: Add TypeScript type declaration

**File:** [ui/src/vite-env.d.ts](../../../ui/src/vite-env.d.ts)

- **Action:** Append declaration for the global constant:
  ```typescript
  declare const __APP_VERSION__: string;
  ```
- **Why:** Provides type safety for the compile-time constant; prevents TypeScript errors when using `__APP_VERSION__`
- **Dependencies:** Step 2.1 (define block)
- **Risk:** Low — type declaration only
- **Verification:** TypeScript compilation should succeed; IDE should recognize `__APP_VERSION__` as `string`

---

### Phase 3: UI Component

#### Step 3.1: Create VersionBadge component

**File:** `ui/src/components/VersionBadge.tsx` **(new file)**

- **Action:** Create a simple functional component that displays the version:
  ```tsx
  export function VersionBadge(): React.JSX.Element {
    return (
      <span className="text-xs text-gray-500">
        v{__APP_VERSION__}
      </span>
    );
  }
  ```
- **Why:** Encapsulates version display for reusability and testability
- **Dependencies:** Step 2.2 (type declaration)
- **Risk:** Low — simple presentational component
- **Verification:** Component renders without errors in Storybook or test

#### Step 3.2: Create VersionBadge unit test

**File:** `ui/src/components/VersionBadge.test.tsx` **(new file)**

- **Action:** Create test file with:
  1. Test that component renders without crashing
  2. Test that component displays the expected version format (`v{version}`)
- **Why:** Ensures component works correctly and catches regressions
- **Dependencies:** Step 3.1 (component)
- **Risk:** Low
- **Verification:** Run `npm test` — test should pass

#### Step 3.3: Update App.tsx footer

**File:** [ui/src/App.tsx](../../../ui/src/App.tsx)

- **Action:** 
  1. Import `VersionBadge` component
  2. Update footer text to include the version badge:
     ```tsx
     <p>
       VstKit Audio Plugin <VersionBadge /> | React + WKWebView
     </p>
     ```
- **Why:** Displays version in the UI footer as specified in requirements
- **Dependencies:** Step 3.1 (VersionBadge component)
- **Risk:** Low — minimal UI change
- **Verification:** Open plugin UI, verify footer shows version

---

### Phase 4: End-to-End Verification

#### Step 4.1: Test development mode fallback

- **Action:** Run `npm run dev` directly (without xtask)
- **Why:** Verify that `vdev` is displayed when `VITE_APP_VERSION` is not set
- **Dependencies:** All Phase 2 and 3 steps
- **Risk:** Low
- **Verification:** UI footer shows `VstKit Audio Plugin vdev | React + WKWebView`

#### Step 4.2: Test production build with xtask

- **Action:** Run `cargo xtask bundle --features webview_editor`
- **Why:** Verify full build pipeline injects the correct version
- **Dependencies:** All previous phases
- **Risk:** Low
- **Verification:** 
  1. Check `ui/dist/` assets contain the version from `engine/Cargo.toml`
  2. Load plugin in DAW, verify footer shows correct version (e.g., `v0.1.0`)

#### Step 4.3: Verify version matches plugin metadata

- **Action:** Load bundled plugin in a DAW
- **Why:** Confirm UI version matches what the DAW reports in its plugin manager
- **Dependencies:** Step 4.2
- **Risk:** Low
- **Verification:** Version displayed in DAW plugin info matches version in UI footer

---

## Testing Strategy

### Unit Tests

| Test | File | Description |
|------|------|-------------|
| `read_workspace_version` | `engine/xtask/src/commands/bundle.rs` | Parse version from TOML string |
| `VersionBadge renders` | `ui/src/components/VersionBadge.test.tsx` | Component renders with version |

### Integration Tests

| Test | Description |
|------|-------------|
| Build with version | `cargo xtask bundle` injects version into UI assets |
| Dev mode fallback | `npm run dev` shows `vdev` |

### Manual Verification

1. Build plugin: `cargo xtask bundle --features webview_editor`
2. Open in DAW (e.g., Ableton Live, Logic Pro)
3. Verify version in DAW's plugin manager matches UI footer

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `toml` crate version conflict | Low | Low | Pin to `0.8`, verify with `cargo tree` |
| Vite env var not passed | Low | Medium | Log version in verbose mode, fail build if missing |
| Missing Cargo.toml version | Low | High | Fail build with clear error message |

---

## Success Criteria

From [user-stories.md](./user-stories.md):

- [ ] Version number visible in plugin UI footer
- [ ] Version follows SemVer format (e.g., `0.1.0`)
- [ ] Version defined only in `engine/Cargo.toml` (single source of truth)
- [ ] Plugin metadata (VST3/CLAP/AU) reports correct version
- [ ] UI version matches plugin metadata
- [ ] Dev mode shows `vdev` when built without xtask
- [ ] No manual synchronization required across files

---

## Related Documents

- [User Stories](./user-stories.md) — Feature requirements
- [Low-Level Design](./low-level-design-semantic-versioning.md) — Technical design
- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Code conventions
