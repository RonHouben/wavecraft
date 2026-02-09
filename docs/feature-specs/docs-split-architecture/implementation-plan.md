# Implementation Plan: Architecture Documentation Split

## Overview

Split the two monolithic architecture documents (`coding-standards.md` at 1,501 lines and `high-level-design.md` at 1,579 lines) into 9 focused topic-specific documents (150–600 lines each), rewrite the originals as navigation hubs, and update all cross-references. This is a documentation-only change with no code modifications.

## Source Documents

- **Low-level design:** `docs/feature-specs/docs-split-architecture/low-level-design-docs-split.md`
- **User stories:** `docs/feature-specs/docs-split-architecture/user-stories.md`

## Requirements

- 9 new topic-specific documents created under `docs/architecture/`
- 2 existing documents rewritten as navigation hubs
- All cross-references updated (no broken links)
- `scripts/check-links.sh` passes with zero broken links
- No content lost from original documents
- No changes to archived feature specs, roadmap, or code files

---

## Phase 1: Create New Documents (zero-breakage — new files only)

All steps in this phase create new files. Nothing existing is modified, so nothing can break.

---

### Step 1.1: Create `coding-standards-typescript.md`

**File:** `docs/architecture/coding-standards-typescript.md`

**Action:**
1. Create the file with the following structure:
   - Title: `# Coding Standards — TypeScript & React`
   - "Related Documents" section linking to:
     - `[Coding Standards Overview](./coding-standards.md)` — Quick reference and navigation hub
     - `[CSS Standards](./coding-standards-css.md)` — TailwindCSS and theming
     - `[Testing Standards](./coding-standards-testing.md)` — Testing, logging, error handling
   - Horizontal rule (`---`)
   - Copy content from `coding-standards.md` L7–389 (the `## TypeScript / JavaScript` section, inclusive of all subsections through `### Global Object Access`)

**Content to extract from `coding-standards.md`:**
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

**Important:** Preserve the `## TypeScript / JavaScript` heading from the source. Keep all subheadings, code examples, do/don't blocks, and tables exactly as they appear.

**Verification:**
```bash
test -f docs/architecture/coding-standards-typescript.md && echo "✅ File exists"
head -10 docs/architecture/coding-standards-typescript.md  # Should show title + related docs
wc -l docs/architecture/coding-standards-typescript.md     # Should be ~400 lines
grep -c '^###' docs/architecture/coding-standards-typescript.md  # Should be 10
```

---

### Step 1.2: Create `coding-standards-css.md`

**File:** `docs/architecture/coding-standards-css.md`

**Action:**
1. Create the file with the following structure:
   - Title: `# Coding Standards — CSS & Styling`
   - "Related Documents" section linking to:
     - `[Coding Standards Overview](./coding-standards.md)` — Quick reference and navigation hub
     - `[TypeScript Standards](./coding-standards-typescript.md)` — TypeScript and React conventions
     - `[High-Level Design](./high-level-design.md)` — Architecture overview
   - Horizontal rule (`---`)
   - Copy content from `coding-standards.md` L391–527 (the `## CSS / Styling (TailwindCSS)` section)

**Content to extract from `coding-standards.md`:**
- `### Utility-First Styling` (L393–416)
- `### Theme Tokens` (L418–443)
- `### Custom CSS (Exceptions)` (L445–461)
- `### Class Organization` (L463–479)
- `### WebView Background Color` (L481–511)
- `### File Structure` (L513–527)

**Important:** Preserve the `## CSS / Styling (TailwindCSS)` heading from the source.

**Verification:**
```bash
test -f docs/architecture/coding-standards-css.md && echo "✅ File exists"
wc -l docs/architecture/coding-standards-css.md  # Should be ~150 lines
grep -c '^###' docs/architecture/coding-standards-css.md  # Should be 6
```

---

### Step 1.3: Create `coding-standards-rust.md`

**File:** `docs/architecture/coding-standards-rust.md`

**Action:**
1. Create the file with the following structure:
   - Title: `# Coding Standards — Rust`
   - "Related Documents" section linking to:
     - `[Coding Standards Overview](./coding-standards.md)` — Quick reference and navigation hub
     - `[Declarative Plugin DSL](./declarative-plugin-dsl.md)` — DSL architecture and macro system
     - `[SDK Architecture](./sdk-architecture.md)` — Crate structure and distribution
     - `[Testing Standards](./coding-standards-testing.md)` — Testing, logging, error handling
   - Horizontal rule (`---`)
   - Copy content from `coding-standards.md` L529–987 (the `## Rust` section)
   - Add a horizontal rule, then a `## Validation` heading
   - Copy content from `coding-standards.md` L1381–1433 (`### Validation Against Language Specifications`)
   - Add a horizontal rule, then a `## Error Prevention` heading
   - Copy content from `coding-standards.md` L1435–1501 (`### Rust unwrap() and expect() Usage`)

**Content to extract (main Rust section):**
- `### Module Organization` (L531–541)
- `### Declarative Plugin DSL` (L543–616)
- `### xtask Commands` (L617–657)
- `### Naming Conventions` (L649–658) — Rust rows
- `### Platform-Specific Code` (L660–723)
- `### Real-Time Safety` (L725–732)
- `### Lock-Free Parameter Bridge Pattern` (L734–792)
- `### SPSC Ring Buffer for Inter-Thread Communication` (L794–831)
- `### nih-plug Buffer Write Pattern` (L833–899)
- `### FFI Safety Patterns` (L901–987)

