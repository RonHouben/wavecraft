# Wavecraft Backlog

This document contains ideas and tasks for future consideration that haven't been prioritized into a milestone yet.

**Related:** [Roadmap](roadmap.md) — committed milestones with timelines

---

## How Items Move to the Roadmap

When planning a new milestone, the Product Owner reviews this backlog and promotes items to the roadmap with a target milestone. Promoted items get proper status tracking and acceptance criteria.

---

## SDK Publication (Future)

| Item | Notes |
|------|-------|
| CLI scaffolding tool | `cargo wavecraft new my-plugin` — Scaffolds new plugin project from template. Required before crates.io publication for good developer UX. |
| End-to-end SDK testing in Ableton | Create plugin from template as end-user developer, test full workflow in Ableton Live. Validates SDK is production-ready. |
| Publish to crates.io | Publish `wavecraft-*` crates. Requires CLI and Ableton validation first. |
| Documentation site | Consider `wavecraft.dev` domain (available at €10.89/yr). Host API docs, guides, examples. |

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
| ~~CI pipeline cache optimization~~ | ✅ **Addressed** in v0.6.2 — `prepare-engine` job now pre-compiles test binaries with `cargo test --no-run`, eliminating rebuild in `test-engine` (~3-5 min savings per PR). |
| ~~GitHub artifacts storage alternative~~ | ✅ **Partially addressed** in v0.6.2 — Implemented tiered retention (7 days main / 90 days tags), reducing storage by ~75-80%. External storage (R2/S3) deferred unless limits become critical again. |

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

## AU Issues (Deprioritized)

| Item | Notes |
|------|-------|
| Investigate AU custom UI issue | clap-wrapper shows generic view instead of React UI; root cause TBD. **Deprioritized** — focusing on VST3/CLAP for now. |

---

## Project Rename

**Status:** ✅ Complete — [Milestone 9](roadmap.md#milestone-9-project-rename-vstkit--wavecraft) implemented, PR #17 pending merge

| Item | Notes |
|------|-------|
| ~~Rename VstKit → Wavecraft~~ | ✅ **Complete** — Version 0.5.0, PR #17 pending merge |
| Request GitHub `wavecraft` username | GitHub username `WaveCraft` is held by an inactive user (no activity since 2020, 1 repo about electronics). Submit request via [GitHub's Name Squatting Policy](https://docs.github.com/en/site-policy/other-site-policies/github-username-policy) after project is stable/public. Current repo: `RonHouben/wavecraft`. |
| Register `wavecraft.dev` domain | Available at €10.89/yr on Namecheap. Optional — register when ready for public docs site. |

**Availability (Verified 2026-02-02):**
- [x] **crates.io**: `wavecraft`, `wavecraft-core`, `wavecraft-dsp`, etc. — ✅ Available
- [x] **npm**: `@wavecraft/*` namespace — ✅ Available
- [x] **Domain**: `wavecraft.dev` — ✅ Available (€10.89/yr)
- [ ] **GitHub**: `wavecraft` — ⚠️ Taken by inactive user (request later)

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
| 2026-02-03 | **CI/CD Optimization**: Added GitHub artifacts storage alternative item — investigate solutions to avoid pipeline failures from artifact storage limits (compress, external storage, release-only uploads, cleanup workflow). |
| 2026-02-02 | **Backlog grooming**: Added SDK Publication section (CLI scaffolding, end-to-end testing, crates.io, docs site). Deprioritized AU custom UI investigation. Updated Project Rename status to complete (M9 done, PR #17 pending). |
| 2026-02-02 | **Project Rename updated**: Availability verified — Wavecraft available on crates.io, npm, domain. GitHub username taken by inactive user; added task to request via Name Squatting Policy. Main rename work moved to Milestone 9. |
| 2026-02-01 | **Code Quality section added**: Logger class (UI) and `log`/`tracing` crate (Engine) to replace direct console output |
| 2026-02-01 | **UI Polish section added**: Horizontal scroll wiggle issue — block elastic scrolling on macOS |
| 2026-02-01 | **Backlog created**: Split from roadmap Milestone 8 to separate committed work from future ideas. Items moved: CI cache optimization, performance profiling, platform support, DAW compatibility, AU issues, Apple Developer-dependent items. |
