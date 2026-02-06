# Low-Level Design: Template Reorganization

**Status:** Draft  
**Created:** 2026-02-06  
**Updated:** 2026-02-06  
**Author:** Architect Agent

---

## Problem Statement

The `plugin-template/` directory was moved from the repository root into `cli/plugin-template/` to fix a `cargo publish` packaging issue. This fix is complete, but:

1. The name `plugin-template` is ambiguous
2. The structure doesn't support future template variants (vanilla UI, different frameworks)
3. Documentation still references the old location

### Proposed Structure

```
cli/
├── src/                              # CLI source code
└── sdk-templates/                    # All SDK templates
    └── new-project/                  # Templates for `wavecraft new`
        └── react/                    # React UI variant (current default)
            ├── Cargo.toml.template
            ├── engine/
            └── ui/
```

### Why This Structure

| Aspect | Rationale |
|--------|-----------|
| `sdk-templates/` | Clear that these are SDK assets, not CLI internals |
| `new-project/` | Maps to CLI command (`wavecraft new`) |
| `react/` | Explicit UI framework choice; extensible for `vanilla/`, `svelte/` |

### Change Summary

| Before | After |
|--------|-------|
| `/plugin-template/` (root) | — |
| `/cli/plugin-template/` | `/cli/sdk-templates/new-project/react/` |

---

## Scope

This LLD covers:

1. **Directory rename**: `cli/plugin-template/` → `cli/sdk-templates/new-project/react/`
2. **Code updates**: Template extraction path in CLI
3. **CI updates**: Path filters in workflows
4. **Documentation updates**: All references to old path

---

## Files Requiring Updates

### Code Changes

| File | Change Required |
|------|-----------------|
| `cli/src/template/mod.rs` | Update `include_dir!` path |

### CI/CD Changes

| File | Change Required |
|------|-----------------|
| `.github/workflows/continuous-deploy.yml` | Update path filter for CLI changes |

### Documentation (High-Priority)

| File | Change Required |
|------|-----------------|
| [README.md](../../README.md) | Update repository structure diagram |
| [high-level-design.md](../architecture/high-level-design.md) | Update monorepo structure |
| [ci-pipeline.md](../guides/ci-pipeline.md) | Update path filter table |

### Documentation (Medium-Priority)

| File | Change Required |
|------|-----------------|
| [backlog.md](../backlog.md) | Update `xtask clean` entry |
| `feature-specs/internal-testing/test-plan.md` | Update path reference |
| `feature-specs/cli-publish-fix/test-plan.md` | Update path references |

### Archived Files (Do Not Modify)

These files in `_archive/` preserve historical context and should **not** be updated:

- `_archive/declarative-plugin-dsl/*`
- `_archive/developer-sdk/*`

---

## Detailed Changes

### 1. Directory Rename

```bash
# Execute
mv cli/plugin-template cli/sdk-templates
mkdir -p cli/sdk-templates/new-project
mv cli/sdk-templates/* cli/sdk-templates/new-project/ 2>/dev/null || true
mv cli/sdk-templates/new-project cli/sdk-templates/new-project-tmp
mkdir -p cli/sdk-templates/new-project
mv cli/sdk-templates/new-project-tmp cli/sdk-templates/new-project/react

# Resulting structure:
cli/sdk-templates/new-project/react/
├── Cargo.toml.template
├── LICENSE
├── README.md
├── engine/
│   ├── Cargo.toml.template
│   ├── build.rs
│   ├── src/
│   └── xtask/
└── ui/
    ├── package.json
    ├── src/
    └── ...
```

---

### 2. cli/src/template/mod.rs — Update include_dir! Path

**Current:**
```rust
static TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/plugin-template");
```

**Updated:**
```rust
static TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/sdk-templates/new-project/react");
```

---

### 3. continuous-deploy.yml — Update Path Filter

**Current:**
```yaml
cli:
  - 'cli/src/**'
  - 'cli/Cargo.toml'
  - 'cli/plugin-template/**'
```

**Updated:**
```yaml
cli:
  - 'cli/src/**'
  - 'cli/Cargo.toml'
  - 'cli/sdk-templates/**'
```

---

