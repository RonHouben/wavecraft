# Low-Level Design: Rename `standalone` → `wavecraft-dev-server`

**Status:** Draft  
**Created:** 2026-02-06  
**Author:** Architect Agent

---

## Summary

Rename the `standalone` crate to `wavecraft-dev-server` to better communicate its purpose as a development server for testing plugin UIs against the real Rust backend.

---

## Rationale

| Current Name | Issue |
|--------------|-------|
| `standalone` | Ambiguous — doesn't convey what it does |
| | Conflicts with "standalone plugin" terminology (which means running a plugin outside a DAW) |
| | Inconsistent with `wavecraft-*` crate naming pattern |

| New Name | Benefit |
|----------|---------|
| `wavecraft-dev-server` | Clear purpose — "dev-server" is universally understood |
| | Follows `wavecraft-*` crate naming convention |
| | Aligns with `cargo xtask dev` command |

---

## Scope

### In Scope

1. Rename crate folder and package metadata
2. Update all code references (imports, comments, CLI)
3. Update xtask commands
4. Update active documentation
5. Update CLI (`wavecraft` command)

### Out of Scope

- Archived feature specs (per project guidelines)
- Semantic/behavioral changes to the crate
- API changes

---

## Change Inventory

### 1. Crate Folder Rename

```
engine/crates/standalone/  →  engine/crates/wavecraft-dev-server/
```

### 2. Cargo Configuration

| File | Change |
|------|--------|
| `engine/crates/wavecraft-dev-server/Cargo.toml` | `name = "wavecraft-dev-server"`, `[[bin]] name = "wavecraft-dev-server"` |
| `engine/Cargo.toml` | Update workspace member path |
| `engine/Cargo.lock` | Auto-regenerated |

### 3. Source Files (within the crate)

| File | Change |
|------|--------|
| `src/lib.rs` | Update doc comment |
| `src/main.rs` | Update doc comments, CLI `#[command(name = "...")]`, log messages |
| `tests/integration_test.rs` | `use wavecraft_dev_server::AppState;` |
| `tests/latency_bench.rs` | `use wavecraft_dev_server::AppState;` |
| `README.md` | Update title and references |

### 4. xtask Commands

| File | Change |
|------|--------|
| `engine/xtask/src/commands/dev.rs` | `"-p", "wavecraft-dev-server"` |
| `engine/xtask/src/commands/desktop.rs` | Update doc comment |

### 5. CLI Tool

| File | Change |
|------|--------|
| `cli/src/commands/start.rs` | `"-p", "wavecraft-dev-server"` |

### 6. Documentation (Active)

| File | Changes Required |
|------|------------------|
| `README.md` (root) | Update crate list |
| `docs/architecture/coding-standards.md` | Update crate list |
| `docs/architecture/high-level-design.md` | Update diagram, `cargo run -p` example |
| `docs/roadmap.md` | Update crate references |
| `docs/feature-specs/cli-start-command/*.md` | Update command examples |
| `docs/feature-specs/internal-testing/user-stories.md` | Update command examples |
| `ui/packages/core/src/transports/WebSocketTransport.ts` | Update comments |

### 7. Documentation (Archived) — DO NOT MODIFY

The following archived files reference `standalone` but **must not be modified** per project guidelines:

- `docs/feature-specs/_archive/websocket-ipc-bridge/implementation-progress.md`
- `docs/feature-specs/_archive/websocket-ipc-bridge/implementation-plan.md`

---

## Implementation Steps

### Phase 1: Crate Rename (Atomic)

These changes must be made together to avoid build breakage:

1. **Rename folder**: `git mv engine/crates/standalone engine/crates/wavecraft-dev-server`

2. **Update `engine/crates/wavecraft-dev-server/Cargo.toml`**:
   ```toml
   [package]
   name = "wavecraft-dev-server"
   # ... rest unchanged ...
   description = "Development server for Wavecraft plugin UI testing"
   
   [[bin]]
   name = "wavecraft-dev-server"
   path = "src/main.rs"
   ```

3. **Update `engine/Cargo.toml`** workspace member:
   ```toml
   wavecraft-dev-server = { path = "crates/wavecraft-dev-server" }
   ```

4. **Update test imports**:
   ```rust
   // tests/integration_test.rs, tests/latency_bench.rs
   use wavecraft_dev_server::AppState;
   ```

5. **Verify build**: `cargo build -p wavecraft-dev-server`

### Phase 2: xtask & CLI Updates

1. **Update `engine/xtask/src/commands/dev.rs`**:
   ```rust
   "wavecraft-dev-server",
   ```

2. **Update `cli/src/commands/start.rs`**:
   ```rust
   "wavecraft-dev-server",
   ```

3. **Verify**: `cargo xtask dev` and `wavecraft start` (if applicable)

### Phase 3: Source Comments & CLI Metadata

1. **Update `src/main.rs`**:
   - Doc comment: `//! Wavecraft dev server entry point`
   - `#[command(name = "wavecraft-dev-server")]`
   - `#[command(about = "Wavecraft development server for UI testing")]`
   - Log messages referencing "VstKit" → "Wavecraft"

2. **Update `src/lib.rs`**:
   - Doc comment: `//! Wavecraft dev server library`

3. **Update `README.md`** (within crate):
   - Title: `# Wavecraft Dev Server`
   - Description updates

### Phase 4: Documentation Updates

Update all active documentation files listed in the Change Inventory section.

---

## Verification Checklist

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes
- [ ] `cargo xtask dev` starts the dev server correctly
- [ ] `wavecraft start` (CLI) works in a scaffolded project
- [ ] `cargo run -p wavecraft-dev-server -- --help` shows correct name
- [ ] No broken links in documentation
- [ ] Git history preserved via `git mv`

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Stale references missed | Low | Medium | Comprehensive grep search completed |
| User projects break | None | N/A | Crate is `publish = false`, not consumed externally |
| CI failures | Low | Low | Single atomic PR with build verification |

---

## Binary Name Consideration

### Option A: Match crate name (recommended)
```
Binary: wavecraft-dev-server
Invocation: cargo run -p wavecraft-dev-server -- --dev-server
```

### Option B: Short alias
```
Binary: wavecraft-dev
Invocation: cargo run -p wavecraft-dev-server --bin wavecraft-dev -- --dev-server
```

**Recommendation**: Option A for simplicity. The full name `wavecraft-dev-server` clearly identifies the binary in process lists and logs.

---

## Process Kill Commands Update

The terminal command `pkill -f "standalone"` will need to change to `pkill -f "wavecraft-dev-server"` in developer workflows.

---

## Related Documents

- [High-Level Design](../../architecture/high-level-design.md) — Architecture overview
- [Coding Standards](../../architecture/coding-standards.md) — Crate naming conventions
