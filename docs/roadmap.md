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

**Status: ⏳ Not Started**

| Task | Status | Notes |
|------|--------|-------|
| Create minimal React app (Vite + TypeScript) | ⏳ | |
| Embed React app in Rust desktop app via wry | ⏳ | |
| Implement basic IPC bridge (JSON-RPC style) | ⏳ | |
| Test `setParameter` / `getParameter` roundtrip | ⏳ | |
| Test message latency characteristics | ⏳ | |
| Bundle static assets into Rust binary | ⏳ | |

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

---

## Next Steps

1. Validate AU build in Logic Pro and GarageBand to fully close out Milestone 1
2. Begin Milestone 2: Set up React app scaffolding with Vite
3. Research wry integration patterns for desktop POC
