# Implementation Plan: Code Quality & OSS Prep (Milestone 11)

## Overview

This plan implements the Code Quality & OSS Prep milestone to prepare Wavecraft for open-source release.

**Low-Level Design:** [low-level-design-code-quality-polish.md](low-level-design-code-quality-polish.md)  
**User Stories:** [user-stories.md](user-stories.md)  
**Target Version:** `0.6.1`  
**Estimated Effort:** 10-16 hours (2-3 days)

---

## Implementation Phases

The implementation is organized into 8 phases, following the priority order from user stories. Each phase is independently testable and can be committed separately.

---

## Phase 1: Horizontal Scroll Fix (30 min)

**User Story:** #3 — Horizontal Scroll Fix  
**Priority:** 🔥 Highest (quick win, visible polish)

### Step 1.1: Fix CSS overflow

**File:** `ui/src/index.css`

**Action:** Add `overflow-x-hidden` to `#root` element

**Current:**
```css
#root {
  @apply h-screen w-full overflow-y-auto bg-plugin-dark;
}
```

**After:**
```css
#root {
  @apply h-screen w-full overflow-x-hidden overflow-y-auto bg-plugin-dark;
}
```

**Why:** Prevents horizontal scroll/wiggle when content (resize handle) extends past viewport

**Dependencies:** None

**Risk:** Low — simple CSS change

### Step 1.2: Update template project

**File:** `wavecraft-plugin-template/ui/src/index.css`

**Action:** Apply same fix to template project

### Step 1.3: Manual test

| # | Step | Expected |
|---|------|----------|
| 1 | Run `npm run dev`, open browser | UI loads |
| 2 | Resize browser window to minimum width | No horizontal scrollbar |
| 3 | Drag resize handle | No horizontal wiggle |
| 4 | Run `cargo xtask dev`, connect browser | Same behavior |

**Commit:** `fix(ui): prevent horizontal scroll wiggle on #root element`

---

## Phase 2: LICENSE File (15 min)

**User Story:** #5 — Open Source License Review  
**Priority:** High (legal clarity)

### Step 2.1: Create LICENSE file

**File:** `LICENSE` (repository root)

**Action:** Create MIT license file

```
MIT License

Copyright (c) 2026 Ron Houben

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

**Why:** MIT is simple, permissive, compatible with all dependencies

**Dependencies:** None

**Risk:** Low

### Step 2.2: Add LICENSE to template

**File:** `wavecraft-plugin-template/LICENSE`

**Action:** Copy same LICENSE file to template (plugins inherit license)

**Commit:** `docs: add MIT license`

---

## Phase 3: GitHub Issue Templates (1 hr)

**User Story:** #7 — GitHub Issue Templates  
**Priority:** High (structured feedback)

### Step 3.1: Create bug report template

**File:** `.github/ISSUE_TEMPLATE/bug_report.yml`

**Action:** Create YAML-based bug report form with:
- Wavecraft version (required)
- Operating system dropdown (required)
- DAW and version (optional)
- Steps to reproduce (required)
- Expected behavior (required)
- Actual behavior (required)
- Logs (optional)

**Why:** Form-based templates provide better UX than markdown templates

### Step 3.2: Create feature request template

**File:** `.github/ISSUE_TEMPLATE/feature_request.yml`

**Action:** Create YAML-based feature request form with:
- Problem statement (required)
- Proposed solution (required)
- Alternatives considered (optional)

### Step 3.3: Create config to show blank issue option

**File:** `.github/ISSUE_TEMPLATE/config.yml`

**Action:** Configure issue template chooser
```yaml
blank_issues_enabled: true
contact_links:
  - name: Documentation
    url: https://github.com/RonHouben/wavecraft/tree/main/docs
    about: Check the documentation before opening an issue
