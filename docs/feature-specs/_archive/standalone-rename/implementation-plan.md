# Implementation Plan: Rename `standalone` → `wavecraft-dev-server`

**Status:** Ready for Implementation  
**Created:** 2026-02-06  
**Based on:** [low-level-design-standalone-rename.md](./low-level-design-standalone-rename.md)

---

## Overview

Rename the `standalone` crate to `wavecraft-dev-server` to clearly communicate its purpose as a development server for testing plugin UIs against the real Rust backend.

## Requirements

- Rename crate folder preserving git history
- Update all Rust code references (imports, comments, CLI metadata)
- Update CLI dependency on the dev-server crate (path + import)
- Update CLI lockfile to reflect crate rename
- Update xtask build commands
- Update active documentation (not archived specs)
- Maintain build stability throughout

## Architecture Changes

- **Directory**: `engine/crates/standalone/` → `engine/crates/wavecraft-dev-server/`
- **Package name**: `standalone` → `wavecraft-dev-server`
- **Binary name**: `standalone` → `wavecraft-dev-server`

---

## Implementation Steps

### Phase 1: Crate Rename (Atomic — must be completed together)

#### 1.1 Rename folder via git
**Action:** Terminal command  
**Why:** Preserves git history for the folder  
**Dependencies:** None  
**Risk:** Low

```bash
git mv engine/crates/standalone engine/crates/wavecraft-dev-server
```

---

#### 1.2 Update crate Cargo.toml
**File:** `engine/crates/wavecraft-dev-server/Cargo.toml`  
**Why:** Package name and binary name must match new crate name  
**Dependencies:** Step 1.1  
**Risk:** Low

**Changes:**
```toml
# Line 2: Change package name
name = "wavecraft-dev-server"

# Line 7: Update description  
description = "Development server for Wavecraft plugin UI testing"

# Line 11: Change binary name
name = "wavecraft-dev-server"
```

---

#### 1.3 Update workspace Cargo.toml
**File:** `engine/Cargo.toml`  
**Why:** Workspace must reference new crate path  
**Dependencies:** Step 1.1  
**Risk:** Low

**Change on line 31:**
```toml
# Before:
standalone = { path = "crates/standalone" }

# After:
wavecraft-dev-server = { path = "crates/wavecraft-dev-server" }
```

---

#### 1.4 Update test imports
**File:** `engine/crates/wavecraft-dev-server/tests/integration_test.rs`  
**Why:** Crate imports must use new package name (with underscores)  
**Dependencies:** Steps 1.2, 1.3  
**Risk:** Low

**Change on line 3:**
```rust
// Before:
use standalone::AppState;

// After:
use wavecraft_dev_server::AppState;
```

---

#### 1.5 Update benchmark imports
**File:** `engine/crates/wavecraft-dev-server/tests/latency_bench.rs`  
**Why:** Crate imports must use new package name (with underscores)  
**Dependencies:** Steps 1.2, 1.3  
**Risk:** Low

**Change on line 3:**
```rust
// Before:
use standalone::AppState;

// After:
use wavecraft_dev_server::AppState;
```

---

#### 1.6 Verify Phase 1 build
**Action:** Terminal command  
**Why:** Ensure crate compiles before proceeding  
**Dependencies:** Steps 1.1–1.5  
**Risk:** Low

```bash
cd engine && cargo build -p wavecraft-dev-server
```

---

### Phase 2: xtask & CLI Updates

#### 2.1 Update xtask dev command
**File:** `engine/xtask/src/commands/dev.rs`  
**Why:** Command must invoke new package name  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change on line 35:**
```rust
// Before:
        "standalone",

// After:
        "wavecraft-dev-server",
```

---

#### 2.2 Update CLI dependency path
**File:** `cli/Cargo.toml`  
**Why:** CLI depends on the dev server crate for the embedded WebSocket server  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change at dependency list:**
```toml
# Before:
[dependencies.standalone]
path = "../engine/crates/standalone"

# After:
[dependencies.wavecraft-dev-server]
path = "../engine/crates/wavecraft-dev-server"
```

---

#### 2.3 Update CLI import
**File:** `cli/src/commands/start.rs`  
**Why:** Import must use the renamed crate  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change near top of file:**
```rust
// Before:
use standalone::ws_server::WsServer;

// After:
use wavecraft_dev_server::ws_server::WsServer;
```

---