**Content to extract (from General section):**
- `### Validation Against Language Specifications` (L1381–1433) — place under new `## Validation` heading
- `### Rust unwrap() and expect() Usage` (L1435–1501) — place under new `## Error Prevention` heading

**Important:** The `### FFI Safety Patterns` section contains a link `[Dev Audio via FFI](./high-level-design.md#dev-audio-via-ffi)`. Update this to `[Dev Audio via FFI](./development-workflows.md#dev-audio-via-ffi)` since that section will move to `development-workflows.md`.

**Verification:**
```bash
test -f docs/architecture/coding-standards-rust.md && echo "✅ File exists"
wc -l docs/architecture/coding-standards-rust.md  # Should be ~550 lines
grep -c '^###' docs/architecture/coding-standards-rust.md  # Should be 12
grep 'development-workflows.md' docs/architecture/coding-standards-rust.md  # Should find the updated FFI link
```

---

### Step 1.4: Create `coding-standards-testing.md`

**File:** `docs/architecture/coding-standards-testing.md`

**Action:**
1. Create the file with the following structure:
   - Title: `# Coding Standards — Testing, Linting & Quality`
   - "Related Documents" section linking to:
     - `[Coding Standards Overview](./coding-standards.md)` — Quick reference and navigation hub
     - `[Development Workflows](./development-workflows.md)` — Build system and CI/CD
     - `[Agent Development Flow](./agent-development-flow.md)` — Agent testing workflow
   - Horizontal rule (`---`)
   - Copy content from `coding-standards.md` L991–1155 (the `## Testing` section)
   - Horizontal rule
   - Copy content from `coding-standards.md` L1157–1223 (the `## Linting & Formatting` section)
   - Horizontal rule, then `## Logging` heading
   - Copy content from `coding-standards.md` L1299–1373 (`### Logging`)
   - Horizontal rule, then `## Error Handling` heading
   - Copy content from `coding-standards.md` L1375–1379 (`### Error Handling`)

**Content to extract:**
- `## Testing` (L991–1155): doctests, pre-push validation, running tests, file org, mocking, config, CLI-generated plugins
- `## Linting & Formatting` (L1157–1223): running linters, UI linting, engine linting, CI integration
- `### Logging` (L1299–1373): structured logging for UI and Engine
- `### Error Handling` (L1375–1379): error handling rules

**Important:** Promote `### Logging` and `### Error Handling` to `## Logging` and `## Error Handling` in this document (they become top-level sections since they are no longer under `## General`).

**Verification:**
```bash
test -f docs/architecture/coding-standards-testing.md && echo "✅ File exists"
wc -l docs/architecture/coding-standards-testing.md  # Should be ~350 lines
grep '^## ' docs/architecture/coding-standards-testing.md  # Should show: Testing, Linting, Logging, Error Handling + Related Documents
```

---

### Step 1.5: Create `sdk-architecture.md`

**File:** `docs/architecture/sdk-architecture.md`

**Action:**
1. Create the file with the following structure:
   - Title: `# SDK Architecture`
   - Brief intro: "Wavecraft is designed as a Developer SDK that enables other developers to build VST3/CLAP audio plugins with Rust + React."
   - "Related Documents" section linking to:
     - `[High-Level Design](./high-level-design.md)` — Architecture overview and system diagram
     - `[Declarative Plugin DSL](./declarative-plugin-dsl.md)` — Macro system and parameter discovery
     - `[Coding Standards — Rust](./coding-standards-rust.md)` — Rust coding conventions
     - `[Development Workflows](./development-workflows.md)` — Build system and dev server
     - `[SDK Getting Started](../guides/sdk-getting-started.md)` — User-facing setup guide
   - Horizontal rule (`---`)
   - Copy content from `high-level-design.md` L289–579 (the `## Wavecraft SDK Architecture` section)

**Content to extract:**
- `### SDK Distribution Model` (L293–345) — includes distribution diagram
- `### SDK Crate Structure (Rust)` (L347–362) — crate table
- `### npm Package Structure (UI)` (L364–408) — package table, subpath exports
- `### Public API Surface (Rust)` (L410–494) — prelude, traits, macros
- `### User Project Structure` (L496–559) — template Cargo.toml, package.json, usage example
- `### SDK Design Principles` (L561–571)
- `### Testability & Environment` (L573–579)

**Important:** The source uses `### ` (H3) headings because they were nested under a `## `. In the new document, keep them as `### ` since the document title is `#` and the section heading below "Related Documents" will be from the original content. Alternatively, promote the `## Wavecraft SDK Architecture` heading's subsections to stay as `###` — the key point is to preserve the internal structure as-is.

**Verification:**
```bash
test -f docs/architecture/sdk-architecture.md && echo "✅ File exists"
wc -l docs/architecture/sdk-architecture.md  # Should be ~500 lines
grep -c '^###' docs/architecture/sdk-architecture.md  # Should be 7+
```

---

### Step 1.6: Create `declarative-plugin-dsl.md`

**File:** `docs/architecture/declarative-plugin-dsl.md`

**Action:**
1. Create the file with the following structure:
   - Title: `# Declarative Plugin DSL`
   - Brief intro: "Wavecraft provides a declarative domain-specific language (DSL) for defining plugins with minimal boilerplate."
   - "Related Documents" section linking to:
     - `[High-Level Design](./high-level-design.md)` — Architecture overview
     - `[SDK Architecture](./sdk-architecture.md)` — Crate structure and public API
     - `[Coding Standards — Rust](./coding-standards-rust.md)` — Rust DSL usage conventions
   - Horizontal rule (`---`)
   - Copy content from `high-level-design.md` L581–815 (the `## Declarative Plugin DSL` section)

