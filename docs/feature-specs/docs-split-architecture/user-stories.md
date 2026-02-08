# User Stories: Architecture Documentation Split

## Overview

The Wavecraft architecture documentation has grown to over 3,000 lines across two files (`coding-standards.md` and `high-level-design.md`). This creates friction for both human readers and AI agents:

- **Developers** struggle with navigation, information overload, and slow page loads (1,500+ lines per document)
- **AI agents** consume 3,000-6,000 tokens per documentation read, even when they only need specific sections
- **Contributors** face difficulty finding and updating specific topics buried in large files

This refactoring splits the two large documents into focused, topic-specific files that are easier to navigate, maintain, and consume.

---

## Version

**Target Version:** `0.10.1` (patch bump from `0.10.0`)

**Rationale:** 
This is a documentation restructuring with no code changes. Following the [Coding Standards — Versioning](../../architecture/coding-standards.md#versioning) guidelines, documentation-only changes qualify as patch bumps. The change improves developer experience but doesn't introduce new features or modify behavior.

---

## User Story 1: As a Developer, I Want Focused Documentation So I Can Find Information Quickly

**As a** developer new to Wavecraft  
**I want** focused, topic-specific documentation files  
**So that** I can quickly find the information I need without scrolling through 1,500+ line documents

### Acceptance Criteria

- [ ] `coding-standards.md` is reduced from 1,511 lines to ~200 lines (overview with navigation)
- [ ] `high-level-design.md` is reduced from 1,562 lines to ~400 lines (architecture overview)
- [ ] Each new document is between 150-600 lines (readable in one sitting)
- [ ] Main overview documents contain clear "Related Documents" sections with links
- [ ] All documents have descriptive titles that match their content

### Notes

- Apply the "one document, one topic" principle
- Keep minimum document size ~150 lines to avoid over-fragmentation
- Ensure each document can stand alone with minimal cross-referencing

---

## User Story 2: As an AI Agent, I Want Token-Efficient Documentation So I Can Load Only Relevant Context

**As an** AI agent (Architect, Coder, Tester, QA)  
**I want** documentation split by domain/topic  
**So that** I can load only the sections relevant to my current task, reducing token consumption by 80-90%

### Acceptance Criteria

- [ ] Coding standards split into 4 focused documents (TypeScript, CSS, Rust, Testing)
- [ ] High-level design split into 5 focused documents (SDK, DSL, Workflows, Formats, Versioning)
- [ ] Agent instructions (`.github/copilot-instructions.md`) reference the new document structure
- [ ] Token usage reduction validated: baseline 3,000-6,000 tokens → 200-600 tokens per focused read

### Notes

- Agents should be able to determine which document to read based on file names
- Cross-references between documents should use relative links
- Overview documents should clearly indicate which topics live in which files

---

## User Story 3: As a Coder Agent, I Want Language-Specific Guides So I Only Load Relevant Coding Standards

**As a** Coder agent working on Rust or TypeScript code  
**I want** separate coding standards for each language  
**So that** I don't load irrelevant guidelines (e.g., TypeScript rules when writing Rust)

### Acceptance Criteria

- [ ] New file: `coding-standards-typescript.md` (~400 lines) — TypeScript, React, hooks, build constants
- [ ] New file: `coding-standards-css.md` (~150 lines) — TailwindCSS, theming, WebView styling
- [ ] New file: `coding-standards-rust.md` (~600 lines) — Module org, DSL, xtask, platform-specific, FFI
- [ ] New file: `coding-standards-testing.md` (~300 lines) — Testing, logging, error handling, validation
- [ ] Each file begins with a "Related Documents" section linking to the overview

### Notes

- Language-specific documents should be self-contained for that language
- Cross-language patterns (e.g., naming conventions) should be in the overview
- Avoid duplicating content between language-specific docs

---

## User Story 4: As an Architect Agent, I Want Topic-Focused Design Docs So I Can Navigate Architecture Efficiently

**As an** Architect agent reviewing or updating system design  
**I want** separate documents for SDK, DSL, workflows, plugin formats, and versioning  
**So that** I can quickly locate and update specific architectural areas without navigating a 1,500-line monolith

### Acceptance Criteria

- [ ] New file: `sdk-architecture.md` (~500 lines) — SDK distribution, crate structure, npm packages, API
- [ ] New file: `declarative-plugin-dsl.md` (~300 lines) — DSL architecture, macros, parameter discovery
- [ ] New file: `development-workflows.md` (~400 lines) — Browser dev mode, FFI audio, build system, testing
- [ ] New file: `plugin-formats.md` (~300 lines) — VST3, CLAP, AU architecture and format specifics
- [ ] New file: `versioning-and-distribution.md` (~200 lines) — Version flow, build-time injection, packaging
- [ ] Updated: `high-level-design.md` becomes a 400-line overview with clear navigation

### Notes

- Each document should be independently comprehensible (assume reader hasn't read others)
- Overview should contain the architecture diagram and component relationships
- Heavy technical details move to topic-specific docs

---

## User Story 5: As a Contributor, I Want Clear Navigation Hubs So I Know Where to Find and Update Information

**As a** contributor updating documentation  
**I want** overview documents that act as navigation hubs with clear links  
**So that** I can quickly find which file to edit when updating specific topics

### Acceptance Criteria

- [ ] `coding-standards.md` contains a table mapping topics → new document names
- [ ] `high-level-design.md` contains a "Documentation Structure" section listing all split documents
- [ ] Each split document has a "Related Documents" section at the top
- [ ] All internal links use relative paths (e.g., `[SDK Architecture](sdk-architecture.md)`)
- [ ] No broken links after split (validated by `scripts/check-links.sh`)

### Notes

- Navigation hubs should be scannable (use tables, lists, clear headings)
- Prefer breadcrumb-style navigation ("Overview → Topic → Detail")
- Update agent instructions to reference hub documents as entry points

---

## User Story 6: As a Developer, I Want All Documentation Links to Work So I Don't Hit Dead Ends

**As a** developer navigating documentation  
**I want** all cross-references and links to remain functional after the split  
**So that** I can follow links without encountering 404s or broken references

### Acceptance Criteria

- [ ] All links within architecture docs updated to reference new file names
- [ ] All links in agent instructions updated (`.github/copilot-instructions.md`)
- [ ] All links in skills updated (`.github/skills/**/SKILL.md`)
- [ ] All links in guides updated (`docs/guides/*.md`)
- [ ] `scripts/check-links.sh` passes with zero broken links
- [ ] README documentation links updated

### Notes

- Use systematic search/replace for common patterns
- Check for both markdown links `[text](path)` and inline references
- Test links in both GitHub web UI and local markdown viewers
- Update any automated documentation generators that reference these files

---

## User Story 7: As a Maintainer, I Want a Clear Migration Guide So Future Documentation Doesn't Regress

**As a** project maintainer  
**I want** clear guidelines on when to create new documentation vs extending existing files  
**So that** documentation stays organized as the project grows

### Acceptance Criteria

- [ ] New section added to `CONTRIBUTING.md` documenting the documentation structure
- [ ] Guidelines specify: single-topic focus, 150-600 line target, when to split
- [ ] Table listing all architecture documents with their purposes and scopes
- [ ] Contact points for documentation questions (link to agent-development-flow.md)

### Notes

- Document the "why" behind the split to prevent future regression
- Specify that architecture docs should remain focused, not grow indefinitely
- Mention token efficiency as a key consideration for AI agent consumption
- Link to the agent development flow for documentation update responsibilities

---

## Implementation Notes

### Document Split Mapping

#### Coding Standards (1,511 lines → 200 + 4 focused docs)

| New Document | Lines | Content |
|--------------|-------|---------|
| `coding-standards.md` (overview) | 200 | Principles, quick reference, navigation |
| `coding-standards-typescript.md` | 400 | TypeScript, React, hooks, build constants |
| `coding-standards-css.md` | 150 | TailwindCSS, theming, WebView styling |
| `coding-standards-rust.md` | 600 | Module org, DSL, xtask, platform-specific, FFI |
| `coding-standards-testing.md` | 300 | Testing, logging, error handling, validation |

#### High-Level Design (1,562 lines → 400 + 5 focused docs)

| New Document | Lines | Content |
|--------------|-------|---------|
| `high-level-design.md` (overview) | 400 | Assumptions, executive summary, architecture diagram, navigation |
| `sdk-architecture.md` | 500 | SDK distribution, crate structure, npm packages, public API |
| `declarative-plugin-dsl.md` | 300 | DSL architecture, macros, parameter discovery, limitations |
| `development-workflows.md` | 400 | Browser dev mode, FFI audio, build system, visual testing |
| `plugin-formats.md` | 300 | VST3, CLAP, AU architecture and format specifics |
| `versioning-and-distribution.md` | 200 | Version flow, build-time injection, packaging, signing |

### Cross-Reference Update Scope

Files that reference architecture docs (estimated):
- `.github/copilot-instructions.md` (3 attachments)
- `.github/skills/**/SKILL.md` (unknown, needs scan)
- `docs/guides/*.md` (4 files)
- `docs/feature-specs/_archive/**/low-level-design-*.md` (many)
- `docs/roadmap.md`
- `README.md`

### Validation Checklist

- [ ] Run `scripts/check-links.sh` to verify all links
- [ ] Grep for old file references: `rg 'coding-standards\.md' docs/`
- [ ] Grep for old file references: `rg 'high-level-design\.md' docs/`
- [ ] Test with AI agents: verify token reduction
- [ ] Manual review: each new document reads coherently standalone

---

## Success Metrics

| Metric | Baseline | Target | How to Measure |
|--------|----------|--------|----------------|
| **Largest doc size** | 1,562 lines | <600 lines | `wc -l docs/architecture/*.md` |
| **Token usage (typical agent read)** | 3,000-6,000 | 200-600 | Count tokens in focused docs |
| **Broken links** | N/A | 0 | `scripts/check-links.sh` exit code |
| **Time to find info** | ~2-3 minutes scrolling | <30 seconds | Manual testing with navigation hubs |
| **Documents in architecture/** | 3 | 14 | `ls docs/architecture/ \| wc -l` |

---

## Dependencies

- No code dependencies
- No build system changes
- No external tool changes

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Broken links** | High (documentation unusable) | Systematic search/replace + check-links.sh |
| **Agent confusion** | Medium (agents load wrong docs) | Update `.github/copilot-instructions.md` |
| **Over-fragmentation** | Low (too many small docs) | Minimum 150 lines per document |
| **Loss of context** | Medium (documents feel disconnected) | Strong navigation hubs with related docs sections |

---

## Out of Scope

- Content updates to the documentation (this is purely structural)
- Documentation generation automation
- CI validation of documentation structure
- Documentation linting/style enforcement
