## Summary

Split two large architecture documents (`coding-standards.md` at 1,502 lines and `high-level-design.md` at 1,579 lines) into 9 focused topic-specific documents plus 2 concise navigation hubs. This achieves an 84.5% reduction in hub document size (3,081 → 479 lines), improves developer navigation, and reduces AI agent token consumption by ~80%.

## Changes

- **Documentation (new)**: 9 topic-specific architecture documents:
  - `coding-standards-typescript.md` (391 lines) — TypeScript, React, hooks, build constants
  - `coding-standards-css.md` (145 lines) — TailwindCSS, theming, WebView styling
  - `coding-standards-rust.md` (600 lines) — Module org, DSL, xtask, FFI safety
  - `coding-standards-testing.md` (328 lines) — Testing, logging, error handling
  - `sdk-architecture.md` (299 lines) — SDK distribution, crates, npm packages
  - `declarative-plugin-dsl.md` (241 lines) — DSL architecture, macros, discovery
  - `development-workflows.md` (457 lines) — Browser dev, FFI audio, build system
  - `plugin-formats.md` (204 lines) — VST3, CLAP, AU architecture
  - `versioning-and-distribution.md` (128 lines) — Version flow, packaging, signing
- **Documentation (rewritten)**: 2 hub documents as navigation pages:
  - `coding-standards.md` (1,502 → 98 lines)
  - `high-level-design.md` (1,579 → 381 lines)
- **Cross-references**: Updated 8 files (`.github/copilot-instructions.md`, `PO.agent.md`, `CONTRIBUTING.md`, `README.md`, `ci-pipeline.md`, `sdk-getting-started.md`, template README, PR-summary)
- **Documentation (archived)**: Feature spec archived to `_archive/docs-split-architecture/`
- **Roadmap**: M18.6 marked complete, progress 91% (20/22)

## Commits

- `b77fd25` docs: add low-level design for architecture documentation split
- `145f127` docs: add implementation plan and progress tracker for documentation split
- `b2bb0ff` docs: create 9 topic-specific architecture documents (Phase 1)
- `cc7140e` docs: rewrite coding-standards.md and high-level-design.md as navigation hubs
- `34ef9e7` docs: update all cross-references for documentation split
- `c6e20af` docs: fix validation issues from documentation split
- `de18cad` docs: update implementation progress tracker — all phases complete
- `8aac11d` test: add test plan for docs-split-architecture (18/18 passing)
- `c93fddd` docs: add QA report for docs-split-architecture (Milestone 18.6)
- `eaf4fcd` docs: fix QA findings for docs-split-architecture (QA-M1, QA-L1)
- `5f0743d` chore: complete Milestone 18.6 — Documentation Architecture Split

## Related Documentation

- [User Stories](docs/feature-specs/_archive/docs-split-architecture/user-stories.md)
- [Low-Level Design](docs/feature-specs/_archive/docs-split-architecture/low-level-design-docs-split.md)
- [Implementation Plan](docs/feature-specs/_archive/docs-split-architecture/implementation-plan.md)
- [Test Plan](docs/feature-specs/_archive/docs-split-architecture/test-plan.md)
- [QA Report](docs/feature-specs/_archive/docs-split-architecture/QA-report.md)

## Testing

- [x] `cargo xtask ci-check` passes (187 tests, all lint clean)
- [x] `scripts/check-links.sh` — 27 files checked, 0 broken links
- [x] Manual verification: all 9 new documents have "Related Documents" sections
- [x] Manual verification: hub documents link to all split documents
- [x] Grep for stale anchors: 0 matches

## Checklist

- [x] Documentation follows project coding standards
- [x] No linting errors
- [x] All cross-references updated
- [x] Feature spec archived
- [x] Roadmap updated
- [x] QA approved (1 Medium finding fixed)
