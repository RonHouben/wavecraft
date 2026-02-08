# Wavecraft Backlog

This document contains ideas and tasks for future consideration that haven't been prioritized into a milestone yet.

**Related:** [Roadmap](roadmap.md) — committed milestones with timelines

---

## How Items Move to the Roadmap

When planning a new milestone, the Product Owner reviews this backlog and promotes items to the roadmap with a target milestone. Promoted items get proper status tracking and acceptance criteria.

---

## Developer Experience

| Item | Notes |
|------|-------|
| Remove audio signal mocking to UI | Remove all audio signal mocking infrastructure since it's not currently needed. Reduces complexity and maintenance burden. Can be re-implemented later if real audio visualization is required. YAGNI principle — keep the codebase lean. |
| Browser audio input via WASM | Enable testing UI with real audio input (mic, files, test tones) in browser dev mode. Tiered architecture: Mock DSP (JS) for fast HMR, optional WASM DSP for integration testing. Rust remains parameter source of truth. See [high-level design](feature-specs/audio-input-via-wasm/high-level-design.md). |

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

## nih-plug Independence Strategy

**Context:** nih-plug maintenance has slowed significantly (last commit: Sep 2025, 84 open issues, single maintainer risk). Meanwhile, Steinberg released VST3 under MIT license (Oct 2025), removing legal barriers to custom implementations. This epic ensures Wavecraft isn't held hostage by external dependency maintenance.

**Strategy:** Prepare for independence, don't abandon what works.

| Item | Priority | Notes |
|------|----------|-------|
| Abstraction Layer for Plugin Host | High | Create `wavecraft-plugin-api` trait abstraction to decouple Wavecraft from nih-plug internals. Enables future backend swapping without breaking user code. **Effort:** 2 weeks |
| nih-plug Fork Contingency | Medium (On-Trigger) | **Trigger:** Critical bug affecting Wavecraft unresolved upstream >4 weeks. Fork to `wavecraft-org/nih-plug`, apply targeted fixes, contribute upstream. **Effort:** As needed |
| Native VST3 Spike | Medium | Evaluate MIT-licensed VST3 SDK + bindgen approach. Build minimal PoC (audio passthrough, no GUI), test in Ableton Live. **Timeline:** Q2 2026. **Effort:** 2-4 weeks spike |
| Monitor nih-plug Health | Low (Ongoing) | Monthly check: commit activity, issue response times, macOS/Ableton-critical issues. Update strategy if activity resumes or stops entirely. |

**Known nih-plug Issues Affecting Wavecraft:**
- macOS window resize bugs (unresolved)
- Retina display issues (unresolved)
- DAW crash reports (multiple hosts)

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

| Item | Notes |
|------|-------|
| Request GitHub `wavecraft` username | GitHub username `WaveCraft` is held by an inactive user (no activity since 2020, 1 repo about electronics). Submit request via [GitHub's Name Squatting Policy](https://docs.github.com/en/site-policy/other-site-policies/github-username-policy) after project is stable/public. Current repo: `RonHouben/wavecraft`. |
| Register `wavecraft.dev` domain | Available at €10.89/yr on Namecheap. Optional — register when ready for public docs site. |

**Availability (Verified 2026-02-02):**
- [x] **crates.io**: `wavecraft`, `wavecraft-core`, `wavecraft-dsp`, etc. — ✅ Available
- [x] **npm**: `@wavecraft/*` namespace — ✅ Available
- [x] **Domain**: `wavecraft.dev` — ✅ Available (€10.89/yr)
- [ ] **GitHub**: `wavecraft` — ⚠️ Taken by inactive user (request later)

---

## Changelog

| Date | Update |
|------|--------|
| 2026-02-08 | **Backlog addition:** Remove audio signal mocking to UI — reduces complexity and technical debt by removing unused infrastructure. YAGNI principle applied. |
| 2026-02-08 | **Item promoted to Milestone 15**: Comprehensive workspace cleanup (`cargo xtask clean` extension) moved from backlog to roadmap as new Milestone 15 (Developer Tooling Polish). Target version 0.8.6. User stories created at `docs/feature-specs/workspace-cleanup/user-stories.md`. |
| 2026-02-08 | **Items promoted to Milestone 14**: CLI `-v`/`--version` flag and CLI `update` command moved from backlog to new Milestone 14 (CLI Enhancements). Target version 0.8.1. |
| 2026-02-07 | **Backlog addition:** Add CLI `update` command to update all project dependencies and packages (Rust + npm) in a plugin workspace. |
| 2026-02-07 | **Backlog cleanup:** Removed the SDK Publication chapter. |
| 2026-02-07 | **Backlog cleanup:** Removed the Apple Developer Account deferred chapter. |
| 2026-02-07 | **Backlog cleanup:** Removed completed SDK publication items (CLI scaffolding, crates.io publish). |
| 2026-02-07 | **Backlog cleanup:** Removed completed items to keep the backlog focused on pending work. |
| 2026-02-07 | **Backlog addition:** Add CLI `-v`/`--version` flag so users can easily verify installed version. |
| 2026-02-06 | **nih-plug Independence Strategy added**: New epic for defensive architecture — abstraction layer (High), fork contingency (Medium, on-trigger), native VST3 spike (Medium, Q2 2026), health monitoring (Low, ongoing). Motivated by nih-plug maintenance slowdown and VST3 MIT license change. |
| 2026-02-03 | **CI/CD Optimization complete**: Marked all three CI items as complete — cache optimization, tiered artifact retention, and new `cargo xtask check` command for fast local validation (~52s). |
| 2026-02-03 | **Code Quality complete**: Marked Logger class (UI) and `tracing` crate (Engine) as complete — both implemented in v0.6.1. |
| 2026-02-03 | **UI Polish complete**: Marked horizontal scroll fix as complete — implemented in v0.6.1. |
| 2026-02-03 | **CI/CD Optimization**: Added GitHub artifacts storage alternative item — investigate solutions to avoid pipeline failures from artifact storage limits (compress, external storage, release-only uploads, cleanup workflow). |
| 2026-02-02 | **Backlog grooming**: Added SDK Publication section (CLI scaffolding, end-to-end testing, crates.io, docs site). Deprioritized AU custom UI investigation. Updated Project Rename status to complete (M9 done, PR #17 pending). |
| 2026-02-02 | **Project Rename updated**: Availability verified — Wavecraft available on crates.io, npm, domain. GitHub username taken by inactive user; added task to request via Name Squatting Policy. Main rename work moved to Milestone 9. |
| 2026-02-01 | **Code Quality section added**: Logger class (UI) and `log`/`tracing` crate (Engine) to replace direct console output |
| 2026-02-01 | **UI Polish section added**: Horizontal scroll wiggle issue — block elastic scrolling on macOS |
| 2026-02-01 | **Backlog created**: Split from roadmap Milestone 8 to separate committed work from future ideas. Items moved: CI cache optimization, performance profiling, platform support, DAW compatibility, AU issues, Apple Developer-dependent items. |
