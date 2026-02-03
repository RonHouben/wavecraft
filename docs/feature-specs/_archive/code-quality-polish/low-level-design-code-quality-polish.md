# Low-Level Design: Code Quality & OSS Prep

## 1. Overview

This document provides the technical design for Milestone 11, preparing Wavecraft for open-source release through logging infrastructure, UX polish, CI optimization, and contributor documentation.

**User Stories:** [user-stories.md](user-stories.md)  
**Target Version:** `0.6.1`

---

## 2. Scope

### In Scope
1. **UI Logger** — Structured logging class replacing `console.log`
2. **Engine Logging** — `tracing` crate integration
3. **Horizontal Scroll Fix** — CSS overflow correction
4. **CI Cache Optimization** — Reduce build times
5. **Open Source Preparation** — LICENSE, CONTRIBUTING.md, issue templates, README polish

### Out of Scope
- New features or architectural changes
- SDK publication to crates.io
- CLI scaffolding tool
- Additional platform support

---

## 3. Current State Analysis

### 3.1 UI Logging (Current)

**Files with `console.log`:**
| File | Count | Purpose |
|------|-------|---------|
| `WebSocketTransport.ts` | 2 | Connection status, graceful degradation messages |
| `resize.ts` | 1 | Documentation example (comment only) |

**Assessment:** Minimal `console.log` usage — only 2 actual calls in production code. Both in `WebSocketTransport.ts`.

### 3.2 Engine Logging (Current)

**Files with `println!`/`eprintln!`:**
| File | Count | Purpose |
|------|-------|---------|
| `standalone/src/ws_server.rs` | 12 | WebSocket connection events |
| `standalone/src/main.rs` | 5 | Server startup messages |
| `standalone/src/webview.rs` | 4 | IPC debug messages |
| `standalone/src/assets.rs` | 2 | Asset discovery (debug) |
| `standalone/tests/latency_bench.rs` | 16 | Benchmark output |
| `wavecraft-protocol/src/ipc.rs` | 1 | Notification debug (test only) |
| `xtask/src/commands/*.rs` | Many | CLI output (acceptable) |

