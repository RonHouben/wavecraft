# Low-Level Design: Architecture Documentation Split

## Related Documents

- [User Stories](./user-stories.md) — Requirements and acceptance criteria
- [High-Level Design (current)](../../architecture/high-level-design.md) — Source document for split
- [Coding Standards (current)](../../architecture/coding-standards.md) — Source document for split
- [Agent Development Flow](../../architecture/agent-development-flow.md) — Unchanged (254 lines, within target)

---

## 1. Design Overview

### Problem

Two architecture documents have grown beyond practical limits:

| Document | Lines | Sections | Topics |
|----------|-------|----------|--------|
| `coding-standards.md` | 1,502 | 40+ | TypeScript, CSS, Rust, Testing, Linting, Logging, Error handling |
| `high-level-design.md` | 1,580 | 50+ | Architecture, SDK, DSL, Workflows, Formats, Versioning, Tooling |

This causes:
- AI agents loading 3,000–6,000 tokens per file when they need 200–600
- Developers scrolling through 1,500+ lines to find a specific rule
- `.github/copilot-instructions.md` attaching entire monolithic files as context

### Solution

Split each document into focused topic files (150–600 lines each), keep the originals as navigation hubs (~200–400 lines), and update all cross-references.

### Target State

```
docs/architecture/
├── agent-development-flow.md          (254 lines — unchanged)
├── coding-standards.md                (~200 lines — hub with navigation)
├── coding-standards-typescript.md     (~400 lines — TS, React, hooks, build)
├── coding-standards-css.md            (~150 lines — Tailwind, theming)
├── coding-standards-rust.md           (~550 lines — modules, DSL, FFI, RT-safety)
├── coding-standards-testing.md        (~350 lines — testing, linting, logging, errors)
├── high-level-design.md               (~400 lines — overview hub)
├── sdk-architecture.md                (~500 lines — SDK, crates, npm, API)
├── declarative-plugin-dsl.md          (~300 lines — DSL, macros, params)
├── development-workflows.md           (~350 lines — browser dev, FFI, build, visual testing)
├── plugin-formats.md                  (~300 lines — VST3, CLAP, AU, testing matrix)
└── versioning-and-distribution.md     (~200 lines — versioning, packaging, signing)
```

Total: 12 files, down from 3. All within 150–600 line target.

---

## 2. Coding Standards Split Plan

### 2.1 Source: `coding-standards.md` (1,502 lines)

#### Section Inventory

| # | Section | Lines | Destination |
|---|---------|-------|-------------|
| 1 | Title + intro (L1–5) | 5 | Hub |
| 2 | `## TypeScript / JavaScript` (L7–389) | 383 | `coding-standards-typescript.md` |
| 3 | `## CSS / Styling (TailwindCSS)` (L391–527) | 137 | `coding-standards-css.md` |
| 4 | `## Rust` (L529–987) | 459 | `coding-standards-rust.md` |
| 5 | `## Testing` (L991–1155) | 165 | `coding-standards-testing.md` |
| 6 | `## Linting & Formatting` (L1157–1223) | 67 | `coding-standards-testing.md` |
| 7 | `## General` (L1226–end) | 276 | Split (see below) |

**Section 7 breakdown** (`## General`, L1226–1502):

| Subsection | Lines | Destination |
|------------|-------|-------------|
| `### Versioning` (L1228–1264) | 37 | Hub (brief summary + link to `versioning-and-distribution.md`) |
| `### Comments and Documentation` (L1266–1270) | 5 | Hub |
| `### Documentation References` (L1272–1297) | 26 | Hub |
| `### Logging` (L1299–1373) | 75 | `coding-standards-testing.md` |
| `### Error Handling` (L1375–1379) | 5 | `coding-standards-testing.md` |
| `### Validation Against Language Specifications` (L1381–1433) | 53 | `coding-standards-rust.md` |
| `### Rust unwrap() and expect() Usage` (L1435–end) | 67 | `coding-standards-rust.md` |

---

### 2.2 New File: `coding-standards-typescript.md` (~400 lines)

**Source sections (from `coding-standards.md`):**
- `## TypeScript / JavaScript` (L7–389)
  - `### Class-Based Architecture` (L9–77)
  - `### React Components` (L79–101)
  - `### Custom Hooks` (L103–123)
  - `### Environment-Aware Hooks` (L125–165)
  - `### Build-Time Constants` (L167–198)
  - `### Naming Conventions` (L200–211) — TypeScript table rows only
  - `### File Organization` (L213–270)
  - `### Barrel Files` (L272–301)
  - `### Import Aliases` (L303–341)
  - `### Global Object Access` (L343–389)

**Document template:**

