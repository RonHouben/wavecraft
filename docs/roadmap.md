# Wavecraft Roadmap

This document tracks implementation progress against the milestones defined in the [High-Level Design](architecture/high-level-design.md).

---

## Progress Overview

```
┌──────────────────────────────────────────────┐
│  WAVECRAFT ROADMAP          v0.13.0 | 100%  │
├──────────────────────────────────────────────┤
│  ✅ M1-M18.7 Foundation → UI Race Fix       │
│  ✅ M18.8    Agent Search Delegation        │
│  ✅ M18.9    Rust Hot-Reload for Dev        │
│  ✅ M18.10   TS Param Autocomplete          │
│  ✅ M1-M18.10 Foundation → TS Autocomplete │
├──────────────────────────────────────────────┤
│  [████████████████████████] 24/24          │
└──────────────────────────────────────────────┘
```

**See also:** [Backlog](backlog.md) — unprioritized ideas for future consideration

---

## Status Legend

| Symbol | Meaning     |
| ------ | ----------- |
| ✅     | Complete    |
| 🚧     | In Progress |
| ⏳     | Not Started |
| ⚠️     | Blocked     |

---

## Milestone 1: Plugin Skeleton (Week 0–2)

**Status: ✅ Complete**

| Task                               | Status | Notes                                                |
| ---------------------------------- | ------ | ---------------------------------------------------- |
| Rust plugin skeleton with nih-plug | ✅     | Core plugin structure in place                       |
| VST3 export                        | ✅     |                                                      |
| CLAP export                        | ✅     |                                                      |
| Native placeholder UI              | ✅     |                                                      |
| Confirm Ableton host load (VST3)   | ✅     |                                                      |
| Set up clap-wrapper build for AU   | ✅     | CMake configuration in `packaging/macos/au-wrapper/` |
| Confirm Logic Pro load (AU)        | ⏳     |                                                      |
| Confirm GarageBand load (AU)       | ✅     |                                                      |

---

## Milestone 2: WebView Desktop POC (Week 2–4)

**Status: ✅ Complete**

| Task                                           | Status | Notes                                              |
| ---------------------------------------------- | ------ | -------------------------------------------------- |
| Create minimal React app (Vite + TypeScript)   | ✅     | Full React 18 + TypeScript 5 + Vite 6 setup        |
| Embed React app in Rust desktop app via wry    | ✅     | wry 0.47 with WKWebView (macOS)                    |
| Implement basic IPC bridge (JSON-RPC style)    | ✅     | Complete JSON-RPC 2.0 implementation               |
| Test `setParameter` / `getParameter` roundtrip | ✅     | All tests passing (6/6 integration tests)          |
| Test message latency characteristics           | ✅     | p50: 0.003ms, p95: 0.003ms (well below 5ms target) |
| Bundle static assets into Rust binary          | ✅     | `include_dir!` embedding, single binary            |

**Key Deliverables:**

- Protocol layer: JSON-RPC 2.0 message contracts (`engine/crates/protocol`)
- Bridge layer: IPC handler with `ParameterHost` trait (`engine/crates/bridge`)
- Desktop app: Standalone Rust app with embedded WebView (`engine/crates/desktop`)
- React UI: Complete UI library with hooks and components (`ui/`)
- xtask command: `cargo xtask desktop [--build-ui]` for easy testing

**Performance Results:**

- IPC latency: 0.97ms average (runtime), 0.003ms p50 (handler benchmark)
- Bundle size: 150KB JS + 3.69KB CSS (gzipped)
- All 30 unit + integration tests passing

**Lessons Learned:**

- wry 0.47 requires `ControlFlow::Poll` for continuous IPC response delivery
- Responses must be sent via `evaluate_script()` calling `window.__WAVECRAFT_IPC__._receive()`
- Channel-based approach works well for decoupling IPC handler from event loop
- Windows/Linux untested (no dev machines available) but theoretically supported

---

## Milestone 3: Plugin UI Integration (Week 4–8)

**Status: ✅ Complete**

| Task                                                | Status | Notes                                             |
| --------------------------------------------------- | ------ | ------------------------------------------------- |
| Integrate webview into plugin GUI (nih-plug editor) | ✅     | WKWebView with Editor trait                       |
| WKWebView integration (macOS)                       | ✅     | Custom URL scheme handler for assets              |
| WebView2 integration (Windows)                      | ⏳     | Deprioritized — macOS + Ableton is primary target |
| Implement parameter bridge (UI ↔ host params)       | ✅     | GuiContext integration                            |
| Implement SPSC ring buffer for audio → UI metering  | ✅     | rtrb-based MeterProducer/Consumer                 |
| Implement meter visualization in React              | ✅     | Peak/RMS meters with dB display                   |
| Show clipping indicator in meter UI                 | ✅     | Red pulsing button, 2s hold, click-to-reset       |
| Test parameter automation roundtrip                 | ✅     | Tested in Ableton Live                            |
| Plugin editor window resizing                       | ✅     | IPC-based resize with host approval               |

---

## Milestone 4: macOS Hardening & Packaging (Week 8–12)

**Status: ✅ Complete**

> **Scope:** Focused on macOS + Ableton Live as the primary target. Windows/Linux support is deprioritized.

| Task                                      | Status | Notes                                                                   |
| ----------------------------------------- | ------ | ----------------------------------------------------------------------- |
| macOS code signing                        | ✅     | `cargo xtask sign` command implemented                                  |
| macOS notarization                        | ✅     | `cargo xtask notarize` command (deferred until Apple Developer account) |
| Windows code signing                      | ⏳     | Deprioritized                                                           |
| Windows installer (MSI)                   | ⏳     | Deprioritized                                                           |
| Linux packaging (AppImage/Flatpak)        | ⏳     | Deprioritized                                                           |
| **Host Compatibility Testing**            |        |                                                                         |
| Ableton Live (macOS)                      | ✅     | **Primary target** — validated 2026-01-31                               |
| Ableton Live (Windows)                    | ⏳     | Deprioritized                                                           |
| Logic Pro (macOS, AU)                     | ⏳     | Secondary (nice-to-have)                                                |
| GarageBand (macOS, AU)                    | ⏳     | Secondary (nice-to-have)                                                |
| Reaper (all platforms)                    | ⏳     | Deprioritized                                                           |
| Cubase                                    | ⏳     | Deprioritized                                                           |
| FL Studio                                 | ⏳     | Deprioritized                                                           |
| **AU Validation**                         |        |                                                                         |
| `auval` passes without errors             | ✅     | Validated 2026-01-30                                                    |
| Investigate AU custom UI issue            | ⏳     | clap-wrapper shows generic view; root cause TBD                         |
| State save/restore (`.aupreset`)          | ⏳     |                                                                         |
| AU cache invalidation workflow documented | ⏳     |                                                                         |

**Completed:**

- ✅ Entitlements files (production + debug)
- ✅ `cargo xtask sign` command (ad-hoc + Developer ID)
- ✅ `cargo xtask notarize` command (submit/status/staple/full)
- ✅ `cargo xtask release` command (complete workflow)
- ✅ GitHub Actions CI/CD pipeline (build + ad-hoc signing)
- ✅ Comprehensive documentation (`docs/guides/macos-signing.md`)
- ✅ **Ableton Live validation** — plugin loads, UI works, automation syncs, state persists

**Deferred (requires Apple Developer Program):**

- Developer ID signing (Phase 3)
- Notarization submission (Phase 4)
- Signed release CI/CD (Phase 5b)

---

## Milestone 5: Polish & Optimization (Ongoing)

**Status: ✅ Complete**

| Task                            | Status | Notes                                                                                                                                                                                                                                                                       |
| ------------------------------- | ------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Linting infrastructure**      | ✅     | ESLint + Prettier (UI), Clippy + fmt (Engine), `cargo xtask lint` command, CI workflow. Completed 2026-01-31.                                                                                                                                                               |
| **TailwindCSS for React UI**    | ✅     | Utility-first CSS replacing component CSS files. Custom theme with semantic tokens. 3.74KB gzipped (under 10KB target). Completed 2026-01-31.                                                                                                                               |
| **UI unit testing framework**   | ✅     | Vitest + React Testing Library. IPC mock module for isolated component testing. 25 passing tests. `cargo xtask test --ui` command. CI workflow ready (PR trigger disabled pending pipeline redesign). Completed 2026-01-31.                                                 |
| **Resize handle visibility**    | ✅     | Handle visibility improved: 50% white (was 30%), accent blue on hover/drag, 36×36px (was 24×24), 20px scrollbar clearance. WebView background color fixed. Completed 2026-02-01.                                                                                            |
| **Make React UI default**       | ✅     | Removed `webview_editor` feature flag; React UI is now the only editor. Deleted egui fallback. Version bumped to 0.2.0. Completed 2026-02-01.                                                                                                                               |
| **Dead code cleanup**           | ✅     | Platform-gating pattern established for macOS/Windows-only code. Reduced `#[allow(dead_code)]` suppressions from 14 to 3 (79% reduction). Remaining 3 are valid (trait methods called by platform impls). Patterns documented in coding-standards.md. Completed 2026-02-01. |
| **Semantic versioning**         | ✅     | Version extracted from `engine/Cargo.toml` (single source of truth), injected at build time via Vite `define`. VersionBadge component displays version in UI. **Bonus:** Browser dev mode with environment detection and lazy IPC init (partial M6). Completed 2026-01-31.  |
| CI/CD pipeline (GitHub Actions) | ✅     | Redesigned staged pipeline with 6 jobs across 3 stages. Ubuntu for lint/test (cost optimization), macos for build. Branch protection configured. Completed 2026-01-31.                                                                                                      |
| CI pipeline optimization        | ✅     | v0.6.2 — `cargo xtask check` command for fast local validation (~52s vs ~9-12min Docker). Pre-compile test binaries. Tiered artifact retention (7d main / 90d tags). Documentation updated. Completed 2026-02-03.                                                           |
| Performance profiling           | ➡️     | Moved to Milestone 8 (Backlog).                                                                                                                                                                                                                                             |
| Format-specific feature parity  | ➡️     | Moved to Milestone 8 (Backlog).                                                                                                                                                                                                                                             |

---

## Milestone 6: WebSocket IPC Bridge

**Status: ✅ Complete**

> **Goal:** Enable real IPC communication between the React UI running in a browser and the Rust engine, eliminating the need for mock data during development.

**Problem Statement:**
Currently, the UI can only communicate with the Rust engine when running inside WKWebView (plugin or desktop app). When developing with `npm run dev` in a browser, the UI falls back to static mock data. This creates:

- **Double implementation** — Mock behavior can drift from real engine behavior
- **Limited dev experience** — Can't see real meters, test actual parameter changes
- **Testing gaps** — Automated browser testing (Playwright) can't use real engine data

**Solution:**
Add a WebSocket server to the standalone app that exposes the same IPC protocol over `ws://127.0.0.1:9000`. The UI auto-detects the environment and connects via WebSocket when not in WKWebView.

**Benefits:**

- **Single source of truth** — Same `IpcHandler` serves both native and WebSocket transports
- **Real dev experience** — Hot reload with `npm run dev` + live meters, real parameter sync
- **Testing foundation** — Enables Milestone 7 (Playwright visual testing)
- **Future extensibility** — Remote debugging, external tools, mobile companion apps

| Task                                                   | Status | Notes                                         |
| ------------------------------------------------------ | ------ | --------------------------------------------- |
| **Architecture & Design**                              |        |                                               |
| WebSocket IPC bridge design doc                        | ✅     | Transport abstraction, protocol compatibility |
| User stories                                           | ✅     | 7 user stories covering dev workflow          |
| **Rust Implementation**                                |        |                                               |
| Add WebSocket server to standalone crate               | ✅     | `tokio-tungstenite` with async broadcast      |
| Route WebSocket messages through existing `IpcHandler` | ✅     | Same JSON-RPC protocol                        |
| Add `--ws-only` CLI flag                               | ✅     | Headless mode for browser-only dev            |
| Meter data streaming over WebSocket                    | ✅     | Push-based updates at 30fps                   |
| **UI Implementation**                                  |        |                                               |
| Create `WebSocketTransport` class                      | ✅     | Exponential backoff reconnection              |
| Abstract `IpcBridge` to support multiple transports    | ✅     | Factory pattern with lazy init                |
| Auto-detect environment and select transport           | ✅     | WKWebView → native, browser → WebSocket       |
| Reconnection handling                                  | ✅     | Max 5 attempts with backoff (1s→16s)          |
| **Developer Experience**                               |        |                                               |
| Document dev workflow                                  | ✅     | `cargo xtask dev` runs both servers           |
| Unified dev command                                    | ✅     | Single command starts WS + Vite               |
| Graceful degradation in browser                        | ✅     | Shows helpful status when disconnected        |
| **Cleanup**                                            |        |                                               |
| Remove static mock data from `IpcBridge`               | ✅     | Browser mode uses real engine data            |

**Key Deliverables:**

- `WebSocketTransport` class with automatic reconnection
- Transport factory with environment-based selection
- `cargo xtask dev` command for unified development workflow
- Graceful degradation UI for connection status
- 14/14 manual integration tests passing
- 35 UI unit tests, 17 Rust tests passing
- Comprehensive documentation in high-level-design.md

---

## Milestone 7: Browser-Based Visual Testing

**Status: ✅ Complete**

> **Goal:** Enable agent-driven visual testing using Playwright MCP with real engine data (enabled by Milestone 6).

**Depends on:** Milestone 6 (WebSocket IPC Bridge) ✅

| Task                        | Status | Notes                                                      |
| --------------------------- | ------ | ---------------------------------------------------------- |
| **Infrastructure**          |        |                                                            |
| Playwright installation     | ✅     | @playwright/test ^1.41.0, Chromium 145.0.7632.6            |
| Playwright configuration    | ✅     | playwright.config.ts with Chromium, baseURL localhost:5173 |
| .gitignore updates          | ✅     | Excluded playwright-report/ and test-results/              |
| **Test ID Implementation**  |        |                                                            |
| App root test ID            | ✅     | `data-testid="app-root"`                                   |
| Meter component test IDs    | ✅     | 10 IDs (meter, meter-L/R, peak/rms, dB, clip button)       |
| ParameterSlider test IDs    | ✅     | 4 dynamic IDs using template literals                      |
| VersionBadge test ID        | ✅     | `data-testid="version-badge"`                              |
| ResizeHandle test ID        | ✅     | `data-testid="resize-handle"`                              |
| ConnectionStatus test ID    | ✅     | `data-testid="connection-status"`                          |
| **Documentation**           |        |                                                            |
| Visual Testing Guide        | ✅     | 11KB comprehensive guide at docs/guides/visual-testing.md  |
| README link                 | ✅     | Added to Documentation section                             |
| High-level design update    | ✅     | New Visual Testing section with architecture diagram       |
| **Additional Improvements** |        |                                                            |
| Version badge visibility    | ✅     | Improved styling (text-sm, font-medium, text-accent)       |
| Dev mode version display    | ✅     | Reads from Cargo.toml via vite.config.ts parser            |

**Key Deliverables:**

- 18 test IDs across all UI components for reliable Playwright selection
- External baseline storage design (`~/.wavecraft/visual-baselines/`)
- Comprehensive documentation with selector examples and test scenarios
- Version badge now displays correctly in development mode (v0.3.1)
- High-level design updated with Visual Testing architecture

**Test Results:**

- 35/35 UI unit tests passing
- 18/18 manual feature tests passing
- All linting checks passing (ESLint, Prettier, Clippy, fmt)
- QA approved with no blocking issues

**Design Decisions:**
| Decision | Choice | Rationale |
|----------|--------|-----------|
| Automation tool | Playwright MCP | Agent-native, no custom scripts |
| Baseline storage | External (`~/.wavecraft/`) | Keep repo lean |
| Test orchestration | Agent-driven | On-demand, not CI (avoids screenshot flakiness) |
| Component targeting | `data-testid` attributes | Stable, framework-agnostic selectors |

---

## Milestone 8: Developer SDK

**Status: ✅ Complete (Phase 1)**

> **Goal:** Transform Wavecraft from an internal framework into a reusable development kit that other developers can use to build their own VST/CLAP plugins with Rust + React.

**Strategic Context:**
Wavecraft has achieved its internal development goals (Milestones 1–7). The next step is to make it **accessible to other developers** as a proper SDK/toolkit. This required rethinking packaging, documentation, and developer experience.

**User Stories:** [docs/feature-specs/\_archive/developer-sdk/user-stories.md](feature-specs/_archive/developer-sdk/user-stories.md)

### Phase 1: SDK Architecture & Implementation ✅

| Task                                     | Status | Notes                                       |
| ---------------------------------------- | ------ | ------------------------------------------- |
| **Research & Planning**                  |        |                                             |
| User stories                             | ✅     | 6 stories covering SDK design               |
| Low-level design                         | ✅     | 5-crate architecture with clear boundaries  |
| Implementation plan                      | ✅     | 25-step plan across 4 phases                |
| **SDK Crate Restructuring**              |        |                                             |
| `wavecraft-protocol` — IPC contracts     | ✅     | JSON-RPC types, parameter specs             |
| `wavecraft-dsp` — Pure audio processing  | ✅     | `Processor` trait, no framework deps        |
| `wavecraft-bridge` — IPC handling        | ✅     | `ParameterHost` trait, handler              |
| `wavecraft-metering` — Real-time meters  | ✅     | SPSC ring buffer, lock-free                 |
| `wavecraft-core` — Framework integration | ✅     | `wavecraft_plugin!` macro, nih-plug wrapper |
| **Developer Experience**                 |        |                                             |
| `wavecraft_plugin!` macro                | ✅     | Single-line plugin declaration              |
| Prelude re-exports                       | ✅     | `use wavecraft_core::prelude::*`            |
| Plugin template                          | ✅     | Working example with xtask bundler          |
| **Documentation**                        |        |                                             |
| SDK Getting Started guide                | ✅     | `docs/guides/sdk-getting-started.md`        |
| High-level design updates                | ✅     | SDK architecture documented                 |
| Coding standards updates                 | ✅     | `unwrap()`/`expect()` guidelines added      |
| **Quality Assurance**                    |        |                                             |
| 111 Engine tests                         | ✅     | All passing                                 |
| 35 UI tests                              | ✅     | All passing                                 |
| 22 manual tests                          | ✅     | All passing (incl. visual testing)          |
| Linting                                  | ✅     | Rust + TypeScript clean                     |
| Code signing                             | ✅     | Ad-hoc signing verified                     |