**Content to extract:**
- `### DSL Architecture` (L585–607) — includes diagram
- `### Macro System` (L609–651) — wavecraft_processor!, wavecraft_plugin!, derive
- `### Parameter Runtime Discovery` (L653–686) — includes diagram
- `### UI Parameter Grouping` (L688–700)
- `### Design Decisions` (L702–712)
- `### Achieved Code Reduction` (L714–724) — comparison table
- `### Known Limitations and Trade-offs (v0.9.0)` (L726–815) — parameter sync limitation

**Verification:**
```bash
test -f docs/architecture/declarative-plugin-dsl.md && echo "✅ File exists"
wc -l docs/architecture/declarative-plugin-dsl.md  # Should be ~300 lines
grep -c '^###' docs/architecture/declarative-plugin-dsl.md  # Should be 7+
```

---

### Step 1.7: Create `development-workflows.md`

**File:** `docs/architecture/development-workflows.md`

**Action:**
1. Create the file with the following structure:
   - Title: `# Development Workflows`
   - Brief intro: "This document covers the development, build, and testing workflows for Wavecraft."
   - "Related Documents" section linking to:
     - `[High-Level Design](./high-level-design.md)` — Architecture overview
     - `[Testing Standards](./coding-standards-testing.md)` — Test conventions and CI
     - `[Agent Development Flow](./agent-development-flow.md)` — Agent testing procedures
     - `[Visual Testing Guide](../guides/visual-testing.md)` — Detailed visual testing instructions
     - `[CI Pipeline Guide](../guides/ci-pipeline.md)` — CI/CD pipeline details
     - `[macOS Signing Guide](../guides/macos-signing.md)` — Code signing setup
   - Horizontal rule (`---`)
   - Copy content from `high-level-design.md` L817–1029 (`## Browser Development Mode`)
   - Horizontal rule
   - Copy content from `high-level-design.md` L1031–1205 (`## Build System & Tooling`)
   - Horizontal rule
   - Copy content from `high-level-design.md` L1207–1259 (`## Visual Testing`)

**Content to extract:**
- `## Browser Development Mode` (L817–1029): transport abstraction, factory pattern, how it works, dev workflow, module-level detection, dev audio via FFI (architecture, vtable, key components, memory safety, backward compat, parameter sync), benefits
- `## Build System & Tooling` (L1031–1205): available commands, dev/release/visual workflows, signing, notarization, entitlements, CI/CD pipelines, secrets
- `## Visual Testing` (L1207–1259): architecture, test ID convention, baseline storage, key design decisions

**Important:** Keep these as `## ` headings (they were already `## ` in the source).

**Verification:**
```bash
test -f docs/architecture/development-workflows.md && echo "✅ File exists"
wc -l docs/architecture/development-workflows.md  # Should be ~350-450 lines
grep '^## ' docs/architecture/development-workflows.md  # Should show: Related Documents, Browser Dev Mode, Build System, Visual Testing
```

---

### Step 1.8: Create `plugin-formats.md`

**File:** `docs/architecture/plugin-formats.md`

**Action:**
1. Create the file with the following structure:
   - Title: `# Plugin Formats — VST3, CLAP & AU`
   - Brief intro: "This document covers plugin format architectures, behavioral differences, and host compatibility requirements."
   - "Related Documents" section linking to:
     - `[High-Level Design](./high-level-design.md)` — Architecture overview
     - `[Development Workflows](./development-workflows.md)` — Build and release workflows
     - `[Versioning and Distribution](./versioning-and-distribution.md)` — Packaging and signing
   - Horizontal rule (`---`)
   - Add a `## Plugin Format Overview` section with the summary table from the LLD:
     ```
     | Format | Support | Mechanism | Primary Host |
     |--------|---------|-----------|-------------|
     | VST3 | ✅ Native | nih-plug direct export | Ableton Live |
     | CLAP | ✅ Native | nih-plug direct export | Bitwig, Reaper |
     | AU (AUv2) | ⚠️ Wrapped | clap-wrapper (CLAP → AU) | Logic Pro |
     ```
   - Horizontal rule
   - Copy content from `high-level-design.md` L1262–1419 (`## Audio Unit (AU) Architecture`)
   - Horizontal rule
   - Copy content from `high-level-design.md` L1469–1492 (`## Testing matrix (focused on macOS + Ableton)`)

**Content to extract:**
- `## Audio Unit (AU) Architecture` (L1262–1419): overview, clap-wrapper, requirements, integration, why not native AU, behavioral differences table, constraints, Logic Pro notes
- `## Testing matrix (focused on macOS + Ableton)` (L1469–1492): primary/secondary/deprioritized

**Verification:**
```bash
test -f docs/architecture/plugin-formats.md && echo "✅ File exists"
wc -l docs/architecture/plugin-formats.md  # Should be ~300 lines
grep '^## ' docs/architecture/plugin-formats.md  # Should show: Related Documents, Plugin Format Overview, Audio Unit, Testing matrix
```

---

### Step 1.9: Create `versioning-and-distribution.md`

**File:** `docs/architecture/versioning-and-distribution.md`

**Action:**
1. Create the file with the following structure:
   - Title: `# Versioning and Distribution`
   - Brief intro: "This document covers Wavecraft's version management, build-time injection, and platform-specific packaging."
   - "Related Documents" section linking to:
     - `[High-Level Design](./high-level-design.md)` — Architecture overview
     - `[Coding Standards Overview](./coding-standards.md)` — Versioning rules for developers
     - `[Plugin Formats](./plugin-formats.md)` — Format-specific packaging details
     - `[Development Workflows](./development-workflows.md)` — Build and release commands
     - `[CI Pipeline Guide](../guides/ci-pipeline.md)` — CD pipeline details
   - Horizontal rule (`---`)
   - Copy content from `high-level-design.md` L235–287 (`## Versioning`) — version flow diagram and key design decisions
   - Horizontal rule
   - Copy content from `high-level-design.md` L1494–1508 (`## Packaging & distribution notes`)