```markdown
# Coding Standards — TypeScript & React

## Related Documents

- [Coding Standards Overview](./coding-standards.md) — Quick reference and navigation hub
- [CSS Standards](./coding-standards-css.md) — TailwindCSS and theming
- [Testing Standards](./coding-standards-testing.md) — Testing, logging, error handling

---

{all content from ## TypeScript / JavaScript, preserving structure exactly}
```

**Estimated size:** ~400 lines

---

### 2.3 New File: `coding-standards-css.md` (~150 lines)

**Source sections (from `coding-standards.md`):**
- `## CSS / Styling (TailwindCSS)` (L391–527)
  - `### Utility-First Styling` (L393–416)
  - `### Theme Tokens` (L418–443)
  - `### Custom CSS (Exceptions)` (L445–461)
  - `### Class Organization` (L463–479)
  - `### WebView Background Color` (L481–511)
  - `### File Structure` (L513–527)

**Document template:**

```markdown
# Coding Standards — CSS & Styling

## Related Documents

- [Coding Standards Overview](./coding-standards.md) — Quick reference and navigation hub
- [TypeScript Standards](./coding-standards-typescript.md) — TypeScript and React conventions
- [High-Level Design](./high-level-design.md) — Architecture overview

---

{all content from ## CSS / Styling (TailwindCSS), preserving structure exactly}
```

**Estimated size:** ~150 lines

---

### 2.4 New File: `coding-standards-rust.md` (~550 lines)

**Source sections (from `coding-standards.md`):**
- `## Rust` (L529–987)
  - `### Module Organization` (L531–541)
  - `### Declarative Plugin DSL` (L543–616) — coding conventions only; DSL architecture details live in `declarative-plugin-dsl.md`
  - `### xtask Commands` (L617–657)
  - `### Naming Conventions` (L649–658) — Rust table rows only
  - `### Platform-Specific Code` (L660–723)
  - `### Real-Time Safety` (L725–732)
  - `### Lock-Free Parameter Bridge Pattern` (L734–792)
  - `### SPSC Ring Buffer for Inter-Thread Communication` (L794–831)
  - `### nih-plug Buffer Write Pattern` (L833–899)
  - `### FFI Safety Patterns` (L901–987)
- From `## General`:
  - `### Validation Against Language Specifications` (L1381–1433)
  - `### Rust unwrap() and expect() Usage` (L1435–end)

**Document template:**

```markdown
# Coding Standards — Rust

## Related Documents

- [Coding Standards Overview](./coding-standards.md) — Quick reference and navigation hub
- [Declarative Plugin DSL](./declarative-plugin-dsl.md) — DSL architecture and macro system
- [SDK Architecture](./sdk-architecture.md) — Crate structure and distribution
- [Testing Standards](./coding-standards-testing.md) — Testing, logging, error handling

---

{content from ## Rust}

---

## Validation

{### Validation Against Language Specifications from ## General}

---

## Error Prevention

{### Rust unwrap() and expect() Usage from ## General}
```

**Estimated size:** ~550 lines

---

### 2.5 New File: `coding-standards-testing.md` (~350 lines)

**Source sections (from `coding-standards.md`):**
- `## Testing` (L991–1155)
  - `### Documentation Examples (Rust doctests)` (L995–1017)
  - `### Pre-Push Validation` (L1019–1042)
  - `### Running Tests` (L1044–1061)
  - `### Test File Organization` (L1063–1085)
  - `### Mocking IPC for Tests` (L1087–1110)
  - `### Test Configuration` (L1112–1121)
  - `### Testing CLI-Generated Plugins` (L1123–1155)
- `## Linting & Formatting` (L1157–1223)
  - `### Running Linters` (L1161–1175)
  - `### UI Linting (TypeScript/React)` (L1177–1199)
  - `### Engine Linting (Rust)` (L1201–1216)
  - `### CI Integration` (L1218–1223)
- From `## General`:
  - `### Logging` (L1299–1373)
  - `### Error Handling` (L1375–1379)

**Document template:**

```markdown
# Coding Standards — Testing, Linting & Quality

## Related Documents

- [Coding Standards Overview](./coding-standards.md) — Quick reference and navigation hub
- [Development Workflows](./development-workflows.md) — Build system and CI/CD
- [Agent Development Flow](./agent-development-flow.md) — Agent testing workflow

---

{content from ## Testing}

---

{content from ## Linting & Formatting}

---

## Logging

{### Logging from ## General}

---

## Error Handling

{### Error Handling from ## General}
```

**Estimated size:** ~350 lines

---

### 2.6 Trimmed Hub: `coding-standards.md` (~200 lines)