**Assessment:** 
- ~24 `println!`/`eprintln!` in production code (non-test, non-xtask)
- xtask commands should keep `println!` for CLI output (that's intentional)
- Test files can keep `println!` for benchmark reports
- Focus on `standalone` crate and `wavecraft-protocol`

### 3.3 Horizontal Scroll (Current)

```css
/* ui/src/index.css */
#root {
  @apply h-screen w-full overflow-y-auto bg-plugin-dark;
}
```

**Problem:** `overflow-x` is implicitly `auto`, allowing horizontal scrolling when content exceeds viewport width (e.g., from resize handle extending past boundaries).

### 3.4 CI Pipeline (Current)

The CI already uses `Swatinem/rust-cache@v2` and `setup-node` with `cache: 'npm'`. Current optimizations:
- ✅ Rust cache via `rust-cache@v2`
- ✅ npm cache via `setup-node` with `cache-dependency-path`
- ✅ APT packages cached via `cache-apt-pkgs-action`
- ⚠️ UI dist rebuilt in multiple jobs (some redundancy)

**Assessment:** Caching is already well-implemented. Main optimization opportunity is reducing redundant UI builds.

---

## 4. Technical Design

### 4.1 UI Logger Class

**Design Rationale:**
The UI has minimal logging currently (2 calls). Rather than over-engineering, we'll create a simple but extensible `Logger` class that:
1. Provides structured output with timestamps and context
2. Supports log levels for filtering
3. Can be easily extended later (e.g., remote logging, file output)

**File:** `ui/src/lib/logger/Logger.ts`

```typescript
/**
 * Log levels from most verbose to most severe
 */
export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

/**
 * Logger configuration
 */
interface LoggerConfig {
  /** Minimum level to output (default: INFO in production, DEBUG in dev) */
  level: LogLevel;
  /** Whether to include timestamps (default: true) */
  timestamps: boolean;
}

/**
 * Structured logger for Wavecraft UI
 * 
 * Usage:
 *   const log = Logger.create('WebSocketTransport');
 *   log.info('Connected');
 *   log.debug('Message received', { id: 123 });
 *   log.warn('Reconnecting...');
 *   log.error('Connection failed', error);
 */
export class Logger {
  private static globalLevel: LogLevel = import.meta.env.DEV 
    ? LogLevel.DEBUG 
    : LogLevel.INFO;
  
  private constructor(private context: string) {}

  /**
   * Create a logger for a specific module/component
   */
  static create(context: string): Logger {
    return new Logger(context);
  }

  /**
   * Set global log level (affects all loggers)
   */
  static setLevel(level: LogLevel): void {
    Logger.globalLevel = level;
  }

  debug(message: string, data?: unknown): void {
    this.log(LogLevel.DEBUG, message, data);
  }

  info(message: string, data?: unknown): void {
    this.log(LogLevel.INFO, message, data);
  }

  warn(message: string, data?: unknown): void {
    this.log(LogLevel.WARN, message, data);
  }

  error(message: string, error?: unknown): void {
    this.log(LogLevel.ERROR, message, error);
  }

  private log(level: LogLevel, message: string, data?: unknown): void {
    if (level < Logger.globalLevel) return;

    const timestamp = new Date().toISOString().slice(11, 23); // HH:mm:ss.SSS
    const prefix = `[${timestamp}] [${this.context}]`;
    
    const consoleMethod = this.getConsoleMethod(level);
    const style = this.getStyle(level);
    
    if (data !== undefined) {
      consoleMethod(`%c${prefix} ${message}`, style, data);
    } else {
      consoleMethod(`%c${prefix} ${message}`, style);
    }
  }

  private getConsoleMethod(level: LogLevel): typeof console.log {
    switch (level) {
      case LogLevel.DEBUG: return console.debug;
      case LogLevel.INFO: return console.info;
      case LogLevel.WARN: return console.warn;
      case LogLevel.ERROR: return console.error;
    }
  }

  private getStyle(level: LogLevel): string {
    switch (level) {
      case LogLevel.DEBUG: return 'color: #888';
      case LogLevel.INFO: return 'color: #4a9eff';
      case LogLevel.WARN: return 'color: #ffeb3b';
      case LogLevel.ERROR: return 'color: #ff1744; font-weight: bold';
    }
  }
}
```

**Export:** `ui/src/lib/logger/index.ts`
```typescript
export { Logger, LogLevel } from './Logger';
```

**Migration:**
| Current | After |
|---------|-------|
| `console.log(\`WebSocketTransport: Connected to ${this.url}\`)` | `this.log.info(\`Connected to ${this.url}\`)` |
| `console.log(...)` in graceful degradation | `this.log.info('Connection closed', { reason })` |

### 4.2 Engine Logging with `tracing`

**Design Rationale:**
The Rust ecosystem standard for structured logging is `tracing`. It provides:
- Zero-cost abstractions (disabled logs compile away)
- Structured key-value logging
- Async-aware spans
- Multiple subscriber backends

**CRITICAL CONSTRAINT:** No logging on the audio thread. The `tracing` crate allocates, which violates real-time safety.

**Dependencies to add:**

```toml
# engine/Cargo.toml (workspace)
[workspace.dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**Crate-specific usage:**

| Crate | Add tracing? | Rationale |
|-------|--------------|-----------|
| `wavecraft-core` | ❌ No | Contains audio thread code |
| `wavecraft-dsp` | ❌ No | Pure DSP, no logging needed |
| `wavecraft-bridge` | ⚠️ Limited | IPC handling, but called from audio context |
| `wavecraft-protocol` | ❌ No | Pure types, no runtime |
| `wavecraft-metering` | ❌ No | Real-time ring buffers |
| `standalone` | ✅ Yes | Development server, not real-time |

**Standalone crate integration:**

```rust
// engine/crates/standalone/src/main.rs
use tracing::{info, debug, warn, error};
use tracing_subscriber::{fmt, EnvFilter};

fn init_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::new("wavecraft=debug,info")
    } else {
        EnvFilter::new("wavecraft=info")
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)  // Don't show module path (cleaner output)
        .with_thread_names(false)
        .init();
}
```

**Migration examples:**

| Current | After |
|---------|-------|
| `println!("[WebSocket] Server listening on ws://{}", addr)` | `info!("WebSocket server listening on ws://{}", addr)` |
| `eprintln!("[WebSocket] Accept error: {}", e)` | `error!("WebSocket accept error: {}", e)` |
| `println!("[WebSocket] Client connected: {}", addr)` | `info!(client = %addr, "WebSocket client connected")` |

**Environment variable control:**
```bash
# Default: info level
cargo run -p standalone

