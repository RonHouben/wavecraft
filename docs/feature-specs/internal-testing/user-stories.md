# User Stories: Internal Testing (Milestone 12)

## Overview

This milestone focuses on **comprehensive internal validation** of the complete Wavecraft SDK workflow before external beta testing. The goal is to catch issues that would frustrate external testers — missing documentation, unclear instructions, broken workflows, or regressions from previous milestones.

Think of this as a "dress rehearsal" before inviting real plugin developers to try Wavecraft.

---

## Version

**Target Version:** `0.6.3` (patch bump from `0.6.2`)

**Rationale:** This milestone is focused on bug fixes and polish discovered during internal testing, not new features. A patch bump is appropriate. Any critical issues found will be fixed before this version is tagged.

---

## Context: Who Are We Testing For?

The external beta testers (Milestone 13) will be:

1. **Rust developers** new to audio plugins — They know Rust but not nih-plug or audio DSP
2. **Audio plugin developers** new to Rust — They've built plugins before (JUCE, iPlug2) but are learning Rust
3. **React developers** interested in audio — They know React well but not audio or Rust

Internal testing should simulate these perspectives to catch documentation gaps and UX issues.

---

## User Story 1: Fresh Clone Experience

**As a** new developer discovering Wavecraft  
**I want** to clone the template and build a working plugin within 30 minutes  
**So that** I can quickly validate whether Wavecraft is right for my project

### Acceptance Criteria
- [ ] Scaffold new project using `wavecraft create test-plugin --vendor "Test Company"` (outside main repo)
- [ ] Follow the README.md instructions exactly — no undocumented steps required
- [ ] `cargo xtask bundle --release` completes without errors
- [ ] Plugin files appear in `target/bundled/` (VST3 + CLAP)
- [ ] Total time from clone to bundled plugin is under 30 minutes (excluding download time)

### Notes
- This simulates the "first 30 minutes" experience that determines if developers continue
- Pay attention to any point where you need to "just know" something not in the docs
- Document any error messages that require Googling

---

## User Story 2: DAW Integration Validation

**As a** music producer testing a Wavecraft plugin  
**I want** the plugin to behave like any professional plugin in my DAW  
**So that** I can use it in real productions without worrying about stability

### Acceptance Criteria
- [ ] Plugin loads in Ableton Live 12 without errors or warnings
- [ ] Plugin UI opens and renders correctly
- [ ] Audio passes through without glitches or dropouts
- [ ] Parameter changes from UI update DAW automation lanes
- [ ] DAW automation playback updates plugin UI in real-time
- [ ] Save project → Close project → Reopen project → Plugin state is restored correctly
- [ ] Multiple instances of the plugin can run simultaneously
- [ ] Plugin handles DAW transport (play/stop/loop) without issues

### Notes
- Test with a real project, not just an empty session
- Try parameter automation during playback
- Primary target: Ableton Live on macOS

---

## User Story 3: Developer Workflow Validation

**As a** plugin developer iterating on my plugin  
**I want** a fast edit-preview-test cycle  
**So that** I can develop efficiently without constant full rebuilds

### Acceptance Criteria
- [ ] `cargo xtask dev` starts WebSocket server + Vite dev server successfully
- [ ] UI changes hot-reload in browser (localhost:5173)
- [ ] UI correctly connects to Rust engine via WebSocket
- [ ] Real parameter values and meter data display (not mock data)
- [ ] Connection status shows "Connected" state
- [ ] Browser refresh reconnects automatically
- [ ] Ctrl+C cleanly stops both servers

### Notes
- This is the primary development workflow for UI iteration
- Should work seamlessly with no extra configuration

---

## User Story 4: Documentation Accuracy

**As a** new Wavecraft developer  
**I want** documentation that matches the actual code and workflows  
**So that** I don't waste time debugging discrepancies between docs and reality

### Acceptance Criteria
- [ ] **SDK Getting Started guide** (`docs/guides/sdk-getting-started.md`) is accurate end-to-end
- [ ] **High-level design** (`docs/architecture/high-level-design.md`) reflects current architecture
- [ ] **Coding standards** (`docs/architecture/coding-standards.md`) are followed by actual code
- [ ] **CI pipeline guide** (`docs/guides/ci-pipeline.md`) instructions work correctly
- [ ] **macOS signing guide** (`docs/guides/macos-signing.md`) steps execute successfully
- [ ] **Visual testing guide** (`docs/guides/visual-testing.md`) workflow is reproducible
- [ ] Template README.md instructions produce expected results
- [ ] No broken links in documentation files

### Notes
- Read each guide as if you've never seen the project before
- Highlight any jargon that isn't explained
- Note any prerequisites that aren't mentioned

---

## User Story 5: DSL Usability

**As a** developer creating a custom processor  
**I want** the declarative DSL to be intuitive and well-documented  
**So that** I can define parameters without reading the macro source code

### Acceptance Criteria
- [ ] `#[derive(ProcessorParams)]` works as documented
- [ ] `#[param(...)]` attributes (range, default, unit, group) work correctly
- [ ] Compile errors from incorrect usage provide helpful messages
- [ ] `wavecraft_processor!` macro creates valid processor wrappers
- [ ] `wavecraft_plugin!` macro generates working plugin with custom processor
- [ ] Template project demonstrates all DSL features
- [ ] Documentation covers all parameter attribute options

