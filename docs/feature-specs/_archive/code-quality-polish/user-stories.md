# User Stories: Code Quality & OSS Prep (Milestone 11)

## Overview

This milestone prepares Wavecraft for open-source release by addressing code quality issues, adding proper logging infrastructure, and creating the necessary documentation and templates for external contributors.

**Problem Statement:**
Wavecraft has achieved feature completeness through Milestone 10, but several polish items remain before public release:
- UI uses `console.log` scattered throughout (unprofessional, hard to filter)
- Engine lacks structured logging (makes debugging difficult)
- Horizontal scroll wiggle is a visible UX annoyance
- CI builds are slower than necessary (no caching)
- Missing contributor documentation and templates

**Target Audience:**
1. **Plugin developers** using the SDK — need clear logs for debugging
2. **Contributors** — need guidelines and templates
3. **End users** — expect polished, bug-free experience

---

## Version

**Target Version:** `0.6.1` (patch)

**Rationale:** This milestone contains polish and infrastructure improvements without new user-facing features. Per coding standards, polish items warrant a patch version bump.

---

## User Story 1: Structured UI Logging

**As a** plugin developer using the Wavecraft SDK  
**I want** structured logging in the UI with severity levels  
**So that** I can filter debug output and troubleshoot issues efficiently

### Acceptance Criteria
- [ ] `Logger` class with methods: `debug()`, `info()`, `warn()`, `error()`
- [ ] Each log level has distinct console styling (colors/prefixes)
- [ ] Log level can be configured (e.g., suppress debug in production)
- [ ] All existing `console.log` calls replaced with appropriate Logger calls
- [ ] Timestamps included in log output
- [ ] Component/module context included (e.g., `[IpcBridge]`, `[Meter]`)

### Notes
- Follow class-based architecture per coding standards
- Consider `LOG_LEVEL` environment variable for configuration
- Don't break hot reload — logger should work in dev and production

---

## User Story 2: Structured Engine Logging

**As a** plugin developer using the Wavecraft SDK  
**I want** structured logging in the Rust engine with the `tracing` crate  
**So that** I can debug audio processing and IPC issues with proper context

### Acceptance Criteria
- [ ] `tracing` crate added to workspace dependencies
- [ ] `tracing-subscriber` configured for console output
- [ ] Log levels: `trace`, `debug`, `info`, `warn`, `error`
- [ ] Key modules instrumented with appropriate log calls:
  - IPC handler (request/response logging)
  - Parameter updates
  - WebSocket connections (connect/disconnect/errors)
  - Meter data streaming
- [ ] Existing `println!` and `eprintln!` replaced with tracing macros
- [ ] Log output includes timestamps and module paths

### Notes
- Real-time audio code should NOT log (would cause glitches)
- Consider `RUST_LOG` environment variable for filtering
- Ensure logs don't spam during normal operation (use appropriate levels)

---

## User Story 3: Horizontal Scroll Fix

**As a** music producer using a Wavecraft plugin  
**I want** the UI to not "wiggle" horizontally when scrolling  
**So that** my experience feels polished and professional

### Acceptance Criteria
- [ ] Horizontal scroll/wiggle eliminated on the main UI
- [ ] Vertical scroll (if any) still works normally
- [ ] Fix works in both WKWebView (plugin) and browser (dev mode)
- [ ] No visual regressions to existing layout

### Notes
- This is likely a CSS overflow issue
- Quick fix — should take ~30 minutes
- Test in Ableton Live after fix

---

## User Story 4: CI Cache Optimization

**As a** developer contributing to Wavecraft  
**I want** faster CI builds through proper caching  
**So that** I get feedback on my PRs quickly

### Acceptance Criteria
- [ ] Rust build artifacts cached between CI runs
- [ ] `node_modules` cached for UI jobs
- [ ] Cache keys include lock file hashes for proper invalidation
- [ ] Measurable improvement in CI run time (target: 50% reduction on cache hit)
- [ ] Cache doesn't cause stale build issues