**Key Deliverables:**

- **5-crate SDK architecture** with clear domain boundaries
- **`wavecraft_plugin!` macro** for zero-boilerplate plugin declaration
- **Template project** (`plugin-template/`) demonstrating full SDK usage
- **SDK Getting Started guide** for developers
- **Version 0.4.0** released

**Test Results:**

```
Engine Tests: 111 passed, 0 failed, 4 ignored (environment-dependent)
UI Tests:     35 passed, 0 failed
Manual Tests: 22/22 passed (template compilation, bundling, visual, signing)
Linting:      All checks passed (cargo fmt, clippy, ESLint, Prettier)
```

### Phase 2: Publication (Future)

_To be planned when ready to publish to crates.io._

Potential areas:

- Publish SDK crates to crates.io
- CLI tool for project scaffolding (`cargo wavecraft create my-plugin`)
- Additional example plugins (EQ, synth starter)
- Migration guides for SDK updates

### Phase 2: Publication (Future)

_To be planned when ready to publish to crates.io._

Potential areas:

- Publish SDK crates to crates.io
- CLI tool for project scaffolding (`cargo wavecraft create my-plugin`)
- Additional example plugins (EQ, synth starter)
- Migration guides for SDK updates

---

## Milestone 9: Project Rename (VstKit → Wavecraft)

**Status: ✅ Complete**

> **Goal:** Rename the project from "VstKit" to "Wavecraft" to avoid potential "VST" trademark concerns before public/open-source release.

**Rationale:**
"VST" is a Steinberg trademark. While "VstKit" may be defensible as a toolkit name, rebranding to "Wavecraft" eliminates any trademark risk and establishes a unique, memorable identity for the project.

**User Stories:** [docs/feature-specs/\_archive/project-rename-wavecraft/user-stories.md](feature-specs/_archive/project-rename-wavecraft/user-stories.md)

**Scope:**
| Area | Changes Required |
|------|------------------|
| GitHub | Repository name, organization (if applicable) |
| Rust crates | `wavecraft-*` → `wavecraft-*` (all 5 SDK crates) |
| npm packages | `@wavecraft/*` → `@wavecraft/*` namespace |
| Documentation | All docs, guides, README references |
| UI | Any user-facing "Wavecraft" branding |
| Code | Module names, comments, macro names |

### Pre-Rename Checklist (Verified 2026-02-02)

| Check                              | Status | Notes                                                                   |
| ---------------------------------- | ------ | ----------------------------------------------------------------------- |
| GitHub: `wavecraft` available      | ⚠️     | User exists (inactive since 2020). Using `RonHouben/wavecraft` for now. |
| crates.io: `wavecraft-*` available | ✅     | All names available (`wavecraft`, `wavecraft-core`, etc.)               |
| npm: `@wavecraft/*` available      | ✅     | Namespace available                                                     |
| Domain: `wavecraft.dev` available  | ✅     | Available at €10.89/yr (optional, not registering now)                  |

### Tasks

| Task                                                 | Status | Notes                                        |
| ---------------------------------------------------- | ------ | -------------------------------------------- |
| **Planning**                                         |        |                                              |
| Availability checks (GitHub, crates.io, npm, domain) | ✅     | Verified 2026-02-02                          |
| Create user stories                                  | ✅     | 9 user stories created                       |
| Create low-level design                              | ✅     | Comprehensive 13-section design              |
| Create implementation plan                           | ✅     | 8-phase, 50-step plan                        |
| **Implementation**                                   |        |                                              |
| Rename Rust crates                                   | ✅     | `vstkit-*` → `wavecraft-*` (5 crates)        |
| Update `Cargo.toml` workspace                        | ✅     | Package names, dependencies, authors         |
| Update `vstkit_plugin!` macro                        | ✅     | → `wavecraft_plugin!`                        |
| Update npm package names                             | ✅     | `@vstkit/*` → `@wavecraft/*`                 |
| Update all documentation                             | ✅     | README, guides, architecture docs            |
| Update UI branding                                   | ✅     | IPC global `__WAVECRAFT_IPC__`               |
| Update template project                              | ✅     | Full `plugin-template/`                      |
| Update AU wrapper                                    | ✅     | CMakeLists.txt with Wavecraft naming         |
| **CI/CD**                                            |        |                                              |
| Update GitHub Actions workflows                      | ✅     | Artifact names: `wavecraft-*`                |
| Update bundle paths                                  | ✅     | `wavecraft-core.vst3`, `wavecraft-core.clap` |
| **Testing & QA**                                     |        |                                              |
| Manual testing (24 test cases)                       | ✅     | All passing                                  |
| QA review                                            | ✅     | Approved, all findings resolved              |
| Architect review                                     | ✅     | Architectural docs updated                   |
| **Migration (Deferred)**                             |        |                                              |
| GitHub repository rename                             | ⏳     | Post-merge task (creates redirect)           |

**Key Deliverables:**

- **156 files changed** in initial rename commit
- **Version 0.5.0** (breaking change, minor version bump)
- **5 SDK crates renamed**: `wavecraft-protocol`, `wavecraft-dsp`, `wavecraft-bridge`, `wavecraft-metering`, `wavecraft-core`
- **Template fully updated**: `plugin-template/` with correct dependencies and IPC
- **24/24 manual tests passing**, all automated checks clean
- **All QA findings resolved** (5 issues fixed including AU wrapper)

**Test Results:**

```
Engine Tests: All passing (cargo test --workspace)
UI Tests:     35 passed, 0 failed (Vitest)
Linting:      All checks passed (Clippy, ESLint, Prettier, TypeScript)
Manual Tests: 24/24 passed
```

---

## Milestone 10: Declarative Plugin DSL ✅

**Status: ✅ Complete**

> **Goal:** Introduce macro-based DSL to dramatically simplify plugin creation — reduce boilerplate from ~190 lines to ~9 lines.

**Branch:** `feature/declarative-plugin-dsl`  
**Version:** `0.6.0` (minor — new public API, significant DX improvement)

**User Stories:** [docs/feature-specs/\_archive/declarative-plugin-dsl/user-stories.md](feature-specs/_archive/declarative-plugin-dsl/user-stories.md)  
**Low-Level Design:** [docs/feature-specs/\_archive/declarative-plugin-dsl/low-level-design-declarative-plugin-dsl.md](feature-specs/_archive/declarative-plugin-dsl/low-level-design-declarative-plugin-dsl.md)  
**Implementation Plan:** [docs/feature-specs/\_archive/declarative-plugin-dsl/implementation-plan.md](feature-specs/_archive/declarative-plugin-dsl/implementation-plan.md)

| Task                              | Status | Notes                                                    |
| --------------------------------- | ------ | -------------------------------------------------------- |
| **Phase 1: Core Traits**          | ✅     | ProcessorParams trait, Processor::Params associated type |
| **Phase 2: Derive Macro**         | ✅     | #[derive(ProcessorParams)] with #[param] attributes      |
| **Phase 3: Built-in Processors**  | ✅     | Gain, Passthrough (Filter/Compressor/Limiter deferred)   |
| **Phase 4: Chain Combinator**     | ✅     | Type-safe signal chain composition                       |
| **Phase 5: wavecraft_processor!** | ✅     | User-defined processor types                             |
| **Phase 6: wavecraft_plugin!**    | ✅     | Top-level plugin declaration macro                       |
| **Phase 7: Integration**          | ✅     | Template project updated with DSL                        |
| **Phase 8: Documentation**        | ✅     | Architecture docs, coding standards updated              |
| **Phase 9: UI Parameter Groups**  | ✅     | ParameterGroup component, useParameterGroups hook        |
| **Testing & QA**                  | ✅     | 63 tests (28 engine + 35 UI), manual DAW verification    |

**Key Deliverables:**

- **95% code reduction** — Plugin definition from 190 lines to 9 lines
- **`wavecraft_plugin!` macro** — Zero-boilerplate plugin declaration
- **`#[derive(ProcessorParams)]`** — Automatic parameter metadata with `#[param(...)]` attributes
- **`wavecraft_processor!` macro** — Named processor wrappers for signal chains
- **Built-in processors** — Gain, Passthrough with full parameter support
- **Chain combinator** — Type-safe `Chain!` macro for signal composition
- **UI parameter groups** — `ParameterGroup` component, `useParameterGroups` hook
- **DAW verified** — Plugin loads and works correctly in Ableton Live

**Test Results:**

```
Engine Tests: 28 passed, 0 failed
UI Tests:     35 passed, 0 failed
Manual Tests: 18/18 passed (DAW loading, parameter sync, UI rendering)
Linting:      All checks passed (cargo fmt, clippy, ESLint, Prettier)
```

**Rationale:** This significantly improves developer experience and is a differentiator for Wavecraft. Completing this before open-source release makes the SDK much more appealing to early adopters.

---

## Milestone 11: Code Quality & OSS Prep ✅

> **Goal:** Polish codebase for open-source release — proper logging, code quality fixes, CI optimization.

**Branch:** `feature/code-quality-polish`  
**Version:** `0.6.2` (patch — polish, CI optimization, no new features)

**User Stories:** [docs/feature-specs/\_archive/code-quality-polish/user-stories.md](feature-specs/_archive/code-quality-polish/user-stories.md)

| Task                             | Status | Notes                                                     |
| -------------------------------- | ------ | --------------------------------------------------------- |
| **Code Quality**                 |        |                                                           |
| Disable horizontal scroll wiggle | ✅     | CSS `overflow-x: hidden` on `#root`                       |
| Logger class for UI              | ✅     | `Logger` in `@wavecraft/ipc` with severity levels         |
| Log/tracing crate for Engine     | ✅     | `tracing` crate in standalone, 24 calls migrated          |
| **CI/CD Optimization**           |        |                                                           |
| `cargo xtask check` command      | ✅     | Fast local validation (~52s, 26x faster than Docker CI)   |
| Pre-compile test binaries        | ✅     | `cargo test --no-run` in prepare-engine job               |
| Tiered artifact retention        | ✅     | 7 days (main) / 90 days (tags), ~75-80% storage reduction |
| Agent documentation updates      | ✅     | Tester, QA, coder agent docs updated for new workflow     |
| **Open Source Prep**             |        |                                                           |
| LICENSE file                     | ✅     | MIT License added to root and template                    |
| Contributing guidelines          | ✅     | CONTRIBUTING.md with development workflow                 |
| Code of Conduct                  | ✅     | CODE_OF_CONDUCT.md (Contributor Covenant)                 |
| Issue templates                  | ✅     | Bug report and feature request templates                  |
| PR template                      | ✅     | Pull request template with checklist                      |
| README polish                    | ✅     | Status badges, updated structure, docs links              |
| Version bump                     | ✅     | `0.6.1` (Cargo.toml)                                      |

**Key Deliverables:**

- **UI Logger** — `Logger` class with `debug/info/warn/error` methods, exported from `@wavecraft/ipc`
- **Engine logging** — `tracing` crate replacing `println!` in standalone crate (24 calls migrated)
- **CI optimization** — `cargo xtask check` command for 26x faster local validation, pre-compiled test binaries, tiered artifact retention
- **Open source infrastructure** — LICENSE, CONTRIBUTING.md, CODE_OF_CONDUCT.md, issue/PR templates
- **Template synchronization** — Logger and CSS fixes propagated to `plugin-template/`
- **Documentation updates** — Logging standards added to coding-standards.md, IPC exports documented in high-level-design.md, agent workflows updated

**Test Results:**

```
Engine Tests: 110+ passed, 0 failed
UI Tests:     43 passed, 0 failed
Manual Tests: 19/19 passed
Linting:      All checks passed (cargo fmt, clippy, ESLint, Prettier)
QA:           5 findings (1 Critical, 4 Medium) — all resolved
```

---

## Milestone 12: Open Source Readiness ✅

> **Goal:** Prepare the repository for open source release — make the template truly independent, create a CLI for project scaffolding, and fix documentation for external developers.

**Depends on:** Milestone 11 (Code Quality & OSS Prep)

**Branch:** `feature/open-source-readiness`  
**Target Version:** `0.7.0` (minor — new CLI tool, significant public API changes)

**User Stories:** [docs/feature-specs/\_archive/open-source-readiness/user-stories.md](feature-specs/_archive/open-source-readiness/user-stories.md)

| Task                                                      | Status | Notes                                                         |
| --------------------------------------------------------- | ------ | ------------------------------------------------------------- |
| **Template Independence**                                 |        |                                                               |
| Replace path deps with git deps                           | ✅     | Uses `git = "https://github.com/RonHouben/wavecraft"`         |
| Version-locked dependencies                               | ✅     | Uses git tags (e.g., `tag = "v0.7.0"`)                        |
| Template builds standalone                                | ✅     | CI validates generated projects compile                       |
| Template variable system                                  | ✅     | `{{plugin_name}}`, `{{vendor}}`, etc.                         |
| **CLI Tool**                                              |        |                                                               |
| Create `cli/` crate                                       | ✅     | `wavecraft` CLI crate with `include_dir!` template            |
| `wavecraft create <name>` command                         | ✅     | Interactive project creation with prompts                     |
| Plugin name/vendor/email/URL prompts                      | ✅     | Optional fields handled gracefully                            |
| Template variable replacement                             | ✅     | heck crate for case conversions                               |
| Crate name validation                                     | ✅     | syn-based keyword validation (authoritative)                  |
| CLI unit tests                                            | ✅     | 7 tests passing                                               |
| **Documentation**                                         |        |                                                               |
| Fix broken links                                          | ✅     | Link checker script, 0 broken links                           |
| Update SDK Getting Started                                | ✅     | CLI workflow documented                                       |
| Update template README                                    | ✅     | Standalone instructions                                       |
| Add link checker to CI                                    | ✅     | check-docs job in ci.yml                                      |
| **CI for Template**                                       |        |                                                               |
| Template validation workflow                              | ✅     | template-validation.yml validates builds                      |
| `--local-dev` CLI flag                                    | ✅     | Path deps for CI (fixes tag chicken-egg problem)              |
| CLI release workflow                                      | ✅     | cli-release.yml for crates.io                                 |
| **UI Package Publishing**                                 |        |                                                               |
| Set up npm org `@wavecraft`                               | ✅     | npm organization registered                                   |
| Package `@wavecraft/core` for npm                         | ✅     | IPC bridge, hooks, utilities, Logger                          |
| Package `@wavecraft/components` for npm                   | ✅     | Meter, ParameterSlider, ParameterGroup, VersionBadge          |
| Export components (Meter, ParameterSlider, VersionBadge)  | ✅     | Public component API via @wavecraft/components                |
| Export hooks (useParameter, useMeterFrame)                | ✅     | React hooks via @wavecraft/core                               |
| Export IPC utilities (IpcBridge, ParameterClient, logger) | ✅     | Bridge to Rust engine via @wavecraft/core                     |
| Add npm package README                                    | ✅     | Usage examples, API documentation                             |
| Template uses npm package                                 | ✅     | Uses @wavecraft/core and @wavecraft/components                |
| Publish to npm registry                                   | ✅     | @wavecraft/core@0.7.0, @wavecraft/components@0.7.0            |
| **Release (Post-Merge)**                                  |        |                                                               |
| Version bump to 0.7.0                                     | ✅     | engine/Cargo.toml + cli/Cargo.toml (now 0.7.1)                |
| Create git tag `v0.7.0`                                   | ⏳     | After PR merge                                                |
| Publish CLI to crates.io                                  | ⏳     | Requires repo to be public                                    |
| End-to-end testing (external clone)                       | ⏳     | Requires repo to be public                                    |
| **Continuous Deployment**                                 |        |                                                               |
| `continuous-deploy.yml` workflow                          | ✅     | Auto-publish on merge to main                                 |
| Path-based change detection                               | ✅     | dorny/paths-filter for selective publishing                   |
| Auto-version bumping                                      | ✅     | Patch versions bumped automatically via `[auto-bump]` commits |
| CLI cascade trigger                                       | ✅     | CLI re-publishes when any SDK component changes               |
| `[auto-bump]` loop prevention                             | ✅     | Replaces `[skip ci]` — other workflows still run              |
| npm publish-only model                                    | ✅     | No build step — uses pre-built `dist/` in repo                |
| Upstream failure guards                                   | ✅     | `!cancelled()` prevents cascade on upstream failures          |
| npm release workflow                                      | ✅     | `npm-release.yml` (manual override)                           |
| CLI release workflow                                      | ✅     | `cli-release.yml` (manual override)                           |
| CI pipeline documentation                                 | ✅     | Full CD section in ci-pipeline.md                             |

**Key Deliverables:**

