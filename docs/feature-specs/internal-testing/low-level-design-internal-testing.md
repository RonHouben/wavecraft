# Low-Level Design: Internal Testing (Milestone 12)

## Overview

This document defines the **testing strategy, methodology, and execution framework** for Milestone 12: Internal Testing. Unlike feature development milestones, this milestone produces no new code — it validates the complete Wavecraft SDK workflow before external beta testing.

The "design" here is the **testing architecture**: how tests are organized, executed, tracked, and how discovered issues are resolved.

---

## Goals

1. **Validate the complete SDK workflow** — Clone template → build plugin → use in DAW
2. **Verify documentation accuracy** — All guides match reality with no undocumented steps
3. **Confirm no regressions** — All features from M1-M11 still work correctly
4. **Surface edge cases** — Low buffer sizes, rapid automation, multi-instance scenarios
5. **Prepare for external testers** — Fix any issues that would frustrate beta testers

---

## Testing Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     INTERNAL TESTING FRAMEWORK                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────┐   ┌──────────────────┐   ┌─────────────────┐  │
│  │ PHASE 1          │   │ PHASE 2          │   │ PHASE 3         │  │
│  │ Automated        │   │ Manual Workflow  │   │ Documentation   │  │
│  │ Verification     │   │ Testing          │   │ Review          │  │
│  ├──────────────────┤   ├──────────────────┤   ├─────────────────┤  │
│  │ • cargo xtask    │   │ • Template clone │   │ • SDK Guide     │  │
│  │   check          │   │ • DAW integration│   │ • HLD review    │  │
│  │ • Unit tests     │   │ • Dev workflow   │   │ • Coding stds   │  │
│  │ • Lint passes    │   │ • Edge cases     │   │ • CI guide      │  │
│  │ • Visual tests   │   │ • Multi-instance │   │ • Link checker  │  │
│  └────────┬─────────┘   └────────┬─────────┘   └────────┬────────┘  │
│           │                      │                      │           │
│           └──────────────────────┼──────────────────────┘           │
│                                  ▼                                   │
│                    ┌─────────────────────────┐                       │
│                    │ PHASE 4                 │                       │
│                    │ Issue Resolution        │                       │
│                    ├─────────────────────────┤                       │
│                    │ • Categorize findings   │                       │
│                    │ • Prioritize by severity│                       │
│                    │ • Fix critical issues   │                       │
│                    │ • Document known issues │                       │
│                    └─────────────────────────┘                       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Automated Verification

### Objective
Ensure the existing automated test suite passes completely before manual testing begins.

### Scope

| Test Type | Tool | Command | Expected Result |
|-----------|------|---------|-----------------|
| Engine tests | Cargo | `cargo test --workspace` | 110+ tests pass |
| UI tests | Vitest | `npm run test` (in ui/) | 43+ tests pass |
| Rust linting | Clippy + fmt | `cargo xtask lint --engine` | No errors/warnings |
| TS linting | ESLint + Prettier | `cargo xtask lint --ui` | No errors |
| Combined check | xtask | `cargo xtask check` | All phases pass |

### Execution Order

```
1. cargo xtask check
   ├── Lint (Engine)
   │   ├── cargo fmt --check
   │   └── cargo clippy --workspace
   ├── Lint (UI)
   │   ├── npm run lint
   │   └── npm run format:check
   ├── Test (Engine)
   │   └── cargo test --workspace
   └── Test (UI)
       └── npm run test
```

### Pass Criteria
- Zero test failures
- Zero lint errors
- No warnings treated as errors

---

## Phase 2: Manual Workflow Testing

### Objective
Validate the complete SDK experience as a new developer would encounter it.

### 2.1 Fresh Clone Test

**Test Environment Setup:**
```bash
# Create isolated test directory (outside main repo)
mkdir -p /tmp/wavecraft-internal-test
cd /tmp/wavecraft-internal-test

# Clone template (simulating external developer)
git clone /Users/ronhouben/code/private/wavecraft/wavecraft-plugin-template test-plugin
cd test-plugin
```

