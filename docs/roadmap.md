# VstKit Roadmap

This document tracks implementation progress against the milestones defined in the [High-Level Design](architecture/high-level-design.md).

---

## Progress Overview

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│  ✅ M1        ✅ M2        ✅ M3        ✅ M4           ✅ M5        ✅ M6            ✅ M7          ⭐    │
│  Skeleton ─── WebView ─── Plugin UI ─── macOS ─────── Polish ───── WebSocket ───── Visual Testing ── Done  │
│                                                                                       ▲                    │
│                                                                              ALL MILESTONES COMPLETE!      │
│                                                                                                            │
│  Progress: [████████████████████████████████████████████████████████████████████████████████████████] 100% │
└────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

**See also:** [Backlog](backlog.md) — unprioritized ideas for future consideration

---

## Status Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Complete |
| 🚧 | In Progress |
| ⏳ | Not Started |
| ⚠️ | Blocked |

---

## Milestone 1: Plugin Skeleton (Week 0–2)

**Status: ✅ Complete**

| Task | Status | Notes |
|------|--------|-------|
| Rust plugin skeleton with nih-plug | ✅ | Core plugin structure in place |
| VST3 export | ✅ | |
| CLAP export | ✅ | |
| Native placeholder UI | ✅ | |
| Confirm Ableton host load (VST3) | ✅ | |
| Set up clap-wrapper build for AU | ✅ | CMake configuration in `packaging/macos/au-wrapper/` |
| Confirm Logic Pro load (AU) | ⏳ | |
| Confirm GarageBand load (AU) | ✅ | |

---

## Milestone 2: WebView Desktop POC (Week 2–4)

**Status: ✅ Complete**

| Task | Status | Notes |
|------|--------|-------|
| Create minimal React app (Vite + TypeScript) | ✅ | Full React 18 + TypeScript 5 + Vite 6 setup |
| Embed React app in Rust desktop app via wry | ✅ | wry 0.47 with WKWebView (macOS) |
| Implement basic IPC bridge (JSON-RPC style) | ✅ | Complete JSON-RPC 2.0 implementation |
| Test `setParameter` / `getParameter` roundtrip | ✅ | All tests passing (6/6 integration tests) |
| Test message latency characteristics | ✅ | p50: 0.003ms, p95: 0.003ms (well below 5ms target) |
| Bundle static assets into Rust binary | ✅ | `include_dir!` embedding, single binary |

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
- Responses must be sent via `evaluate_script()` calling `window.__VSTKIT_IPC__._receive()`
- Channel-based approach works well for decoupling IPC handler from event loop
- Windows/Linux untested (no dev machines available) but theoretically supported

---

## Milestone 3: Plugin UI Integration (Week 4–8)

**Status: ✅ Complete**

| Task | Status | Notes |
|------|--------|-------|
| Integrate webview into plugin GUI (nih-plug editor) | ✅ | WKWebView with Editor trait |
| WKWebView integration (macOS) | ✅ | Custom URL scheme handler for assets |
| WebView2 integration (Windows) | ⏳ | Deprioritized — macOS + Ableton is primary target |
| Implement parameter bridge (UI ↔ host params) | ✅ | GuiContext integration |
| Implement SPSC ring buffer for audio → UI metering | ✅ | rtrb-based MeterProducer/Consumer |
| Implement meter visualization in React | ✅ | Peak/RMS meters with dB display |
| Show clipping indicator in meter UI | ✅ | Red pulsing button, 2s hold, click-to-reset |
| Test parameter automation roundtrip | ✅ | Tested in Ableton Live |
| Plugin editor window resizing | ✅ | IPC-based resize with host approval |

---

## Milestone 4: macOS Hardening & Packaging (Week 8–12)

**Status: ✅ Complete**

> **Scope:** Focused on macOS + Ableton Live as the primary target. Windows/Linux support is deprioritized.