**Content to extract:**
- `## Versioning` (L235–287): version flow diagram, key design decisions
- `## Packaging & distribution notes` (L1494–1508): macOS/Windows/Linux packaging

**Verification:**
```bash
test -f docs/architecture/versioning-and-distribution.md && echo "✅ File exists"
wc -l docs/architecture/versioning-and-distribution.md  # Should be ~200 lines
grep '^## ' docs/architecture/versioning-and-distribution.md  # Should show: Related Documents, Versioning, Packaging
```

---

### Step 1.10: Phase 1 Commit

**Action:** Stage and commit all 9 new files.

```bash
git add docs/architecture/coding-standards-typescript.md \
        docs/architecture/coding-standards-css.md \
        docs/architecture/coding-standards-rust.md \
        docs/architecture/coding-standards-testing.md \
        docs/architecture/sdk-architecture.md \
        docs/architecture/declarative-plugin-dsl.md \
        docs/architecture/development-workflows.md \
        docs/architecture/plugin-formats.md \
        docs/architecture/versioning-and-distribution.md

git commit -m "docs: create 9 topic-specific architecture documents

Phase 1 of the documentation split. New files only, no existing
files modified. Content extracted from coding-standards.md and
high-level-design.md with Related Documents sections added."
```

**Verification:**
```bash
ls docs/architecture/*.md | wc -l  # Should be 12 (3 original + 9 new)
git diff --cached --stat            # Should show 9 new files
```

---

## Phase 2: Rewrite Hub Documents

This phase replaces the content of the two existing monolithic files with concise navigation hubs. After this phase, the originals become entry points with links to the split documents.

---

### Step 2.1: Rewrite `coding-standards.md` as a Navigation Hub

**File:** `docs/architecture/coding-standards.md`

**Action:** Replace the entire contents of `coding-standards.md` with a ~200-line navigation hub. The hub contains:

1. **Title and intro** (preserve L1–5 but update intro text):
   ```markdown
   # Coding Standards

   This document defines the coding standards and conventions for the Wavecraft project.
   For detailed rules, see the topic-specific documents below.
   ```

2. **Documentation Structure** — navigation table:
   ```markdown
   ## Documentation Structure

   | Document | Scope | When to Read |
   |----------|-------|-------------|
   | [TypeScript & React](./coding-standards-typescript.md) | Classes, hooks, React components, build constants, imports | Writing TypeScript or React code |
   | [CSS & Styling](./coding-standards-css.md) | TailwindCSS, theme tokens, WebView background | Styling or theming work |
   | [Rust](./coding-standards-rust.md) | Module org, DSL conventions, real-time safety, FFI, xtask | Writing Rust code |
   | [Testing & Quality](./coding-standards-testing.md) | Testing, linting, logging, error handling | Writing tests, debugging, CI |
   ```

3. **Quick Reference — Naming Conventions** — unified table combining the TypeScript naming table (original L200–211) and Rust naming table (original L649–658). Present as a single table with a "Language" column or two sub-tables.

4. **General** section (trimmed) containing:
   - `### Versioning` — Brief rule ("All version bumping is handled automatically by the CD pipeline.") + link: "See [Versioning and Distribution](./versioning-and-distribution.md) for the full version flow."
   - `### Comments and Documentation` — original content from L1266–1270 (5 lines, keep verbatim)
   - `### Documentation References` — original content from L1272–1297 (26 lines). **Update the "Required links" list** to include the new documents alongside the originals.

5. **Related Documents** section at the bottom:
   ```markdown
   ## Related Documents

   - [High-Level Design](./high-level-design.md) — Architecture overview
   - [Agent Development Flow](./agent-development-flow.md) — Agent roles and handoffs
   - [Roadmap](../roadmap.md) — Milestone tracking
   ```

**What is REMOVED from this file:**
- `## TypeScript / JavaScript` (L7–389) → now in `coding-standards-typescript.md`
- `## CSS / Styling (TailwindCSS)` (L391–527) → now in `coding-standards-css.md`
- `## Rust` (L529–987) → now in `coding-standards-rust.md`
- `## Testing` (L991–1155) → now in `coding-standards-testing.md`
- `## Linting & Formatting` (L1157–1223) → now in `coding-standards-testing.md`
- `### Logging` (L1299–1373) → now in `coding-standards-testing.md`
- `### Error Handling` (L1375–1379) → now in `coding-standards-testing.md`
- `### Validation Against Language Specifications` (L1381–1433) → now in `coding-standards-rust.md`
- `### Rust unwrap() and expect() Usage` (L1435–1501) → now in `coding-standards-rust.md`

**Verification:**
```bash
wc -l docs/architecture/coding-standards.md  # Should be ~200 lines (±30)
grep -c '\./coding-standards-' docs/architecture/coding-standards.md  # Should be 4+ (links to split docs)
grep 'versioning-and-distribution.md' docs/architecture/coding-standards.md  # Should find the versioning link
```

---

### Step 2.2: Rewrite `high-level-design.md` as a Navigation Hub

**File:** `docs/architecture/high-level-design.md`

**Action:** Replace the entire contents of `high-level-design.md` with a ~400-line overview hub. The hub **retains** the following sections verbatim:

**Sections to KEEP (verbatim):**
1. Title (L1)
2. `## Related Documents` (L5–13) — **expand** to include all new documents
3. `## Assumptions (explicit)` (L15–27, 13 lines)
4. `## Executive summary (one paragraph)` (L29–31, 3 lines)
5. `## Repository Structure (Monorepo)` (L33–155, 123 lines) — includes tree, benefits, component diagram, phase table
6. `## Architecture overview (block diagram, logical)` (L157–186, 30 lines)
7. `## Main components (concrete)` (L188–231, 44 lines)
8. `## Data flows and timing constraints` (L1421–1426, 6 lines)
9. `## Implementation recommendations (practical steps)` (L1428–1442, 15 lines)
10. `## Trade-offs and alternatives` (L1444–1456, 13 lines)
11. `## Real-time safety checklist (musts)` (L1458–1466, 9 lines)
12. `## Risks & mitigations` (L1510–1533, 24 lines)
13. `## Recommended libraries & tools (quick list)` (L1535–1544, 10 lines)
14. `## Minimal interface contract (example)` (L1546–1566, 21 lines)
15. `## Roadmap` (L1568–1572, 5 lines)
16. `## Appendix — Key references` (L1574–1579, 7 lines)

**New section to INSERT after `## Main components` (before `## Data flows`):**

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

**Sections to REMOVE (moved to split documents):**
- `## Versioning` (L235–287) → now in `versioning-and-distribution.md`
- `## Wavecraft SDK Architecture` (L289–579) → now in `sdk-architecture.md`
- `## Declarative Plugin DSL` (L581–815) → now in `declarative-plugin-dsl.md`
- `## Browser Development Mode` (L817–1029) → now in `development-workflows.md`
- `## Build System & Tooling` (L1031–1205) → now in `development-workflows.md`
- `## Visual Testing` (L1207–1259) → now in `development-workflows.md`
- `## Audio Unit (AU) Architecture` (L1262–1419) → now in `plugin-formats.md`
- `## Testing matrix` (L1469–1492) → now in `plugin-formats.md`
- `## Packaging & distribution notes` (L1494–1508) → now in `versioning-and-distribution.md`

**Update the `## Related Documents` section** (currently L5–13) to include links to all new documents:
```markdown
## Related Documents

- [Coding Standards](./coding-standards.md) — Conventions for TypeScript, Rust, and React code
- [Agent Development Flow](./agent-development-flow.md) — Agent roles and handoffs
- [Roadmap](../roadmap.md) — Project milestones and implementation plan
- [macOS Signing Guide](../guides/macos-signing.md) — Code signing and notarization setup
- [Visual Testing Guide](../guides/visual-testing.md) — Browser-based visual testing with Playwright
- [SDK Getting Started](../guides/sdk-getting-started.md) — Building plugins with Wavecraft SDK
- [SDK Architecture](./sdk-architecture.md) — SDK distribution, crate structure, npm packages
- [Declarative Plugin DSL](./declarative-plugin-dsl.md) — Macro system and parameter discovery
- [Development Workflows](./development-workflows.md) — Browser dev mode, build system, CI/CD
- [Plugin Formats](./plugin-formats.md) — VST3, CLAP, AU architecture
- [Versioning and Distribution](./versioning-and-distribution.md) — Version flow, packaging, signing
```

**Also update the internal link** in `## Main components` at L199:
- Current: `See [Coding Standards](./coding-standards.md#css--styling-tailwindcss) for details.`
- New: `See [CSS Standards](./coding-standards-css.md) for details.`

**Verification:**
```bash
wc -l docs/architecture/high-level-design.md  # Should be ~400 lines (±50)
grep '## Documentation Structure' docs/architecture/high-level-design.md  # Should exist
grep -c 'sdk-architecture\|declarative-plugin-dsl\|development-workflows\|plugin-formats\|versioning-and-distribution' docs/architecture/high-level-design.md  # Should be 10+
# Verify removed sections are gone:
grep '## Wavecraft SDK Architecture' docs/architecture/high-level-design.md  # Should NOT match
grep '## Declarative Plugin DSL' docs/architecture/high-level-design.md      # Should NOT match
grep '## Browser Development Mode' docs/architecture/high-level-design.md    # Should NOT match
```

---

### Step 2.3: Phase 2 Commit

**Action:** Stage and commit the two rewritten hub files.

```bash
git add docs/architecture/coding-standards.md \
        docs/architecture/high-level-design.md

git commit -m "docs: rewrite coding-standards.md and high-level-design.md as navigation hubs

Phase 2 of the documentation split. Replaced monolithic content with
concise navigation hubs linking to the 9 topic-specific documents
created in Phase 1."
```

**Verification:**
```bash
wc -l docs/architecture/coding-standards.md docs/architecture/high-level-design.md
# coding-standards.md: ~200 lines
# high-level-design.md: ~400 lines
```

---

## Phase 3: Cross-Reference Updates

This phase updates all files outside `docs/architecture/` that reference the split documents. The key insight from the LLD is that **most external references point to the hub files without anchors** and remain valid. Only anchor-specific links (`#testing`, `#versioning`, `#css--styling-tailwindcss`) need updating.

---

### Step 3.1: Update `.github/copilot-instructions.md`

**File:** `.github/copilot-instructions.md`

**Action:** Update lines 9–10 to reference the hub files with updated descriptions.

**Current (L9–10):**
```
- Before making changes in the code, check the #file:../docs/architecture/coding-standards.md file for relevant coding standards and conventions.
- For understanding the overall project architecture, SDK structure, and design decisions, refer to #file:../docs/architecture/high-level-design.md document.
```