**What stays:**
1. Title and introduction paragraph (L1–5)
2. New navigation table (see below)
3. Cross-language naming conventions table (unified from TS L200–211 + Rust L649–658)
4. Trimmed `## General` section with:
   - `### Versioning` — brief rule + link to `versioning-and-distribution.md`
   - `### Comments and Documentation` (L1266–1270, 5 lines)
   - `### Documentation References` (L1272–1297, 26 lines)

**New hub structure:**

```markdown
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

{unified naming conventions table from both TS and Rust sections}

---

## General

### Versioning

All version bumping is handled automatically by the CD pipeline.
See [Versioning and Distribution](./versioning-and-distribution.md) for the full version flow.

### Comments and Documentation

{existing 5-line section}

### Documentation References

{existing 26-line section — updated with new document links}

---

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview
- [Agent Development Flow](./agent-development-flow.md) — Agent roles and handoffs
- [Roadmap](../roadmap.md) — Milestone tracking
```

**Estimated size:** ~200 lines

---

## 3. High-Level Design Split Plan

### 3.1 Source: `high-level-design.md` (1,580 lines)

#### Section Inventory

| # | Section | Lines | Destination |
|---|---------|-------|-------------|
| 1 | Title + Related Documents (L1–13) | 13 | Hub |
| 2 | `## Assumptions` (L15–27) | 13 | Hub |
| 3 | `## Executive summary` (L29–31) | 3 | Hub |
| 4 | `## Repository Structure (Monorepo)` (L33–155) | 123 | Hub |
| 5 | `## Architecture overview (block diagram)` (L157–186) | 30 | Hub |
| 6 | `## Main components (concrete)` (L188–231) | 44 | Hub |
| 7 | `## Versioning` (L235–287) | 53 | `versioning-and-distribution.md` |
| 8 | `## Wavecraft SDK Architecture` (L289–579) | 291 | `sdk-architecture.md` |
| 9 | `## Declarative Plugin DSL` (L581–815) | 235 | `declarative-plugin-dsl.md` |
| 10 | `## Browser Development Mode` (L817–1029) | 213 | `development-workflows.md` |
| 11 | `## Build System & Tooling` (L1031–1205) | 175 | `development-workflows.md` |
| 12 | `## Visual Testing` (L1207–1259) | 53 | `development-workflows.md` |
| 13 | `## Audio Unit (AU) Architecture` (L1262–1419) | 158 | `plugin-formats.md` |
| 14 | `## Data flows and timing constraints` (L1421–1426) | 6 | Hub |
| 15 | `## Implementation recommendations` (L1428–1442) | 15 | Hub |
| 16 | `## Trade-offs and alternatives` (L1444–1456) | 13 | Hub |
| 17 | `## Real-time safety checklist` (L1458–1466) | 9 | Hub |
| 18 | `## Testing matrix` (L1469–1492) | 24 | `plugin-formats.md` |
| 19 | `## Packaging & distribution notes` (L1494–1508) | 15 | `versioning-and-distribution.md` |
| 20 | `## Risks & mitigations` (L1510–1533) | 24 | Hub |
| 21 | `## Recommended libraries & tools` (L1535–1544) | 10 | Hub |
| 22 | `## Minimal interface contract` (L1546–1566) | 21 | Hub |
| 23 | `## Roadmap` (L1568–1572) | 5 | Hub |
| 24 | `## Appendix — Key references` (L1574–end) | 7 | Hub |

---

### 3.2 New File: `sdk-architecture.md` (~500 lines)

**Source sections (from `high-level-design.md`):**
- `## Wavecraft SDK Architecture` (L289–579)
  - `### SDK Distribution Model` (L293–345) — includes distribution diagram
  - `### SDK Crate Structure (Rust)` (L347–362) — crate table
  - `### npm Package Structure (UI)` (L364–408) — package table, subpath exports
  - `### Public API Surface (Rust)` (L410–494) — prelude, traits, macros
  - `### User Project Structure` (L496–559) — template Cargo.toml, package.json, usage example
  - `### SDK Design Principles` (L561–571)
  - `### Testability & Environment` (L573–579)

**Document template:**

```markdown
# SDK Architecture

Wavecraft is designed as a Developer SDK that enables other developers to build
VST3/CLAP audio plugins with Rust + React.

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview and system diagram
- [Declarative Plugin DSL](./declarative-plugin-dsl.md) — Macro system and parameter discovery
- [Coding Standards — Rust](./coding-standards-rust.md) — Rust coding conventions
- [Development Workflows](./development-workflows.md) — Build system and dev server
- [SDK Getting Started](../guides/sdk-getting-started.md) — User-facing setup guide

---

{all content from ## Wavecraft SDK Architecture, with ## promoted to ##}
```

**Estimated size:** ~500 lines

---

### 3.3 New File: `declarative-plugin-dsl.md` (~300 lines)

