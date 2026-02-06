# Implementation Plan: CLI Start Command (`wavecraft start`)

**Feature:** `wavecraft start` command for unified development experience  
**Version:** 0.8.0  
**Created:** 2026-02-06  
**Based on:** [Low-Level Design](./low-level-design-cli-start-command.md)

---

## Overview

This plan implements a `wavecraft start` command that:
1. Detects Wavecraft projects by directory structure
2. Checks for npm dependencies and prompts to install
3. Spawns WebSocket + Vite dev servers with graceful shutdown
4. Updates `wavecraft create` output to reference the new command

**Estimated effort:** 2-3 hours

---

## Requirements

- [User Stories](./user-stories.md)
- [Low-Level Design](./low-level-design-cli-start-command.md)

---

## Architecture Changes

| File | Change Type | Description |
|------|-------------|-------------|
| `cli/src/project/mod.rs` | **New** | Module declaration for project utilities |
| `cli/src/project/detection.rs` | **New** | Project detection logic (`ProjectMarkers`) |
| `cli/src/commands/start.rs` | **New** | Start command implementation |
| `cli/src/commands/mod.rs` | **Modify** | Export `StartCommand` |
| `cli/src/main.rs` | **Modify** | Add `Start` variant to `Commands` enum |
| `cli/src/commands/new.rs` | **Modify** | Update "Next steps" message |
| `cli/Cargo.toml` | **Modify** | Add `ctrlc` and `nix` dependencies |

---

## Implementation Steps

### Phase 1: Project Detection Module
**Goal:** Create utility to detect valid Wavecraft projects

#### Step 1.1: Create project module structure
**File:** `cli/src/project/mod.rs`
- Action: Create new module file with `pub mod detection;`
- Why: Organizes project-related utilities
- Dependencies: None
- Risk: Low

#### Step 1.2: Implement ProjectMarkers struct
**File:** `cli/src/project/detection.rs`
- Action: Create `ProjectMarkers` struct with `detect()` method
- Why: Core logic for validating project structure
- Dependencies: Step 1.1
- Risk: Low

**Key implementation details:**
- Check for `ui/` and `engine/` directories
- Verify `ui/package.json` and `engine/Cargo.toml` exist
- Return descriptive errors when validation fails
- Add `has_node_modules()` helper function

#### Step 1.3: Add unit tests for detection
**File:** `cli/src/project/detection.rs`
- Action: Add tests using `tempfile` crate
- Why: Ensure detection logic is correct
- Dependencies: Step 1.2
- Risk: Low

**Test cases:**
- Valid project structure → Ok
- Missing `ui/` → Error with helpful message
- Missing `engine/` → Error with helpful message
- Missing `package.json` → Error
- `node_modules` present/absent detection

---

### Phase 2: Start Command Implementation
**Goal:** Implement the core `wavecraft start` command

#### Step 2.1: Add dependencies to Cargo.toml
**File:** `cli/Cargo.toml`
- Action: Add `ctrlc = "3"` and platform-specific `nix` dependency
- Why: Required for signal handling and process management
- Dependencies: None
- Risk: Low

```toml
[dependencies]
ctrlc = "3"

[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["signal"] }
```

#### Step 2.2: Create StartCommand struct
**File:** `cli/src/commands/start.rs`
- Action: Create module with `StartCommand` struct and fields
- Why: Holds command options (port, ui_port, install flags, verbose)
- Dependencies: Step 2.1
- Risk: Low

**Fields:**
- `port: u16` (default 9000)
- `ui_port: u16` (default 5173)
- `install: bool`
- `no_install: bool`
- `verbose: bool`

#### Step 2.3: Implement dependency check flow
**File:** `cli/src/commands/start.rs`
- Action: Add `prompt_install()` and `install_dependencies()` functions
- Why: Handles missing npm dependencies gracefully
- Dependencies: Steps 1.2, 2.2
- Risk: Low

**Flow:**
1. Check `has_node_modules()`
2. If missing + `--no-install` → fail with message
3. If missing + `--install` → auto-install
4. If missing + interactive → prompt user
5. Run `npm install` if confirmed

#### Step 2.4: Implement server spawning
**File:** `cli/src/commands/start.rs`
- Action: Add `run_dev_servers()` function
- Why: Core functionality - spawns both development servers
- Dependencies: Step 2.3
- Risk: Medium

**Details:**
- Spawn `cargo run -p standalone --release -- --dev-server --port <PORT>` in engine/
- Sleep 500ms for server startup
- Spawn `npm run dev -- --port=<PORT>` in ui/
- Print success message with URLs

#### Step 2.5: Implement graceful shutdown
**File:** `cli/src/commands/start.rs`
- Action: Add `wait_for_shutdown()` and `kill_process()` functions
- Why: Clean process termination on Ctrl+C
- Dependencies: Step 2.4
- Risk: Medium

**Details:**
- Use `ctrlc` crate for signal handling
- On Unix: send SIGTERM to process group (kills children)
- On Windows: call `child.kill()` directly
- Print shutdown status messages

#### Step 2.6: Wire up execute method
**File:** `cli/src/commands/start.rs`
- Action: Implement `StartCommand::execute()`
- Why: Entry point that orchestrates the full flow
- Dependencies: Steps 2.3, 2.4, 2.5
- Risk: Low