#### 2.4 Update CLI dev_server module comments
**File:** `cli/src/dev_server/mod.rs`  
**Why:** Comments mention the standalone crate by name  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change comment references:**
```rust
// Example before:
//! │  WsServer<H>    │  from standalone crate

// After:
//! │  WsServer<H>    │  from wavecraft-dev-server crate
```

---

#### 2.5 Update CLI Cargo.lock
**File:** `cli/Cargo.lock`  
**Why:** Lockfile will still reference the old crate name  
**Dependencies:** Steps 2.2–2.3  
**Risk:** Low

**Action:** Run `cargo update -p wavecraft-dev-server` or regenerate lockfile via `cargo build` in `cli/` after updating the dependency.

---

#### 2.6 Verify Phase 2 functionality
**Action:** Terminal commands  
**Why:** Ensure xtask dev works  
**Dependencies:** Steps 2.1–2.5  
**Risk:** Low

```bash
cd engine && cargo xtask dev &
sleep 3 && pkill -f "wavecraft-dev-server"
```

---

### Phase 3: Source Comments & CLI Metadata

#### 3.1 Update main.rs doc comment
**File:** `engine/crates/wavecraft-dev-server/src/main.rs`  
**Why:** Doc comment should reflect new name  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change on line 1:**
```rust
// Before:
//! Standalone application entry point

// After:
//! Wavecraft dev server entry point
```

---

#### 3.2 Update CLI command metadata
**File:** `engine/crates/wavecraft-dev-server/src/main.rs`  
**Why:** CLI help text should show correct name  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Changes on lines 17-20:**
```rust
// Before:
/// VstKit Standalone - Audio plugin UI testing tool
#[derive(Parser, Debug)]
#[command(name = "standalone")]
#[command(about = "VstKit standalone app for UI development and testing")]

// After:
/// Wavecraft dev server - Audio plugin UI testing tool
#[derive(Parser, Debug)]
#[command(name = "wavecraft-dev-server")]
#[command(about = "Wavecraft development server for plugin UI testing")]
```

---

#### 3.3 Update dev server log message
**File:** `engine/crates/wavecraft-dev-server/src/main.rs`  
**Why:** Log messages should reference correct product name  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change on line 55:**
```rust
// Before:
    info!("Starting VstKit dev server on port {}", port);

// After:
    info!("Starting Wavecraft dev server on port {}", port);
```

---

#### 3.4 Update GUI app log message
**File:** `engine/crates/wavecraft-dev-server/src/main.rs`  
**Why:** Log messages should reference correct product name  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change on line 88:**
```rust
// Before:
    info!("Starting VstKit Standalone...");

// After:
    info!("Starting Wavecraft Dev Server (GUI mode)...");
```

---

#### 3.5 Update lib.rs doc comment
**File:** `engine/crates/wavecraft-dev-server/src/lib.rs`  
**Why:** Doc comment should reflect new name  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change on lines 1-3:**
```rust
// Before:
//! Standalone app library
//!
//! Exports modules for testing and integration.

// After:
//! Wavecraft dev server library
//!
//! Exports modules for testing and integration.
```

---

#### 3.6 Update crate README title and description
**File:** `engine/crates/wavecraft-dev-server/README.md`  
**Why:** README should reflect new name and purpose  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Changes on lines 1-5:**
```markdown
# Before:
# Wavecraft Desktop POC

**Status:** ✅ Complete

A standalone desktop application demonstrating WebView ↔ Rust IPC communication for the Wavecraft plugin framework.

# After:
# Wavecraft Dev Server

**Status:** ✅ Complete

A development server for testing Wavecraft plugin UIs against a real Rust backend. Supports WebSocket mode for browser development and GUI mode with embedded WebView.
```

---

### Phase 4: Documentation Updates

#### 4.1 Update root README project structure
**File:** `README.md`  
**Why:** Project structure docs must reflect new name  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change on line 90:**
```markdown
# Before:
│       └── standalone/           # Standalone dev server (WebSocket, WebView)

# After:
│       └── wavecraft-dev-server/ # Development server (WebSocket, WebView)
```

---

#### 4.2 Update coding-standards.md crate list
**File:** `docs/architecture/coding-standards.md`  
**Why:** Documentation must match actual crate structure  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change on line 541:**
```markdown
# Before:
- `standalone` — Standalone desktop app for browser-based development

# After:
- `wavecraft-dev-server` — Development server for browser-based UI testing
```

---

