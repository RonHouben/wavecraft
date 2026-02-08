# Macro API Simplification (v0.9.0)

## Summary

This PR implements Milestone 16 (Macro API Simplification), reducing the `wavecraft_plugin!` macro API from 9 lines to 4 lines—a 56% reduction. The feature simplifies plugin declarations by:

- Renaming `SignalChain` to `signal` for clarity
- Eliminating the `processors` array wrapper (directly accept single Processor type)
- Updating CLI templates with the streamlined DSL
- Applying 7 documentation improvements from comprehensive QA review

The implementation includes full testing (107 tests passing), QA validation (9 findings addressed), and architectural review. Known limitations (parameter sync for DSL-generated plugins) are documented and approved as acceptable for v0.9.0.

## Changes

### Engine/DSP
- **wavecraft-macros**: Core API changes to `wavecraft_plugin!` macro
  - Renamed `SignalChain` field to `signal`
  - Removed `processors` array wrapper
  - Enhanced SAFETY comments for unsafe buffer operations
  - Improved error messages (expect() instead of unwrap())
  - Added documentation for known limitations
- **wavecraft-dsp**: Updated `Chain![]` macro and combinators to support new syntax
- **wavecraft-core**: Updated prelude exports
- **Version bumps**: All workspace crates bumped to 0.9.0

### CLI
- **Templates**: Updated `new-project/react` template with 4-line DSL syntax
- **Template variables**: Enhanced project template generation
- **CLI version**: Bumped to 0.9.0

### Build/Config
- **Cargo.toml**: Version updates across workspace (engine + CLI)
- **Cargo.lock**: Dependency updates

### Documentation
- **Feature Spec Archive**: Complete feature documentation moved to `_archive/`
  - User stories, low-level design, implementation plan
  - Implementation progress, test plan (10/10 passing)
  - QA report (9 findings), architect review
- **Architectural Docs**: 
  - coding-standards.md: Added Known Limitations section
  - high-level-design.md: Documented parameter sync trade-off
- **Roadmap**: Milestone 16 marked complete (16/18 = 89%)

## Commits

- `d90e118` Complete Macro API Simplification (v0.9.0)
- `e59cf6d` WIP: macro api simplification implementation in progress
- `1a8536c` docs: clean up architectural assessments for dev audio input
- `1d4a7d3` docs: create Milestone 16 - Macro API Simplification

## Related Documentation

- [User Stories](./user-stories.md) - Requirements and acceptance criteria
- [Low-Level Design](./low-level-design.md) - Technical design decisions
- [Implementation Plan](./implementation-plan.md) - Step-by-step implementation strategy
- [Implementation Progress](./implementation-progress.md) - Development tracking
- [Test Plan](./test-plan.md) - Comprehensive functional testing (10/10 passing)
- [QA Report](./QA-report.md) - Code quality analysis (9 findings addressed)
- [Architect Review](./architect-review.md) - Technical validation
- [Architect Review Summary](./architect-review-summary.md) - Key takeaways

## Testing

### Automated Testing
- ✅ **CI Checks**: All checks passing (`cargo xtask ci-check`)
  - Linting: 5.7s (ESLint, Prettier, cargo fmt, clippy)
  - Tests: 10.7s (107 tests: 69 engine + 28 UI + 10 doctests)

### Manual Testing
- ✅ **Functional Tests**: 10/10 passing (see [test-plan.md](./test-plan.md))
  - Template generation with new 4-line syntax
  - Plugin compilation with simplified DSL
  - Parameter discovery and IPC communication
  - Dev server WebSocket transport
  - CLI template variable substitution
  - Documentation completeness

### QA Validation
- ✅ **Code Quality**: 9 findings identified and addressed
  - 2 High severity (approved as acceptable for v0.9.0)
  - 4 Medium severity (documentation improvements applied)
  - 3 Low severity (documentation improvements applied)

### Architectural Review
- ✅ **Technical Validation**: Architect approved all design decisions
- ✅ **Documentation Updated**: Known limitations documented in:
  - Macro API documentation (wavecraft-macros/src/lib.rs)
  - Architectural docs (coding-standards.md, high-level-design.md)

## Known Limitations (v0.9.0)

### Parameter Sync for DSL-Generated Plugins
DSL-generated plugins receive **default parameter values** in the `process()` method. Host automation and UI updates work correctly, but DSP code cannot read parameter values.

**Workaround**: Use manual `Plugin` trait implementation for parameter-driven effects.

**Status**: Documented and approved for v0.9.0. Full parameter sync targeted for future release.

### Breaking Changes
- **VST3 Class IDs**: Changed to use package name instead of vendor (acceptable pre-1.0)
- **Migration**: Documented in changelog and architectural docs

## Checklist

- ✅ Code follows project coding standards
- ✅ Tests added/updated (10 new functional tests)
- ✅ Documentation updated (feature spec, architectural docs, roadmap)
- ✅ No linting errors (`cargo xtask lint` passing)
- ✅ CI pipeline passing (107 tests)
- ✅ QA review complete (9 findings addressed)
- ✅ Architect review complete (technical validation)
- ✅ Manual testing complete (10/10 passing)
- ✅ Feature spec archived
- ✅ Roadmap updated (M16 complete, 89% progress)

## Merge Status

**READY FOR MERGE** ✅

All validation gates passed:
- Implementation complete (4 phases)
- Testing verified (automated + manual)
- QA approved (findings addressed)
- Architect approved (technical review)
- Product Owner approved (feature complete)