# Verbose: debug level
RUST_LOG=debug cargo run -p standalone
# or
cargo run -p standalone -- --verbose
```

### 4.3 Horizontal Scroll Fix

**Root Cause:** The `#root` element has `overflow-y: auto` but `overflow-x` defaults to `auto`, allowing horizontal scroll when content (like resize handle) extends past the viewport.

**Fix:**

```css
/* ui/src/index.css */
#root {
  @apply h-screen w-full overflow-x-hidden overflow-y-auto bg-plugin-dark;
}
```

**Alternative (if `overflow-hidden` causes issues):**
```css
#root {
  @apply h-screen w-full overflow-y-auto overflow-x-clip bg-plugin-dark;
}
```

`overflow-x-clip` is more aggressive than `hidden` and prevents scrolling even if content is programmatically shifted.

**Testing:**
1. Run `npm run dev` in browser
2. Resize window to various sizes
3. Drag resize handle
4. Verify no horizontal scrollbar appears
5. Test in WKWebView (plugin in Ableton)

### 4.4 CI Cache Optimization

**Current State Analysis:**
The CI pipeline already has good caching:
- `Swatinem/rust-cache@v2` for Rust builds
- `setup-node` with `cache: 'npm'` for npm
- `cache-apt-pkgs-action` for system dependencies

**Identified Issue:**
The UI `dist` is built in `prepare-engine` job and shared via artifact. This is already optimal.

**Potential Optimization: Shared Key Strategy**

Currently each job uses independent rust-cache instances with implicit keys. We can ensure consistent cache keys:

```yaml
- name: Cache cargo
  uses: Swatinem/rust-cache@v2
  with:
    workspaces: engine
    shared-key: "wavecraft-engine"  # Consistent across jobs
```

**Measurement Plan:**
1. Run current CI pipeline 3 times, record times
2. Implement shared-key optimization
3. Run optimized pipeline 3 times, compare
4. Target: ≥30% reduction on cache hits

**Conclusion:** The CI is already well-optimized. We'll add `shared-key` and document the caching strategy, but don't expect dramatic improvements.

### 4.5 Open Source Preparation

#### 4.5.1 LICENSE File

**Choice:** MIT License

**Rationale:**
- Simple, permissive, widely understood
- Compatible with all dependencies (nih-plug is ISC, wry is MIT/Apache-2.0)
- No copyleft obligations that might deter commercial use
- Standard for Rust ecosystem

**File:** `LICENSE` (root)

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

**Update README:** Add "License: MIT" with link.

#### 4.5.2 CONTRIBUTING.md

**File:** `CONTRIBUTING.md` (root)

**Structure:**
1. Welcome & Code of Conduct reference
2. Development setup (link to SDK guide)
3. Coding standards (link to docs)
4. Pull request process
5. Testing requirements
6. Commit message format

**Key Points:**
- Reference existing documentation rather than duplicating
- Emphasize running `cargo xtask lint` and `cargo xtask test` before PR
- Require conventional commits (feat:, fix:, docs:, etc.)
- Squash merge policy

#### 4.5.3 GitHub Issue Templates

**Directory:** `.github/ISSUE_TEMPLATE/`

**Templates:**