#### 4.3 Update high-level-design.md diagram
**File:** `docs/architecture/high-level-design.md`  
**Why:** Architecture diagram must reflect actual component names  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change on line 723:**
```markdown
# Before:
  │ Standalone Dev      │                │ Plugin Binary       │

# After:
  │ Dev Server          │                │ Plugin Binary       │
```

---

#### 4.4 Update high-level-design.md command example
**File:** `docs/architecture/high-level-design.md`  
**Why:** Command examples must work  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change on line 794:**
```markdown
# Before:
cargo run -p standalone -- --dev-server --port 9000

# After:
cargo run -p wavecraft-dev-server -- --dev-server --port 9000
```

---

#### 4.5 Update roadmap.md crate references
**File:** `docs/roadmap.md`  
**Why:** Roadmap updates are PO-only  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Note:** Skip in coder work; hand off to PO if needed.

---

#### 4.6 Update CLI dev server reuse specs
**Files:**
- `docs/feature-specs/cli-dev-server-reuse/implementation-plan.md`
- `docs/feature-specs/cli-dev-server-reuse/implementation-progress.md`
**Why:** Active specs reference the old crate name  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change examples:**
```markdown
- Optional cleanups cover meter frame type unification and wavecraft-dev-server host reuse.
- Align wavecraft-dev-server host with shared host
```

---

#### 4.7 Update embedded dev server specs
**Files:**
- `docs/feature-specs/embedded-dev-server/low-level-design-embedded-dev-server.md`
- `docs/feature-specs/embedded-dev-server/implementation-plan.md`
- `docs/feature-specs/embedded-dev-server/implementation-progress.md`
**Why:** Active specs reference the old crate name and paths  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Change examples:**
```markdown
use wavecraft_dev_server::ws_server::WsServer;
wavecraft-dev-server = { path = "../engine/crates/wavecraft-dev-server" }
```

---

#### 4.8 Update audio-input-via-wasm spec
**File:** `docs/feature-specs/audio-input-via-wasm/high-level-design.md`  
**Why:** Spec references the old "standalone" dev server name  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Update examples:**
```markdown
Currently, the only way to test with real audio is through the dev server (`cargo xtask dev`), which:
```

---

#### 4.9 Update WebSocketTransport comments
**File:** `ui/packages/core/src/transports/WebSocketTransport.ts`  
**Why:** Code comments should reference correct component  
**Dependencies:** Phase 1 complete  
**Risk:** Low

**Update lines 4 and 31 — replace "standalone dev server" with "dev server" or "wavecraft-dev-server"**

---

### Phase 5: Final Verification

#### 5.1 Full workspace build
**Action:** Terminal command  
**Why:** Ensure all crates compile  
**Dependencies:** All phases complete  
**Risk:** Low

```bash
cd engine && cargo build --workspace
```

---

#### 5.2 Full test suite
**Action:** Terminal command  
**Why:** Ensure no regressions  
**Dependencies:** Step 5.1  
**Risk:** Low

```bash
cargo xtask check
```

---

#### 5.3 Manual dev server test
**Action:** Terminal command  
**Why:** Verify xtask dev works end-to-end  
**Dependencies:** Steps 5.1, 5.2  
**Risk:** Low

```bash
cargo xtask dev
# Verify: WebSocket server starts, UI connects successfully
# Press Ctrl+C to stop
```

---

#### 5.4 Check help output
**Action:** Terminal command  
**Why:** Verify CLI metadata displays correctly  
**Dependencies:** Step 5.1  
**Risk:** Low

```bash
cargo run -p wavecraft-dev-server -- --help
# Verify: Shows "wavecraft-dev-server" and correct description
```

---

## Testing Strategy

### Unit Tests
- `cargo test -p wavecraft-dev-server` — Verify all crate tests pass

### Integration Tests  
- `cargo xtask dev` — Verify dev server starts and UI connects

### E2E Tests
- Manual browser test at http://localhost:5173 with `cargo xtask dev` running

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Missed references | Comprehensive grep search completed; final verification step |
| Build breakage | Atomic Phase 1 ensures crate compiles before other changes |
| Git history loss | Using `git mv` preserves history |

---

## Success Criteria

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes
- [ ] `cargo xtask check` passes
- [ ] `cargo xtask dev` starts both servers correctly
- [ ] `cargo run -p wavecraft-dev-server -- --help` shows correct name
- [ ] No "standalone" references in active code/docs (except archived specs)
