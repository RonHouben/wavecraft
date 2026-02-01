# VstKit Backlog

This document contains ideas and tasks for future consideration that haven't been prioritized into a milestone yet.

**Related:** [Roadmap](roadmap.md) — committed milestones with timelines

---

## How Items Move to the Roadmap

When planning a new milestone, the Product Owner reviews this backlog and promotes items to the roadmap with a target milestone. Promoted items get proper status tracking and acceptance criteria.

---

## UI Polish

| Item | Notes |
|------|-------|
| Disable horizontal scroll/wiggle | macOS elastic scrolling causes visual "wiggle" on horizontal scroll. Block with `overflow-x: hidden` on body/root. Low effort, improves feel. |

---

## Code Quality

| Item | Notes |
|------|-------|
| Replace console.log with Logger class (UI) | Create a centralized Logger class instead of using `console.log` directly. Enables log levels (debug/info/warn/error), consistent formatting, and easier filtering/disabling in production builds. |
| Use `log` or `tracing` crate (Engine) | Replace direct `println!`/`eprintln!` in runtime crates with the `log` facade or `tracing`. Enables log levels, consistent formatting, and runtime filtering. Note: `xtask` CLI commands can keep `println!` for user-facing output. |

---

## CI/CD Optimization

| Item | Notes |
|------|-------|
| CI pipeline cache optimization | Test Engine job rebuilds instead of using cache from Check Engine (different profiles: check vs test). Consider adding `cargo test --no-run` to prepare-engine job or combining check + test jobs. |

---

## Performance

| Item | Notes |
|------|-------|
| Performance profiling (low buffer sizes: 32/64 samples) | Stress test DSP at extreme settings |
| CPU stress testing | Multi-instance load testing |
| Memory usage optimization | Profile and reduce allocations |

---

## Platform Support

| Item | Notes |
|------|-------|
| WebView2 integration (Windows) | Deprioritized — macOS + Ableton is primary target |
| Linux packaging (AppImage/Flatpak) | Deprioritized |

---

## DAW Compatibility

| Item | Notes |
|------|-------|
| Logic Pro (macOS, AU) | Secondary (nice-to-have) |
| GarageBand (macOS, AU) | Secondary (nice-to-have) |
| Reaper (all platforms) | Deprioritized |
| Cubase | Deprioritized |
| FL Studio | Deprioritized |

---

## AU Issues

| Item | Notes |
|------|-------|
| Investigate AU custom UI issue | clap-wrapper shows generic view instead of React UI; root cause TBD |

---

## Project Rename

| Item | Notes |
|------|-------|
| Rename VstKit → Wavecraft | Avoid "VST" trademark concerns for open-source release. Scope: repo name, crate names, npm packages, documentation, UI references, README. |

**Pre-rename availability checks:**
- [ ] **GitHub**: `wavecraft` org or repo name available
- [ ] **crates.io**: `wavecraft`, `wavecraft-core`, `wavecraft-dsp` available
- [ ] **npm**: `@wavecraft/*` namespace available
- [ ] **Domain**: `wavecraft.dev` or `wavecraftaudio.dev` available

---

## Deferred (Requires Apple Developer Account)

These items are ready to implement but require an Apple Developer Program membership:

| Item | Notes |
|------|-------|
| Developer ID signing validation | Phase 3 of macOS hardening |
| Notarization submission | Phase 4 of macOS hardening |
| Gatekeeper testing | Verify signed/notarized plugin works for end users |
| Signed release CI/CD pipeline | Phase 5b — automated signed releases |

---

## Changelog

| Date | Update |
|------|--------|
| 2026-02-01 | **Code Quality section added**: Logger class (UI) and `log`/`tracing` crate (Engine) to replace direct console output |
| 2026-02-01 | **UI Polish section added**: Horizontal scroll wiggle issue — block elastic scrolling on macOS |
| 2026-02-01 | **Backlog created**: Split from roadmap Milestone 8 to separate committed work from future ideas. Items moved: CI cache optimization, performance profiling, platform support, DAW compatibility, AU issues, Apple Developer-dependent items. |