**Source sections (from `high-level-design.md`):**
- `## Declarative Plugin DSL` (L581–815)
  - `### DSL Architecture` (L585–607) — includes diagram
  - `### Macro System` (L609–651) — wavecraft_processor!, wavecraft_plugin!, derive
  - `### Parameter Runtime Discovery` (L653–686) — includes diagram
  - `### UI Parameter Grouping` (L688–700)
  - `### Design Decisions` (L702–712)
  - `### Achieved Code Reduction` (L714–724) — comparison table
  - `### Known Limitations and Trade-offs (v0.9.0)` (L726–815) — parameter sync limitation

**Document template:**

```markdown
# Declarative Plugin DSL

Wavecraft provides a declarative domain-specific language (DSL) for defining
plugins with minimal boilerplate.

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview
- [SDK Architecture](./sdk-architecture.md) — Crate structure and public API
- [Coding Standards — Rust](./coding-standards-rust.md) — Rust DSL usage conventions

---

{all content from ## Declarative Plugin DSL, with ## promoted to ##}
```

**Estimated size:** ~300 lines

---

### 3.4 New File: `development-workflows.md` (~350 lines)

**Source sections (from `high-level-design.md`):**
- `## Browser Development Mode` (L817–1029)
  - `### Development Mode Architecture` (L821–849) — includes transport diagram
  - `### Transport Factory Pattern` (L851–871) — includes diagram
  - `### How It Works` (L873–906)
  - `### Development Workflow` (L908–927)
  - `### Why Module-Level Detection?` (L929–931)
  - `### Dev Audio via FFI` (L933–1021) — includes FFI architecture, vtable, key components, memory safety, backward compat, parameter sync
  - `### Benefits` (L1019–1029)
- `## Build System & Tooling` (L1031–1205)
  - `### Available Commands` (L1036–1054) — command table
  - `### Development Workflow` (L1056–1081)
  - `### Visual Testing Workflow` (L1083–1097)
  - `### Release Workflow` (L1098–1108)
  - `### Code Signing (macOS)` (L1110–1129)
  - `### Notarization (macOS)` (L1131–1146)
  - `### Entitlements` (L1148–1162)
  - `### CI/CD Pipelines` (L1164–1205) — CI, CD, Release workflows + secrets table
- `## Visual Testing` (L1207–1259)
  - `### Architecture` (L1211–1228) — includes diagram
  - `### Test ID Convention` (L1230–1240) — test ID table
  - `### Baseline Storage` (L1242–1247)
  - `### Key Design Decisions` (L1249–1259)

**Document template:**

```markdown
# Development Workflows

This document covers the development, build, and testing workflows for Wavecraft.

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview
- [Testing Standards](./coding-standards-testing.md) — Test conventions and CI
- [Agent Development Flow](./agent-development-flow.md) — Agent testing procedures
- [Visual Testing Guide](../guides/visual-testing.md) — Detailed visual testing instructions
- [CI Pipeline Guide](../guides/ci-pipeline.md) — CI/CD pipeline details
- [macOS Signing Guide](../guides/macos-signing.md) — Code signing setup

---

{content from ## Browser Development Mode}

---

{content from ## Build System & Tooling}

---

{content from ## Visual Testing}
```

**Estimated size:** ~350 lines

---

### 3.5 New File: `plugin-formats.md` (~300 lines)

**Source sections (from `high-level-design.md`):**
- `## Audio Unit (AU) Architecture` (L1262–1419)
  - `### Overview` (L1264–1268)
  - `### AU Support via clap-wrapper` (L1270–1277)
  - `### AU-Specific Requirements` (L1279–1295) — 4CC codes, bundle structure, registration
  - `### clap-wrapper Integration` (L1297–1368) — CMakeLists, build commands, notes
  - `### Why Not Native AU in nih-plug?` (L1370–1380)
  - `### AU vs VST3 vs CLAP Behavioral Differences` (L1382–1394) — comparison table
  - `### AU-Specific Constraints` (L1396–1410) — threading, RT priority, view lifecycle
  - `### Logic Pro Specific Notes` (L1412–1419)
- `## Testing matrix` (L1469–1492) — primary, secondary, deprioritized targets

**Document template:**

```markdown
# Plugin Formats — VST3, CLAP & AU

This document covers plugin format architectures, behavioral differences, and
host compatibility requirements.

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview
- [Development Workflows](./development-workflows.md) — Build and release workflows
- [Versioning and Distribution](./versioning-and-distribution.md) — Packaging and signing

---

## Plugin Format Overview

Wavecraft supports three plugin formats via nih-plug's export system:

| Format | Support | Mechanism | Primary Host |
|--------|---------|-----------|-------------|
| VST3 | ✅ Native | nih-plug direct export | Ableton Live |
| CLAP | ✅ Native | nih-plug direct export | Bitwig, Reaper |
| AU (AUv2) | ⚠️ Wrapped | clap-wrapper (CLAP → AU) | Logic Pro |

---

{content from ## Audio Unit (AU) Architecture}

---

{content from ## Testing matrix}
```