### Notes
- Use GitHub Actions cache action
- Consider separate caches for different job types
- Document cache strategy in CI pipeline guide

---

## User Story 5: Open Source License Review

**As a** potential user or contributor to Wavecraft  
**I want** clear licensing information  
**So that** I understand how I can use and contribute to the project

### Acceptance Criteria
- [ ] All direct dependencies reviewed for license compatibility
- [ ] LICENSE file present in repository root (if not already)
- [ ] License type clearly stated in README
- [ ] No GPL-incompatible dependencies in the public API
- [ ] Third-party licenses documented if required

### Notes
- Wavecraft likely MIT or Apache-2.0
- Check nih-plug, wry, and other key dependencies
- Template project should inherit compatible license

---

## User Story 6: Contributing Guidelines

**As a** developer who wants to contribute to Wavecraft  
**I want** clear contribution guidelines  
**So that** I understand the process and expectations for submitting changes

### Acceptance Criteria
- [ ] `CONTRIBUTING.md` file in repository root
- [ ] Development setup instructions
- [ ] Code style and formatting requirements
- [ ] Pull request process documented
- [ ] Testing requirements documented
- [ ] Link to coding standards

### Notes
- Reference existing docs (coding-standards.md, CI pipeline guide)
- Keep it concise — link to detailed docs where appropriate
- Include Code of Conduct reference

---

## User Story 7: GitHub Issue Templates

**As a** user who found a bug or wants a feature  
**I want** structured issue templates  
**So that** I can provide the right information for the maintainers

### Acceptance Criteria
- [ ] Bug report template with:
  - OS and DAW version fields
  - Steps to reproduce
  - Expected vs actual behavior
  - Plugin version
- [ ] Feature request template with:
  - Problem description
  - Proposed solution
  - Alternatives considered
- [ ] Pull request template with:
  - Description of changes
  - Testing performed
  - Checklist (lint, tests, docs)

### Notes
- Use GitHub's `.github/ISSUE_TEMPLATE/` directory
- Keep templates focused — don't ask for too much

---

## User Story 8: README Polish

**As a** developer discovering Wavecraft for the first time  
**I want** a polished, informative README  
**So that** I understand what Wavecraft is and how to get started

### Acceptance Criteria
- [ ] Clear project description (what is Wavecraft?)
- [ ] Key features highlighted
- [ ] Quick start / installation instructions
- [ ] Links to documentation (guides, API docs)
- [ ] Status badges (CI, version)
- [ ] Screenshots or demo GIF of plugin UI
- [ ] License information
- [ ] Contributing link

### Notes
- First impression matters for open source adoption
- Keep above-the-fold content concise and compelling
- Technical details can link to dedicated docs

---

## Priority Order

| Priority | Story | Rationale |
|----------|-------|-----------|
| 1 | Horizontal Scroll Fix | Quick win, visible polish |
| 2 | README Polish | First thing users see |
| 3 | Contributing Guidelines | Enables contributions |
| 4 | GitHub Issue Templates | Structured feedback |
| 5 | License Review | Legal clarity |
| 6 | UI Logger | Developer experience |
| 7 | Engine Logging | Developer experience |
| 8 | CI Cache Optimization | Nice-to-have, not blocking |

---

## Out of Scope

The following items are explicitly **not** part of this milestone:
- New features or architectural changes
- SDK publication to crates.io (future milestone)
- CLI scaffolding tool (future milestone)
- Additional example plugins (future milestone)
- Windows/Linux support expansion

---

## Success Metrics

- [ ] All 8 user stories completed
- [ ] Zero `console.log` calls in production UI code
- [ ] Zero `println!` calls in production engine code
- [ ] CI build time reduced by ≥30% on cache hits
- [ ] README provides clear path to getting started
- [ ] External contributor can submit a PR following the guidelines