**Validation Steps:**

| Step | Command | Success Criteria |
|------|---------|------------------|
| 1. Clone | `git clone ... test-plugin` | Directory created with all files |
| 2. Install deps | `cd ui && npm install` | No npm errors, node_modules created |
| 3. Build UI | `npm run build` | dist/ folder created with index.html |
| 4. Bundle plugin | `cd engine && cargo xtask bundle` | VST3 + CLAP in target/bundled/ |
| 5. Timing | N/A | Total time < 30 minutes |

**Artifacts to Verify:**
```
test-plugin/
├── ui/dist/                          # Built React app
│   ├── index.html
│   ├── assets/
│   └── ...
└── engine/target/bundled/
    ├── My Plugin.vst3/               # VST3 bundle
    │   └── Contents/
    │       ├── Info.plist
    │       └── MacOS/My Plugin
    └── My Plugin.clap                # CLAP bundle
```

### 2.2 DAW Integration Test

**Test Environment:**
- DAW: Ableton Live 12
- Platform: macOS (primary target)
- Sample rate: 44.1kHz
- Buffer sizes: 256 samples (default), then test 64 and 1024

**Test Matrix:**

| Test Case | Action | Expected Result |
|-----------|--------|-----------------|
| Plugin load | Add plugin to track | No errors, UI opens |
| Audio passthrough | Play audio through plugin | Clean output, no glitches |
| Parameter from UI | Adjust slider in plugin UI | DAW automation lane updates |
| Parameter from DAW | Draw automation | Plugin UI slider moves |
| Automation playback | Play with automation | Parameters animate smoothly |
| State save | Save Ableton project | No warnings |
| State restore | Close and reopen project | Plugin state preserved |
| Multi-instance | Add 3 instances | All work independently |
| Transport sync | Play/stop/loop | Plugin handles correctly |
| Bypass | Enable DAW bypass | Audio passes clean |

**Edge Cases:**

| Test Case | Action | Expected Result |
|-----------|--------|-----------------|
| Low buffer (64) | Change buffer to 64 samples | No dropouts |
| High buffer (4096) | Change buffer to 4096 | No issues |
| Rapid automation | Dense automation curves | UI keeps up, no crashes |
| Open/close UI | Toggle UI 10 times rapidly | No memory leaks, no crashes |

### 2.3 Developer Workflow Test

**Test Environment:**
```bash
cd test-plugin
cargo xtask dev   # Should start both servers
```

**Validation Steps:**

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1. Start dev | `cargo xtask dev` | Both servers start |
| 2. Open browser | Navigate to localhost:5173 | UI renders |
| 3. Connection | Check status indicator | Shows "Connected" |
| 4. Real data | Check meters | Show real audio levels (not mock) |
| 5. Parameter sync | Move slider | Engine receives update |
| 6. Hot reload | Edit App.tsx | Browser updates automatically |
| 7. Reconnection | Stop servers, restart | Browser reconnects |
| 8. Clean shutdown | Ctrl+C | Both servers stop cleanly |

---

## Phase 3: Documentation Review

### Objective
Verify all documentation is accurate, complete, and followable by newcomers.

### Documents to Review

| Document | Path | Focus Areas |
|----------|------|-------------|
| SDK Getting Started | `docs/guides/sdk-getting-started.md` | End-to-end accuracy |
| High-Level Design | `docs/architecture/high-level-design.md` | Architecture diagrams current |
| Coding Standards | `docs/architecture/coding-standards.md` | Standards followed in code |
| CI Pipeline | `docs/guides/ci-pipeline.md` | Instructions work |
| macOS Signing | `docs/guides/macos-signing.md` | Commands execute correctly |
| Visual Testing | `docs/guides/visual-testing.md` | Playwright workflow works |
| Template README | `wavecraft-plugin-template/README.md` | Instructions produce results |

