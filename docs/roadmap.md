# VstKit Roadmap

This document tracks implementation progress against the milestones defined in the [High-Level Design](architecture/high-level-design.md).

---

## Progress Overview

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│  ✅ M1        ✅ M2        ✅ M3        ✅ M4           🚧 M5        ⏳ M6            ⏳ M7              ⭐       │
│  Skeleton ─── WebView ─── Plugin UI ─── macOS ─────── Polish ───── WebSocket ───── Visual Testing ─── Complete │
│                                                         ▲                                                      │
│                                                       YOU ARE HERE                                             │
│                                                                                                                │
│  Progress: [██████████████████████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 67%        │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

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

**Status: 🚧 In Progress**

| Task | Status | Notes |
|------|--------|-------|
| **Linting infrastructure** | ✅ | ESLint + Prettier (UI), Clippy + fmt (Engine), `cargo xtask lint` command, CI workflow. Completed 2026-01-31. |
| **TailwindCSS for React UI** | ✅ | Utility-first CSS replacing component CSS files. Custom theme with semantic tokens. 3.74KB gzipped (under 10KB target). Completed 2026-01-31. |
| **UI unit testing framework** | ✅ | Vitest + React Testing Library. IPC mock module for isolated component testing. 25 passing tests. `cargo xtask test --ui` command. CI workflow ready (PR trigger disabled pending pipeline redesign). Completed 2026-01-31. |
| Performance profiling (low buffer sizes: 32/64 samples) | ⏳ | |
| CPU stress testing | ⏳ | |
| Memory usage optimization | ⏳ | |
| UX polish | ⏳ | |
| **Resize handle visibility** | ✅ | Handle visibility improved: 50% white (was 30%), accent blue on hover/drag, 36×36px (was 24×24), 20px scrollbar clearance. WebView background color fixed. Completed 2026-02-01. |
| Format-specific feature parity verification | ⏳ | |
| Cross-engine rendering consistency (WebKit vs Chromium) | ⏳ | |
| Automated visual regression tests | ⏳ | |
| **Make React UI default** | ✅ | Removed `webview_editor` feature flag; React UI is now the only editor. Deleted egui fallback. Version bumped to 0.2.0. Completed 2026-02-01. |
| **Dead code cleanup** | ⏳ | Remove `#[allow(dead_code)]` suppressions and associated unused code in editor modules (webview.rs, bridge.rs, assets.rs, mod.rs, windows.rs). ~12 instances added as workaround during resize-handle feature. Now that React UI is default, assess what's actually needed vs. deletable. |
| **Semantic versioning** | ✅ | Version extracted from `engine/Cargo.toml` (single source of truth), injected at build time via Vite `define`. VersionBadge component displays version in UI. **Bonus:** Browser dev mode with environment detection and lazy IPC init (partial M6). Completed 2026-01-31. |
| CI/CD pipeline (GitHub Actions) | ✅ | Redesigned staged pipeline with 6 jobs across 3 stages. Ubuntu for lint/test (cost optimization), macos for build. Branch protection configured. Completed 2026-01-31. |
| CI pipeline cache optimization | ⏳ | Test Engine job rebuilds instead of using cache from Check Engine (different profiles: check vs test). Consider adding `cargo test --no-run` to prepare-engine job or combining check + test jobs. |

---

## Milestone 6: WebSocket IPC Bridge

**Status: ⏳ Not Started**

> **Goal:** Enable real IPC communication between the React UI running in a browser and the Rust engine, eliminating the need for mock data during development.

**Problem Statement:**
Currently, the UI can only communicate with the Rust engine when running inside WKWebView (plugin or desktop app). When developing with `npm run dev` in a browser, the UI falls back to static mock data. This creates:
- **Double implementation** — Mock behavior can drift from real engine behavior
- **Limited dev experience** — Can't see real meters, test actual parameter changes
- **Testing gaps** — Automated browser testing (Playwright) can't use real engine data

**Solution:**
Add a WebSocket server to the desktop app that exposes the same IPC protocol over `ws://localhost:9000`. The UI auto-detects the environment and connects via WebSocket when not in WKWebView.

**Benefits:**
- **Single source of truth** — Same `IpcHandler` serves both native and WebSocket transports
- **Real dev experience** — Hot reload with `npm run dev` + live meters, real parameter sync
- **Testing foundation** — Enables Milestone 7 (Playwright visual testing)
- **Future extensibility** — Remote debugging, external tools, mobile companion apps

| Task | Status | Notes |
|------|--------|-------|
| **Architecture & Design** | | |
| WebSocket IPC bridge design doc | ⏳ | Transport abstraction, protocol compatibility |
| User stories | ⏳ | |
| **Rust Implementation** | | |
| Add WebSocket server to desktop crate | ⏳ | `tokio-tungstenite` or similar |
| Route WebSocket messages through existing `IpcHandler` | ⏳ | Same protocol, different transport |
| Add `--dev-server` CLI flag | ⏳ | Starts WebSocket server without UI window |
| Meter data streaming over WebSocket | ⏳ | Push-based updates for real-time meters |
| **UI Implementation** | | |
| Create `WebSocketTransport` class | ⏳ | Implements same interface as native bridge |
| Abstract `IpcBridge` to support multiple transports | ⏳ | Factory pattern or strategy |
| Auto-detect environment and select transport | ⏳ | WKWebView → native, browser → WebSocket |
| Reconnection handling | ⏳ | Auto-reconnect on disconnect |
| **Developer Experience** | | |
| Document dev workflow (two-terminal setup) | ⏳ | `cargo run -p desktop -- --dev-server` + `npm run dev` |
| Consider Vite plugin for auto-starting Rust dev server | ⏳ | Nice-to-have |
| **Cleanup** | | |
| Remove static mock data from `IpcBridge` | ⏳ | No longer needed once WebSocket works |

---

## Milestone 7: Browser-Based Visual Testing

**Status: ⏳ Not Started**

> **Goal:** Automated visual regression testing using Playwright with real engine data (enabled by Milestone 6).

**Depends on:** Milestone 6 (WebSocket IPC Bridge)

| Task | Status | Notes |
|------|--------|-------|
| Playwright MCP integration | ⏳ | Automated browser control |
| Visual regression test suite | ⏳ | Screenshot comparisons |
| CI integration for visual tests | ⏳ | Run on PR, compare against baseline |
| Test scenarios (meters, parameters, resize) | ⏳ | Cover key UI behaviors |

---

## Changelog

| Date | Update |
|------|--------|
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

> **Focus:** Milestone 5 (Polish & Optimization) is the active milestone. Milestone 6 (WebSocket IPC Bridge) is next.

1. **Milestone 5**: Polish & Optimization (wrapping up)
   - ✅ ~~Linting infrastructure implementation~~ (completed 2026-01-31)
   - ✅ ~~TailwindCSS implementation for React UI~~ (completed 2026-01-31)
   - ✅ ~~UI unit testing framework~~ (completed 2026-01-31)
   - ✅ ~~CI/CD pipeline redesign~~ (completed 2026-01-31)
   - ✅ ~~Semantic versioning~~ (completed 2026-01-31)
   - ✅ ~~Make React UI default~~ (completed 2026-02-01)
   - ✅ ~~Resize handle visibility fix~~ (completed 2026-02-01)
   - **Remaining:** Dead code cleanup, CI cache optimization
2. **Milestone 6**: WebSocket IPC Bridge (next major feature)
   - Eliminates mock data problem in development
   - Enables real engine communication from browser
   - Foundation for automated visual testing
3. **Milestone 7**: Browser-Based Visual Testing
   - Playwright integration (depends on M6)
   - Visual regression test suite
4. **Investigate AU Custom UI Issue** (nice-to-have)
   - Understand why clap-wrapper shows generic parameter view
   - Research CLAP GUI extension forwarding in clap-wrapper

### Deferred (Future Consideration)
- **When Apple Developer account available**:
   - Developer ID signing validation
   - Notarization submission and Gatekeeper testing
   - Signed release CI/CD pipeline
- Windows WebView2 integration
- Linux support
- Non-Ableton DAW compatibility (Reaper, Cubase, FL Studio)
- Logic Pro / GarageBand AU testing