- **`wavecraft` CLI** — `cargo install wavecraft && wavecraft create my-plugin` project scaffolding
- **Independent template** — Builds without monorepo, uses git dependencies
- **Fixed documentation** — All links work, written for external users
- **Version-locked deps** — Stable builds with git tags
- **syn-based validation** — Authoritative Rust keyword checking (architectural best practice)
- **`@wavecraft/core` npm package** — IPC bridge, React hooks, Logger, utilities
- **`@wavecraft/components` npm package** — Meter, ParameterSlider, ParameterGroup, VersionBadge
- **Continuous Deployment** — Auto-publish to npm/crates.io on merge to main, CLI cascade trigger, `[auto-bump]` loop prevention

**Test Results:**

```
CLI Tests:    7 passed, 0 failed
Engine Tests: 95 passed, 0 failed
UI Tests:     51 passed, 0 failed (43 existing + 8 new package tests)
Manual Tests: 20/20 passed (100%)
Linting:      All checks passed (cargo fmt, clippy, ESLint, Prettier)
QA:           PASS (0 Critical/High, 2 Medium non-blocking, 3 Low optional)
```

**Bug Fixes Applied:**

- Empty URL/email template variables now handled gracefully
- Reserved keyword validation uses syn crate (future-proof)

**Success Criteria:**

- [x] External developer can: `cargo install wavecraft && wavecraft create my-plugin && cd my-plugin && cargo xtask bundle`
- [x] Template builds in < 5 minutes (first time, with downloads)
- [x] Zero broken documentation links
- [x] `@wavecraft/core` published to npm (v0.7.1)
- [x] `@wavecraft/components` published to npm (v0.7.1)
- [x] Template uses npm packages instead of bundled UI copy
- [x] Continuous deployment workflow for automatic publishing
- [ ] CLI published to crates.io (requires public repo)

**Completed:** 2026-02-04

---

## Milestone 14: CLI Enhancements ✅

> **Goal:** Improve CLI developer experience with version checking and dependency management. Small quality-of-life improvements before user testing.

**Status: ✅ Complete**

**Branch:** `feature/cli-version-and-update`  
**Target Version:** `0.8.5` (patch — CLI improvements, no breaking changes)

**User Stories:** [docs/feature-specs/\_archive/cli-version-and-update/user-stories.md](feature-specs/_archive/cli-version-and-update/user-stories.md)  
**Low-Level Design:** [docs/feature-specs/\_archive/cli-version-and-update/low-level-design.md](feature-specs/_archive/cli-version-and-update/low-level-design.md)  
**Implementation Plan:** [docs/feature-specs/\_archive/cli-version-and-update/implementation-plan.md](feature-specs/_archive/cli-version-and-update/implementation-plan.md)  
**Architectural Review:** [docs/feature-specs/\_archive/cli-version-and-update/architectural-review.md](feature-specs/_archive/cli-version-and-update/architectural-review.md)

| Task                                     | Status | Notes                                                     |
| ---------------------------------------- | ------ | --------------------------------------------------------- |
| **Version Flag**                         |        |                                                           |
| Add `-V` flag (short form)               | ✅     | Follows Rust CLI conventions (capital V)                  |
| Add `--version` flag (long form)         | ✅     | clap built-in support                                     |
| Display format: `wavecraft 0.x.y`        | ✅     | Clean, standard output from CARGO_PKG_VERSION             |
| Update CLI help text                     | ✅     | Automatic via clap                                        |
| **Update Command**                       |        |                                                           |
| Add `wavecraft update` subcommand        | ✅     | `cli/src/commands/update.rs` (137 lines)                  |
| Update Rust crates (Cargo.toml)          | ✅     | Runs `cargo update` in engine/                            |
| Update npm packages (package.json)       | ✅     | Runs `npm update` in ui/                                  |
| Detect workspace structure               | ✅     | File-based detection (engine/Cargo.toml, ui/package.json) |
| Error handling for missing dirs          | ✅     | Graceful failures with context                            |
| **CLI Self-Update (v0.9.1 Enhancement)** |        |                                                           |
| Self-update CLI via `cargo install`      | ✅     | Runs before project deps, non-fatal on failure            |
| Work from any directory                  | ✅     | No longer errors outside projects                         |
| Version change notification              | ✅     | Shows old→new version, re-run hint                        |
| Graceful error handling                  | ✅     | Self-update failure doesn't block project deps            |
| Output parsing + version detection       | ✅     | `is_already_up_to_date()`, `parse_version_output()`       |
| 19 unit + integration tests              | ✅     | +12 tests from QA findings (was 7)                        |
| **Testing**                              |        |                                                           |
| CLI unit tests for version flag          | ✅     | 4 integration tests (TC-014)                              |
| Integration tests for update command     | ✅     | 5 integration tests (TC-015)                              |
| Manual testing in plugin project         | ✅     | 18/22 test cases passing                                  |
| Documentation updates                    | ✅     | high-level-design.md, sdk-getting-started.md              |
| **Quality Assurance**                    |        |                                                           |
| QA review                                | ✅     | QA-report.md: 0 Critical/High issues                      |
| Architectural review                     | ✅     | architectural-review.md: ⭐⭐⭐⭐⭐ (5/5)                 |

**Key Deliverables:**

- **Version flags**: `-V` and `--version` using clap's built-in support
- **Update command**: `wavecraft update` for Rust + npm dependency updates
- **Error accumulation pattern**: Continues updating even if one component fails
- **File-based detection**: O(1) filesystem checks for engine/ and ui/
- **Integration tests**: 9 tests (4 version + 5 update) all passing
- **Documentation**: Architecture docs updated (high-level-design, sdk-getting-started)

**Test Results:**

```
CLI Integration Tests: 9 passed (version_flag.rs: 4, update_command.rs: 5)
Engine Tests:          All passing
UI Tests:              All passing
Manual Tests:          18/22 passed (4 E2E tests blocked, not in scope)
Linting:               All checks passed (cargo fmt, clippy, ESLint, Prettier)
QA:                    APPROVED (0 Critical/High, 2 Medium non-blocking)
```

**Success Criteria:**

- [x] `wavecraft -V` and `wavecraft --version` display correct version
- [x] `wavecraft update` successfully updates both Rust and npm dependencies
- [x] Error messages are clear and actionable
- [x] Documentation reflects new commands
- [x] QA approval received
- [x] Architectural review approved (5/5 rating)

**CLI Self-Update Enhancement (v0.9.1, completed 2026-02-08):**

- `wavecraft update` now self-updates the CLI first via `cargo install wavecraft`
- Works from any directory (no longer errors outside plugin projects)
- Non-fatal: self-update failure doesn't block project dependency updates
- Clear version change notification with re-run hint
- 19 tests total (7 original + 12 added from QA findings)
- QA approved (PASS — 0 Critical/High/Medium after fixes)
- Architecture docs updated (high-level-design.md, sdk-getting-started.md)

**Completed:** 2026-02-08

---

## Milestone 15: Developer Tooling Polish ✅

> **Goal:** Small quality-of-life improvements to developer tooling before user testing. Extend `cargo xtask clean` to comprehensively clean the entire workspace.

**Status: ✅ Complete**

**Branch:** `feature/workspace-cleanup`  
**Target Version:** `0.8.6` (patch — tooling improvement, no breaking changes)

**User Stories:** [docs/feature-specs/\_archive/workspace-cleanup/user-stories.md](feature-specs/_archive/workspace-cleanup/user-stories.md)  
**Implementation Progress:** [docs/feature-specs/\_archive/workspace-cleanup/implementation-progress.md](feature-specs/_archive/workspace-cleanup/implementation-progress.md)  
**Test Plan:** [docs/feature-specs/\_archive/workspace-cleanup/test-plan.md](feature-specs/_archive/workspace-cleanup/test-plan.md)  
**QA Report:** [docs/feature-specs/\_archive/workspace-cleanup/QA-report.md](feature-specs/_archive/workspace-cleanup/QA-report.md)  
**Architectural Review:** [docs/feature-specs/\_archive/workspace-cleanup/architectural-review.md](feature-specs/_archive/workspace-cleanup/architectural-review.md)

| Task                                             | Status | Notes                                            |
| ------------------------------------------------ | ------ | ------------------------------------------------ |
| **Workspace Cleanup**                            |        |                                                  |
| Extend `cargo xtask clean` to clean `cli/target` | ✅     | Implemented with `cargo clean`                   |
| Clean `ui/dist/` (Vite build outputs)            | ✅     | `fs::remove_dir_all` with size tracking          |
| Clean `ui/coverage/` (test artifacts)            | ✅     | Idempotent removal                               |
| Clean `target/tmp/` (test plugins)               | ✅     | Recursive cleanup                                |
| Clean `bundled/` (VST3/CLAP bundles)             | ✅     | Bundle cleanup added                             |
| Clean AU wrapper build directory                 | ✅     | macOS-specific cleanup                           |
| Add clear output with disk space reporting       | ✅     | Shows size per directory + total                 |
| Replace Python scripts with `xtask`              | ✅     | Tooling consolidation, Python dependency removed |
| **Testing**                                      |        |                                                  |
| Unit tests for cleanup function                  | ✅     | 8 tests (dir_size, format_size, remove_dir)      |
| Manual testing                                   | ✅     | 12/12 test cases passed (100%)                   |
| Documentation updates                            | ✅     | high-level-design.md updated                     |
| **Quality Assurance**                            |        |                                                  |
| QA review                                        | ✅     | 0 issues found, approved                         |
| Architectural review                             | ✅     | Fully compliant, approved                        |

**Key Deliverables:**

- **Comprehensive clean command** — Cleans 7 directories: `engine/target`, `cli/target`, `ui/dist`, `ui/coverage`, `target/tmp`, `bundled/`, AU wrapper build
- **Helper functions** — `dir_size()`, `format_size()`, `remove_dir()` for size calculation and human-readable output
- **Clear output** — Shows ✓ checkmarks, directory names, sizes, and total space reclaimed
- **Idempotent** — Handles missing directories gracefully (no errors)
- **Well-tested** — 8 unit tests + 12 manual test cases (all passing)
- **Documentation** — Architecture docs, test plan, QA report, architectural review

**Test Results:**

```
Engine Tests: 106 passed (8 new clean.rs tests)
UI Tests:     51 passed
Manual Tests: 12/12 passed (100%)
Linting:      All checks passed (cargo fmt, clippy, ESLint, Prettier)
QA:           APPROVED (0 Critical/High/Medium/Low issues)
Architecture: APPROVED (fully compliant with all conventions)
```

**Success Criteria:**

- [x] Single command cleans all Rust + UI + temp build artifacts
- [x] No errors on missing directories (idempotent)
- [x] Clear output showing what was cleaned with disk space reclaimed
- [x] Developer experience: "this just works" (validated via manual testing)

**Completed:** 2026-02-08

---

## Milestone 13: User Testing ⏳

> **Goal:** Comprehensive internal validation of the complete SDK workflow before external beta testing. Catch issues that would frustrate external testers.

**Status: ✅ Complete**

**Branch:** `feature/cli-ux-improvements`  
**Target Version:** `0.8.0` (minor — CLI improvements, zero prompts, better UX)

**User Stories:** [docs/feature-specs/\_archive/cli-ux-improvements/user-stories.md](feature-specs/_archive/cli-ux-improvements/user-stories.md)

| Task                             | Status | Notes                                                           |
| -------------------------------- | ------ | --------------------------------------------------------------- |
| **SDK Workflow Validation**      |        |                                                                 |
| Fresh clone & setup              | ✅     | Template validated with CLI                                     |
| Build plugin from template       | ✅     | `wavecraft create` → `cargo xtask bundle` succeeds              |
| Load in Ableton Live             | ✅     | Plugin loads, UI renders, no crashes                            |
| Parameter sync test              | ✅     | UI ↔ DAW automation works correctly                             |
| State persistence test           | ✅     | Save/load project preserves plugin state                        |
| Multi-instance test              | ✅     | Multiple plugin instances work correctly                        |
| **crates.io Publishing Prep**    |        |                                                                 |
| Crate metadata validation        | ✅     | All 6 publishable crates have required fields                   |
| Version specifiers added         | ✅     | `version = "0.7.1"` on all workspace deps                       |
| **wavecraft-core crate split**   | ✅     | Enables crates.io publishing (nih_plug blocker resolved)        |
| Dry-run publish verification     | ✅     | protocol, metering, macros pass `cargo publish --dry-run`       |
| **Documentation Walkthrough**    |        |                                                                 |
| SDK Getting Started guide        | ✅     | Updated with zero-prompt workflow, PATH guidance                |
| High-level design review         | ✅     | Architecture docs updated for CLI behavior                      |
| Coding standards review          | ✅     | Module organization updated                                     |
| CI pipeline guide review         | ✅     | Local testing instructions work                                 |
| **Regression Testing**           |        |                                                                 |
| All `cargo xtask check` passes   | ✅     | Lint + tests clean (all tests pass)                             |
| Visual testing with Playwright   | ✅     | UI renders correctly in browser                                 |
| Desktop app (`cargo xtask dev`)  | ✅     | WebSocket bridge works                                          |
| Signing workflow                 | ✅     | Ad-hoc signing succeeds                                         |
| **Template Project Validation**  |        |                                                                 |
| Template builds standalone       | ✅     | No monorepo dependencies leak                                   |
| Template xtask commands work     | ✅     | bundle, dev, install                                            |
| Template README accurate         | ✅     | Instructions match reality                                      |
| **Edge Cases & Stress Testing**  |        |                                                                 |
| Low buffer sizes (32/64 samples) | ✅     | No audio glitches                                               |
| Rapid parameter changes          | ✅     | No UI lag or crashes                                            |
| DAW project with many tracks     | ✅     | Performance acceptable                                          |
| **CLI UX Improvements**          |        |                                                                 |
| Help command documentation       | ✅     | `--help` works via clap                                         |
| Remove personal data prompts     | ✅     | Zero prompts, uses placeholder defaults                         |
| Clean CLI interface              | ✅     | Removed `--sdk-version`, renamed `--local-dev` to `--local-sdk` |
| PATH troubleshooting guidance    | ✅     | Documentation added                                             |

**Crate Split Details (Completed 2026-02-06):**

The wavecraft-core crate was split to enable crates.io publishing:

| Crate                | Purpose                              | Publishable                     |
| -------------------- | ------------------------------------ | ------------------------------- |
| `wavecraft-nih_plug` | nih-plug integration, WebView editor | ❌ Git-only (`publish = false`) |
| `wavecraft-core`     | Core SDK types, declarative macros   | ✅ crates.io (no nih_plug dep)  |

**Key changes:**

- `__nih` module in wavecraft-nih_plug exports all nih_plug types for proc-macro
- `wavecraft_plugin!` macro supports `crate:` field for path customization
- Template uses Cargo package rename: `wavecraft = { package = "wavecraft-nih_plug", ... }`
- All 6 publishable crates validated with dry-run publish

**CLI UX Improvements (Completed 2026-02-06):**

Based on internal testing, the CLI was improved for better developer experience:

| Improvement                        | Implementation                                                                                                             | Impact                                                                  |
| ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| **Zero prompts**                   | Removed `dialoguer` dependency, uses placeholder defaults                                                                  | Faster onboarding                                                       |
| **SDK version auto-determination** | Uses `env!("CARGO_PKG_VERSION")` from CLI                                                                                  | No manual version input                                                 |
| **Git tag format**                 | `wavecraft-cli-v{version}` (matches repo convention)                                                                       | Consistent release tagging                                              |
| **Clean interface**                | `--local-sdk` boolean flag (hidden), no `--sdk-version`                                                                    | Less confusing help output                                              |
| **PATH troubleshooting**           | Clear documentation in Getting Started guide                                                                               | Better error handling                                                   |
| **Embedded dev server**            | `wavecraft start` builds plugin, loads params via FFI, starts WS + Vite; preflight port checks with strict UI port binding | Enables browser dev from plugin projects; fail-fast if ports are in use |

**Test Results (M13 Complete):**

```
CLI Tests:    All passing
Engine Tests: All passing
UI Tests:     All passing
Manual Tests: 10/10 passed (user workflow validation)
Linting:      All checks passed (cargo fmt, clippy)
QA:           PASS (0 Critical/High/Medium/Low issues)
```

**Success Criteria:**

- [x] Complete SDK workflow works end-to-end
- [x] All documentation is accurate and followable
- [x] No regressions from previous milestones
- [x] Template project works independently
- [x] No critical bugs discovered
- [x] CLI UX polished for external users

---

## Milestone 16: Macro API Simplification ✅

> **Goal:** Simplify the `wavecraft_plugin!` macro API to reduce boilerplate and improve developer experience. Remove unnecessary properties, enforce consistent signal chain syntax, and derive metadata from `Cargo.toml`.

**Status: ✅ Complete** — All phases implemented, CI validated, tested and approved

**Branch:** `feature/macro-api-simplification` (merged)  
**Version:** `0.9.0` (minor — breaking API change, simplified interface)  
**Completed:** 2026-02-08

**Documentation:** [docs/feature-specs/\_archive/macro-api-simplification/](feature-specs/_archive/macro-api-simplification/)

### Motivation

The current `wavecraft_plugin!` macro requires too many properties that add boilerplate without value:

- `vendor`, `url`, `email` — Should be derived from `Cargo.toml`
- `crate` — Internal implementation detail, shouldn't be user-facing
- `signal` — Accepts both bare `Processor` and `SignalChain![]`, causing API confusion

### Proposed Changes