```

### Step 3.4: Create PR template

**File:** `.github/pull_request_template.md`

**Action:** Create PR template with:
- Description section
- Related issues section
- Testing section
- Checklist (lint, test, docs, coding standards)

**Commit:** `docs: add GitHub issue and PR templates`

---

## Phase 4: Contributing Guidelines (1-2 hrs)

**User Story:** #6 — Contributing Guidelines  
**Priority:** High (enables contributions)

### Step 4.1: Create CONTRIBUTING.md

**File:** `CONTRIBUTING.md` (repository root)

**Action:** Create contributing guidelines with sections:
1. Welcome message
2. Code of Conduct reference
3. Development setup (link to SDK guide)
4. Coding standards (link to docs)
5. Running tests (`cargo xtask test`)
6. Linting (`cargo xtask lint --fix`)
7. Pull request process
8. Commit message format (conventional commits)

**Why:** External contributors need clear guidance

**Dependencies:** LICENSE file (Phase 2)

**Risk:** Low

### Step 4.2: Create CODE_OF_CONDUCT.md

**File:** `CODE_OF_CONDUCT.md` (repository root)

**Action:** Add Contributor Covenant Code of Conduct (industry standard)

**Commit:** `docs: add contributing guidelines and code of conduct`

---

## Phase 5: README Polish (1-2 hrs)

**User Story:** #8 — README Polish  
**Priority:** High (first impression)

### Step 5.1: Add status badges

**File:** `README.md`

**Action:** Add badges at top:
```markdown
[![CI](https://github.com/RonHouben/wavecraft/actions/workflows/ci.yml/badge.svg)](https://github.com/RonHouben/wavecraft/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
```

### Step 5.2: Update license section

**File:** `README.md`

**Action:** Replace "License: TBD" with:
```markdown
## License

Wavecraft is released under the [MIT License](LICENSE).
```

### Step 5.3: Update project structure

**File:** `README.md`

**Action:** Update project structure diagram to reflect current crate names:
- `wavecraft-core`
- `wavecraft-dsp`
- `wavecraft-bridge`
- `wavecraft-protocol`
- `wavecraft-metering`

### Step 5.4: Add contributing link

**File:** `README.md`

**Action:** Add contributing section:
```markdown
## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.
```

### Step 5.5: Capture UI screenshot (optional)

**Action:** Use Playwright MCP to capture plugin UI screenshot for README

**Dependencies:** Phases 2-4

**Risk:** Low

**Commit:** `docs: polish README with badges, license, and updated structure`

---

## Phase 6: UI Logger (2-3 hrs)

**User Story:** #1 — Structured UI Logging  
**Priority:** Medium (developer experience)

### Step 6.1: Create Logger class

**File:** `ui/src/lib/logger/Logger.ts`

**Action:** Implement Logger class per low-level design:
- `LogLevel` enum (DEBUG, INFO, WARN, ERROR)
- `Logger.create(context)` factory method
- `Logger.setLevel(level)` static method
- Instance methods: `debug()`, `info()`, `warn()`, `error()`
- Timestamp formatting
- Colored console output

### Step 6.2: Create barrel export

**File:** `ui/src/lib/logger/index.ts`

**Action:** Export Logger and LogLevel

### Step 6.3: Add Logger tests

**File:** `ui/src/lib/logger/Logger.test.ts`

**Action:** Write unit tests:
- Creates logger with context
- Respects log level filtering
- Formats output with timestamp and context
- Handles data parameter
- Uses correct console method per level

### Step 6.4: Migrate WebSocketTransport

**File:** `ui/src/lib/wavecraft-ipc/transports/WebSocketTransport.ts`

**Action:** 
1. Add private logger: `private readonly log = Logger.create('WebSocketTransport');`
2. Replace `console.log` calls with logger calls

**Current (line 153):**
```typescript
console.log(`WebSocketTransport: Connected to ${this.url}`);
```

**After:**
```typescript
this.log.info(`Connected to ${this.url}`);
```

**Current (line 203-207):**
```typescript
console.log(
  `WebSocketTransport: Connection closed. Code: ${event.code}, Reason: ${event.reason || 'none'}, Clean: ${event.wasClean}`
);
```

**After:**
```typescript
this.log.info('Connection closed', {
  code: event.code,
  reason: event.reason || 'none',
  wasClean: event.wasClean,
});
```

### Step 6.5: Update template project

**Files:**
- `wavecraft-plugin-template/ui/src/lib/logger/Logger.ts`
- `wavecraft-plugin-template/ui/src/lib/logger/index.ts`
- `wavecraft-plugin-template/ui/src/lib/wavecraft-ipc/transports/WebSocketTransport.ts`

**Action:** Copy Logger files and apply same migration

### Step 6.6: Run tests

**Command:** `npm test` in `ui/` directory

**Expected:** All tests pass including new Logger tests

**Commit:** `feat(ui): add structured Logger class, replace console.log calls`

---

## Phase 7: Engine Logging (3-4 hrs)

**User Story:** #2 — Structured Engine Logging  
**Priority:** Medium (developer experience)

### Step 7.1: Add workspace dependencies

**File:** `engine/Cargo.toml`

**Action:** Add to `[workspace.dependencies]`:
```toml
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Step 7.2: Add standalone crate dependencies

**File:** `engine/crates/standalone/Cargo.toml`

**Action:** Add to `[dependencies]`:
```toml
tracing.workspace = true
tracing-subscriber.workspace = true
```

### Step 7.3: Initialize tracing in main.rs

**File:** `engine/crates/standalone/src/main.rs`

**Action:**
1. Add imports: `use tracing::{info, debug, warn, error};`
2. Add `use tracing_subscriber::EnvFilter;`
3. Create `init_logging(verbose: bool)` function
4. Call `init_logging()` at start of `main()`
5. Replace all `println!` calls with tracing macros

**Migration examples:**
| Current | After |
|---------|-------|
| `println!("Starting VstKit dev server on port {}...", port)` | `info!(port, "Starting Wavecraft dev server")` |
| `println!("Verbose mode: showing all JSON-RPC messages")` | `debug!("Verbose mode enabled")` |
| `println!("Press Ctrl+C to stop")` | `info!("Press Ctrl+C to stop")` |
| `println!("\nShutting down...")` | `info!("Shutting down")` |

### Step 7.4: Migrate ws_server.rs

**File:** `engine/crates/standalone/src/ws_server.rs`

**Action:** Replace all 12 `println!`/`eprintln!` calls with tracing macros

**Migration examples:**
| Current | After |
|---------|-------|
| `println!("[WebSocket] Server listening on ws://{}", addr)` | `info!(%addr, "WebSocket server listening")` |
| `println!("[WebSocket] Client connected: {}", addr)` | `info!(%addr, "WebSocket client connected")` |
| `eprintln!("[WebSocket] Accept error: {}", e)` | `error!(?e, "WebSocket accept error")` |
| `println!("[WebSocket] Connection closed: {}", addr)` | `info!(%addr, "WebSocket connection closed")` |

### Step 7.5: Migrate webview.rs

**File:** `engine/crates/standalone/src/webview.rs`

**Action:** Replace 4 `eprintln!` calls with tracing macros

### Step 7.6: Migrate assets.rs

**File:** `engine/crates/standalone/src/assets.rs`

**Action:** Replace 2 `println!` calls with tracing macros (debug level)

### Step 7.7: Update wavecraft-protocol (test only)

**File:** `engine/crates/wavecraft-protocol/src/ipc.rs`

**Action:** The single `println!` on line 395 is in a test. Leave as-is (test output).

### Step 7.8: Run engine tests

**Command:** `cargo test --workspace` in `engine/` directory

**Expected:** All tests pass

### Step 7.9: Manual test

| # | Step | Expected |
|---|------|----------|
| 1 | Run `cargo run -p standalone` | Info-level logs visible |
| 2 | Run `cargo run -p standalone -- --verbose` | Debug-level logs visible |
| 3 | Run `RUST_LOG=error cargo run -p standalone` | Only errors visible |
| 4 | Connect browser to WebSocket | Connection logs appear |

**Commit:** `feat(engine): add tracing-based logging to standalone crate`

---

## Phase 8: CI Cache Optimization (1-2 hrs)

**User Story:** #4 — CI Cache Optimization  
**Priority:** Low (nice-to-have)

### Step 8.1: Add shared-key to rust-cache

**File:** `.github/workflows/ci.yml`

**Action:** Add `shared-key` to all `Swatinem/rust-cache@v2` steps:

```yaml
- name: Cache cargo
  uses: Swatinem/rust-cache@v2
  with:
    workspaces: engine
    shared-key: "wavecraft-engine"
```

Apply to:
- `prepare-engine` job
- `check-engine` job
- `test-engine` job
- `build-plugin` job

### Step 8.2: Document caching strategy

**File:** `docs/guides/ci-pipeline.md`

**Action:** Add section documenting:
- Rust cache configuration
- npm cache via setup-node
- APT package caching
- Shared key strategy
- Cache invalidation (when lock files change)

### Step 8.3: Measure improvement

**Action:** 
1. Run CI pipeline on current commit (before optimization)
2. Record job durations
3. Apply optimization, push
4. Run CI pipeline again (after optimization)
5. Compare durations on cache hit

**Target:** ≥30% reduction on cache hits (may not be achievable given existing optimization)

**Commit:** `ci: add shared-key for consistent rust-cache across jobs`

---

## Phase 9: Version Bump & Finalization

### Step 9.1: Bump version to 0.6.1

**File:** `engine/Cargo.toml`

**Action:** Update `[workspace.package]` version:
```toml
version = "0.6.1"
```

### Step 9.2: Run full test suite

**Commands:**
```bash
cd engine && cargo xtask test
cd engine && cargo xtask lint
```

**Expected:** All tests pass, no lint errors

### Step 9.3: Manual verification

| # | Step | Expected |
|---|------|----------|
| 1 | Run `cargo xtask dev` | Dev server starts with tracing output |
| 2 | Open browser to localhost:5173 | UI loads, no horizontal scroll |
| 3 | Check DevTools console | Logger output with timestamps |
| 4 | Load plugin in Ableton Live | Works correctly |
| 5 | Check version badge in UI | Shows "v0.6.1" |

**Commit:** `chore: bump version to 0.6.1`

---

## Summary

| Phase | Description | Files Changed | Effort |
|-------|-------------|---------------|--------|
| 1 | Horizontal Scroll Fix | 2 | 30 min |
| 2 | LICENSE File | 2 | 15 min |
| 3 | GitHub Issue Templates | 4 | 1 hr |
| 4 | Contributing Guidelines | 2 | 1-2 hrs |
| 5 | README Polish | 1 | 1-2 hrs |
| 6 | UI Logger | 6 | 2-3 hrs |
| 7 | Engine Logging | 5 | 3-4 hrs |
| 8 | CI Cache Optimization | 2 | 1-2 hrs |
| 9 | Version Bump | 1 | 15 min |
| **Total** | | **25 files** | **10-16 hrs** |

---

## Commit History (Expected)

```
fix(ui): prevent horizontal scroll wiggle on #root element
docs: add MIT license
docs: add GitHub issue and PR templates
docs: add contributing guidelines and code of conduct
docs: polish README with badges, license, and updated structure
feat(ui): add structured Logger class, replace console.log calls
feat(engine): add tracing-based logging to standalone crate
ci: add shared-key for consistent rust-cache across jobs
chore: bump version to 0.6.1
```

---

## Risk Mitigation

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `overflow-x-hidden` clips content | Low | Test thoroughly, use `overflow-x-clip` if needed |
| Logger breaks hot reload | Low | Use `import.meta.env` which is stable |
| tracing adds binary size | Low | Only in standalone crate (dev tool) |
| CI cache not improving | Medium | Document findings, existing cache is good |

---

## Success Criteria

- [ ] No horizontal scroll/wiggle in UI
- [ ] MIT LICENSE file in repo root
- [ ] GitHub issue templates work
- [ ] CONTRIBUTING.md provides clear guidance
- [ ] README has badges and updated license
- [ ] Zero `console.log` in production UI code
- [ ] Zero `println!` in standalone crate (except tests)
- [ ] All tests pass (UI + Engine)
- [ ] Version displays as 0.6.1 in UI