**New (L9–10):**
```
- Before making changes in the code, check the #file:../docs/architecture/coding-standards.md file for relevant coding standards and navigation to language-specific guides.
- For understanding the overall project architecture, refer to #file:../docs/architecture/high-level-design.md (overview with links to detailed topic docs).
```

**Rationale:** Keep the same two `#file:` directives — they now load ~200 + ~400 = ~600 lines instead of ~1,500 + ~1,580 = ~3,080 lines. Agents follow links in the hubs to topic-specific docs as needed.

**Verification:**
```bash
grep 'coding-standards.md' .github/copilot-instructions.md  # Should show updated description
grep 'high-level-design.md' .github/copilot-instructions.md  # Should show updated description
```

---

### Step 3.2: Update `.github/agents/PO.agent.md`

**File:** `.github/agents/PO.agent.md`

**Action:** Update L175 to point to the new versioning document.

**Current:**
```
See the [Coding Standards — Versioning](../../docs/architecture/coding-standards.md#versioning) section for details.
```

**New:**
```
See [Versioning and Distribution](../../docs/architecture/versioning-and-distribution.md) for details.
```

**Verification:**
```bash
grep 'versioning' .github/agents/PO.agent.md  # Should show new link, not #versioning anchor
```

---

### Step 3.3: Update `CONTRIBUTING.md`

**File:** `CONTRIBUTING.md`

**Action:** Two changes:

1. **L94** — Update the testing anchor reference:
   - Current: `See [docs/architecture/coding-standards.md#testing](docs/architecture/coding-standards.md#testing) for testing guidelines.`
   - New: `See [Testing & Quality Standards](docs/architecture/coding-standards-testing.md) for testing guidelines.`

2. **Add a new section** before the final section (or at an appropriate location) documenting the documentation structure:
   ```markdown
   ## Documentation Structure

   Architecture documentation lives in `docs/architecture/` and follows these conventions:

   - **Overview hubs** (`coding-standards.md`, `high-level-design.md`) — Navigation entry points with links to topic-specific docs
   - **Topic-specific docs** — Self-contained documents for specific domains (e.g., `coding-standards-rust.md`, `sdk-architecture.md`)
   - **Target size**: 150–600 lines per document
   - **Required sections**: Title, description, Related Documents, content

   See the navigation tables in [Coding Standards](docs/architecture/coding-standards.md)
   and [High-Level Design](docs/architecture/high-level-design.md) for the full document list.
   ```

**Note:** The L38 reference (`docs/architecture/coding-standards.md`) does NOT need updating — it links to the hub file without an anchor, and the hub still exists at the same path.

**Verification:**
```bash
grep 'coding-standards-testing.md' CONTRIBUTING.md       # Should find testing link
grep '## Documentation Structure' CONTRIBUTING.md         # Should find new section
grep 'coding-standards.md#testing' CONTRIBUTING.md        # Should NOT match (stale anchor)
```

---

### Step 3.4: Update `README.md`

**File:** `README.md`

**Action:** Update L111–112 descriptions to reflect that documents are now hubs.

**Current:**
```
- [High-Level Design](docs/architecture/high-level-design.md) — Architecture overview, component design, data flows, and implementation roadmap
- [Coding Standards](docs/architecture/coding-standards.md) — TypeScript, Rust, and React conventions
```

**New:**
```
- [High-Level Design](docs/architecture/high-level-design.md) — Architecture overview and navigation hub for detailed topic docs
- [Coding Standards](docs/architecture/coding-standards.md) — Coding conventions overview and navigation hub for language-specific guides
```

**Verification:**
```bash
grep 'navigation hub' README.md  # Should find both updated descriptions
```

---

### Step 3.5: Update `docs/guides/ci-pipeline.md`

**File:** `docs/guides/ci-pipeline.md`

**Action:** Update L291 description.

**Current:**
```
- [Coding Standards](../architecture/coding-standards.md) — Code conventions including linting rules
```

**New:**
```
- [Coding Standards](../architecture/coding-standards.md) — Code conventions overview (see [Testing & Quality](../architecture/coding-standards-testing.md) for linting rules)
```

**Verification:**
```bash
grep 'coding-standards' docs/guides/ci-pipeline.md  # Should show updated description
```

---

### Step 3.6: Update `docs/guides/sdk-getting-started.md`

**File:** `docs/guides/sdk-getting-started.md`

**Action:** Update L599–600 descriptions.

**Current:**
```
- **[High-Level Design](../architecture/high-level-design.md)** — Understand the architecture
- **[Coding Standards](../architecture/coding-standards.md)** — Follow project conventions
```

**New:**
```
- **[High-Level Design](../architecture/high-level-design.md)** — Architecture overview and detailed topic docs
- **[Coding Standards](../architecture/coding-standards.md)** — Coding conventions and language-specific guides
```

**Verification:**
```bash
grep -A1 'High-Level Design' docs/guides/sdk-getting-started.md | tail -2  # Should show updated descriptions
```

---

### Step 3.7: Update `cli/sdk-templates/new-project/react/README.md`

**File:** `cli/sdk-templates/new-project/react/README.md`

**Action:** Update L404–405 descriptions (these use absolute GitHub URLs, not relative paths).

**Current:**
```
- **[Coding Standards](https://github.com/RonHouben/wavecraft/blob/main/docs/architecture/coding-standards.md)** — Best practices
- **[High-Level Design](https://github.com/RonHouben/wavecraft/blob/main/docs/architecture/high-level-design.md)** — Architecture overview
```