| Change                  | Current API                          | New API                      | Impact         |
| ----------------------- | ------------------------------------ | ---------------------------- | -------------- |
| **Required properties** | 5 (name, vendor, url, email, signal) | 2 (name, signal)             | 60% reduction  |
| **Metadata source**     | Manual in macro                      | Auto-derived from Cargo.toml | No duplication |
| **Signal syntax**       | `InputGain` or `Chain![InputGain]`   | `SignalChain![InputGain]`    | Consistent     |
| **`crate` property**    | Required                             | Optional (hidden)            | Simplified     |

**Before (9 lines):**

```rust
wavecraft_plugin! {
    name: "My Plugin",
    vendor: "Wavecraft",
    url: "https://example.com",
    email: "info@example.com",
    signal: InputGain,  // or Chain![InputGain]
}
```

**After (4 lines):**

```rust
wavecraft_plugin! {
    name: "My Plugin",
    signal: SignalChain![InputGain],
}
```

### Technical Approach

| Area                    | Solution                                                                      |
| ----------------------- | ----------------------------------------------------------------------------- |
| **Metadata derivation** | Use Cargo's compile-time env vars (`CARGO_PKG_AUTHORS`, `CARGO_PKG_HOMEPAGE`) |
| **VST3 Class ID**       | Hash `CARGO_PKG_NAME + name` instead of `vendor + name` (**breaking change**) |
| **Signal validation**   | Compile-time check that `signal` is a `SignalChain![]`                        |
| **Macro rename**        | `Chain!` → `SignalChain!` with deprecation path                               |
| **`crate` property**    | Optional field with default `::wavecraft` (not removed for edge cases)        |

### Tasks

| Task                                  | Status | Notes                                      |
| ------------------------------------- | ------ | ------------------------------------------ |
| **Phase 1: Core Macro Changes**       | ✅     |                                            |
| Simplify `PluginDef` struct           | ✅     | Removed vendor, url, email fields          |
| Derive metadata from Cargo env vars   | ✅     | `CARGO_PKG_AUTHORS`, `CARGO_PKG_HOMEPAGE`  |
| Update VST3/CLAP ID generation        | ✅     | Uses package name instead of vendor        |
| Add signal validation                 | ✅     | Compile-time error for bare identifiers    |
| **Phase 2: SignalChain Rename**       | ✅     |                                            |
| Create `SignalChain!` macro           | ✅     | New macro with clear naming                |
| Deprecate `Chain!`                    | ✅     | Backward compatible with warnings          |
| Update prelude exports                | ✅     | `SignalChain!` available via prelude       |
| **Phase 3: CLI Template Updates**     | ✅     |                                            |
| Update template to use new API        | ✅     | 4-line minimal macro call                  |
| Add metadata to template Cargo.toml   | ✅     | authors, homepage fields added             |
| **Phase 4: Documentation & QA Fixes** | ✅     |                                            |
| Update architecture docs              | ✅     | high-level-design.md, coding-standards.md  |
| Add parameter sync limitation docs    | ✅     | Known limitations section added            |
| Enhance safety documentation          | ✅     | Comprehensive SAFETY comments              |
| **Phase 5: Testing**                  | ✅     |                                            |
| CI validation                         | ✅     | 107 tests passing (engine + UI + doctests) |
| Manual testing                        | ✅     | 10/10 functional tests passing             |
| QA review                             | ✅     | All findings addressed                     |

**Key Deliverables:**

- **95% simpler API** — 9 lines → 4 lines (55% reduction)
- **Single source of truth** — Metadata from `Cargo.toml` only
- **Consistent signal syntax** — `SignalChain![]` always
- **Compile-time validation** — Clear error messages for API misuse
- **Migration guide** — MIGRATION-0.9.md with upgrade instructions

**Breaking Changes:**

- VST3 Class IDs will change (now based on package name, not vendor)
- `Chain!` deprecated (removed in 0.10.0)
- Bare processor syntax removed (must use `SignalChain![]`)

**Success Criteria:**

- [x] Template uses new minimal API (4 lines)
- [x] All tests pass with new macro
- [x] VST3/CLAP export still works correctly
- [x] Clear compile-time errors for misuse
- [x] Documentation updated with migration guide

**Completed:** 2026-02-08

---

## Milestone 17: OS Audio Input for Dev Mode ✅

**Status: ✅ Complete**

**Branch:** `feature/dev-audio-os-input`  
**Target Version:** `0.8.0` (minor — new CLI development feature)

**User Stories:** [docs/feature-specs/\_archive/dev-audio-os-input/user-stories.md](feature-specs/_archive/dev-audio-os-input/user-stories.md)

> **Goal:** Enable OS microphone input during development mode for testing audio processing without plugin host. Simplifies development workflow by providing real audio input via `wavecraft start`.

| Task                                      | Status | Notes                                          |
| ----------------------------------------- | ------ | ---------------------------------------------- |
| **Protocol Extensions**                   |        |                                                |
| Add `registerAudio` method                | ✅     | JSON-RPC method for audio service registration |
| Add `meterUpdate` notification            | ✅     | Binary WebSocket message for meter data        |
| **Audio Server**                          |        |                                                |
| Integrate cpal for OS audio input         | ✅     | Cross-platform audio I/O library               |
| Implement real-time audio processing loop | ✅     | Microphone → Processor → meters                |
| WebSocket binary communication            | ✅     | MessagePack for meter updates                  |
| **CLI Integration**                       |        |                                                |
| Auto-detect audio-dev binary              | ✅     | Checks for dev-audio.rs in plugin projects     |
| Compile and start audio binary            | ✅     | `cargo build` + process spawning               |
| Graceful fallback when missing            | ✅     | Helpful messages, continues without audio      |
| **SDK Templates**                         |        |                                                |
| Optional audio-dev binary in templates    | ✅     | dev-audio.rs with feature flags                |
| Template README documentation             | ✅     | Usage instructions for audio development       |
| **Testing**                               |        |                                                |
| End-to-end testing                        | ✅     | WebSocket client verified meter updates        |
| Protocol serialization tests              | ✅     | registerAudio and meterUpdate                  |
| Template compilation tests                | ✅     | All template projects compile                  |
| Manual testing                            | ✅     | Full flow validated                            |

**Key Deliverables:**

- **Always-on design** — Zero configuration, automatic detection
- **CLI integration** — `wavecraft start` detects, compiles, and starts audio binary
- **Real-time safe** — No tokio panics from audio thread
- **WebSocket protocol** — Binary meter updates via MessagePack
- **Template support** — Optional audio-dev binary in SDK templates
- **10 commits** on feature branch with comprehensive testing

**Test Results:**

```
Protocol Tests: All passing (registerAudio, meterUpdate serialization)
Template Tests: All projects compile successfully
E2E Tests:      WebSocket client received meter updates with real audio
Manual Tests:   Full flow validated (microphone → processor → UI)
```

**Success Criteria:**

- [x] `wavecraft start` automatically detects and compiles audio binary
- [x] Audio flows: microphone → user's Processor → meters → WebSocket → UI
- [x] Meter updates verified with real audio values (RMS/peak)
- [x] Graceful fallback when audio binary missing
- [x] Zero configuration required (always-on design)
- [x] Real-time safe (no tokio panics from audio thread)
- [x] All template projects compile successfully

**Completed:** 2026-02-08

---

## Milestone 18: Audio Pipeline Fixes & Mocking Cleanup ✅

> **Goal:** Fix critical audio gaps in dev mode (`wavecraft start`) — add audio output, bridge parameter changes to DSP, and remove unused synthetic metering. Ensures beta testers get a working audio development experience.

**Status: ✅ Complete**

**Branch:** `feature/audio-pipeline-fixes`
**Version:** `0.10.0` (minor — significant audio pipeline changes)

**User Stories:** [docs/feature-specs/\_archive/audio-pipeline-fixes/user-stories.md](feature-specs/_archive/audio-pipeline-fixes/user-stories.md)
**Low-Level Design:** [docs/feature-specs/\_archive/audio-pipeline-fixes/low-level-design-audio-pipeline-fixes.md](feature-specs/_archive/audio-pipeline-fixes/low-level-design-audio-pipeline-fixes.md)
**Implementation Plan:** [docs/feature-specs/\_archive/audio-pipeline-fixes/implementation-plan.md](feature-specs/_archive/audio-pipeline-fixes/implementation-plan.md)
**QA Report:** [docs/feature-specs/\_archive/audio-pipeline-fixes/QA-report.md](feature-specs/_archive/audio-pipeline-fixes/QA-report.md)
**Architectural Review:** [docs/feature-specs/\_archive/audio-pipeline-fixes/architectural-review.md](feature-specs/_archive/audio-pipeline-fixes/architectural-review.md)

| Task                                                 | Status | Notes                                                   |
| ---------------------------------------------------- | ------ | ------------------------------------------------------- |
| **Audio Output**                                     |        |                                                         |
| Add output stream to `AudioServer`                   | ✅     | cpal `build_output_stream()` for effects path           |
| Audio flow: input → process → output                 | ✅     | Full duplex via rtrb SPSC ring buffer                   |
| Meters from processed output                         | ✅     | Computed post-DSP via rtrb ring buffer                  |
| **Parameter Sync (Dev Mode)**                        |        |                                                         |
| Lock-free param bridge (WS → audio thread)           | ✅     | `AtomicParameterBridge` with `Arc<AtomicF32>` per param |
| `setParameter` reaches `FfiProcessor::process()`     | ✅     | Block-level updates via `Relaxed` atomics               |
| **Mocking Cleanup**                                  |        |                                                         |
| Remove `MeterGenerator` from `wavecraft-metering`    | ✅     | `dev.rs` module deleted                                 |
| Update fallback behavior (zeros, not fake animation) | ✅     | Silent meters when no vtable                            |
| **UI Fix**                                           |        |                                                         |
| `useAllParameters()` retry on connection             | ✅     | Event-driven re-fetch on WS connect                     |
| **Testing & Documentation**                          |        |                                                         |
| Manual testing with real audio                       | ✅     | Gain plugin: slider changes audio output                |
| Architecture docs updated                            | ✅     | high-level-design.md, coding-standards.md               |

**Key Deliverables:**

- **Full-duplex AudioServer** — Separate cpal input/output streams connected by rtrb SPSC ring buffer
- **AtomicParameterBridge** — Lock-free `HashMap<String, Arc<AtomicF32>>` for WS→audio thread parameter sync
- **RT-safe meter delivery** — rtrb ring buffer replacing tokio mpsc channel (zero allocations on audio thread)
- **MeterGenerator removed** — Synthetic metering infrastructure deleted; fallback = silent zeros
- **UI reconnection retry** — `useAllParameters` re-fetches on WebSocket connection
- **Pre-allocated audio buffers** — FfiProcessor uses stack arrays instead of heap Vec

**Test Results:**

```
Engine Tests: 146 passed (including 18 audio-feature tests)
UI Tests:     28 passed
CLI Tests:    57 passed
Linting:      All checks passed (cargo fmt, clippy, ESLint, Prettier)
QA:           PASS (3 Medium findings fixed, 0 Critical/High)
Architecture: APPROVED (full compliance with coding standards)
```

**Success Criteria:**

- [x] `wavecraft start` produces audible processed output
- [x] Moving a gain slider in browser UI audibly changes output level
- [x] No synthetic/fake meter data in production code
- [x] Parameters load correctly even on slow WebSocket connection
- [x] All existing tests still pass

**Completed:** 2026-02-08

---

## Milestone 18.5: Template Structure Improvement (Processors Module) ✅

> **Goal:** Improve CLI template structure with `processors/` module and complete oscillator example. Teaches proper code organization from day one while providing engaging learning experience with real audio generation.

**Status: ✅ Complete**

**Depends on:** Milestone 18 (Audio Pipeline Fixes) ✅

**Branch:** `feature/template-processors-module` (merged)
**Version:** `0.11.0` (minor — breaking template change)
**Completed:** 2026-02-09

**User Stories:** [docs/feature-specs/\_archive/template-processors-module/user-stories.md](feature-specs/_archive/template-processors-module/user-stories.md)

| Task                                          | Status | Notes                            |
| --------------------------------------------- | ------ | -------------------------------- |
| **Template Structure**                        |        |                                  |
| Create `processors/` module in template       | ✅     | New directory structure          |
| Add `processors/mod.rs` exports               | ✅     | Clean module pattern             |
| Add `processors/oscillator.rs` implementation | ✅     | Complete Processor trait example |
| Update `lib.rs` to use modular structure      | ✅     | Import from processors module    |
| **Oscillator Implementation**                 |        |                                  |
| Parameter struct with ProcessorParams derive  | ✅     | frequency: 20-20kHz, level: 0-1  |
| Oscillator processor with phase state         | ✅     | Sine wave generation             |
| Implement `set_sample_rate()`                 | ✅     | Proper initialization            |
| Implement `reset()`                           | ✅     | Clear phase state                |
| Add comprehensive code comments               | ✅     | Explain DSP concepts             |
| **Signal Chain Configuration**                |        |                                  |
| Default to gain-only chain (silent)           | ✅     | No unexpected sound              |
| Add oscillator as commented-out example       | ✅     | Easy to enable for testing       |
| Clear comments explaining options             | ✅     | User understands choices         |
| **Documentation**                             |        |                                  |
| Update SDK Getting Started guide              | ✅     | New template structure           |
| Update High-Level Design docs                 | ✅     | Template structure diagram       |
| Update Coding Standards                       | ✅     | Processor organization patterns  |
| Update template README                        | ✅     | How to add processors            |
| **Testing**                                   |        |                                  |
| Template generation tests                     | ✅     | `wavecraft create` succeeds      |
| Template compilation tests                    | ✅     | Plugin builds without errors     |
| Manual testing with oscillator                | ✅     | Generates audible sine wave      |
| DAW loading validation                        | ✅     | Works in Ableton Live            |

**Key Deliverables:**

- **Modular structure** — `processors/` folder for scalable organization
- **Complete example** — Oscillator with full Processor trait implementation (~60 lines)
- **Educational value** — Teaches DSP concepts (phase accumulation, sample rate)
- **Safe defaults** — Gain-only chain by default, oscillator opt-in via commented-out line
- **Clear documentation** — How to add processors, understand the pattern
- **23 files changed** — 873 insertions, 192 deletions

**Test Results:**

```
Engine Tests: All passing
UI Tests:     All passing
CLI Tests:    All passing
Manual Tests: 12/12 passed (Tester)
Linting:      All checks passed (cargo fmt, clippy, ESLint, Prettier)
QA:           PASS (0 Critical/High/Medium/Low issues)
```

**Success Criteria:**

- [x] Template has clear `processors/` module pattern
- [x] Oscillator generates audible sine wave (440Hz default)
- [x] Documentation explains where to add new processors
- [x] Pattern scales naturally (adding `processors/filter.rs` is obvious)
- [x] Template compiles and loads in DAW
- [x] Zero breaking changes to existing plugins

---

## Milestone 18.6: Documentation Architecture Split ✅

> **Goal:** Split large architecture documents (~1,500 lines each) into focused, topic-specific files to improve navigation for both human developers and AI agents, reducing token consumption by 80-90%.

**Status: ✅ Complete**

**Depends on:** None — pure documentation refactoring

**Branch:** `feature/docs-split-architecture`  
**Target Version:** `0.10.1` (patch — documentation-only change)

**User Stories:** [docs/feature-specs/\_archive/docs-split-architecture/user-stories.md](feature-specs/_archive/docs-split-architecture/user-stories.md)

| Task                                                   | Status | Notes                                     |
| ------------------------------------------------------ | ------ | ----------------------------------------- |
| **Phase 1: Coding Standards Split**                    |        |                                           |
| Create `coding-standards-typescript.md` (~400 lines)   | ✅     | TypeScript, React, hooks, build constants |
| Create `coding-standards-css.md` (~150 lines)          | ✅     | TailwindCSS, theming, WebView styling     |
| Create `coding-standards-rust.md` (~600 lines)         | ✅     | Module org, DSL, xtask, FFI safety        |
| Create `coding-standards-testing.md` (~300 lines)      | ✅     | Testing, logging, error handling          |
| Update `coding-standards.md` to overview (~200 lines)  | ✅     | Navigation hub with quick reference       |
| **Phase 2: High-Level Design Split**                   |        |                                           |
| Create `sdk-architecture.md` (~500 lines)              | ✅     | SDK distribution, crates, npm packages    |
| Create `declarative-plugin-dsl.md` (~300 lines)        | ✅     | DSL architecture, macros, discovery       |
| Create `development-workflows.md` (~400 lines)         | ✅     | Browser dev, FFI audio, build system      |
| Create `plugin-formats.md` (~300 lines)                | ✅     | VST3, CLAP, AU specifics                  |
| Create `versioning-and-distribution.md` (~200 lines)   | ✅     | Version flow, packaging, signing          |
| Update `high-level-design.md` to overview (~400 lines) | ✅     | Architecture diagram + navigation         |
| **Phase 3: Cross-Reference Updates**                   |        |                                           |
| Update `.github/copilot-instructions.md`               | ✅     | Reference new document structure          |
| Update `.github/skills/**/SKILL.md`                    | ✅     | Update architecture doc links             |
| Update `docs/guides/*.md`                              | ✅     | Update all cross-references               |
| Update `README.md`                                     | ✅     | Documentation links section               |
| Update archived feature specs if needed                | ✅     | Low priority (historical docs)            |
| **Phase 4: Validation**                                |        |                                           |
| Run `scripts/check-links.sh`                           | ✅     | Zero broken links                         |
| Grep for old references                                | ✅     | `rg 'coding-standards\.md'` etc           |
| Test with AI agents                                    | ✅     | Verify token reduction                    |
| Manual navigation testing                              | ✅     | Hub documents are clear                   |
| Add documentation structure guide                      | ✅     | `CONTRIBUTING.md` section                 |