1. **bug_report.md**
```yaml
name: Bug Report
description: Report a bug in Wavecraft
labels: ["bug", "triage"]
body:
  - type: input
    id: version
    attributes:
      label: Wavecraft Version
      placeholder: "0.6.1"
    validations:
      required: true
  - type: dropdown
    id: os
    attributes:
      label: Operating System
      options:
        - macOS
        - Windows
        - Linux
    validations:
      required: true
  - type: input
    id: daw
    attributes:
      label: DAW and Version
      placeholder: "Ableton Live 12.1"
  - type: textarea
    id: steps
    attributes:
      label: Steps to Reproduce
      placeholder: "1. ...\n2. ...\n3. ..."
    validations:
      required: true
  - type: textarea
    id: expected
    attributes:
      label: Expected Behavior
    validations:
      required: true
  - type: textarea
    id: actual
    attributes:
      label: Actual Behavior
    validations:
      required: true
  - type: textarea
    id: logs
    attributes:
      label: Relevant Logs
      render: shell
```

2. **feature_request.md**
```yaml
name: Feature Request
description: Suggest a new feature
labels: ["enhancement"]
body:
  - type: textarea
    id: problem
    attributes:
      label: Problem Statement
      description: What problem does this solve?
    validations:
      required: true
  - type: textarea
    id: solution
    attributes:
      label: Proposed Solution
    validations:
      required: true
  - type: textarea
    id: alternatives
    attributes:
      label: Alternatives Considered
```

3. **pull_request_template.md** (in `.github/`)
```markdown
## Description

<!-- What does this PR do? -->

## Related Issues

<!-- Fixes #123 -->

## Testing

<!-- How was this tested? -->

## Checklist

- [ ] I have run `cargo xtask lint --fix`
- [ ] I have run `cargo xtask test`
- [ ] I have updated documentation (if needed)
- [ ] My changes follow the [coding standards](docs/architecture/coding-standards.md)
```

#### 4.5.4 README Polish

**Current Issues:**
1. ❌ No logo/banner image
2. ❌ "License: TBD" needs updating
3. ❌ No badges (CI status, version)
4. ⚠️ Project structure diagram is outdated (references old paths)
5. ⚠️ No screenshot of the plugin UI
6. ✅ Good documentation links
7. ✅ Clear build instructions

**Improvements:**

1. **Add badges:**
```markdown
[![CI](https://github.com/RonHouben/wavecraft/actions/workflows/ci.yml/badge.svg)](https://github.com/RonHouben/wavecraft/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
```

2. **Update license section:**
```markdown
## License

Wavecraft is released under the [MIT License](LICENSE).
```

3. **Add screenshot** (once available, agent can use Playwright to capture)

4. **Update project structure** to match current crate names

5. **Add "Quick Start" section** at top for discoverability

---

## 5. File Changes Summary

### New Files
| File | Purpose |
|------|---------|
| `ui/src/lib/logger/Logger.ts` | Structured UI logging |
| `ui/src/lib/logger/index.ts` | Export barrel |
| `LICENSE` | MIT license |
| `CONTRIBUTING.md` | Contributor guidelines |
| `.github/ISSUE_TEMPLATE/bug_report.md` | Bug report template |
| `.github/ISSUE_TEMPLATE/feature_request.md` | Feature request template |
| `.github/pull_request_template.md` | PR template |

### Modified Files
| File | Changes |
|------|---------|
| `ui/src/index.css` | Add `overflow-x-hidden` |
| `ui/src/lib/wavecraft-ipc/transports/WebSocketTransport.ts` | Replace `console.log` with Logger |
| `engine/Cargo.toml` | Add `tracing`, `tracing-subscriber` |
| `engine/crates/standalone/Cargo.toml` | Add tracing deps |
| `engine/crates/standalone/src/main.rs` | Initialize tracing, replace println |
| `engine/crates/standalone/src/ws_server.rs` | Replace println/eprintln with tracing |
| `engine/crates/standalone/src/webview.rs` | Replace eprintln with tracing |
| `engine/crates/standalone/src/assets.rs` | Replace println with tracing |
| `.github/workflows/ci.yml` | Add shared-key to rust-cache |
| `README.md` | Add badges, license, update structure |

### Unchanged (Intentionally)
| File | Rationale |
|------|-----------|
| `engine/xtask/src/commands/*.rs` | CLI output should use println |
| `engine/crates/standalone/tests/*.rs` | Test output can use println |
| `engine/crates/wavecraft-core/*` | No logging in real-time code |
| `engine/crates/wavecraft-dsp/*` | No logging in DSP code |