**Estimated size:** ~300 lines

---

### 3.6 New File: `versioning-and-distribution.md` (~200 lines)

**Source sections (from `high-level-design.md`):**
- `## Versioning` (L235–287)
  - `### Version Flow` (L239–271) — includes flow diagram
  - `### Key Design Decisions` (L273–287)
- `## Packaging & distribution notes` (L1494–1508)

**Also references content from `coding-standards.md`:**
- `### Versioning` (L1228–1264) — the hub will contain a brief summary and link here

**Document template:**

```markdown
# Versioning and Distribution

This document covers Wavecraft's version management, build-time injection, and
platform-specific packaging.

## Related Documents

- [High-Level Design](./high-level-design.md) — Architecture overview
- [Coding Standards Overview](./coding-standards.md) — Versioning rules for developers
- [Plugin Formats](./plugin-formats.md) — Format-specific packaging details
- [Development Workflows](./development-workflows.md) — Build and release commands
- [CI Pipeline Guide](../guides/ci-pipeline.md) — CD pipeline details

---

{content from ## Versioning — version flow diagram and design decisions}

---

{content from ## Packaging & distribution notes}
```

**Estimated size:** ~200 lines

---

### 3.7 Trimmed Hub: `high-level-design.md` (~400 lines)

**What stays:**

1. Title (L1)
2. `## Related Documents` (L5–13) — **expanded** to include all new documents
3. `## Assumptions` (L15–27, 13 lines)
4. `## Executive summary` (L29–31, 3 lines)
5. `## Repository Structure (Monorepo)` (L33–155, 123 lines) — includes tree, benefits, component diagram, phase table
6. `## Architecture overview (block diagram)` (L157–186, 30 lines)
7. `## Main components (concrete)` (L188–231, 44 lines)
8. **New: `## Documentation Structure`** — navigation table mapping topics to documents
9. `## Data flows and timing constraints` (L1421–1426, 6 lines)
10. `## Implementation recommendations` (L1428–1442, 15 lines)
11. `## Trade-offs and alternatives` (L1444–1456, 13 lines)
12. `## Real-time safety checklist` (L1458–1466, 9 lines)
13. `## Risks & mitigations` (L1510–1533, 24 lines)
14. `## Recommended libraries & tools` (L1535–1544, 10 lines)
15. `## Minimal interface contract` (L1546–1566, 21 lines)
16. `## Roadmap` (L1568–1572, 5 lines)
17. `## Appendix — Key references` (L1574–end, 7 lines)

**New navigation section to insert after `## Main components`:**

```markdown
---

## Documentation Structure

For detailed coverage of specific topics, see:

| Document | Scope |
|----------|-------|
| [SDK Architecture](./sdk-architecture.md) | Crate structure, npm packages, public API, distribution model |
| [Declarative Plugin DSL](./declarative-plugin-dsl.md) | Macro system, parameter discovery, DSL limitations |
| [Development Workflows](./development-workflows.md) | Browser dev mode, FFI audio, build system, CI/CD, visual testing |
| [Plugin Formats](./plugin-formats.md) | VST3, CLAP, AU architecture, host compatibility, testing matrix |
| [Versioning and Distribution](./versioning-and-distribution.md) | Version flow, build-time injection, packaging, signing |

For coding conventions, see:

| Document | Scope |
|----------|-------|
| [Coding Standards](./coding-standards.md) | Overview hub, naming conventions, general rules |
| [TypeScript & React](./coding-standards-typescript.md) | Classes, hooks, React components, imports |
| [CSS & Styling](./coding-standards-css.md) | TailwindCSS, theme tokens, WebView styling |
| [Rust](./coding-standards-rust.md) | Module org, real-time safety, FFI, xtask |
| [Testing & Quality](./coding-standards-testing.md) | Testing, linting, logging, error handling |
```

**Estimated size:** ~400 lines

---

## 4. Cross-Reference Update Plan

### 4.1 Files Requiring Updates

| File | References | Update Required |
|------|-----------|-----------------|
| `.github/copilot-instructions.md` | L9: `#file:coding-standards.md`, L10: `#file:high-level-design.md` | **Critical** — restructure for topic-specific references |
| `.github/agents/QA.agent.md` | L47, L48, L88, L197 | Update to reference split docs |
| `.github/agents/PO.agent.md` | L175: `coding-standards.md#versioning` | Update to `versioning-and-distribution.md` |
| `README.md` | L111–112 | Update descriptions |
| `CONTRIBUTING.md` | L38, L94 | Update `coding-standards.md` and `#testing` anchor |
| `docs/guides/ci-pipeline.md` | L291 | Update description |
| `docs/guides/sdk-getting-started.md` | L599–600 | Update descriptions |
| `docs/architecture/coding-standards.md` | Internal `./high-level-design.md` links | Updated as part of hub rewrite |
| `docs/architecture/high-level-design.md` | Internal `./coding-standards.md` links | Updated as part of hub rewrite |