| Task | Status | Notes |
|------|--------|-------|
| macOS code signing | ✅ | `cargo xtask sign` command implemented |
| macOS notarization | ✅ | `cargo xtask notarize` command (deferred until Apple Developer account) |
| Windows code signing | ⏳ | Deprioritized |
| Windows installer (MSI) | ⏳ | Deprioritized |
| Linux packaging (AppImage/Flatpak) | ⏳ | Deprioritized |
| **Host Compatibility Testing** | | |
| Ableton Live (macOS) | ✅ | **Primary target** — validated 2026-01-31 |
| Ableton Live (Windows) | ⏳ | Deprioritized |
| Logic Pro (macOS, AU) | ⏳ | Secondary (nice-to-have) |
| GarageBand (macOS, AU) | ⏳ | Secondary (nice-to-have) |
| Reaper (all platforms) | ⏳ | Deprioritized |
| Cubase | ⏳ | Deprioritized |
| FL Studio | ⏳ | Deprioritized |
| **AU Validation** | | |
| `auval` passes without errors | ✅ | Validated 2026-01-30 |
| Investigate AU custom UI issue | ⏳ | clap-wrapper shows generic view; root cause TBD |
| State save/restore (`.aupreset`) | ⏳ | |
| AU cache invalidation workflow documented | ⏳ | |

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

| Task | Status | Notes |
|------|--------|-------|
| **Linting infrastructure** | ✅ | ESLint + Prettier (UI), Clippy + fmt (Engine), `cargo xtask lint` command, CI workflow. Completed 2026-01-31. |
| **TailwindCSS for React UI** | ✅ | Utility-first CSS replacing component CSS files. Custom theme with semantic tokens. 3.74KB gzipped (under 10KB target). Completed 2026-01-31. |
| **UI unit testing framework** | ✅ | Vitest + React Testing Library. IPC mock module for isolated component testing. 25 passing tests. `cargo xtask test --ui` command. CI workflow ready (PR trigger disabled pending pipeline redesign). Completed 2026-01-31. |
| **Resize handle visibility** | ✅ | Handle visibility improved: 50% white (was 30%), accent blue on hover/drag, 36×36px (was 24×24), 20px scrollbar clearance. WebView background color fixed. Completed 2026-02-01. |
| **Make React UI default** | ✅ | Removed `webview_editor` feature flag; React UI is now the only editor. Deleted egui fallback. Version bumped to 0.2.0. Completed 2026-02-01. |
| **Dead code cleanup** | ✅ | Platform-gating pattern established for macOS/Windows-only code. Reduced `#[allow(dead_code)]` suppressions from 14 to 3 (79% reduction). Remaining 3 are valid (trait methods called by platform impls). Patterns documented in coding-standards.md. Completed 2026-02-01. |
| **Semantic versioning** | ✅ | Version extracted from `engine/Cargo.toml` (single source of truth), injected at build time via Vite `define`. VersionBadge component displays version in UI. **Bonus:** Browser dev mode with environment detection and lazy IPC init (partial M6). Completed 2026-01-31. |
| CI/CD pipeline (GitHub Actions) | ✅ | Redesigned staged pipeline with 6 jobs across 3 stages. Ubuntu for lint/test (cost optimization), macos for build. Branch protection configured. Completed 2026-01-31. |
| CI pipeline cache optimization | ➡️ | Moved to Milestone 8 (Backlog). |
| Performance profiling | ➡️ | Moved to Milestone 8 (Backlog). |
| Format-specific feature parity | ➡️ | Moved to Milestone 8 (Backlog). |

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

| Task | Status | Notes |
|------|--------|-------|
| **Architecture & Design** | | |
| WebSocket IPC bridge design doc | ✅ | Transport abstraction, protocol compatibility |
| User stories | ✅ | 7 user stories covering dev workflow |
| **Rust Implementation** | | |
| Add WebSocket server to standalone crate | ✅ | `tokio-tungstenite` with async broadcast |
| Route WebSocket messages through existing `IpcHandler` | ✅ | Same JSON-RPC protocol |
| Add `--ws-only` CLI flag | ✅ | Headless mode for browser-only dev |
| Meter data streaming over WebSocket | ✅ | Push-based updates at 30fps |
| **UI Implementation** | | |
| Create `WebSocketTransport` class | ✅ | Exponential backoff reconnection |
| Abstract `IpcBridge` to support multiple transports | ✅ | Factory pattern with lazy init |
| Auto-detect environment and select transport | ✅ | WKWebView → native, browser → WebSocket |
| Reconnection handling | ✅ | Max 5 attempts with backoff (1s→16s) |
| **Developer Experience** | | |
| Document dev workflow | ✅ | `cargo xtask dev` runs both servers |
| Unified dev command | ✅ | Single command starts WS + Vite |
| Graceful degradation in browser | ✅ | Shows helpful status when disconnected |
| **Cleanup** | | |
| Remove static mock data from `IpcBridge` | ✅ | Browser mode uses real engine data |

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