---

## 6. Testing Strategy

### 6.1 UI Logger Tests

**File:** `ui/src/lib/logger/Logger.test.ts`

```typescript
describe('Logger', () => {
  it('creates logger with context', () => { ... });
  it('respects log level filtering', () => { ... });
  it('formats output with timestamp and context', () => { ... });
  it('handles data parameter', () => { ... });
});
```

### 6.2 Horizontal Scroll Manual Test

| # | Step | Expected |
|---|------|----------|
| 1 | Run `npm run dev`, open in browser | UI loads |
| 2 | Resize browser window to minimum width | No horizontal scrollbar |
| 3 | Drag resize handle to various positions | No horizontal scroll/wiggle |
| 4 | Load plugin in Ableton Live | No horizontal scroll in plugin window |

### 6.3 Engine Logging Manual Test

| # | Step | Expected |
|---|------|----------|
| 1 | Run `cargo run -p standalone` | Info-level logs visible |
| 2 | Run with `--verbose` flag | Debug-level logs visible |
| 3 | Run with `RUST_LOG=error` | Only errors visible |
| 4 | Connect browser to WebSocket | Connection logs appear |

### 6.4 CI Cache Verification

| # | Step | Expected |
|---|------|----------|
| 1 | Push commit, note CI time | Baseline measurement |
| 2 | Push another commit (no Cargo.lock change) | Cache hit, faster build |
| 3 | Update Cargo.lock, push | Cache miss, full rebuild |

---

## 7. Implementation Order

Per user story priority:

1. **Horizontal Scroll Fix** (30 min) — Quick win, immediate polish
2. **README Polish** (1-2 hrs) — First impression for OSS
3. **CONTRIBUTING.md** (1-2 hrs) — Enables contributions
4. **GitHub Issue Templates** (1 hr) — Structured feedback
5. **LICENSE** (15 min) — Legal clarity
6. **UI Logger** (2-3 hrs) — DX improvement
7. **Engine Logging** (3-4 hrs) — DX improvement
8. **CI Cache Optimization** (1-2 hrs) — Nice-to-have

**Total estimated effort:** 10-16 hours (2-3 days)

---

## 8. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| `overflow-x-hidden` clips resize handle | Low | Medium | Use `overflow-x-clip`, test thoroughly |
| tracing adds binary size | Low | Low | Only in standalone crate (dev tool) |
| Logger overhead in production | Low | Low | Level filtering, console methods are native |
| CI cache not improving times | Medium | Low | Already well-cached; document findings |

---

## 9. Dependencies

### External Dependencies (New)

| Crate | Version | Purpose | License |
|-------|---------|---------|---------|
| `tracing` | 0.1 | Structured logging | MIT |
| `tracing-subscriber` | 0.3 | Log formatting | MIT |

Both are widely used Rust ecosystem standards with permissive licenses.

### No New npm Dependencies

The Logger class is pure TypeScript with no external dependencies.

---

## 10. Decision Log

| Decision | Alternatives Considered | Rationale |
|----------|------------------------|-----------|
| MIT License | Apache-2.0, dual MIT/Apache-2.0 | Simpler, familiar, compatible |
| `tracing` crate | `log` crate, custom | Ecosystem standard, structured logging |
| Logger class (not hooks) | useLogger hook | Class matches coding standards, works outside React |
| `overflow-x-hidden` | JavaScript scroll prevention | CSS is simpler, more reliable |
| Issue templates (YAML) | Markdown templates | Better UX, form-based |

---

## 11. Appendix: Dependency License Audit

### Core Dependencies

| Dependency | License | Compatible? |
|------------|---------|-------------|
| nih-plug | ISC | ✅ Yes |
| wry | MIT/Apache-2.0 | ✅ Yes |
| rtrb | MIT/Apache-2.0 | ✅ Yes |
| serde | MIT/Apache-2.0 | ✅ Yes |
| tokio | MIT | ✅ Yes |
| React | MIT | ✅ Yes |
| Vite | MIT | ✅ Yes |
| TailwindCSS | MIT | ✅ Yes |

All dependencies are compatible with MIT licensing.
