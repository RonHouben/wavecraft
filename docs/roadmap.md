# VstKit Roadmap

This document tracks implementation progress against the milestones defined in the [High-Level Design](architecture/high-level-design.md).

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

**Status: ⏳ Not Started**

| Task | Status | Notes |
|------|--------|-------|
| Integrate webview into plugin GUI (nih-plug editor) | ⏳ | |
| WKWebView integration (macOS) | ⏳ | |
| WebView2 integration (Windows) | ⏳ | |
| Implement parameter bridge (UI ↔ host params) | ⏳ | |
| Implement SPSC ring buffer for audio → UI metering | ⏳ | |
| Implement meter visualization in React | ⏳ | |
| Test parameter automation roundtrip | ⏳ | |

---

## Milestone 4: Cross-Platform Hardening & Packaging (Week 8–12)

**Status: ⏳ Not Started**

| Task | Status | Notes |
|------|--------|-------|
| macOS code signing | ⏳ | |
| macOS notarization | ⏳ | |
| Windows code signing | ⏳ | |
| Windows installer (MSI) | ⏳ | |
| Linux packaging (AppImage/Flatpak) | ⏳ | |
| **Host Compatibility Testing** | | |
| Ableton Live (macOS) | ⏳ | |
| Ableton Live (Windows) | ⏳ | |
| Logic Pro (macOS, AU) | ⏳ | |
| GarageBand (macOS, AU) | ⏳ | |
| Reaper (all platforms) | ⏳ | |
| Cubase | ⏳ | |
| FL Studio | ⏳ | |
| **AU Validation** | | |
| `auval` passes without errors | ⏳ | |
| State save/restore (`.aupreset`) | ⏳ | |
| AU cache invalidation workflow documented | ⏳ | |

---

## Milestone 5: Polish & Optimization (Ongoing)

**Status: ⏳ Not Started**

| Task | Status | Notes |
|------|--------|-------|
| Performance profiling (low buffer sizes: 32/64 samples) | ⏳ | |
| CPU stress testing | ⏳ | |
| Memory usage optimization | ⏳ | |
| UX polish | ⏳ | |
| Format-specific feature parity verification | ⏳ | |
| Cross-engine rendering consistency (WebKit vs Chromium) | ⏳ | |
| Automated visual regression tests | ⏳ | |

---

## Changelog

| Date | Update |
|------|--------|
| 2026-01-30 | Initial roadmap created. Milestone 1 (Plugin Skeleton) marked complete. |
| 2026-01-30 | **Milestone 2 complete**: WebView Desktop POC fully functional with <1ms IPC latency. Ready for plugin integration. |

---

## Next Steps

1. **Milestone 3**: Integrate WebView into nih-plug plugin editor
   - Adapt desktop POC's WebView setup for nih-plug's editor trait
   - Bridge nih-plug parameter system with existing IPC protocol
   - Test in Ableton Live VST3 host
2. Implement SPSC ring buffers for audio → UI metering
3. Validate AU build in Logic Pro (Milestone 1 completion)