| Task | Status | Notes |
|------|--------|-------|
| **Infrastructure** | | |
| Playwright installation | ✅ | @playwright/test ^1.41.0, Chromium 145.0.7632.6 |
| Playwright configuration | ✅ | playwright.config.ts with Chromium, baseURL localhost:5173 |
| .gitignore updates | ✅ | Excluded playwright-report/ and test-results/ |
| **Test ID Implementation** | | |
| App root test ID | ✅ | `data-testid="app-root"` |
| Meter component test IDs | ✅ | 10 IDs (meter, meter-L/R, peak/rms, dB, clip button) |
| ParameterSlider test IDs | ✅ | 4 dynamic IDs using template literals |
| VersionBadge test ID | ✅ | `data-testid="version-badge"` |
| ResizeHandle test ID | ✅ | `data-testid="resize-handle"` |
| ConnectionStatus test ID | ✅ | `data-testid="connection-status"` |
| **Documentation** | | |
| Visual Testing Guide | ✅ | 11KB comprehensive guide at docs/guides/visual-testing.md |
| README link | ✅ | Added to Documentation section |
| High-level design update | ✅ | New Visual Testing section with architecture diagram |
| **Additional Improvements** | | |
| Version badge visibility | ✅ | Improved styling (text-sm, font-medium, text-accent) |
| Dev mode version display | ✅ | Reads from Cargo.toml via vite.config.ts parser |

**Key Deliverables:**
- 18 test IDs across all UI components for reliable Playwright selection
- External baseline storage design (`~/.vstkit/visual-baselines/`)
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
| Baseline storage | External (`~/.vstkit/`) | Keep repo lean |
| Test orchestration | Agent-driven | On-demand, not CI (avoids screenshot flakiness) |
| Component targeting | `data-testid` attributes | Stable, framework-agnostic selectors |

---

## Changelog