**New:**
```
- **[Coding Standards](https://github.com/RonHouben/wavecraft/blob/main/docs/architecture/coding-standards.md)** — Coding conventions and language-specific guides
- **[High-Level Design](https://github.com/RonHouben/wavecraft/blob/main/docs/architecture/high-level-design.md)** — Architecture overview and detailed topic docs
```

**Verification:**
```bash
grep 'coding-standards\|high-level-design' cli/sdk-templates/new-project/react/README.md | grep -c 'guides\|topic'  # Should be 2
```

---

### Step 3.8: Update `docs/feature-specs/remove-manual-versioning/PR-summary.md`

**File:** `docs/feature-specs/remove-manual-versioning/PR-summary.md`

**Action:** Update L37–38 anchor links (these are active feature spec files, not archived).

**Current:**
```
- [Coding Standards — Versioning](/docs/architecture/coding-standards.md#versioning) — Authoritative versioning policy
- [High-Level Design — Versioning](/docs/architecture/high-level-design.md#versioning) — Version flow architecture
```

**New:**
```
- [Versioning and Distribution](/docs/architecture/versioning-and-distribution.md) — Authoritative versioning policy and version flow architecture
```

**Note:** Since both anchors (`#versioning`) pointed to content now in a single document, merge them into one link.

**Verification:**
```bash
grep 'versioning-and-distribution' docs/feature-specs/remove-manual-versioning/PR-summary.md  # Should match
grep '#versioning' docs/feature-specs/remove-manual-versioning/PR-summary.md  # Should NOT match
```

---

### Step 3.9: Verify `.github/agents/QA.agent.md` — No Changes Needed

**File:** `.github/agents/QA.agent.md`

**Action:** Verify that the existing references are still correct (they reference hub files without anchors):
- L47: `docs/architecture/coding-standards.md` — hub still exists ✅
- L48: `docs/architecture/high-level-design.md` — hub still exists ✅
- L88: `high-level-design.md` — still valid ✅
- L197: `high-level-design.md` — still valid ✅

**No changes required.** The QA agent references the hub files, which remain valid.

**Verification:**
```bash
rg 'coding-standards\.md#|high-level-design\.md#' .github/agents/QA.agent.md  # Should return nothing (no anchor refs)
```

---

### Step 3.10: Phase 3 Commit

**Action:** Stage and commit all cross-reference updates.

```bash
git add .github/copilot-instructions.md \
        .github/agents/PO.agent.md \
        CONTRIBUTING.md \
        README.md \
        docs/guides/ci-pipeline.md \
        docs/guides/sdk-getting-started.md \
        cli/sdk-templates/new-project/react/README.md \
        docs/feature-specs/remove-manual-versioning/PR-summary.md

git commit -m "docs: update all cross-references for documentation split

Phase 3 of the documentation split. Updated anchor-specific links,
descriptions, and added documentation structure section to
CONTRIBUTING.md."
```

---

## Phase 4: Validation

This phase verifies the entire split is consistent and complete.

---

### Step 4.1: Run Link Checker Script

**Action:**
```bash
scripts/check-links.sh
```

**Expected:** Exit code 0, zero broken links.

**If failures:** Fix any broken links in the files identified by the script, then re-run.

---

### Step 4.2: Grep for Stale Anchor References

**Action:** Run these grep commands to find any remaining stale anchor references:

```bash
# Stale anchors in coding-standards.md (sections that moved)
rg 'coding-standards\.md#(typescript|css|rust|testing|linting|logging|error-handling|ffi|real-time|spsc|nih-plug|unwrap)' docs/ .github/ CONTRIBUTING.md README.md cli/ --type md | grep -v _archive | grep -v feature-specs/docs-split

# Stale anchors in high-level-design.md (sections that moved)
rg 'high-level-design\.md#(sdk|dsl|browser-dev|build-system|visual-testing|audio-unit|versioning|packaging|ci-cd)' docs/ .github/ CONTRIBUTING.md README.md cli/ --type md | grep -v _archive | grep -v feature-specs/docs-split

# Specific known anchors that should no longer appear
rg 'coding-standards\.md#testing' docs/ .github/ CONTRIBUTING.md --type md | grep -v _archive | grep -v feature-specs/docs-split
rg 'coding-standards\.md#versioning' docs/ .github/ CONTRIBUTING.md --type md | grep -v _archive | grep -v feature-specs/docs-split
rg 'coding-standards\.md#css--styling-tailwindcss' docs/ .github/ --type md | grep -v _archive | grep -v feature-specs/docs-split
rg 'coding-standards\.md#sdk-distribution-versioning' docs/ .github/ --type md | grep -v _archive | grep -v feature-specs/docs-split
rg 'high-level-design\.md#dev-audio-via-ffi' docs/ .github/ --type md | grep -v _archive | grep -v feature-specs/docs-split
```

**Expected:** All commands return empty (no matches).

**If matches found:** Update the references in the identified files.

---

### Step 4.3: Verify Document Count and Sizes

**Action:**
```bash
# Count all architecture docs
ls docs/architecture/*.md | wc -l
# Expected: 12 (agent-development-flow.md + coding-standards.md + coding-standards-typescript.md + coding-standards-css.md + coding-standards-rust.md + coding-standards-testing.md + high-level-design.md + sdk-architecture.md + declarative-plugin-dsl.md + development-workflows.md + plugin-formats.md + versioning-and-distribution.md)

# Check sizes — all should be ≤600 lines
wc -l docs/architecture/*.md

# Verify no document exceeds 600 lines
awk 'END{if(NR>600) print FILENAME, NR, "EXCEEDS 600 LINES"}' docs/architecture/*.md

# Verify no document is below 150 lines (except the hub might be close)
awk 'END{if(NR<100) print FILENAME, NR, "SUSPICIOUSLY SMALL"}' docs/architecture/*.md
```