**Files NOT requiring updates** (out of scope or no change needed):
- `docs/feature-specs/_archive/**` — archived, must not edit per project rules
- `docs/roadmap.md` — only PO can edit
- `docs/backlog.md` L32 — references `audio-input-via-wasm/high-level-design.md`, unrelated
- `docs/feature-specs/remove-manual-versioning/PR-summary.md` — references current file, but this is an active feature spec (OK to update if still open)
- `cli/sdk-templates/new-project/react/README.md` — references from template; check if template links to architecture docs
- `.github/skills/*` — no references found, no update needed

### 4.2 Copilot Instructions Update (CRITICAL)

**Current** (`.github/copilot-instructions.md` L9–11):
```
- Before making changes in the code, check the #file:../docs/architecture/coding-standards.md file for relevant coding standards and conventions.
- For understanding the overall project architecture, SDK structure, and design decisions, refer to #file:../docs/architecture/high-level-design.md document.
```

**Problem:** This loads entire 1,500+ line files into every agent's context.

**Proposed replacement:**

```markdown
- Before making changes in the code, check the relevant coding standards:
  - For TypeScript/React code: #file:../docs/architecture/coding-standards-typescript.md
  - For CSS/styling: #file:../docs/architecture/coding-standards-css.md
  - For Rust code: #file:../docs/architecture/coding-standards-rust.md
  - For testing/linting: #file:../docs/architecture/coding-standards-testing.md
  - For general conventions (naming, docs): #file:../docs/architecture/coding-standards.md
- For understanding the overall project architecture, refer to #file:../docs/architecture/high-level-design.md (overview hub with links to detailed docs).
```

**Rationale:** Agents can now be directed to load only the relevant standards file (~200–550 lines) instead of the full monolith (~1,500 lines). The `#file:` directive means Copilot attaches the file content — smaller files = fewer tokens.

**Trade-off considered:** We could remove the `#file:` directives entirely and trust agents to search, but explicit references ensure agents always have context. The split means the cost of always-loading is now ~200 lines (the hub) instead of ~1,500.

### 4.3 Agent File Updates

**`.github/agents/QA.agent.md`:**
- L47: `- Coding standards: docs/architecture/coding-standards.md` → keep as-is (hub is the correct entry point for QA)
- L48: `- Architecture: docs/architecture/high-level-design.md` → keep as-is (hub is correct)
- L88: `Verify boundaries per high-level-design.md` → keep as-is (reference is to the overview)
- L197: `Deviations from high-level-design.md` → keep as-is

**`.github/agents/PO.agent.md`:**
- L175: `See the [Coding Standards — Versioning](../../docs/architecture/coding-standards.md#versioning)` → update to `See [Versioning and Distribution](../../docs/architecture/versioning-and-distribution.md)`

### 4.4 Link Update Patterns

Search-and-replace patterns for bulk updates:

| Pattern | Search | Replace | Scope |
|---------|--------|---------|-------|
| CS testing anchor | `coding-standards.md#testing` | `coding-standards-testing.md` | `CONTRIBUTING.md` |
| CS versioning anchor | `coding-standards.md#versioning` | `versioning-and-distribution.md` | `PO.agent.md` |
| CS full link | `[Coding Standards](docs/architecture/coding-standards.md)` | `[Coding Standards](docs/architecture/coding-standards.md)` | No change (hub still valid) |
| HLD full link | `[High-Level Design](docs/architecture/high-level-design.md)` | `[High-Level Design](docs/architecture/high-level-design.md)` | No change (hub still valid) |

**Key insight:** Most external references point to the hub files (`coding-standards.md`, `high-level-design.md`) without anchors. Since the hubs remain at the same paths with navigation tables, most links don't need updating. Only anchor-specific links (e.g., `#testing`, `#versioning`) need to point to the new dedicated files.

### 4.5 Template File Check

The template `cli/sdk-templates/new-project/react/README.md` may reference architecture docs. Verify its content and update if it links to `coding-standards.md` or `high-level-design.md` with topic-specific anchors.

---

## 5. Agent Instructions Architecture

### 5.1 Design Philosophy

The `.github/copilot-instructions.md` file is the most important file for AI agent token efficiency because its content is **always loaded** as context for every agent interaction. The `#file:` directives cause referenced files to be attached in full.

