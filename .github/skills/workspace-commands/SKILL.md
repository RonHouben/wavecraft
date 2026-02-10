---
name: workspace-commands
description: Standardizes terminal command execution in the Wavecraft workspace. Use this skill whenever running cargo, npm, or other CLI commands to ensure they execute from the correct directory.
---

# Workspace Commands

This skill ensures all terminal commands execute from the correct directory in the Wavecraft monorepo.

## Workspace Root

**All cargo commands MUST run from the workspace root:**
```
/Users/ronhouben/code/private/wavecraft
```

**Why?** The workspace has a specific structure with multiple Cargo.toml files:
- Root: `Cargo.toml` (workspace manifest)
- CLI: `cli/Cargo.toml`
- Engine: `engine/Cargo.toml`
- xtask: `engine/xtask/Cargo.toml`

Running cargo from the wrong directory causes "package not found" or wrong target errors.

## Command Patterns

### Cargo Commands (Rust)

**Always run from workspace root:**
```bash
# xtask commands (preferred for CI tasks)
cargo xtask ci-check
cargo xtask lint
cargo xtask test
cargo xtask dev
cargo xtask bundle

# Direct cargo commands
cargo build --release
cargo test --all
cargo clippy --all
cargo fmt --all
```

**If you're in a subdirectory, cd to root first:**
```bash
cd /Users/ronhouben/code/private/wavecraft && cargo xtask ci-check
```

### npm Commands (UI)

**Run from `ui/` directory:**
```bash
cd /Users/ronhouben/code/private/wavecraft/ui && npm test
cd /Users/ronhouben/code/private/wavecraft/ui && npm run typecheck
cd /Users/ronhouben/code/private/wavecraft/ui && npm run lint
```

**Or from workspace root with explicit path:**
```bash
cd /Users/ronhouben/code/private/wavecraft
npm --prefix ui test
npm --prefix ui run typecheck
```

### wavecraft CLI Commands

**Run from generated plugin directories:**
```bash
# When testing CLI-generated plugins
cd /path/to/generated-plugin && wavecraft start
```

**For SDK development (path dependencies):**
```bash
cd /Users/ronhouben/code/private/wavecraft
cargo run --manifest-path cli/Cargo.toml -- create TestPlugin --output target/tmp/test-plugin
```

## Common Mistakes to Avoid

| Mistake | Why It Fails | Correct Approach |
|---------|--------------|------------------|
| `cargo test` from `ui/` | No Cargo.toml | `cd /Users/ronhouben/code/private/wavecraft && cargo test` |
| `npm test` from workspace root | No package.json | `cd ui && npm test` or `npm --prefix ui test` |
| `cargo xtask` from `engine/` | xtask manifest location | Run from workspace root |
| `wavecraft start` from SDK repo | Wrong plugin context | Run from a plugin project |

## Quick Reference

| Command Type | Run From | Example |
|--------------|----------|---------|
| `cargo xtask *` | Workspace root | `cargo xtask ci-check` |
| `cargo *` | Workspace root | `cargo build --release` |
| `npm *` | `ui/` directory | `cd ui && npm test` |
| `wavecraft *` | Plugin project | `cd my-plugin && wavecraft start` |

## Verification

Before running any command, verify your working directory:
```bash
pwd  # Should show the expected directory
```

When in doubt, use absolute paths:
```bash
cd /Users/ronhouben/code/private/wavecraft && cargo xtask ci-check
```
