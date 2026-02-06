# PR Summary: Crates.io Publishing Preparation

## Summary

Splits `wavecraft-core` into two crates to enable crates.io publishing:
- **`wavecraft-core`** (publishable) — Core SDK types, declarative macros, no nih_plug dependency
- **`wavecraft-nih_plug`** (git-only) — nih-plug integration, WebView editor, plugin exports

This allows the SDK ecosystem to be discoverable on crates.io while keeping nih_plug integration separate (nih_plug has unpublished dependencies that prevent crates.io publishing).

## Changes

### Engine/DSP
- New `wavecraft-nih_plug` crate with editor, assets, and plugin infrastructure
- Refactored `wavecraft-core` to remove nih_plug dependencies
- Updated `wavecraft-macros` to generate code compatible with crate split
- Updated all crate dependencies for the new architecture

### Build/Config
- Updated `continuous-deploy.yml` for crates.io publishing workflow
- Added git tagging support in CI
- Updated template to use Cargo rename pattern (`wavecraft = { package = "wavecraft-nih_plug" }`)

### Documentation
- Updated `high-level-design.md` with SDK Distribution Model diagram
- Updated `coding-standards.md` module organization
- Updated `roadmap.md` for Milestone 13 progress
- Comprehensive feature specs (LLD, implementation plan, test plan, QA report)

## Commits

- `e48cc65` docs: update implementation progress with QA results
- `bfd2ec9` docs: update roadmap for crate split (M13 In Progress)
- `344723b` docs: update coding-standards crate structure for nih_plug split
- `47b62f1` docs: update architecture docs for wavecraft-nih_plug crate split
- `45bbcbc` docs: update QA report - mark Finding #1 as resolved
- `10749b7` docs: document parameter sync as known limitation
- `8ec70d6` docs: update README to clarify project status
- `2a88154` qa: add QA report for crate split implementation
- `62b08b9` test: verify crate split implementation (TC-017 to TC-026)
- `aa8b620` feat: split wavecraft-core for crates.io publishing
- `bb81fea` feat: add nih-plug independence strategy to backlog
- `52f5af2` Add test plan for crates.io publishing
- `549ec1a` feat: update continuous deployment workflow
- `bb15bb5` feat: add implementation plan and progress tracking
- `e9443b8` feat: enhance npm publishing with scoped packages

## Related Documentation

- [Implementation Plan - Core Split](./implementation-plan-core-split.md)
- [Low-Level Design - Core Split](./low-level-design-core-split.md)
- [Test Plan](./test-plan.md)
- [QA Report](./QA-report.md)

## Testing

- [x] Build passes: `cargo build --workspace`
- [x] Tests pass: 24/24 engine tests passing
- [x] wavecraft-core compiles independently (no nih_plug)
- [x] Plugin builds successfully with new crate structure
- [x] QA review completed (0 Critical/High/Medium, 1 Low resolved)

## Checklist

- [x] Code follows project coding standards
- [x] Tests added/updated as needed
- [x] Documentation updated
- [x] No linting errors