**Current token budget impact:**
- `coding-standards.md` (1,502 lines) ≈ 5,000+ tokens — loaded for every agent
- `high-level-design.md` (1,580 lines) ≈ 5,500+ tokens — loaded for every agent

**Post-split token budget impact:**
- `coding-standards.md` hub (~200 lines) ≈ 700 tokens — loaded for every agent
- `high-level-design.md` hub (~400 lines) ≈ 1,400 tokens — loaded for every agent
- Topic-specific docs loaded on-demand by agents as needed

**Net reduction per agent call:** ~8,500 tokens → ~2,100 tokens (75% reduction in always-loaded context).

### 5.2 Alternative Considered: Remove `#file:` Directives

We could remove all `#file:` directives from copilot-instructions and let agents discover docs via search. This was rejected because:
- Agents need guaranteed access to coding standards when editing code
- Search-based discovery is non-deterministic
- Hub files are small enough post-split to justify always-loading

### 5.3 Alternative Considered: Per-Language `#file:` Directives

```
- For TypeScript code: #file:../docs/architecture/coding-standards-typescript.md
- For Rust code: #file:../docs/architecture/coding-standards-rust.md
```

This was considered but would conditionally load ~400–550 lines based on file type. Since `copilot-instructions.md` doesn't support conditional `#file:` loading (all directives are always resolved), this would increase total token load to ~1,100–1,250 lines — still less than the current 1,502 but more than just loading the ~200-line hub.

**Recommendation:** Load only the hub files via `#file:` and let agents follow links to specific docs as needed. This achieves the best token efficiency while maintaining guaranteed baseline context.

**Final proposed `copilot-instructions.md` approach:**

```markdown
- Before making changes in the code, check the #file:../docs/architecture/coding-standards.md file for relevant coding standards and navigation to language-specific guides.
- For understanding the overall project architecture, refer to #file:../docs/architecture/high-level-design.md (overview with links to detailed topic docs).
```

This keeps the same two `#file:` directives but now loads ~200 + ~400 = ~600 lines instead of ~1,500 + ~1,580 = ~3,080 lines. Topic-specific docs are linked from the hubs and agents can read them on-demand.

---

## 6. Migration Rules

### 6.1 Document Size Guidelines

| Metric | Target | Action |
|--------|--------|--------|
| Minimum document size | 150 lines | Do not split below this — merge with related topic |
| Maximum document size | 600 lines | Split when a document exceeds this |
| Ideal document size | 200–400 lines | Sweet spot for readability and token efficiency |

### 6.2 When to Create a New Document

Create a new architecture document when:
1. A new topic has **150+ lines** of content that is self-contained
2. An existing document exceeds **600 lines** and contains clearly separable topics
3. A new domain is introduced that doesn't fit any existing document (e.g., new plugin format, new language support)

Do NOT create a new document when:
1. The topic is < 150 lines — add it to the most relevant existing document
2. The content is specific to a single feature — use `docs/feature-specs/{feature}/`
3. The content is a guide or tutorial — use `docs/guides/`

### 6.3 Required Document Structure

Every architecture document MUST contain:

```markdown
# {Title}

{1-2 sentence description}

## Related Documents

- [Document Name](./relative-path.md) — Brief description
- ...

---

{content}
```

### 6.4 Hub Update Requirement

When creating a new architecture document:
1. Add it to the navigation table in `high-level-design.md` or `coding-standards.md`
2. Add it to the "Related Documents" section of at least one existing document
3. Ensure `scripts/check-links.sh` passes after the addition

### 6.5 CONTRIBUTING.md Addition

Add a new section to `CONTRIBUTING.md` documenting the documentation structure:

```markdown
## Documentation Structure

Architecture documentation lives in `docs/architecture/` and follows these conventions:

- **Overview hubs** (`coding-standards.md`, `high-level-design.md`) — Navigation entry points
- **Topic-specific docs** — Self-contained documents for specific domains
- **Target size**: 150–600 lines per document
- **Required sections**: Title, description, Related Documents, content

See the navigation tables in [Coding Standards](docs/architecture/coding-standards.md)
and [High-Level Design](docs/architecture/high-level-design.md) for the full document list.
```

---

## 7. Validation Strategy

### 7.1 Link Checking

**Automated:**
```bash
scripts/check-links.sh
```
This script already exists and checks all `docs/` markdown links, excluding `_archive/`. It must pass with zero broken links after the split.

