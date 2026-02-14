## Summary

This PR removes cargo-derived plugin email behavior and aligns plugin metadata handling across macros, the template engine scaffold, and project documentation. It also updates architecture/agent docs for consistency with the metadata and workflow clarifications introduced in this branch.

## Changes

- **Engine/DSP**
  - Updated macro/plugin metadata handling in `engine/crates/wavecraft-macros/src/lib.rs` and `engine/crates/wavecraft-macros/src/plugin.rs`.
  - Updated generated template plugin metadata usage in `sdk-template/engine/src/lib.rs`.

- **UI**
  - No UI package or app code changes.

- **Build/Config**
  - No Cargo/package manager config changes.

- **Documentation**
  - Updated architecture docs and guides:
    - `docs/architecture/coding-standards-rust.md`
    - `docs/architecture/declarative-plugin-dsl.md`
    - `docs/guides/sdk-getting-started.md`
  - Updated planning/tracking docs:
    - `docs/backlog.md`
    - `docs/roadmap.md`
  - Updated agent configuration docs:
    - `.github/agents/architect.agent.md`
    - `.github/agents/orchestrator.agent.md`

## Commits

- `e796e29` fix(docs): formalize Pre-M19 initiative for CLI update UX improvements and optional dev build profile spike
- `8d8f01d` fix(docs): update roadmap and DSL documentation for accuracy and clarity
- `8977906` fix(docs): update plugin metadata notes to clarify email handling and derived properties

## Related Documentation

- Feature folder created for this PR summary:
  - [PR Summary](./PR-summary.md)

## Testing

- [ ] Build passes: `cargo xtask build`
- [ ] Linting passes: `cargo xtask lint`
- [ ] Tests pass: `cargo xtask test`
- [ ] Macro expansion and generated template validation verified

## Checklist

- [x] Code follows project coding standards
- [ ] Tests added/updated as needed
- [x] Documentation updated
- [ ] No linting errors (`cargo xtask lint`)