**Flow:**
1. Get current directory
2. Call `ProjectMarkers::detect()`
3. Handle dependency check
4. Call `run_dev_servers()`

---

### Phase 3: CLI Integration
**Goal:** Wire the new command into the CLI

#### Step 3.1: Export StartCommand
**File:** `cli/src/commands/mod.rs`
- Action: Add `pub mod start;` and `pub use start::StartCommand;`
- Why: Makes command available to main.rs
- Dependencies: Phase 2
- Risk: Low

#### Step 3.2: Add project module to main
**File:** `cli/src/main.rs`
- Action: Add `mod project;` declaration
- Why: Makes project detection available
- Dependencies: Phase 1
- Risk: Low

#### Step 3.3: Add Start variant to Commands enum
**File:** `cli/src/main.rs`
- Action: Add `Start { ... }` variant with clap attributes
- Why: Defines CLI interface for the command
- Dependencies: Step 3.1
- Risk: Low

**Arguments:**
```rust
Start {
    #[arg(short, long, default_value_t = 9000)]
    port: u16,
    
    #[arg(long, default_value_t = 5173)]
    ui_port: u16,
    
    #[arg(long, conflicts_with = "no_install")]
    install: bool,
    
    #[arg(long, conflicts_with = "install")]
    no_install: bool,
    
    #[arg(short, long)]
    verbose: bool,
}
```

#### Step 3.4: Handle Start command in main
**File:** `cli/src/main.rs`
- Action: Add match arm for `Commands::Start { ... }`
- Why: Routes to StartCommand::execute()
- Dependencies: Step 3.3
- Risk: Low

---

### Phase 4: Update New Command Output
**Goal:** Update next steps to reference `wavecraft start`

#### Step 4.1: Update success message
**File:** `cli/src/commands/new.rs`
- Action: Change "cargo xtask dev" to "wavecraft start"
- Why: Consistent CLI experience
- Dependencies: None
- Risk: Low

**Before:**
```rust
println!("  cargo xtask dev    # Start development servers");
```

**After:**
```rust
println!("  wavecraft start    # Start development servers");
```

---

### Phase 5: Version Bump & Testing
**Goal:** Finalize version and verify functionality

#### Step 5.1: Bump CLI version
**File:** `cli/Cargo.toml`
- Action: Change version from "0.7.2" to "0.8.0"
- Why: New feature warrants minor version bump
- Dependencies: All previous phases
- Risk: Low

#### Step 5.2: Run unit tests
- Action: `cd cli && cargo test`
- Why: Verify all tests pass including new detection tests
- Dependencies: Step 5.1
- Risk: Low

#### Step 5.3: Manual integration test
- Action: Create test plugin and run `wavecraft start`
- Why: End-to-end verification
- Dependencies: Step 5.2
- Risk: Low

**Test steps:**
1. `wavecraft create test_plugin`
2. `cd test_plugin`
3. `wavecraft start --install`
4. Verify both servers start
5. Press Ctrl+C, verify clean shutdown

#### Step 5.4: Test error cases
- Action: Verify error messages for edge cases
- Dependencies: Step 5.3
- Risk: Low

**Test cases:**
- Run from non-project directory → "Not a Wavecraft project" error
- Run with `--no-install` when deps missing → helpful error
- Custom ports → servers use specified ports

---

## Testing Strategy

### Unit Tests
| Test | Location | Description |
|------|----------|-------------|
| `test_project_detection_valid` | `detection.rs` | Valid project structure detected |
| `test_project_detection_missing_ui` | `detection.rs` | Error when ui/ missing |
| `test_project_detection_missing_engine` | `detection.rs` | Error when engine/ missing |
| `test_has_node_modules_true` | `detection.rs` | Detects installed deps |
| `test_has_node_modules_false` | `detection.rs` | Detects missing deps |

### Manual Tests
| Test | Steps | Expected |
|------|-------|----------|
| Happy path | `wavecraft create test && cd test && wavecraft start --install` | Both servers start |
| Custom ports | `wavecraft start -p 8000 --ui-port 3000` | Uses custom ports |
| Not a project | `cd /tmp && wavecraft start` | Clear error message |
| Missing deps | `wavecraft start --no-install` | Error about missing deps |
| Ctrl+C shutdown | Start servers, press Ctrl+C | Both servers stop cleanly |

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| npm not installed | Low | Medium | Check for npm in PATH before spawning |
| Port already in use | Medium | Low | User can specify custom ports |
| Windows process handling | Medium | Medium | Simple kill() fallback works |

---

## Success Criteria

- [ ] `wavecraft start` works from a scaffolded plugin directory
- [ ] Prompts to install dependencies when missing
- [ ] `--install` auto-installs without prompt
- [ ] `--no-install` fails fast with helpful message
- [ ] Custom ports work (`-p`, `--ui-port`)
- [ ] Ctrl+C cleanly stops both servers
- [ ] Clear error when run outside a Wavecraft project
- [ ] `wavecraft create` shows updated next steps
- [ ] All unit tests pass
- [ ] Version bumped to 0.8.0