### 4. README.md — Repository Structure

**Current:**
```markdown
├── plugin-template/              # Template project for SDK users
```

**Updated:**
```markdown
├── cli/                          # Wavecraft CLI tool
│   ├── src/                      # CLI source code
│   └── sdk-templates/            # Project templates
│       └── new-project/react/    # React UI template (default)
```

---

### 5. high-level-design.md — Monorepo Structure

**Current (line ~71):**
```markdown
└── plugin-template/               # Plugin template (scaffolded by CLI)
```

**Updated:**
```markdown
wavecraft/
├── cli/                           # CLI tool (cargo install wavecraft)
│   ├── src/                       # CLI source code
│   │   ├── main.rs                # Entry point, clap CLI
│   │   ├── validation.rs          # Crate name validation (syn-based)
│   │   ├── commands/              # Command implementations
│   │   └── template/              # Template extraction & variables
│   └── sdk-templates/             # Embedded project templates
│       └── new-project/           # `wavecraft new` templates
│           └── react/             # React UI variant (default)
│               ├── Cargo.toml.template
│               ├── engine/        # Rust audio engine template
│               └── ui/            # React UI template
```

Remove the standalone `plugin-template/` entry from the root level.

---

### 6. ci-pipeline.md — Path Filter Table

**Current (line ~348):**
```markdown
| `publish-cli` | `cli/**`, `plugin-template/**` | crates.io (`wavecraft`) |
```

**Updated:**
```markdown
| `publish-cli` | `cli/**` (includes `sdk-templates/`) | crates.io (`wavecraft`) |
```

---

### 7. backlog.md — xtask clean Entry

**Current (line ~20):**
```markdown
| Extend `cargo xtask clean` to cover full workspace | Currently only cleans `engine/target`. Should also clean `cli/target`, `plugin-template/target`, and `ui/node_modules`+`ui/dist`. |
```

**Updated:**
```markdown
| Extend `cargo xtask clean` to cover full workspace | Currently only cleans `engine/target`. Should also clean `cli/target` and `ui/node_modules`+`ui/dist`. |
```

---

### 8. Test Plans — Update Path References

**internal-testing/test-plan.md (line ~206):**
```markdown
# Current
- ✅ Template location: `/Users/ronhouben/code/private/wavecraft/plugin-template`

# Updated
- ✅ Template location: `/Users/ronhouben/code/private/wavecraft/cli/sdk-templates/new-project/react`
```

**cli-publish-fix/test-plan.md:**
Update all references from `cli/plugin-template/` to `cli/sdk-templates/new-project/react/`.

---

## Future Extensibility

This structure supports future template variants:

```
cli/sdk-templates/
├── new-project/
│   ├── react/           # Current default
│   ├── vanilla/         # Future: DOM-only, no React
│   └── svelte/          # Future: Svelte UI
└── examples/            # Future: working example projects
    ├── gain/
    └── eq/
```

CLI could eventually support:
```bash
wavecraft new my-plugin --template vanilla
wavecraft new my-plugin --template react    # default
```

---

## Validation

After implementation, verify:

1. **CLI builds**: `cd cli && cargo build`
2. **Template extracts**: `wavecraft new test-plugin --vendor Test --no-git`
3. **Generated project builds**: `cd test-plugin && cargo xtask bundle`
4. **CI passes**: Push to branch and verify workflow triggers correctly

---

## Effort Estimate

| Task | Effort |
|------|--------|
| Directory rename | 2 min |
| Code update (template/mod.rs) | 2 min |
| CI workflow update | 2 min |
| Documentation updates | 15 min |
| Testing & validation | 10 min |
| **Total** | **~30 min** |

---

## Non-Changes (Rationale)

### Archived Feature Specs

Per [coding standards](../architecture/coding-standards.md), archived specs in `docs/feature-specs/_archive/` are **never modified**. They preserve historical context.

---

## Related Documents

- [CLI Publish Fix Test Plan](../cli-publish-fix/test-plan.md) — Original fix implementation
- [High-Level Design](../architecture/high-level-design.md) — Main architecture document
- [Coding Standards](../architecture/coding-standards.md) — Rules on archived file modification