### Review Methodology

For each document:

1. **Read as newcomer** — Pretend you've never seen the project
2. **Execute literally** — Run every command exactly as written
3. **Note issues:**
   - Missing prerequisites
   - Outdated commands
   - Unclear explanations
   - Broken links
   - Jargon without definition

### Link Validation

```bash
# Find all markdown links and verify they resolve
find docs/ -name "*.md" -exec grep -l "\[.*\](.*)" {} \;
# Manual: Check each relative link resolves
```

---

## Phase 4: Issue Resolution

### Issue Categorization

| Severity | Definition | Resolution |
|----------|------------|------------|
| **Critical** | Blocks SDK usage (build fails, plugin crashes) | Must fix before M12 complete |
| **High** | Significant UX issue (confusing docs, silent failures) | Should fix before M12 complete |
| **Medium** | Minor friction (unclear wording, missing example) | Fix if time permits |
| **Low** | Polish (typos, formatting) | Defer to M13 or backlog |

### Issue Tracking Format

Issues found during testing are documented in the test plan with this format:

```markdown
### Issue: [SHORT_TITLE]

**Severity:** Critical | High | Medium | Low
**Found in:** Phase X, Step Y
**Symptom:** What was observed
**Expected:** What should have happened
**Root Cause:** (if known)
**Resolution:** Fixed in commit XXX | Deferred to backlog | Known limitation
```

### Resolution Workflow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ Issue Found │────▶│ Categorize  │────▶│ Prioritize  │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                    ┌──────────────────────────┼──────────────────────────┐
                    ▼                          ▼                          ▼
            ┌───────────────┐          ┌───────────────┐          ┌───────────────┐
            │ Critical/High │          │ Medium        │          │ Low           │
            │               │          │               │          │               │
            │ Fix now       │          │ Fix if time   │          │ Add to        │
            │ Re-test       │          │ Or document   │          │ backlog       │
            └───────┬───────┘          └───────────────┘          └───────────────┘
                    │
                    ▼
            ┌───────────────┐
            │ Re-run Phase  │
            │ to confirm    │
            └───────────────┘