**Key Deliverables:**

- **9 new focused documents** (150-600 lines each)
- **2 updated overview documents** (navigation hubs)
- **Zero broken links** (validated by check-links.sh)
- **80-90% token reduction** for focused documentation reads
- **Clear navigation structure** (Related Documents sections)

**Success Metrics:**

| Metric                     | Baseline    | Target     |
| -------------------------- | ----------- | ---------- |
| Largest doc size           | 1,562 lines | <600 lines |
| Token usage (typical read) | 3,000-6,000 | 200-600    |
| Broken links               | N/A         | 0          |
| Time to find info          | ~2-3 min    | <30 sec    |
| Docs in architecture/      | 3           | 14         |

**Estimated Effort:** 4-6 hours (careful extraction, link updates, validation)

**Rationale:**

- **Developer friction:** Scrolling through 1,500+ line docs is slow and overwhelming
- **AI token waste:** Agents load 3,000-6,000 tokens even for narrow queries
- **Maintenance difficulty:** Contributors struggle to find and update specific topics
- **Scalability:** Documentation will only grow; needs proper structure now

**Benefits:**

- Developers find information in <30 seconds (vs ~2-3 minutes scrolling)
- AI agents consume 80-90% fewer tokens per documentation read
- Contributors can quickly locate and update specific topics
- Documentation scales naturally as project grows

---

## Milestone 18.7: Fix UI Race Condition on Parameter Load ✅

> **Goal:** Fix silent failure when `useAllParameters()` hook mounts before WebSocket connection is established in browser dev mode.

**Depends on:** Milestone 18.6 (Documentation Split) — latest SDK architecture

**Status:** ✅ Complete — February 10, 2026

**Target Version:** `0.11.1` (patch — bug fix)

**Scope:** UI-only change in `@wavecraft/core` package. No Rust engine changes required.

| Task                                      | Status | Notes                                              |
| ----------------------------------------- | ------ | -------------------------------------------------- |
| **Requirements & Design**                 |        |                                                    |
| User stories                              | ✅     | 3 user stories with edge cases defined             |
| Low-level design                          | ✅     | Connection-aware retry mechanism with timeout      |
| Implementation plan                       | ✅     | 5-phase plan with 12 implementation steps          |
| **Implementation**                        |        |                                                    |
| Review existing connection awareness      | ✅     | `useConnectionStatus()` integration analyzed       |
| Implement retry mechanism                 | ✅     | Auto-retry on WebSocket connection                 |
| Add connection state synchronization      | ✅     | Hook waits for ready state with 15s timeout        |
| Handle edge cases                         | ✅     | Component unmount, rapid reconnects, max 3 retries |
| Improve error messaging                   | ✅     | Actionable errors with dev server guidance         |
| **Testing & QA**                          |        |                                                    |
| Unit tests for retry logic                | ✅     | 57/57 tests passing (all edge cases covered)       |
| Integration tests with delayed connection | ✅     | Simulate slow WebSocket connect                    |
| Manual testing in browser dev mode        | ✅     | 3/4 tests passing (MT4 deferred)                   |
| Regression testing in native mode         | ⏳     | MT4: DAW smoke test deferred to pre-release        |
| QA review                                 | ✅     | 0 blocking issues, approved for merge              |
| **Documentation**                         |        |                                                    |
| Update JSDoc comments                     | ✅     | Retry behavior documented                          |
| Changelog entry                           | ✅     | v0.11.1 release notes added                        |

**Problem Statement:**

When using browser dev mode (`wavecraft start`), the `useAllParameters()` hook may fire before the WebSocket connection to the dev server is established. This causes the initial parameter fetch to fail silently with no retry, leaving the UI with empty parameter lists until a manual page refresh.

**Current Behavior:**

- Hook calls `getAllParameters()` on mount
- If WebSocket isn't connected, request fails
- Error is caught and set in state
- User sees empty parameter list or loading state indefinitely

**Expected Behavior:**

- Hook should wait for WebSocket connection or retry automatically
- When connection becomes ready, parameters fetch automatically
- Clear error messages guide user to run `wavecraft start` if server not running
- No manual page refresh required

**Technical Approach:**

The hook already uses `useConnectionStatus()` to monitor transport state. Two potential solutions:

1. **Event-based (recommended):** WebSocketTransport emits "connected" event, hook subscribes and triggers reload
2. **Polling-based (current):** Hook monitors connection state changes via `useConnectionStatus()` polling

The implementation should:

- Leverage existing `useConnectionStatus()` hook
- Add retry logic with reasonable timeout (10-15s)
- Differentiate between connection failures and request failures
- Handle React lifecycle edge cases (unmount during retry, Strict Mode)
- Maintain backward compatibility with NativeTransport (WKWebView)

**Success Criteria:**

- [x] Parameters load automatically when WebSocket connects after component mount
- [x] Parameters reload automatically after WebSocket reconnection
- [x] Clear error messages when dev server isn't running
- [x] No regressions in native plugin mode (WKWebView)
- [x] All unit and integration tests passing (57/57)
- [x] Manual testing validates fix in browser dev mode (3/4 tests, MT4 deferred)

**Deliverables:**

- Connection-aware parameter loading with automatic retry
- 15-second timeout with actionable error messages
- Auto-refetch on reconnection
- Zero breaking changes to public API
- Comprehensive test coverage (57/57 passing)

**User Stories:** [docs/feature-specs/\_archive/ui-parameter-load-race-condition/user-stories.md](feature-specs/_archive/ui-parameter-load-race-condition/user-stories.md)

**Estimated Effort:** 2-3 days

---

## Milestone 18.8: Agent Search Delegation Instructions ✅

> **Goal:** Add "Codebase Research" guidance to all specialized agent instructions to ensure proper delegation to the Search agent for deep codebase analysis.

**Depends on:** Milestone 18.7 (UI Race Condition Fix) — polished developer workflow

**Status:** ✅ Complete

**Branch:** `feature/agent-search-delegation`  
**Target Version:** Documentation-only (no version change)

**User Stories:** [docs/feature-specs/\_archive/agent-search-delegation/user-stories.md](feature-specs/_archive/agent-search-delegation/user-stories.md)

| Task                                               | Status | Notes                                             |
| -------------------------------------------------- | ------ | ------------------------------------------------- |
| **Requirements & Design**                          |        |                                                   |
| User stories                                       | ✅     | 8 user stories covering all agent types           |
| Low-level design                                   | ✅     | Instruction section structure defined and applied |
| Implementation plan                                | ✅     | Agent-specific rollout completed                  |
| **Instruction Updates**                            |        |                                                   |
| Add "Codebase Research" section to Architect agent | ✅     | Added with delegation rules and examples          |
| Add "Codebase Research" section to Planner agent   | ✅     | Added with dependency-mapping examples            |
| Add "Codebase Research" section to Coder agent     | ✅     | Added with implementation pattern examples        |
| Add "Codebase Research" section to Tester agent    | ✅     | Added with test-coverage analysis guidance        |
| Add "Codebase Research" section to QA agent        | ✅     | Added with quality review pattern guidance        |
| Add "Codebase Research" section to PO agent        | ✅     | Added with product/impact research guidance       |
| Add "Codebase Research" section to DocWriter agent | ✅     | Added with doc scope discovery guidance           |
| **Agent Development Flow Updates**                 |        |                                                   |
| Update agent invocation table                      | ✅     | Search delegation notes documented                |
| Document delegation pattern                        | ✅     | Delegate-vs-direct-search guidance documented     |
| Add Search agent usage examples                    | ✅     | Concrete examples included                        |
| **Testing & Validation**                           |        |                                                   |
| Validate instruction clarity                       | ✅     | Verified across specialized agent instructions    |
| Measure Search agent invocation increase           | ✅     | Adoption pattern established post-update          |
| Review instruction consistency                     | ✅     | Uniform format applied across agents              |
| **Documentation**                                  |        |                                                   |
| Update agent-development-flow.md                   | ✅     | Search delegation pattern documented              |
| Update README (if needed)                          | ✅     | Workflow updates reflected where relevant         |

**Problem Statement:**

Specialized agents (Architect, Planner, Coder, Tester, QA, PO, DocWriter) have the **capability** to invoke the Search agent (272K context window, can analyze 50-100 files simultaneously) but their instructions **do not tell them when or how** to do so.

**Current Behavior:**

- Agents use their own search tools (`semantic_search`, `grep_search`) for deep analysis
- Search agent's 272K context is underutilized
- Inconsistent research quality across agents
- Violates specialization pattern (agents should delegate, not do everything themselves)

**Expected Behavior:**

- Agents recognize when a task requires comprehensive codebase analysis
- Agents invoke Search agent with clear research queries
- Search agent analyzes 50-100 files, returns synthesized findings
- Agents use findings to inform their specialized work (design, planning, implementation, testing, QA)

**Technical Approach:**

Add a "Codebase Research" section to each specialized agent's instructions that:

1. Tells them NOT to do deep research themselves (stay focused on specialization)
2. Instructs them to invoke Search agent for comprehensive analysis
3. Provides concrete examples of when/how to invoke Search
4. Explains Search's 272K context advantage over direct search tools

**Success Metrics:**

| Metric                                          | Current  | Target                      |
| ----------------------------------------------- | -------- | --------------------------- |
| Search agent invocations per feature            | ~0-1     | ~3-5                        |
| Agents using own search tools for deep analysis | High     | Low (quick lookups only)    |
| Research quality consistency                    | Variable | Consistent (all use Search) |
| Time spent on research per agent                | High     | Low (delegate to Search)    |

**Success Criteria:**

- [ ] All 7 specialized agents have clear "Codebase Research" instructions
- [ ] Instructions follow consistent format (when/how/examples)
- [ ] agent-development-flow.md documents delegation pattern with examples
- [ ] Search agent invocation count increases after implementation
- [ ] Agents naturally delegate research tasks without prompting

**Estimated Effort:** 1-2 days (instructions-only update, no code changes)

**Rationale:** This infrastructure improvement ensures proper utilization of the Search agent's specialized capabilities and maintains the agent specialization philosophy. Should be completed before user testing to ensure agents work at peak efficiency.

---

## Milestone 18.9: Rust Hot-Reload for Dev Mode 🚧

> **Goal:** Automatically rebuild the plugin and reload parameters when Rust source files change during `wavecraft start`, bringing Rust development to parity with React HMR.

**Depends on:** Milestone 18.7 (UI Race Condition Fix) — ensures parameter reload works reliably on WebSocket reconnect

**Status:** ✅ Complete

**Branch:** `feature/rust-hot-reload`  
**Target Version:** `0.12.0` (minor — new feature)

**User Stories:** [docs/feature-specs/\_archive/rust-hot-reload/user-stories.md](feature-specs/_archive/rust-hot-reload/user-stories.md)

| Task                             | Status | Notes                                                      |
| -------------------------------- | ------ | ---------------------------------------------------------- |
| **Requirements & Design**        |        |                                                            |
| User stories                     | ✅     | Completed and archived                                     |
| Low-level design                 | ✅     | Completed and archived                                     |
| Implementation plan              | ✅     | Completed and archived                                     |
| **Implementation**               |        |                                                            |
| File watcher integration         | ✅     | Implemented (archived feature)                             |
| Debouncing strategy              | ✅     | Implemented and covered by automated tests                 |
| Rebuild orchestration            | ✅     | Implemented with rebuild/error handling                    |
| WebSocket server restart         | ✅     | Parameter reload path implemented                          |
| UI error display                 | ⏳     | Optional enhancement (not required for closure)            |
| Ignore patterns                  | ✅     | Implemented for watched sources                            |
| **Testing & QA**                 |        |                                                            |
| Unit tests for file watcher      | ✅     | Automated coverage present                                 |
| Integration tests                | ✅     | CI checks passing for covered flows                        |
| Manual testing                   | ✅     | User manually validated hot-reload behavior (2026-02-14)   |
| QA review                        | ✅     | No remaining blockers after manual validation confirmation |
| **Documentation**                |        |                                                            |
| Update SDK Getting Started guide | ✅     | Hot-reload workflow documented                             |
| Update Development Workflows doc | ✅     | File watching architecture documented                      |

**Problem Statement:**

When using `wavecraft start` for development, the CLI builds the Rust plugin once at startup for parameter discovery. It starts a WebSocket server serving those parameters and spawns Vite for UI hot-reload.

**Current Behavior:**

- User modifies Rust source (e.g., adds `Oscillator` to signal chain)
- Changes require manually stopping and restarting `wavecraft start`
- No feedback that rebuild is needed
- Breaks flow state

**Expected Behavior:**

- User saves Rust file
- CLI detects change via file watcher
- Rebuilds plugin dylib (`cargo build --lib`)
- Reloads parameters via FFI
- Restarts WebSocket server with new parameters
- UI automatically reconnects and fetches new parameter list
- User sees new oscillator parameters without manual restart

**Closure Note (2026-02-14):**

Manual user validation confirmed hot-reload behavior works as expected in practice. With implementation + automated coverage already in place and no active blockers remaining, M18.9 is closed as ✅ complete.

**Technical Approach:**

1. **File Watcher**: Use `notify` crate to watch Rust source files
2. **Debouncing**: Wait 500ms after last change before rebuilding (avoid rapid rebuilds)
3. **Rebuild**: Run `cargo build --lib --message-format=json` for structured errors
4. **Parameter Reload**: Use existing FFI parameter discovery from M13
5. **Server Restart**: Gracefully shut down and restart WebSocket server
6. **UI Reconnection**: Frontend already handles this (M18.7 ensures parameters reload)

**Known Patterns:**

- `cargo-watch` — Watches files and runs commands on change
- `notify` crate — Cross-platform file system notifications
- `watchexec` — Generic file watcher with debouncing

**Success Criteria:**

- [ ] CLI detects Rust file changes within 1 second
- [ ] Rebuilds complete within 10 seconds for typical changes
- [ ] WebSocket clients auto-reconnect and reload parameters
- [ ] Build errors shown in terminal (UI display optional)
- [ ] No performance impact when files aren't changing
- [ ] Works with typical Rust project structure

**Rationale:** Hot-reload is table-stakes in modern development tooling (2026). Beta testers (M19) will experiment with the template — if they can't see Rust changes without manual restarts, it creates immediate friction and bad first impressions. This is critical polish for the final ~10% before V1.0.

**User Impact:**

- **Who benefits:** Every plugin developer using `wavecraft start`
- **Frequency:** Dozens of times per development session
- **Time savings:** ~10-15 seconds per change (eliminate stop/restart cycle)
- **Flow preservation:** Maintains developer focus, avoids context switching

**Estimated Effort:** 4-6 days

---

## Milestone 18.10: TypeScript Parameter ID Autocompletion ✅

> **Goal:** Generate a TypeScript union type of all parameter IDs at build time so SDK developers get IDE autocompletion and compile-time type safety when referencing parameters in React code.

**Status: ✅ Complete**

| Task                                                               | Status | Notes                                                  |
| ------------------------------------------------------------------ | ------ | ------------------------------------------------------ |
| Generate `ui/src/generated/parameters.ts` during `wavecraft start` | ✅     | Union type generated from FFI-extracted parameter list |
| Update hooks/clients to accept `ParameterId` directly              | ✅     | Direct `ParameterId` API implemented                   |
| Hot-reload: regenerate types on Rust source changes                | ✅     | Integrated with rebuild/reload workflow                |
| Add `.gitignore` entry for generated file                          | ✅     | Generated file treated as build artifact               |
| Update SDK template with type-safe example                         | ✅     | Template demonstrates direct typed parameter usage     |
| Handle edge cases (empty params, special chars)                    | ✅     | Codegen handles edge cases and safe fallback           |
| Documentation updates                                              | ✅     | Docs and archived validation artifacts updated         |

**Depends on:**

- Existing `wavecraft start` FFI parameter extraction — parameter IDs extracted from compiled plugin

**Approach:** Build-Time Codegen (approved via architectural analysis of 6 alternatives)

**Generated File Example:**

```typescript
// Auto-generated by wavecraft start — do not edit
export type ParameterId =
  | 'inputgain_level'
  | 'outputgain_level'
  | 'oscillator_frequency';
```

**Usage:**

```typescript
const { param, setValue } = useParameter('oscillator_frequency');
// ✅ IDE autocompletion + compile-time type checking — no imports or generics needed
```

**Key Design Decisions:**

- Breaking change (acceptable pre-1.0): `useParameter(id: ParameterId)` directly — no generic type parameter
- `ParameterId` type is re-exported from `@wavecraft/core` and `@wavecraft/components`
- Integration point: `wavecraft start` (and `cargo xtask dev` which delegates to it)
- No new dependencies required
- Generated file is `.gitignore`d (build artifact)

**Target Version:** `0.13.0` (delivered)

**Estimated Effort:** 1-2 days (completed)

**User Stories:** See [docs/feature-specs/\_archive/ts-param-autocomplete/user-stories.md](feature-specs/_archive/ts-param-autocomplete/user-stories.md)

---

## Pre-M19 Initiative: CLI Update UX Quick Wins + Optional Dev Build Profile Spike ✅

> **Goal:** Land two high-impact CLI polish items before/alongside M19, plus an optional low-risk Rust dev build optimization spike that must not delay M19.

**Status:** ✅ Complete (all 3 items shipped)

**Execution Guardrail:** If any item risks delaying M19 timeline, defer immediately and continue M19.

### Scope & Acceptance Criteria

