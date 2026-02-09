# Coding Standards

This document defines the coding standards and conventions for the Wavecraft project.
For detailed rules, see the topic-specific documents below.

---

## Documentation Structure

| Document | Scope | When to Read |
|----------|-------|-------------|
| [TypeScript & React](./coding-standards-typescript.md) | Classes, hooks, React components, build constants, imports | Writing TypeScript or React code |
| [CSS & Styling](./coding-standards-css.md) | TailwindCSS, theme tokens, WebView background | Styling or theming work |
| [Rust](./coding-standards-rust.md) | Module org, DSL conventions, real-time safety, FFI, xtask | Writing Rust code |
| [Testing & Quality](./coding-standards-testing.md) | Testing, linting, logging, error handling | Writing tests, debugging, CI |

---

## Quick Reference — Naming Conventions

> For full details, see the language-specific guides in the [Documentation Structure](#documentation-structure) table above.

### TypeScript / JavaScript

| Type | Convention | Example |
|------|------------|---------|
| Classes | PascalCase | `IpcBridge`, `ParameterClient` |
| Interfaces | PascalCase | `ParameterInfo`, `IpcError` |
| Type aliases | PascalCase | `EventCallback`, `RequestId` |
| Methods | camelCase | `getParameter`, `setReceiveCallback` |
| Private members | camelCase (no underscore prefix) | `private requestId` |
| Constants | UPPER_SNAKE_CASE | `DEFAULT_TIMEOUT_MS` |
| React components | PascalCase | `ParameterSlider` |
| React hooks | camelCase with `use` prefix | `useParameter` |

### Rust

| Type | Convention | Example |
|------|------------|---------|
| Structs | PascalCase | `IpcHandler`, `AppState` |
| Traits | PascalCase | `ParameterHost` |
| Functions | snake_case | `handle_request`, `get_parameter` |
| Methods | snake_case | `fn set_sample_rate(&mut self)` |
| Constants | UPPER_SNAKE_CASE | `const WINDOW_WIDTH: u32` |
| Modules | snake_case | `mod params`, `mod handler` |

---

## General

### Versioning

**Rule:** All version bumping is handled automatically by the CD pipeline. Do not manually bump versions during feature development.

See [Versioning and Distribution](./versioning-and-distribution.md) for the full version flow, build-time injection, and packaging details.

### Comments and Documentation

- Use `///` doc comments for public APIs
- Include examples in doc comments where helpful
- Keep comments up-to-date with code changes

### Documentation References

**Rule:** Always link to relevant documentation in the `docs/` folder.

All project documentation (README, specs, design docs) must include links to related architecture documents. This ensures discoverability and keeps documentation interconnected.

**Required links:**
- [High-Level Design](./high-level-design.md) — Architecture overview, component design, data flows
- [Coding Standards](./coding-standards.md) — Code conventions and patterns (this document)
- [Roadmap](../roadmap.md) — Milestone tracking and progress
- [SDK Architecture](./sdk-architecture.md) — SDK distribution, crate structure, npm packages
- [Development Workflows](./development-workflows.md) — Browser dev mode, build system, CI/CD
- [Plugin Formats](./plugin-formats.md) — VST3, CLAP, AU architecture

**Do:**
```markdown
## Documentation

- [High-Level Design](docs/architecture/high-level-design.md) — Architecture overview
- [Coding Standards](docs/architecture/coding-standards.md) — Code conventions
- [Roadmap](docs/roadmap.md) — Implementation progress
```

**Don't:**
```markdown
## Documentation

See the docs folder for more information.
```

---

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview
- [Agent Development Flow](./agent-development-flow.md) — Agent roles and handoffs
- [Roadmap](../roadmap.md) — Milestone tracking