| Date | Update |
|------|--------|
| 2026-02-01 | **Milestone 7 complete**: Browser-Based Visual Testing infrastructure fully implemented. Playwright @1.41.0 with Chromium installed, 18 test IDs added across all UI components (Meter, ParameterSlider, VersionBadge, ResizeHandle, ConnectionStatus, App root). External baseline storage design (`~/.vstkit/visual-baselines/`). Comprehensive 11KB documentation guide. **Bonus:** Fixed version display — now reads from Cargo.toml in dev mode, improved VersionBadge styling for visibility. 35/35 unit tests, 18/18 feature tests passing. QA approved. Architecture docs updated. Version 0.3.1. Archived to `_archive/browser-visual-testing/`. **ALL COMMITTED MILESTONES COMPLETE!** |
| 2026-02-01 | **Milestone 6 complete**: WebSocket IPC Bridge fully implemented and tested. Transport abstraction with factory pattern, `WebSocketTransport` with exponential backoff reconnection, `cargo xtask dev` unified development command, graceful degradation UI. 14/14 integration tests, 35 UI tests, 17 Rust tests passing. QA approved, architectural docs updated. Version 0.3.0. Archived to `_archive/websocket-ipc-bridge/`. Ready to merge `feature/websocket-ipc-bridge` branch. |
| 2026-02-01 | **Backlog split from roadmap**: Created separate [backlog.md](backlog.md) for unprioritized future ideas. Removed Milestone 8 from roadmap — committed milestones now end at M7. Backlog contains: CI optimization, performance profiling, platform support, DAW compatibility, AU issues, Apple Developer-dependent items. |
| 2026-02-01 | **Milestone 5 complete, starting M6**: Marked M5 (Polish & Optimization) as complete. Moved remaining low-priority tasks (CI cache optimization, performance profiling, format-specific parity) to new Milestone 8 (Backlog). Started Milestone 6 (WebSocket IPC Bridge) on `feature/websocket-ipc-bridge` branch. |
| 2026-02-01 | **Dead code cleanup complete**: Established platform-gating pattern using `#[cfg(any(target_os = "macos", target_os = "windows"))]` for code that only runs on GUI platforms. Reduced `#[allow(dead_code)]` suppressions from 14 to 3 (79% reduction). Remaining 3 are valid cases (trait methods called by platform implementations). Pattern documented in new "Platform-Specific Code" section of coding-standards.md. Archived to `_archive/m5-dead-code-cleanup/`. |
| 2026-02-01 | **Resize handle visibility complete**: Handle visibility significantly improved — opacity increased (30%→50% white), hover/drag states use accent blue (#4a9eff/#6bb0ff), size increased (24×24→36×36px button, 16×16→20×20px icon), positioned 20px from right edge (scrollbar clearance). **Bonus:** Fixed WebView background color mismatch during over-scroll (was white, now matches dark theme). Version bumped to 0.2.1. All 13 tests passing, QA approved. Archived to `_archive/resize-handle-visibility/`. |
| 2026-02-01 | **Milestone 6 elevated to WebSocket IPC Bridge**: Expanded scope from "Browser-Based UI Testing" to full WebSocket IPC infrastructure. Addresses development workflow pain point (mock data double implementation). Original testing goals moved to new Milestone 7. Added detailed task breakdown for Rust (WebSocket server, `--dev-server` flag) and UI (transport abstraction, auto-detect). |
| 2026-02-01 | **Added Milestone 7: Browser-Based Visual Testing**: Playwright integration and visual regression testing. Depends on M6 WebSocket bridge. Separated from M6 to maintain single-responsibility milestones. |
| 2026-02-01 | **Added dead code cleanup task**: ~12 `#[allow(dead_code)]` suppressions in editor modules (webview.rs, bridge.rs, assets.rs, mod.rs, windows.rs) need review. Added as workaround during resize-handle feature; now that React UI is default, unused code should be removed. |
| 2026-02-01 | **React UI default complete**: Removed `webview_editor` feature flag, deleted egui fallback editor, simplified build commands. React UI is now the only editor implementation. Version bumped to 0.2.0. QA approved. Archived to `_archive/react-ui-default/`. |
| 2026-01-31 | **Semantic versioning complete**: Version extracted from `engine/Cargo.toml` (single source of truth), injected at build time via Vite `define`, displayed in UI via `VersionBadge` component. 8/8 manual tests + 35/35 unit tests passing. **Bonus delivery:** Browser development mode with environment detection and lazy IPC initialization — unblocks browser-based UI testing (partial Milestone 6). QA approved. Archived to `_archive/semantic-versioning/`. |
| 2026-01-31 | **CI/CD pipeline redesign complete**: New staged pipeline with 6 specialized jobs (typecheck-ui, lint-ui, lint-engine, test-ui, test-engine, build-plugin). Stage 1 (fast feedback) on ubuntu, Stage 2 (tests) on ubuntu, Stage 3 (build) on macos (main only). Concurrency control, artifact sharing, branch protection. PR time <5 min, cost optimized (~90% ubuntu runners). Archived to `_archive/ci-cd-pipeline-redesign/`. |
| 2026-01-31 | **UI unit testing framework complete**: Vitest + React Testing Library with IPC mock module. 25 passing tests covering ParameterSlider, Meter, and audio-math utilities. Unified `cargo xtask test` command with `--ui` and `--engine` flags. CI workflow ready (PR trigger disabled pending pipeline redesign). QA approved. Archived to `_archive/ui-unit-testing/`. |
| 2026-01-31 | **TailwindCSS implementation complete**: Migrated all 7 component CSS files to Tailwind utilities. Custom theme with semantic tokens (plugin-dark, plugin-surface, accent, meter colors). Bundle size 3.74KB gzipped (63% under 10KB target). QA approved. Architectural docs updated. Archived to `_archive/tailwindcss/`. |
| 2026-01-31 | **Added Milestone 6: Browser-Based UI Testing Infrastructure**: WebSocket IPC bridge to enable Playwright testing with real engine communication. Addresses limitation that UI can only talk to engine inside WKWebView. Enables automated visual testing, remote debugging, and hot-reload development with engine connectivity. |
| 2026-01-31 | **Added UI unit testing framework to Milestone 5**: Vitest + React Testing Library for component testing. Enables test-driven development and regression prevention for React UI components. |
| 2026-01-31 | **Linting infrastructure complete**: Full implementation of unified linting system. ESLint 9 + Prettier for UI (TypeScript/React), Clippy + fmt for Engine (Rust). New `cargo xtask lint` command with `--ui`, `--engine`, `--fix` flags. CI workflow in `.github/workflows/lint.yml`. All 12 test scenarios passing. QA approved. Archived to `_archive/linting-infrastructure/`. |
| 2026-01-31 | **Added TailwindCSS implementation to Milestone 5**: Upgraded from "investigate" to full implementation item. Rationale: industry standard for React, excellent flexibility, strong documentation and LLM tooling support. |
| 2026-01-31 | **Archived signing-validation feature**: All in-scope phases complete (ad-hoc signing, Ableton Live testing, CI/CD). Docs moved to `_archive/signing-validation/`. Developer ID + notarization deferred until Apple Developer account available. |
| 2026-01-31 | **Renamed `docs/specs` to `docs/feature-specs`**: Directory and all 16 references across 8 agent/config files updated. Clearer naming communicates these are feature specifications under active development. Archive references preserved as historical records. |
| 2026-01-31 | **Milestone 4 fully validated**: Ableton Live (macOS) testing complete — plugin loads without security warnings, React UI renders, parameters work, automation syncs, state persists, multi-instance works. Ad-hoc signing validated. Developer ID signing/notarization deferred until Apple Developer account available. |
| 2026-01-31 | **CI/CD pipeline paused for redesign**: Current pipeline disabled on PRs (was blocking). Scheduled for dedicated architecture review to define proper phases (build, lint, test, release). Will collaborate with architect. |
| 2026-01-31 | **Linting infrastructure design complete**: User stories (7) and low-level design created. Covers ESLint + Prettier for UI, Clippy + fmt for Rust, `cargo xtask lint` commands, QA agent integration, and CI workflow. Ready for implementation. |
| 2026-01-31 | Added **Linting infrastructure** to Milestone 5 — ESLint/Prettier for UI, Clippy/fmt for Rust, xtask commands, QA agent integration, CI enforcement. User stories in `docs/feature-specs/linting-infrastructure/`. |
| 2026-01-31 | **Milestone 4 implementation complete**: Code signing and notarization infrastructure implemented. Three new xtask commands (`sign`, `notarize`, `release`) with full CI/CD pipeline and documentation. Ready for manual testing with Apple Developer credentials. |
| 2026-01-31 | Added "CI/CD pipeline (GitHub Actions)" to Milestone 5 — automated builds, tests, and release workflow. |
| 2026-01-31 | Added "Implement semantic versioning" to Milestone 5 — SemVer for consistent release tracking. |
| 2026-01-30 | Initial roadmap created. Milestone 1 (Plugin Skeleton) marked complete. |
| 2026-01-30 | **Milestone 2 complete**: WebView Desktop POC fully functional with <1ms IPC latency. Ready for plugin integration. |
| 2025-01-31 | **Milestone 3 in progress**: WKWebView integration complete, working in Ableton Live. Added resizing and TailwindCSS investigation to roadmap. |
| 2026-01-31 | **Clipping indicator complete**: Pure UI implementation with peak detection, 2-second sticky hold, pulsing red button, and click-to-reset. |
| 2026-01-30 | AU wrapper validated with auval, but shows generic view (clap-wrapper limitation). Added "AU custom UI" to roadmap. |
| 2026-01-31 | **Plugin editor window resizing complete**: Implemented IPC-based resize system with `requestResize()` method. UI can request size changes via React hook, host approves/rejects. Tested with preset sizes (600x400 to 1280x960). |

---

## Next Steps

> 🎉 **All committed milestones are complete!** VstKit has achieved its initial development goals.

### Completed Milestones
1. ✅ **Milestone 1**: Plugin Skeleton — Rust plugin with VST3/CLAP export
2. ✅ **Milestone 2**: WebView Desktop POC — React embedded with <1ms IPC latency
3. ✅ **Milestone 3**: Plugin UI Integration — Full React UI in plugin with metering
4. ✅ **Milestone 4**: macOS Hardening — Code signing, notarization infrastructure
5. ✅ **Milestone 5**: Polish & Optimization — Linting, testing, TailwindCSS, CI/CD
6. ✅ **Milestone 6**: WebSocket IPC Bridge — Real engine data in browser development
7. ✅ **Milestone 7**: Browser-Based Visual Testing — Playwright infrastructure with test IDs

### What's Next?

VstKit is now a **production-ready framework** for building audio plugins with Rust + React. Future work should focus on:

1. **Build real plugins** — Use VstKit to create actual audio effects/instruments
2. **Community feedback** — Gather user feedback from real-world usage
3. **Backlog prioritization** — Review [backlog.md](backlog.md) for next priorities

**Potential next priorities from backlog:**
- Apple Developer account setup (enables signed distribution)
- Performance profiling under high CPU load
- Additional DAW compatibility testing (Logic Pro AU)
- Windows platform support (if demand exists)

**Future ideas:** See [backlog.md](backlog.md) for unprioritized items (platform support, performance, DAW compatibility, etc.)