**Expected:**
- 12 files in `docs/architecture/`
- No file exceeds 600 lines
- No file is suspiciously small (< 100 lines)

---

### Step 4.4: Verify Content Completeness

**Action:** Check that all original H2/H3 headings still exist across all documents.

```bash
# Count total H2 + H3 headings in all architecture docs
grep -c '^##' docs/architecture/*.md | awk -F: '{sum+=$2} END {print "Total headings:", sum}'

# Original heading counts for comparison:
grep -c '^##' /dev/stdin <<< "$(git show HEAD~2:docs/architecture/coding-standards.md 2>/dev/null || echo '')"
# Or just compare manually — the split should have >= the original number of headings
```

**Also verify specific critical headings exist _somewhere_ in the architecture docs:**
```bash
grep -rl 'FFI Safety Patterns' docs/architecture/        # Should find coding-standards-rust.md
grep -rl 'Lock-Free Parameter Bridge' docs/architecture/  # Should find coding-standards-rust.md
grep -rl 'Browser Development Mode' docs/architecture/    # Should find development-workflows.md
grep -rl 'Audio Unit (AU)' docs/architecture/             # Should find plugin-formats.md
grep -rl 'SDK Distribution Model' docs/architecture/      # Should find sdk-architecture.md
grep -rl 'DSL Architecture' docs/architecture/            # Should find declarative-plugin-dsl.md
grep -rl 'Pre-Push Validation' docs/architecture/         # Should find coding-standards-testing.md
grep -rl 'Theme Tokens' docs/architecture/                # Should find coding-standards-css.md
```

---

### Step 4.5: Final Commit (if any fixes from validation)

**Action:** If any fixes were needed from steps 4.1–4.4, commit them:

```bash
git add -A
git commit -m "docs: fix validation issues from documentation split"
```

If no fixes were needed, skip this step.

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Broken links after hub rewrite** | High | Phase 1 creates new files first (zero breakage). Phase 3 systematically updates all references. Phase 4 validates with `check-links.sh`. |
| **Content loss during extraction** | Medium | Step 4.4 verifies all critical headings exist. Coder should use exact line ranges from source files. |
| **Agent confusion (wrong doc loaded)** | Medium | Hub files contain clear navigation tables. `copilot-instructions.md` still references hubs. |
| **Stale anchor links in archived specs** | None | Archived specs are out of scope per project rules. `check-links.sh` already excludes `_archive/`. |
| **Template file missed** | Low | Step 3.7 explicitly handles the SDK template README. |

---

## Success Criteria

- [ ] 9 new topic-specific documents exist in `docs/architecture/`
- [ ] `coding-standards.md` is ≤250 lines (hub)
- [ ] `high-level-design.md` is ≤450 lines (hub)
- [ ] All new documents have "Related Documents" sections
- [ ] All new documents are between 150–600 lines
- [ ] `scripts/check-links.sh` passes with zero broken links
- [ ] No stale anchor references found by Phase 4 grep patterns
- [ ] All critical headings (FFI, SPSC, DSL, AU, SDK, etc.) exist in split docs
- [ ] `CONTRIBUTING.md` has new "Documentation Structure" section
- [ ] `.github/copilot-instructions.md` updated for token efficiency
- [ ] 12 total files in `docs/architecture/`

---

## Files Modified Summary

### New Files (9)
| File | Estimated Lines |
|------|----------------|
| `docs/architecture/coding-standards-typescript.md` | ~400 |
| `docs/architecture/coding-standards-css.md` | ~150 |
| `docs/architecture/coding-standards-rust.md` | ~550 |
| `docs/architecture/coding-standards-testing.md` | ~350 |
| `docs/architecture/sdk-architecture.md` | ~500 |
| `docs/architecture/declarative-plugin-dsl.md` | ~300 |
| `docs/architecture/development-workflows.md` | ~350 |
| `docs/architecture/plugin-formats.md` | ~300 |
| `docs/architecture/versioning-and-distribution.md` | ~200 |

### Modified Files (8)
| File | Change |
|------|--------|
| `docs/architecture/coding-standards.md` | Rewritten as ~200-line hub |
| `docs/architecture/high-level-design.md` | Rewritten as ~400-line hub |
| `.github/copilot-instructions.md` | Updated descriptions (L9–10) |
| `.github/agents/PO.agent.md` | Updated versioning link (L175) |
| `CONTRIBUTING.md` | Updated testing link (L94) + new Documentation Structure section |
| `README.md` | Updated descriptions (L111–112) |
| `docs/guides/ci-pipeline.md` | Updated description (L291) |
| `docs/guides/sdk-getting-started.md` | Updated descriptions (L599–600) |
| `cli/sdk-templates/new-project/react/README.md` | Updated descriptions (L404–405) |
| `docs/feature-specs/remove-manual-versioning/PR-summary.md` | Updated versioning links (L37–38) |

### Unchanged Files
| File | Reason |
|------|--------|
| `docs/architecture/agent-development-flow.md` | 254 lines, within target, standalone |
| `docs/feature-specs/_archive/**` | Must not edit archived specs |
| `docs/roadmap.md` | Only PO can edit |
| `.github/agents/QA.agent.md` | References hub files without anchors — still valid |
| `.github/skills/**` | No references to split files found |