**Manual grep patterns for stale references:**
```bash
# Find references to old section anchors that no longer exist in hub files
rg '#testing|#linting|#rust|#typescript|#css' docs/ .github/ --type md | grep -v _archive

# Find references to old anchors in coding-standards.md
rg 'coding-standards\.md#(typescript|css|rust|testing|linting|logging|error-handling|ffi|real-time|spsc|nih-plug|unwrap)' docs/ .github/ --type md | grep -v _archive

# Find references to old anchors in high-level-design.md  
rg 'high-level-design\.md#(sdk|dsl|browser-dev|build-system|visual-testing|audio-unit|versioning|packaging|ci-cd)' docs/ .github/ --type md | grep -v _archive
```

### 7.2 Content Completeness

Verify no content was lost during the split:

```bash
# Line count check — total should be >= original
wc -l docs/architecture/*.md

# Verify all original H2/H3 headings exist in some document
grep '^##' docs/architecture/coding-standards-*.md docs/architecture/coding-standards.md | wc -l
# Should be >= original heading count from coding-standards.md

grep '^##' docs/architecture/sdk-architecture.md docs/architecture/declarative-plugin-dsl.md docs/architecture/development-workflows.md docs/architecture/plugin-formats.md docs/architecture/versioning-and-distribution.md docs/architecture/high-level-design.md | wc -l
# Should be >= original heading count from high-level-design.md
```

### 7.3 Agent Testing

After the split is implemented, verify token efficiency:

1. Open a new Copilot chat with coder mode
2. Ask a Rust-specific question — verify the agent loads `coding-standards-rust.md` (not the full 1,500-line file)
3. Ask an architecture question — verify the agent loads the hub, then navigates to a specific doc
4. Check that agents can still find all information they previously had access to

### 7.4 Navigation Testing

For each new document:
1. Open the hub file (`coding-standards.md` or `high-level-design.md`)
2. Click every link in the navigation table — verify it resolves
3. Open the target document — verify the "Related Documents" links resolve back
4. Verify "Related Documents" in each split doc links to at least the parent hub

---

## 8. Implementation Order

The split should be executed in this order to minimize broken-link windows:

| Step | Action | Risk |
|------|--------|------|
| 1 | Create all new topic-specific documents (9 files) | No risk — new files, nothing breaks |
| 2 | Rewrite `coding-standards.md` as hub | Links to hub still work; anchors may break |
| 3 | Rewrite `high-level-design.md` as hub | Links to hub still work; anchors may break |
| 4 | Update cross-references in `.github/copilot-instructions.md` | Critical for agent token efficiency |
| 5 | Update cross-references in other files | Fixes any broken anchors |
| 6 | Update `CONTRIBUTING.md` with documentation guidelines | No risk |
| 7 | Run validation (`check-links.sh` + grep patterns) | Catches any missed references |

Steps 1–3 should be done in a single commit to maintain atomicity. Steps 4–6 can be separate commits.

---

## 9. Summary Table

### New Files Created

| File | Source | Lines | Topic |
|------|--------|-------|-------|
| `coding-standards-typescript.md` | CS L7–389 | ~400 | TypeScript, React, hooks, build constants |
| `coding-standards-css.md` | CS L391–527 | ~150 | TailwindCSS, theming, WebView styling |
| `coding-standards-rust.md` | CS L529–987 + L1381–end | ~550 | Rust modules, DSL, FFI, RT-safety, validation, unwrap |
| `coding-standards-testing.md` | CS L991–1223 + L1299–1379 | ~350 | Testing, linting, logging, error handling |
| `sdk-architecture.md` | HLD L289–579 | ~500 | SDK distribution, crates, npm, public API |
| `declarative-plugin-dsl.md` | HLD L581–815 | ~300 | DSL macros, parameter discovery, limitations |
| `development-workflows.md` | HLD L817–1259 | ~350 | Browser dev, FFI audio, build system, visual testing |
| `plugin-formats.md` | HLD L1262–1419 + L1469–1492 | ~300 | VST3, CLAP, AU, testing matrix |
| `versioning-and-distribution.md` | HLD L235–287 + L1494–1508 | ~200 | Version flow, packaging, signing |

### Modified Files

| File | Change |
|------|--------|
| `coding-standards.md` | Rewritten as ~200-line hub |
| `high-level-design.md` | Rewritten as ~400-line hub |
| `.github/copilot-instructions.md` | Updated `#file:` approach (keep hub refs) |
| `.github/agents/PO.agent.md` | Update versioning link |
| `CONTRIBUTING.md` | Update testing link + add docs structure section |
| `README.md` | Update descriptions (optional, links still valid) |
| `docs/guides/ci-pipeline.md` | Minor description update |
| `docs/guides/sdk-getting-started.md` | Minor description update |

### Unchanged Files

| File | Reason |
|------|--------|
| `agent-development-flow.md` | 254 lines, within target, standalone topic |
| `docs/feature-specs/_archive/**` | Must not edit archived specs |
| `docs/roadmap.md` | Only PO can edit |
| `.github/skills/**` | No references to split files |