### Notes
- Try creating a new parameter type not in the template
- Try intentionally making mistakes to see error quality

---

## User Story 6: UI Customization

**As a** plugin developer with specific branding requirements  
**I want** to easily customize the plugin UI appearance  
**So that** my plugin has a unique visual identity

### Acceptance Criteria
- [ ] TailwindCSS custom theme in template is clearly documented
- [ ] Changing colors in `tailwind.config.js` applies throughout UI
- [ ] Adding new components works with existing styling patterns
- [ ] Meter component styling is customizable
- [ ] ParameterSlider styling is customizable
- [ ] Hot reload works for CSS/styling changes

### Notes
- Try changing the accent color and verify it propagates
- Try adding a custom component with the plugin's styling

---

## User Story 7: Build System Reliability

**As a** developer building plugins for distribution  
**I want** the build system to reliably produce correct artifacts  
**So that** I can trust that my builds will work for end users

### Acceptance Criteria
- [ ] `cargo xtask bundle --release` produces both VST3 and CLAP formats
- [ ] `cargo xtask bundle --debug` works for development builds
- [ ] `cargo xtask sign` succeeds with ad-hoc signing
- [ ] Built plugins include embedded UI assets (not external files)
- [ ] Plugin metadata (name, vendor, version) is correct in DAW scanner
- [ ] No warnings about missing entitlements in macOS Console

### Notes
- Check the actual file sizes — embedded assets should be reasonable
- Verify plugin info in Ableton's plugin manager

---

## User Story 8: Regression Testing

**As a** Wavecraft maintainer  
**I want** to verify no regressions exist from previous milestones  
**So that** external testers don't encounter bugs we already fixed

### Acceptance Criteria
- [ ] `cargo xtask check` passes completely (lint + tests)
- [ ] All 110+ engine tests pass
- [ ] All 43+ UI tests pass
- [ ] Playwright visual testing works (`cargo xtask dev` + MCP tools)
- [ ] WebSocket IPC bridge connects and syncs data
- [ ] Code signing workflow completes without errors
- [ ] Desktop standalone app (`cargo run -p standalone`) works

### Notes
- Run the full test suite before and after any fixes
- Any test failures should be investigated and fixed

---

## User Story 9: Edge Cases & Stress Testing

**As a** plugin developer shipping to diverse users  
**I want** my plugin to handle edge cases gracefully  
**So that** end users don't experience crashes or undefined behavior

### Acceptance Criteria
- [ ] Low buffer sizes (32, 64 samples) work without audio glitches
- [ ] High buffer sizes (4096 samples) work correctly
- [ ] Rapid parameter automation doesn't cause UI lag or crashes
- [ ] Very fast tempo changes don't cause issues
- [ ] Opening/closing plugin UI repeatedly doesn't leak memory or handles
- [ ] Plugin survives DAW bypass/enable toggle
- [ ] Plugin handles sample rate changes (if DAW supports mid-session changes)

### Notes
- These tests may surface issues that are acceptable trade-offs
- Document any known limitations for external testers

---

## User Story 10: Template Independence

**As a** developer using the Wavecraft template  
**I want** the template to work completely independently from the main Wavecraft repo  
**So that** I can develop my plugin without cloning the framework source

### Acceptance Criteria
- [ ] Template can be cloned and built without main Wavecraft repo present
- [ ] No path dependencies that assume monorepo structure
- [ ] Template's `engine/Cargo.toml` uses proper dependency specifications
- [ ] Template's `ui/package.json` has all required dependencies listed
- [ ] `cargo clean && cargo xtask bundle` works (no stale cache assumptions)
- [ ] Template can be moved to any filesystem location

### Notes
- This is critical for developer adoption
- Test by cloning template to `/tmp/test-plugin/` and building there

---

## Testing Approach

### Phase 1: Automated Verification
1. Run `cargo xtask check` — must pass completely
2. Run visual testing with Playwright MCP

### Phase 2: Manual Workflow Testing
3. Clone template to fresh location, follow docs, build plugin
4. Load plugin in Ableton, test all integration points
5. Run `cargo xtask dev` and iterate on UI

### Phase 3: Documentation Review
6. Read all guides as a newcomer
7. Follow each guide's instructions literally
8. Document any gaps or inaccuracies

### Phase 4: Edge Case Testing
9. Test low/high buffer sizes
10. Test rapid automation
11. Test multi-instance scenarios

---

## Success Metrics

| Metric | Target |
|--------|--------|
| Time to first build (from clone) | < 30 minutes |
| Test suite pass rate | 100% |
| Documentation accuracy | No undocumented steps required |
| DAW integration | All acceptance criteria pass |
| Edge cases | No crashes or data corruption |

---

## Out of Scope

The following are explicitly **not** in scope for M12:

- Windows or Linux testing (macOS is primary target)
- DAWs other than Ableton Live (Logic Pro/Reaper are future targets)
- Performance optimization (unless critical bugs are found)
- New features (this is validation, not development)
- External testers (that's M13)

---

## Exit Criteria

Internal Testing is complete when:

1. ✅ All user story acceptance criteria are met
2. ✅ All bugs discovered are fixed (or explicitly documented as known issues)
3. ✅ Documentation is updated to reflect any findings
4. ✅ Version `0.6.3` is ready for tagging
5. ✅ PO approves that codebase is ready for external beta testing