| Item                                            | Scope                                                                                                                   | Acceptance Criteria                                                                                                                              |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1. CLI update rerun elimination                 | `wavecraft update` should complete CLI self-update and dependency update in one user invocation (no manual second run). | `wavecraft update` does not instruct user to re-run for normal self-update path; project deps update continues automatically in same invocation. |
| 2. Split progress messaging                     | Replace combined "Downloading and installing" with distinct progress states.                                            | CLI output clearly separates download step and install step for self-update flow.                                                                |
| 3. Conditional: optimize Rust dev build profile | Timeboxed experiment for dev build/runtime feedback improvements.                                                       | Only ship if improvement is measurable and implementation is low-risk with no workflow regressions.                                              |

### Priority / Order of Execution

1. **Item #2 first** (lowest risk, tiny UX improvement, quick confidence)
2. **Item #1 second** (high user-value, moderate implementation complexity)
3. **Item #3 last and conditional** (timeboxed, explicitly non-blocking)

### Timebox & Guardrails (Must Not Delay M19)

- **Total initiative budget:** max 2 working days
- **Item #2:** target same-day
- **Item #1:** target ≤1 day
- **Item #3:** max 0.5 day spike only, proceed only after #1 and #2 are complete
- If #1 needs architectural churn or introduces regressions, ship partial improvements and defer remainder
- If #3 does not show clear low-risk gains within timebox, **defer to backlog immediately**

### Risks & Defer Rules (Item #3)

| Risk                             | Trigger                                                  | Defer Rule                                |
| -------------------------------- | -------------------------------------------------------- | ----------------------------------------- |
| Slower incremental compile times | Noticeable rebuild slowdown during normal dev loop       | Revert/defer and keep current profile     |
| Debug ergonomics degrade         | Reduced debuggability, confusing profiler/stack behavior | Revert/defer                              |
| Cross-workspace side effects     | Affects CLI/dev-server/test workflows unexpectedly       | Scope down to engine-only or defer        |
| CI/local mismatch confusion      | Team uncertainty about what profile is active            | Defer until docs/tooling clarity is ready |

**Definition of done for #3:** only considered complete if net developer experience improves with no meaningful downside; otherwise "spike complete, implementation deferred" is the expected safe outcome.

---

## Changelog

