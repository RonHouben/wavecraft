# VstKit Roadmap

This document tracks implementation progress against the milestones defined in the [High-Level Design](architecture/high-level-design.md).

---

## Progress Overview

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│  ✅ M1        ✅ M2        ✅ M3        ✅ M4           ⏳ M5                      │
│  Skeleton ─── WebView ─── Plugin UI ─── macOS ─────── Polish                     │
│                                          ▲                                       │
│                                        YOU ARE HERE                              │
│                                                                                  │
│  Progress: [████████████████████████████████████████████████░░░░░░░░░░░░] 80%    │
└──────────────────────────────────────────────────────────────────────────────────┘
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

**Status: ✅ Complete (Implementation)**

> **Scope:** Focused on macOS + Ableton Live as the primary target. Windows/Linux support is deprioritized.

| Task | Status | Notes |
|------|--------|-------|
| macOS code signing | ✅ | `cargo xtask sign` command implemented |
| macOS notarization | ✅ | `cargo xtask notarize` command implemented |
| Windows code signing | ⏳ | Deprioritized |
| Windows installer (MSI) | ⏳ | Deprioritized |
| Linux packaging (AppImage/Flatpak) | ⏳ | Deprioritized |
| **Host Compatibility Testing** | | |
| Ableton Live (macOS) | ⏳ | **Primary target** — ready for testing |
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

**Implementation Complete:**
- ✅ Entitlements files (production + debug)
- ✅ `cargo xtask sign` command (ad-hoc + Developer ID)
- ✅ `cargo xtask notarize` command (submit/status/staple/full)
- ✅ `cargo xtask release` command (complete workflow)
- ✅ GitHub Actions CI/CD pipeline
- ✅ Comprehensive documentation (`docs/guides/macos-signing.md`)

**Next:** Manual testing with Apple Developer credentials

---

## Milestone 5: Polish & Optimization (Ongoing)

**Status: ⏳ Not Started**

| Task | Status | Notes |
|------|--------|-------|
| **Linting infrastructure** | 🚧 | Design complete — [user stories](specs/linting-infrastructure/user-stories.md) + [low-level design](specs/linting-infrastructure/low-level-design-linting-infrastructure.md). Ready for implementation (~2h). |
| Performance profiling (low buffer sizes: 32/64 samples) | ⏳ | |
| CPU stress testing | ⏳ | |
| Memory usage optimization | ⏳ | |
| UX polish | ⏳ | |
| Investigate TailwindCSS for React UI | ⏳ | Evaluate utility-first CSS for plugin UI styling |
| Format-specific feature parity verification | ⏳ | |
| Cross-engine rendering consistency (WebKit vs Chromium) | ⏳ | |
| Automated visual regression tests | ⏳ | |
| Make React UI default (remove feature flag) | ⏳ | Remove `webview` feature flag; React UI should be the default editor. Investigate if old Rust GUI code (`nih-plug` native UI) can be fully removed. |
| Implement semantic versioning | ⏳ | SemVer for plugin releases; version in Cargo.toml, plugin metadata (VST3/CLAP/AU), **and visible in UI** so users can verify they're running the latest version |
| CI/CD pipeline (GitHub Actions) | ⚠️ | **Blocked for redesign** — Current pipeline disabled on PRs. Needs architecture review to define phases (build, lint, test, release). Work with architect to design proper pipeline structure. |

---

## Changelog

| Date | Update |
|------|--------|
| 2026-01-31 | **CI/CD pipeline paused for redesign**: Current pipeline disabled on PRs (was blocking). Scheduled for dedicated architecture review to define proper phases (build, lint, test, release). Will collaborate with architect. |
| 2026-01-31 | **Linting infrastructure design complete**: User stories (7) and low-level design created. Covers ESLint + Prettier for UI, Clippy + fmt for Rust, `cargo xtask lint` commands, QA agent integration, and CI workflow. Ready for implementation. |
| 2026-01-31 | Added **Linting infrastructure** to Milestone 5 — ESLint/Prettier for UI, Clippy/fmt for Rust, xtask commands, QA agent integration, CI enforcement. User stories in `docs/specs/linting-infrastructure/`. |
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

> **Focus:** macOS + Ableton Live is the primary target. Windows/Linux and other DAWs are deprioritized.

1. **Milestone 4**: macOS packaging & Ableton Live compatibility
   - macOS code signing and notarization
   - Thorough Ableton Live (macOS) testing
2. **Investigate AU Custom UI Issue** (nice-to-have)
   - Understand why clap-wrapper shows generic parameter view
   - Research CLAP GUI extension forwarding in clap-wrapper
   - Document findings and potential solutions
3. **Secondary**: Logic Pro AU validation (if time permits)
4. Investigate plugin editor resizing behavior in Ableton Live
5. Evaluate TailwindCSS for UI styling consistency

### Deprioritized (Future Consideration)
- Windows WebView2 integration
- Linux support
- Non-Ableton DAW compatibility (Reaper, Cubase, FL Studio)