```

---

## Test Environment Requirements

### Hardware
- Mac (Apple Silicon or Intel)
- Minimum 8GB RAM
- ~5GB disk space for builds and DAW

### Software

| Software | Version | Purpose |
|----------|---------|---------|
| macOS | 12.0+ | Primary OS |
| Rust | 1.75+ | Engine compilation |
| Node.js | 18+ | UI build |
| Ableton Live | 12.x | DAW testing |
| Chromium | Latest | Visual testing (via Playwright) |

### Fresh Environment Simulation

To simulate a new developer environment:

1. **Clear Cargo cache** (optional, extreme): `rm -rf ~/.cargo/registry`
2. **Use temp directory**: Test in `/tmp/` not inside main repo
3. **No path dependencies**: Template must not reference main repo

---

## Success Criteria

### Phase 1 (Automated)
- [ ] `cargo xtask check` passes with 0 failures
- [ ] All 110+ engine tests pass
- [ ] All 43+ UI tests pass

### Phase 2 (Manual Workflow)
- [ ] Fresh clone → bundled plugin in < 30 minutes
- [ ] Plugin loads and functions correctly in Ableton Live
- [ ] `cargo xtask dev` workflow enables efficient iteration
- [ ] No crashes during edge case testing

### Phase 3 (Documentation)
- [ ] All 7 guides reviewed and accurate
- [ ] All commands execute successfully
- [ ] No broken links
- [ ] No undocumented steps required

### Phase 4 (Issues)
- [ ] All Critical issues fixed
- [ ] All High issues fixed or documented
- [ ] Issue resolution documented in test plan

### Overall Exit Criteria
- [ ] All phases pass
- [ ] Version bumped to 0.6.3
- [ ] PO sign-off for external beta readiness

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Template has hidden monorepo dependencies | High — external devs can't build | Test template in isolated /tmp/ directory |
| Documentation drift from code | Medium — confused developers | Execute all docs literally during review |
| DAW-specific issues undiscovered | Medium — works in test, fails in production | Test with real Ableton projects |
| Edge cases cause intermittent crashes | High — frustrating for users | Systematic edge case matrix |
| Time overrun fixing issues | Medium — delays M13 | Strict severity triage, defer Low issues |

---

## Deliverables

| Deliverable | Owner | Description |
|-------------|-------|-------------|
| `test-plan.md` | Tester | Detailed test cases with pass/fail status |
| Bug fixes (if any) | Coder | Commits fixing Critical/High issues |
| Documentation updates | Coder | Fixes to guides based on findings |
| `implementation-progress.md` | Tester | Progress tracking |
| Version 0.6.3 | Coder | Final version bump after all fixes |

---

## Relationship to Other Milestones

```
┌─────────────────────────────────────────────────────────────────────┐
│                           MILESTONE FLOW                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────┐                                                │
│  │ M11: Code Quality│  ← Completed (v0.6.2)                          │
│  │ & OSS Prep       │    Logger, CI optimization, LICENSE, etc.     │
│  └────────┬─────────┘                                                │
│           │                                                          │
│           ▼                                                          │
│  ┌──────────────────┐                                                │
│  │ M12: Internal    │  ← THIS MILESTONE (v0.6.3)                     │
│  │ Testing          │    Validate everything works end-to-end        │
│  └────────┬─────────┘                                                │
│           │                                                          │
│           ▼                                                          │
│  ┌──────────────────┐                                                │
│  │ M13: User Testing│  ← Next milestone (v0.7.0)                     │
│  │ (Beta)           │    External testers provide feedback           │
│  └────────┬─────────┘                                                │
│           │                                                          │
│           ▼                                                          │
│  ┌──────────────────┐                                                │
│  │ M14: V1.0 Release│  ← Final milestone                             │
│  └──────────────────┘                                                │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Appendix A: Test Checklist Summary

Quick reference checklist for the Tester agent:

### Phase 1: Automated
```
[ ] cargo xtask check --fix (if needed)
[ ] cargo xtask check (final verification)
```

### Phase 2: Manual Workflow
```
[ ] Fresh clone to /tmp/
[ ] npm install (no errors)
[ ] npm run build (dist/ created)
[ ] cargo xtask bundle (VST3 + CLAP created)
[ ] Plugin loads in Ableton
[ ] Parameter sync works both directions
[ ] State save/restore works
[ ] Multi-instance works
[ ] cargo xtask dev works
[ ] WebSocket connection works
[ ] Hot reload works
[ ] Edge cases (low buffer, rapid automation)
```

### Phase 3: Documentation
```
[ ] SDK Getting Started guide
[ ] High-Level Design
[ ] Coding Standards
[ ] CI Pipeline guide
[ ] macOS Signing guide
[ ] Visual Testing guide
[ ] Template README
[ ] No broken links
```

### Phase 4: Resolution
```
[ ] All Critical issues fixed
[ ] All High issues fixed
[ ] Documentation updated
[ ] Version bumped to 0.6.3
```

---

## Appendix B: Template Independence Verification

Specific checks for template isolation:

```bash
# 1. Clone template to isolated location
cd /tmp
rm -rf wavecraft-internal-test
git clone /path/to/wavecraft/wavecraft-plugin-template test-plugin

# 2. Verify no path dependencies in Cargo.toml
grep "path = " test-plugin/engine/Cargo.toml
# Should only show relative paths within template, not "../../../" to main repo

# 3. Build without main repo
cd test-plugin/ui && npm install && npm run build && cd ..
cd engine && cargo xtask bundle --release

# 4. Verify bundles exist
ls engine/target/bundled/
# Should show: "My Plugin.clap" and "My Plugin.vst3/"
```

If any step fails, there's a hidden dependency on the main monorepo that must be fixed.