| Date       | Update                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| ---------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 2026-02-15 | **PO archival/finalization complete: `oscilloscope`** — feature-spec documentation archived to `docs/feature-specs/_archive/oscilloscope/` with no content edits beyond move operation.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 2026-02-15 | **PO archival/finalization complete: `processor-presence-hook`** — feature-spec documentation archived to `docs/feature-specs/_archive/processor-presence-hook/` with no content edits beyond move operation.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| 2026-02-15 | **Feature closeout complete: `processor-presence-hook`** — roadmap finalized after implementation validation (`cargo xtask ci-check` passing) and QA PASS (no blockers). Follow-up architecture doc sync identified by Architect and tracked for: `docs/architecture/development-workflows.md`, `docs/architecture/sdk-architecture.md`, `docs/architecture/high-level-design.md`, `docs/architecture/declarative-plugin-dsl.md`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 2026-02-15 | **PO closeout verification: `processors-crate-migration`** — confirmed roadmap completion tracking and validated canonical archived spec location at `docs/feature-specs/_archive/processors-crate-migration/`; active spec path remains a redirect stub (no duplicate active spec content).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| 2026-02-15 | **Feature closeout complete: `processors-crate-migration`** — PO closeout recorded after Planner/Coder/Tester/QA/Architect approvals; feature-spec artifact archived to `docs/feature-specs/_archive/processors-crate-migration/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| 2026-02-15 | **Feature closeout complete: `npm-cohort-lockstep-versioning`** — roadmap updated for final closure after implementation, testing, and QA PASS; feature-spec documentation archived to `docs/feature-specs/_archive/npm-cohort-lockstep-versioning/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-15 | **Feature closeout complete: `browser-dev-audio-start-silence`** — marked complete after user validation and archived feature-spec documentation to `docs/feature-specs/_archive/browser-dev-audio-start-silence/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| 2026-02-14 | **Pre-M19 Initiative fully complete (all 3 items):** Items #1 & #2 shipped earlier (streaming progress, re-exec). Item #3 (dev build profile optimization) now also shipped: added `[profile.dev]` (opt-level 0, incremental) and `[profile.dev.package."*"]` (opt-level 2) to `sdk-template/Cargo.toml.template` — generated plugins now compile local code fast while dependencies run optimized. Template validation passed. Backlog entry updated to reflect completion.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| 2026-02-14 | **Pre-M19 initiative formalized (non-blocking):** Added execution-ready initiative for two CLI update quick wins + one conditional dev-build optimization spike. Defined scope and acceptance criteria, strict guardrails (max 2 working days), explicit priority order (#2 → #1 → optional #3), and defer rules ensuring **no delay to M19**. Updated Immediate Tasks ordering to run quick wins before/alongside M19 and treat item #3 as strictly timeboxed/conditional.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| 2026-02-14 | **Milestone 18.9 closed (manual validation confirmed):** Updated **M18.9 Rust Hot-Reload for Dev Mode** from 🚧 to ✅ based on direct user confirmation: _"I manually tested the hotreloading and it works great!"_ Manual testing status marked complete, QA blocker cleared, progress updated to **24/26**, and Next Steps ordering adjusted to proceed with M19 then M20.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| 2026-02-14 | **Roadmap audit corrections applied (status alignment):** Marked **M18.8 Agent Search Delegation** as ✅ complete (archived feature + agent instruction updates + delegation pattern documented). Marked **M18.10 TS Parameter ID Autocompletion** as ✅ complete (ts codegen wiring, generated parameters flow, archived validation docs). Reclassified **M18.9 Rust Hot-Reload** to 🚧 in progress with explicit manual-test blocker note from archived test-plan (`PARTIAL PASS`, blocked TC-001–TC-006). Updated progress overview to **23/26 (88%)**, corrected Up Next + Immediate Tasks ordering, and aligned roadmap narrative with validated delivery state.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-13 | **CI/CD parity for `cargo xtask ci-check`**: Follow-up to canonical SDK template on same PR branch (#66). Fixed CI pipeline (`ci.yml` and `build_ui.rs`) to build from `sdk-template/ui/` instead of old `ui/` app path. Extended `cargo xtask ci-check` from 3 phases to 6 phases with full CI/CD parity: Phase 0 (doc link validation), Phase 1 (UI dist build — always rebuilds, strict `npm ci`), Phase 2 (lint), Phase 3 (tests), Phase 4 (template validation, `--full` only), Phase 5 (CD dry-run with git-based change detection, `--full` only). Added `--full` flag for comprehensive pre-push validation (gates Phases 4-5). Added granular skip flags (`--skip-docs`, `--skip-lint`, `--skip-tests`, `--skip-template`, `--skip-cd`). Fixed `cli/build.rs` `include_dir!` build-time staging to filter `sdk-template/` excluding `target/`, `node_modules/`, `dist/` before embedding. QA fixes: expanded CLI change detection, strict npm install parity, replaced `unwrap()` with `expect()`.                                                                                                                                                                                                                                                                                                                                                                |
| 2026-02-13 | **Canonical SDK Template complete**: Consolidated three overlapping plugin scaffold sources (`cli/sdk-templates/`, `engine/crates/wavecraft-example/`, `ui/src/`) into a single canonical `sdk-template/` directory at repo root. CLI embedding, SDK-mode detection, and dev workflow all use this single source of truth. Setup script (`scripts/setup-dev-template.sh`) bootstraps template for local dev. Vite aliases preserved for package development. Legacy paths removed, `ui/` is now pure npm package workspace. 13/13 test cases passing, QA approved (0 Critical/High). Archived to `_archive/canonical-sdk-template/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| 2026-02-12 | **SDK Example Plugin complete**: Added `wavecraft-example` crate to enable `cargo xtask dev` from SDK root without creating separate plugin projects. Example plugin mirrors template structure (uses `wavecraft_plugin!`, `wavecraft_processor!`, `SignalChain![]` macros, `_param-discovery` feature, cdylib export). Modified CLI project detection to support "SDK mode" with backward compatibility. 392-line implementation plan, 7/7 test cases passing, QA approved with 0 issues. Enables SDK developers to test UI and dev-server changes in-tree. Version 0.11.2. Archived to `_archive/sdk-example-plugin/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| 2026-02-12 | **Hot-reload cancellation bugfix**: Fixed hang during hot-reload cancellation when user kills `cargo xtask dev` before reload completes. Added proper shutdown coordination between file watcher and reload guard, cleanup of stale state. All tests passing, QA approved. Archived to `_archive/hot-reload-cancellation/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| 2026-02-12 | **CD auto-bump bugfix complete**: Fixed continuous-deploy pipeline failures when local crate versions fall behind published versions on crates.io. Implemented auto-bump logic (Option A) in all three Rust publish jobs (CLI, engine, dev-server) to automatically increment versions instead of failing with error code 1. All crate versions bumped from 0.12.1 to 0.12.6 to align with published versions. Bugfix using lightweight workflow (no full feature spec). PR merged, archived to `_archive/version-auto-bump/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| 2026-02-11 | **Dev Server Unification complete**: Unified CLI's `dev_server` module and engine's `wavecraft-dev-server` crate into single `dev-server/` crate at repository root. Eliminates ~1,100 LOC of duplication, clarifies ownership, creates clean builder API for `wavecraft start`. Feature-gated audio support preserved (`audio` feature flag). Testing: 17/17 test cases passed, QA approved. Branch `fix/hotreloading` merged. Infrastructure improvement with zero breaking changes to public API. Documentation updated (high-level-design.md, development-workflows.md). Archived to `_archive/dev-server-unification/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| 2026-02-13 | **Milestone 18.10 added: TypeScript Parameter ID Autocompletion**: New developer experience milestone to generate TypeScript union type of all parameter IDs at build time during `wavecraft start`. Enables IDE autocompletion and compile-time type safety for `useParameter()` calls. Approach: Build-Time Codegen (approved via architectural analysis of 6 alternatives — codegen was unanimously selected). Rides existing FFI parameter extraction in `wavecraft start`. No new dependencies. 6 user stories covering autocompletion, auto-generation, hot-reload, type safety, project conventions, and template example. **Updated 2026-02-13:** Backward compatibility removed (not required pre-1.0) — `ParameterId` used directly by hooks/clients, no generics needed. Simpler API: `useParameter('param_id')` with direct autocompletion. Priority: HIGH — high-value, low-effort DX improvement (1-2 days) that should ship before user testing (M19). Target version 0.13.0. Progress: 81% (21/26 milestones).                                                                                                                                                                                                                                                                                                                                             |
| 2026-02-10 | **Milestone 18.9 added: Rust Hot-Reload for Dev Mode**: New developer experience milestone to automatically rebuild plugin and reload parameters when Rust source files change during `wavecraft start`. User request from real developer who added `Oscillator` to signal chain and expected changes to appear without manual restart (like React HMR). Assessment: HIGH priority — table-stakes feature in modern tooling, critical for first impressions with beta testers (M19). File watcher + rebuild orchestration + WebSocket restart. Frontend already handles reconnection (M18.7). Target version 0.12.0 (minor). Estimated effort 4-6 days. Depends on M18.7 (ensures parameter reload works reliably), blocks M19 (want this working before beta testers). User stories + low-level design to be created. Renumbered User Testing (M19→M24) and V1.0 Release (M20→M25). Progress: 84% (21/25 milestones). Technical approach: `notify` crate for file watching, 500ms debounce, `cargo build --lib --message-format=json` for structured errors, FFI parameter reload, graceful WebSocket restart. Known patterns: `cargo-watch`, `watchexec`. Success criteria: <1s change detection, <10s rebuild, auto-reconnect/reload parameters, build errors in terminal. User impact: dozens of times per session, 10-15s time savings per change, flow preservation. |
| 2026-02-10 | **Milestone 18.7 complete (v0.11.1)**: UI Parameter Load Race Condition Fix. `useAllParameters()` hook now waits for WebSocket connection before fetching (15s timeout with actionable error if dev server not running), auto-refetches on reconnection. Implementation: connection state synchronization with exponential backoff retry, max 3 attempts, graceful timeout handling. Testing: 57/57 unit tests (100% edge case coverage), 3/4 manual tests (MT4 deferred to pre-release validation). QA: 0 blocking issues, approved for merge. Zero breaking changes to public API. Eliminates silent failures and manual page refreshes, significantly improving developer experience in browser dev mode. Archived to `_archive/ui-parameter-load-race-condition/`. Progress: 87% (21/24 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| 2026-02-10 | **Milestone 18.8 added: Agent Search Delegation Instructions**: New infrastructure milestone to add "Codebase Research" guidance to all specialized agent instructions (Architect, Planner, Coder, Tester, QA, PO, DocWriter). Problem: Agents have capability (`agents: [..., search]`) but lack instructions on when/how to invoke Search. Solution: Add consistent "Codebase Research" section to each agent explaining delegation pattern, with concrete examples. Search agent has 272K context (50-100 files simultaneously) but is underutilized. Target: 3-5 Search invocations per feature (up from ~0-1). Documentation-only update (no version change). User stories created (8 stories + success metrics). Renumbered User Testing (M19→M20, now depends on M18.8), V1.0 Release (M20→M21). Progress: 83% (20/24 milestones). Estimated effort: 1-2 days. Rationale: Ensures proper Search agent utilization and maintains agent specialization philosophy before user testing.                                                                                                                                                                                                                                                                                                                                                                                |
| 2026-02-10 | **Milestone 18.7 added: Fix UI Race Condition on Parameter Load**: New bugfix milestone to address silent failure when `useAllParameters()` hook mounts before WebSocket connection is established. Affects browser dev mode (`wavecraft start`). Hook should automatically retry when connection becomes ready, eliminating need for manual page refresh. UI-only change in `@wavecraft/core`, no engine changes. Item promoted from backlog ("SDK Audio Architecture Gaps", Minor severity). Target version 0.11.1 (patch). User stories created (3 stories + 4 edge cases), comprehensive acceptance criteria defined. Renumbered User Testing (M19→M20, depends on M18.7), V1.0 Release (M20→M21). Progress: 87% (20/23 milestones). Estimated effort: 2-3 days. Status: User stories complete, awaiting Architect low-level design.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| 2026-02-09 | **Build-time parameter discovery fix**: Fixed `wavecraft start` hanging at "Loading plugin parameters..." on macOS. Root cause: Loading plugin dylib triggered nih-plug's static initializers (VST3/CLAP), which block on `AudioComponentRegistrar`. Solution: Feature-gated `nih_export_clap!` / `nih_export_vst3!` behind `#[cfg(not(feature = "_param-discovery"))]` in `wavecraft_plugin!` macro output. CLI now uses two-phase approach: sidecar JSON cache → discovery build without nih-plug init → graceful fallback for older plugins. Template updated with `_param-discovery` feature. 87 engine + 28 UI tests passing, template validation clean (clippy, symbol verification), QA approved. Architecture docs updated (`development-workflows.md`, `declarative-plugin-dsl.md`). Archived to `_archive/build-time-param-discovery/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 2026-02-09 | **Remove manual versioning complete**: Documentation and agent guidance updated to make versioning fully CI-automated with no manual per-feature or milestone bumps. User stories updated, PO template simplified, and policy aligned with CD behavior. Archived to `_archive/remove-manual-versioning/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| 2026-02-09 | **Replace Python with `xtask` complete**: Tooling scripts migrated from Python to `cargo xtask`, simplifying setup by removing Python dependency. QA approved (final sign-off), 11/11 tests passing, clippy clean. Archived to `_archive/replace-python-with-xtask/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-09 | **Milestone 18.6 complete (v0.10.1)**: Documentation Architecture Split. Split `coding-standards.md` (1,502 lines) and `high-level-design.md` (1,579 lines) into 9 focused topic documents + 2 navigation hubs. New docs: `coding-standards-typescript.md`, `coding-standards-css.md`, `coding-standards-rust.md`, `coding-standards-testing.md`, `sdk-architecture.md`, `declarative-plugin-dsl.md`, `development-workflows.md`, `plugin-formats.md`, `versioning-and-distribution.md`. 84.5% reduction in hub document size (3,081 → 479 lines). No document exceeds 600 lines. Zero broken links, 187 tests passing. Cross-references updated across 8 files including `.github/copilot-instructions.md`. QA approved (1 Medium finding fixed: missing `## Rust` heading). Archived to `_archive/docs-split-architecture/`. Progress: 91% (20/22 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| 2026-02-09 | **Milestone 18.5 complete (v0.11.0)**: Template Structure Improvement — Processors Module. Added `processors/` module to CLI template with sine-wave oscillator example demonstrating custom DSP processors with `#[derive(ProcessorParams)]`. Modular code organization teaches proper project structure from day one. Oscillator with frequency (20-20kHz) and level (0-1) parameters, phase accumulation, `set_sample_rate()` and `reset()` lifecycle methods. Safe defaults: gain-only signal chain by default, oscillator available as commented-out example. 23 files changed (873 insertions, 192 deletions). 12/12 manual tests, QA approved (PASS), architecture docs updated. Archived to `_archive/template-processors-module/`. Progress: 86% (19/22 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| 2026-02-08 | **Milestone 18.6 added: Documentation Architecture Split**: New infrastructure milestone to split large architecture documents into focused, topic-specific files. `coding-standards.md` (1,511 lines) splits into 5 documents (overview + TypeScript/CSS/Rust/Testing), `high-level-design.md` (1,562 lines) splits into 6 documents (overview + SDK/DSL/Workflows/Formats/Versioning). Target: 80-90% token reduction for AI agents, <30s navigation time for developers. Target version 0.10.1 (patch — documentation-only). 24 tasks across 4 phases (extraction, cross-references, validation). Milestone inserted between M18 (Complete) and M18.5 (Template Structure). Progress: 18/22 milestones (82%). Estimated effort: 4-6 hours.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| 2026-02-08 | **Milestone 18 complete (v0.10.0)**: Audio Pipeline Fixes & Mocking Cleanup fully implemented. (1) Full-duplex `AudioServer` with separate cpal input/output streams connected by `rtrb` SPSC ring buffer — audio now flows input → FfiProcessor::process() → output. (2) `AtomicParameterBridge` with `Arc<AtomicF32>` per parameter for lock-free WebSocket→audio thread parameter sync. (3) `MeterGenerator` deleted, fallback = silent zeros. (4) `useAllParameters` re-fetches on WebSocket reconnection. QA found 3 Medium issues (meter rate, tokio allocating on audio thread, unnecessary unsafe impl) — all fixed. 146 engine + 28 UI + 57 CLI tests passing. Architecture docs updated (high-level-design.md full-duplex diagram, coding-standards.md new patterns). Archived to `_archive/audio-pipeline-fixes/`. Progress: 86% (18/21 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| 2026-02-08 | **Milestone 18.5 added: Template Structure Improvement (Processors Module)**: New quality-of-life milestone to improve CLI template structure with `processors/` module and complete oscillator example. Teaches proper code organization from day one while providing engaging learning experience with real audio generation. Post-M18 feature per user request and architectural approval. Includes ~60-line oscillator implementation with phase accumulation, complete Processor trait example, safe defaults (gain-only chain), and comprehensive documentation updates. Target version 0.11.0 (minor — breaking template change). Renumbered User Testing (M19→M20) and V1.0 Release (M20→M21). Priority: Medium (quality improvement). Estimated effort: 3-5 days. Progress: 81% (17/21 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| 2026-02-08 | **Milestone 18 created: Audio Pipeline Fixes & Mocking Cleanup**: New milestone to fix two critical audio architecture gaps before user testing. (1) Add audio output stream to `AudioServer` (input → process → output), (2) Bridge parameter changes from WebSocket to audio thread via lock-free mechanism, (3) Remove synthetic `MeterGenerator` and related mocking infrastructure (YAGNI cleanup), (4) Fix `useAllParameters()` race condition on WebSocket connect. Items promoted from backlog. User Testing renumbered to M19, V1.0 Release to M20. Target version 0.10.0. Progress: 85% (17/20 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-08 | **Dev Audio FFI Abstraction (v0.9.1)**: Replaced template-embedded `dev-audio.rs` binary with in-process FFI/dlopen approach. The CLI now loads the user's DSP processor from their compiled cdylib via C-ABI FFI vtable (`DevProcessorVTable` in `wavecraft-protocol`). `wavecraft_plugin!` macro auto-generates FFI exports with `catch_unwind` for panic safety. Users never see or touch audio capture code — template simplified (removed `src/bin/`, 6 optional deps, `[[bin]]` section). Backward compatible: plugins compiled without vtable gracefully fall back to metering-only mode. 7 implementation phases across 5 crates (protocol, macros, bridge, dev-server, CLI). 150+ engine tests, 28 UI tests passing. QA: 4/5 findings resolved (1 minor deferred — low risk), all Critical/Major/Medium addressed. Architecture docs updated (high-level-design.md, coding-standards.md). Archived to `_archive/dev-audio-ffi/`.                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| 2026-02-08 | **CLI Self-Update enhancement (v0.9.1)**: Enhanced `wavecraft update` to self-update the CLI binary via `cargo install wavecraft` before updating project dependencies. Command now works from any directory (self-updates CLI only when outside a project). Two-phase execution: Phase 1 (CLI self-update) is non-fatal, Phase 2 (project deps) preserved from M14. Version change notification with re-run hint. 19 tests (12 added from QA findings). QA approved (0 Critical/High/Medium). Architecture docs updated (high-level-design.md, sdk-getting-started.md). Also fixed pre-existing `test_apply_local_dev_overrides` test. Archived to `_archive/cli-self-update/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| 2026-02-08 | **Milestone 17 complete (v0.8.0)**: OS Audio Input for Dev Mode fully implemented. `wavecraft start` automatically detects, compiles, and starts audio-dev binary if present in plugin projects. Audio flows from OS microphone → user's Processor → meters → WebSocket → UI. Zero configuration required (always-on design with feature flags). Real-time safe (no tokio panics from audio thread). Protocol extensions: `registerAudio` method and `meterUpdate` notification. Audio server with cpal integration. WebSocket client for binary communication. SDK templates with optional audio-dev binary. 10 commits on feature branch. End-to-end testing complete (WebSocket client received meter updates). All template projects compile successfully. Manual testing validates full flow. Ready to archive feature spec and merge to main. Progress: 94% (17/18 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-08 | **Milestone 16 complete (v0.9.0)**: Macro API Simplification fully implemented. Plugin definition reduced from 9 lines to 4 lines (56% reduction). Metadata auto-derived from `Cargo.toml` (`CARGO_PKG_AUTHORS`, `CARGO_PKG_HOMEPAGE`). `SignalChain!` replaces `Chain!` (backward compatible with deprecation). Compile-time validation rejects bare processors. Breaking change: VST3 Class IDs now use package name. Implementation: 4 phases (Core macro, SignalChain rename, CLI template, QA docs). Testing: 107 tests passing (69 engine + 28 UI + 10 doctests), 10/10 functional tests. QA approved (9 findings, all addressed). Known limitation documented: DSL-generated plugins have parameters visible in DAW/UI but DSP receives defaults (workaround: manual `Plugin` trait implementation). Architecture docs updated (high-level-design.md, coding-standards.md). Archived to `_archive/macro-api-simplification/`. Progress: 89% (16/18 milestones).                                                                                                                                                                                                                                                                                                                                                                                                     |
| 2026-02-08 | **Milestone 16 created: Macro API Simplification**: New milestone for reducing boilerplate in `wavecraft_plugin!` macro. Based on user feedback, removes unnecessary properties (`vendor`, `url`, `email`), derives metadata from `Cargo.toml`, enforces consistent `SignalChain!` syntax, and renames `Chain!` → `SignalChain!`. Also makes `crate` property optional (hidden). Target version 0.9.0 (minor — breaking API change). Planning complete (user stories, low-level design by Architect, implementation plan by Planner). Renumbered User Testing (M16→M17) and V1.0 Release (M17→M18). Progress: 83% (15/18 milestones). Design decisions: metadata from `CARGO_PKG_*` env vars, VST3 ID uses package name instead of vendor (breaking), compile-time signal validation. Feature request from developer feedback about reducing boilerplate.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| 2026-02-08 | **Milestone 15 complete (v0.8.6)**: Developer Tooling Polish fully implemented. Extended `cargo xtask clean` to comprehensively clean entire workspace (7 directories: engine/target, cli/target, ui/dist, ui/coverage, target/tmp, bundled/, AU wrapper). Added 3 helper functions (`dir_size`, `format_size`, `remove_dir`) with 8 unit tests. Clear output with checkmarks and disk space reporting. Idempotent (no errors on missing dirs). 12/12 manual tests passing (100%). QA approved (0 issues). Architectural review: Fully compliant with all conventions, serves as reference implementation for future xtask commands. Documentation updated (high-level-design.md, implementation-progress.md, test-plan.md, QA-report.md, architectural-review.md). Ready for PO handoff to archive feature spec and merge to main. Progress: 88% (15/17 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-08 | **Milestone 15 added: Developer Tooling Polish**: New milestone for extending `cargo xtask clean` to comprehensively clean the entire workspace (cli/target, ui/dist, ui/coverage, target/tmp). Small quality-of-life improvement to reclaim disk space with a single command. Renumbered User Testing (M15→M16) and V1.0 Release (M16→M17). Progress: 82% (14/17 milestones). Target version 0.8.6 (patch). Item promoted from backlog with user stories created.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| 2026-02-08 | **Milestone 14 complete (v0.8.5)**: CLI Enhancements fully implemented. Version flags (`-V`/`--version`) using clap's built-in support (follows Rust CLI conventions with capital V). Update command (`wavecraft update`) updates both Rust and npm dependencies with graceful degradation (continues on partial failure). 9 integration tests passing (4 version + 5 update), 18/22 manual tests. QA approved (0 Critical/High issues). Architectural review: ⭐⭐⭐⭐⭐ (5/5) — excellent architectural quality, idiomatic Rust, proper error handling. Documentation updated (high-level-design.md, sdk-getting-started.md, architectural-review.md). Ready for PO handoff to archive feature spec and merge to main. Progress: 88% (14/16 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| 2026-02-08 | **Milestone 14 added: CLI Enhancements**: New milestone for version flag (`-v`/`--version`) and `wavecraft update` command to update all project dependencies (Rust + npm). Small quality-of-life improvements before user testing. Renumbered User Testing (M14→M15) and V1.0 Release (M15→M16). Progress: 81% (13/16 milestones). Target version 0.8.1 (patch). Items moved from backlog with documentation that `-v`/`--version` should display format `wavecraft 0.x.y`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| 2026-02-09 | **CD CLI cascade publish**: Enhanced Continuous Deployment pipeline with CLI cascade trigger, `[auto-bump]` loop prevention, and publish-only npm model. CLI now re-publishes whenever _any_ SDK component changes (engine crates, npm packages, or CLI itself), ensuring the git tag always reflects the latest SDK state. Replaced `[skip ci]` with `[auto-bump]` commit marker so other workflows (CI, template validation) still run on auto-bump commits. npm jobs switched to publish-only model (no build step — relies on pre-built `dist/` in repo). Added upstream failure guards (`!cancelled()` instead of `always()`). 12/12 test cases passing, QA approved (0 Critical/High/Medium). Architecture docs updated (high-level-design.md, ci-pipeline.md). Archived to `_archive/cd-cli-cascade-publish/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-08 | **CLI auto-detect local SDK**: CLI now auto-detects when running from monorepo source checkout (`cargo run` or `target/debug/wavecraft`) and uses path dependencies instead of git tags. Eliminates the need for `--local-sdk` flag during SDK development. Runtime binary path inspection with SDK marker validation (`engine/crates/wavecraft-nih_plug/Cargo.toml`). 9/9 manual tests, 32 CLI unit tests, QA approved. Architecture docs updated (high-level-design.md, coding-standards.md, agent-development-flow.md). Archived to `_archive/cli-auto-local-sdk/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| 2026-02-08 | **CLI `wavecraft start` port preflight**: Added preflight port checks and strict UI port binding. Startup now fails fast when UI or WS ports are in use, avoiding partial startup and Vite auto-port switching. Docs updated (High-Level Design, Getting Started, coding standards/agent flow). Test plan re-run and QA completed.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| 2026-02-07 | **npm OIDC trusted publishing validation**: Branch run succeeded for `@wavecraft/components` and confirmed provenance publishing; `@wavecraft/core` publish on `main` still fails due to token injection. Workflow fix pending merge to `main` before re-validating OIDC publishes.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| 2026-02-07 | **Doctest cleanup + documentation guidance**: Replaced ignored Rust doctests with `rust,no_run` or `text` blocks where appropriate, updated examples to compile, and documented doctest conventions in coding standards. `cargo xtask ci-check` now runs with zero ignored doctests.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| 2026-02-07 | **Dev server rename (v0.7.2)**: Renamed `standalone` crate to `wavecraft-dev-server` to clarify purpose. Updated CLI/xtask wiring, docs/specs, and verified help output + dev server smoke tests. Test plan and QA report completed.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| 2026-02-07 | **CLI dev server reuse (v0.7.3)**: Refactored CLI dev server to reuse engine crates (shared in-memory host, FFI parameter loader, synthetic meter generator). Unified MeterFrame via protocol re-export and removed duplication. Tests and manual dev-server checks passing.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| 2026-02-06 | **Embedded Dev Server (v0.8.0)**: Added `wavecraft start` embedded WebSocket dev server with FFI parameter discovery. CLI now builds the plugin dylib, loads parameters via FFI, and starts WS + Vite for browser dev in plugin projects. Manual test plan updated and passing. Architecture docs updated (High-Level Design).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| 2026-02-06 | **Milestone 13 complete (v0.8.0)**: CLI UX Improvements. Internal testing revealed friction points in CLI workflow, resulting in 4 targeted improvements: (1) Zero prompts — removed `dialoguer` dependency, uses placeholder defaults, (2) SDK version auto-determined from CLI version, (3) Cleaner interface — removed `--sdk-version`, `--local-sdk` hidden boolean flag, (4) PATH troubleshooting guidance added to docs. 10/10 manual tests passing, QA approved with zero issues. Documentation updated (Getting Started, High-Level Design). Archived to `_archive/cli-ux-improvements/`. Progress: 87% (13/15 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| 2026-02-06 | **CI Build Stage Removal (v0.7.2)**: Removed redundant `build-plugin` job from CI workflow. Job never executed (workflow triggers on PRs, job condition required main branch). Simplifies CI from 7 jobs to 6, reduces confusion. Updated ci-pipeline.md, high-level-design.md, skill documentation. Version bumped to 0.7.2. PR #30. Archived to `_archive/ci-build-stage-removal/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-06 | **Template Reorganization**: Restructured CLI template from `cli/plugin-template/` to `cli/sdk-templates/new-project/react/` for better organization and future extensibility (vanilla, svelte variants). Fixed template xtask hardcoded plugin names (Issue #1) — now uses `{{plugin_name}}` and `{{plugin_name_snake}}` variables. Updated CLI default SDK version from v0.7.0 to v0.7.1 (Issue #2). CI path filters updated. All documentation updated (high-level-design.md, README.md, ci-pipeline.md). 10/10 tests passing, QA approved. Archived to `_archive/template-relocation-docs/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| 2026-02-06 | **wavecraft-core crate split for crates.io publishing**: Split wavecraft-core into wavecraft-core (publishable, no nih_plug dependency) + wavecraft-nih_plug (git-only, contains nih-plug integration). Added `__nih` module for proc-macro type exports. Template uses Cargo package rename (`wavecraft = { package = "wavecraft-nih_plug" }`). All 6 publishable crates validated with dry-run publish. 24/24 manual tests, QA approved. Architecture docs updated (high-level-design.md, coding-standards.md). Milestone 13 now **In Progress**.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| 2026-02-05 | **CI Workflow Simplification**: Removed redundant `push` triggers from CI and Template Validation workflows — they now only run on PRs (not on merge to main). Added `workflow_dispatch` for manual runs when needed. Eliminates ~10-14 CI minutes of redundant validation per merge. Documentation updated (ci-pipeline.md, high-level-design.md). Archived to `_archive/ci-workflow-simplification/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 2026-02-04 | **CLI `--local-dev` flag**: Added `--local-dev` CLI option to `wavecraft create` for SDK development and CI. Generates path dependencies (e.g., `path = "/path/to/engine/crates/wavecraft-core"`) instead of git tag dependencies. Solves CI chicken-egg problem where template validation fails because git tags don't exist until after PR merge. Mutually exclusive with `--sdk-version`. 10/10 unit tests, 10/10 manual tests. Documentation updated (sdk-getting-started.md, ci-pipeline.md). Archived to `_archive/ci-local-dev-dependencies/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-04 | **Continuous Deployment implemented (v0.7.1)**: Added `continuous-deploy.yml` workflow for automatic package publishing on merge to main. Path-based change detection using `dorny/paths-filter` — only changed packages are published. Auto-patch version bumping with bot commits. Supports: CLI (crates.io), 6 engine crates (crates.io), `@wavecraft/core` (npm), `@wavecraft/components` (npm). Existing `cli-release.yml` and `npm-release.yml` converted to manual overrides. Full documentation added to `docs/guides/ci-pipeline.md`. Version bumped to 0.7.1 across all packages.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| 2026-02-04 | **Milestone 12 complete (v0.7.0)**: Open Source Readiness fully implemented. **wavecraft CLI** published to crates.io (`cargo install wavecraft && wavecraft create my-plugin`). **npm packages** published: `@wavecraft/core@0.7.0` (IPC bridge, hooks, Logger, utilities) and `@wavecraft/components@0.7.0` (Meter, ParameterSlider, ParameterGroup, VersionBadge). **Template system** converted to use npm packages instead of bundled UI copy. **CI workflows** for template validation and CLI release. 75/75 implementation tasks complete. 20/20 manual tests passing. QA approved (0 Critical/High issues). Architecture docs updated (npm package imports, subpath exports). Archived to `_archive/open-source-readiness/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-03 | **CI Pipeline Optimization complete**: Added `cargo xtask check` command for fast local validation (~52s, 26x faster than Docker CI). Pre-compile test binaries in CI with `cargo test --no-run`. Tiered artifact retention (7 days main / 90 days tags, ~75-80% storage reduction). Updated agent documentation (Tester uses `cargo xtask check`, QA focuses on manual review). Architecture docs updated (high-level-design.md, ci-pipeline.md, coding-standards.md). Version 0.6.2.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| 2026-02-03 | **Milestone 11 complete**: Code Quality & OSS Prep fully implemented. UI Logger (`Logger` class in `@wavecraft/ipc` with debug/info/warn/error methods), Engine logging (`tracing` crate, 24 println! calls migrated), open source infrastructure (LICENSE, CONTRIBUTING.md, CODE_OF_CONDUCT.md, issue/PR templates), README polish. Horizontal scroll fix applied. Template project synchronized. 110+ engine tests, 43 UI tests, 19/19 manual tests passing. QA approved (5 findings resolved). Logging standards documented in coding-standards.md. Version 0.6.1. Archived to `_archive/code-quality-polish/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| 2026-02-03 | **Added Milestones 12 & 13**: User Testing (v0.7.0) and V1.0 Release (v1.0.0). User Testing focuses on validating SDK with 3-5 beta testers before stable release. V1.0 is the final milestone marking first production-ready release. Updated progress to 77% (10/13 milestones complete).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| 2026-02-04 | **Added Milestone 12: Internal Testing**: Inserted comprehensive internal validation milestone before User Testing. Ensures polished experience before external beta testers. Tasks include: fresh clone setup, template plugin build, DAW loading, parameter sync, state persistence, multi-instance, documentation walkthrough, regression testing, template validation, edge cases. Renumbered User Testing (M12→M13) and V1.0 Release (M13→M14). Progress now 79% (11/14 milestones).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| 2026-02-03 | **CI optimization complete (v0.6.2)**: Pre-push validation with `cargo xtask check` (~52s, 26x faster than Docker). Pre-compile test binaries with `--no-run`. Tiered artifact retention (7d main / 90d tags). Archived to `_archive/ci-optimization/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 2026-02-03 | **Milestone 10 complete**: Declarative Plugin DSL fully implemented. 95% code reduction (190 lines → 9 lines), `wavecraft_plugin!` macro for zero-boilerplate plugins, `#[derive(ProcessorParams)]` with `#[param(...)]` attributes, `wavecraft_processor!` for named wrappers, `Chain!` combinator for signal chains. Built-in processors (Gain, Passthrough). UI parameter groups (`ParameterGroup` component, `useParameterGroups` hook). 63 tests (28 engine + 35 UI), 18/18 manual tests, all linting clean. DAW verified in Ableton Live. VstKit branding updated to Wavecraft. ProcessorParams `group` field fixed. QA approved. Version 0.6.0. Archived to `_archive/declarative-plugin-dsl/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| 2026-02-03 | **Milestone reprioritization**: Declarative Plugin DSL promoted to Milestone 10 (was unscheduled). Code Quality & OSS Prep moved to Milestone 11. Rationale: DSL significantly improves DX and is a key differentiator before open-source release. Planning complete (user stories, low-level design, implementation plan with 40 steps across 9 phases).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| 2026-02-03 | **Project rename fully deployed**: PR #17 merged to main, GitHub repository renamed `vstkit` → `wavecraft`. All source code references updated. Milestone 9 complete and in production.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 2026-02-02 | **Milestone 10 created**: Code Quality & OSS Prep. Focus on polish before open-source release: logging infrastructure (UI Logger class, Engine tracing crate), horizontal scroll fix, CI cache optimization, open-source readiness (license review, CONTRIBUTING.md, issue templates). Target version 0.5.1.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| 2026-02-02 | **Milestone 9 complete**: Project renamed from VstKit to Wavecraft (v0.5.0). 156 files changed across 7 implementation phases. 5 SDK crates renamed (`wavecraft-*`), `wavecraft_plugin!` macro, `@wavecraft/*` npm aliases, `__WAVECRAFT_IPC__` global, AU wrapper updated. 24/24 manual tests, all automated checks passing, all QA findings resolved. Architecture docs updated. Ready for open-source release. Archived to `_archive/project-rename-wavecraft/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| 2026-02-02 | **Added Milestone 9: Project Rename (Wavecraft → Wavecraft)**: Rebrand to avoid "VST" trademark concerns before open-source release. Scope includes Rust crates, npm packages, GitHub repo, documentation, and UI branding. Pending availability checks for name.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 2026-02-02 | **Milestone 8 complete**: Developer SDK Phase 1 fully implemented. 5-crate SDK architecture (`wavecraft-protocol`, `wavecraft-dsp`, `wavecraft-bridge`, `wavecraft-metering`, `wavecraft-core`), `wavecraft_plugin!` macro for zero-boilerplate plugins, template project, comprehensive documentation. 111 engine + 35 UI tests passing, 22/22 manual tests. QA approved, architect review complete (added `unwrap()`/`expect()` coding standards). Version 0.4.0. **ALL MILESTONES COMPLETE!** Archived to `_archive/developer-sdk/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 2026-02-01 | **Milestone 8 created**: Developer SDK initiative. Phase 1 focuses on investigation with architect to define packaging strategy, SDK boundaries, and developer experience. Goal: make Wavecraft usable by external developers.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| 2026-02-01 | **Milestone 7 complete**: Browser-Based Visual Testing infrastructure fully implemented. Playwright @1.41.0 with Chromium installed, 18 test IDs added across all UI components (Meter, ParameterSlider, VersionBadge, ResizeHandle, ConnectionStatus, App root). External baseline storage design (`~/.wavecraft/visual-baselines/`). Comprehensive 11KB documentation guide. **Bonus:** Fixed version display — now reads from Cargo.toml in dev mode, improved VersionBadge styling for visibility. 35/35 unit tests, 18/18 feature tests passing. QA approved. Architecture docs updated. Version 0.3.1. Archived to `_archive/browser-visual-testing/`. **ALL COMMITTED MILESTONES COMPLETE!**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| 2026-02-01 | **Milestone 6 complete**: WebSocket IPC Bridge fully implemented and tested. Transport abstraction with factory pattern, `WebSocketTransport` with exponential backoff reconnection, `cargo xtask dev` unified development command, graceful degradation UI. 14/14 integration tests, 35 UI tests, 17 Rust tests passing. QA approved, architectural docs updated. Version 0.3.0. Archived to `_archive/websocket-ipc-bridge/`. Ready to merge `feature/websocket-ipc-bridge` branch.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| 2026-02-01 | **Backlog split from roadmap**: Created separate [backlog.md](backlog.md) for unprioritized future ideas. Removed Milestone 8 from roadmap — committed milestones now end at M7. Backlog contains: CI optimization, performance profiling, platform support, DAW compatibility, AU issues, Apple Developer-dependent items.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| 2026-02-01 | **Milestone 5 complete, starting M6**: Marked M5 (Polish & Optimization) as complete. Moved remaining low-priority tasks (CI cache optimization, performance profiling, format-specific parity) to new Milestone 8 (Backlog). Started Milestone 6 (WebSocket IPC Bridge) on `feature/websocket-ipc-bridge` branch.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| 2026-02-01 | **Dead code cleanup complete**: Established platform-gating pattern using `#[cfg(any(target_os = "macos", target_os = "windows"))]` for code that only runs on GUI platforms. Reduced `#[allow(dead_code)]` suppressions from 14 to 3 (79% reduction). Remaining 3 are valid cases (trait methods called by platform implementations). Pattern documented in new "Platform-Specific Code" section of coding-standards.md. Archived to `_archive/m5-dead-code-cleanup/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 2026-02-01 | **Resize handle visibility complete**: Handle visibility significantly improved — opacity increased (30%→50% white), hover/drag states use accent blue (#4a9eff/#6bb0ff), size increased (24×24→36×36px button, 16×16→20×20px icon), positioned 20px from right edge (scrollbar clearance). **Bonus:** Fixed WebView background color mismatch during over-scroll (was white, now matches dark theme). Version bumped to 0.2.1. All 13 tests passing, QA approved. Archived to `_archive/resize-handle-visibility/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| 2026-02-01 | **Milestone 6 elevated to WebSocket IPC Bridge**: Expanded scope from "Browser-Based UI Testing" to full WebSocket IPC infrastructure. Addresses development workflow pain point (mock data double implementation). Original testing goals moved to new Milestone 7. Added detailed task breakdown for Rust (WebSocket server, `--dev-server` flag) and UI (transport abstraction, auto-detect).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| 2026-02-01 | **Added Milestone 7: Browser-Based Visual Testing**: Playwright integration and visual regression testing. Depends on M6 WebSocket bridge. Separated from M6 to maintain single-responsibility milestones.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| 2026-02-01 | **Added dead code cleanup task**: ~12 `#[allow(dead_code)]` suppressions in editor modules (webview.rs, bridge.rs, assets.rs, mod.rs, windows.rs) need review. Added as workaround during resize-handle feature; now that React UI is default, unused code should be removed.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| 2026-02-01 | **React UI default complete**: Removed `webview_editor` feature flag, deleted egui fallback editor, simplified build commands. React UI is now the only editor implementation. Version bumped to 0.2.0. QA approved. Archived to `_archive/react-ui-default/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| 2026-01-31 | **Semantic versioning complete**: Version extracted from `engine/Cargo.toml` (single source of truth), injected at build time via Vite `define`, displayed in UI via `VersionBadge` component. 8/8 manual tests + 35/35 unit tests passing. **Bonus delivery:** Browser development mode with environment detection and lazy IPC initialization — unblocks browser-based UI testing (partial Milestone 6). QA approved. Archived to `_archive/semantic-versioning/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| 2026-01-31 | **CI/CD pipeline redesign complete**: New staged pipeline with 6 specialized jobs (typecheck-ui, lint-ui, lint-engine, test-ui, test-engine, build-plugin). Stage 1 (fast feedback) on ubuntu, Stage 2 (tests) on ubuntu, Stage 3 (build) on macos (main only). Concurrency control, artifact sharing, branch protection. PR time <5 min, cost optimized (~90% ubuntu runners). Archived to `_archive/ci-cd-pipeline-redesign/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| 2026-01-31 | **UI unit testing framework complete**: Vitest + React Testing Library with IPC mock module. 25 passing tests covering ParameterSlider, Meter, and audio-math utilities. Unified `cargo xtask test` command with `--ui` and `--engine` flags. CI workflow ready (PR trigger disabled pending pipeline redesign). QA approved. Archived to `_archive/ui-unit-testing/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| 2026-01-31 | **TailwindCSS implementation complete**: Migrated all 7 component CSS files to Tailwind utilities. Custom theme with semantic tokens (plugin-dark, plugin-surface, accent, meter colors). Bundle size 3.74KB gzipped (63% under 10KB target). QA approved. Architectural docs updated. Archived to `_archive/tailwindcss/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| 2026-01-31 | **Added Milestone 6: Browser-Based UI Testing Infrastructure**: WebSocket IPC bridge to enable Playwright testing with real engine communication. Addresses limitation that UI can only talk to engine inside WKWebView. Enables automated visual testing, remote debugging, and hot-reload development with engine connectivity.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 2026-01-31 | **Added UI unit testing framework to Milestone 5**: Vitest + React Testing Library for component testing. Enables test-driven development and regression prevention for React UI components.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| 2026-01-31 | **Linting infrastructure complete**: Full implementation of unified linting system. ESLint 9 + Prettier for UI (TypeScript/React), Clippy + fmt for Engine (Rust). New `cargo xtask lint` command with `--ui`, `--engine`, `--fix` flags. CI workflow in `.github/workflows/lint.yml`. All 12 test scenarios passing. QA approved. Archived to `_archive/linting-infrastructure/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| 2026-01-31 | **Added TailwindCSS implementation to Milestone 5**: Upgraded from "investigate" to full implementation item. Rationale: industry standard for React, excellent flexibility, strong documentation and LLM tooling support.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| 2026-01-31 | **Archived signing-validation feature**: All in-scope phases complete (ad-hoc signing, Ableton Live testing, CI/CD). Docs moved to `_archive/signing-validation/`. Developer ID + notarization deferred until Apple Developer account available.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| 2026-01-31 | **Renamed `docs/specs` to `docs/feature-specs`**: Directory and all 16 references across 8 agent/config files updated. Clearer naming communicates these are feature specifications under active development. Archive references preserved as historical records.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| 2026-01-31 | **Milestone 4 fully validated**: Ableton Live (macOS) testing complete — plugin loads without security warnings, React UI renders, parameters work, automation syncs, state persists, multi-instance works. Ad-hoc signing validated. Developer ID signing/notarization deferred until Apple Developer account available.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| 2026-01-31 | **CI/CD pipeline paused for redesign**: Current pipeline disabled on PRs (was blocking). Scheduled for dedicated architecture review to define proper phases (build, lint, test, release). Will collaborate with architect.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| 2026-01-31 | **Linting infrastructure design complete**: User stories (7) and low-level design created. Covers ESLint + Prettier for UI, Clippy + fmt for Rust, `cargo xtask lint` commands, QA agent integration, and CI workflow. Ready for implementation.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| 2026-01-31 | Added **Linting infrastructure** to Milestone 5 — ESLint/Prettier for UI, Clippy/fmt for Rust, xtask commands, QA agent integration, CI enforcement. User stories in `docs/feature-specs/_archive/linting-infrastructure/`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| 2026-01-31 | **Milestone 4 implementation complete**: Code signing and notarization infrastructure implemented. Three new xtask commands (`sign`, `notarize`, `release`) with full CI/CD pipeline and documentation. Ready for manual testing with Apple Developer credentials.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| 2026-01-31 | Added "CI/CD pipeline (GitHub Actions)" to Milestone 5 — automated builds, tests, and release workflow.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 2026-01-31 | Added "Implement semantic versioning" to Milestone 5 — SemVer for consistent release tracking.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| 2026-01-30 | Initial roadmap created. Milestone 1 (Plugin Skeleton) marked complete.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| 2026-01-30 | **Milestone 2 complete**: WebView Desktop POC fully functional with <1ms IPC latency. Ready for plugin integration.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| 2025-01-31 | **Milestone 3 in progress**: WKWebView integration complete, working in Ableton Live. Added resizing and TailwindCSS investigation to roadmap.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| 2026-01-31 | **Clipping indicator complete**: Pure UI implementation with peak detection, 2-second sticky hold, pulsing red button, and click-to-reset.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| 2026-01-30 | AU wrapper validated with auval, but shows generic view (clap-wrapper limitation). Added "AU custom UI" to roadmap.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| 2026-01-31 | **Plugin editor window resizing complete**: Implemented IPC-based resize system with `requestResize()` method. UI can request size changes via React hook, host approves/rejects. Tested with preset sizes (600x400 to 1280x960).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |

---

## Next Steps

> 🚀 **Roadmap status** — Core milestones are complete; future work continues via roadmap additions and backlog promotion.

### Completed Milestones

1. ✅ **Milestone 1**: Plugin Skeleton — Rust plugin with VST3/CLAP export
2. ✅ **Milestone 2**: WebView Desktop POC — React embedded with <1ms IPC latency
3. ✅ **Milestone 3**: Plugin UI Integration — Full React UI in plugin with metering
4. ✅ **Milestone 4**: macOS Hardening — Code signing, notarization infrastructure
5. ✅ **Milestone 5**: Polish & Optimization — Linting, testing, TailwindCSS, CI/CD
6. ✅ **Milestone 6**: WebSocket IPC Bridge — Real engine data in browser development
7. ✅ **Milestone 7**: Browser-Based Visual Testing — Playwright infrastructure with test IDs
8. ✅ **Milestone 8**: Developer SDK — 5-crate SDK architecture, macro, template, docs
9. ✅ **Milestone 9**: Project Rename — VstKit → Wavecraft (v0.5.0)
10. ✅ **Milestone 10**: Declarative Plugin DSL — Macro-based DSL for 95% code reduction (v0.6.0)
11. ✅ **Milestone 11**: Code Quality & OSS Prep — Logging, CI optimization, open-source readiness (v0.6.2)
12. ✅ **Milestone 12**: Open Source Readiness — CLI, npm packages, template independence (v0.7.0)
13. ✅ **Milestone 13**: Internal Testing — CLI UX improvements, comprehensive validation (v0.8.0)
14. ✅ **Milestone 14**: CLI Enhancements — Version flags, update command, CLI self-update (v0.9.1)
15. ✅ **Milestone 15**: Developer Tooling Polish — Comprehensive workspace cleanup (v0.8.6)
16. ✅ **Milestone 16**: Macro API Simplification — Reduced boilerplate, automatic metadata (v0.9.0)
17. ✅ **Milestone 17**: OS Audio Input for Dev Mode — Automatic audio input detection and processing (v0.8.0)
18. ✅ **Milestone 18**: Audio Pipeline Fixes — Full-duplex audio, parameter sync, mocking cleanup (v0.10.0)
19. ✅ **Milestone 18.5**: Template Structure Improvement — Processors module with oscillator example (v0.11.0)
20. ✅ **Milestone 18.6**: Documentation Architecture Split — Split large docs into focused files (v0.10.1)
21. ✅ **Milestone 18.7**: UI Parameter Load Race Condition — Auto-retry parameter fetch on WebSocket connect (v0.11.1)
22. ✅ **Milestone 18.8**: Agent Search Delegation — Codebase research guidance across specialized agents
23. ✅ **Milestone 18.9**: Rust Hot-Reload for Dev Mode — Automatic Rust rebuild + parameter reload workflow validated manually
24. ✅ **Milestone 18.10**: TypeScript Parameter ID Autocompletion — Build-time generated typed parameter IDs (v0.13.0)

### Up Next

- ✅ No active upcoming milestones currently listed

### Immediate Tasks

1. ✅ ~~Execute **Pre-M19 Initiative** (Item #2 → Item #1) under strict 2-day guardrail~~ — Completed 2026-02-14
2. 📝 **Pre-release validation:** MT4 (native plugin DAW testing) deferred from M18.7 — smoke test in Ableton before final release window
3. 📝 **Architecture-doc sync follow-up (`processor-presence-hook`)**: apply Architect-identified updates in `docs/architecture/development-workflows.md`, `docs/architecture/sdk-architecture.md`, `docs/architecture/high-level-design.md`, and `docs/architecture/declarative-plugin-dsl.md` (post-archival follow-up)

**Future ideas:** See [backlog.md](backlog.md) for unprioritized items (crates.io publication, additional example plugins, etc.)
